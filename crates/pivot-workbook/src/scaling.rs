//! Non-production input-overlay feasibility prototype. Structural edits use the
//! accepted full-copy workbook and full validation; no partial optimizer replaces it.
use super::*;
use pivot_core::{evaluation::evaluate_shared_inputs, model::ModelSnapshot};
use std::sync::Arc;

struct Patch {
    key: CellId,
    value: Content,
    priority: u64,
    left: Root,
    right: Root,
}
impl Clone for Patch {
    fn clone(&self) -> Self {
        pivot_core::experiment_profile::copy(std::mem::size_of::<Self>());
        Self {
            key: self.key,
            value: self.value.clone(),
            priority: self.priority,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}
type Root = Option<Arc<Patch>>;
fn priority(key: CellId) -> u64 {
    let k = key.uuid().as_u128();
    let mut x = (k as u64) ^ ((k >> 64) as u64);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
fn get(root: &Root, key: CellId) -> Option<&Content> {
    let mut node = root.as_ref();
    while let Some(n) = node {
        match key.cmp(&n.key) {
            std::cmp::Ordering::Equal => return Some(&n.value),
            std::cmp::Ordering::Less => node = n.left.as_ref(),
            std::cmp::Ordering::Greater => node = n.right.as_ref(),
        }
    }
    None
}
fn split(root: &Root, key: CellId) -> (Root, Root) {
    match root {
        None => (None, None),
        Some(n) if n.key < key => {
            let (a, b) = split(&n.right, key);
            (
                Some(Arc::new(Patch {
                    right: a,
                    ..(**n).clone()
                })),
                b,
            )
        }
        Some(n) => {
            let (a, b) = split(&n.left, key);
            (
                a,
                Some(Arc::new(Patch {
                    left: b,
                    ..(**n).clone()
                })),
            )
        }
    }
}
fn set(root: &Root, key: CellId, value: Content) -> Root {
    let rank = priority(key);
    match root {
        None => Some(Arc::new(Patch {
            key,
            value,
            priority: rank,
            left: None,
            right: None,
        })),
        Some(n) if n.key == key => Some(Arc::new(Patch {
            value,
            ..(**n).clone()
        })),
        Some(n) if rank < n.priority => {
            let (left, right) = split(root, key);
            Some(Arc::new(Patch {
                key,
                value,
                priority: rank,
                left,
                right,
            }))
        }
        Some(n) if key < n.key => Some(Arc::new(Patch {
            left: set(&n.left, key, value),
            ..(**n).clone()
        })),
        Some(n) => Some(Arc::new(Patch {
            right: set(&n.right, key, value),
            ..(**n).clone()
        })),
    }
}
fn visit(root: &Root, f: &mut impl FnMut(CellId, &Content)) {
    if let Some(n) = root {
        visit(&n.left, f);
        f(n.key, &n.value);
        visit(&n.right, f);
    }
}
fn copy(w: &Workbook) -> Workbook {
    Workbook {
        session: w.session,
        state: w.state,
        worksheet: w.worksheet,
        graph_id: w.graph_id,
        rows: w.rows.clone(),
        columns: w.columns.clone(),
        data: w.data.clone(),
        core: w.core.stage(),
        limits: w.limits,
    }
}
struct Topology {
    book: Workbook,
    snapshot: std::result::Result<ModelSnapshot, SnapshotError>,
    ids: BTreeMap<NodeId, CellId>,
}
impl Topology {
    fn new(book: Workbook) -> Self {
        let snapshot = book.core.snapshot();
        let ids = book
            .data
            .bindings
            .iter()
            .map(|(&c, b)| (b.runtime, c))
            .collect();
        Self {
            book,
            snapshot,
            ids,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version {
    pub session: Uuid,
    pub sequence: u64,
}
pub struct SharedWorkbook {
    base: Arc<Topology>,
    patches: Root,
    version: Version,
}
impl SharedWorkbook {
    pub fn new(book: Workbook) -> Self {
        Self {
            version: Version {
                session: book.session,
                sequence: 0,
            },
            base: Arc::new(Topology::new(book)),
            patches: None,
        }
    }
    pub fn expected(&self) -> Version {
        self.version
    }
    pub fn cell_at(&self, pos: Position) -> Result<Option<CellView>> {
        let mut cell = self.base.book.cell_at(pos)?;
        if let Some(c) = &mut cell
            && let Some(value) = get(&self.patches, c.reference.cell)
        {
            c.content = value.clone();
        }
        Ok(cell)
    }
    pub fn apply(&mut self, expected: Version, edits: &[Edit]) -> Result<Version> {
        let _phase = pivot_core::experiment_profile::scope(11);
        if expected.session != self.version.session {
            return Err(Error::StaleSession);
        }
        if expected != self.version {
            return Err(Error::StaleRevision);
        }
        self.base.book.preflight(edits)?;
        let mut patches = self.patches.clone();
        let mut touched = BTreeSet::new();
        let mut fast = true;
        for edit in edits {
            let (pos, content) = match edit {
                Edit::SetDecimal { position, value } => (*position, Content::Decimal(*value)),
                Edit::Clear(position) => (*position, Content::Blank),
                _ => {
                    fast = false;
                    break;
                }
            };
            self.base.book.axes_at(pos)?;
            let Some(&id) = self.base.book.data.positions.get(&pos) else {
                fast = false;
                break;
            };
            let original = self.base.book.data.records[&id].content.as_ref().unwrap();
            if !matches!(original, Content::Blank | Content::Decimal(_)) {
                fast = false;
                break;
            }
            let current = get(&patches, id).unwrap_or(original);
            if current != &content {
                patches = set(&patches, id, content);
                touched.insert(id);
            }
        }
        if fast {
            let changed = touched.iter().any(|&id| {
                let original = self.base.book.data.records[&id].content.as_ref().unwrap();
                get(&patches, id).unwrap_or(original) != get(&self.patches, id).unwrap_or(original)
            });
            if !changed {
                return Ok(self.version);
            }
            let next = self
                .version
                .sequence
                .checked_add(1)
                .ok_or(Error::Exhausted)?;
            self.patches = patches;
            self.version.sequence = next;
        } else {
            // Slow path is intentionally whole-state: preserve every accepted edit.
            let mut candidate = copy(&self.base.book);
            let mut failure = None;
            visit(&self.patches, &mut |id, content| {
                let value = match content {
                    Content::Decimal(v) => v.value(),
                    Content::Blank => PivotDecimal::ZERO,
                    _ => unreachable!(),
                };
                if let Err(e) = candidate
                    .core
                    .replace_input(candidate.data.bindings[&id].runtime, value)
                {
                    failure = Some(graph_error(e));
                }
                candidate.data.records.get_mut(&id).unwrap().content = Some(content.clone());
            });
            if let Some(e) = failure {
                return Err(e);
            }
            let before = candidate.expected();
            let after = candidate.apply(before, edits)?;
            if before == after {
                return Ok(self.version);
            }
            let next = self
                .version
                .sequence
                .checked_add(1)
                .ok_or(Error::Exhausted)?;
            let topology = Arc::new(Topology::new(candidate));
            self.base = topology;
            self.patches = None;
            self.version.sequence = next;
        }
        Ok(self.version)
    }
    pub fn capture(&self) -> Result<SharedCapture> {
        self.base
            .snapshot
            .as_ref()
            .map_err(|_| Error::InvalidDraft)?;
        Ok(SharedCapture {
            base: self.base.clone(),
            patches: self.patches.clone(),
            version: self.version,
        })
    }
    pub fn read(&self, rectangle: Rectangle, max: usize) -> Result<(Version, RectangleRead)> {
        let mut read = self
            .base
            .book
            .read(self.base.book.expected(), rectangle, max)?;
        for cell in &mut read.cells {
            if let Some(content) = get(&self.patches, cell.reference.cell) {
                cell.content = content.clone();
            }
        }
        // RectangleRead's embedded base tag is not an experimental version.
        // Return the authoritative experiment version separately.
        Ok((self.version, read))
    }
}
pub struct SharedCapture {
    base: Arc<Topology>,
    patches: Root,
    pub version: Version,
}
impl SharedCapture {
    pub fn evaluate(
        &self,
        outputs: &[CellRef],
    ) -> Result<BTreeMap<NodeId, std::result::Result<PivotDecimal, EvaluationError>>> {
        if outputs.len() > self.base.book.limits.read_records {
            return Err(Error::Limit);
        }
        let mut nodes = Vec::new();
        for &r in outputs {
            let id = self.base.book.target(r)?;
            if self.base.book.data.records[&id].position.is_none() {
                return Err(Error::InvalidTarget);
            }
            nodes.push(self.base.book.data.bindings[&id].runtime);
        }
        evaluate_shared_inputs(
            self.base
                .snapshot
                .as_ref()
                .map_err(|_| Error::InvalidDraft)?,
            &nodes,
            |id| {
                let cell = self.base.ids.get(&id)?;
                match get(&self.patches, *cell)? {
                    Content::Decimal(v) => Some(v.value()),
                    Content::Blank => Some(PivotDecimal::ZERO),
                    _ => unreachable!(),
                }
            },
        )
        .map_err(graph_error)
    }
    pub fn value(&self, output: CellRef) -> Result<PivotDecimal> {
        let cell = self.base.book.target(output)?;
        if self.base.book.data.records[&cell].position.is_none() {
            return Err(Error::InvalidTarget);
        }
        let id = self.base.book.data.bindings[&cell].runtime;
        self.evaluate(&[output])?
            .get(&id)
            .ok_or(Error::InvalidTarget)?
            .clone()
            .map_err(graph_error)
    }
}

/// Benchmark-only construction. Production constructors and limits are unchanged.
/// Bulk setup skips per-edit staging/quota scans; timed edits use actual apply.
pub fn build(size: usize, shape: &str) -> Workbook {
    assert!((100..=1_000_000).contains(&size));
    let limits = Limits {
        rows: ((size as u32 / 100 + 2) * 2).max(100),
        columns: 202,
        live_cells: size + 2048,
        allocated_cells: size + 4096,
        bytes: 1_073_741_824,
        read_area: 5000,
        read_records: (size + 2048).max(5000),
        ..Limits::default()
    };
    let mut w = Workbook::new_checked("Scaling", limits, true).unwrap();
    for i in 0..size {
        let id = w.ensure_anchor(position(i)).unwrap();
        w.core
            .replace_input(w.data.bindings[&id].runtime, PivotDecimal::ONE)
            .unwrap();
        w.data.records.get_mut(&id).unwrap().content = Some(Content::Decimal(
            DecimalInput::from_decimal(PivotDecimal::ONE),
        ));
    }
    for i in 1..size {
        let target = match shape {
            "independent" if i == size - 1 => Some(0),
            "chain" if i <= 1000 => Some(i - 1),
            "fanout" => Some(0),
            "mixed" if i % 5 == 4 => Some(i - 1),
            _ => None,
        };
        if let Some(target) = target {
            let source = format!("={}+1", address(position(target)));
            w.edit_text(position(i), &source, w.expected(), i).unwrap();
        }
    }
    w.budget().unwrap();
    w
}
pub fn position(i: usize) -> Position {
    Position::new((i / 100 * 2 + 1) as u32, (i % 100 * 2 + 1) as u32)
}
pub fn address(pos: Position) -> String {
    let mut c = pos.column;
    let mut letters = Vec::new();
    while c > 0 {
        c -= 1;
        letters.push((b'A' + (c % 26) as u8) as char);
        c /= 26;
    }
    format!(
        "{}{}",
        letters.into_iter().rev().collect::<String>(),
        pos.row
    )
}

pub fn stage(w: &Workbook) -> Workbook {
    copy(w)
}
pub fn quota(w: &Workbook) {
    w.budget().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(p: Position, n: u32) -> Edit {
        Edit::SetDecimal {
            position: p,
            value: DecimalInput::from_parts(n, 0, 0, false, 0).unwrap(),
        }
    }
    fn formula(p: Position, s: &str) -> Edit {
        Edit::SetFormulaText {
            position: p,
            source: s.into(),
        }
    }
    fn pair() -> (Workbook, SharedWorkbook) {
        let mut w = Workbook::new("Oracle", Limits::default()).unwrap();
        w.apply(
            w.expected(),
            &[
                input(position(0), 100),
                input(position(1), 25),
                formula(position(2), "=A1+C1"),
            ],
        )
        .unwrap();
        let shared = SharedWorkbook::new(copy(&w));
        (w, shared)
    }
    fn compare(w: &Workbook, s: &SharedWorkbook) {
        let wc = w.capture(w.expected());
        let sc = s.capture();
        assert_eq!(wc.is_ok(), sc.is_ok());
        for i in 0..8 {
            let a = w.cell_at(position(i)).unwrap();
            let b = s.cell_at(position(i)).unwrap();
            assert_eq!(
                a.as_ref().map(|c| c.position),
                b.as_ref().map(|c| c.position)
            );
            if let (Some(a), Some(b)) = (a, b) {
                if s.base.book.data.records.contains_key(&a.reference.cell) {
                    assert_eq!((a.reference, a.node), (b.reference, b.node));
                }
                assert_eq!(
                    w.data.bindings[&a.reference.cell].runtime,
                    s.base.book.data.bindings[&b.reference.cell].runtime
                );
                match (&a.content, &b.content) {
                    (Content::TextFormula(a), Content::TextFormula(b)) => {
                        assert_eq!(a.original_source, b.original_source);
                        assert_eq!(a.references.len(), b.references.len());
                        for (a, b) in a.references.iter().zip(&b.references) {
                            assert_eq!(
                                (&a.span, a.submitted_position),
                                (&b.span, b.submitted_position)
                            );
                            assert_eq!(
                                w.data.bindings[&a.target.cell].runtime,
                                s.base.book.data.bindings[&b.target.cell].runtime
                            );
                        }
                        assert_eq!(a.diagnostic, b.diagnostic);
                    }
                    _ => assert_eq!(a.content, b.content),
                }
                if let (Ok(wc), Ok(sc)) = (&wc, &sc) {
                    let original = wc.evaluate(&[a.reference]).unwrap();
                    let id = w.data.bindings[&a.reference.cell].runtime;
                    assert_eq!(
                        original.values[0].1,
                        sc.evaluate(&[b.reference]).unwrap()[&id]
                            .clone()
                            .map_err(|e| match e {
                                EvaluationError::Arithmetic { error, .. } =>
                                    CellError::Arithmetic(error),
                                EvaluationError::DependencyFailed { .. } =>
                                    CellError::DependencyFailed,
                                _ => CellError::InvalidExpression,
                            })
                    );
                }
            }
        }
        if wc.is_err() {
            assert_eq!(
                w.validate(w.expected()).unwrap(),
                s.base.book.validate(s.base.book.expected()).unwrap()
            );
        }
    }
    #[test]
    fn accepted_semantics_and_snapshot_oracle_survive_fast_and_structural_paths() {
        let (mut w, mut s) = pair();
        compare(&w, &s);
        let out = w.cell_at(position(2)).unwrap().unwrap().reference;
        let old = s.capture().unwrap();
        assert_eq!(old.value(out).unwrap(), PivotDecimal::from(125));
        for edits in [
            vec![input(position(0), 200)],
            vec![
                Edit::Move {
                    from: position(0),
                    to: position(3),
                },
                input(position(0), 999),
            ],
            vec![Edit::Clear(position(3))],
            vec![Edit::Retire(position(3)), input(position(3), 500)],
            vec![formula(position(2), "=G1+C1")],
            vec![formula(position(2), "=A1+C1")],
            vec![formula(position(2), "=A0+")],
            vec![input(position(0), 1)],
            vec![formula(position(2), "=1/0")],
            vec![formula(position(2), "=E1")],
            vec![formula(position(2), "=G1+1")],
        ] {
            w.apply(w.expected(), &edits).unwrap();
            s.apply(s.expected(), &edits).unwrap();
            compare(&w, &s);
            assert_eq!(old.value(out).unwrap(), PivotDecimal::from(125));
        }
        assert_ne!(old.version, s.expected());
    }
    #[test]
    fn failed_batches_noops_stale_and_quota_checks_remain_atomic() {
        let (mut w, mut s) = pair();
        let stale = s.expected();
        for edits in [
            vec![input(position(0), 2), input(Position::new(0, 1), 9)],
            vec![
                input(position(0), 2),
                Edit::Move {
                    from: position(1),
                    to: position(0),
                },
            ],
            vec![
                input(position(0), 2),
                formula(position(2), &"x".repeat(4097)),
            ],
        ] {
            let before = s.expected();
            assert!(s.apply(before, &edits).is_err());
            assert_eq!(s.expected(), before);
            compare(&w, &s);
        }
        let before = s.expected();
        s.apply(before, &[input(position(0), 2), input(position(0), 100)])
            .unwrap();
        assert_eq!(s.expected(), before);
        w.apply(w.expected(), &[input(position(0), 2)]).unwrap();
        s.apply(s.expected(), &[input(position(0), 2)]).unwrap();
        compare(&w, &s);
        assert_eq!(
            s.apply(stale, &[input(position(0), 7)]),
            Err(Error::StaleRevision)
        );
        assert_eq!(
            s.apply(
                Version {
                    session: Uuid::new_v4(),
                    sequence: 0
                },
                &[]
            ),
            Err(Error::StaleSession)
        );
        let mut small = Workbook::new(
            "quota",
            Limits {
                live_cells: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        small
            .apply(small.expected(), &[input(position(0), 1)])
            .unwrap();
        let mut small = SharedWorkbook::new(small);
        let before = small.expected();
        assert_eq!(
            small.apply(before, &[input(position(0), 2), input(position(1), 3)]),
            Err(Error::Limit)
        );
        assert_eq!(small.expected(), before);
        assert_eq!(
            small.cell_at(position(0)).unwrap().unwrap().content,
            Content::Decimal(DecimalInput::from_decimal(PivotDecimal::ONE))
        );
    }
    #[test]
    fn many_patches_and_live_generations_match_independent_values() {
        let mut s = SharedWorkbook::new(build(1000, "mixed"));
        let mut generations = vec![];
        for pass in 2..6u32 {
            for i in 0..1000 {
                if i % 5 != 4 {
                    s.apply(s.expected(), &[input(position(i), pass)]).unwrap();
                }
            }
            let capture = s.capture().unwrap();
            let output = s.cell_at(position(999)).unwrap().unwrap().reference;
            assert_eq!(
                capture.value(output).unwrap(),
                PivotDecimal::from(pass as i64 + 1)
            );
            assert_eq!(capture.evaluate(&[output]).unwrap().len(), 2);
            generations.push((capture, output, pass));
        }
        drop(s);
        for (capture, output, pass) in generations {
            assert_eq!(
                capture.value(output).unwrap(),
                PivotDecimal::from(pass as i64 + 1)
            );
        }
    }
    #[test]
    fn full_workload_shapes_match_closed_form_oracles() {
        for shape in ["independent", "chain", "fanout", "mixed"] {
            let mut s = SharedWorkbook::new(build(10000, shape));
            let index = if shape == "chain" { 1000 } else { 9999 };
            let output = s.cell_at(position(index)).unwrap().unwrap().reference;
            let before = s.capture().unwrap();
            let expected = if shape == "chain" { 1001 } else { 2 };
            assert_eq!(before.value(output).unwrap(), PivotDecimal::from(expected));
            s.apply(s.expected(), &[input(position(0), 7)]).unwrap();
            let expected = if shape == "chain" {
                1007
            } else if shape == "mixed" {
                2
            } else {
                8
            };
            assert_eq!(
                s.capture().unwrap().value(output).unwrap(),
                PivotDecimal::from(expected)
            );
            let expected = if shape == "chain" { 1001 } else { 2 };
            assert_eq!(before.value(output).unwrap(), PivotDecimal::from(expected));
        }
    }
}
