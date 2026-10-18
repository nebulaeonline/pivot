//! Daemon connection execution, shared with transport lifecycle tests.
use pivot_host::management::connection::Connection;
use pivot_protocol::validation::MAX_MESSAGE_BYTES;
use pivot_transport::{self as transport, tokio};
use std::io;

pub async fn serve_connection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    mut stream: S,
    mut session: Connection,
    timeouts: transport::FrameTimeouts,
) -> io::Result<()> {
    while let Some(request) =
        transport::read_frame_with_timeouts(&mut stream, MAX_MESSAGE_BYTES, timeouts).await?
    {
        // A bounded blocking job owns the session. Cancellation drops it only
        // after dispatch has released all of its service guards.
        let (returned, response) = tokio::task::spawn_blocking(move || {
            let response = session.dispatch(&request);
            (session, response)
        })
        .await
        .map_err(io::Error::other)?;
        session = returned;
        transport::write_frame_with_timeouts(&mut stream, &response, MAX_MESSAGE_BYTES, timeouts)
            .await?;
    }
    Ok(())
}
