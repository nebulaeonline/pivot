//! Bounded feature-only research encoding primitives; not a document/wire format.
use super::super::*;
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub fn malformed() -> ModelError {
    ModelError::Backend("malformed cold history")
}
pub struct Blob {
    pub bytes: Vec<u8>,
    _charge: Charge,
}
pub struct Writer {
    bytes: Vec<u8>,
    failed: bool,
    charge: Charge,
}
impl Writer {
    pub fn new(memory: &Memory) -> Result<Self, ModelError> {
        Ok(Self {
            bytes: Vec::new(),
            failed: false,
            charge: memory.reserve(MAX_BYTES + 4096).map_err(|_| malformed())?,
        })
    }
    pub fn raw(&mut self, b: &[u8]) {
        if self.bytes.len().saturating_add(b.len()) > MAX_BYTES {
            self.failed = true;
            return;
        }
        self.bytes.extend_from_slice(b);
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
    pub fn finish(self) -> Result<Blob, ModelError> {
        if self.failed {
            Err(malformed())
        } else {
            Ok(Blob {
                bytes: self.bytes,
                _charge: self.charge,
            })
        }
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
