//! Bounded Stage 2 research stores. Not a durable transaction coordinator/container.
use super::*;
use pivot_core::model::integration::{
    history_codec::{Blob, Reader, RecordKey, Writer},
    resources::Memory,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::Instant,
};
const INDEX_BYTES: u64 = 96;
const PAGE: usize = 64;
fn bad() -> Error {
    Error::Graph("invalid representation experiment record".into())
}
fn io(e: std::io::Error) -> Error {
    Error::Graph(format!("representation I/O: {e}"))
}
fn hash(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|n| format!("{n:02x}")).collect()
}
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Full,
    Delta { interval: u64 },
    Objects,
}
#[derive(Default, Clone, Debug)]
pub struct Metrics {
    pub checkpoints: u64,
    pub forced_checkpoints: u64,
    pub payload_bytes: u64,
    pub object_bytes: u64,
    pub journal_bytes: u64,
    pub object_files: u64,
    pub compared_records: u64,
    pub changed_records: u64,
    pub write_ns: u128,
    pub record_plan_ns: u128,
    pub object_io_ns: u128,
    pub data_root_io_ns: u128,
    pub sync_ns: u128,
    pub records_capacity: usize,
    pub pages_cached: usize,
}
#[derive(Default, Debug)]
pub struct ReadMetrics {
    pub bytes: u64,
    pub file_reads: u64,
    pub records: usize,
    pub chain_steps: u64,
}
struct Buffer {
    bytes: Vec<u8>,
    _charge: Charge,
}
fn read_file(path: &Path, memory: &Memory) -> Result<Buffer> {
    let mut f = File::open(path).map_err(io)?;
    read_buffer(&mut f, 0, fsize(path)?, memory)
}
fn fsize(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path).map_err(io)?.len())
}
fn read_buffer(f: &mut File, offset: u64, len: u64, memory: &Memory) -> Result<Buffer> {
    let len = usize::try_from(len).map_err(|_| bad())?;
    if len > pivot_core::model::integration::history_codec::MAX_BYTES {
        return Err(bad());
    }
    let charge = memory.reserve(len + 1024).map_err(|_| Error::Limit)?;
    let mut bytes = vec![0; len];
    f.seek(SeekFrom::Start(offset)).map_err(io)?;
    f.read_exact(&mut bytes).map_err(io)?;
    Ok(Buffer {
        bytes,
        _charge: charge,
    })
}
struct Records {
    map: BTreeMap<RecordKey, Vec<u8>>,
    bytes: usize,
    _charge: Charge,
}
impl Records {
    fn from_blob(blob: &Blob, memory: &Memory) -> Result<Self> {
        let charge = memory
            .reserve(
                blob.bytes
                    .len()
                    .saturating_mul(2)
                    .saturating_add(blob.records.len() * 256)
                    .saturating_add(4096),
            )
            .map_err(|_| Error::Limit)?;
        let mut map = BTreeMap::new();
        let mut at = 0;
        let mut previous = None;
        for span in &blob.records {
            if span.range.start != at
                || span.range.end > blob.bytes.len()
                || previous.is_some_and(|k| k >= span.key)
            {
                return Err(bad());
            }
            let data = blob.bytes[span.range.clone()].to_vec();
            at = span.range.end;
            previous = Some(span.key);
            if map.insert(span.key, data).is_some() {
                return Err(bad());
            }
        }
        if at != blob.bytes.len() {
            return Err(bad());
        }
        Ok(Self {
            map,
            bytes: at,
            _charge: charge,
        })
    }
    fn encode(&self, memory: &Memory) -> Result<Blob> {
        let mut w = Writer::new(memory).map_err(graph_error)?;
        w.n(self.map.len() as u64);
        for (k, v) in &self.map {
            w.raw(&k.0);
            w.bytes(v);
        }
        w.finish().map_err(graph_error)
    }
    fn decode(bytes: &[u8], memory: &Memory) -> Result<Self> {
        let mut r = Reader::new(bytes).map_err(graph_error)?;
        let count = r.count(65536).map_err(graph_error)?;
        let charge = memory
            .reserve(
                bytes
                    .len()
                    .saturating_mul(2)
                    .saturating_add(count * 256)
                    .saturating_add(4096),
            )
            .map_err(|_| Error::Limit)?;
        let mut map = BTreeMap::new();
        let mut total = 0;
        let mut previous = None;
        for _ in 0..count {
            let k = RecordKey(r.take(33).map_err(graph_error)?.try_into().unwrap());
            if previous.is_some_and(|p| p >= k) {
                return Err(bad());
            }
            previous = Some(k);
            let data = r
                .bytes(pivot_core::model::integration::history_codec::MAX_BYTES)
                .map_err(graph_error)?
                .to_vec();
            total += data.len();
            if total > pivot_core::model::integration::history_codec::MAX_BYTES {
                return Err(bad());
            }
            map.insert(k, data);
        }
        r.end().map_err(graph_error)?;
        Ok(Self {
            map,
            bytes: total,
            _charge: charge,
        })
    }
    fn delta(&self, old: &Self, memory: &Memory) -> Result<(Blob, usize)> {
        let mut changes = 0;
        for (k, v) in &self.map {
            if old.map.get(k) != Some(v) {
                changes += 1;
            }
        }
        for k in old.map.keys() {
            if !self.map.contains_key(k) {
                changes += 1;
            }
        }
        let mut w = Writer::new(memory).map_err(graph_error)?;
        w.n(changes as u64);
        for (k, v) in &self.map {
            if old.map.get(k) != Some(v) {
                w.raw(&k.0);
                w.flag(true);
                w.bytes(v);
            }
        }
        for k in old.map.keys() {
            if !self.map.contains_key(k) {
                w.raw(&k.0);
                w.flag(false);
            }
        }
        Ok((w.finish().map_err(graph_error)?, changes))
    }
    fn apply_delta(&mut self, bytes: &[u8], memory: &Memory) -> Result<usize> {
        let _scratch = memory
            .reserve(bytes.len().saturating_mul(4).saturating_add(4096))
            .map_err(|_| Error::Limit)?;
        let mut r = Reader::new(bytes).map_err(graph_error)?;
        let count = r.count(65536).map_err(graph_error)?;
        let mut changes = Vec::new();
        let mut seen = BTreeSet::new();
        let mut total = self.bytes;
        let mut n = self.map.len();
        for _ in 0..count {
            let key = RecordKey(r.take(33).map_err(graph_error)?.try_into().unwrap());
            if !seen.insert(key) {
                return Err(bad());
            }
            let old = self.map.get(&key);
            if let Some(v) = old {
                total -= v.len();
                n -= 1;
            }
            let value = if r.flag().map_err(graph_error)? {
                let v = r
                    .bytes(pivot_core::model::integration::history_codec::MAX_BYTES)
                    .map_err(graph_error)?
                    .to_vec();
                total = total.checked_add(v.len()).ok_or_else(bad)?;
                n += 1;
                Some(v)
            } else {
                if old.is_none() {
                    return Err(bad());
                }
                None
            };
            changes.push((key, value));
        }
        r.end().map_err(graph_error)?;
        if total > pivot_core::model::integration::history_codec::MAX_BYTES || n > 65536 {
            return Err(bad());
        }
        let charge = memory
            .reserve(
                total
                    .saturating_mul(2)
                    .saturating_add(n * 256)
                    .saturating_add(4096),
            )
            .map_err(|_| Error::Limit)?;
        for (k, v) in changes {
            if let Some(v) = v {
                self.map.insert(k, v);
            } else {
                self.map.remove(&k);
            }
        }
        self.bytes = total;
        self._charge = charge;
        Ok(count)
    }
    fn assemble(&self, memory: &Memory) -> Result<Blob> {
        let mut w = Writer::new(memory).map_err(graph_error)?;
        for (k, v) in &self.map {
            w.boundary(*k);
            w.raw(v);
        }
        w.finish().map_err(graph_error)
    }
    fn capacity(&self) -> usize {
        self.map.values().map(Vec::capacity).sum::<usize>()
            + self.map.len() * std::mem::size_of::<(RecordKey, Vec<u8>)>()
    }
}
struct Entry {
    offset: u64,
    len: u64,
    base: u64,
    expected: [u8; 32],
    payload_hash: [u8; 32],
    kind: u64,
}
impl Entry {
    fn encode(&self) -> [u8; 96] {
        let mut b = [0; 96];
        b[..8].copy_from_slice(&self.offset.to_le_bytes());
        b[8..16].copy_from_slice(&self.len.to_le_bytes());
        b[16..24].copy_from_slice(&self.base.to_le_bytes());
        b[24..56].copy_from_slice(&self.expected);
        b[56..88].copy_from_slice(&self.payload_hash);
        b[88..].copy_from_slice(&self.kind.to_le_bytes());
        b
    }
    fn decode(b: [u8; 96]) -> Self {
        Self {
            offset: u64::from_le_bytes(b[..8].try_into().unwrap()),
            len: u64::from_le_bytes(b[8..16].try_into().unwrap()),
            base: u64::from_le_bytes(b[16..24].try_into().unwrap()),
            expected: b[24..56].try_into().unwrap(),
            payload_hash: b[56..88].try_into().unwrap(),
            kind: u64::from_le_bytes(b[88..].try_into().unwrap()),
        }
    }
}
/// One previous state and one page-hash set are cached, never a transaction catalog.
pub struct Store {
    root: PathBuf,
    data: File,
    index: File,
    memory: Memory,
    mode: Mode,
    count: u64,
    previous: Option<Records>,
    known_pages: BTreeSet<[u8; 32]>,
    _pages_charge: Charge,
    pub metrics: Metrics,
    read_only: bool,
    work_limits: Option<(usize, usize)>,
    replay_bytes: usize,
    replay_records: usize,
    checkpoint: u64,
}
impl Store {
    pub fn create(path: &Path, mode: Mode, memory: Memory) -> Result<Self> {
        if matches!(mode, Mode::Delta { interval: 0 }) {
            return Err(bad());
        }
        fs::create_dir(path).map_err(io)?;
        let data = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.join("data"))
            .map_err(io)?;
        let index = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.join("roots"))
            .map_err(io)?;
        fs::create_dir(path.join("objects")).map_err(io)?;
        let pages = memory.reserve(1024).map_err(|_| Error::Limit)?;
        Ok(Self {
            root: path.to_owned(),
            data,
            index,
            memory,
            mode,
            count: 0,
            previous: None,
            known_pages: BTreeSet::new(),
            _pages_charge: pages,
            metrics: Metrics::default(),
            read_only: false,
            work_limits: None,
            replay_bytes: 0,
            replay_records: 0,
            checkpoint: 0,
        })
    }
    pub fn reopen(path: &Path, mode: Mode, memory: Memory) -> Result<Self> {
        let data = File::open(path.join("data")).map_err(io)?;
        let index = File::open(path.join("roots")).map_err(io)?;
        let bytes = index.metadata().map_err(io)?.len();
        if bytes % INDEX_BYTES != 0 {
            return Err(bad());
        }
        let pages = memory.reserve(1024).map_err(|_| Error::Limit)?;
        Ok(Self {
            root: path.to_owned(),
            data,
            index,
            memory,
            mode,
            count: bytes / INDEX_BYTES,
            previous: None,
            known_pages: BTreeSet::new(),
            _pages_charge: pages,
            metrics: Metrics::default(),
            read_only: true,
            work_limits: None,
            replay_bytes: 0,
            replay_records: 0,
            checkpoint: 0,
        })
    }
    /// Research-only cumulative delta limits, excluding the independently bounded checkpoint.
    pub fn set_replay_work_limits(&mut self, bytes: usize, records: usize) -> Result<()> {
        if bytes == 0 || records == 0 || (!self.read_only && self.count != 0) {
            return Err(bad());
        }
        self.work_limits = Some((bytes, records));
        Ok(())
    }
    pub fn count(&self) -> u64 {
        self.count
    }
    pub fn resident_records(&self) -> usize {
        self.previous.as_ref().map_or(0, |r| r.map.len())
    }
    fn object_path(&self, digest: &[u8; 32]) -> PathBuf {
        let name = hex(digest);
        self.root.join("objects").join(&name[..2]).join(name)
    }
    fn put_object(&mut self, bytes: &[u8], known: bool, sync: bool) -> Result<[u8; 32]> {
        let digest = hash(bytes);
        if known {
            return Ok(digest);
        }
        let at_io = Instant::now();
        let before_sync = self.metrics.sync_ns;
        let path = self.object_path(&digest);
        fs::create_dir_all(path.parent().unwrap()).map_err(io)?;
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(bytes).map_err(io)?;
                self.metrics.object_bytes += bytes.len() as u64;
                self.metrics.object_files += 1;
                if sync {
                    let at = Instant::now();
                    file.sync_data().map_err(io)?;
                    self.metrics.sync_ns += at.elapsed().as_nanos();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if fsize(&path)? != bytes.len() as u64 {
                    return Err(bad());
                }
            }
            Err(e) => return Err(io(e)),
        }
        self.metrics.object_io_ns += at_io
            .elapsed()
            .as_nanos()
            .saturating_sub(self.metrics.sync_ns - before_sync);
        Ok(digest)
    }
    fn object_manifest(&mut self, current: &Records, sync: bool) -> Result<Blob> {
        let pages = current.map.len().div_ceil(PAGE);
        let _scratch = self
            .memory
            .reserve(current.map.len() * 128 + 4096)
            .map_err(|_| Error::Limit)?;
        let new_charge = self
            .memory
            .reserve(pages * 128 + 1024)
            .map_err(|_| Error::Limit)?;
        let mut root = Writer::new(&self.memory).map_err(graph_error)?;
        root.n(current.map.len() as u64);
        root.n(pages as u64);
        let entries: Vec<_> = current.map.iter().collect();
        let mut known_pages = BTreeSet::new();
        for page in entries.chunks(PAGE) {
            let mut w = Writer::new(&self.memory).map_err(graph_error)?;
            w.n(page.len() as u64);
            for (k, v) in page {
                let known = self
                    .previous
                    .as_ref()
                    .is_some_and(|old| old.map.get(k) == Some(*v));
                let digest = self.put_object(v, known, sync)?;
                w.raw(&k.0);
                w.raw(&digest);
            }
            let page = w.finish().map_err(graph_error)?;
            let digest = hash(&page.bytes);
            let known = self.known_pages.contains(&digest);
            let stored = self.put_object(&page.bytes, known, sync)?;
            known_pages.insert(stored);
            root.raw(&stored);
        }
        self.known_pages = known_pages;
        self._pages_charge = new_charge;
        self.metrics.pages_cached = pages;
        root.finish().map_err(graph_error)
    }
    pub fn append(&mut self, capsule: &Blob, sync: bool) -> Result<u64> {
        if self.read_only {
            return Err(bad());
        }
        let start = Instant::now();
        let before_object_io = self.metrics.object_io_ns;
        let before_sync = self.metrics.sync_ns;
        let tx = self.count;
        let expected = hash(&capsule.bytes);
        let mut current = None;
        let (kind, base, payload) = match self.mode {
            Mode::Full => (0, tx, None),
            Mode::Delta { interval } => {
                let records = Records::from_blob(capsule, &self.memory)?;
                self.metrics.compared_records += records.map.len() as u64
                    + self.previous.as_ref().map_or(0, |x| x.map.len() as u64);
                let (k, base, bytes) = if tx.is_multiple_of(interval) {
                    self.metrics.changed_records += records.map.len() as u64;
                    self.metrics.checkpoints += 1;
                    self.checkpoint = tx;
                    self.replay_bytes = 0;
                    self.replay_records = 0;
                    (1, tx, records.encode(&self.memory)?)
                } else {
                    let (bytes, changed) =
                        records.delta(self.previous.as_ref().ok_or_else(bad)?, &self.memory)?;
                    let next_bytes = self
                        .replay_bytes
                        .checked_add(bytes.bytes.len())
                        .ok_or_else(bad)?;
                    let next_records = self.replay_records.checked_add(changed).ok_or_else(bad)?;
                    if self
                        .work_limits
                        .is_some_and(|(b, r)| next_bytes > b || next_records > r)
                    {
                        // Oversized delta is replaced by a complete independently loadable checkpoint.
                        drop(bytes);
                        self.metrics.changed_records += records.map.len() as u64;
                        self.metrics.checkpoints += 1;
                        self.metrics.forced_checkpoints += 1;
                        self.checkpoint = tx;
                        self.replay_bytes = 0;
                        self.replay_records = 0;
                        (1, tx, records.encode(&self.memory)?)
                    } else {
                        self.metrics.changed_records += changed as u64;
                        self.replay_bytes = next_bytes;
                        self.replay_records = next_records;
                        (2, self.checkpoint, bytes)
                    }
                };
                current = Some(records);
                (k, base, Some(bytes))
            }
            Mode::Objects => {
                let records = Records::from_blob(capsule, &self.memory)?;
                self.metrics.compared_records += records.map.len() as u64;
                self.metrics.changed_records += records
                    .map
                    .iter()
                    .filter(|(k, v)| {
                        self.previous
                            .as_ref()
                            .is_none_or(|old| old.map.get(k) != Some(v))
                    })
                    .count() as u64;
                let root = self.object_manifest(&records, sync)?;
                current = Some(records);
                (3, tx, Some(root))
            }
        };
        let bytes = payload
            .as_ref()
            .map_or(capsule.bytes.as_slice(), |b| b.bytes.as_slice());
        self.metrics.record_plan_ns += start
            .elapsed()
            .as_nanos()
            .saturating_sub(self.metrics.object_io_ns - before_object_io)
            .saturating_sub(self.metrics.sync_ns - before_sync);
        let io_start = Instant::now();
        let sync_at_io = self.metrics.sync_ns;
        let offset = self.data.seek(SeekFrom::End(0)).map_err(io)?;
        self.data.write_all(bytes).map_err(io)?;
        let entry = Entry {
            offset,
            len: bytes.len() as u64,
            base,
            expected,
            payload_hash: hash(bytes),
            kind,
        };
        if sync {
            let at = Instant::now();
            self.data.sync_data().map_err(io)?;
            self.metrics.sync_ns += at.elapsed().as_nanos();
        }
        self.index.seek(SeekFrom::End(0)).map_err(io)?;
        self.index.write_all(&entry.encode()).map_err(io)?;
        if sync {
            let at = Instant::now();
            self.index.sync_data().map_err(io)?;
            self.metrics.sync_ns += at.elapsed().as_nanos();
        }
        self.metrics.data_root_io_ns += io_start
            .elapsed()
            .as_nanos()
            .saturating_sub(self.metrics.sync_ns - sync_at_io);
        self.metrics.payload_bytes += bytes.len() as u64;
        self.metrics.journal_bytes += INDEX_BYTES;
        self.previous = current;
        self.metrics.records_capacity = self.previous.as_ref().map_or(0, Records::capacity);
        self.count += 1;
        self.metrics.write_ns += start.elapsed().as_nanos();
        Ok(tx)
    }
    fn entry(&mut self, tx: u64, stats: &mut ReadMetrics) -> Result<Entry> {
        if tx >= self.count {
            return Err(bad());
        }
        let mut bytes = [0; 96];
        self.index
            .seek(SeekFrom::Start(
                tx.checked_mul(INDEX_BYTES).ok_or_else(bad)?,
            ))
            .map_err(io)?;
        self.index.read_exact(&mut bytes).map_err(io)?;
        stats.bytes += 96;
        stats.file_reads += 1;
        let e = Entry::decode(bytes);
        if e.offset
            .checked_add(e.len)
            .is_none_or(|end| self.data.metadata().map(|m| end > m.len()).unwrap_or(true))
        {
            return Err(bad());
        }
        Ok(e)
    }
    fn payload(&mut self, e: &Entry, stats: &mut ReadMetrics) -> Result<Buffer> {
        let b = read_buffer(&mut self.data, e.offset, e.len, &self.memory)?;
        stats.bytes += b.bytes.len() as u64;
        stats.file_reads += 1;
        if hash(&b.bytes) != e.payload_hash {
            return Err(bad());
        }
        Ok(b)
    }
    fn object(&self, digest: [u8; 32], stats: &mut ReadMetrics) -> Result<Buffer> {
        let b = read_file(&self.object_path(&digest), &self.memory)?;
        stats.bytes += b.bytes.len() as u64;
        stats.file_reads += 1;
        if hash(&b.bytes) != digest {
            return Err(bad());
        }
        Ok(b)
    }
    fn load_objects(&self, bytes: &[u8], stats: &mut ReadMetrics) -> Result<Records> {
        let mut r = Reader::new(bytes).map_err(graph_error)?;
        // Records live in referenced page objects, not inline in this root descriptor.
        let count = usize::try_from(r.n().map_err(graph_error)?).map_err(|_| bad())?;
        if count > 65536 {
            return Err(bad());
        }
        let pages = r.count(1024).map_err(graph_error)?;
        if pages != count.div_ceil(PAGE) {
            return Err(bad());
        }
        let charge = self
            .memory
            .reserve(count * 256 + 4096)
            .map_err(|_| Error::Limit)?;
        let mut out = Records {
            map: BTreeMap::new(),
            bytes: 0,
            _charge: charge,
        };
        for _ in 0..pages {
            let digest = r.take(32).map_err(graph_error)?.try_into().unwrap();
            let page = self.object(digest, stats)?;
            let mut p = Reader::new(&page.bytes).map_err(graph_error)?;
            for _ in 0..p.count(PAGE).map_err(graph_error)? {
                let key = RecordKey(p.take(33).map_err(graph_error)?.try_into().unwrap());
                let hash = p.take(32).map_err(graph_error)?.try_into().unwrap();
                let data = self.object(hash, stats)?;
                if out.map.contains_key(&key) {
                    return Err(bad());
                }
                let new_total = out.bytes.checked_add(data.bytes.len()).ok_or_else(bad)?;
                if new_total > pivot_core::model::integration::history_codec::MAX_BYTES {
                    return Err(bad());
                }
                let new_charge = self
                    .memory
                    .reserve(
                        new_total
                            .saturating_mul(2)
                            .saturating_add(count * 256)
                            .saturating_add(4096),
                    )
                    .map_err(|_| Error::Limit)?;
                out.map.insert(key, data.bytes);
                out.bytes = new_total;
                out._charge = new_charge;
            }
            p.end().map_err(graph_error)?;
        }
        r.end().map_err(graph_error)?;
        if out.map.len() != count {
            return Err(bad());
        }
        Ok(out)
    }
    pub fn load(&mut self, tx: u64) -> Result<(Blob, ReadMetrics)> {
        let mut stats = ReadMetrics::default();
        let entry = self.entry(tx, &mut stats)?;
        let payload = self.payload(&entry, &mut stats)?;
        let result = match entry.kind {
            0 => {
                let mut w = Writer::new(&self.memory).map_err(graph_error)?;
                w.raw(&payload.bytes);
                w.finish().map_err(graph_error)?
            }
            1 | 2 => {
                let interval = match self.mode {
                    Mode::Delta { interval } => interval,
                    _ => return Err(bad()),
                };
                if entry.base > tx || tx - entry.base >= interval {
                    return Err(bad());
                }
                let cp = self.entry(entry.base, &mut stats)?;
                if cp.kind != 1 || cp.base != entry.base {
                    return Err(bad());
                }
                let base = self.payload(&cp, &mut stats)?;
                let mut records = Records::decode(&base.bytes, &self.memory)?;
                let mut replay_bytes = 0usize;
                let mut replay_records = 0usize;
                for current in entry.base + 1..=tx {
                    let e = self.entry(current, &mut stats)?;
                    if e.kind != 2 || e.base != entry.base {
                        return Err(bad());
                    }
                    let bytes = self.payload(&e, &mut stats)?;
                    replay_bytes = replay_bytes
                        .checked_add(bytes.bytes.len())
                        .ok_or_else(bad)?;
                    let count = usize::try_from(u64::from_le_bytes(
                        bytes.bytes.get(..8).ok_or_else(bad)?.try_into().unwrap(),
                    ))
                    .map_err(|_| bad())?;
                    replay_records = replay_records.checked_add(count).ok_or_else(bad)?;
                    if self
                        .work_limits
                        .is_some_and(|(b, r)| replay_bytes > b || replay_records > r)
                    {
                        return Err(bad());
                    }
                    stats.records += records.apply_delta(&bytes.bytes, &self.memory)?;
                    stats.chain_steps += 1;
                }
                stats.records += records.map.len();
                records.assemble(&self.memory)?
            }
            3 => {
                let records = self.load_objects(&payload.bytes, &mut stats)?;
                stats.records = records.map.len();
                records.assemble(&self.memory)?
            }
            _ => return Err(bad()),
        };
        if hash(&result.bytes) != entry.expected {
            return Err(bad());
        }
        Ok((result, stats))
    }
    pub fn file_bytes(&self) -> Result<u64> {
        Ok(self.data.metadata().map_err(io)?.len()
            + self.index.metadata().map_err(io)?.len()
            + self.metrics.object_bytes)
    }
}
