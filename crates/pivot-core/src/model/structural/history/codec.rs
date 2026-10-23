//! Bounded feature-only research encoding primitives; not a document/wire format.
use super::super::*;
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub fn malformed() -> ModelError {
    ModelError::Backend("malformed cold history")
}
/// Stable canonical section/record key; numeric ordinals use big-endian ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordKey(pub [u8; 33]);
impl RecordKey {
    pub fn new(tag: u8, key: &[u8]) -> Self {
        assert!(key.len() <= 32);
        let mut out = [0; 33];
        out[0] = tag;
        out[1..1 + key.len()].copy_from_slice(key);
        Self(out)
    }
    pub fn number(tag: u8, n: u64) -> Self {
        Self::new(tag, &n.to_be_bytes())
    }
}
#[derive(Clone, Debug)]
pub struct RecordSpan {
    pub key: RecordKey,
    pub range: std::ops::Range<usize>,
}
pub struct Blob {
    pub bytes: Vec<u8>,
    pub records: Vec<RecordSpan>,
    memory: Memory,
    _bytes_charge: Charge,
    _records_charge: Charge,
}
impl Blob {
    pub fn buffer_capacity(&self) -> usize {
        self.bytes.capacity() + self.records.capacity() * std::mem::size_of::<RecordSpan>()
    }
    pub fn append(&mut self, key: RecordKey, bytes: &[u8]) -> Result<(), ModelError> {
        let mut writer = Writer {
            bytes: std::mem::take(&mut self.bytes),
            records: std::mem::take(&mut self.records),
            failed: false,
            memory: self.memory.clone(),
            bytes_charge: None,
            records_charge: None,
            key,
            start: 0,
        };
        writer.start = writer.bytes.len();
        // Existing charges remain alive during possible buffer replacement.
        writer.raw(bytes);
        writer.close();
        let new = writer.finish()?;
        *self = new;
        Ok(())
    }
}
pub struct Writer {
    bytes: Vec<u8>,
    records: Vec<RecordSpan>,
    failed: bool,
    memory: Memory,
    bytes_charge: Option<Charge>,
    records_charge: Option<Charge>,
    key: RecordKey,
    start: usize,
}
impl Writer {
    pub fn new(memory: &Memory) -> Result<Self, ModelError> {
        Ok(Self {
            bytes: Vec::new(),
            records: Vec::new(),
            failed: false,
            memory: memory.clone(),
            bytes_charge: Some(memory.reserve(2048).map_err(|_| malformed())?),
            records_charge: Some(memory.reserve(1024).map_err(|_| malformed())?),
            key: RecordKey::new(0, &[]),
            start: 0,
        })
    }
    fn admit_bytes(&mut self, needed: usize) -> bool {
        if needed > MAX_BYTES {
            self.failed = true;
            return false;
        }
        if needed > self.bytes.capacity() || self.bytes_charge.is_none() {
            let capacity = needed
                .max(self.bytes.capacity())
                .max(128)
                .next_power_of_two()
                .min(MAX_BYTES);
            let Ok(charge) = self.memory.reserve(capacity + 2048) else {
                self.failed = true;
                return false;
            };
            if self
                .bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .is_err()
            {
                self.failed = true;
                return false;
            }
            self.bytes_charge = Some(charge);
        }
        true
    }
    fn push_span(&mut self, span: RecordSpan) {
        if self.failed {
            return;
        }
        if self.records.len() >= 65536 {
            self.failed = true;
            return;
        }
        if self.records.len() == self.records.capacity() || self.records_charge.is_none() {
            let capacity = (self.records.len() + 1)
                .max(self.records.capacity())
                .max(8)
                .next_power_of_two();
            let Ok(charge) = self
                .memory
                .reserve(capacity * std::mem::size_of::<RecordSpan>() + 1024)
            else {
                self.failed = true;
                return;
            };
            if self
                .records
                .try_reserve_exact(capacity - self.records.len())
                .is_err()
            {
                self.failed = true;
                return;
            }
            self.records_charge = Some(charge);
        }
        self.records.push(span);
    }
    fn close(&mut self) {
        if self.start < self.bytes.len() {
            self.push_span(RecordSpan {
                key: self.key,
                range: self.start..self.bytes.len(),
            });
        }
        self.start = self.bytes.len();
    }
    pub fn boundary(&mut self, key: RecordKey) {
        self.close();
        self.key = key;
    }
    pub fn append_blob(&mut self, blob: &Blob) {
        self.close();
        let base = self.bytes.len();
        self.raw(&blob.bytes);
        for span in &blob.records {
            self.push_span(RecordSpan {
                key: span.key,
                range: base + span.range.start..base + span.range.end,
            });
        }
        self.start = self.bytes.len();
    }
    pub fn raw(&mut self, b: &[u8]) {
        if self.failed {
            return;
        }
        let needed = self.bytes.len().saturating_add(b.len());
        if self.admit_bytes(needed) {
            self.bytes.extend_from_slice(b);
        }
    }
    pub fn n(&mut self, n: u64) {
        self.raw(&n.to_le_bytes());
    }
    pub fn flag(&mut self, b: bool) {
        self.n(u64::from(b));
    }
    pub fn bytes(&mut self, b: &[u8]) {
        self.n(b.len() as u64);
        self.raw(b);
    }
    pub fn text(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
    pub fn decimal(&mut self, d: PivotDecimal) {
        let (a, b, c, s, e) = d.parts();
        for n in [a, b, c, u32::from(s), e] {
            self.n(n.into());
        }
    }
    pub fn parse_error(&mut self, e: &ParseError) {
        self.n(e.offset as u64);
        self.text(e.message);
    }
    pub fn finish(mut self) -> Result<Blob, ModelError> {
        self.close();
        if self.failed {
            return Err(malformed());
        }
        if self.bytes_charge.is_none() && !self.admit_bytes(self.bytes.len()) {
            return Err(malformed());
        }
        if self.records_charge.is_none() {
            let charge = self
                .memory
                .reserve(self.records.capacity() * std::mem::size_of::<RecordSpan>() + 1024)
                .map_err(|_| malformed())?;
            self.records_charge = Some(charge);
        }
        Ok(Blob {
            bytes: self.bytes,
            records: self.records,
            memory: self.memory,
            _bytes_charge: self.bytes_charge.unwrap(),
            _records_charge: self.records_charge.unwrap(),
        })
    }
}
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ModelError> {
        if bytes.len() > MAX_BYTES {
            return Err(malformed());
        }
        Ok(Self { bytes, at: 0 })
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], ModelError> {
        let end = self.at.checked_add(n).ok_or_else(malformed)?;
        let b = self.bytes.get(self.at..end).ok_or_else(malformed)?;
        self.at = end;
        Ok(b)
    }
    pub fn n(&mut self) -> Result<u64, ModelError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn count(&mut self, max: usize) -> Result<usize, ModelError> {
        let n = usize::try_from(self.n()?).map_err(|_| malformed())?;
        if n > max || n > self.bytes.len() - self.at {
            Err(malformed())
        } else {
            Ok(n)
        }
    }
    pub fn u32(&mut self) -> Result<u32, ModelError> {
        self.n()?.try_into().map_err(|_| malformed())
    }
    pub fn flag(&mut self) -> Result<bool, ModelError> {
        match self.n()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(malformed()),
        }
    }
    pub fn bytes(&mut self, max: usize) -> Result<&'a [u8], ModelError> {
        let n = self.count(max)?;
        self.take(n)
    }
    pub fn text(&mut self, max: usize) -> Result<String, ModelError> {
        String::from_utf8(self.bytes(max)?.to_vec()).map_err(|_| malformed())
    }
    pub fn id(&mut self) -> Result<[u8; 16], ModelError> {
        let id = self.take(16)?.try_into().unwrap();
        if id == [0; 16] {
            Err(malformed())
        } else {
            Ok(id)
        }
    }
    pub fn decimal(&mut self) -> Result<PivotDecimal, ModelError> {
        let (a, b, c, s, e) = (
            self.u32()?,
            self.u32()?,
            self.u32()?,
            self.flag()?,
            self.u32()?,
        );
        PivotDecimal::from_parts(a, b, c, s, e).map_err(|_| malformed())
    }
    pub fn parse_error(&mut self) -> Result<ParseError, ModelError> {
        let offset = usize::try_from(self.n()?).map_err(|_| malformed())?;
        let text = self.text(128)?;
        const MESSAGES: &[&str] = &[
            "expression too long",
            "unexpected trailing input",
            "expression nesting too deep",
            "expected closing parenthesis",
            "invalid or unrepresentable decimal",
            "expected decimal, name, sign, or parenthesis",
            "invalid cell reference",
            "expected SUM rectangle",
            "expected range colon",
            "retired cell reference",
            "cell reference outside worksheet",
            "structured binding unavailable",
            "invalid proof label",
            "proof label too long",
            "expected structured opening brace",
            "expected structured output separator",
            "expected structured closing brace",
            "copied rejected draft; explicitly resubmit to execute",
        ];
        let message = *MESSAGES
            .iter()
            .find(|s| **s == text)
            .ok_or_else(malformed)?;
        Ok(ParseError { offset, message })
    }
    pub fn end(&self) -> Result<(), ModelError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(malformed())
        }
    }
}
