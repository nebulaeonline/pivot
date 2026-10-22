//! Narrow integration-only captured scalar identity. Producer semantics stay outside the core.
use pivot_types::PivotDecimal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapturedScalarId(pub(crate) [u8; 16]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapturedFaultId(pub [u8; 16]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarValue {
    Decimal(PivotDecimal),
    Fault(CapturedFaultId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextError {
    Resource,
    Missing,
    Duplicate,
    Unknown,
    Snapshot,
    Fault,
    Type,
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "captured context: {self:?}")
    }
}
impl std::error::Error for ContextError {}
