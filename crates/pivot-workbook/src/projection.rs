//! Opt-in ADR-0019 fixed-record proof. No worksheet identities are created here.
use pivot_core::{
    interface::{InputRequirement, NamedInvocationRequest},
    model::{
        WorkingModel,
        integration::{
            CalculationCapture, IntegrationRuntime,
            resources::{Charge, Memory},
        },
    },
};
use pivot_types::{PivotDecimal, PivotValue};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use uuid::Uuid;
macro_rules! id { ($($name:ident),*) => {$(#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct $name(pub Uuid);)*}; }
id!(
    ProducerId,
    GenerationId,
    SchemaId,
    FieldId,
    OutputId,
    ProjectionId
);
const OWNERSHIP: usize = 1_048_576;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Bounds,
    DuplicateKey,
    MissingKey,
    MissingRecord,
    Unavailable,
    Schema,
    Stale,
    Budget,
    Numeric,
    Core,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "projection proof: {self:?}")
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
struct Budget {
    used: AtomicUsize,
    pins: AtomicUsize,
    memory: Memory,
}
struct Hold {
    budget: Arc<Budget>,
    bytes: usize,
    _charge: Charge,
}
impl Hold {
    fn new(b: &Arc<Budget>, bytes: usize) -> Result<Self> {
        b.used
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                v.checked_add(bytes).filter(|v| *v <= OWNERSHIP)
            })
            .map_err(|_| Error::Budget)?;
        match b.memory.reserve(bytes) {
            Ok(charge) => Ok(Self {
                budget: b.clone(),
                bytes,
                _charge: charge,
            }),
            Err(_) => {
                b.used.fetch_sub(bytes, Ordering::SeqCst);
                Err(Error::Budget)
            }
        }
    }
}
impl Drop for Hold {
    fn drop(&mut self) {
        self.budget.used.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}
struct Pin {
    budget: Arc<Budget>,
    _hold: Hold,
}
impl Drop for Pin {
    fn drop(&mut self) {
        self.budget.pins.fetch_sub(1, Ordering::SeqCst);
    }
}
#[derive(Clone, Debug)]
pub struct Record {
    pub key: String,
    pub description: String,
    pub quantity: PivotDecimal,
    pub unit_price: PivotDecimal,
    pub amount: PivotDecimal,
}
pub struct Generation {
    pub id: GenerationId,
    pub schema: SchemaId,
    rows: Vec<Record>,
    keys: BTreeMap<String, usize>,
    _hold: Hold,
}
impl Generation {
    pub fn record(&self, key: &str) -> Result<&Record> {
        self.keys
            .get(key)
            .map(|&i| &self.rows[i])
            .ok_or(Error::MissingRecord)
    }
    pub fn rows(&self) -> &[Record] {
        &self.rows
    }
    pub fn total(&self) -> Result<PivotDecimal> {
        self.rows.iter().try_fold(PivotDecimal::ZERO, |v, r| {
            v.checked_add(r.amount).map_err(|_| Error::Numeric)
        })
    }
}
#[derive(Clone, Debug)]
pub struct Projection {
    pub id: ProjectionId,
    pub after_row: u32,
    pub reverse: bool,
}
pub struct State {
    pub producer: ProducerId,
    pub schema: SchemaId,
    pub fields: [FieldId; 4],
    pub output: OutputId,
    pub projections: [Projection; 2],
    pub epoch: u64,
    pub presentation: u64,
    pub changed: bool,
    pub last_error: Option<Error>,
    generation: Option<Arc<Generation>>,
    numeric: WorkingModel,
    budget: Arc<Budget>,
    _hold: Hold,
}
pub struct Capture {
    pub generation: Arc<Generation>,
    pub producer: ProducerId,
    pub epoch: u64,
    numeric: CalculationCapture,
    _pin: Pin,
}
impl Capture {
    pub fn evaluate(&self) -> Result<(PivotDecimal, PivotDecimal)> {
        let total = self.generation.total()?;
        let result = self
            .numeric
            .invoke_named(&NamedInvocationRequest {
                inputs: vec![("total".into(), PivotValue::Decimal(total))],
                outputs: vec!["taxed".into()],
            })
            .map_err(|_| Error::Core)?;
        let taxed = match result.outputs().get("taxed") {
            Some(Ok(PivotValue::Decimal(v))) => *v,
            _ => return Err(Error::Numeric),
        };
        Ok((total, taxed))
    }
}
#[derive(Clone, Debug)]
pub struct Row {
    pub kind: u32,
    pub ordinary: u32,
    pub projection: Option<ProjectionId>,
    pub key: String,
    pub ordinal: u32,
    pub fields: Vec<String>,
}
pub struct Frame {
    pub rows: Vec<Row>,
    pub count: u32,
    pub generation: Option<GenerationId>,
    pub epoch: u64,
    pub presentation: u64,
    pub placements: [Projection; 2],
    pub total: Result<PivotDecimal>,
    pub failure: Option<Error>,
    _hold: Hold,
}
impl State {
    pub fn new(runtime: &IntegrationRuntime) -> Result<Self> {
        let budget = Arc::new(Budget {
            used: AtomicUsize::new(0),
            pins: AtomicUsize::new(0),
            memory: runtime.memory(),
        });
        let hold = Hold::new(&budget, 16_384)?;
        let mut numeric =
            WorkingModel::new_in("Projection numeric adapter", runtime).map_err(|_| Error::Core)?;
        let input = numeric
            .add_input("total", PivotDecimal::ZERO)
            .map_err(|_| Error::Core)?;
        let taxed = numeric
            .add_computed("taxed", "total*1.1")
            .map_err(|_| Error::Core)?;
        numeric
            .add_input_port("total", input, InputRequirement::Required)
            .map_err(|_| Error::Core)?;
        numeric
            .add_output_port("taxed", taxed)
            .map_err(|_| Error::Core)?;
        Ok(Self {
            producer: ProducerId(Uuid::new_v4()),
            schema: SchemaId(Uuid::new_v4()),
            fields: std::array::from_fn(|_| FieldId(Uuid::new_v4())),
            output: OutputId(Uuid::new_v4()),
            projections: std::array::from_fn(|_| Projection {
                id: ProjectionId(Uuid::new_v4()),
                after_row: 8,
                reverse: false,
            }),
            epoch: 0,
            presentation: 0,
            changed: false,
            last_error: None,
            generation: None,
            numeric,
            budget,
            _hold: hold,
        })
    }
    pub fn fixture(count: usize) -> Result<Vec<Record>> {
        if ![0, 3, 5].contains(&count) {
            return Err(Error::Bounds);
        }
        Ok((0..count)
            .map(|i| Record {
                key: format!("L{}", i + 1),
                description: if i < 2 {
                    "Widget A".into()
                } else {
                    format!("Service {}", i + 1)
                },
                quantity: PivotDecimal::from((i + 1) as i64),
                unit_price: PivotDecimal::from(10),
                amount: PivotDecimal::from(((i + 1) * 10) as i64),
            })
            .collect())
    }
    pub fn publish(&mut self, expected: u64, schema: SchemaId, rows: &[Record]) -> Result<()> {
        let result = self.stage_publish(expected, schema, rows);
        self.last_error = result.as_ref().err().copied();
        result
    }
    fn stage_publish(&mut self, expected: u64, schema: SchemaId, rows: &[Record]) -> Result<()> {
        if expected != self.epoch {
            return Err(Error::Stale);
        }
        if schema != self.schema {
            return Err(Error::Schema);
        }
        if rows.len() > 64 {
            return Err(Error::Bounds);
        }
        let mut bytes = 256 + rows.len() * (std::mem::size_of::<Record>() + 256);
        for r in rows {
            if r.key.is_empty() {
                return Err(Error::MissingKey);
            }
            if r.key.len() > 128
                || r.key.chars().any(char::is_control)
                || crate::entry::validate_text(&r.description, 4096).is_err()
            {
                return Err(Error::Bounds);
            }
            bytes = bytes
                .checked_add(r.key.len() * 2 + r.description.len())
                .ok_or(Error::Budget)?;
        }
        if bytes > 256 * 1024 {
            return Err(Error::Budget);
        }
        let hold = Hold::new(&self.budget, bytes)?;
        let mut keys = BTreeMap::new();
        for (i, r) in rows.iter().enumerate() {
            if keys.insert(r.key.clone(), i).is_some() {
                return Err(Error::DuplicateKey);
            }
        }
        let epoch = self.epoch.checked_add(1).ok_or(Error::Bounds)?;
        let presentation = self.presentation.checked_add(1).ok_or(Error::Bounds)?;
        let generation = Arc::new(Generation {
            id: GenerationId(Uuid::new_v4()),
            schema,
            rows: rows.to_vec(),
            keys,
            _hold: hold,
        });
        self.generation = Some(generation);
        self.epoch = epoch;
        self.presentation = presentation;
        self.changed = true;
        Ok(())
    }
    pub fn capture(&self) -> Result<Capture> {
        let generation = self.generation.as_ref().ok_or(Error::Unavailable)?.clone();
        let hold = Hold::new(&self.budget, 512)?;
        self.budget
            .pins
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                (v < 2).then_some(v + 1)
            })
            .map_err(|_| Error::Budget)?;
        let pin = Pin {
            budget: self.budget.clone(),
            _hold: hold,
        };
        Ok(Capture {
            generation,
            producer: self.producer,
            epoch: self.epoch,
            numeric: self
                .numeric
                .capture_calculation()
                .map_err(|_| Error::Core)?,
            _pin: pin,
        })
    }
    pub fn layout(
        &mut self,
        expected: u64,
        id: ProjectionId,
        after: u32,
        reverse: bool,
    ) -> Result<()> {
        if expected != self.presentation {
            return Err(Error::Stale);
        }
        if !(1..=1024).contains(&after) {
            return Err(Error::Bounds);
        }
        let presentation = self.presentation.checked_add(1).ok_or(Error::Bounds)?;
        let projection = self
            .projections
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(Error::Bounds)?;
        projection.after_row = after;
        projection.reverse = reverse;
        self.presentation = presentation;
        self.changed = true;
        Ok(())
    }
    pub fn view(&self, first: u32, count: u32) -> Result<Frame> {
        if first == 0 || count == 0 || count > 20 {
            return Err(Error::Bounds);
        }
        let generation = self.generation.as_ref();
        let n = generation.map_or(0, |g| g.rows.len());
        let height = n.max(1) + 2;
        let hold = Hold::new(&self.budget, 24_576 + count as usize * 10_240)?;
        let mut rows = Vec::with_capacity(count as usize);
        let mut visible = 0u32;
        let mut push = |row: &mut dyn FnMut() -> Row| {
            visible += 1;
            if visible >= first && rows.len() < count as usize {
                rows.push(row())
            }
        };
        // Bound iteration to worksheet axes plus two admitted bands; never allocate cells.
        for ordinary in 1..=1024 {
            push(&mut || Row {
                kind: 0,
                ordinary,
                projection: None,
                key: String::new(),
                ordinal: 0,
                fields: vec![],
            });
            for p in self.projections.iter().filter(|p| p.after_row == ordinary) {
                push(&mut || Row {
                    kind: 1,
                    ordinary: 0,
                    projection: Some(p.id),
                    key: String::new(),
                    ordinal: 0,
                    fields: vec!["InvoiceLines — read-only".into()],
                });
                if let Some(g) = generation.filter(|g| !g.rows.is_empty()) {
                    for ordinal in 0..g.rows.len() {
                        let index = if p.reverse {
                            g.rows.len() - 1 - ordinal
                        } else {
                            ordinal
                        };
                        let r = &g.rows[index];
                        push(&mut || Row {
                            kind: 2,
                            ordinary: 0,
                            projection: Some(p.id),
                            key: r.key.clone(),
                            ordinal: ordinal as u32 + 1,
                            fields: vec![
                                r.description.clone(),
                                r.quantity.to_string(),
                                r.unit_price.to_string(),
                                r.amount.to_string(),
                            ],
                        });
                    }
                } else {
                    push(&mut || Row {
                        kind: 4,
                        ordinary: 0,
                        projection: Some(p.id),
                        key: String::new(),
                        ordinal: 0,
                        fields: vec![if generation.is_some() {
                            "Empty result".into()
                        } else {
                            "Unavailable result".into()
                        }],
                    });
                }
                push(&mut || Row {
                    kind: 3,
                    ordinary: 0,
                    projection: Some(p.id),
                    key: String::new(),
                    ordinal: 0,
                    fields: vec![format!(
                        "TotalAmount: {}",
                        generation
                            .map_or(Err(Error::Unavailable), |g| g.total())
                            .map(|v| v.to_string())
                            .unwrap_or_else(|e| format!("{e:?}"))
                    )],
                });
            }
        }
        if first > 1024 + 2 * height as u32 {
            return Err(Error::Bounds);
        }

        Ok(Frame {
            rows,
            count: n as u32,
            generation: generation.map(|g| g.id),
            epoch: self.epoch,
            presentation: self.presentation,
            placements: self.projections.clone(),
            total: generation.map_or(Err(Error::Unavailable), |g| g.total()),
            failure: self.last_error,
            _hold: hold,
        })
    }
    pub fn record_keys(&self) -> Vec<String> {
        self.generation
            .as_ref()
            .map(|g| g.rows.iter().map(|r| r.key.clone()).collect())
            .unwrap_or_default()
    }
    pub fn used_bytes(&self) -> usize {
        self.budget.used.load(Ordering::SeqCst)
    }
}
