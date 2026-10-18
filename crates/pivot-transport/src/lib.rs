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

/// Operational policy, not wire semantics. Idle time ends on the first byte;
/// the remaining header and payload then share a fixed partial-frame deadline.
#[derive(Debug, Clone, Copy)]
pub struct FrameTimeouts {
    pub idle: Duration,
    pub partial_read: Duration,
    pub write: Duration,
}
impl Default for FrameTimeouts {
    fn default() -> Self {
        Self {
            idle: Duration::from_secs(300),
            partial_read: Duration::from_secs(5),
            write: Duration::from_secs(5),
        }
    }
}
impl FrameTimeouts {
    fn validate(self) -> io::Result<Self> {
        if self.idle.is_zero() || self.partial_read.is_zero() || self.write.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "timeouts must be positive",
            ));
        }
        Ok(self)
    }
}

/// EOF before the first header byte is clean; any later EOF is truncation.
pub async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    maximum: usize,
) -> io::Result<Option<Vec<u8>>> {
    read_frame_with_timeouts(reader, maximum, FrameTimeouts::default()).await
}
pub async fn read_frame_with_timeouts<R: AsyncRead + Unpin>(
    reader: &mut R,
    maximum: usize,
    timeouts: FrameTimeouts,
) -> io::Result<Option<Vec<u8>>> {
    let timeouts = timeouts.validate()?;
    let mut header = [0; 4];
    let count = tokio::time::timeout(timeouts.idle, reader.read(&mut header[..1]))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "connection idle timeout"))??;
    if count == 0 {
        return Ok(None);
    }
    tokio::time::timeout(timeouts.partial_read, async {
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
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "partial frame read timeout"))?
}
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
    maximum: usize,
) -> io::Result<()> {
    write_frame_with_timeouts(writer, payload, maximum, FrameTimeouts::default()).await
}
pub async fn write_frame_with_timeouts<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
    maximum: usize,
    timeouts: FrameTimeouts,
) -> io::Result<()> {
    let timeouts = timeouts.validate()?;
    if payload.is_empty() || payload.len() > maximum || payload.len() > u32::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame length outside accepted limits",
        ));
    }
    tokio::time::timeout(timeouts.write, async {
        writer
            .write_all(&(payload.len() as u32).to_le_bytes())
            .await?;
        writer.write_all(payload).await?;
        writer.flush().await
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "frame write timeout"))?
}
