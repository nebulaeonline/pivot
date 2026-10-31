//! Current spatial assignments, never a formatting-operation history.
use super::*;
use pivot_core::model::integration::resources::{Index, Key, Memory, Shared};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub sheet: WorksheetId,
    pub property: u8,
    pub rectangle: Rectangle,
    pub value: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MapKey(WorksheetId, u8);
impl Key for MapKey {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    rectangle: Rectangle,
    value: u32,
}
struct Map {
    entries: Vec<Entry>,
    max_row: Vec<u32>,
}
impl Footprint for Map {
    fn heap_bytes(&self) -> usize {
        self.entries.capacity() * std::mem::size_of::<Entry>() + self.max_row.capacity() * 4
    }
}
pub(crate) struct Work<'a> {
    remaining: usize,
    limit: usize,
    cancel: &'a CancellationToken,
}
impl<'a> Work<'a> {
    pub(super) fn new(limit: usize, cancel: &'a CancellationToken) -> Self {
        Self {
            remaining: limit,
            limit,
            cancel,
        }
    }
    pub(super) fn used(&self) -> usize {
        self.limit - self.remaining
    }
    fn tick(&mut self) -> Result<()> {
        if self.cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        self.remaining = self.remaining.checked_sub(1).ok_or(Error::Capacity {
            resource: "formatting work",
            limit: self.limit,
        })?;
        Ok(())
    }
}
fn overlap(a: Rectangle, b: Rectangle) -> bool {
    a.first.row <= b.last.row
        && b.first.row <= a.last.row
        && a.first.column <= b.last.column
        && b.first.column <= a.last.column
}
fn intersection(a: Rectangle, b: Rectangle) -> Rectangle {
    Rectangle {
        first: Position::new(
            a.first.row.max(b.first.row),
            a.first.column.max(b.first.column),
        ),
        last: Position::new(a.last.row.min(b.last.row), a.last.column.min(b.last.column)),
    }
}
fn area(r: Rectangle) -> u64 {
    u64::from(r.last.row - r.first.row + 1) * u64::from(r.last.column - r.first.column + 1)
}
fn subtract(a: Rectangle, b: Rectangle, mut push: impl FnMut(Rectangle)) {
    let i = intersection(a, b);
    if a.first.row < i.first.row {
        push(Rectangle {
            first: a.first,
            last: Position::new(i.first.row - 1, a.last.column),
        });
    }
    if i.last.row < a.last.row {
        push(Rectangle {
            first: Position::new(i.last.row + 1, a.first.column),
            last: a.last,
        });
    }
    if a.first.column < i.first.column {
        push(Rectangle {
            first: Position::new(i.first.row, a.first.column),
            last: Position::new(i.last.row, i.first.column - 1),
        });
    }
    if i.last.column < a.last.column {
        push(Rectangle {
            first: Position::new(i.first.row, i.last.column + 1),
            last: Position::new(i.last.row, a.last.column),
        });
    }
}
impl Map {
    fn build(mut entries: Vec<Entry>, work: &mut Work<'_>) -> Result<Self> {
        // Unchanged regions were already normalized. Paint coalesces new pieces
        // with their neighbors; avoid sorting every old region in both axes.
        let mut comparisons = 0;
        entries.sort_by_key(|e| {
            comparisons += 1;
            let r = e.rectangle;
            (r.first.row, r.first.column, r.last.row, r.last.column)
        });
        for _ in 0..comparisons {
            work.tick()?;
        }
        entries.shrink_to_fit();
        let mut max_row = vec![0; entries.len()];
        for _ in &entries {
            work.tick()?;
        }
        fn index(entries: &[Entry], max: &mut [u32]) -> u32 {
            if entries.is_empty() {
                return 0;
            }
            let m = entries.len() / 2;
            let (left, rest) = max.split_at_mut(m);
            let (mid, right) = rest.split_at_mut(1);
            mid[0] = entries[m]
                .rectangle
                .last
                .row
                .max(index(&entries[..m], left))
                .max(index(&entries[m + 1..], right));
            mid[0]
        }
        index(&entries, &mut max_row);
        Ok(Self { entries, max_row })
    }
    fn query(
        &self,
        r: Rectangle,
        work: &mut Work<'_>,
        mut visit: impl FnMut(Entry) -> Result<()>,
    ) -> Result<()> {
        fn walk(
            entries: &[Entry],
            max: &[u32],
            r: Rectangle,
            work: &mut Work<'_>,
            visit: &mut impl FnMut(Entry) -> Result<()>,
        ) -> Result<()> {
            if entries.is_empty() {
                return Ok(());
            }
            work.tick()?;
            let m = entries.len() / 2;
            if max[m] < r.first.row {
                return Ok(());
            }
            walk(&entries[..m], &max[..m], r, work, visit)?;
            if entries[m].rectangle.first.row > r.last.row {
                return Ok(());
            }
            if overlap(entries[m].rectangle, r) {
                visit(entries[m])?;
            }
            walk(&entries[m + 1..], &max[m + 1..], r, work, visit)
        }
        walk(&self.entries, &self.max_row, r, work, &mut visit)
    }
}
pub(crate) fn values(s: Style) -> [u32; 10] {
    let b = s.encode();
    let mut v = [0; 10];
    for i in 0..8 {
        v[i] = b[i].into();
    }
    v[8] = u32::from_le_bytes(b[9..13].try_into().unwrap());
    v[9] = u32::from_le_bytes(b[13..17].try_into().unwrap());
    v
}
pub(super) fn from_values(v: [u32; 10]) -> Result<Style> {
    let mut b = [0; 17];
    for i in 0..8 {
        b[i] = u8::try_from(v[i]).map_err(|_| Error::InvalidTarget)?;
    }
    b[9..13].copy_from_slice(&v[8].to_le_bytes());
    b[13..17].copy_from_slice(&v[9].to_le_bytes());
    Style::decode(&b)
}
#[derive(Clone)]
pub(crate) struct Store {
    maps: Index<MapKey, Shared<Map>>,
    defaults: Index<MapKey, Style>,
    memory: Memory,
    pub(crate) work: usize,
    count: usize,
    bytes: usize,
}
impl Store {
    pub(crate) fn permute_rows(
        &mut self,
        sheet: WorksheetId,
        rectangle: Rectangle,
        permutation: &[u32],
        limits: StyleLimits,
        budget: usize,
        cancel: &CancellationToken,
    ) -> Result<(bool, usize)> {
        let cap = limits.positions.saturating_mul(5);
        let transient = self
            .count
            .saturating_mul(5)
            .saturating_add(permutation.len())
            .min(cap)
            .max(1)
            .next_power_of_two();
        let _scratch = self
            .memory
            .reserve(transient * std::mem::size_of::<Entry>() * 3 + permutation.len() * 16 + 8192)
            .map_err(|_| Error::Limit)?;
        let mut work = Work::new(budget, cancel);
        let mut changed = false;
        for property in 0..10 {
            let key = MapKey(sheet, property);
            let Some(old) = self.maps.get(&key).cloned() else {
                continue;
            };
            let mut entries = Vec::new();
            for &e in &old.entries {
                work.tick()?;
                if !overlap(e.rectangle, rectangle)
                    || (e.rectangle.first.row <= rectangle.first.row
                        && e.rectangle.last.row >= rectangle.last.row)
                {
                    // A constant assignment across the entire permuted row interval is invariant.
                    entries.push(e);
                    continue;
                }
                let i = intersection(e.rectangle, rectangle);
                subtract(e.rectangle, rectangle, |r| {
                    entries.push(Entry { rectangle: r, ..e })
                });
                let mut rows = Vec::with_capacity((i.last.row - i.first.row + 1) as usize);
                for row in i.first.row..=i.last.row {
                    work.tick()?;
                    rows.push(permutation[(row - rectangle.first.row) as usize]);
                }
                let moves = crate::sorting::sort_move_units(rows.len());
                for _ in 0..moves {
                    work.tick()?;
                }
                crate::sorting::sort_by(&mut rows, |a, b| {
                    work.tick()?;
                    Ok(a.cmp(&b))
                })?;
                let mut start = 0;
                while start < rows.len() {
                    let mut end = start;
                    while end + 1 < rows.len() && rows[end] + 1 == rows[end + 1] {
                        work.tick()?;
                        end += 1;
                    }
                    work.tick()?;
                    entries.push(Entry {
                        rectangle: Rectangle {
                            first: Position::new(rows[start], i.first.column),
                            last: Position::new(rows[end], i.last.column),
                        },
                        ..e
                    });
                    if entries.len() > cap {
                        return Err(Error::Capacity {
                            resource: "sorting format fragments",
                            limit: limits.positions,
                        });
                    }
                    start = end + 1;
                }
            }
            for vertical in [false, true] {
                for _ in 0..crate::sorting::sort_move_units(entries.len()) {
                    work.tick()?;
                }
                crate::sorting::sort_by(&mut entries, |a, b| {
                    work.tick()?;
                    let key = |e: Entry| {
                        let r = e.rectangle;
                        if vertical {
                            (e.value, r.first.column, r.last.column, r.first.row)
                        } else {
                            (e.value, r.first.row, r.last.row, r.first.column)
                        }
                    };
                    Ok(key(a).cmp(&key(b)))
                })?;
                let mut n = 0;
                for i in 0..entries.len() {
                    work.tick()?;
                    let e = entries[i];
                    if n > 0 {
                        let previous: &mut Entry = &mut entries[n - 1];
                        let (a, b) = (previous.rectangle, e.rectangle);
                        if previous.value == e.value
                            && if vertical {
                                a.first.column == b.first.column
                                    && a.last.column == b.last.column
                                    && a.last.row + 1 == b.first.row
                            } else {
                                a.first.row == b.first.row
                                    && a.last.row == b.last.row
                                    && a.last.column + 1 == b.first.column
                            }
                        {
                            previous.rectangle.last = b.last;
                            continue;
                        }
                    }
                    entries[n] = e;
                    n += 1;
                }
                entries.truncate(n);
            }
            let map = Map::build(entries, &mut work)?;
            let same = if old.entries == map.entries {
                true
            } else if old.entries.iter().map(|e| area(e.rectangle)).sum::<u64>()
                != map.entries.iter().map(|e| area(e.rectangle)).sum::<u64>()
            {
                false
            } else {
                let mut same = true;
                for e in &old.entries {
                    let mut covered = 0;
                    map.query(e.rectangle, &mut work, |other| {
                        if e.value == other.value {
                            covered += area(intersection(e.rectangle, other.rectangle));
                        }
                        Ok(())
                    })?;
                    if covered != area(e.rectangle) {
                        same = false;
                        break;
                    }
                }
                same
            };
            if same {
                continue;
            }
            self.count = self.count - old.entries.len() + map.entries.len();
            self.bytes = self.bytes - old.heap_bytes() + map.heap_bytes();
            if self.count > limits.positions || self.bytes() > limits.bytes {
                return Err(Error::Capacity {
                    resource: "sorting formatting capacity",
                    limit: limits.positions,
                });
            }
            if map.entries.is_empty() {
                self.maps.remove(&key).map_err(|_| Error::Limit)?;
            } else {
                let map = self.memory.alloc(map).map_err(|_| Error::Limit)?;
                self.maps.insert(key, map).map_err(|_| Error::Limit)?;
            }
            changed = true;
        }
        Ok((changed, work.used()))
    }
    pub(crate) fn structural(
        &mut self,
        sheet: WorksheetId,
        change: crate::structure::Change,
        limit: u32,
        limits: StyleLimits,
        cancel: &CancellationToken,
    ) -> Result<(bool, bool, usize)> {
        use crate::structure::{Action, AxisKind};
        let _scratch = self
            .memory
            .reserve(self.count.saturating_mul(240) + 4096)
            .map_err(|_| Error::Limit)?;
        let mut work = Work::new(crate::structure::MAX_WORK, cancel);
        let mut changed = false;
        let mut clipped = false;
        let profile = match change.action {
            Action::Insert => Some(change.start.saturating_sub(1).max(1)),
            Action::Delete => {
                if change.start + change.count <= limit {
                    Some(limit)
                } else {
                    change.start.checked_sub(1).filter(|x| *x > 0)
                }
            }
        };
        for property in 0..10 {
            let key = MapKey(sheet, property);
            let Some(old) = self.maps.get(&key).cloned() else {
                continue;
            };
            let axis = |r: Rectangle| match change.axis {
                AxisKind::Rows => (r.first.row, r.last.row),
                AxisKind::Columns => (r.first.column, r.last.column),
            };
            let band = |r: Rectangle, a: u32, b: u32| match change.axis {
                AxisKind::Rows => Rectangle {
                    first: Position::new(a, r.first.column),
                    last: Position::new(b, r.last.column),
                },
                AxisKind::Columns => Rectangle {
                    first: Position::new(r.first.row, a),
                    last: Position::new(r.last.row, b),
                },
            };
            let mut entries = Vec::new();
            for &e in &old.entries {
                work.tick()?;
                let (a, b) = axis(e.rectangle);
                match change.action {
                    Action::Insert => {
                        if b >= change.start.max(limit - change.count + 1) {
                            clipped = true;
                        }
                        if a < change.start {
                            entries.push(Entry {
                                rectangle: band(e.rectangle, a, b.min(change.start - 1)),
                                ..e
                            });
                        }
                        if b >= change.start {
                            let first = a.max(change.start) + change.count;
                            if first <= limit {
                                entries.push(Entry {
                                    rectangle: band(
                                        e.rectangle,
                                        first,
                                        (b + change.count).min(limit),
                                    ),
                                    ..e
                                });
                            }
                        }
                    }
                    Action::Delete => {
                        if let Some((a, b)) = change.range(a, b, limit) {
                            entries.push(Entry {
                                rectangle: band(e.rectangle, a, b),
                                ..e
                            });
                        }
                    }
                }
                if profile.is_some_and(|p| a <= p && p <= b) {
                    let (first, last) = match change.action {
                        Action::Insert => (change.start, change.start + change.count - 1),
                        Action::Delete => (limit - change.count + 1, limit),
                    };
                    entries.push(Entry {
                        rectangle: band(e.rectangle, first, last),
                        ..e
                    });
                }
            }
            // Axis-local normalization after mapping; representation is current
            // assignments, never a log of inserted/deleted rows.
            for vertical in [false, true] {
                let mut comparisons = 0;
                entries.sort_by_key(|e| {
                    comparisons += 1;
                    let r = e.rectangle;
                    if vertical {
                        (e.value, r.first.column, r.last.column, r.first.row)
                    } else {
                        (e.value, r.first.row, r.last.row, r.first.column)
                    }
                });
                for _ in 0..comparisons {
                    work.tick()?;
                }
                let mut n = 0;
                for i in 0..entries.len() {
                    work.tick()?;
                    let e = entries[i];
                    if n > 0 {
                        let p: &mut Entry = &mut entries[n - 1];
                        let a = p.rectangle;
                        let b = e.rectangle;
                        if p.value == e.value
                            && if vertical {
                                a.first.column == b.first.column
                                    && a.last.column == b.last.column
                                    && a.last.row + 1 == b.first.row
                            } else {
                                a.first.row == b.first.row
                                    && a.last.row == b.last.row
                                    && a.last.column + 1 == b.first.column
                            }
                        {
                            p.rectangle.last = b.last;
                            continue;
                        }
                    }
                    entries[n] = e;
                    n += 1;
                }
                entries.truncate(n);
            }
            let map = Map::build(entries, &mut work)?;
            let same = if old.entries == map.entries {
                true
            } else if old.entries.iter().map(|e| area(e.rectangle)).sum::<u64>()
                != map.entries.iter().map(|e| area(e.rectangle)).sum::<u64>()
            {
                false
            } else {
                let mut same = true;
                for e in &old.entries {
                    let mut covered = 0;
                    map.query(e.rectangle, &mut work, |other| {
                        if e.value == other.value {
                            covered += area(intersection(e.rectangle, other.rectangle));
                        }
                        Ok(())
                    })?;
                    if covered != area(e.rectangle) {
                        same = false;
                        break;
                    }
                }
                same
            };
            if same {
                continue;
            }
            self.count = self.count - old.entries.len() + map.entries.len();
            self.bytes = self.bytes - old.heap_bytes() + map.heap_bytes();
            if self.count > limits.positions || self.bytes() > limits.bytes {
                return Err(Error::Limit);
            }
            if map.entries.is_empty() {
                self.maps.remove(&key).map_err(|_| Error::Limit)?;
            } else {
                let map = self.memory.alloc(map).map_err(|_| Error::Limit)?;
                self.maps.insert(key, map).map_err(|_| Error::Limit)?;
            }
            changed = true;
        }
        Ok((changed, clipped, work.used()))
    }
    pub(crate) fn viewport(
        &self,
        sheet: WorksheetId,
        r: Rectangle,
        limits: StyleLimits,
    ) -> Result<Vec<(Position, Style)>> {
        let cancel = CancellationToken::default();
        let mut work = Work::new(limits.query_work, &cancel);
        let width = (r.last.column - r.first.column + 1) as usize;
        let count = area(r) as usize;
        let _scratch = self
            .memory
            .reserve(count * 128 + 4096)
            .map_err(|_| Error::Limit)?;
        let mut v = vec![values(self.default_style(sheet)); count];
        let mut writes = 0usize;
        for property in 0..10 {
            if let Some(m) = self.maps.get(&MapKey(sheet, property)) {
                m.query(r, &mut work, |e| {
                    let i = intersection(r, e.rectangle);
                    writes += area(i) as usize;
                    for row in i.first.row..=i.last.row {
                        for col in i.first.column..=i.last.column {
                            v[(row - r.first.row) as usize * width
                                + (col - r.first.column) as usize]
                                [property as usize] = e.value;
                        }
                    }
                    Ok(())
                })?;
            }
        }
        for _ in 0..writes {
            work.tick()?;
        }
        let mut out = Vec::with_capacity(count);
        for (i, v) in v.into_iter().enumerate() {
            work.tick()?;
            out.push((
                Position::new(
                    r.first.row + (i / width) as u32,
                    r.first.column + (i % width) as u32,
                ),
                from_values(v)?,
            ));
        }
        Ok(out)
    }
    pub(crate) fn restore(
        &mut self,
        defaults: &[(WorksheetId, Style)],
        regions: &[Region],
        limits: StyleLimits,
    ) -> Result<()> {
        use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
        if regions.len() > limits.positions {
            return Err(Error::Limit);
        }
        let _scratch = self
            .memory
            .reserve(regions.len() * 160 + 4096)
            .map_err(|_| Error::Limit)?;
        let cancel = CancellationToken::default();
        let mut work = Work::new(64 * 1024 * 1024, &cancel);
        let mut seen = BTreeSet::new();
        for &(sheet, style) in defaults {
            if !seen.insert(sheet) || style == Style::default() {
                return Err(Error::InvalidTarget);
            }
            self.set_default(sheet, style)?;
        }
        let mut grouped: BTreeMap<MapKey, Vec<Entry>> = BTreeMap::new();
        for &r in regions {
            if r.property >= 10 {
                return Err(Error::InvalidTarget);
            }
            let mut v = values(Style::default());
            v[r.property as usize] = r.value;
            from_values(v)?;
            grouped
                .entry(MapKey(r.sheet, r.property))
                .or_default()
                .push(Entry {
                    rectangle: r.rectangle,
                    value: r.value,
                });
        }
        for (key, mut entries) in grouped {
            entries.sort_by_key(|e| (e.rectangle.first.row, e.rectangle.first.column));
            // Sweep active, mutually disjoint column intervals. End events remove
            // rectangles before a later row; predecessor/successor detect overlaps.
            let mut active: BTreeMap<u32, u32> = BTreeMap::new();
            let mut ends = BinaryHeap::new();
            for e in &entries {
                work.tick()?;
                let r = e.rectangle;
                while ends
                    .peek()
                    .is_some_and(|std::cmp::Reverse((end, _))| *end < r.first.row)
                {
                    let std::cmp::Reverse((_, col)) = ends.pop().unwrap();
                    active.remove(&col);
                }
                if active
                    .range(..=r.last.column)
                    .next_back()
                    .is_some_and(|(&first, &last)| {
                        first >= r.first.column || last >= r.first.column
                    })
                {
                    return Err(Error::InvalidTarget);
                }
                active.insert(r.first.column, r.last.column);
                ends.push(std::cmp::Reverse((r.last.row, r.first.column)));
            }
            let map = Map::build(entries, &mut work)?;
            self.count += map.entries.len();
            self.bytes += map.heap_bytes();
            let map = self.memory.alloc(map).map_err(|_| Error::Limit)?;
            self.maps.insert(key, map).map_err(|_| Error::Limit)?;
        }
        if self.bytes() > limits.bytes {
            return Err(Error::Limit);
        }
        Ok(())
    }
    pub(crate) fn new(memory: Memory) -> Self {
        Self {
            maps: Index::new(memory.clone()),
            defaults: Index::new(memory.clone()),
            memory,
            count: 0,
            work: 0,
            bytes: 0,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.count
    }
    pub(crate) fn bytes(&self) -> usize {
        self.bytes + self.maps.len() * 256 + self.defaults.len() * 256
    }
    pub(crate) fn default_style(&self, sheet: WorksheetId) -> Style {
        self.defaults
            .get(&MapKey(sheet, 10))
            .copied()
            .unwrap_or_default()
    }
    pub(crate) fn defaults(&self) -> impl Iterator<Item = (WorksheetId, Style)> + '_ {
        self.defaults.iter().map(|(k, v)| (k.0, *v))
    }
    pub(crate) fn regions(&self) -> impl Iterator<Item = Region> + '_ {
        self.maps.iter().flat_map(|(k, m)| {
            m.entries.iter().map(move |e| Region {
                sheet: k.0,
                property: k.1,
                rectangle: e.rectangle,
                value: e.value,
            })
        })
    }
    pub(crate) fn set_default(&mut self, sheet: WorksheetId, style: Style) -> Result<bool> {
        style.validate()?;
        if self.default_style(sheet) == style {
            return Ok(false);
        }
        if style == Style::default() {
            self.defaults
                .remove(&MapKey(sheet, 10))
                .map_err(|_| Error::Limit)?;
        } else {
            self.defaults
                .insert(MapKey(sheet, 10), style)
                .map_err(|_| Error::Limit)?;
        }
        Ok(true)
    }
    pub(crate) fn style_at(
        &self,
        sheet: WorksheetId,
        p: Position,
        work: &mut Work<'_>,
    ) -> Result<Style> {
        let mut v = values(self.default_style(sheet));
        let r = Rectangle { first: p, last: p };
        for property in 0..10 {
            if let Some(m) = self.maps.get(&MapKey(sheet, property)) {
                m.query(r, work, |e| {
                    v[property as usize] = e.value;
                    Ok(())
                })?;
            }
        }
        from_values(v)
    }
    pub(crate) fn summary(
        &self,
        sheet: WorksheetId,
        r: Rectangle,
        work: &mut Work<'_>,
    ) -> Result<(Style, u32)> {
        let defaults = values(self.default_style(sheet));
        let mut v = defaults;
        let mut mixed = 0;
        for property in 0..10 {
            let mut seen = None;
            let mut covered = 0;
            if let Some(m) = self.maps.get(&MapKey(sheet, property)) {
                m.query(r, work, |e| {
                    covered += area(intersection(r, e.rectangle));
                    if seen.is_some_and(|x| x != e.value) {
                        mixed |= 1 << property;
                    }
                    seen = Some(e.value);
                    Ok(())
                })?;
            }
            if let Some(value) = seen {
                v[property as usize] = value;
                if covered < area(r) && value != defaults[property as usize] {
                    mixed |= 1 << property;
                }
            }
        }
        Ok((from_values(v)?, mixed))
    }
    pub(crate) fn paint(
        &mut self,
        sheet: WorksheetId,
        r: Rectangle,
        property: u8,
        value: Option<u32>,
        limits: StyleLimits,
        work: &mut Work<'_>,
    ) -> Result<bool> {
        if property >= 10 {
            return Err(Error::InvalidTarget);
        }
        if let Some(value) = value {
            let mut v = values(Style::default());
            v[property as usize] = value;
            from_values(v)?;
        }
        let key = MapKey(sheet, property);
        let old = self.maps.get(&key);
        let mut same = 0;
        let mut hits = 0;
        if let Some(m) = old {
            m.query(r, work, |e| {
                hits += 1;
                if Some(e.value) == value {
                    same += area(intersection(r, e.rectangle));
                }
                Ok(())
            })?;
        }
        if (value.is_none() && hits == 0) || (value.is_some() && same == area(r)) {
            return Ok(false);
        }
        let old_len = old.map_or(0, |m| m.entries.len());
        let scratch = self
            .memory
            .reserve((old_len * 4 + 1) * std::mem::size_of::<Entry>() * 2 + 4096)
            .map_err(|_| Error::Limit)?;
        let mut entries = Vec::with_capacity(old_len * 4 + 1);
        let mut additions = Vec::new();
        if let Some(m) = old {
            for &e in &m.entries {
                work.tick()?;
                if overlap(e.rectangle, r) {
                    subtract(e.rectangle, r, |rectangle| {
                        additions.push(Entry {
                            rectangle,
                            value: e.value,
                        })
                    });
                } else {
                    entries.push(e);
                }
            }
        }
        if let Some(value) = value {
            additions.push(Entry {
                rectangle: r,
                value,
            });
        }
        // Only replacements/split pieces can expose new merge opportunities.
        // Two deterministic axis passes inspect current neighbors under the same
        // work authority; unrelated properties and old region partitions stay put.
        for mut entry in additions {
            for vertical in [false, true] {
                let mut i = 0;
                while i < entries.len() {
                    work.tick()?;
                    let a = entry.rectangle;
                    let b = entries[i].rectangle;
                    let adjacent = if vertical {
                        a.first.column == b.first.column
                            && a.last.column == b.last.column
                            && (a.last.row + 1 == b.first.row || b.last.row + 1 == a.first.row)
                    } else {
                        a.first.row == b.first.row
                            && a.last.row == b.last.row
                            && (a.last.column + 1 == b.first.column
                                || b.last.column + 1 == a.first.column)
                    };
                    if entry.value == entries[i].value && adjacent {
                        entry.rectangle = Rectangle {
                            first: Position::new(
                                a.first.row.min(b.first.row),
                                a.first.column.min(b.first.column),
                            ),
                            last: Position::new(
                                a.last.row.max(b.last.row),
                                a.last.column.max(b.last.column),
                            ),
                        };
                        entries.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            entries.push(entry);
        }
        let map = Map::build(entries, work)?;
        let next = self.count - old_len + map.entries.len();
        if next > limits.positions {
            return Err(Error::Capacity {
                resource: "format property regions",
                limit: limits.positions,
            });
        }
        let old_bytes = old.map_or(0, |m| m.heap_bytes());
        let bytes = self.bytes - old_bytes + map.heap_bytes();
        if bytes + (self.maps.len() + 1) * 256 + self.defaults.len() * 256 > limits.bytes {
            return Err(Error::Capacity {
                resource: "format bytes",
                limit: limits.bytes,
            });
        }
        if map.entries.is_empty() {
            self.maps.remove(&key).map_err(|_| Error::Limit)?;
        } else {
            let map = self.memory.alloc(map).map_err(|_| Error::Limit)?;
            self.maps.insert(key, map).map_err(|_| Error::Limit)?;
        }
        self.count = next;
        self.bytes = bytes;
        drop(scratch);
        Ok(true)
    }
}
