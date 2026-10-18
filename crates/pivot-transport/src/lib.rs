//! Local byte streams and bounded framing. No engine or protocol dependency.
use std::{io, time::Duration};
pub use tokio;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{Listener, Stream, connect, default_endpoint};
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::{Listener, Stream, connect, default_endpoint};

pub const FRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// EOF before the first header byte is clean; any later EOF is truncation.
pub async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    maximum: usize,
) -> io::Result<Option<Vec<u8>>> {
    tokio::time::timeout(FRAME_TIMEOUT, async {
        let mut header = [0; 4];
        if reader.read(&mut header[..1]).await? == 0 {
            return Ok(None);
        }
        reader.read_exact(&mut header[1..]).await?;
        let length = u32::from_le_bytes(header) as usize;
        if length == 0 || length > maximum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame length outside accepted limits",
            ));
        }
        let mut payload = vec![0; length];
        reader.read_exact(&mut payload).await?;
        Ok(Some(payload))
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "frame read timeout"))?
}
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
    maximum: usize,
) -> io::Result<()> {
    if payload.is_empty() || payload.len() > maximum || payload.len() > u32::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame length outside accepted limits",
        ));
    }
    tokio::time::timeout(FRAME_TIMEOUT, async {
        writer
            .write_all(&(payload.len() as u32).to_le_bytes())
            .await?;
        writer.write_all(payload).await?;
        writer.flush().await
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "frame write timeout"))?
}
