use std::{
    fs, io,
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};
pub type Stream = tokio::net::UnixStream;
fn uid() -> u32 {
    unsafe { libc::geteuid() }
}
pub fn default_endpoint() -> io::Result<String> {
    Ok(format!("/tmp/pivot-{}/pivot.sock", uid()))
}
fn private_parent(endpoint: &str) -> io::Result<PathBuf> {
    let path = Path::new(endpoint);
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "socket path must be absolute",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing socket directory"))?;
    match fs::DirBuilder::new().mode(0o700).create(parent) {
        Ok(()) => (),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e),
    }
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != uid()
        || metadata.mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "socket parent must be a private directory owned by the current user",
        ));
    }
    // Reject writable ancestor directories except sticky directories such as /tmp.
    for ancestor in parent.ancestors().skip(1) {
        let m = fs::metadata(ancestor)?;
        if m.mode() & 0o022 != 0 && m.mode() & 0o1000 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "untrusted socket ancestor",
            ));
        }
    }
    Ok(path.to_owned())
}
pub struct Listener {
    listener: tokio::net::UnixListener,
    path: PathBuf,
    identity: (u64, u64),
}
impl Listener {
    pub fn bind(endpoint: &str) -> io::Result<Self> {
        let path = private_parent(endpoint)?;
        // Never delete a preexisting/stale endpoint automatically.
        let listener = tokio::net::UnixListener::bind(&path)?;
        let m = fs::symlink_metadata(&path)?;
        let owner = Self {
            listener,
            path,
            identity: (m.dev(), m.ino()),
        };
        fs::set_permissions(&owner.path, fs::Permissions::from_mode(0o600))?;
        Ok(owner)
    }
    pub async fn accept(&mut self) -> io::Result<Stream> {
        let (stream, _) = self.listener.accept().await?;
        if stream.peer_cred()?.uid() != uid() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "peer user mismatch",
            ));
        }
        Ok(stream)
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        if let Ok(m) = fs::symlink_metadata(&self.path)
            && m.file_type().is_socket()
            && (m.dev(), m.ino()) == self.identity
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
pub async fn connect(endpoint: &str) -> io::Result<Stream> {
    let path = private_parent(endpoint)?;
    let m = fs::symlink_metadata(&path)?;
    if !m.file_type().is_socket() || m.uid() != uid() || m.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "insecure socket",
        ));
    }
    let stream = tokio::net::UnixStream::connect(path).await?;
    if stream.peer_cred()?.uid() != uid() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "server user mismatch",
        ));
    }
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn private_endpoint_permissions_peer_checks_and_cleanup() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = PathBuf::from(format!(
            "/tmp/pivot-security-{}-{unique}",
            std::process::id()
        ));
        fs::DirBuilder::new().mode(0o700).create(&parent).unwrap();
        let path = parent.join("test.sock");
        let endpoint = path.to_str().unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(Listener::bind(endpoint).is_err());
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let mut listener = Listener::bind(endpoint).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
        assert!(Listener::bind(endpoint).is_err());
        let client = connect(endpoint).await.unwrap();
        let peer = tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(peer.peer_cred().unwrap().uid(), uid());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(connect(endpoint).await.is_err());
        drop(client);
        drop(peer);
        drop(listener);
        assert!(!path.exists());
        // Cleanup must not delete a replacement file at the old endpoint path.
        let listener = Listener::bind(endpoint).unwrap();
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"replacement").unwrap();
        drop(listener);
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert!(Listener::bind(endpoint).is_err());
        fs::remove_file(&path).unwrap();
        let alias = parent.with_extension("alias");
        std::os::unix::fs::symlink(&parent, &alias).unwrap();
        assert!(Listener::bind(alias.join("test.sock").to_str().unwrap()).is_err());
        fs::remove_file(&alias).unwrap();
        fs::remove_dir(&parent).unwrap();
    }
}
