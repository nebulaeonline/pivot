use pivot_host::management::{
    self as m,
    adapter::{AdapterLimits, AuthoringAdapter},
    connection::Connection,
};
use pivot_protocol::{authoring::v1 as a, foundation, v1 as p, validation::MAX_MESSAGE_BYTES};
use pivot_transport::{self as t, tokio};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
struct Policy(Arc<AtomicBool>);
impl m::Authorizer for Policy {
    fn allowed(&self, _: &str, action: m::Action, _: Option<m::WorkingModelId>) -> bool {
        action != m::Action::Edit || !self.0.load(Ordering::SeqCst)
    }
}
struct Server {
    endpoint: String,
    task: tokio::task::JoinHandle<()>,
    deny: Arc<AtomicBool>,
    lose: Arc<AtomicBool>,
    submitted: Arc<Mutex<Vec<a::ApplyEdits>>>,
}
impl Server {
    fn start(enabled: bool) -> Self {
        Self::limited(enabled, AdapterLimits::default())
    }
    fn limited(enabled: bool, limits: AdapterLimits) -> Self {
        let id = uuid::Uuid::new_v4();
        #[cfg(windows)]
        let endpoint = format!(r"\\.\pipe\pivot-repl-{id}");
        #[cfg(unix)]
        let endpoint = format!("/tmp/pivot-repl-{id}/service.sock");
        let mut listener = t::Listener::bind(&endpoint).unwrap();
        let deny = Arc::new(AtomicBool::new(false));
        let lose = Arc::new(AtomicBool::new(false));
        let submitted = Arc::new(Mutex::new(vec![]));
        let service = Arc::new(Mutex::new(
            m::WorkingModelService::new(m::Limits::default(), Box::new(Policy(deny.clone())))
                .unwrap(),
        ));
        let adapter = enabled.then(|| AuthoringAdapter::new(service.clone(), limits).unwrap());
        let drop_next = lose.clone();
        let edits = submitted.clone();
        let task = tokio::spawn(async move {
            let mut sessions = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted=listener.accept()=>{
                        let mut stream=accepted.unwrap();
                        let principal=t::verified_peer_principal(&stream).unwrap();
                        let mut session=Connection::new(service.clone(),adapter.clone(),principal,Arc::new(pivot_host::AllowAll)).unwrap();
                        let drop_next=drop_next.clone();let edits=edits.clone();
                        sessions.spawn(async move {
                        // Reuse the production router and service. This fault
                        // seam records edit IDs and can discard a committed reply.
                            while let Some(bytes)=t::read_frame(&mut stream,MAX_MESSAGE_BYTES).await.unwrap_or_default() {
                                let edit=foundation::decode_session_request(&bytes,Default::default()).ok().and_then(|r|match r.operation {
                                    Some(p::request::Operation::Authoring(v))=>match v.operation { Some(a::authoring_request::Operation::ApplyEdits(v))=>Some(v),_=>None },_=>None });
                                if let Some(edit)=&edit { edits.lock().unwrap().push(edit.clone()); }
                                let (returned,response)=tokio::task::spawn_blocking(move||{let response=session.dispatch(&bytes);(session,response)}).await.unwrap();
                                session=returned;
                                if edit.is_some() && drop_next.swap(false,Ordering::SeqCst) { return; }
                                if t::write_frame(&mut stream,&response,MAX_MESSAGE_BYTES).await.is_err() { return; }
                            }
                        });
                    },
                    _=sessions.join_next(),if !sessions.is_empty()=>{}
                }
            }
        });
        Self {
            endpoint,
            task,
            deny,
            lose,
            submitted,
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
struct Repl {
    child: Child,
    lines: std::sync::mpsc::Receiver<String>,
    log: String,
}
impl Repl {
    fn start(endpoint: &str) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_pivot-cli"))
            .args(["--endpoint", endpoint, "repl"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let (tx, lines) = std::sync::mpsc::channel();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let output = tx.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = output.send(line);
            }
        });
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let _ = tx.send(line);
            }
        });
        let mut repl = Self {
            child,
            lines,
            log: String::new(),
        };
        repl.wait("Connected.");
        repl
    }
    fn send(&mut self, line: &str) {
        writeln!(self.child.stdin.as_mut().unwrap(), "{line}").unwrap();
    }
    fn wait(&mut self, needle: &str) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            let line = self
                .lines
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .unwrap_or_else(|_| panic!("missing {needle}: {}", self.log));
            self.log.push_str(&line);
            self.log.push('\n');
            if line.contains(needle) {
                return line;
            }
        }
    }
    fn command(&mut self, line: &str, needle: &str) -> String {
        self.send(line);
        self.wait(needle)
    }
}
impl Drop for Repl {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn complete_repl_authoring_workflow_and_immutable_publication() {
    let server = Server::start(true);
    let mut repl = Repl::start(&server.endpoint);
    repl.command("working create ProfitModel", "revision=0");
    repl.command("input Revenue = 250000", "revision=1");
    repl.command("input Costs = 85000", "revision=2");
    repl.command("formula Net = Revenue - Costs", "revision=3");
    repl.command("eval Net", "Net=165000");
    repl.command("eval Net Revenue=100000 Costs=25000", "Net=75000");
    repl.command("port input Sales Revenue required", "revision=4");
    repl.command("port input Expenses Costs optional 85000", "revision=5");
    repl.command("port output Profit Net", "revision=6");
    repl.command("validate", "Valid=true");
    repl.command("working show", "port Profit");
    let publication = repl.command("publish", "Published");
    let published_id = publication.split_whitespace().nth(1).unwrap();
    repl.command(
        &format!("working use {published_id}"),
        "working model must resolve uniquely",
    );
    repl.command("invoke ProfitModel Sales=250000", "Profit=165000");
    repl.command("input Costs = 90000", "revision=7");
    repl.command("formula Net = Revenue - Costs + 1", "revision=8");
    repl.command("eval Net", "Net=160001");
    repl.command("invoke ProfitModel Sales=250000", "Profit=165000");
    repl.command("reconnect", "Reconnected.");
    repl.command("working use ProfitModel", "revision=8");
    repl.command("formula Net = Revenue +", "revision=9");
    repl.command("validate", "Valid=false");
    repl.command("working show", "Diagnostic Parse");
    repl.command("formula Net = Revenue - Costs", "revision=10");
    repl.command("eval Net", "Net=160000");
    repl.command("working list", "ProfitModel");
    assert!(
        repl.log.contains(
            "Working models are currently memory-only and are lost when pivotd restarts."
        )
    );
    repl.command("exit", "Goodbye.");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn explicit_conflict_and_daemon_authorization_are_preserved() {
    let server = Server::start(true);
    let mut first = Repl::start(&server.endpoint);
    let mut second = Repl::start(&server.endpoint);
    first.command("working create Shared", "revision=0");
    first.command("input Value = 1", "revision=1");
    second.command("working use Shared", "revision=1");
    first.command("input Value = 2", "revision=2");
    second.command("input Value = 3", "RevisionConflict");
    second.command("working refresh", "revision=2");
    server.deny.store(true, Ordering::SeqCst);
    second.command("input Value = 99", "Unauthorized");
    server.deny.store(false, Ordering::SeqCst);
    second.command("working abandon", "Forgot pending mutation");
    second.command("eval Value", "Value=2");
    second.command("input Value = 4", "revision=3");
    first.command("working refresh", "revision=3");
    first.command("eval Value", "Value=4");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn lost_response_retains_operation_and_explicit_retry_uses_identical_intent() {
    let server = Server::start(true);
    let mut repl = Repl::start(&server.endpoint);
    repl.command("working create Replay", "revision=0");
    server.lose.store(true, Ordering::SeqCst);
    repl.command("input Value = 1", "mutation outcome unknown");
    let submitted = server.submitted.lock().unwrap().clone();
    assert_eq!(submitted.len(), 1);
    repl.command("input Other = 2", "unresolved mutation");
    assert_eq!(server.submitted.lock().unwrap().len(), 1);
    repl.command("reconnect", "Reconnected.");
    repl.command("working retry", "revision=1");
    let submitted = server.submitted.lock().unwrap().clone();
    assert_eq!(submitted.len(), 2);
    assert_eq!(submitted[0], submitted[1]);
    repl.command("working use Replay", "revision=1");
    server.lose.store(true, Ordering::SeqCst);
    repl.command("input Value = 3", "mutation outcome unknown");
    repl.command("reconnect", "Reconnected.");
    repl.command("working recover", "revision=2");
    assert_eq!(server.submitted.lock().unwrap().len(), 3);
    repl.command("working use Replay", "revision=2");
    repl.command("eval Value", "Value=3");
    server.lose.store(true, Ordering::SeqCst);
    repl.command("input Value = 4", "mutation outcome unknown");
    repl.command("working abandon", "Forgot pending mutation");
    repl.command("working create MustNotBeSent", "authoring connection lost");
    repl.command("reconnect", "Reconnected.");
    // A rejected local command on a known-dead connection did not manufacture
    // another uncertain operation that would block this explicit new mutation.
    repl.command("working create Fresh", "revision=0");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn invocation_only_daemon_rejects_authoring_without_breaking_repl() {
    let server = Server::start(false);
    let mut repl = Repl::start(&server.endpoint);
    repl.command("working create Denied", "authoring unavailable");
    repl.command("help", "Commands: help");
    repl.command("exit", "Goodbye.");
    assert!(server.submitted.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn negotiated_message_limits_reject_oversize_edits_without_pending_mutation() {
    let server = Server::limited(
        true,
        AdapterLimits {
            response_bytes: 512,
            page_target_bytes: 512,
            sessions: 1,
            ..Default::default()
        },
    );
    let mut repl = Repl::start(&server.endpoint);
    repl.command("working create Bounded", "revision=0");
    repl.command("input Value = 1", "revision=1");
    repl.command(
        &format!("formula TooLarge = {}1", " ".repeat(600)),
        "ResourceExhausted",
    );
    assert_eq!(server.submitted.lock().unwrap().len(), 1);
    repl.command("input Value = 2", "revision=2");
    repl.command("eval Value", "Value=2");
    let mut second = Repl::start(&server.endpoint);
    second.command("working list", "authoring unavailable");
}
