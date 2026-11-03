//! Canonical fixed-point section map. Inputs are resolved presentation profiles,
//! never cell or calculation identities. Workbook roots resolve authored anchors
//! before building this immutable, domain-charged map.
use crate::{Position, Rectangle};
use pivot_core::model::integration::{
    CancellationToken,
    resources::{Charge, Memory},
};
use std::collections::BTreeMap;

pub const UNITS_PER_POINT: i64 = 64;
pub const MIN_SIZE: i64 = UNITS_PER_POINT;
pub const MAX_SIZE: i64 = 4096 * UNITS_PER_POINT;
pub const DEFAULT_WIDTH: i64 = 4512;
pub const DEFAULT_HEIGHT: i64 = 1248;
pub const MAX_BANDS: usize = 64;
pub const MAX_RUNS: usize = 16_384;
pub const MAX_TILES: usize = 65_536;
pub const MAX_PLAN_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_WORK: usize = 10_000_000;
pub(crate) mod stored;
pub use stored::{
    Anchor, Command, GeometryImage, ProfileImage, ProfilePreview, SectionImage, SizeImage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    Invalid,
    Limit,
    Cancelled,
    Overlap { first: Position, second: Position },
}
pub type MapResult<T> = std::result::Result<T, MapError>;

/// Inclusive logical interval with a constant positive size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeRun {
    pub first: u32,
    pub last: u32,
    pub size: i64,
}
/// Resolved bands start at rank one and cover the extent. Width runs cover all
/// columns for a horizontal band; height runs cover all rows for a vertical band.
#[derive(Clone, Copy, Debug)]
pub struct Band<'a> {
    pub start: u32,
    pub sizes: &'a [SizeRun],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRect {
    pub left: i64,
    pub top: i64,
    pub right: i64,
    pub bottom: i64,
}
impl CellRect {
    pub fn contains(self, x: i64, y: i64) -> bool {
        self.left <= x && x < self.right && self.top <= y && y < self.bottom
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub requested: Rectangle,
    pub effective: Rectangle,
    pub sections: u32,
    pub enlarged: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocatedRect {
    pub position: Position,
    pub rectangle: CellRect,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub column: bool,
    pub index: u32,
    pub first: i64,
    pub last: i64,
    pub section_first: u32,
    pub section_last: u32,
}
#[derive(Clone, Copy)]
struct HeaderRun {
    band: usize,
    first: u32,
    last: u32,
    origin: i64,
    size: i64,
    visible_start: usize,
}
pub struct Headers {
    pub items: Vec<Header>,
    _charge: Charge,
}
pub struct ViewportPage {
    pub cells: Vec<LocatedRect>,
    pub total: usize,
    pub next: Option<usize>,
    _charge: Charge,
}
#[derive(Clone, Copy, Debug)]
struct Tile {
    logical: Rectangle,
    physical: CellRect,
    width: i64,
    height: i64,
    visible_start: usize,
}
/// Owns its complete geometry and optional visible-row rank/select vector. The
/// reservation includes conservative sweep-node storage and stays with the plan,
/// so even retained plans cannot escape domain admission. It is accounting, not RSS.
pub struct Map {
    pub(crate) construction_work: usize,
    rows: u32,
    columns: u32,
    horizontal: Vec<u32>,
    vertical: Vec<u32>,
    visible: Option<Vec<u32>>,
    tiles: Vec<Tile>,
    x_headers: Vec<HeaderRun>,
    y_headers: Vec<HeaderRun>,
    _charge: Charge,
}
pub(crate) struct BuildInput<'a> {
    pub rows: u32,
    pub columns: u32,
    pub horizontal: &'a [Band<'a>],
    pub vertical: &'a [Band<'a>],
    pub visible: Option<&'a [u32]>,
}
struct Work<'a> {
    remaining: usize,
    cancel: &'a CancellationToken,
}
impl Work<'_> {
    fn tick(&mut self, n: usize) -> MapResult<()> {
        if self.cancel.is_cancelled() {
            return Err(MapError::Cancelled);
        }
        self.remaining = self.remaining.checked_sub(n).ok_or(MapError::Limit)?;
        Ok(())
    }
}
fn validate_bands(
    bands: &[Band<'_>],
    length: u32,
    opposite: u32,
    work: &mut Work<'_>,
) -> MapResult<usize> {
    if bands.is_empty() || bands.len() > MAX_BANDS || bands[0].start != 1 {
        return Err(MapError::Invalid);
    }
    let mut previous = 0;
    let mut count = 0usize;
    for band in bands {
        work.tick(1)?;
        if band.start <= previous || band.start > length || band.sizes.is_empty() {
            return Err(MapError::Invalid);
        }
        previous = band.start;
        count = count.checked_add(band.sizes.len()).ok_or(MapError::Limit)?;
        if count > MAX_RUNS {
            return Err(MapError::Limit);
        }
        let mut next = 1;
        for run in band.sizes {
            work.tick(1)?;
            if run.first != next
                || run.last < run.first
                || run.last > opposite
                || !(MIN_SIZE..=MAX_SIZE).contains(&run.size)
            {
                return Err(MapError::Invalid);
            }
            next = run.last.checked_add(1).ok_or(MapError::Invalid)?;
        }
        if next != opposite + 1 {
            return Err(MapError::Invalid);
        }
    }
    Ok(count)
}
fn band_end(bands: &[Band<'_>], index: usize, length: u32) -> u32 {
    bands.get(index + 1).map_or(length, |b| b.start - 1)
}
fn rank(visible: Option<&[u32]>, before: u32) -> usize {
    visible.map_or((before - 1) as usize, |v| {
        v.partition_point(|r| *r < before)
    })
}
fn prefix(
    runs: &[SizeRun],
    before: u32,
    visible: Option<&[u32]>,
    work: &mut Work<'_>,
) -> MapResult<i64> {
    let mut sum = 0i64;
    for run in runs {
        work.tick(1)?;
        if run.first >= before {
            break;
        }
        let end = (run.last + 1).min(before);
        let count = rank(visible, end) - rank(visible, run.first);
        sum = sum
            .checked_add(
                run.size
                    .checked_mul(count as i64)
                    .ok_or(MapError::Invalid)?,
            )
            .ok_or(MapError::Invalid)?;
    }
    Ok(sum)
}
impl Map {
    pub fn construction_work(&self) -> usize {
        self.construction_work
    }
    /// Rulers use the active section's canonical profiles, even when its active
    /// cell is outside the viewport. Header targets carry their logical band.
    pub fn headers(
        &self,
        active: Position,
        view: CellRect,
        memory: &Memory,
        cancel: &CancellationToken,
    ) -> MapResult<Headers> {
        let charge = memory
            .reserve(4096 * std::mem::size_of::<Header>())
            .map_err(|_| MapError::Limit)?;
        let mut items = Vec::with_capacity(4096);
        if self.rows == 0 || self.columns == 0 {
            return Ok(Headers {
                items,
                _charge: charge,
            });
        }
        let row = active.row.clamp(1, self.rows);
        let column = active.column.clamp(1, self.columns);
        let hi = self.horizontal.partition_point(|r| *r <= row) - 1;
        let vi = self.vertical.partition_point(|c| *c <= column) - 1;
        let mut work = Work {
            remaining: MAX_WORK,
            cancel,
        };
        for columns in [true, false] {
            let (runs, band, start, end, limit, section_first, section_last) = if columns {
                (
                    &self.x_headers,
                    hi,
                    view.left,
                    view.right,
                    self.columns,
                    self.horizontal[hi],
                    self.horizontal.get(hi + 1).map_or(self.rows, |n| n - 1),
                )
            } else {
                (
                    &self.y_headers,
                    vi,
                    view.top,
                    view.bottom,
                    self.rows,
                    self.vertical[vi],
                    self.vertical.get(vi + 1).map_or(self.columns, |n| n - 1),
                )
            };
            for run in runs {
                work.tick(1)?;
                if run.band != band || run.first > limit {
                    continue;
                }
                let count = if columns {
                    run.last.min(limit) - run.first + 1
                } else {
                    (rank(self.visible.as_deref(), run.last.min(limit) + 1) - run.visible_start)
                        as u32
                };
                let last = run.origin + i64::from(count) * run.size;
                if start >= last || end <= run.origin || count == 0 {
                    continue;
                }
                let first_offset = (start.max(run.origin) - run.origin) / run.size;
                let last_offset = (end.min(last) - 1 - run.origin) / run.size;
                for offset in first_offset..=last_offset {
                    work.tick(1)?;
                    if items.len() == 4096 {
                        return Err(MapError::Limit);
                    }
                    let index = if columns {
                        run.first + offset as u32
                    } else {
                        self.visible
                            .as_ref()
                            .map_or(run.first + offset as u32, |v| {
                                v[run.visible_start + offset as usize]
                            })
                    };
                    items.push(Header {
                        column: columns,
                        index,
                        first: run.origin + offset * run.size,
                        last: run.origin + (offset + 1) * run.size,
                        section_first,
                        section_last,
                    });
                }
            }
        }
        Ok(Headers {
            items,
            _charge: charge,
        })
    }
    pub fn row_is_visible(&self, row: u32) -> bool {
        row > 0
            && row <= self.rows
            && self
                .visible
                .as_ref()
                .is_none_or(|v| v.binary_search(&row).is_ok())
    }
    pub fn scroll_anchor(&self, position: Position) -> Option<Position> {
        if self.rows == 0 || self.columns == 0 {
            return None;
        }
        let mut row = position.row.clamp(1, self.rows);
        if let Some(visible) = &self.visible {
            let index = visible.partition_point(|r| *r < row);
            row = *visible.get(index).or_else(|| visible.last())?;
        }
        Some(Position::new(row, position.column.clamp(1, self.columns)))
    }
    pub fn scroll_origin(&self, position: Position) -> Option<(i64, i64)> {
        let anchor = self.scroll_anchor(position)?;
        let rectangle = self.cell_rect(anchor)?;
        Some((rectangle.left, rectangle.top))
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.rows, self.columns)
    }
    /// Clip a fully validated bound presentation envelope to actual captured data.
    /// Dormant root metadata remains intact; absent sources expose no cell targets.
    pub(crate) fn clip_extent(&mut self, rows: u32, columns: u32) -> MapResult<()> {
        if rows > self.rows || columns > self.columns {
            return Err(MapError::Invalid);
        }
        self.rows = rows;
        self.columns = columns;
        self.horizontal.retain(|r| *r <= rows);
        self.vertical.retain(|c| *c <= columns);
        if rows == 0 || columns == 0 {
            self.tiles.clear();
            return Ok(());
        }
        let visible = self.visible.as_deref();
        self.tiles.retain_mut(|t| {
            if t.logical.first.row > rows || t.logical.first.column > columns {
                return false;
            }
            t.logical.last.row = t.logical.last.row.min(rows);
            t.logical.last.column = t.logical.last.column.min(columns);
            let count = rank(visible, t.logical.last.row + 1) - t.visible_start;
            t.physical.right = t.physical.left
                + i64::from(t.logical.last.column - t.logical.first.column + 1) * t.width;
            t.physical.bottom = t.physical.top + count as i64 * t.height;
            count > 0
        });
        Ok(())
    }
    pub fn horizontal_starts(&self) -> &[u32] {
        &self.horizontal
    }
    pub fn vertical_starts(&self) -> &[u32] {
        &self.vertical
    }
    /// Viewport membership is computed tile-wise. No per-cell hit-test scan and
    /// no logical-area enumeration. Each page respects the existing 256-cell cap.
    pub fn viewport_page(
        &self,
        view: CellRect,
        after: usize,
        memory: &Memory,
        cancel: &CancellationToken,
    ) -> MapResult<ViewportPage> {
        if view.left >= view.right || view.top >= view.bottom || after > 4096 {
            return Err(MapError::Invalid);
        }
        let mut work = Work {
            remaining: MAX_WORK,
            cancel,
        };
        let bounds = |t: &Tile| -> Option<(i64, i64, i64, i64)> {
            if !(view.left < t.physical.right
                && view.right > t.physical.left
                && view.top < t.physical.bottom
                && view.bottom > t.physical.top)
            {
                return None;
            }
            Some((
                (view.left.max(t.physical.left) - t.physical.left) / t.width,
                (view.top.max(t.physical.top) - t.physical.top) / t.height,
                (view.right.min(t.physical.right) - 1 - t.physical.left) / t.width,
                (view.bottom.min(t.physical.bottom) - 1 - t.physical.top) / t.height,
            ))
        };
        let mut total = 0usize;
        for t in &self.tiles {
            work.tick(1)?;
            if let Some((a, b, c, d)) = bounds(t) {
                total = total
                    .checked_add(((c - a + 1) * (d - b + 1)) as usize)
                    .filter(|n| *n <= 4096)
                    .ok_or(MapError::Limit)?;
            }
        }
        if after > total {
            return Err(MapError::Invalid);
        }
        let count = (total - after).min(256);
        // Reserve server rectangles and a conservative mirrored client allowance.
        let charge = memory
            .reserve(count * std::mem::size_of::<LocatedRect>() * 2 + 256)
            .map_err(|_| MapError::Limit)?;
        let mut cells = Vec::with_capacity(count);
        let mut ordinal = 0;
        for t in &self.tiles {
            work.tick(1)?;
            if let Some((a, b, c, d)) = bounds(t) {
                let tile_count = ((c - a + 1) * (d - b + 1)) as usize;
                if ordinal + tile_count <= after {
                    ordinal += tile_count;
                    continue;
                }
                for row in b..=d {
                    for column in a..=c {
                        work.tick(1)?;
                        if ordinal >= after && cells.len() < count {
                            let left = t.physical.left + column * t.width;
                            let top = t.physical.top + row * t.height;
                            cells.push(LocatedRect {
                                position: Self::tile_hit(*t, left, top, self.visible.as_deref()),
                                rectangle: CellRect {
                                    left,
                                    top,
                                    right: left + t.width,
                                    bottom: top + t.height,
                                },
                            });
                        }
                        ordinal += 1;
                        if cells.len() == count {
                            break;
                        }
                    }
                    if cells.len() == count {
                        break;
                    }
                }
            }
            if cells.len() == count {
                break;
            }
        }
        Ok(ViewportPage {
            cells,
            total,
            next: (after + count < total).then_some(after + count),
            _charge: charge,
        })
    }
    /// `visible` is a strictly increasing list of included rows, or None for all
    /// rows. Hidden holes remain one tile, not one tile/run per visible cell.
    pub fn build(
        rows: u32,
        columns: u32,
        horizontal: &[Band<'_>],
        vertical: &[Band<'_>],
        visible: Option<&[u32]>,
        memory: &Memory,
        cancel: &CancellationToken,
    ) -> MapResult<Self> {
        Self::build_admitted(
            BuildInput {
                rows,
                columns,
                horizontal,
                vertical,
                visible,
            },
            memory,
            cancel,
            MAX_WORK,
            MAX_PLAN_BYTES,
        )
    }
    pub(crate) fn build_admitted(
        input: BuildInput<'_>,
        memory: &Memory,
        cancel: &CancellationToken,
        work_limit: usize,
        byte_limit: usize,
    ) -> MapResult<Self> {
        let BuildInput {
            rows,
            columns,
            horizontal,
            vertical,
            visible,
        } = input;
        let mut work = Work {
            remaining: work_limit,
            cancel,
        };
        work.tick(1)?;
        if rows == 0 || rows > crate::MAX_ROWS || columns == 0 || columns > crate::MAX_COLUMNS {
            return Err(MapError::Invalid);
        }
        let runs = validate_bands(horizontal, rows, columns, &mut work)?
            + validate_bands(vertical, columns, rows, &mut work)?;
        if runs > MAX_RUNS {
            return Err(MapError::Limit);
        }
        if let Some(v) = visible {
            if v.len() > rows as usize {
                return Err(MapError::Invalid);
            }
            let mut previous = 0;
            for r in v {
                work.tick(1)?;
                if *r <= previous || *r > rows {
                    return Err(MapError::Invalid);
                }
                previous = *r;
            }
        }
        // Count without allocating or enumerating declared cells. Stop as soon as
        // either the topology or metered-work budget is exhausted.
        let mut count = 0usize;
        for (hi, h) in horizontal.iter().enumerate() {
            let last_row = band_end(horizontal, hi, rows);
            for (vi, v) in vertical.iter().enumerate() {
                let last_col = band_end(vertical, vi, columns);
                for width in h.sizes {
                    work.tick(1)?;
                    if width.last < v.start || width.first > last_col {
                        continue;
                    }
                    for height in v.sizes {
                        work.tick(1)?;
                        let first = height.first.max(h.start);
                        let last = height.last.min(last_row);
                        if first > last || rank(visible, first) == rank(visible, last + 1) {
                            continue;
                        }
                        count += 1;
                        if count > MAX_TILES {
                            return Err(MapError::Limit);
                        }
                    }
                }
            }
        }
        let bytes = count
            .checked_mul(std::mem::size_of::<Tile>() + 2 * 32 + 128)
            .and_then(|n| n.checked_add(visible.map_or(0, |v| v.len() * 4)))
            .and_then(|n| n.checked_add(runs * std::mem::size_of::<HeaderRun>()))
            .and_then(|n| {
                n.checked_add((horizontal.len() + vertical.len()) * 4 + std::mem::size_of::<Self>())
            })
            .filter(|n| *n <= byte_limit)
            .ok_or(MapError::Limit)?;
        let charge = memory.reserve(bytes).map_err(|_| MapError::Limit)?;
        let mut x_headers = Vec::with_capacity(horizontal.iter().map(|b| b.sizes.len()).sum());
        let mut y_headers = Vec::with_capacity(vertical.iter().map(|b| b.sizes.len()).sum());
        for (columns, bands) in [(true, horizontal), (false, vertical)] {
            for (band, b) in bands.iter().enumerate() {
                let mut origin = 0i64;
                for size in b.sizes {
                    work.tick(1)?;
                    let visible_start = rank(if columns { None } else { visible }, size.first);
                    let count =
                        rank(if columns { None } else { visible }, size.last + 1) - visible_start;
                    let run = HeaderRun {
                        band,
                        first: size.first,
                        last: size.last,
                        origin,
                        size: size.size,
                        visible_start,
                    };
                    if columns {
                        x_headers.push(run);
                    } else {
                        y_headers.push(run);
                    }
                    origin += count as i64 * size.size;
                }
            }
        }
        let mut tiles = Vec::with_capacity(count);
        for (hi, h) in horizontal.iter().enumerate() {
            let last_row = band_end(horizontal, hi, rows);
            for (vi, v) in vertical.iter().enumerate() {
                let last_col = band_end(vertical, vi, columns);
                for width in h.sizes {
                    work.tick(1)?;
                    let c = width.first.max(v.start);
                    let end_c = width.last.min(last_col);
                    if c > end_c {
                        continue;
                    }
                    let left = prefix(h.sizes, c, None, &mut work)?;
                    for height in v.sizes {
                        work.tick(1)?;
                        let r = height.first.max(h.start);
                        let end_r = height.last.min(last_row);
                        if r > end_r {
                            continue;
                        }
                        let start = rank(visible, r);
                        let visible_count = rank(visible, end_r + 1) - start;
                        if visible_count == 0 {
                            continue;
                        }
                        let top = prefix(v.sizes, r, visible, &mut work)?;
                        let physical = CellRect {
                            left,
                            top,
                            right: left + i64::from(end_c - c + 1) * width.size,
                            bottom: top + visible_count as i64 * height.size,
                        };
                        tiles.push(Tile {
                            logical: Rectangle {
                                first: Position::new(r, c),
                                last: Position::new(end_r, end_c),
                            },
                            physical,
                            width: width.size,
                            height: height.size,
                            visible_start: start,
                        });
                    }
                }
            }
        }
        debug_assert_eq!(count, tiles.len());
        // End events precede starts on shared edges. Active intervals are
        // disjoint until the first collision, so only two neighbors are needed.
        let mut events = Vec::with_capacity(count * 2);
        for (i, t) in tiles.iter().enumerate() {
            events.push((t.physical.left, 1u8, i));
            events.push((t.physical.right, 0u8, i));
        }
        let log = (usize::BITS - (events.len().max(1) - 1).leading_zeros()) as usize;
        work.tick(events.len().saturating_mul(log + 1))?;
        events.sort_unstable();
        let mut active = BTreeMap::<(i64, usize), i64>::new();
        for (_, kind, i) in events {
            work.tick(log + 1)?;
            let t = tiles[i];
            let key = (t.physical.top, i);
            if kind == 0 {
                active.remove(&key);
                continue;
            }
            let predecessor = active
                .range(..key)
                .next_back()
                .filter(|(_, end)| **end > t.physical.top);
            let successor = active
                .range(key..)
                .next()
                .filter(|((start, _), _)| *start < t.physical.bottom);
            if let Some(((_, other), _)) = predecessor.or(successor) {
                let o = tiles[*other];
                let x = t.physical.left.max(o.physical.left);
                let y = t.physical.top.max(o.physical.top);
                return Err(MapError::Overlap {
                    first: Self::tile_hit(t, x, y, visible),
                    second: Self::tile_hit(o, x, y, visible),
                });
            }
            active.insert(key, t.physical.bottom);
        }
        Ok(Self {
            construction_work: work_limit - work.remaining,
            rows,
            columns,
            horizontal: horizontal.iter().map(|b| b.start).collect(),
            vertical: vertical.iter().map(|b| b.start).collect(),
            visible: visible.map(<[u32]>::to_vec),
            tiles,
            x_headers,
            y_headers,
            _charge: charge,
        })
    }
    fn tile_hit(t: Tile, x: i64, y: i64, visible: Option<&[u32]>) -> Position {
        let offset = ((y - t.physical.top) / t.height) as usize;
        let row = visible.map_or(t.logical.first.row + offset as u32, |v| {
            v[t.visible_start + offset]
        });
        Position::new(
            row,
            t.logical.first.column + ((x - t.physical.left) / t.width) as u32,
        )
    }
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
    pub fn hit(&self, x: i64, y: i64) -> Option<Position> {
        self.tiles
            .iter()
            .find(|t| t.physical.contains(x, y))
            .map(|t| Self::tile_hit(*t, x, y, self.visible.as_deref()))
    }
    pub fn cell_rect(&self, p: Position) -> Option<CellRect> {
        if let Some(v) = &self.visible {
            v.binary_search(&p.row).ok()?;
        }
        let t = self.tiles.iter().find(|t| {
            t.logical.first.row <= p.row
                && p.row <= t.logical.last.row
                && t.logical.first.column <= p.column
                && p.column <= t.logical.last.column
        })?;
        let x = t.physical.left + i64::from(p.column - t.logical.first.column) * t.width;
        let y = t.physical.top
            + (rank(self.visible.as_deref(), p.row) - t.visible_start) as i64 * t.height;
        Some(CellRect {
            left: x,
            top: y,
            right: x + t.width,
            bottom: y + t.height,
        })
    }
    pub fn section_closure(&self, requested: Rectangle) -> MapResult<Selection> {
        if requested.first.row == 0
            || requested.first.column == 0
            || requested.first.row > requested.last.row
            || requested.first.column > requested.last.column
            || requested.last.row > self.rows
            || requested.last.column > self.columns
        {
            return Err(MapError::Invalid);
        }
        let rh = self
            .horizontal
            .partition_point(|r| *r <= requested.first.row)
            - 1;
        let rt = self
            .horizontal
            .partition_point(|r| *r <= requested.last.row)
            - 1;
        let ch = self
            .vertical
            .partition_point(|c| *c <= requested.first.column)
            - 1;
        let ct = self
            .vertical
            .partition_point(|c| *c <= requested.last.column)
            - 1;
        let effective = if rh == rt && ch == ct {
            requested
        } else {
            Rectangle {
                first: Position::new(self.horizontal[rh], self.vertical[ch]),
                last: Position::new(
                    self.horizontal.get(rt + 1).map_or(self.rows, |r| r - 1),
                    self.vertical.get(ct + 1).map_or(self.columns, |c| c - 1),
                ),
            }
        };
        Ok(Selection {
            requested,
            effective,
            sections: ((rt - rh + 1) * (ct - ch + 1)) as u32,
            enlarged: effective != requested,
        })
    }
}

#[cfg(test)]
mod tests;
