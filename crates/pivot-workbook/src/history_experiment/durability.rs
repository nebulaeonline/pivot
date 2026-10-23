//! Stage 3 single-file decision research. Not a production history/container contract.
use super::{ColdLedger, representation::Records};
use crate::*;
use pivot_core::model::integration::{
    IntegrationRuntime,
    history_codec::{Blob, MAX_BYTES},
    resources::{Charge, Memory},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    time::Instant,
};

const FILE_HEADER: u64 = 128;
const PREPARE: usize = 256;
const DECISION: usize = 64;
fn bad() -> Error {
    Error::Graph("invalid Stage 3 history journal; quarantine required".into())
}
fn io(e: std::io::Error) -> Error {
    Error::Graph(format!("Stage 3 I/O: {e}"))
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn number(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn put(bytes: &mut [u8], at: usize, value: u64) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
fn content(bytes: &[u8]) -> Result<Uuid> {
    if bytes.len() < 136
        || &bytes[..8] != b"PIVHWB01"
        || hash(&bytes[..bytes.len() - 32]) != bytes[bytes.len() - 32..]
    {
        return Err(bad());
    }
    Ok(Uuid::from_bytes(bytes[88..104].try_into().unwrap()))
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub interval: u64,
    pub delta_bytes: u64,
    pub delta_records: u64,
    pub full: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            interval: 16,
            delta_bytes: 64 * 1024,
            delta_records: 256,
            full: false,
        }
    }
}
impl Config {
    fn valid(self) -> bool {
        self.interval > 0 && self.interval <= 64 && self.delta_bytes > 0 && self.delta_records > 0
    }
}
/// Authoritative record-construction seam. Full export is deliberately isolated here.
pub struct Candidate {
    capsule: Blob,
    content: Uuid,
    pub export_ns: u128,
}
impl Candidate {
    pub fn bytes(&self) -> &[u8] {
        &self.capsule.bytes
    }
    pub fn capture(book: &Workbook, ledger: &ColdLedger) -> Result<Self> {
        let at = Instant::now();
        let capsule = book.history_cold_export(book.expected(), ledger)?;
        Ok(Self {
            capsule,
            content: book.history_content_id(),
            export_ns: at.elapsed().as_nanos(),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Point {
    BeforePrepare,
    PartialPrepare,
    AfterPrepare,
    PartialPayload,
    AfterPayload,
    BeforePayloadSync,
    AfterPayloadSync,
    PartialDecision,
    AfterDecisionWrite,
    BeforeDecisionSync,
    AfterDecisionSync,
    BeforeAcknowledge,
}
impl Point {
    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::BeforePrepare,
            Self::PartialPrepare,
            Self::AfterPrepare,
            Self::PartialPayload,
            Self::AfterPayload,
            Self::BeforePayloadSync,
            Self::AfterPayloadSync,
            Self::PartialDecision,
            Self::AfterDecisionWrite,
            Self::BeforeDecisionSync,
            Self::AfterDecisionSync,
            Self::BeforeAcknowledge,
        ]
        .into_iter()
        .find(|p| format!("{p:?}") == s)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Error,
    Pause,
}
#[derive(Default, Clone, Debug)]
pub struct Fault {
    pub at: Option<(Point, Action)>,
    pub write_budget: Option<usize>,
}
impl Fault {
    fn hit(&self, point: Point) -> Result<()> {
        if let Some((p, action)) = self.at
            && p == point
        {
            match action {
                Action::Error => {
                    return Err(io(std::io::Error::other(format!("injected {point:?}"))));
                }
                Action::Pause => {
                    println!("STAGE3_READY {point:?}");
                    std::io::stdout().flush().map_err(io)?;
                    loop {
                        std::thread::park();
                    }
                }
            }
        }
        Ok(())
    }
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> Result<()> {
        if let Some(remaining) = self.write_budget.as_mut() {
            let n = (*remaining).min(bytes.len());
            file.write_all(&bytes[..n]).map_err(io)?;
            *remaining -= n;
            if n != bytes.len() {
                return Err(io(std::io::Error::from(std::io::ErrorKind::StorageFull)));
            }
        } else {
            file.write_all(bytes).map_err(io)?;
        }
        Ok(())
    }
}
#[derive(Default, Debug)]
pub struct Metrics {
    pub validation_ns: u128,
    pub construction_ns: u128,
    pub write_ns: u128,
    pub sync_ns: u128,
    pub reconcile_ns: u128,
    pub payload_bytes: u64,
    pub decisions: u64,
    pub forced_checkpoints: u64,
    pub scanned_records: u64,
}
#[derive(Clone, Debug)]
struct Record {
    offset: u64,
    seq: u64,
    id: Uuid,
    kind: u64,
    previous: u64,
    previous_hash: [u8; 32],
    len: u64,
    payload_hash: [u8; 32],
    content: Uuid,
    parent: u64,
    head: u64,
    cursor: u64,
    ledger: u64,
    base: u64,
    depth: u64,
    encoding: u64,
    redo: u64,
    redo_next: u64,
    work_bytes: u64,
    work_records: u64,
    digest: [u8; 32],
}
impl Record {
    fn encode(&self) -> [u8; PREPARE] {
        let mut b = [0; PREPARE];
        b[..8].copy_from_slice(b"PVH3PRE1");
        put(&mut b, 8, self.seq);
        b[16..32].copy_from_slice(self.id.as_bytes());
        put(&mut b, 32, self.kind);
        put(&mut b, 40, self.previous);
        b[48..80].copy_from_slice(&self.previous_hash);
        put(&mut b, 80, self.len);
        b[88..120].copy_from_slice(&self.payload_hash);
        b[120..136].copy_from_slice(self.content.as_bytes());
        for (at, n) in [
            (136, self.parent),
            (144, self.head),
            (152, self.cursor),
            (160, self.ledger),
            (168, self.base),
            (176, self.depth),
            (184, self.encoding),
            (192, self.redo),
            (200, self.redo_next),
            (208, self.work_bytes),
            (216, self.work_records),
        ] {
            put(&mut b, at, n);
        }
        let digest = hash(&b[..224]);
        b[224..].copy_from_slice(&digest);
        b
    }
    fn decode(offset: u64, b: [u8; PREPARE]) -> Result<Self> {
        if &b[..8] != b"PVH3PRE1"
            || hash(&b[..224]) != b[224..]
            || number(&b, 80) > MAX_BYTES as u64
        {
            return Err(bad());
        }
        Ok(Self {
            offset,
            seq: number(&b, 8),
            id: Uuid::from_bytes(b[16..32].try_into().unwrap()),
            kind: number(&b, 32),
            previous: number(&b, 40),
            previous_hash: b[48..80].try_into().unwrap(),
            len: number(&b, 80),
            payload_hash: b[88..120].try_into().unwrap(),
            content: Uuid::from_bytes(b[120..136].try_into().unwrap()),
            parent: number(&b, 136),
            head: number(&b, 144),
            cursor: number(&b, 152),
            ledger: number(&b, 160),
            base: number(&b, 168),
            depth: number(&b, 176),
            encoding: number(&b, 184),
            redo: number(&b, 192),
            redo_next: number(&b, 200),
            work_bytes: number(&b, 208),
            work_records: number(&b, 216),
            digest: b[224..].try_into().unwrap(),
        })
    }
    fn end(&self) -> Result<u64> {
        self.offset
            .checked_add(PREPARE as u64)
            .and_then(|x| x.checked_add(self.len))
            .and_then(|x| x.checked_add(DECISION as u64))
            .ok_or_else(bad)
    }
    fn decision(&self) -> [u8; DECISION] {
        let mut b = [0; DECISION];
        b[..8].copy_from_slice(b"PVH3YES1");
        put(&mut b, 8, self.offset);
        b[16..48].copy_from_slice(&self.digest);
        b
    }
}
#[derive(Clone, Debug)]
pub struct Receipt {
    pub transaction: Uuid,
    pub content: Uuid,
    pub sequence: u64,
    pub head: u64,
    pub cursor: u64,
    pub ledger: u64,
    pub redo: u64,
}
impl From<&Record> for Receipt {
    fn from(r: &Record) -> Self {
        Self {
            transaction: r.id,
            content: r.content,
            sequence: r.seq,
            head: r.head,
            cursor: r.cursor,
            ledger: r.ledger,
            redo: r.redo,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Outcome {
    Committed(Receipt),
    Uncommitted(Uuid),
}

/// One exclusive writer; bounded resident state. Startup/reconciliation scan disk, not RAM catalogs.
pub struct Journal {
    file: File,
    config: Config,
    memory: Memory,
    last: Option<Record>,
    end: u64,
    previous: Option<Records>,
    pending: Option<Uuid>,
    _charge: Charge,
    pub metrics: Metrics,
}
impl Journal {
    pub fn create(path: &Path, config: Config, runtime: &IntegrationRuntime) -> Result<Self> {
        if !config.valid() {
            return Err(bad());
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(io)?;
        file.try_lock()
            .map_err(|e| Error::Graph(format!("journal lock: {e}")))?;
        let mut b = [0; 128];
        b[..8].copy_from_slice(b"PVH3LOG1");
        b[8..24].copy_from_slice(Uuid::new_v4().as_bytes());
        for (at, n) in [
            (24, config.interval),
            (32, config.delta_bytes),
            (40, config.delta_records),
            (48, u64::from(config.full)),
        ] {
            put(&mut b, at, n);
        }
        let digest = hash(&b[..96]);
        b[96..].copy_from_slice(&digest);
        file.write_all(&b).map_err(io)?;
        file.sync_all().map_err(io)?;
        // No rename/rotation after creation. Windows namespace durability is explicitly unproven.
        #[cfg(unix)]
        File::open(path.parent().ok_or_else(bad)?)
            .map_err(io)?
            .sync_all()
            .map_err(io)?;
        let memory = runtime.memory();
        let charge = memory.reserve(4096).map_err(|_| Error::Limit)?;
        Ok(Self {
            file,
            config,
            memory,
            last: None,
            end: FILE_HEADER,
            previous: None,
            pending: None,
            _charge: charge,
            metrics: Metrics::default(),
        })
    }
    pub fn open(path: &Path, runtime: &IntegrationRuntime) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(io)?;
        file.try_lock()
            .map_err(|e| Error::Graph(format!("journal lock: {e}")))?;
        let mut b = [0; 128];
        file.read_exact(&mut b).map_err(io)?;
        if &b[..8] != b"PVH3LOG1"
            || hash(&b[..96]) != b[96..]
            || b[56..96].iter().any(|n| *n != 0)
            || number(&b, 48) > 1
        {
            return Err(bad());
        }
        let config = Config {
            interval: number(&b, 24),
            delta_bytes: number(&b, 32),
            delta_records: number(&b, 40),
            full: number(&b, 48) == 1,
        };
        if !config.valid() {
            return Err(bad());
        }
        let memory = runtime.memory();
        let charge = memory.reserve(4096).map_err(|_| Error::Limit)?;
        let mut journal = Self {
            file,
            config,
            memory,
            last: None,
            end: FILE_HEADER,
            previous: None,
            pending: None,
            _charge: charge,
            metrics: Metrics::default(),
        };
        journal.recover(runtime)?;
        Ok(journal)
    }
    fn read_record(&mut self, offset: u64) -> Result<Record> {
        if offset < FILE_HEADER {
            return Err(bad());
        }
        self.file.seek(SeekFrom::Start(offset)).map_err(io)?;
        let mut b = [0; PREPARE];
        self.file.read_exact(&mut b).map_err(io)?;
        Record::decode(offset, b)
    }
    fn payload(&mut self, r: &Record) -> Result<(Vec<u8>, Charge)> {
        let n = usize::try_from(r.len).map_err(|_| bad())?;
        if n > MAX_BYTES {
            return Err(bad());
        }
        let charge = self.memory.reserve(n + 1024).map_err(|_| Error::Limit)?;
        let mut b = vec![0; n];
        self.file
            .seek(SeekFrom::Start(r.offset + PREPARE as u64))
            .map_err(io)?;
        self.file.read_exact(&mut b).map_err(io)?;
        if hash(&b) != r.payload_hash {
            return Err(bad());
        }
        Ok((b, charge))
    }
    fn load(&mut self, offset: u64) -> Result<Blob> {
        let _scratch = self
            .memory
            .reserve(64 * std::mem::size_of::<Record>() + 4096)
            .map_err(|_| Error::Limit)?;
        let target = self.read_record(offset)?;
        if target.kind != 1 || target.depth >= self.config.interval {
            return Err(bad());
        }
        let mut chain = Vec::with_capacity(64);
        let mut at = target.clone();
        while at.encoding == 2 {
            if at.depth == 0 || chain.len() >= 63 || at.base >= at.offset {
                return Err(bad());
            }
            let previous = self.read_record(at.base)?;
            if previous.kind != 1 || previous.depth + 1 != at.depth {
                return Err(bad());
            }
            chain.push(at);
            at = previous;
        }
        if at.encoding != 1 || at.depth != 0 || at.base != at.offset {
            return Err(bad());
        }
        let (b, c) = self.payload(&at)?;
        let mut records = Records::decode(&b, &self.memory)?;
        drop((b, c));
        let mut bytes = 0u64;
        let mut work = 0u64;
        for r in chain.iter().rev() {
            let (b, c) = self.payload(r)?;
            if b.len() < 8 {
                return Err(bad());
            }
            bytes = bytes.checked_add(r.len).ok_or_else(bad)?;
            work = work.checked_add(number(&b, 0)).ok_or_else(bad)?;
            if bytes > self.config.delta_bytes
                || work > self.config.delta_records
                || r.work_bytes != bytes
                || r.work_records != work
            {
                return Err(bad());
            }
            records.apply_delta(&b, &self.memory)?;
            drop((b, c));
        }
        let blob = records.assemble(&self.memory)?;
        if content(&blob.bytes)? != target.content {
            return Err(bad());
        }
        Ok(blob)
    }
    fn validate_transition(&mut self, r: &Record, last: Option<&Record>) -> Result<()> {
        if r.seq
            != last
                .map_or(Some(1), |x| x.seq.checked_add(1))
                .ok_or_else(bad)?
            || r.previous != last.map_or(0, |x| x.offset)
            || r.previous_hash != last.map_or([0; 32], |x| x.digest)
            || r.id.is_nil()
        {
            return Err(bad());
        }
        match r.kind {
            1 => {
                if r.parent != last.map_or(0, |x| x.cursor)
                    || r.head != r.offset
                    || r.cursor != r.offset
                    || r.ledger != r.offset
                    || r.redo != 0
                    || r.redo_next != 0
                {
                    return Err(bad());
                }
                if r.encoding == 1 {
                    if r.base != r.offset
                        || r.depth != 0
                        || r.work_bytes != 0
                        || r.work_records != 0
                    {
                        return Err(bad());
                    }
                } else if r.encoding == 2 {
                    let l = last.ok_or_else(bad)?;
                    let base = self.read_record(l.ledger)?;
                    if self.config.full
                        || r.base != l.ledger
                        || r.depth != base.depth + 1
                        || r.depth >= self.config.interval
                        || r.work_bytes > self.config.delta_bytes
                        || r.work_records > self.config.delta_records
                    {
                        return Err(bad());
                    }
                } else {
                    return Err(bad());
                }
            }
            2 | 3 => {
                let l = last.ok_or_else(bad)?;
                if r.len != 0
                    || r.encoding != 0
                    || r.base != 0
                    || r.depth != 0
                    || r.work_bytes != 0
                    || r.work_records != 0
                    || r.head != l.head
                    || r.ledger != l.ledger
                    || r.parent != l.cursor
                {
                    return Err(bad());
                }
                let target = if r.kind == 2 {
                    let current = self.read_record(l.cursor)?;
                    if current.parent == 0 || r.redo != r.offset || r.redo_next != l.redo {
                        return Err(bad());
                    }
                    current.parent
                } else {
                    if l.redo == 0 {
                        return Err(bad());
                    }
                    let undo = self.read_record(l.redo)?;
                    if undo.kind != 2
                        || undo.cursor != l.cursor
                        || r.redo != undo.redo_next
                        || r.redo_next != 0
                    {
                        return Err(bad());
                    }
                    undo.parent
                };
                if r.cursor != target || self.read_record(target)?.content != r.content {
                    return Err(bad());
                }
            }
            _ => return Err(bad()),
        }
        Ok(())
    }
    fn recover(&mut self, runtime: &IntegrationRuntime) -> Result<()> {
        let size = self.file.metadata().map_err(io)?.len();
        let mut offset = FILE_HEADER;
        let mut last = None;
        while offset < size {
            if size - offset < PREPARE as u64 {
                break;
            }
            let r = self.read_record(offset)?;
            let end = r.end()?;
            if end > size {
                break;
            }
            self.file
                .seek(SeekFrom::Start(end - DECISION as u64))
                .map_err(io)?;
            let mut decision = [0; DECISION];
            self.file.read_exact(&mut decision).map_err(io)?;
            if decision != r.decision() {
                return Err(bad());
            }
            self.payload(&r)?;
            self.validate_transition(&r, last.as_ref())?;
            last = Some(r);
            offset = end;
            self.metrics.scanned_records += 1;
        }
        // Verify reconstructed payloads before changing/truncating any file or enabling mutation.
        self.last = last;
        self.end = offset;
        if let Some(r) = self.last.clone() {
            let authority = self.load(r.ledger)?;
            let current = self.load(r.cursor)?;
            let head = self.load(r.head)?;
            if content(&current.bytes)? != r.content
                || authority.bytes[8..88] != current.bytes[8..88]
                || head.bytes[8..88] != current.bytes[8..88]
            {
                return Err(bad());
            }
            let (book, ledger) = Workbook::history_cold_reopen(
                &authority.bytes,
                &current.bytes,
                Limits::default(),
                runtime,
            )?;
            drop((book, ledger));
            self.previous = Some(Records::from_blob(&authority, &self.memory)?);
        }
        if offset != size {
            self.file.set_len(offset).map_err(io)?;
        }
        // Establish durability before classifying a complete-but-previously-unsynced decision.
        self.file.sync_all().map_err(io)?;
        Ok(())
    }
    pub fn receipt(&self) -> Option<Receipt> {
        if self.pending.is_some() {
            return None;
        }
        self.last.as_ref().map(Receipt::from)
    }
    pub fn current_bytes(&mut self) -> Result<Blob> {
        if self.pending.is_some() {
            return Err(Error::Graph("reconciliation required".into()));
        }
        let r = self.last.clone().ok_or_else(bad)?;
        self.load(r.cursor)
    }
    pub fn authority_bytes(&mut self) -> Result<Blob> {
        if self.pending.is_some() {
            return Err(Error::Graph("reconciliation required".into()));
        }
        let r = self.last.clone().ok_or_else(bad)?;
        self.load(r.ledger)
    }
    pub fn pending(&self) -> Option<Uuid> {
        self.pending
    }
    pub fn restore(&mut self, runtime: &IntegrationRuntime) -> Result<(Workbook, ColdLedger)> {
        if self.pending.is_some() {
            return Err(Error::Graph("reconciliation required".into()));
        }
        let r = self.last.clone().ok_or_else(bad)?;
        let authority = self.load(r.ledger)?;
        let current = self.load(r.cursor)?;
        Workbook::history_cold_reopen(&authority.bytes, &current.bytes, Limits::default(), runtime)
    }
    fn lookup_committed(&mut self, id: Uuid) -> Result<Outcome> {
        let mut offset = FILE_HEADER;
        while offset < self.end {
            let r = self.read_record(offset)?;
            if r.id == id {
                return Ok(Outcome::Committed(Receipt::from(&r)));
            }
            offset = r.end()?;
        }
        Ok(Outcome::Uncommitted(id))
    }
    pub fn lookup(&mut self, id: Uuid) -> Result<Outcome> {
        if self.pending.is_some() {
            return Err(Error::Graph("reconciliation required".into()));
        }
        self.lookup_committed(id)
    }
    pub fn reconcile(&mut self, id: Uuid, runtime: &IntegrationRuntime) -> Result<Outcome> {
        if self.pending != Some(id) {
            return Err(Error::Graph("wrong reconciliation TransactionId".into()));
        }
        let at = Instant::now();
        self.recover(runtime)?;
        let outcome = self.lookup_committed(id)?;
        self.pending = None;
        self.metrics.reconcile_ns += at.elapsed().as_nanos();
        Ok(outcome)
    }
    fn start(&mut self, id: Uuid, kind: u64) -> Result<Record> {
        if self.pending.is_some() {
            return Err(Error::Graph(
                "reconciliation required; mutations blocked".into(),
            ));
        }
        if id.is_nil() || matches!(self.lookup(id)?, Outcome::Committed(_)) {
            return Err(Error::Graph("duplicate/invalid TransactionId".into()));
        }
        let l = self.last.as_ref();
        Ok(Record {
            offset: self.end,
            seq: l
                .map_or(Some(1), |r| r.seq.checked_add(1))
                .ok_or_else(bad)?,
            id,
            kind,
            previous: l.map_or(0, |r| r.offset),
            previous_hash: l.map_or([0; 32], |r| r.digest),
            len: 0,
            payload_hash: hash(&[]),
            content: Uuid::nil(),
            parent: l.map_or(0, |r| r.cursor),
            head: l.map_or(0, |r| r.head),
            cursor: l.map_or(0, |r| r.cursor),
            ledger: l.map_or(0, |r| r.ledger),
            base: 0,
            depth: 0,
            encoding: 0,
            redo: 0,
            redo_next: 0,
            work_bytes: 0,
            work_records: 0,
            digest: [0; 32],
        })
    }
    /// Persist one canonical candidate. Caller may publish/acknowledge only after success.
    pub fn commit(
        &mut self,
        id: Uuid,
        candidate: &Candidate,
        runtime: &IntegrationRuntime,
        fault: &mut Fault,
    ) -> Result<Receipt> {
        let mut r = self.start(id, 1)?;
        if self
            .last
            .as_ref()
            .is_some_and(|l| l.content == candidate.content)
        {
            return Err(Error::Graph(
                "unchanged content is not a history transaction".into(),
            ));
        }
        let at = Instant::now();
        let (book, ledger) = Workbook::history_cold_reopen(
            &candidate.capsule.bytes,
            &candidate.capsule.bytes,
            Limits::default(),
            runtime,
        )?;
        drop((book, ledger));
        if let Some(last) = self.last.clone() {
            let authority = self.load(last.ledger)?;
            if candidate.capsule.bytes[8..88] != authority.bytes[8..88] {
                return Err(bad());
            }
            let (book, ledger) = Workbook::history_cold_reopen(
                &candidate.capsule.bytes,
                &authority.bytes,
                Limits::default(),
                runtime,
            )?;
            drop((book, ledger));
        }
        self.metrics.validation_ns += at.elapsed().as_nanos();
        let at = Instant::now();
        let records = Records::from_blob(&candidate.capsule, &self.memory)?;
        let mut payload = None;
        if !self.config.full
            && let Some(last) = self.last.clone()
        {
            let base = self.read_record(last.ledger)?;
            let previous = self.previous.as_ref().ok_or_else(bad)?;
            if base.depth + 1 < self.config.interval {
                let (delta, count) = records.delta(previous, &self.memory)?;
                let bytes = base
                    .work_bytes
                    .checked_add(delta.bytes.len() as u64)
                    .ok_or_else(bad)?;
                let work = base
                    .work_records
                    .checked_add(count as u64)
                    .ok_or_else(bad)?;
                if bytes <= self.config.delta_bytes && work <= self.config.delta_records {
                    r.base = base.offset;
                    r.depth = base.depth + 1;
                    r.encoding = 2;
                    r.work_bytes = bytes;
                    r.work_records = work;
                    payload = Some(delta);
                } else {
                    self.metrics.forced_checkpoints += 1;
                }
            }
        }
        let payload = match payload {
            Some(x) => x,
            None => {
                r.base = r.offset;
                r.encoding = 1;
                records.encode(&self.memory)?
            }
        };
        r.content = candidate.content;
        r.head = r.offset;
        r.cursor = r.offset;
        r.ledger = r.offset;
        r.len = payload.bytes.len() as u64;
        r.payload_hash = hash(&payload.bytes);
        self.metrics.construction_ns += at.elapsed().as_nanos();
        let receipt = self.persist(r, &payload.bytes, fault)?;
        self.previous = Some(records);
        Ok(receipt)
    }
    pub fn undo(&mut self, id: Uuid, fault: &mut Fault) -> Result<Receipt> {
        let mut r = self.start(id, 2)?;
        let current = self.read_record(r.parent)?;
        if current.parent == 0 {
            return Err(Error::InvalidTarget);
        }
        let target = self.read_record(current.parent)?;
        r.cursor = target.offset;
        r.content = target.content;
        r.redo = r.offset;
        r.redo_next = self.last.as_ref().ok_or_else(bad)?.redo;
        self.persist(r, &[], fault)
    }
    pub fn redo(&mut self, id: Uuid, fault: &mut Fault) -> Result<Receipt> {
        let mut r = self.start(id, 3)?;
        let redo = self.last.as_ref().ok_or_else(bad)?.redo;
        if redo == 0 {
            return Err(Error::InvalidTarget);
        }
        let undo = self.read_record(redo)?;
        let target = self.read_record(undo.parent)?;
        r.cursor = target.offset;
        r.content = target.content;
        r.redo = undo.redo_next;
        self.persist(r, &[], fault)
    }
    fn persist(&mut self, mut r: Record, bytes: &[u8], fault: &mut Fault) -> Result<Receipt> {
        self.validate_transition(&r, self.last.clone().as_ref())?;
        fault.hit(Point::BeforePrepare)?;
        let header = r.encode();
        r.digest = header[224..].try_into().unwrap();
        self.pending = Some(r.id);
        self.file.seek(SeekFrom::Start(self.end)).map_err(io)?;
        let at = Instant::now();
        fault.write(&mut self.file, &header[..128])?;
        fault.hit(Point::PartialPrepare)?;
        fault.write(&mut self.file, &header[128..])?;
        fault.hit(Point::AfterPrepare)?;
        let middle = bytes.len() / 2;
        fault.write(&mut self.file, &bytes[..middle])?;
        fault.hit(Point::PartialPayload)?;
        fault.write(&mut self.file, &bytes[middle..])?;
        fault.hit(Point::AfterPayload)?;
        self.metrics.write_ns += at.elapsed().as_nanos();
        fault.hit(Point::BeforePayloadSync)?;
        let at = Instant::now();
        self.file.sync_all().map_err(io)?;
        self.metrics.sync_ns += at.elapsed().as_nanos();
        fault.hit(Point::AfterPayloadSync)?;
        let decision = r.decision();
        let at = Instant::now();
        fault.write(&mut self.file, &decision[..32])?;
        fault.hit(Point::PartialDecision)?;
        fault.write(&mut self.file, &decision[32..])?;
        self.metrics.write_ns += at.elapsed().as_nanos();
        fault.hit(Point::AfterDecisionWrite)?;
        fault.hit(Point::BeforeDecisionSync)?;
        let at = Instant::now();
        self.file.sync_all().map_err(io)?;
        self.metrics.sync_ns += at.elapsed().as_nanos();
        fault.hit(Point::AfterDecisionSync)?;
        fault.hit(Point::BeforeAcknowledge)?;
        self.end = r.end()?;
        self.metrics.payload_bytes += r.len;
        self.metrics.decisions += 1;
        let receipt = Receipt::from(&r);
        self.last = Some(r);
        self.pending = None;
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    fn directory() -> PathBuf {
        let p = std::env::temp_dir().join(format!("pivot-stage3-{}", Uuid::new_v4()));
        fs::create_dir(&p).unwrap();
        p
    }
    fn cleanup(dir: PathBuf, rt: IntegrationRuntime) {
        rt.flush();
        let m = rt.memory();
        assert!(rt.finish().is_ok());
        assert_eq!(m.stats().charged, 0);
        let dir = dir.canonicalize().unwrap();
        assert!(dir.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(
            dir.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("pivot-stage3-")
        );
        fs::remove_dir_all(dir).unwrap();
    }
    fn edit(book: &mut Workbook, text: &str) {
        book.apply(
            book.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 1),
                source: text.into(),
            }],
        )
        .unwrap();
    }

    #[test]
    fn uncertain_decision_blocks_all_mutations_and_failed_reconciliation_stays_blocked() {
        let rt = IntegrationRuntime::standalone().unwrap();
        let dir = directory();
        let path = dir.join("journal");
        let mut j = Journal::create(&path, Config::default(), &rt).unwrap();
        let mut book = Workbook::new_in("journal", Limits::default(), &rt).unwrap();
        edit(&mut book, "10");
        let mut ledger = ColdLedger::new(&book).unwrap();
        let a = Candidate::capture(&book, &ledger).unwrap();
        j.commit(Uuid::new_v4(), &a, &rt, &mut Fault::default())
            .unwrap();
        let end = fs::metadata(&path).unwrap().len();
        edit(&mut book, "20");
        ledger.observe(&book).unwrap();
        let b = Candidate::capture(&book, &ledger).unwrap();
        let op = Uuid::new_v4();
        let mut fault = Fault {
            at: Some((Point::AfterDecisionWrite, Action::Error)),
            ..Fault::default()
        };
        assert!(j.commit(op, &b, &rt, &mut fault).is_err());
        assert_eq!(j.pending(), Some(op));
        assert!(j.receipt().is_none());
        assert!(j.current_bytes().is_err());
        assert!(j.authority_bytes().is_err());
        let size = fs::metadata(&path).unwrap().len();
        assert!(j.lookup(op).is_err());
        assert!(j.restore(&rt).is_err());
        assert!(
            j.commit(Uuid::new_v4(), &b, &rt, &mut Fault::default())
                .is_err()
        );
        assert!(j.undo(Uuid::new_v4(), &mut Fault::default()).is_err());
        assert!(j.redo(Uuid::new_v4(), &mut Fault::default()).is_err());
        assert!(j.reconcile(Uuid::new_v4(), &rt).is_err());
        assert_eq!(fs::metadata(&path).unwrap().len(), size);
        // Readable decision + damaged referenced payload cannot be reconciled as committed/uncommitted.
        j.file.seek(SeekFrom::Start(end + 256)).unwrap();
        j.file.write_all(&[255]).unwrap();
        j.file.sync_all().unwrap();
        assert!(j.reconcile(op, &rt).is_err());
        assert_eq!(j.pending(), Some(op));
        assert!(j.receipt().is_none());
        assert!(j.current_bytes().is_err());
        assert!(j.authority_bytes().is_err());
        assert!(j.undo(Uuid::new_v4(), &mut Fault::default()).is_err());
        drop(j);
        assert!(Journal::open(&path, &rt).is_err());
        drop((book, ledger, a, b));
        cleanup(dir, rt);
    }
}
