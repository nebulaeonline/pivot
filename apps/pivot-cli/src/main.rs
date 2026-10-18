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
    healthy: bool,
}
impl Client {
    async fn open(endpoint: &str) -> Result<Self> {
        let mut client = Self {
            stream: transport::connect(endpoint).await?,
            next: 0,
            maximum: validation::MAX_MESSAGE_BYTES,
            healthy: true,
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
        if !self.healthy {
            return Err("disconnected; use reconnect (no invocation was retried)".into());
        }
        let received: Result<Response> = async {
            transport::write_frame(&mut self.stream, &request.encode_to_vec(), self.maximum)
                .await?;
            let policy = transport::FrameTimeouts {
                idle: std::time::Duration::from_secs(10),
                ..Default::default()
            };
            let bytes = transport::read_frame_with_timeouts(&mut self.stream, self.maximum, policy)
                .await?
                .ok_or("server disconnected")?;
            Ok(validation::decode_response(&bytes, &request)?)
        }
        .await;
        let response = match received {
            Ok(response) => response,
            Err(error) => {
                self.healthy = false;
                return Err(format!("connection lost or invalid response: {error}; invocation outcome may be unknown; use reconnect; no automatic retry").into());
            }
        };
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
#[derive(Default)]
struct CommandArgs {
    endpoint: Option<String>,
    revision: Option<String>,
    output_labels: Option<Vec<String>>,
    positional: Vec<String>,
}
fn parse_command(tokens: Vec<String>) -> Result<CommandArgs> {
    let mut parsed = CommandArgs::default();
    let mut args = tokens.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--endpoint" => parsed.endpoint = Some(args.next().ok_or("missing endpoint")?),
            "--revision" => parsed.revision = Some(args.next().ok_or("missing revision")?),
            "--outputs" => {
                parsed.output_labels = Some(
                    args.next()
                        .ok_or("missing outputs")?
                        .split(',')
                        .map(str::to_owned)
                        .collect(),
                )
            }
            "--help" => parsed.positional.push("help".into()),
            _ if arg.starts_with("--") => return Err(format!("unknown option: {arg}").into()),
            _ => parsed.positional.push(arg),
        }
    }
    Ok(parsed)
}
fn help() {
    println!(
        "Commands: help | models | describe MODEL [--revision ID] | invoke MODEL Input=Decimal ... [--revision ID] [--outputs Label,Label] | reconnect [--endpoint LOCAL_ENDPOINT] | exit"
    );
    println!(
        "Start interactive mode: pivot-cli [--endpoint LOCAL_ENDPOINT] repl (or omit the command). History is session-local; no input values are saved to disk."
    );
}
async fn execute(client: &mut Client, args: CommandArgs) -> Result<bool> {
    let CommandArgs {
        positional,
        revision,
        output_labels,
        ..
    } = args;
    let command = positional.first().ok_or("expected command; use help")?;
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
            if let Err(error) = validation::validate_result(&result, &invocation, &d) {
                client.healthy = false;
                return Err(error.into());
            }
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
async fn repl(endpoint: String) -> Result<bool> {
    use rustyline::error::ReadlineError;
    use std::io::{IsTerminal, Write};
    let terminal = std::io::stdin().is_terminal();
    let mut editor = if terminal {
        Some(rustyline::DefaultEditor::new()?)
    } else {
        None
    };
    let mut endpoint = endpoint;
    let mut client = match Client::open(&endpoint).await {
        Ok(client) => {
            println!("Connected. Type help for commands.");
            Some(client)
        }
        Err(error) => {
            eprintln!("Connection failed: {error}; use reconnect.");
            None
        }
    };
    std::io::stdout().flush()?;
    loop {
        let line = if let Some(editor) = &mut editor {
            match editor.readline("pivot> ") {
                Ok(line) => {
                    editor.add_history_entry(line.as_str())?;
                    line
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted. Type exit to quit.");
                    continue;
                }
                Err(ReadlineError::Eof) => break,
                Err(error) => return Err(error.into()),
            }
        } else {
            // Pipe mode shares command parsing/execution, but deliberately has
            // no terminal editing. Flush results for supervisors and tests.
            let mut line = String::new();
            if std::io::stdin().read_line(&mut line)? == 0 {
                break;
            }
            line
        };
        if line.trim().is_empty() {
            continue;
        }
        // Commands are whitespace-delimited labels/decimal bindings. No shell,
        // quotes, script execution, or computational expression language.
        let parsed = match parse_command(line.split_whitespace().map(str::to_owned).collect()) {
            Ok(args) => args,
            Err(error) => {
                eprintln!("Error: {error}");
                continue;
            }
        };
        match parsed.positional.first().map(String::as_str) {
            Some("exit")
                if parsed.positional.len() == 1
                    && parsed.endpoint.is_none()
                    && parsed.revision.is_none()
                    && parsed.output_labels.is_none() =>
            {
                break;
            }
            Some("help") if parsed.positional.len() == 1 => help(),
            Some("reconnect")
                if parsed.positional.len() == 1
                    && parsed.revision.is_none()
                    && parsed.output_labels.is_none() =>
            {
                // Explicit reconnect always discards the previous connection.
                client = None;
                if let Some(selected) = parsed.endpoint {
                    endpoint = selected;
                }
                match Client::open(&endpoint).await {
                    Ok(connected) => {
                        client = Some(connected);
                        println!("Reconnected.");
                    }
                    Err(error) => eprintln!("Connection failed: {error}; use reconnect."),
                }
            }
            _ if parsed.endpoint.is_some() => {
                eprintln!("Error: use reconnect --endpoint to change endpoint")
            }
            Some("models" | "describe" | "invoke") => {
                if let Some(connected) = &mut client {
                    if let Err(error) = execute(connected, parsed).await {
                        eprintln!("Error: {error}");
                    }
                    if !connected.healthy {
                        client = None;
                    }
                } else {
                    eprintln!("Disconnected; use reconnect. No command was sent.");
                }
            }
            _ => eprintln!("Error: unknown or invalid command; use help"),
        }
        std::io::stdout().flush()?;
        std::io::stderr().flush()?;
    }
    println!("Goodbye.");
    Ok(true)
}
async fn run() -> Result<bool> {
    let mut args = parse_command(std::env::args().skip(1).collect())?;
    if args.positional.is_empty() {
        args.positional.push("repl".into());
    }
    let endpoint = match args.endpoint.take() {
        Some(endpoint) => endpoint,
        None => transport::default_endpoint()?,
    };
    if args.positional == ["help"] {
        help();
        return Ok(true);
    }
    if args.positional == ["repl"] {
        if args.revision.is_some() || args.output_labels.is_some() {
            return Err("specify revision and outputs per REPL command".into());
        }
        return repl(endpoint).await;
    }
    let mut client = Client::open(&endpoint).await?;
    execute(&mut client, args).await
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
