use pivot_core::{interface::InputRequirement, model::WorkingModel};
use pivot_host::{
    AllowAll,
    management::{
        self as management,
        adapter::{AdapterLimits, AuthoringAdapter},
        connection::Connection,
    },
};
use pivot_protocol::v1::ModelRef;
use pivot_transport::{self as transport, tokio};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

fn demo(host: &mut management::WorkingModelService) -> Result<(), Box<dyn Error>> {
    let mut model = WorkingModel::new("RestaurantProfit")?;
    let revenue = model.add_input("Revenue", "250000".parse()?)?;
    let costs = model.add_input("Costs", "85000".parse()?)?;
    let net = model.add_computed("Net", "Revenue - Costs")?;
    let margin = model.add_computed("Margin", "Net / Revenue")?;
    model.add_input_port("Revenue", revenue, InputRequirement::Required)?;
    model.add_input_port(
        "Costs",
        costs,
        InputRequirement::Optional {
            default: "85000".parse::<pivot_types::PivotDecimal>()?.into(),
        },
    )?;
    model.add_output_port("Net", net)?;
    model.add_output_port("Margin", margin)?;
    host.register_bootstrap(
        ModelRef {
            model_id: "restaurant-profit".into(),
            revision_id: "demo-v1".into(),
        },
        model.snapshot()?,
        BTreeMap::from([
            ("Revenue".into(), "revenue".into()),
            ("Costs".into(), "costs".into()),
            ("Net".into(), "net".into()),
            ("Margin".into(), "margin".into()),
        ]),
    )?;
    Ok(())
}
/// This daemon accepts only OS-verified current-user peers. Authoring rights are
/// explicitly enabled by its operator, independently of public invocation rights.
struct LocalAuthoringPolicy(bool);
impl management::Authorizer for LocalAuthoringPolicy {
    fn allowed(
        &self,
        _: &str,
        _: management::Action,
        _: Option<management::WorkingModelId>,
    ) -> bool {
        self.0
    }
}
async fn shutdown_signal() -> io::Result<()> {
    #[cfg(windows)]
    {
        let mut interrupt = tokio::signal::windows::ctrl_break()?;
        tokio::select! { result = tokio::signal::ctrl_c() => result, _ = interrupt.recv() => Ok(()) }
    }
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! { result = tokio::signal::ctrl_c() => result, _ = terminate.recv() => Ok(()) }
    }
}
fn seconds(value: Option<String>) -> Result<std::time::Duration, Box<dyn Error>> {
    let value: u64 = value.ok_or("missing timeout")?.parse()?;
    if !(1..=86400).contains(&value) {
        return Err("timeout must be 1..=86400 seconds".into());
    }
    Ok(std::time::Duration::from_secs(value))
}
async fn run() -> Result<(), Box<dyn Error>> {
    let mut endpoint = transport::default_endpoint()?;
    let mut demo_enabled = false;
    let mut authoring_enabled = false;
    let mut authoring_sessions = 16usize;
    let mut shutdown_stdin = false;
    let mut timeouts = transport::FrameTimeouts::default();
    let mut max_connections = 16usize;
    let mut work_budget = 100_000usize;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => demo_enabled = true,
            "--authoring" => authoring_enabled = true,
            "--authoring-sessions" => {
                authoring_sessions = args
                    .next()
                    .ok_or("missing authoring session limit")?
                    .parse()?;
                if !(1..=1024).contains(&authoring_sessions) {
                    return Err("authoring sessions must be 1..=1024".into());
                }
            }
            "--endpoint" => endpoint = args.next().ok_or("missing endpoint")?,
            "--shutdown-stdin" => shutdown_stdin = true,
            "--idle-timeout-secs" => timeouts.idle = seconds(args.next())?,
            "--frame-read-timeout-secs" => timeouts.partial_read = seconds(args.next())?,
            "--frame-write-timeout-secs" => timeouts.write = seconds(args.next())?,
            "--max-connections" => {
                max_connections = args.next().ok_or("missing connection limit")?.parse()?;
                if !(1..=1024).contains(&max_connections) {
                    return Err("connection limit must be 1..=1024".into());
                }
            }
            "--work-budget" => {
                work_budget = args.next().ok_or("missing work budget")?.parse()?;
                if work_budget == 0 {
                    return Err("work budget must be positive".into());
                }
            }
            "--help" => {
                println!(
                    "pivotd [--demo] [--authoring] [--authoring-sessions 16] [--endpoint LOCAL_ENDPOINT] [--shutdown-stdin]\nAuthoring is current-user and MemoryOnly: all created models are lost on restart.\nPolicy: --idle-timeout-secs 300 --frame-read-timeout-secs 5 --frame-write-timeout-secs 5 --max-connections 16 --work-budget 100000"
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    if !demo_enabled && !authoring_enabled {
        return Err("select --demo or --authoring; persistence is not implemented".into());
    }
    let mut service = management::WorkingModelService::with_all_limits(
        management::Limits::default(),
        management::EditLimits::default(),
        management::CalculationLimits {
            work: work_budget,
            ..Default::default()
        },
        Box::new(LocalAuthoringPolicy(authoring_enabled)),
    )?;
    if demo_enabled {
        demo(&mut service)?;
    }
    let service = Arc::new(Mutex::new(service));
    let adapter = if authoring_enabled {
        eprintln!(
            "Authoring enabled for the verified current OS user. Durability: MemoryOnly; all authoring state is lost on restart."
        );
        Some(AuthoringAdapter::new(
            service.clone(),
            AdapterLimits {
                sessions: authoring_sessions,
                ..Default::default()
            },
        )?)
    } else {
        None
    };
    let mut listener = transport::Listener::bind(&endpoint)?;
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel();
    let _stop_guard = if shutdown_stdin {
        std::thread::spawn(move || {
            let _ = io::stdin().read(&mut [0u8]);
            let _ = stop_tx.send(());
        });
        None
    } else {
        // Keep the channel open without a background reader in interactive use.
        Some(stop_tx)
    };
    println!("READY {endpoint}");
    io::stdout().flush()?;
    let mut tasks = tokio::task::JoinSet::new();
    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            result = &mut shutdown => { result?; break; }
            _ = &mut stop_rx => break,
            Some(result) = tasks.join_next(), if !tasks.is_empty() => {
                if let Err(error) = result { eprintln!("connection task failed: {error}"); }
            }
            accepted = listener.accept() => {
                match accepted {
                    Ok(stream) if tasks.len() < max_connections => {
                        match transport::verified_peer_principal(&stream).and_then(|principal|
                            Connection::new(service.clone(), adapter.clone(), principal, Arc::new(AllowAll)).map_err(io::Error::other)) {
                            Ok(session) => { tasks.spawn(async move {
                                if let Err(error) = pivotd::serve_connection(stream, session, timeouts).await { eprintln!("connection closed: {error}"); }
                            }); }
                            Err(error) => eprintln!("connection identity rejected: {error}"),
                        }
                    }
                    Ok(_) => eprintln!("connection limit reached"),
                    Err(error) => { eprintln!("connection rejected: {error}"); tokio::time::sleep(std::time::Duration::from_millis(25)).await; }
                }
            }
        }
    }
    // Stop accepting, cancel idle/partial readers, and join every connection.
    drop(listener);
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    println!("STOPPED");
    Ok(())
}
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("pivotd: {error}");
        std::process::exit(1);
    }
}
