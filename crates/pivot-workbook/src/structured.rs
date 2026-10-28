//! Opt-in ADR-0020 portable binding authority; no producer records enter the core.
use super::*;
use pivot_core::model::integration::resources::{Footprint, Memory, Shared};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
pub(crate) struct Scope {
    pub budget: Arc<projection::Budget>,
    versions: Arc<AtomicUsize>,
    slots: Arc<AtomicUsize>,
    entries: Arc<AtomicUsize>,
    contexts: Arc<AtomicUsize>,
}
pub(crate) struct Lease {
    _hold: projection::Hold,
    counter: Arc<AtomicUsize>,
    units: usize,
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.counter.fetch_sub(self.units, Ordering::SeqCst);
    }
}
impl Scope {
    fn claim(&self, bytes: usize, counter: &Arc<AtomicUsize>, max: usize) -> Result<Lease> {
        self.claim_units(bytes, counter, max, 1)
    }
    fn registry(&self) -> Result<Lease> {
        self.claim_units(2048, &self.entries, 64, 2)
    }
    fn claim_units(
        &self,
        bytes: usize,
        counter: &Arc<AtomicUsize>,
        max: usize,
        units: usize,
    ) -> Result<Lease> {
        counter
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                n.checked_add(units).filter(|v| *v <= max)
            })
            .map_err(|_| Error::Limit)?;
        match projection::Hold::new(&self.budget, bytes) {
            Ok(h) => Ok(Lease {
                _hold: h,
                counter: counter.clone(),
                units,
            }),
            Err(_) => {
                counter.fetch_sub(units, Ordering::SeqCst);
                Err(Error::Limit)
            }
        }
    }
}
pub struct FormulaPayload {
    source: String,
    tokens: Vec<Token>,
    _lease: Lease,
}
impl Footprint for FormulaPayload {
    fn heap_bytes(&self) -> usize {
        self.source.capacity() + self.tokens.capacity() * std::mem::size_of::<Token>()
    }
}
#[derive(Clone)]
pub struct StructuredContent(Shared<FormulaPayload>);
impl std::fmt::Debug for StructuredContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructuredFormula")
            .field("source", &self.0.source)
            .field("tokens", &self.0.tokens)
            .finish()
    }
}
impl PartialEq for StructuredContent {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source() && self.tokens() == other.tokens()
    }
}
impl Eq for StructuredContent {}
impl Footprint for StructuredContent {}
impl StructuredContent {
    pub fn source(&self) -> &str {
        &self.0.source
    }
    pub fn tokens(&self) -> &[Token] {
        &self.0.tokens
    }
    pub(crate) fn new(
        memory: &Memory,
        scope: &Scope,
        source: &str,
        tokens: Vec<Token>,
    ) -> Result<Self> {
        let lease = scope.claim(8192, &scope.versions, 64)?;
        let payload = FormulaPayload {
            source: source.into(),
            tokens,
            _lease: lease,
        };
        Ok(Self(memory.alloc(payload).map_err(|_| Error::Limit)?))
    }
}

use pivot_core::captured::{CapturedScalarId, ScalarValue};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputBinding {
    pub document: Uuid,
    pub producer: Uuid,
    pub output: Uuid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub ordinal: u32,
    pub span: std::ops::Range<usize>,
    pub binding: OutputBinding,
}
#[derive(Clone)]
pub(crate) struct Directory {
    pub scope: Arc<Scope>,
    _lease: Arc<Lease>,
    pub binding: OutputBinding,
    pub producer: Arc<str>,
    pub output: Arc<str>,
    pub available: bool,
    pub output_available: bool,
}
pub(crate) struct Adapter {
    _lease: Lease,
    _registry: Directory,
    pub binding: OutputBinding,
    pub node: NodeId,
    pub slot: CapturedScalarId,
}
impl Adapter {
    pub(crate) fn new(
        directory: &Directory,
        binding: OutputBinding,
        node: NodeId,
        slot: CapturedScalarId,
    ) -> Result<Self> {
        Ok(Self {
            binding,
            node,
            slot,
            _lease: directory.scope.claim(2048, &directory.scope.slots, 1)?,
            _registry: directory.clone(),
        })
    }
}
impl Footprint for Adapter {}
impl Workbook {
    #[doc(hidden)]
    pub fn is_structured_proof(&self) -> bool {
        persistent::state(self).directory.is_some()
    }

    #[doc(hidden)]
    pub fn enable_structured_proof(&mut self, producer: Uuid, output: Uuid) -> Result<()> {
        let memory = self.core.allocation_domain().map_err(graph_error)?;
        self.enable_structured_budget(producer, output, projection::Budget::binding_budget(memory))
    }
    pub(crate) fn enable_structured_budget(
        &mut self,
        producer: Uuid,
        output: Uuid,
        budget: Arc<projection::Budget>,
    ) -> Result<()> {
        if self.sheets().len() != 1
            || self.is_structured_proof()
            || producer.is_nil()
            || output.is_nil()
        {
            return Err(Error::InvalidTarget);
        }
        let scope = Arc::new(Scope {
            budget,
            versions: Arc::new(AtomicUsize::new(0)),
            slots: Arc::new(AtomicUsize::new(0)),
            entries: Arc::new(AtomicUsize::new(0)),
            contexts: Arc::new(AtomicUsize::new(0)),
        });
        let lease = scope.registry()?;
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        data.directory = Some(Directory {
            binding: OutputBinding {
                document: self.graph_id(),
                producer,
                output,
            },
            producer: "InvoiceLines".into(),
            output: "TotalAmount".into(),
            available: true,
            output_available: true,
            scope,
            _lease: Arc::new(lease),
        });
        self.persistent = data.freeze();
        Ok(())
    }
    #[doc(hidden)]
    pub fn resolve_structured(&self, source: &str) -> Result<Vec<Occurrence>> {
        let directory = persistent::state(self)
            .directory
            .as_ref()
            .ok_or(Error::InvalidTarget)?;
        let parsed = expression::parse_structured_proof(source).map_err(|_| Error::InvalidDraft)?;
        let mut out = Vec::new();
        let mut pending = vec![&parsed];
        while let Some(e) = pending.pop() {
            match e {
                Expr::CapturedOutput {
                    producer,
                    output,
                    span,
                } => {
                    if !directory.available
                        || !directory.output_available
                        || producer.as_str() != directory.producer.as_ref()
                        || output.as_str() != directory.output.as_ref()
                    {
                        return Err(Error::InvalidTarget);
                    }
                    if out.len() == 16 {
                        return Err(Error::Limit);
                    }
                    out.push(Occurrence {
                        ordinal: 0,
                        span: span.clone(),
                        binding: directory.binding,
                    });
                }
                Expr::Unary { operand, .. } => pending.push(operand),
                Expr::Binary { left, right, .. } => {
                    pending.push(right);
                    pending.push(left);
                }
                _ => (),
            }
        }
        out.sort_by_key(|o| o.span.start);
        for (i, o) in out.iter_mut().enumerate() {
            o.ordinal = i as u32;
        }
        Ok(out)
    }
    #[doc(hidden)]
    pub fn apply_structured(
        &mut self,
        expected: Expected,
        at: Position,
        source: &str,
        occurrences: &[Occurrence],
    ) -> Result<Expected> {
        self.check(expected)?;
        if self.resolve_structured(source)? != occurrences {
            return Err(Error::InvalidTarget);
        }
        let parsed = expression::parse_structured_proof(source).map_err(|_| Error::InvalidDraft)?;
        let (tokens, _, diagnostic) = Self::lower_text_axes(
            &self.rows,
            &self.columns,
            &Ok(parsed),
            persistent::state(self)
                .directory
                .as_ref()
                .map(|d| d.binding),
        );
        if diagnostic.is_some() {
            return Err(Error::InvalidDraft);
        }
        self.apply(
            expected,
            &[Edit::SetFormula {
                position: at,
                source: source.into(),
                tokens,
            }],
        )
    }
}
pub(crate) struct CapturedAdapterContext {
    pub core: pivot_core::model::integration::ScalarContext,
    _lease: Lease,
}
impl CalculationCapture {
    #[doc(hidden)]
    pub fn prepare_structured_context(
        &mut self,
        value: ScalarValue,
        diagnostics: &[pivot_core::captured::CapturedFaultId],
    ) -> Result<()> {
        if !diagnostics.is_empty() || !matches!(value, ScalarValue::Decimal(_)) {
            return Err(Error::InvalidTarget);
        }
        self.prepare_scalar(value, None)
    }
    fn prepare_scalar(&mut self, value: ScalarValue, reason: Option<SourceFault>) -> Result<()> {
        if self.scalar_context.is_some() {
            return Err(Error::InvalidTarget);
        }
        if let Some(adapter) = &self.pinned.adapter {
            let directory = self.pinned.directory.as_ref().ok_or(Error::InvalidTarget)?;
            let scope = &directory.scope;
            let lease = scope.claim(4096, &scope.contexts, 2)?;
            let producer_missing =
                !directory.available || directory.binding.producer != adapter.binding.producer;
            let missing = producer_missing
                || !directory.output_available
                || directory.binding.output != adapter.binding.output;
            let mut diagnostic = match value {
                ScalarValue::Fault(id) => Some((id, reason.ok_or(Error::InvalidTarget)?)),
                ScalarValue::Decimal(_) => None,
            };
            let value = if missing {
                let id = pivot_core::captured::CapturedFaultId(*Uuid::new_v4().as_bytes());
                diagnostic = Some((
                    id,
                    if producer_missing {
                        SourceFault::MissingProducer
                    } else {
                        SourceFault::MissingOutput
                    },
                ));
                ScalarValue::Fault(id)
            } else {
                value
            };
            let ids: Vec<_> = diagnostic.iter().map(|(id, _)| *id).collect();
            let core = self
                .local_core
                .scalar_context(&[(adapter.slot, value)], &ids)
                .map_err(graph_error)?;
            self.scalar_context = Some(CapturedAdapterContext {
                core,
                _lease: lease,
            });
            self.source_fault = diagnostic;
        }
        Ok(())
    }
}

impl Workbook {
    /// Server-owned same-document duplication; clients supply positions, never trusted bindings.
    #[doc(hidden)]
    pub fn duplicate_structured_block(
        &mut self,
        expected: Expected,
        source: Rectangle,
        destination: Rectangle,
    ) -> Result<Expected> {
        self.check(expected)?;
        let height = source
            .last
            .row
            .checked_sub(source.first.row)
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Position)?;
        let width = source
            .last
            .column
            .checked_sub(source.first.column)
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Position)?;
        if height as u64 * width as u64 > 256
            || destination.last.row.checked_sub(destination.first.row) != Some(height - 1)
            || destination
                .last
                .column
                .checked_sub(destination.first.column)
                != Some(width - 1)
        {
            return Err(Error::Limit);
        }
        self.axes_at(source.first)?;
        self.axes_at(source.last)?;
        self.axes_at(destination.first)?;
        self.axes_at(destination.last)?;
        self.duplicate_pairs(
            expected,
            (0..height).flat_map(|r| {
                (0..width).map(move |c| {
                    (
                        Position::new(source.first.row + r, source.first.column + c),
                        Position::new(destination.first.row + r, destination.first.column + c),
                    )
                })
            }),
            &Default::default(),
        )
    }
}

impl Workbook {
    #[doc(hidden)]
    pub fn rename_structured_producer(
        &mut self,
        expected: Expected,
        name: &str,
    ) -> Result<Expected> {
        self.check(expected)?;
        if name.is_empty()
            || name.len() > 63
            || !name.as_bytes()[0].is_ascii_alphabetic() && name.as_bytes()[0] != b'_'
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(Error::InvalidTarget);
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let directory = data.directory.as_mut().ok_or(Error::InvalidTarget)?;
        directory._lease = Arc::new(directory.scope.registry()?);
        if !directory.available {
            return Err(Error::InvalidTarget);
        }
        directory.producer = Arc::from(name);
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.persistent = data.freeze();
        self.state = next;
        Ok(self.expected())
    }
    #[doc(hidden)]
    pub fn retire_structured_producer(
        &mut self,
        expected: Expected,
        recreate: bool,
    ) -> Result<Expected> {
        self.check(expected)?;
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let directory = data.directory.as_mut().ok_or(Error::InvalidTarget)?;
        directory._lease = Arc::new(directory.scope.registry()?);
        directory.available = recreate;
        if recreate {
            directory.output_available = true;
            directory.binding.producer = Uuid::new_v4();
            directory.binding.output = Uuid::new_v4();
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.persistent = data.freeze();
        self.state = next;
        Ok(self.expected())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceFault {
    MissingProducer,
    MissingOutput,
    UnavailableResult,
    SchemaMismatch,
    TypeMismatch,
    NumericOverflow,
}
impl CalculationCapture {
    pub fn prepare_structured_failure(&mut self, kind: SourceFault) -> Result<()> {
        if self.pinned.adapter.is_none() {
            return Ok(());
        }
        let id = pivot_core::captured::CapturedFaultId(*Uuid::new_v4().as_bytes());
        self.prepare_scalar(ScalarValue::Fault(id), Some(kind))
    }
}

impl CalculationCapture {
    pub fn structured_binding(&self) -> Option<OutputBinding> {
        self.pinned.adapter.as_ref().map(|a| a.binding)
    }
    pub fn structured_labels(&self) -> Option<(&str, &str)> {
        self.pinned.directory.as_ref().map(|d| {
            if d.available
                && self
                    .pinned
                    .adapter
                    .as_ref()
                    .is_some_and(|a| a.binding == d.binding)
            {
                (d.producer.as_ref(), d.output.as_ref())
            } else {
                ("<missing>", "<missing>")
            }
        })
    }
}

impl Workbook {
    pub fn current_structured_binding(&self) -> Option<OutputBinding> {
        persistent::state(self)
            .directory
            .as_ref()
            .filter(|d| d.available)
            .map(|d| d.binding)
    }
}

impl Workbook {
    pub fn rename_structured_output(&mut self, expected: Expected, name: &str) -> Result<Expected> {
        self.check(expected)?;
        if name.is_empty()
            || name.len() > 63
            || !name.as_bytes()[0].is_ascii_alphabetic() && name.as_bytes()[0] != b'_'
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(Error::InvalidTarget);
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let d = data.directory.as_mut().ok_or(Error::InvalidTarget)?;
        d._lease = Arc::new(d.scope.registry()?);
        if !d.available || !d.output_available {
            return Err(Error::InvalidTarget);
        }
        d.output = Arc::from(name);
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.persistent = data.freeze();
        self.state = next;
        Ok(self.expected())
    }
    pub fn retire_structured_output(
        &mut self,
        expected: Expected,
        recreate: bool,
    ) -> Result<Expected> {
        self.check(expected)?;
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let d = data.directory.as_mut().ok_or(Error::InvalidTarget)?;
        d._lease = Arc::new(d.scope.registry()?);
        if !d.available {
            return Err(Error::InvalidTarget);
        }
        d.output_available = recreate;
        if recreate {
            d.binding.output = Uuid::new_v4();
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.persistent = data.freeze();
        self.state = next;
        Ok(self.expected())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceUsage {
    pub versions: usize,
    pub slots: usize,
    pub contexts: usize,
    pub entries: usize,
}
impl Workbook {
    pub fn structured_usage(&self) -> Option<ResourceUsage> {
        persistent::state(self)
            .directory
            .as_ref()
            .map(|d| ResourceUsage {
                versions: d.scope.versions.load(Ordering::SeqCst),
                slots: d.scope.slots.load(Ordering::SeqCst),
                contexts: d.scope.contexts.load(Ordering::SeqCst),
                entries: d.scope.entries.load(Ordering::SeqCst),
            })
    }
}
