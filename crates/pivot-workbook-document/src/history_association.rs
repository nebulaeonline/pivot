//! Stage 4 association/retention research. No production session, WB5 or packaging contract.
use crate::{Document, MAX_BYTES};
use pivot_core::model::integration::{
    IntegrationRuntime,
    history_codec::{Blob, Reader, Writer},
    resources::{Charge, Memory},
};
use pivot_storage::experimental_file::{self as files, native};
use pivot_workbook::{
    history_experiment::{
        ColdLedger,
        durability::{Candidate, Config, Fault, Journal},
    },
    *,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
fn bad() -> Error {
    Error::Graph("invalid/unavailable research history association".into())
}
fn io(e: impl std::fmt::Debug) -> Error {
    Error::Graph(format!("{e:?}"))
}
fn hash(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
fn n(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}
fn put(b: &mut [u8], at: usize, v: u64) {
    b[at..at + 8].copy_from_slice(&v.to_le_bytes());
}
fn id(r: &mut Reader<'_>) -> Result<Uuid> {
    Ok(Uuid::from_bytes(r.id().map_err(io)?))
}
fn text(r: &mut Reader<'_>) -> Result<String> {
    Ok(std::str::from_utf8(r.bytes(4096).map_err(io)?)
        .map_err(io)?
        .to_owned())
}
fn path_text(p: &Path) -> Result<&str> {
    p.to_str().filter(|s| s.len() <= 4096).ok_or_else(bad)
}
fn identity(p: &Path) -> Result<[u8; 32]> {
    native::research_identity(p).map_err(io)
}
fn sync_parent(p: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(p).map_err(io)?.sync_all().map_err(io)?;
    let _ = p;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Retained,
    SessionOnly,
}
#[derive(Clone, Debug)]
struct Epoch {
    name: String,
    instance: Uuid,
    physical: [u8; 32],
}
#[derive(Clone, Debug)]
pub struct Saved {
    pub ticket: Uuid,
    pub path: PathBuf,
    pub checkpoint: Uuid,
    pub content: Uuid,
    physical: [u8; 32],
    digest: [u8; 32],
}
#[derive(Clone)]
struct Intent {
    ticket: Uuid,
    path: PathBuf,
    checkpoint: Uuid,
    content: Uuid,
    bytes: Vec<u8>,
    attempted: bool,
    before: Option<([u8; 32], [u8; 32])>,
}
#[derive(Clone)]
struct State {
    epoch: Epoch,
    saved: Option<Saved>,
    intent: Option<Intent>,
    policy: Policy,
    closed: bool,
    disabled: bool,
    relocating: Option<PathBuf>,
}
impl State {
    fn weight(&self) -> usize {
        self.intent.as_ref().map_or(0, |x| x.bytes.capacity()) + 32768
    }
    fn encode(&self, m: &Memory) -> Result<Blob> {
        let mut w = Writer::new(m).map_err(io)?;
        w.text(&self.epoch.name);
        w.raw(self.epoch.instance.as_bytes());
        w.raw(&self.epoch.physical);
        w.flag(self.policy == Policy::Retained);
        w.flag(self.closed);
        w.flag(self.disabled);
        w.flag(self.relocating.is_some());
        if let Some(p) = &self.relocating {
            w.text(path_text(p)?);
        }
        w.flag(self.saved.is_some());
        if let Some(s) = &self.saved {
            w.raw(s.ticket.as_bytes());
            w.text(path_text(&s.path)?);
            w.raw(s.checkpoint.as_bytes());
            w.raw(s.content.as_bytes());
            w.raw(&s.physical);
            w.raw(&s.digest);
        }
        w.flag(self.intent.is_some());
        if let Some(s) = &self.intent {
            w.raw(s.ticket.as_bytes());
            w.text(path_text(&s.path)?);
            w.raw(s.checkpoint.as_bytes());
            w.raw(s.content.as_bytes());
            w.flag(s.attempted);
            w.flag(s.before.is_some());
            if let Some((a, b)) = s.before {
                w.raw(&a);
                w.raw(&b);
            }
            w.bytes(&s.bytes);
        }
        w.finish().map_err(io)
    }
    fn decode(b: &[u8]) -> Result<Self> {
        let mut r = Reader::new(b).map_err(io)?;
        let name = text(&mut r)?;
        if !name.starts_with("epoch-")
            || !name.ends_with(".journal")
            || Path::new(&name).components().count() != 1
        {
            return Err(bad());
        }
        let epoch = Epoch {
            name,
            instance: id(&mut r)?,
            physical: r.take(32).map_err(io)?.try_into().unwrap(),
        };
        let policy = if r.flag().map_err(io)? {
            Policy::Retained
        } else {
            Policy::SessionOnly
        };
        let closed = r.flag().map_err(io)?;
        let disabled = r.flag().map_err(io)?;
        let relocating = if r.flag().map_err(io)? {
            Some(PathBuf::from(text(&mut r)?))
        } else {
            None
        };
        let saved = if r.flag().map_err(io)? {
            Some(Saved {
                ticket: id(&mut r)?,
                path: PathBuf::from(text(&mut r)?),
                checkpoint: id(&mut r)?,
                content: id(&mut r)?,
                physical: r.take(32).map_err(io)?.try_into().unwrap(),
                digest: r.take(32).map_err(io)?.try_into().unwrap(),
            })
        } else {
            None
        };
        let intent = if r.flag().map_err(io)? {
            Some(Intent {
                ticket: id(&mut r)?,
                path: PathBuf::from(text(&mut r)?),
                checkpoint: id(&mut r)?,
                content: id(&mut r)?,
                attempted: r.flag().map_err(io)?,
                before: if r.flag().map_err(io)? {
                    Some((
                        r.take(32).map_err(io)?.try_into().unwrap(),
                        r.take(32).map_err(io)?.try_into().unwrap(),
                    ))
                } else {
                    None
                },
                bytes: r.bytes(MAX_BYTES).map_err(io)?.to_vec(),
            })
        } else {
            None
        };
        r.end().map_err(io)?;
        Ok(Self {
            epoch,
            saved,
            intent,
            policy,
            closed,
            disabled,
            relocating,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Point {
    BeforeControl,
    PartialControl,
    BeforeControlDecision,
    PartialControlDecision,
    AfterControlDecision,
    AfterInstall,
    BeforeSaveReceipt,
    BeforeMove,
    AfterMove,
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Error,
    Pause,
}
#[derive(Default, Clone, Copy)]
pub struct Hook(pub Option<(Point, Action)>);
impl Hook {
    fn hit(self, p: Point) -> Result<()> {
        if let Some((at, action)) = self.0
            && at == p
        {
            match action {
                Action::Error => return Err(Error::Graph(format!("injected {p:?}"))),
                Action::Pause => {
                    println!("STAGE4_READY {p:?}");
                    std::io::stdout().flush().map_err(io)?;
                    loop {
                        std::thread::park();
                    }
                }
            }
        }
        Ok(())
    }
}

struct Control {
    file: File,
    physical: [u8; 32],
    group: Uuid,
    seq: u64,
    digest: [u8; 32],
    last_id: Uuid,
    end: u64,
    state: State,
    charge: Charge,
    memory: Memory,
    pending: Option<Uuid>,
}
impl Control {
    fn create(root: &Path, state: State, memory: Memory) -> Result<Self> {
        let path = root.join("control");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(io)?;
        let physical = identity(&path)?;
        file.try_lock().map_err(io)?;
        let group = Uuid::new_v4();
        let mut h = [0; 96];
        h[..8].copy_from_slice(b"PVH4CTL2");
        h[8..24].copy_from_slice(group.as_bytes());
        h[24..56].copy_from_slice(&physical);
        let sum = hash(&h[..64]);
        h[64..].copy_from_slice(&sum);
        file.write_all(&h).map_err(io)?;
        file.sync_all().map_err(io)?;
        sync_parent(root)?;
        let charge = memory.reserve(state.weight()).map_err(io)?;
        let mut c = Self {
            file,
            physical,
            group,
            seq: 0,
            digest: [0; 32],
            last_id: Uuid::nil(),
            end: 96,
            state,
            charge,
            memory,
            pending: None,
        };
        let (state, charge) = c.copy()?;
        c.append(state, charge, Hook::default())?;
        Ok(c)
    }
    fn open(root: &Path, memory: Memory) -> Result<Self> {
        let path = root.join("control");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(io)?;
        file.try_lock().map_err(io)?;
        let mut h = [0; 96];
        file.read_exact(&mut h).map_err(io)?;
        if &h[..8] != b"PVH4CTL2" || hash(&h[..64]) != h[64..] || h[24..56] != identity(&path)? {
            return Err(bad());
        }
        let placeholder = State {
            epoch: Epoch {
                name: "epoch-placeholder.journal".into(),
                instance: Uuid::nil(),
                physical: [0; 32],
            },
            saved: None,
            intent: None,
            policy: Policy::Retained,
            closed: false,
            disabled: false,
            relocating: None,
        };
        let charge = memory.reserve(32768).map_err(io)?;
        let mut c = Self {
            file,
            physical: h[24..56].try_into().unwrap(),
            group: Uuid::from_bytes(h[8..24].try_into().unwrap()),
            seq: 0,
            digest: [0; 32],
            last_id: Uuid::nil(),
            end: 96,
            state: placeholder,
            charge,
            memory,
            pending: None,
        };
        c.recover()?;
        if c.seq == 0 {
            return Err(bad());
        }
        Ok(c)
    }
    fn copy(&self) -> Result<(State, Charge)> {
        self.ready()?;
        let charge = self.memory.reserve(self.state.weight()).map_err(io)?;
        Ok((self.state.clone(), charge))
    }
    fn ready(&self) -> Result<()> {
        if self.pending.is_some() {
            Err(Error::Graph("control reconciliation required".into()))
        } else {
            Ok(())
        }
    }
    fn recover(&mut self) -> Result<()> {
        let size = self.file.metadata().map_err(io)?.len();
        let mut offset = 96;
        let mut seq = 0;
        let mut digest = [0; 32];
        let mut selected = None;
        while offset < size {
            if size - offset < 136 {
                break;
            }
            self.file.seek(SeekFrom::Start(offset)).map_err(io)?;
            let mut h = [0; 136];
            self.file.read_exact(&mut h).map_err(io)?;
            if &h[..8] != b"PVH4REC1"
                || hash(&h[..104]) != h[104..]
                || n(&h, 8) != seq + 1
                || h[32..64] != digest
            {
                return Err(bad());
            }
            let len = usize::try_from(n(&h, 64)).map_err(io)?;
            if len > 8 * 1024 * 1024 {
                return Err(bad());
            }
            let end = offset.checked_add(200 + len as u64).ok_or_else(bad)?;
            if end > size {
                break;
            }
            let staging = self.memory.reserve(len * 3 + 32768).map_err(io)?;
            let mut bytes = vec![0; len];
            self.file.read_exact(&mut bytes).map_err(io)?;
            let mut foot = [0; 64];
            self.file.read_exact(&mut foot).map_err(io)?;
            if &foot[..8] != b"PVH4YES1"
                || n(&foot, 8) != n(&h, 8)
                || foot[16..48] != h[104..]
                || foot[48..].iter().any(|b| *b != 0)
                || hash(&bytes) != h[72..104]
            {
                return Err(bad());
            }
            let state = State::decode(&bytes)?;
            let charge = self.memory.reserve(state.weight()).map_err(io)?;
            selected = Some((
                state,
                charge,
                Uuid::from_bytes(h[16..32].try_into().unwrap()),
            ));
            seq += 1;
            digest = h[104..].try_into().unwrap();
            offset = end;
            drop(staging);
        }
        if offset < self.end {
            return Err(bad());
        } // Never reconcile away previously known decisions.
        if offset != size {
            self.file.set_len(offset).map_err(io)?;
        }
        self.file.sync_all().map_err(io)?;
        if let Some((state, charge, id)) = selected {
            self.state = state;
            self.charge = charge;
            self.last_id = id;
        }
        self.seq = seq;
        self.digest = digest;
        self.end = offset;
        Ok(())
    }
    fn append(&mut self, state: State, charge: Charge, hook: Hook) -> Result<Uuid> {
        self.ready()?;
        hook.hit(Point::BeforeControl)?;
        let payload = state.encode(&self.memory)?;
        let mut h = [0; 136];
        h[..8].copy_from_slice(b"PVH4REC1");
        put(&mut h, 8, self.seq.checked_add(1).ok_or_else(bad)?);
        let id = Uuid::new_v4();
        h[16..32].copy_from_slice(id.as_bytes());
        h[32..64].copy_from_slice(&self.digest);
        put(&mut h, 64, payload.bytes.len() as u64);
        h[72..104].copy_from_slice(&hash(&payload.bytes));
        let digest = hash(&h[..104]);
        h[104..].copy_from_slice(&digest);
        self.pending = Some(id);
        self.file.seek(SeekFrom::Start(self.end)).map_err(io)?;
        self.file.write_all(&h[..68]).map_err(io)?;
        hook.hit(Point::PartialControl)?;
        self.file.write_all(&h[68..]).map_err(io)?;
        self.file.write_all(&payload.bytes).map_err(io)?;
        self.file.sync_all().map_err(io)?;
        hook.hit(Point::BeforeControlDecision)?;
        let mut foot = [0; 64];
        foot[..8].copy_from_slice(b"PVH4YES1");
        put(&mut foot, 8, self.seq + 1);
        foot[16..48].copy_from_slice(&digest);
        self.file.write_all(&foot[..32]).map_err(io)?;
        hook.hit(Point::PartialControlDecision)?;
        self.file.write_all(&foot[32..]).map_err(io)?;
        self.file.sync_all().map_err(io)?;
        hook.hit(Point::AfterControlDecision)?;
        self.state = state;
        self.charge = charge;
        self.last_id = id;
        self.seq += 1;
        self.digest = digest;
        self.end += 200 + payload.bytes.len() as u64;
        self.pending = None;
        Ok(id)
    }
}

/// Immutable export and original observed target, scoped to one research authority.
pub struct Ticket {
    pub native_encoding: Option<(u64, u64)>,
    id: Uuid,
    group: Uuid,
    bytes: Vec<u8>,
    checkpoint: Uuid,
    location: native::Location,
    _charge: Charge,
}
impl Ticket {
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn content_checkpoint(&self) -> Uuid {
        self.checkpoint
    }
}
pub struct Association {
    root: PathBuf,
    control: Control,
    journal: Option<Journal>,
}
impl Association {
    fn make_epoch(
        root: &Path,
        book: &Workbook,
        ledger: &ColdLedger,
        rt: &IntegrationRuntime,
    ) -> Result<(Epoch, Journal)> {
        let name = format!("epoch-{}.journal", Uuid::new_v4());
        let path = root.join(&name);
        let mut j = Journal::create(&path, Config::default(), rt)?;
        let c = Candidate::capture(book, ledger)?;
        j.commit(Uuid::new_v4(), &c, rt, &mut Fault::default())?;
        let epoch = Epoch {
            name,
            instance: j.instance_id()?,
            physical: identity(&path)?,
        };
        Ok((epoch, j))
    }
    pub fn create(
        root: &Path,
        book: &Workbook,
        ledger: &ColdLedger,
        policy: Policy,
        rt: &IntegrationRuntime,
    ) -> Result<Self> {
        fs::create_dir(root).map_err(io)?;
        let root = root.canonicalize().map_err(io)?;
        let (epoch, journal) = Self::make_epoch(&root, book, ledger, rt)?;
        let control = Control::create(
            &root,
            State {
                epoch,
                saved: None,
                intent: None,
                policy,
                closed: false,
                disabled: false,
                relocating: None,
            },
            rt.memory(),
        )?;
        Ok(Self {
            root,
            control,
            journal: Some(journal),
        })
    }
    fn load_epoch(&mut self, rt: &IntegrationRuntime) -> Result<()> {
        self.journal = None;
        let e = &self.control.state.epoch;
        let path = self.root.join(&e.name);
        if identity(&path)? != e.physical {
            return Err(bad());
        }
        let mut journal = Journal::open(&path, rt)?;
        if journal.instance_id()? != e.instance {
            return Err(bad());
        }
        self.journal = Some(journal);
        Ok(())
    }
    pub fn open(root: &Path, rt: &IntegrationRuntime) -> Result<Self> {
        let root = root.canonicalize().map_err(io)?;
        let control = Control::open(&root, rt.memory())?;
        if control.state.disabled
            || control.state.closed && control.state.policy == Policy::SessionOnly
        {
            return Err(Error::Graph("history retired by explicit policy".into()));
        }
        let mut s = Self {
            root,
            control,
            journal: None,
        };
        s.load_epoch(rt)?;
        Ok(s)
    }
    fn namespace(&self) -> Result<()> {
        if identity(&self.root.join("control"))? != self.control.physical {
            return Err(bad());
        }
        Ok(())
    }
    fn ready(&self) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        if self.control.state.disabled
            || self.control.state.closed
            || self.control.state.relocating.is_some()
            || self
                .control
                .state
                .intent
                .as_ref()
                .is_some_and(|i| i.attempted)
            || self.journal.as_ref().is_none_or(|j| j.pending().is_some())
        {
            return Err(Error::Graph("association reconciliation required".into()));
        }
        Ok(())
    }
    pub fn journal_pending(&self) -> Option<Uuid> {
        self.journal.as_ref().and_then(|j| j.pending())
    }
    pub fn reconcile_journal(&mut self, id: Uuid, rt: &IntegrationRuntime) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        self.journal.as_mut().ok_or_else(bad)?.reconcile(id, rt)?;
        Ok(())
    }
    pub fn control_pending(&self) -> Option<Uuid> {
        self.control.pending
    }
    pub fn reconcile_control(&mut self, id: Uuid, rt: &IntegrationRuntime) -> Result<bool> {
        if self.control.pending != Some(id) {
            return Err(bad());
        }
        self.control.recover()?;
        let committed = self.control.last_id == id;
        self.load_epoch(rt)?;
        self.control.pending = None;
        Ok(committed)
    }
    pub fn save_pending(&self) -> Result<Option<(Uuid, bool)>> {
        self.namespace()?;
        self.control.ready()?;
        Ok(self
            .control
            .state
            .intent
            .as_ref()
            .map(|i| (i.ticket, i.attempted)))
    }
    pub fn saved(&self) -> Result<Option<&Saved>> {
        self.namespace()?;
        self.control.ready()?;
        Ok(self.control.state.saved.as_ref())
    }
    pub fn restore(&mut self, rt: &IntegrationRuntime) -> Result<(Workbook, ColdLedger)> {
        self.ready()?;
        self.journal.as_mut().ok_or_else(bad)?.restore(rt)
    }
    pub fn edit(&mut self, c: &Candidate, rt: &IntegrationRuntime) -> Result<()> {
        self.ready()?;
        self.journal.as_mut().ok_or_else(bad)?.commit(
            Uuid::new_v4(),
            c,
            rt,
            &mut Fault::default(),
        )?;
        Ok(())
    }
    pub fn history_bytes(&mut self) -> Result<Blob> {
        self.ready()?;
        self.journal.as_mut().ok_or_else(bad)?.current_bytes()
    }
    pub fn redo(&mut self) -> Result<()> {
        self.ready()?;
        self.journal
            .as_mut()
            .ok_or_else(bad)?
            .redo(Uuid::new_v4(), &mut Fault::default())?;
        Ok(())
    }
    pub fn undo(&mut self) -> Result<()> {
        self.ready()?;
        self.journal
            .as_mut()
            .ok_or_else(bad)?
            .undo(Uuid::new_v4(), &mut Fault::default())?;
        Ok(())
    }
    pub fn prepare(&mut self, book: &Workbook, path: &Path, hook: Hook) -> Result<Ticket> {
        self.ready()?;
        if self
            .journal
            .as_ref()
            .and_then(|j| j.receipt())
            .is_none_or(|r| r.content != book.history_content_id())
        {
            return Err(bad());
        }
        if self.control.state.intent.is_some() {
            return Err(Error::Graph("save already prepared".into()));
        }
        let reserve = self
            .control
            .memory
            .reserve(MAX_BYTES * 2 + 32768)
            .map_err(io)?;
        let export =
            Document::export_historical_document(book, book.expected(), &Default::default())?;
        let location = native::destination(path).map_err(io)?;
        if location.path.starts_with(&self.root) {
            return Err(bad());
        }
        let before = if location.fingerprint.is_some() {
            let (b, observed) = native::read(&location.path).map_err(io)?;
            if observed.fingerprint != location.fingerprint {
                return Err(bad());
            }
            Some((identity(&location.path)?, hash(&b)))
        } else {
            None
        };
        let id = Uuid::new_v4();
        let (mut state, charge) = self.control.copy()?;
        state.intent = Some(Intent {
            ticket: id,
            path: location.path.clone(),
            checkpoint: export.checkpoint,
            content: book.history_content_id(),
            bytes: export.bytes.clone(),
            attempted: false,
            before,
        });
        let charge_new = self.control.memory.reserve(state.weight()).map_err(io)?;
        drop(charge);
        self.control.append(state, charge_new, hook)?;
        Ok(Ticket {
            native_encoding: Some((export.timings.capture_ns, export.timings.encode_ns)),
            id,
            group: self.control.group,
            bytes: export.bytes,
            checkpoint: export.checkpoint,
            location,
            _charge: reserve,
        })
    }
    pub fn resume_ticket(&self, ticket: Uuid) -> Result<Ticket> {
        self.ready()?;
        let (i, charge) = self.intent(ticket)?;
        if i.attempted {
            return Err(bad());
        }
        let location = native::destination(&i.path).map_err(io)?;
        match i.before {
            Some((a, b)) => {
                let (bytes, _) = native::read(&i.path).map_err(io)?;
                if identity(&i.path)? != a || hash(&bytes) != b {
                    return Err(bad());
                }
            }
            None => {
                if location.fingerprint.is_some() {
                    return Err(bad());
                }
            }
        }
        Ok(Ticket {
            native_encoding: None,
            id: ticket,
            group: self.control.group,
            bytes: i.bytes,
            checkpoint: i.checkpoint,
            location,
            _charge: charge,
        })
    }
    pub fn cancel(&mut self, ticket: Uuid) -> Result<()> {
        self.ready()?;
        let (mut s, c) = self.control.copy()?;
        if s.intent
            .as_ref()
            .is_none_or(|i| i.ticket != ticket || i.attempted)
        {
            return Err(bad());
        }
        s.intent = None;
        self.control.append(s, c, Hook::default())?;
        Ok(())
    }
    fn intent(&self, ticket: Uuid) -> Result<(Intent, Charge)> {
        self.namespace()?;
        self.control.ready()?;
        let i = self
            .control
            .state
            .intent
            .as_ref()
            .filter(|i| i.ticket == ticket)
            .ok_or_else(bad)?;
        let c = self
            .control
            .memory
            .reserve(i.bytes.capacity() + 16384)
            .map_err(io)?;
        Ok((i.clone(), c))
    }
    pub fn finish(
        &mut self,
        t: &Ticket,
        rt: &IntegrationRuntime,
        hook: Hook,
        mut native_hook: impl FnMut(files::SavePoint) -> std::io::Result<()>,
    ) -> Result<()> {
        self.ready()?;
        if t.group != self.control.group || self.intent(t.id)?.0.bytes != t.bytes {
            return Err(bad());
        }
        let (mut s, c) = self.control.copy()?;
        s.intent.as_mut().ok_or_else(bad)?.attempted = true;
        self.control.append(s, c, Hook::default())?;
        native::install_with(&t.location, &t.bytes, &mut native_hook).map_err(io)?;
        hook.hit(Point::AfterInstall)?;
        self.reconcile_install(t.id, rt, hook)
    }
    pub fn reconcile_install(
        &mut self,
        ticket: Uuid,
        rt: &IntegrationRuntime,
        hook: Hook,
    ) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        if self
            .control
            .state
            .saved
            .as_ref()
            .is_some_and(|s| s.ticket == ticket)
        {
            return self
                .verify_attachment(&self.control.state.saved.as_ref().ok_or_else(bad)?.path, rt);
        }
        let (i, _intent_charge) = self.intent(ticket)?;
        if !i.attempted {
            return Err(bad());
        }
        let (bytes, location) = native::read(&i.path).map_err(io)?;
        if bytes != i.bytes {
            return Err(Error::Graph(
                "installed file does not match prepared checkpoint; remains uncertain".into(),
            ));
        }
        let (book, checkpoint) = Document::open_document(&bytes, Limits::default(), rt)?;
        drop(book);
        if checkpoint != i.checkpoint {
            return Err(bad());
        }
        let physical = identity(&location.path)?;
        if native::destination(&location.path).map_err(io)?.fingerprint != location.fingerprint {
            return Err(bad());
        }
        hook.hit(Point::BeforeSaveReceipt)?;
        let (mut s, c) = self.control.copy()?;
        s.saved = Some(Saved {
            ticket,
            path: location.path,
            checkpoint,
            content: i.content,
            physical,
            digest: hash(&bytes),
        });
        s.intent = None;
        self.control.append(s, c, hook)?;
        Ok(())
    }
    pub fn abort_uninstalled(&mut self, ticket: Uuid) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        let (i, _intent_charge) = self.intent(ticket)?;
        match i.before {
            Some((physical, digest)) => {
                let (bytes, _) = native::read(&i.path).map_err(io)?;
                if identity(&i.path)? != physical || hash(&bytes) != digest {
                    return Err(bad());
                }
            }
            None => {
                if native::destination(&i.path)
                    .map_err(io)?
                    .fingerprint
                    .is_some()
                {
                    return Err(bad());
                }
            }
        }
        let (mut s, c) = self.control.copy()?;
        s.intent = None;
        self.control.append(s, c, Hook::default())?;
        Ok(())
    }
    /// Verify checkpoint and physical observation from the same bounded native read.
    pub fn verify_attachment(&self, path: &Path, rt: &IntegrationRuntime) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        let saved = self.control.state.saved.as_ref().ok_or_else(bad)?;
        let (bytes, location) = native::read(path).map_err(io)?;
        if hash(&bytes) != saved.digest {
            return Err(bad());
        }
        let (book, cp) = Document::open_document(&bytes, Limits::default(), rt)?;
        drop(book);
        if cp != saved.checkpoint
            || identity(&location.path)? != saved.physical
            || native::destination(&location.path).map_err(io)?.fingerprint != location.fingerprint
        {
            return Err(bad());
        }
        Ok(())
    }
    pub fn history_costs(
        &self,
    ) -> Result<&pivot_workbook::history_experiment::durability::Metrics> {
        self.ready()?;
        Ok(&self.journal.as_ref().ok_or_else(bad)?.metrics)
    }
    pub fn is_clean(&self, book: &Workbook, path: &Path, rt: &IntegrationRuntime) -> Result<bool> {
        self.verify_attachment(path, rt)?;
        Ok(self
            .control
            .state
            .saved
            .as_ref()
            .is_some_and(|s| s.content == book.history_content_id())
            && self.control.state.intent.is_none())
    }
    pub fn recover_matching(
        &mut self,
        path: &Path,
        rt: &IntegrationRuntime,
    ) -> Result<(Workbook, ColdLedger)> {
        self.verify_attachment(path, rt)?;
        self.restore(rt)
    }
    pub fn clear(&mut self, rt: &IntegrationRuntime, hook: Hook) -> Result<()> {
        self.ready()?;
        let (book, ledger) = self.restore(rt)?;
        let (epoch, journal) = Self::make_epoch(&self.root, &book, &ledger, rt)?;
        drop((book, ledger));
        let (mut s, c) = self.control.copy()?;
        s.epoch = epoch;
        self.control.append(s, c, hook)?;
        self.journal = Some(journal);
        Ok(())
    }
    /// Explicit research-only break; no edit is returned/published until this retirement commits.
    pub fn break_retention(
        &mut self,
        rt: &IntegrationRuntime,
        hook: Hook,
    ) -> Result<(Workbook, ColdLedger)> {
        self.ready()?;
        if self.control.state.intent.is_some() {
            return Err(bad());
        }
        let (book, ledger) = self.restore(rt)?;
        let (mut state, charge) = self.control.copy()?;
        state.disabled = true;
        self.control.append(state, charge, hook)?;
        Ok((book, ledger))
    }
    pub fn set_policy(&mut self, policy: Policy) -> Result<()> {
        self.ready()?;
        let (mut s, c) = self.control.copy()?;
        s.policy = policy;
        self.control.append(s, c, Hook::default())?;
        Ok(())
    }
    pub fn close(&mut self, cancelled: bool, discard: bool, hook: Hook) -> Result<()> {
        self.ready()?;
        if cancelled {
            return Ok(());
        }
        if self.control.state.policy == Policy::SessionOnly
            && !discard
            && self
                .journal
                .as_ref()
                .and_then(|j| j.receipt())
                .is_some_and(|r| {
                    self.control
                        .state
                        .saved
                        .as_ref()
                        .is_none_or(|s| s.content != r.content)
                })
        {
            return Err(Error::Graph(
                "explicit discard or verified Save required".into(),
            ));
        }
        if self.control.state.intent.is_some() {
            return Err(Error::Graph("pending save prevents close".into()));
        }
        let (mut s, c) = self.control.copy()?;
        s.closed = true;
        self.control.append(s, c, hook)?;
        Ok(())
    }
    pub fn reopen_retained(&mut self) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        if self.control.state.policy != Policy::Retained {
            return Err(bad());
        }
        let (mut s, c) = self.control.copy()?;
        s.closed = false;
        self.control.append(s, c, Hook::default())?;
        Ok(())
    }
    pub fn relocate(
        mut self,
        destination: &Path,
        rt: &IntegrationRuntime,
        hook: Hook,
    ) -> Result<Self> {
        self.ready()?;
        if self.control.state.intent.is_some() || destination.exists() {
            return Err(bad());
        }
        let parent = destination
            .parent()
            .ok_or_else(bad)?
            .canonicalize()
            .map_err(io)?;
        let destination = parent.join(destination.file_name().ok_or_else(bad)?);
        let (mut s, c) = self.control.copy()?;
        s.relocating = Some(destination.clone());
        self.control.append(s, c, Hook::default())?;
        hook.hit(Point::BeforeMove)?;
        let old = self.root.clone();
        drop(self);
        fs::rename(&old, &destination).map_err(io)?;
        sync_parent(old.parent().ok_or_else(bad)?)?;
        sync_parent(&parent)?;
        hook.hit(Point::AfterMove)?;
        let mut opened = Self::open(&destination, rt)?;
        opened.reconcile_relocation()?;
        Ok(opened)
    }
    pub fn reconcile_relocation(&mut self) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        let dest = self.control.state.relocating.as_ref().ok_or_else(bad)?;
        if dest != &self.root {
            return Err(Error::Graph(
                "relocation unresolved at source; explicit cancel required".into(),
            ));
        }
        let (mut s, c) = self.control.copy()?;
        s.relocating = None;
        self.control.append(s, c, Hook::default())?;
        Ok(())
    }
    pub fn cancel_relocation(&mut self) -> Result<()> {
        self.namespace()?;
        self.control.ready()?;
        let dest = self.control.state.relocating.as_ref().ok_or_else(bad)?;
        if dest.exists() || dest == &self.root {
            return Err(bad());
        }
        let (mut s, c) = self.control.copy()?;
        s.relocating = None;
        self.control.append(s, c, Hook::default())?;
        Ok(())
    }
}

pub fn open_saved(path: &Path, rt: &IntegrationRuntime) -> Result<(Workbook, Uuid)> {
    let (bytes, _) = native::read(path).map_err(io)?;
    Document::open_document(&bytes, Limits::default(), rt)
}
