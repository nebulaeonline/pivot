//! Minimal typed authoring client. Formula source is opaque to this module.
use super::{Result, render, wire_decimal};
use pivot_protocol::{authoring::v1 as a, foundation, v1 as p, validation};
use pivot_transport as t;
use prost::Message;
type Op = a::authoring_request::Operation;
type Out = a::authoring_response::Outcome;
const WARNING: &str = "Working models are currently memory-only and are lost when pivotd restarts.";
pub fn help() {
    println!(
        "Authoring: working list | working create NAME | working use NAME_OR_ID | working show | working refresh | working recover | working retry | working abandon"
    );
    println!(
        "Edits: input NAME = DECIMAL | formula NAME = SOURCE | port input LABEL NODE required | port input LABEL NODE optional DECIMAL | port output LABEL NODE"
    );
    println!("Calculation: validate | eval NODE[,NODE...] [INPUT=DECIMAL ...] | publish");
}
pub fn recognizes(line: &str) -> bool {
    matches!(
        line.split_whitespace().next(),
        Some("working" | "input" | "formula" | "port" | "validate" | "eval" | "publish")
    )
}
struct Client {
    stream: t::Stream,
    maximum: usize,
    healthy: bool,
    instance: String,
}
fn request(operation: Op) -> p::Request {
    p::Request {
        request_id: uuid::Uuid::new_v4().to_string(),
        version: Some(p::Version { major: 1, minor: 1 }),
        operation: Some(p::request::Operation::Authoring(a::AuthoringRequest {
            operation: Some(operation),
        })),
    }
}
impl Client {
    async fn open(endpoint: &str) -> Result<Self> {
        let mut stream = t::connect(endpoint).await?;
        let hello = p::Request {
            request_id: "authoring-hello".into(),
            version: Some(p::Version { major: 1, minor: 1 }),
            operation: Some(p::request::Operation::Handshake(p::Handshake {
                supported_versions: vec![p::Version { major: 1, minor: 1 }],
                required_capabilities: vec!["working-models-v1".into(), "decimal96".into()],
                optional_capabilities: vec![],
            })),
        };
        t::write_frame(
            &mut stream,
            &hello.encode_to_vec(),
            validation::MAX_MESSAGE_BYTES,
        )
        .await?;
        let bytes = read(&mut stream, validation::MAX_MESSAGE_BYTES).await?;
        let response = p::Response::decode(bytes.as_slice())?;
        if response.request_id != hello.request_id {
            return Err("invalid authoring handshake correlation".into());
        }
        let Some(p::response::Outcome::Handshake(accepted)) = response.outcome else {
            return Err(
                "authoring unavailable: daemon must enable --authoring (v1.1 working-models-v1)"
                    .into(),
            );
        };
        if accepted.selected_version != hello.version
            || !accepted
                .capabilities
                .iter()
                .any(|c| c == "working-models-v1")
            || !(512..=validation::MAX_MESSAGE_BYTES)
                .contains(&(accepted.max_message_bytes as usize))
        {
            return Err("invalid authoring negotiation".into());
        }
        let mut client = Self {
            stream,
            maximum: accepted.max_message_bytes as usize,
            healthy: true,
            instance: String::new(),
        };
        let Out::ServiceInfo(info) = client
            .call(Op::GetServiceInfo(a::GetServiceInfo {}))
            .await?
        else {
            return Err("unexpected service info".into());
        };
        if info.durability != a::Durability::MemoryOnly as i32 {
            return Err("unsupported durability contract".into());
        }
        client.instance = info.service_instance_id;
        Ok(client)
    }
    fn validate(&self, operation: &Op) -> Result<()> {
        foundation::decode_envelope(
            &request(operation.clone()).encode_to_vec(),
            foundation::Limits {
                bytes: self.maximum,
                ..Default::default()
            },
        )?;
        Ok(())
    }
    async fn call(&mut self, operation: Op) -> Result<Out> {
        if !self.healthy {
            return Err("authoring connection lost; use reconnect; no automatic retry".into());
        }
        self.validate(&operation)?;
        let request = request(operation);
        let received: Result<Out> = async {
            t::write_frame(&mut self.stream, &request.encode_to_vec(), self.maximum).await?;
            let bytes = read(&mut self.stream, self.maximum).await?;
            let response = foundation::decode_response(
                &bytes,
                &request,
                foundation::Limits {
                    bytes: self.maximum,
                    ..Default::default()
                },
            )?;
            let Some(p::response::Outcome::Authoring(v)) = response.outcome else {
                return Err("unexpected authoring response".into());
            };
            Ok(v.outcome.ok_or("missing outcome")?)
        }
        .await;
        if received.is_err() {
            self.healthy = false;
        }
        received
    }
}
async fn read(stream: &mut t::Stream, maximum: usize) -> Result<Vec<u8>> {
    Ok(t::read_frame_with_timeouts(
        stream,
        maximum,
        t::FrameTimeouts {
            idle: std::time::Duration::from_secs(10),
            ..Default::default()
        },
    )
    .await?
    .ok_or("server disconnected")?)
}
#[derive(Default)]
pub struct Working {
    client: Option<Client>,
    selected: Option<a::WorkingRevision>,
    attachment: Option<String>,
    pending: Option<(String, Op)>,
}
impl Working {
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn disconnect(&mut self) {
        self.client = None;
        self.selected = None;
        self.attachment = None;
    }
    async fn call(&mut self, op: Op) -> Result<Out> {
        self.client.as_mut().ok_or("not connected")?.call(op).await
    }
    fn revision(&self) -> Result<a::WorkingRevision> {
        self.selected
            .clone()
            .ok_or_else(|| "select a working model with working use".into())
    }
    fn instance(&self) -> String {
        self.client.as_ref().unwrap().instance.clone()
    }
    async fn list(&mut self) -> Result<Vec<a::WorkingSummary>> {
        let mut models = vec![];
        let mut token = String::new();
        let mut seen = std::collections::BTreeSet::new();
        loop {
            let out = self
                .call(Op::ListWorking(a::ListWorking {
                    service_instance_id: self.instance(),
                    page: Some(a::Page {
                        page_size: 64,
                        continuation_token: token,
                    }),
                }))
                .await?;
            let Out::WorkingModels(page) = checked(out)? else {
                return Err("unexpected list".into());
            };
            models.extend(page.models);
            if page.continuation_token.is_empty() {
                return Ok(models);
            }
            token = page.continuation_token;
            if !seen.insert(token.clone()) || seen.len() > 1024 {
                return Err("invalid/too many pages".into());
            }
        }
    }
    async fn draft(&mut self) -> Result<(Vec<a::RawNode>, Vec<a::RawPort>)> {
        let revision = self.revision()?;
        let mut nodes = vec![];
        let mut ports = vec![];
        for section in [a::DraftSection::Nodes, a::DraftSection::Ports] {
            let mut token = String::new();
            let mut seen = std::collections::BTreeSet::new();
            loop {
                let out = self
                    .call(Op::ReadDraft(a::ReadDraft {
                        revision: Some(revision.clone()),
                        section: section as i32,
                        page: Some(a::Page {
                            page_size: 64,
                            continuation_token: token,
                        }),
                    }))
                    .await?;
                let Out::Draft(page) = checked(out)? else {
                    return Err("unexpected draft".into());
                };
                nodes.extend(page.nodes);
                ports.extend(page.ports);
                if page.continuation_token.is_empty() {
                    break;
                }
                token = page.continuation_token;
                if !seen.insert(token.clone()) || seen.len() > 1024 {
                    return Err("invalid/too many pages".into());
                }
            }
        }
        Ok((nodes, ports))
    }
    async fn select(&mut self, identity: &str) -> Result<()> {
        let models: Vec<_> = self
            .list()
            .await?
            .into_iter()
            .filter(|m| m.working_model_id == identity || m.name == identity)
            .collect();
        if models.len() != 1 {
            return Err("working model must resolve uniquely; use its portable ID".into());
        }
        let model = &models[0];
        // Release the previous attachment before opening another to respect quotas.
        if let Some(id) = self.attachment.take() {
            let out = self
                .call(Op::CloseAttachment(a::CloseAttachment {
                    service_instance_id: self.instance(),
                    attachment_id: id.clone(),
                }))
                .await?;
            if let Err(error) = checked(out) {
                self.attachment = Some(id);
                return Err(error);
            }
        }
        self.selected = None;
        let out = self
            .call(Op::OpenWorking(a::OpenWorking {
                service_instance_id: self.instance(),
                working_model_id: model.working_model_id.clone(),
            }))
            .await?;
        let Out::Opened(opened) = checked(out)? else {
            return Err("unexpected open".into());
        };
        self.selected = opened.revision;
        self.attachment = Some(opened.attachment_id);
        println!("{WARNING}");
        self.status();
        Ok(())
    }
    fn status(&self) {
        if let Some(r) = &self.selected {
            println!(
                "Working {} revision={} durability=MemoryOnly",
                r.working_model_id, r.sequence
            );
        }
    }
    fn accept(&mut self, out: Out) -> Result<()> {
        match checked(out)? {
            Out::Created(v) => {
                self.selected = v.revision;
                println!("Created {}", v.working_model_id);
                println!("{WARNING}");
                self.status();
            }
            Out::Edited(v) => {
                self.selected = v.revision;
                self.status();
            }
            Out::Published(v) => {
                let r = v.published.ok_or("missing published revision")?;
                println!("Published {} revision={}", r.model_id, r.revision_id);
            }
            _ => return Err("unexpected mutation result".into()),
        }
        Ok(())
    }
    async fn mutate(&mut self, op: Op, id: String) -> Result<()> {
        if self.pending.is_some() {
            return Err(
                "unresolved mutation: working recover, working retry or working abandon first"
                    .into(),
            );
        }
        self.client.as_ref().unwrap().validate(&op)?;
        self.pending = Some((id.clone(), op.clone()));
        println!("Operation {id}");
        match self.call(op).await {
            Ok(out) => self.finish(out),
            Err(e) => Err(format!("mutation outcome unknown: {e}; operation={id}; reconnect then working recover; no automatic replay").into()),
        }
    }
    fn finish(&mut self, out: Out) -> Result<()> {
        // Permission can be revoked after commitment; retain its recovery context.
        if let Out::Error(e) = &out
            && [
                a::AuthoringErrorCode::Unauthorized as i32,
                a::AuthoringErrorCode::Internal as i32,
            ]
            .contains(&e.code)
        {
            return Err(format!(
                "{}; outcome may require reconciliation; pending operation retained",
                failure(e.clone())
            )
            .into());
        }
        self.pending = None;
        self.accept(out)
    }
    async fn edit(&mut self, edit: a::edit::Kind) -> Result<()> {
        let r = self.revision()?;
        let id = uuid::Uuid::new_v4().to_string();
        self.mutate(
            Op::ApplyEdits(a::ApplyEdits {
                mutation: Some(context(&r, &id)),
                edits: vec![a::Edit { kind: Some(edit) }],
            }),
            id,
        )
        .await
    }
    pub async fn execute(&mut self, endpoint: &str, line: &str) -> Result<()> {
        let words: Vec<_> = line.split_whitespace().collect();
        if words == ["working", "abandon"] {
            self.pending = None;
            println!("Forgot pending mutation; no server operation was undone.");
            return Ok(());
        }
        if self.pending.is_some()
            && (matches!(
                words.first(),
                Some(&"input" | &"formula" | &"port" | &"publish")
            ) || words.get(..2) == Some(&["working", "create"]))
        {
            return Err("unresolved mutation: working recover, working retry or working abandon first; no new operation was sent".into());
        }
        if self.client.as_ref().is_some_and(|c| !c.healthy) {
            return Err("authoring connection lost; use reconnect; no command or new mutation operation was sent".into());
        }
        if self.client.is_none() {
            self.client = Some(Client::open(endpoint).await?);
        }
        match words.as_slice() {
            ["working", "list"] => {
                for m in self.list().await? {
                    println!(
                        "{}\t{}\trevision={}",
                        m.name,
                        m.working_model_id,
                        m.revision.ok_or("missing revision")?.sequence
                    );
                }
            }
            ["working", "create", name] => {
                let id = uuid::Uuid::new_v4().to_string();
                self.mutate(
                    Op::CreateWorking(a::CreateWorking {
                        service_instance_id: self.instance(),
                        operation_id: id.clone(),
                        name: (*name).into(),
                    }),
                    id,
                )
                .await?;
            }
            ["working", "use", name] => self.select(name).await?,
            ["working", "refresh"] => {
                let id = self.revision()?.working_model_id;
                self.select(&id).await?;
            }
            ["working", "show"] => {
                self.status();
                let (nodes, ports) = self.draft().await?;
                for n in nodes {
                    let kind = n
                        .definition
                        .and_then(|d| d.kind)
                        .ok_or("missing definition")?;
                    match kind {
                        a::node_definition::Kind::Input(v) => {
                            println!("input {} = {} [{}]", n.name, render(&v)?, n.node_id)
                        }
                        a::node_definition::Kind::FormulaSource(s) => {
                            println!("formula {} = {} [{}]", n.name, s, n.node_id)
                        }
                    }
                }
                for p in ports {
                    match p.kind.ok_or("missing port kind")? {
                        a::raw_port::Kind::Input(v) => {
                            let requirement = match v
                                .requirement
                                .and_then(|r| r.kind)
                                .ok_or("missing port requirement")?
                            {
                                a::requirement::Kind::Required(_) => "required".into(),
                                a::requirement::Kind::OptionalDefault(value) => {
                                    format!("optional default={}", render(&value)?)
                                }
                            };
                            println!(
                                "port {} input -> {} {} [{}]",
                                p.name, v.node_id, requirement, p.port_id
                            );
                        }
                        a::raw_port::Kind::Output(v) => {
                            println!("port {} output -> {} [{}]", p.name, v.node_id, p.port_id)
                        }
                    }
                }
                self.diagnostics().await?;
            }
            ["working", "recover"] => {
                let (id, op) = self.pending.clone().ok_or("no pending mutation")?;
                let instance = operation_instance(&op);
                if instance != self.instance() {
                    return Err("service instance changed; old receipt is unavailable; explicitly working abandon".into());
                }
                let out = self
                    .call(Op::GetOperationOutcome(a::GetOperationOutcome {
                        service_instance_id: instance,
                        operation_id: id.clone(),
                    }))
                    .await?;
                let Out::OperationOutcome(v) = checked(out)? else {
                    return Err("unexpected receipt".into());
                };
                match v.outcome.ok_or("missing receipt outcome")? {
                    a::operation_outcome::Outcome::Unknown(_) => println!(
                        "Operation {id} is Unknown; retained intent; working retry explicitly resends the SAME operation ID."
                    ),
                    a::operation_outcome::Outcome::Created(v) => {
                        self.pending = None;
                        self.accept(Out::Created(v))?;
                    }
                    a::operation_outcome::Outcome::Edited(v) => {
                        self.pending = None;
                        self.accept(Out::Edited(v))?;
                    }
                    a::operation_outcome::Outcome::Published(v) => {
                        self.pending = None;
                        self.accept(Out::Published(v))?;
                    }
                    a::operation_outcome::Outcome::Rejected(e) => {
                        self.pending = None;
                        return Err(failure(e));
                    }
                }
            }
            ["working", "retry"] => {
                let (id, op) = self.pending.clone().ok_or("no pending mutation")?;
                if operation_instance(&op) != self.instance() {
                    return Err("service instance changed; explicitly working abandon".into());
                }
                println!("Retrying SAME operation {id}");
                let out = self.call(op).await?;
                self.finish(out)?;
            }
            ["validate"] => {
                let out = self
                    .call(Op::ValidateWorking(a::ValidateWorking {
                        revision: Some(self.revision()?),
                    }))
                    .await?;
                let Out::Validation(v) = checked(out)? else {
                    return Err("unexpected validation".into());
                };
                println!(
                    "Valid={} diagnostics={}",
                    v.valid,
                    v.diagnostics.map_or(0, |d| d.total)
                );
            }
            ["publish"] => {
                let r = self.revision()?;
                let id = uuid::Uuid::new_v4().to_string();
                self.mutate(
                    Op::PublishWorking(a::PublishWorking {
                        mutation: Some(context(&r, &id)),
                    }),
                    id,
                )
                .await?;
            }
            ["input" | "formula", ..] => {
                let (left, source) = line
                    .split_once('=')
                    .ok_or("use input NAME = DECIMAL or formula NAME = SOURCE")?;
                let labels: Vec<_> = left.split_whitespace().collect();
                if labels.len() != 2 {
                    return Err("expected one node name".into());
                }
                let name = labels[1];
                let kind = if labels[0] == "input" {
                    a::node_definition::Kind::Input(wire_decimal(source.trim())?)
                } else {
                    a::node_definition::Kind::FormulaSource(
                        source.strip_prefix(' ').unwrap_or(source).into(),
                    )
                };
                let (nodes, _) = self.draft().await?;
                let definition = Some(a::NodeDefinition { kind: Some(kind) });
                let edit = if let Some(n) = nodes.iter().find(|n| n.name == name) {
                    a::edit::Kind::SetNode(a::SetNode {
                        node: Some(node_ref(&n.node_id)),
                        definition,
                    })
                } else {
                    a::edit::Kind::AddNode(a::AddNode {
                        temporary_id: "node".into(),
                        name: name.into(),
                        definition,
                    })
                };
                self.edit(edit).await?;
            }
            ["port", direction, label, node, rest @ ..] => {
                let (nodes, ports) = self.draft().await?;
                if ports.iter().any(|p| p.name == *label) {
                    return Err("port already exists; minimal REPL only adds public ports".into());
                }
                let node = Some(node_ref(&find_node(&nodes, node)?.node_id));
                let edit=match (*direction,rest) {
                    ("output",[]) => a::edit::Kind::AddOutputPort(a::AddOutputPort { temporary_id:"port".into(),name:(*label).into(),node }),
                    ("input",["required"]) => a::edit::Kind::AddInputPort(a::AddInputPort { temporary_id:"port".into(),name:(*label).into(),node,
                        requirement:Some(a::Requirement { kind:Some(a::requirement::Kind::Required(a::Required {})) }) }),
                    ("input",["optional",value]) => a::edit::Kind::AddInputPort(a::AddInputPort { temporary_id:"port".into(),name:(*label).into(),node,
                        requirement:Some(a::Requirement { kind:Some(a::requirement::Kind::OptionalDefault(wire_decimal(value)?)) }) }),
                    _ => return Err("use port input LABEL NODE required|optional DECIMAL; port output LABEL NODE".into())
                };
                self.edit(edit).await?;
            }
            ["eval", outputs, bindings @ ..] => {
                let (nodes, _) = self.draft().await?;
                let outputs = outputs
                    .split(',')
                    .map(|name| Ok(find_node(&nodes, name)?.node_id.clone()))
                    .collect::<Result<Vec<_>>>()?;
                let inputs = bindings
                    .iter()
                    .map(|b| {
                        let (name, value) =
                            b.split_once('=').ok_or("override must be NODE=DECIMAL")?;
                        Ok(a::Override {
                            node_id: find_node(&nodes, name)?.node_id.clone(),
                            value: Some(wire_decimal(value)?),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let out = self
                    .call(Op::EvaluateWorking(a::EvaluateWorking {
                        revision: Some(self.revision()?),
                        inputs,
                        output_node_ids: outputs,
                    }))
                    .await?;
                let Out::Evaluation(v) = checked(out)? else {
                    return Err("unexpected evaluation".into());
                };
                for result in v.outputs {
                    let name = &find_node(&nodes, &result.node_id)?.name;
                    match result.outcome.ok_or("missing node result")? {
                        a::node_result::Outcome::Value(value) => {
                            println!("{name}={}", render(&value)?)
                        }
                        a::node_result::Outcome::Error(e) => println!(
                            "{name}=ERROR {:?}",
                            p::CalculationErrorCode::try_from(e.code).unwrap_or_default()
                        ),
                    }
                }
                self.status();
            }
            _ => return Err("invalid authoring command; use help".into()),
        }
        Ok(())
    }
    async fn diagnostics(&mut self) -> Result<()> {
        let mut token = String::new();
        let mut seen = std::collections::BTreeSet::new();
        loop {
            let out = self
                .call(Op::ReadDiagnostics(a::ReadDiagnostics {
                    service_instance_id: self.instance(),
                    working_revision: Some(self.revision()?),
                    page: Some(a::Page {
                        page_size: 64,
                        continuation_token: token,
                    }),
                }))
                .await?;
            let Out::Diagnostics(page) = checked(out)? else {
                return Err("unexpected diagnostics".into());
            };
            for d in page.diagnostics {
                println!(
                    "Diagnostic {:?}: {} {}",
                    a::DiagnosticCode::try_from(d.code).unwrap_or_default(),
                    d.message,
                    d.unresolved_name
                );
            }
            if page.continuation_token.is_empty() {
                break;
            }
            token = page.continuation_token;
            if !seen.insert(token.clone()) || seen.len() > 1024 {
                return Err("invalid/too many diagnostic pages".into());
            }
        }
        Ok(())
    }
}
fn find_node<'n>(nodes: &'n [a::RawNode], name: &str) -> Result<&'n a::RawNode> {
    nodes
        .iter()
        .find(|n| n.name == name || n.node_id == name)
        .ok_or_else(|| format!("unknown authoring node: {name}").into())
}
fn node_ref(id: &str) -> a::NodeReference {
    a::NodeReference {
        kind: Some(a::node_reference::Kind::NodeId(id.into())),
    }
}
fn context(r: &a::WorkingRevision, id: &str) -> a::MutationContext {
    a::MutationContext {
        service_instance_id: r.service_instance_id.clone(),
        working_model_id: r.working_model_id.clone(),
        expected_revision: Some(r.clone()),
        operation_id: id.into(),
    }
}
fn operation_instance(op: &Op) -> String {
    match op {
        Op::CreateWorking(v) => v.service_instance_id.clone(),
        Op::ApplyEdits(v) => v.mutation.as_ref().unwrap().service_instance_id.clone(),
        Op::PublishWorking(v) => v.mutation.as_ref().unwrap().service_instance_id.clone(),
        _ => unreachable!(),
    }
}
fn failure(e: a::AuthoringFailure) -> Box<dyn std::error::Error> {
    format!(
        "{:?}: {}; revision unchanged locally; working refresh explicitly adopts the current head",
        a::AuthoringErrorCode::try_from(e.code).unwrap_or_default(),
        e.message
    )
    .into()
}
fn checked(out: Out) -> Result<Out> {
    match out {
        Out::Error(e) => Err(failure(e)),
        other => Ok(other),
    }
}
