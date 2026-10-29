//! Experimental local model files. Not a production .pivot format or IPC contract.
mod platform;
use crate::{
    Limits, Uuid, codec,
    document::{self, DocumentSession, ExportCapture, ModelDocument},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

const MAGIC: &[u8; 8] = b"PIVXFILE";
pub const HEADER_BYTES: usize = 68;
const HARD_PAYLOAD_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct FileLimits {
    pub payload_bytes: usize,
    pub graph: Limits,
}
impl Default for FileLimits {
    fn default() -> Self {
        Self {
            payload_bytes: 4 * 1024 * 1024,
            graph: Limits::default(),
        }
    }
}
impl FileLimits {
    fn total(self) -> Result<usize> {
        if self.payload_bytes == 0 || self.payload_bytes > HARD_PAYLOAD_LIMIT {
            return Err(Error::Limit);
        }
        Ok(self.payload_bytes + HEADER_BYTES)
    }
    fn codec(self) -> Result<Limits> {
        self.total()?;
        Ok(Limits {
            bytes: self.payload_bytes,
            ..self.graph
        })
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Graph(crate::Error),
    Document(document::Error),
    UnsupportedVersion,
    Malformed,
    Integrity,
    Limit,
    Conflict,
    Busy,
    NeedsPath,
    Pending,
    StaleSave,
    Dirty,
    Closed,
    Uncertain(Box<Error>),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "experimental file {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Self::Graph(e)
    }
}
impl From<document::Error> for Error {
    fn from(e: document::Error) -> Self {
        Self::Document(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub fn encode(document: &ModelDocument, version: Uuid, limits: FileLimits) -> Result<Vec<u8>> {
    limits.total()?;
    if version.is_nil() {
        return Err(Error::Malformed);
    }
    document.validate(limits.graph)?;
    let payload = codec::encode_document(document, limits.codec()?)?;
    let mut bytes = Vec::with_capacity(HEADER_BYTES + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&1u32.to_be_bytes());
    bytes.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    bytes.extend_from_slice(version.as_bytes());
    let mut hash = Sha256::new();
    hash.update(&bytes);
    hash.update(&payload);
    bytes.extend_from_slice(&hash.finalize());
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}
pub fn decode(bytes: &[u8], limits: FileLimits) -> Result<(Uuid, ModelDocument)> {
    if bytes.len() > limits.total()? {
        return Err(Error::Limit);
    }
    if bytes.len() < HEADER_BYTES || &bytes[..8] != MAGIC {
        return Err(Error::Malformed);
    }
    if bytes[8..12] != 1u32.to_be_bytes() {
        return Err(Error::UnsupportedVersion);
    }
    let length = u64::from_be_bytes(bytes[12..20].try_into().expect("header"));
    if length > limits.payload_bytes as u64 {
        return Err(Error::Limit);
    }
    if length as usize != bytes.len() - HEADER_BYTES {
        return Err(Error::Malformed);
    }
    let version = Uuid::from_slice(&bytes[20..36]).map_err(|_| Error::Malformed)?;
    if version.is_nil() {
        return Err(Error::Malformed);
    }
    let mut hash = Sha256::new();
    hash.update(&bytes[..36]);
    hash.update(&bytes[HEADER_BYTES..]);
    if hash.finalize().as_slice() != &bytes[36..68] {
        return Err(Error::Integrity);
    }
    let document = codec::decode_document(&bytes[HEADER_BYTES..], limits.codec()?)?;
    document.validate(limits.graph)?;
    Ok((version, document))
}
fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    identity: (u64, u64),
    length: u64,
    modified: Option<SystemTime>,
    digest: [u8; 32],
    mode: u32,
}
#[derive(Clone, Debug)]
pub struct Association {
    path: PathBuf,
    fingerprint: Fingerprint,
    version: Uuid,
}
impl Association {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
    pub fn version(&self) -> Uuid {
        self.version
    }
}
fn target(path: &Path) -> Result<PathBuf> {
    let name = path.file_name().ok_or(Error::Malformed)?;
    let reserved = name.to_string_lossy().to_ascii_lowercase();
    if reserved == ".pivot-experimental-write.lock" || reserved.starts_with(".pivot-exp-") {
        return Err(Error::Conflict);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent)?;
    Ok(parent.join(name))
}
fn read_target(path: &Path, limits: FileLimits) -> Result<(Vec<u8>, Fingerprint)> {
    read_target_bounded(path, limits.total()?)
}
fn read_target_bounded(path: &Path, total: usize) -> Result<(Vec<u8>, Fingerprint)> {
    let before = fs::symlink_metadata(path)?;
    if !before.file_type().is_file() {
        return Err(Error::Conflict);
    }
    let mut file = platform::open_read(path)?;
    let metadata = file.metadata()?;
    let identity = platform::identity(&file)?;
    if metadata.len() > total as u64 {
        return Err(Error::Limit);
    }
    let mut bytes = Vec::new();
    (&mut file).take(total as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > total {
        return Err(Error::Limit);
    }
    let after = file.metadata()?;
    let check = platform::open_read(path)?;
    if platform::identity(&check)? != identity
        || metadata.len() != after.len()
        || bytes.len() as u64 != after.len()
        || metadata.modified().ok() != after.modified().ok()
        || !fs::symlink_metadata(path)?.file_type().is_file()
    {
        return Err(Error::Conflict);
    }
    let fingerprint = Fingerprint {
        identity,
        length: after.len(),
        modified: after.modified().ok(),
        digest: digest(&bytes),
        mode: platform::mode(&after),
    };
    Ok((bytes, fingerprint))
}
pub fn observe(path: impl AsRef<Path>, limits: FileLimits) -> Result<Fingerprint> {
    read_target(&target(path.as_ref())?, limits).map(|(_, f)| f)
}

// One stable per-directory reservation intentionally serializes this small proof.
// Keep the file after unlocking: unlinking a lock file permits split ownership.
struct Reservation(File);
impl Drop for Reservation {
    fn drop(&mut self) {
        // Explicit unlock matters on Unix: a concurrent fork can briefly inherit
        // an open-file description even though the descriptor is close-on-exec.
        let _ = self.0.unlock();
    }
}
fn reserve(parent: &Path) -> Result<Reservation> {
    let path = parent.join(".pivot-experimental-write.lock");
    let file = match platform::private_new(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            if !fs::symlink_metadata(&path)?.file_type().is_file() {
                return Err(Error::Conflict);
            }
            platform::open_lock(&path)?
        }
        Err(e) => return Err(e.into()),
    };
    platform::identity(&file)?; // Reject hard-linked reservation files too.
    file.try_lock().map_err(|e| match e {
        fs::TryLockError::WouldBlock => Error::Busy,
        fs::TryLockError::Error(e) => Error::Io(e),
    })?;
    Ok(Reservation(file))
}
struct Temporary {
    path: PathBuf,
    file: File,
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Clone, Debug)]
pub enum SaveTarget {
    New,
    ReplaceObserved(Fingerprint),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub enum SavePoint {
    BeforeEncode,
    BeforeWrite,
    PartialWrite,
    BeforeInstall,
    AfterInstall,
    BeforeVerify,
}

pub struct SavePlan {
    ticket: Uuid,
    capture: ExportCapture,
    target: PathBuf,
    baseline: Option<Fingerprint>,
    version: Uuid,
    limits: FileLimits,
}
pub struct SaveOutcome {
    plan: SavePlan,
    result: Result<Fingerprint>,
    encoded_digest: Option<[u8; 32]>,
    // Keep reservation through acknowledgment; drop releases the OS lock.
    _reservation: Option<Reservation>,
}
impl SavePlan {
    pub fn commit(self) -> SaveOutcome {
        self.commit_with(|_| Ok(()))
    }
    /// Test seam for write faults/process termination; not part of a file format.
    #[doc(hidden)]
    pub fn commit_with(self, mut hook: impl FnMut(SavePoint) -> io::Result<()>) -> SaveOutcome {
        let mut attempted = false;
        let mut reservation = None;
        let mut encoded_digest = None;
        let result = (|| {
            hook(SavePoint::BeforeEncode)?;
            let bytes = encode(self.capture.document(), self.version, self.limits)?;
            encoded_digest = Some(digest(&bytes));
            let parent = self.target.parent().ok_or(Error::Malformed)?;
            reservation = Some(reserve(parent)?);
            self.check_baseline()?;
            let path = parent.join(format!(".pivot-exp-{}.tmp", Uuid::new_v4()));
            let mut temporary = Temporary {
                file: platform::private_new(&path)?,
                path,
            };
            hook(SavePoint::BeforeWrite)?;
            let middle = bytes.len() / 2;
            temporary.file.write_all(&bytes[..middle])?;
            hook(SavePoint::PartialWrite)?;
            temporary.file.write_all(&bytes[middle..])?;
            temporary.file.sync_all()?;
            // Permission restoration is supported for ordinary Unix mode bits.
            if let Some(baseline) = &self.baseline {
                platform::set_mode(&temporary.file, baseline.mode)?;
            }
            temporary.file.sync_all()?;
            let staged = read_target(&temporary.path, self.limits)?.0;
            if staged != bytes {
                return Err(Error::Integrity);
            }
            decode(&staged, self.limits)?;
            hook(SavePoint::BeforeInstall)?;
            self.check_baseline()?;
            if self.baseline.is_some() {
                attempted = true;
                fs::rename(&temporary.path, &self.target)?;
            } else {
                // Linking the complete staged file atomically creates a new name,
                // failing if any destination exists. Never use rename to clobber.
                fs::hard_link(&temporary.path, &self.target).map_err(|e| {
                    if e.kind() == io::ErrorKind::AlreadyExists {
                        Error::Conflict
                    } else {
                        e.into()
                    }
                })?;
                attempted = true;
                fs::remove_file(&temporary.path)?;
            }
            hook(SavePoint::AfterInstall)?;
            platform::sync_parent(parent)?;
            hook(SavePoint::BeforeVerify)?;
            let (installed, fingerprint) = read_target(&self.target, self.limits)?;
            if installed != bytes {
                return Err(Error::Integrity);
            }
            let (version, _) = decode(&installed, self.limits)?;
            if version != self.version {
                return Err(Error::Integrity);
            }
            Ok(fingerprint)
        })();
        let result = result.map_err(|e| {
            if attempted {
                Error::Uncertain(Box::new(e))
            } else {
                e
            }
        });
        SaveOutcome {
            plan: self,
            result,
            encoded_digest,
            _reservation: reservation,
        }
    }
    fn check_baseline(&self) -> Result<()> {
        match &self.baseline {
            Some(expected) => {
                let observed = read_target(&self.target, self.limits)
                    .map_err(|_| Error::Conflict)?
                    .1;
                if &observed != expected {
                    return Err(Error::Conflict);
                }
            }
            None => match fs::symlink_metadata(&self.target) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Ok(_) => return Err(Error::Conflict),
                Err(e) => return Err(e.into()),
            },
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Unresolved {
    target: PathBuf,
    version: Uuid,
    digest: [u8; 32],
    capture: ExportCapture,
}
pub struct FileDocument {
    session: Option<DocumentSession>,
    association: Option<Association>,
    limits: FileLimits,
    pending: Option<Uuid>,
    unresolved: Option<Unresolved>,
}
impl FileDocument {
    pub fn new(name: &str, limits: FileLimits) -> Result<Self> {
        limits.total()?;
        Ok(Self {
            session: Some(DocumentSession::new(name, limits.graph)?),
            association: None,
            limits,
            pending: None,
            unresolved: None,
        })
    }
    pub fn open(path: impl AsRef<Path>, limits: FileLimits) -> Result<Self> {
        let path = target(path.as_ref())?;
        let (bytes, fingerprint) = read_target(&path, limits)?;
        let (version, document) = decode(&bytes, limits)?;
        let mut session = DocumentSession::load(document, limits.graph)?;
        let capture = session.export(session.expected())?;
        session.acknowledge_saved(&capture)?;
        Ok(Self {
            session: Some(session),
            association: Some(Association {
                path,
                fingerprint,
                version,
            }),
            limits,
            pending: None,
            unresolved: None,
        })
    }
    pub fn session(&self) -> Result<&DocumentSession> {
        self.session.as_ref().ok_or(Error::Closed)
    }
    pub fn session_mut(&mut self) -> Result<&mut DocumentSession> {
        self.session.as_mut().ok_or(Error::Closed)
    }
    pub fn association(&self) -> Option<&Association> {
        self.association.as_ref()
    }
    pub fn is_dirty(&self) -> bool {
        self.unresolved.is_some() || self.session.as_ref().is_some_and(DocumentSession::is_dirty)
    }
    pub fn prepare_save(&mut self) -> Result<SavePlan> {
        let a = self.association.as_ref().ok_or(Error::NeedsPath)?.clone();
        self.prepare_save_as(a.path, SaveTarget::ReplaceObserved(a.fingerprint))
    }
    pub fn prepare_save_as(
        &mut self,
        path: impl AsRef<Path>,
        mode: SaveTarget,
    ) -> Result<SavePlan> {
        if self.pending.is_some() || self.unresolved.is_some() {
            return Err(Error::Pending);
        }
        let target = target(path.as_ref())?;
        let session = self.session_mut()?;
        let capture = session.export(session.expected())?;
        let ticket = Uuid::new_v4();
        self.pending = Some(ticket);
        let baseline = match mode {
            SaveTarget::New => None,
            SaveTarget::ReplaceObserved(f) => Some(f),
        };
        Ok(SavePlan {
            ticket,
            capture,
            target,
            baseline,
            version: Uuid::new_v4(),
            limits: self.limits,
        })
    }
    pub fn cancel_prepared(&mut self, plan: SavePlan) -> Result<()> {
        if self.pending != Some(plan.ticket) {
            return Err(Error::StaleSave);
        }
        self.pending = None;
        Ok(())
    }
    /// A borrowed outcome retains its writer reservation until the caller drops it.
    pub fn finish_save(&mut self, outcome: &SaveOutcome) -> Result<()> {
        if self.pending != Some(outcome.plan.ticket) {
            return Err(Error::StaleSave);
        }
        self.pending = None;
        match &outcome.result {
            Ok(fingerprint) => {
                if let Err(e) = self.session_mut()?.acknowledge_saved(&outcome.plan.capture) {
                    self.remember_uncertain(outcome);
                    return Err(Error::Uncertain(Box::new(e.into())));
                }
                self.association = Some(Association {
                    path: outcome.plan.target.clone(),
                    fingerprint: fingerprint.clone(),
                    version: outcome.plan.version,
                });
                Ok(())
            }
            Err(error) => {
                self.session_mut()?.mark_unsaved();
                if matches!(error, Error::Uncertain(_)) {
                    self.remember_uncertain(outcome);
                }
                // Preserve a typed outcome without cloning io::Error/OS internals.
                Err(match error {
                    Error::Uncertain(_) => {
                        Error::Uncertain(Box::new(Error::Io(io::Error::other(error.to_string()))))
                    }
                    Error::Conflict => Error::Conflict,
                    Error::Busy => Error::Busy,
                    _ => Error::Io(io::Error::other(error.to_string())),
                })
            }
        }
    }
    fn remember_uncertain(&mut self, outcome: &SaveOutcome) {
        self.unresolved = Some(Unresolved {
            target: outcome.plan.target.clone(),
            version: outcome.plan.version,
            digest: outcome
                .encoded_digest
                .expect("installation requires encoding"),
            capture: outcome.plan.capture.clone(),
        });
        if let Some(s) = &mut self.session {
            s.mark_unsaved();
        }
    }
    pub fn reconcile(&mut self) -> Result<()> {
        let unresolved = self.unresolved.clone().ok_or(Error::StaleSave)?;
        let parent = unresolved.target.parent().ok_or(Error::Malformed)?;
        let _reservation = reserve(parent)?;
        let mut file = platform::open_lock(&unresolved.target)?;
        file.flush()?;
        file.sync_all()?;
        platform::sync_parent(parent)?;
        let (bytes, fingerprint) = read_target(&unresolved.target, self.limits)?;
        let (version, _) = decode(&bytes, self.limits)?;
        if version != unresolved.version || digest(&bytes) != unresolved.digest {
            return Err(Error::Conflict);
        }
        self.session_mut()?.acknowledge_saved(&unresolved.capture)?;
        self.association = Some(Association {
            path: unresolved.target,
            fingerprint,
            version,
        });
        self.unresolved = None;
        Ok(())
    }
    pub fn save(&mut self) -> Result<()> {
        let result = (|| {
            let outcome = self.prepare_save()?.commit();
            self.finish_save(&outcome)
        })();
        if result.is_err()
            && let Some(s) = &mut self.session
        {
            s.mark_unsaved();
        }
        result
    }
    pub fn save_as(&mut self, path: impl AsRef<Path>, mode: SaveTarget) -> Result<()> {
        let result = (|| {
            let outcome = self.prepare_save_as(path, mode)?.commit();
            self.finish_save(&outcome)
        })();
        if result.is_err()
            && let Some(s) = &mut self.session
        {
            s.mark_unsaved();
        }
        result
    }
    /// Save explicitly first, or explicitly discard. In-flight plans cannot close.
    pub fn close(&mut self, discard: bool) -> Result<()> {
        if self.pending.is_some() {
            return Err(Error::Pending);
        }
        self.session()?;
        if self.is_dirty() && !discard {
            return Err(Error::Dirty);
        }
        self.session = None;
        self.association = None;
        self.unresolved = None;
        Ok(())
    }
}

/// Shared local-file safety broker for caller-validated native document envelopes.
/// This does not wrap bytes in the experimental model-file format.
pub mod native {
    use super::*;
    #[derive(Clone, Debug)]
    pub struct Location {
        pub path: PathBuf,
        pub fingerprint: Option<Fingerprint>,
    }
    /// Local, feature-only identity token for association research. Not portable authority.
    #[cfg(feature = "history-experiment")]
    pub fn research_identity(path: &Path) -> Result<[u8; 32]> {
        let file = platform::open_read(path)?;
        let (a, b) = platform::identity(&file)?;
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&a.to_le_bytes());
        bytes[8..].copy_from_slice(&b.to_le_bytes());
        Ok(digest(&bytes))
    }
    pub fn destination(path: &Path) -> Result<Location> {
        let path = target(path)?;
        let fingerprint = match fs::symlink_metadata(&path) {
            Ok(_) => Some(read_target_bounded(&path, 64 * 1024 * 1024)?.1),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        Ok(Location { path, fingerprint })
    }
    pub fn read(path: &Path) -> Result<(Vec<u8>, Location)> {
        let path = target(path)?;
        let (bytes, fingerprint) = read_target_bounded(&path, 64 * 1024 * 1024)?;
        Ok((
            bytes,
            Location {
                path,
                fingerprint: Some(fingerprint),
            },
        ))
    }
    pub fn install(location: &Location, bytes: &[u8]) -> Result<Location> {
        install_with(location, bytes, |_| Ok(()))
    }
    /// Failure seam shares the existing file-broker fault points.
    #[doc(hidden)]
    pub fn install_with(
        location: &Location,
        bytes: &[u8],
        mut hook: impl FnMut(SavePoint) -> io::Result<()>,
    ) -> Result<Location> {
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(Error::Limit);
        }
        let parent = location.path.parent().ok_or(Error::Malformed)?;
        let _reservation = reserve(parent)?;
        let check = || -> Result<()> {
            match &location.fingerprint {
                Some(expected) => {
                    if &read_target_bounded(&location.path, 64 * 1024 * 1024)
                        .map_err(|_| Error::Conflict)?
                        .1
                        != expected
                    {
                        return Err(Error::Conflict);
                    }
                }
                None => match fs::symlink_metadata(&location.path) {
                    Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                    Ok(_) => return Err(Error::Conflict),
                    Err(e) => return Err(e.into()),
                },
            }
            Ok(())
        };
        check()?;
        let path = parent.join(format!(".pivot-exp-{}.tmp", Uuid::new_v4()));
        let mut temporary = Temporary {
            file: platform::private_new(&path)?,
            path,
        };
        hook(SavePoint::BeforeWrite)?;
        temporary.file.write_all(&bytes[..bytes.len() / 2])?;
        hook(SavePoint::PartialWrite)?;
        temporary.file.write_all(&bytes[bytes.len() / 2..])?;
        if let Some(old) = &location.fingerprint {
            platform::set_mode(&temporary.file, old.mode)?;
        }
        temporary.file.sync_all()?;
        if read_target_bounded(&temporary.path, 64 * 1024 * 1024)?.0 != bytes {
            return Err(Error::Integrity);
        }
        hook(SavePoint::BeforeInstall)?;
        check()?;
        // A replace error is conservatively uncertain once installation is attempted.
        let installed = (|| -> Result<Location> {
            if location.fingerprint.is_some() {
                fs::rename(&temporary.path, &location.path)?;
            } else {
                fs::hard_link(&temporary.path, &location.path)?;
                fs::remove_file(&temporary.path)?;
            }
            hook(SavePoint::AfterInstall)?;
            platform::sync_parent(parent)?;
            hook(SavePoint::BeforeVerify)?;
            let (actual, fingerprint) = read_target_bounded(&location.path, 64 * 1024 * 1024)?;
            if actual != bytes {
                return Err(Error::Integrity);
            }
            Ok(Location {
                path: location.path.clone(),
                fingerprint: Some(fingerprint),
            })
        })();
        installed.map_err(|e| Error::Uncertain(Box::new(e)))
    }
}
