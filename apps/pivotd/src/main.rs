use pivot_core::{interface::InputRequirement, model::WorkingModel};
use pivot_host::{AllowAll, Limits, ModelHost, Session};
use pivot_protocol::{v1::ModelRef, validation::MAX_MESSAGE_BYTES};
use pivot_transport::{self as transport, tokio};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{self, Read, Write},
    sync::Arc,
};

fn demo() -> Result<ModelHost, Box<dyn Error>> {
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
    let mut host = ModelHost::new(uuid::Uuid::new_v4().to_string())?;
    host.register(
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
    Ok(host)
}
async fn connection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    mut stream: S,
    host: Arc<ModelHost>,
    timeouts: transport::FrameTimeouts,
) -> io::Result<()> {
    // Transport verified current-user peer identity. Model-level policy remains
    // explicit: the demo exposes only its public interface to that local user.
    let mut session =
        Session::new(Limits::default(), Arc::new(AllowAll)).map_err(io::Error::other)?;
    while let Some(request) =
        transport::read_frame_with_timeouts(&mut stream, MAX_MESSAGE_BYTES, timeouts).await?
    {
        let response = host.dispatch(&mut session, &request);
        transport::write_frame_with_timeouts(&mut stream, &response, MAX_MESSAGE_BYTES, timeouts)
            .await?;
    }
    Ok(())
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
    let mut shutdown_stdin = false;
    let mut timeouts = transport::FrameTimeouts::default();
    let mut max_connections = 16usize;
    let mut work_budget = 100_000usize;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => demo_enabled = true,
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
                    "pivotd --demo [--endpoint LOCAL_ENDPOINT] [--shutdown-stdin]\nPolicy: --idle-timeout-secs 300 --frame-read-timeout-secs 5 --frame-write-timeout-secs 5 --max-connections 16 --work-budget 100000"
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    if !demo_enabled {
        return Err("only --demo hosting is available; persistence is not implemented".into());
    }
    let mut host = demo()?;
    host.set_work_budget(work_budget)?;
    let host = Arc::new(host);
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
                    Ok(stream) if tasks.len() < max_connections => { let host = host.clone(); tasks.spawn(async move {
                        if let Err(error) = connection(stream, host, timeouts).await { eprintln!("connection closed: {error}"); }
                    }); }
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
