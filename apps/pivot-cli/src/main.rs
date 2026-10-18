use pivot_protocol::{v1::*, validation};
use pivot_transport::{self as transport, tokio};
use pivot_types::PivotDecimal;
use prost::Message;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
struct Client {
    stream: transport::Stream,
    next: u64,
    maximum: usize,
}
impl Client {
    async fn open(endpoint: &str) -> Result<Self> {
        let mut client = Self {
            stream: transport::connect(endpoint).await?,
            next: 0,
            maximum: validation::MAX_MESSAGE_BYTES,
        };
        let response = client
            .exchange(request::Operation::Handshake(Handshake {
                supported_versions: vec![Version { major: 1, minor: 0 }],
                required_capabilities: vec![
                    "decimal96".into(),
                    "model-discovery".into(),
                    "synchronous-invocation".into(),
                ],
                optional_capabilities: vec![],
            }))
            .await?;
        let response::Outcome::Handshake(hello) = response else {
            return Err("unexpected handshake response".into());
        };
        client.maximum = hello.max_message_bytes as usize;
        Ok(client)
    }
    async fn exchange(&mut self, operation: request::Operation) -> Result<response::Outcome> {
        self.next = self.next.checked_add(1).ok_or("request ID exhausted")?;
        let request = Request {
            request_id: format!("cli_{}", self.next),
            version: Some(Version { major: 1, minor: 0 }),
            operation: Some(operation),
        };
        validation::validate_request(&request)?;
        transport::write_frame(&mut self.stream, &request.encode_to_vec(), self.maximum).await?;
        let bytes = transport::read_frame(&mut self.stream, self.maximum)
            .await?
            .ok_or("server disconnected")?;
        let response = validation::decode_response(&bytes, &request)?;
        match response.outcome.ok_or("missing response")? {
            response::Outcome::Error(error) => Err(format!(
                "{:?}: {}",
                ErrorCode::try_from(error.code).unwrap_or(ErrorCode::Internal),
                error.message
            )
            .into()),
            result => Ok(result),
        }
    }
    async fn models(&mut self) -> Result<Vec<ModelSummary>> {
        let mut entries = Vec::new();
        let mut token = String::new();
        let mut seen = std::collections::BTreeSet::new();
        loop {
            let outcome = self
                .exchange(request::Operation::ListModels(ListModels {
                    page_size: 100,
                    continuation_token: token,
                }))
                .await?;
            let response::Outcome::Models(page) = outcome else {
                return Err("unexpected discovery response".into());
            };
            entries.extend(page.models);
            if page.continuation_token.is_empty() {
                break;
            }
            if !seen.insert(page.continuation_token.clone()) || seen.len() > 1000 {
                return Err("invalid/too many discovery pages".into());
            }
            token = page.continuation_token;
        }
        Ok(entries)
    }
    async fn describe(&mut self, name: &str, revision: Option<&str>) -> Result<ModelDescription> {
        let matches: Vec<_> = self
            .models()
            .await?
            .into_iter()
            .filter(|entry| {
                entry.model.as_ref().is_some_and(|r| {
                    (entry.name == name || r.model_id == name)
                        && revision.is_none_or(|v| v == r.revision_id)
                })
            })
            .collect();
        if matches.len() != 1 {
            return Err("model not found or ambiguous; select an exact --revision".into());
        }
        let outcome = self
            .exchange(request::Operation::DescribeModel(DescribeModel {
                model: matches[0].model.clone(),
            }))
            .await?;
        let response::Outcome::Description(description) = outcome else {
            return Err("unexpected description response".into());
        };
        Ok(description)
    }
}
fn render(value: &Value) -> Result<String> {
    validation::validate_value(value)?;
    let Some(value::Kind::Decimal(d)) = &value.kind else {
        return Err("unsupported value".into());
    };
    Ok(PivotDecimal::from_parts(d.lo, d.mid, d.hi, d.negative, d.scale)?.to_string())
}
fn wire_decimal(text: &str) -> Result<Value> {
    let d: PivotDecimal = text.parse()?;
    let (lo, mid, hi, negative, scale) = d.parts();
    Ok(Value {
        kind: Some(value::Kind::Decimal(Decimal128 {
            lo,
            mid,
            hi,
            negative,
            scale,
        })),
    })
}
async fn run() -> Result<bool> {
    let mut endpoint = transport::default_endpoint()?;
    let mut revision = None;
    let mut output_labels: Option<Vec<String>> = None;
    let mut positional = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--endpoint" => endpoint = args.next().ok_or("missing endpoint")?,
            "--revision" => revision = Some(args.next().ok_or("missing revision")?),
            "--outputs" => {
                output_labels = Some(
                    args.next()
                        .ok_or("missing outputs")?
                        .split(',')
                        .map(str::to_owned)
                        .collect(),
                )
            }
            "--help" => {
                println!(
                    "pivot-cli [--endpoint LOCAL_ENDPOINT] models\npivot-cli [OPTIONS] describe MODEL\npivot-cli [OPTIONS] invoke MODEL Input=Decimal ...\nOPTIONS: --revision ID --outputs Label,Label"
                );
                return Ok(true);
            }
            _ if arg.starts_with("--") => return Err(format!("unknown option: {arg}").into()),
            _ => positional.push(arg),
        }
    }
    let command = positional
        .first()
        .ok_or("expected models, describe, or invoke; use --help")?;
    let mut client = Client::open(&endpoint).await?;
    match command.as_str() {
        "models" if positional.len() == 1 => {
            for entry in client.models().await? {
                let r = entry.model.ok_or("missing model identity")?;
                println!("{}\t{}\t{}", entry.name, r.model_id, r.revision_id);
            }
        }
        "describe" if positional.len() == 2 => {
            let d = client.describe(&positional[1], revision.as_deref()).await?;
            let r = d.model.as_ref().ok_or("missing model identity")?;
            println!("{} [{} @ {}]", d.name, r.model_id, r.revision_id);
            for p in &d.inputs {
                let requirement = match &p.requirement {
                    Some(input_port::Requirement::Required(true)) => "required".to_owned(),
                    Some(input_port::Requirement::DefaultValue(v)) => {
                        format!("optional default={}", render(v)?)
                    }
                    _ => return Err("invalid input requirement".into()),
                };
                println!("input {} Decimal ({requirement})", p.name);
            }
            for p in &d.outputs {
                println!("output {} Decimal", p.name);
            }
        }
        "invoke" if positional.len() >= 2 => {
            let d = client.describe(&positional[1], revision.as_deref()).await?;
            let mut inputs = Vec::new();
            for binding in &positional[2..] {
                let (name, text) = binding
                    .split_once('=')
                    .ok_or("inputs must be Name=Decimal")?;
                let port = d
                    .inputs
                    .iter()
                    .find(|p| p.name == name)
                    .ok_or_else(|| format!("unknown input: {name}"))?;
                inputs.push(InputBinding {
                    port_id: port.port_id.clone(),
                    value: Some(wire_decimal(text)?),
                    source: 0,
                });
            }
            let outputs = match output_labels {
                None => d.outputs.iter().map(|p| p.port_id.clone()).collect(),
                Some(labels) => labels
                    .iter()
                    .map(|name| {
                        d.outputs
                            .iter()
                            .find(|p| p.name == *name)
                            .map(|p| p.port_id.clone())
                            .ok_or_else(|| format!("unknown output: {name}"))
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            };
            let invocation = InvokeModel {
                model: d.model.clone(),
                inputs,
                output_port_ids: outputs,
            };
            validation::validate_invocation(&invocation, &d)?;
            let outcome = client
                .exchange(request::Operation::InvokeModel(invocation.clone()))
                .await?;
            let response::Outcome::Invocation(result) = outcome else {
                return Err("unexpected invocation response".into());
            };
            validation::validate_result(&result, &invocation, &d)?;
            let mut success = true;
            for output in &result.outputs {
                let name = &d
                    .outputs
                    .iter()
                    .find(|p| p.port_id == output.port_id)
                    .ok_or("unknown output")?
                    .name;
                match &output.outcome {
                    Some(output_result::Outcome::Value(value)) => {
                        println!("{name}={}", render(value)?)
                    }
                    Some(output_result::Outcome::Error(error)) => {
                        println!(
                            "{name}=ERROR {:?}",
                            CalculationErrorCode::try_from(error.code).unwrap_or_default()
                        );
                        success = false;
                    }
                    _ => return Err("missing output result".into()),
                }
            }
            println!(
                "revision={} invocation={}",
                result.model.as_ref().ok_or("missing revision")?.revision_id,
                result.invocation_id
            );
            return Ok(success);
        }
        _ => return Err("invalid command arguments; use --help".into()),
    }
    Ok(true)
}
#[tokio::main]
async fn main() {
    match run().await {
        Ok(true) => (),
        Ok(false) => std::process::exit(2),
        Err(error) => {
            eprintln!("pivot-cli: {error}");
            std::process::exit(1);
        }
    }
}
