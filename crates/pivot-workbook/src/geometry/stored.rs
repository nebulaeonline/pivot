//! Immutable scope-owned presentation roots. Authoring commands accept ranks;
//! retained metadata stores stable authored anchors or explicitly tagged ordinals.
use super::*;
use crate::{
    AxisId, Error, Expected, GridScopeId, Result, ScopeCapture, Workbook, axes::Axis, persistent,
};
use pivot_core::model::integration::resources::{Footprint, Shared};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Authored(AxisId),
    Ordinal(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SizeImage {
    pub first: Anchor,
    pub last: Anchor,
    pub size: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileImage {
    pub id: u64,
    pub default: i64,
    pub sizes: Vec<SizeImage>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionImage {
    pub id: u64,
    pub start: Option<Anchor>,
    pub profile: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryImage {
    pub scope: GridScopeId,
    pub next_id: u64,
    pub reviewed_layout: Option<[u8; 32]>,
    pub horizontal: Vec<SectionImage>,
    pub vertical: Vec<SectionImage>,
    pub profiles: Vec<ProfileImage>,
}
impl Footprint for ProfileImage {
    fn heap_bytes(&self) -> usize {
        self.sizes.capacity() * std::mem::size_of::<SizeImage>()
    }
}
#[derive(Clone)]
struct Section {
    id: u64,
    start: Option<Anchor>,
    profile: Shared<ProfileImage>,
}
#[derive(Clone)]
pub(crate) struct Root {
    next_id: u64,
    reviewed_layout: Option<[u8; 32]>,
    horizontal: Vec<Section>,
    vertical: Vec<Section>,
}
impl Footprint for Root {
    fn heap_bytes(&self) -> usize {
        (self.horizontal.capacity() + self.vertical.capacity()) * std::mem::size_of::<Section>()
    }
}
#[derive(Clone, Debug)]
pub struct ProfilePreview {
    pub horizontal: bool,
    pub at: u32,
    pub sizes: Vec<SizeRun>,
}
#[derive(Clone, Debug)]
pub enum Command {
    AddBreak {
        horizontal: bool,
        before: u32,
    },
    RemoveBreak {
        horizontal: bool,
        before: u32,
    },
    /// A batch can update several perpendicular profiles atomically. Replacing
    /// complete resolved runs makes independent numeric previews deterministic.
    SetProfile {
        horizontal: bool,
        at: u32,
        sizes: Vec<SizeRun>,
    },
    /// Numeric sizing or a balanced adjacent divider. Shared edits explicitly
    /// apply the same change to every perpendicular profile in one publication.
    SetSizes {
        horizontal: bool,
        at: u32,
        first: u32,
        last: u32,
        size: i64,
        paired: bool,
        shared: bool,
    },
    AcknowledgeLayout {
        stamp: [u8; 32],
    },
    ResetSizes,
    ResetAll,
}
fn failure(e: MapError) -> Error {
    Error::Graph(format!("section geometry: {e:?}"))
}
fn checked(cancel: &CancellationToken) -> Result<()> {
    if cancel.is_cancelled() {
        Err(Error::Graph("cancelled geometry".into()))
    } else {
        Ok(())
    }
}
struct Axes {
    rows: Axis,
    columns: Axis,
    positional: bool,
}
impl Axes {
    fn anchor(&self, row: bool, rank: u32) -> Result<Anchor> {
        let axis = if row { &self.rows } else { &self.columns };
        if rank == 0 || rank > axis.length {
            return Err(Error::Position);
        }
        if self.positional {
            Ok(Anchor::Ordinal(rank))
        } else {
            Ok(Anchor::Authored(axis.at(rank)?))
        }
    }
    fn rank(&self, row: bool, anchor: Anchor) -> Result<u32> {
        let axis = if row { &self.rows } else { &self.columns };
        match (self.positional, anchor) {
            (true, Anchor::Ordinal(n)) if n > 0 && n <= axis.length => Ok(n),
            (false, Anchor::Authored(id)) => axis.rank(id),
            _ => Err(Error::InvalidTarget),
        }
    }
}
type ResolvedProfiles = (Vec<Vec<SizeRun>>, Vec<Vec<SizeRun>>);
impl Root {
    fn same(&self, other: &Self) -> bool {
        let same = |a: &[Section], b: &[Section]| {
            a.len() == b.len()
                && a.iter().zip(b).all(|(x, y)| {
                    x.id == y.id && x.start == y.start && x.profile.id == y.profile.id
                })
        };
        self.reviewed_layout == other.reviewed_layout
            && self.next_id == other.next_id
            && same(&self.horizontal, &other.horizontal)
            && same(&self.vertical, &other.vertical)
    }
    fn image(&self, scope: GridScopeId) -> GeometryImage {
        let mut profiles = Vec::new();
        for s in self.horizontal.iter().chain(&self.vertical) {
            if !profiles.iter().any(|p: &ProfileImage| p.id == s.profile.id) {
                profiles.push((*s.profile).clone());
            }
        }
        let sections = |v: &[Section]| {
            v.iter()
                .map(|s| SectionImage {
                    id: s.id,
                    start: s.start,
                    profile: s.profile.id,
                })
                .collect()
        };
        GeometryImage {
            scope,
            next_id: self.next_id,
            reviewed_layout: self.reviewed_layout,
            horizontal: sections(&self.horizontal),
            vertical: sections(&self.vertical),
            profiles,
        }
    }
    fn initial(memory: &Memory) -> Result<Self> {
        let width = memory
            .alloc(ProfileImage {
                id: 2,
                default: DEFAULT_WIDTH,
                sizes: Vec::new(),
            })
            .map_err(|_| Error::Limit)?;
        let height = memory
            .alloc(ProfileImage {
                id: 4,
                default: DEFAULT_HEIGHT,
                sizes: Vec::new(),
            })
            .map_err(|_| Error::Limit)?;
        Ok(Self {
            next_id: 5,
            reviewed_layout: None,
            horizontal: vec![Section {
                id: 1,
                start: None,
                profile: width,
            }],
            vertical: vec![Section {
                id: 3,
                start: None,
                profile: height,
            }],
        })
    }
    fn fresh(&mut self) -> Result<u64> {
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or(Error::Exhausted)?;
        Ok(id)
    }
    pub(crate) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.heap_bytes()
            + self
                .horizontal
                .iter()
                .chain(&self.vertical)
                .enumerate()
                .filter(|(i, s)| {
                    !self
                        .horizontal
                        .iter()
                        .chain(&self.vertical)
                        .take(*i)
                        .any(|p| p.profile.id == s.profile.id)
                })
                .map(|(_, s)| std::mem::size_of::<ProfileImage>() + s.profile.heap_bytes() + 64)
                .sum::<usize>()
    }
    fn resolved(&self, axes: &Axes, cancel: &CancellationToken) -> Result<ResolvedProfiles> {
        let resolve = |sections: &[Section], horizontal: bool| -> Result<Vec<Vec<SizeRun>>> {
            let length = if horizontal {
                axes.columns.length
            } else {
                axes.rows.length
            };
            sections
                .iter()
                .map(|s| {
                    checked(cancel)?;
                    let p = &s.profile;
                    let mut runs = Vec::with_capacity(p.sizes.len() * 2 + 1);
                    let mut next = 1;
                    for size in &p.sizes {
                        checked(cancel)?;
                        let first = axes.rank(!horizontal, size.first)?;
                        let last = axes.rank(!horizontal, size.last)?;
                        if first < next
                            || last < first
                            || !(MIN_SIZE..=MAX_SIZE).contains(&size.size)
                        {
                            return Err(Error::InvalidTarget);
                        }
                        if next < first {
                            runs.push(SizeRun {
                                first: next,
                                last: first - 1,
                                size: p.default,
                            });
                        }
                        runs.push(SizeRun {
                            first,
                            last,
                            size: size.size,
                        });
                        next = last + 1;
                    }
                    if next <= length {
                        runs.push(SizeRun {
                            first: next,
                            last: length,
                            size: p.default,
                        });
                    }
                    Ok(runs)
                })
                .collect()
        };
        Ok((
            resolve(&self.horizontal, true)?,
            resolve(&self.vertical, false)?,
        ))
    }
    fn map(
        &self,
        axes: &Axes,
        visible: Option<&[u32]>,
        memory: &Memory,
        cancel: &CancellationToken,
    ) -> Result<Map> {
        let mut remaining = MAX_WORK;
        self.map_budget(axes, visible, memory, cancel, &mut remaining)
    }
    fn map_budget(
        &self,
        axes: &Axes,
        visible: Option<&[u32]>,
        memory: &Memory,
        cancel: &CancellationToken,
        remaining: &mut usize,
    ) -> Result<Map> {
        let scratch = (self.horizontal.len() + self.vertical.len()) * 128
            + self
                .horizontal
                .iter()
                .chain(&self.vertical)
                .map(|s| s.profile.sizes.len() * 2 + 1)
                .sum::<usize>()
                * std::mem::size_of::<SizeRun>();
        let anchors = self
            .horizontal
            .iter()
            .chain(&self.vertical)
            .map(|s| s.profile.sizes.len() * 2 + 1)
            .sum::<usize>();
        let axis_visits = axes.rows.layout.as_ref().map_or(1, |p| p.runs.len())
            + axes.columns.layout.as_ref().map_or(1, |p| p.runs.len());
        let work = anchors
            .checked_mul(axis_visits)
            .and_then(|n| n.checked_add(anchors))
            .filter(|n| *n < MAX_WORK)
            .ok_or(Error::Limit)?;
        if scratch >= MAX_PLAN_BYTES {
            return Err(Error::Limit);
        }
        let _charge = memory.reserve(scratch).map_err(|_| Error::Limit)?;
        let (widths, heights) = self.resolved(axes, cancel)?;
        let bands = |sections: &[Section], row: bool| -> Result<Vec<u32>> {
            sections
                .iter()
                .enumerate()
                .map(|(i, s)| match s.start {
                    None if i == 0 => Ok(1),
                    Some(a) if i > 0 => axes.rank(row, a),
                    _ => Err(Error::InvalidTarget),
                })
                .collect()
        };
        let hs = bands(&self.horizontal, true)?;
        let vs = bands(&self.vertical, false)?;
        let h: Vec<_> = hs
            .iter()
            .zip(&widths)
            .map(|(r, s)| Band {
                start: *r,
                sizes: s,
            })
            .collect();
        let v: Vec<_> = vs
            .iter()
            .zip(&heights)
            .map(|(r, s)| Band {
                start: *r,
                sizes: s,
            })
            .collect();
        *remaining = remaining.checked_sub(work).ok_or(Error::Limit)?;
        let mut map = Map::build_admitted(
            BuildInput {
                rows: axes.rows.length,
                columns: axes.columns.length,
                horizontal: &h,
                vertical: &v,
                visible,
            },
            memory,
            cancel,
            *remaining,
            MAX_PLAN_BYTES - scratch,
        )
        .map_err(failure)?;
        *remaining = remaining
            .checked_sub(map.construction_work)
            .ok_or(Error::Limit)?;
        map.construction_work += work;
        Ok(map)
    }
    fn apply(
        &mut self,
        cmd: &Command,
        axes: &Axes,
        memory: &Memory,
        cancel: &CancellationToken,
        remaining: &mut usize,
    ) -> Result<()> {
        checked(cancel)?;
        let axis_visits = axes.rows.layout.as_ref().map_or(1, |p| p.runs.len())
            + axes.columns.layout.as_ref().map_or(1, |p| p.runs.len());
        let bands = self.horizontal.len() + self.vertical.len();
        let units = match cmd {
            Command::SetProfile { sizes, .. } => {
                (sizes.len() * 4 + bands) * axis_visits + sizes.len()
            }
            Command::SetSizes { .. } => {
                self.horizontal
                    .iter()
                    .chain(&self.vertical)
                    .map(|s| s.profile.sizes.len() * 4 + 1)
                    .sum::<usize>()
                    * axis_visits
                    + bands * axis_visits
            }
            _ => bands * axis_visits + 1,
        };
        *remaining = remaining.checked_sub(units).ok_or(Error::Limit)?;
        match cmd {
            Command::AddBreak { horizontal, before }
            | Command::RemoveBreak { horizontal, before } => {
                if *before <= 1 {
                    return Err(Error::Position);
                }
                let anchor = axes.anchor(*horizontal, *before)?;
                let sections = if *horizontal {
                    &self.horizontal
                } else {
                    &self.vertical
                };
                let found = sections.iter().position(|s| s.start == Some(anchor));
                if matches!(cmd, Command::RemoveBreak { .. }) {
                    let i = found.ok_or(Error::InvalidTarget)?;
                    (if *horizontal {
                        &mut self.horizontal
                    } else {
                        &mut self.vertical
                    })
                    .remove(i);
                } else if found.is_none() {
                    if sections.len() >= MAX_BANDS {
                        return Err(Error::Limit);
                    }
                    let mut index = 1;
                    while index < sections.len()
                        && axes.rank(*horizontal, sections[index].start.unwrap())? < *before
                    {
                        index += 1;
                    }
                    let profile = sections[index - 1].profile.clone();
                    let id = self.fresh()?;
                    (if *horizontal {
                        &mut self.horizontal
                    } else {
                        &mut self.vertical
                    })
                    .insert(
                        index,
                        Section {
                            id,
                            start: Some(anchor),
                            profile,
                        },
                    );
                }
            }
            Command::SetSizes {
                horizontal,
                at,
                first,
                last,
                size,
                paired,
                shared,
            } => {
                let length = if *horizontal {
                    axes.columns.length
                } else {
                    axes.rows.length
                };
                axes.anchor(*horizontal, *at)?;
                if *first == 0
                    || first > last
                    || *last > length
                    || !(MIN_SIZE..=MAX_SIZE).contains(size)
                    || (*paired && (first != last || *last == length))
                {
                    return Err(Error::Position);
                }
                let stored_runs: usize = self
                    .horizontal
                    .iter()
                    .chain(&self.vertical)
                    .map(|s| s.profile.sizes.len() * 2 + 1)
                    .sum();
                let _scratch = memory
                    .reserve(stored_runs * std::mem::size_of::<SizeRun>() * 3 + 32768)
                    .map_err(|_| Error::Limit)?;
                let (widths, heights) = self.resolved(axes, cancel)?;
                let profiles = if *horizontal { widths } else { heights };
                let sections = if *horizontal {
                    &self.horizontal
                } else {
                    &self.vertical
                };
                let mut selected = 0;
                let mut starts = Vec::with_capacity(sections.len());
                for (index, section) in sections.iter().enumerate() {
                    let start = match section.start {
                        None => 1,
                        Some(a) => axes.rank(*horizontal, a)?,
                    };
                    if start <= *at {
                        selected = index;
                    }
                    starts.push(start);
                }
                let selected_old = profiles[selected]
                    .iter()
                    .find(|r| r.first <= *first && r.last >= *first)
                    .ok_or(Error::Position)?
                    .size;
                let delta = size - selected_old;
                for (index, runs) in profiles.iter().enumerate() {
                    checked(cancel)?;
                    if !shared && index != selected {
                        continue;
                    }
                    let old_first = runs
                        .iter()
                        .find(|r| r.first <= *first && r.last >= *first)
                        .ok_or(Error::Position)?
                        .size;
                    let desired = if *shared && *paired {
                        old_first + delta
                    } else {
                        *size
                    };
                    if !(MIN_SIZE..=MAX_SIZE).contains(&desired) {
                        return Err(Error::Position);
                    }
                    let second = if *paired {
                        let old = runs
                            .iter()
                            .find(|r| r.first <= *first && r.last >= *first)
                            .ok_or(Error::Position)?
                            .size;
                        let next = runs
                            .iter()
                            .find(|r| r.first <= *first + 1 && r.last > *first)
                            .ok_or(Error::Position)?
                            .size;
                        let value = old + next - desired;
                        if !(MIN_SIZE..=MAX_SIZE).contains(&value) {
                            return Err(Error::Position);
                        }
                        Some(value)
                    } else {
                        None
                    };
                    let mut updated: Vec<SizeRun> = Vec::with_capacity(runs.len() + 4);
                    for run in runs {
                        let mut cuts = vec![run.first, run.last + 1];
                        for cut in [
                            *first,
                            *last + 1,
                            if *paired { *last + 2 } else { *last + 1 },
                        ] {
                            if cut > run.first && cut <= run.last {
                                cuts.push(cut);
                            }
                        }
                        cuts.sort_unstable();
                        cuts.dedup();
                        for pair in cuts.windows(2) {
                            let value = if pair[0] >= *first && pair[0] <= *last {
                                desired
                            } else if *paired && pair[0] == *last + 1 {
                                second.unwrap()
                            } else {
                                run.size
                            };
                            if let Some(previous) = updated.last_mut()
                                && previous.last + 1 == pair[0]
                                && previous.size == value
                            {
                                previous.last = pair[1] - 1;
                            } else {
                                updated.push(SizeRun {
                                    first: pair[0],
                                    last: pair[1] - 1,
                                    size: value,
                                });
                            }
                        }
                    }
                    self.apply(
                        &Command::SetProfile {
                            horizontal: *horizontal,
                            at: starts[index],
                            sizes: updated,
                        },
                        axes,
                        memory,
                        cancel,
                        remaining,
                    )?;
                }
            }
            Command::SetProfile {
                horizontal,
                at,
                sizes,
            } => {
                let length = if *horizontal {
                    axes.columns.length
                } else {
                    axes.rows.length
                };
                if sizes.len() > MAX_RUNS {
                    return Err(Error::Limit);
                }
                let band = [Band { start: 1, sizes }];
                validate_bands(
                    &band,
                    1,
                    length,
                    &mut Work {
                        remaining: MAX_WORK,
                        cancel,
                    },
                )
                .map_err(failure)?;
                axes.anchor(*horizontal, *at)?;
                let sections = if *horizontal {
                    &self.horizontal
                } else {
                    &self.vertical
                };
                let mut index = 0;
                for (i, s) in sections.iter().enumerate().skip(1) {
                    if axes.rank(*horizontal, s.start.unwrap())? <= *at {
                        index = i;
                    } else {
                        break;
                    }
                }
                let default = sections[index].profile.default;
                let id = self.fresh()?;
                let mut entries = Vec::with_capacity(sizes.len());
                for size in sizes {
                    if size.size != default {
                        entries.push(SizeImage {
                            first: axes.anchor(!horizontal, size.first)?,
                            last: axes.anchor(!horizontal, size.last)?,
                            size: size.size,
                        });
                    }
                }
                let profile = memory
                    .alloc(ProfileImage {
                        id,
                        default,
                        sizes: entries,
                    })
                    .map_err(|_| Error::Limit)?;
                (if *horizontal {
                    &mut self.horizontal
                } else {
                    &mut self.vertical
                })[index]
                    .profile = profile;
            }
            Command::AcknowledgeLayout { stamp } => {
                self.reviewed_layout = Some(*stamp);
            }
            Command::ResetSizes | Command::ResetAll => {
                for horizontal in [true, false] {
                    let id = self.fresh()?;
                    let default = if horizontal {
                        DEFAULT_WIDTH
                    } else {
                        DEFAULT_HEIGHT
                    };
                    let profile = memory
                        .alloc(ProfileImage {
                            id,
                            default,
                            sizes: Vec::new(),
                        })
                        .map_err(|_| Error::Limit)?;
                    let sections = if horizontal {
                        &mut self.horizontal
                    } else {
                        &mut self.vertical
                    };
                    if matches!(cmd, Command::ResetAll) {
                        sections.truncate(1);
                    }
                    for s in sections {
                        s.profile = profile.clone();
                    }
                }
            }
        }
        Ok(())
    }
}
fn axes_for(data: &persistent::Data, scope: GridScopeId) -> Result<Axes> {
    let info = data
        .sheets
        .as_ref()
        .and_then(|d| d.0.iter().find(|s| s.id == scope))
        .ok_or(Error::InvalidTarget)?;
    let positional = data
        .producers
        .as_ref()
        .is_some_and(|d| d.binding(scope).is_some());
    // Presentation ordinals are validated independently of current source extent.
    Ok(Axes {
        rows: if positional {
            Axis {
                namespace: info.rows.namespace,
                length: crate::MAX_ROWS,
                layout: None,
            }
        } else {
            info.rows.clone()
        },
        columns: if positional {
            Axis {
                namespace: info.columns.namespace,
                length: crate::MAX_COLUMNS,
                layout: None,
            }
        } else {
            info.columns.clone()
        },
        positional,
    })
}
impl Workbook {
    #[doc(hidden)]
    pub fn has_geometry_state(&self) -> bool {
        !self.persistent.geometry.is_empty()
    }
    pub(crate) fn retire_geometry(&mut self, scope: GridScopeId) -> Result<()> {
        let Some(root) = self.persistent.geometry.get(&scope) else {
            return Ok(());
        };
        let floor = root.next_id;
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(crate::graph_error)?;
        let retired = Root {
            next_id: floor,
            reviewed_layout: None,
            horizontal: Vec::new(),
            vertical: Vec::new(),
        };
        let shared = data.memory.alloc(retired).map_err(|_| Error::Limit)?;
        data.geometry
            .insert_private(scope, shared)
            .map_err(|_| Error::Limit)?;
        self.persistent = data.freeze();
        Ok(())
    }
    pub fn geometry_images(&self) -> impl Iterator<Item = GeometryImage> + '_ {
        self.persistent
            .geometry
            .iter()
            .map(|(scope, r)| r.image(*scope))
    }
    /// Atomic multi-profile edit: no invalid intermediate map is ever published.
    pub fn edit_geometry(
        &mut self,
        scope: GridScopeId,
        expected: Expected,
        commands: &[Command],
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.geometry_candidate(scope, expected, commands, cancel, false, None)
    }
    /// Validate the identical atomic edit path without publishing or retaining a candidate.
    pub fn preview_geometry(
        &mut self,
        scope: GridScopeId,
        expected: Expected,
        commands: &[Command],
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.geometry_candidate(scope, expected, commands, cancel, true, None)
    }
    pub fn preview_geometry_profiles(
        &mut self,
        scope: GridScopeId,
        expected: Expected,
        commands: &[Command],
        cancel: &CancellationToken,
    ) -> Result<Vec<ProfilePreview>> {
        let mut profiles = Vec::new();
        self.geometry_candidate(scope, expected, commands, cancel, true, Some(&mut profiles))?;
        Ok(profiles)
    }
    fn geometry_candidate(
        &mut self,
        scope: GridScopeId,
        expected: Expected,
        commands: &[Command],
        cancel: &CancellationToken,
        preview: bool,
        output: Option<&mut Vec<ProfilePreview>>,
    ) -> Result<Expected> {
        self.check(expected)?;
        checked(cancel)?;
        if self.is_structured_proof() || commands.is_empty() || commands.len() > 512 {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        self.sheet_info(scope)?;
        let axes = axes_for(&self.persistent, scope)?;
        let mut input_runs = 0usize;
        for command in commands {
            checked(cancel)?;
            if let Command::SetProfile { sizes, .. } = command {
                input_runs = input_runs
                    .checked_add(sizes.len())
                    .filter(|n| *n <= MAX_RUNS)
                    .ok_or(Error::Limit)?;
            }
        }
        if !self.persistent.geometry.contains_key(&scope)
            && commands
                .iter()
                .all(|c| matches!(c, Command::ResetAll | Command::ResetSizes))
        {
            return Ok(expected);
        }
        let _scratch = self
            .persistent
            .memory
            .reserve(
                32768
                    + commands
                        .iter()
                        .map(|c| {
                            if let Command::SetProfile { sizes, .. } = c {
                                sizes.len() * std::mem::size_of::<SizeImage>()
                            } else {
                                0
                            }
                        })
                        .sum::<usize>(),
            )
            .map_err(|_| Error::Limit)?;
        let mut root = self
            .persistent
            .geometry
            .get(&scope)
            .map(|r| (**r).clone())
            .map_or_else(|| Root::initial(&self.persistent.memory), Ok)?;
        let mut remaining = MAX_WORK
            .checked_sub(crate::producers::layout_work(&self.persistent, scope)?)
            .ok_or(Error::Limit)?;
        let current_layout = crate::producers::layout_stamp(&self.persistent, scope, cancel)?;
        if !self.persistent.geometry.contains_key(&scope) {
            root.reviewed_layout = current_layout;
        }
        for command in commands {
            if let Command::AcknowledgeLayout { stamp } = command
                && current_layout != Some(*stamp)
            {
                return Err(Error::InvalidTarget);
            }
            root.apply(
                command,
                &axes,
                &self.persistent.memory,
                cancel,
                &mut remaining,
            )?;
            if matches!(command, Command::ResetAll) {
                root.reviewed_layout = current_layout;
            }
        }
        if self
            .persistent
            .geometry
            .get(&scope)
            .is_some_and(|old| old.same(&root))
        {
            return Ok(expected);
        }
        root.map_budget(&axes, None, &self.persistent.memory, cancel, &mut remaining)?;
        if self.filter(scope)?.is_some_and(|d| !d.needs_review) {
            let visibility = self.filter_visibility(scope, self.expected(), cancel)?;
            root.map_budget(
                &axes,
                Some(visibility.rows()),
                &self.persistent.memory,
                cancel,
                &mut remaining,
            )?;
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(crate::graph_error)?;
        if let Some(output) = output {
            let runs = root
                .horizontal
                .iter()
                .chain(&root.vertical)
                .map(|s| s.profile.sizes.len() * 2 + 1)
                .sum::<usize>();
            if runs > 4096 {
                return Err(Error::Limit);
            }
            let axis_visits = axes.rows.layout.as_ref().map_or(1, |p| p.runs.len())
                + axes.columns.layout.as_ref().map_or(1, |p| p.runs.len());
            if runs
                .checked_mul(axis_visits + 8)
                .is_none_or(|work| work > remaining)
            {
                return Err(Error::Limit);
            }
            let (widths, heights) = root.resolved(&axes, cancel)?;
            for (horizontal, sections, profiles) in [
                (true, &root.horizontal, widths),
                (false, &root.vertical, heights),
            ] {
                for (section, sizes) in sections.iter().zip(profiles) {
                    let at = section.start.map_or(Ok(1), |a| axes.rank(horizontal, a))?;
                    output.push(ProfilePreview {
                        horizontal,
                        at,
                        sizes,
                    });
                }
            }
        }
        let shared = data.memory.alloc(root).map_err(|_| Error::Limit)?;
        data.geometry
            .insert_private(scope, shared)
            .map_err(|_| Error::Limit)?;
        data.check_geometry_limits(self.limits)?;
        checked(cancel)?;
        if preview {
            return Ok(expected);
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.persistent = data.freeze();
        self.state = next;
        Ok(self.expected())
    }
}
impl persistent::Data {
    pub(crate) fn detach_geometry(
        &mut self,
        source: GridScopeId,
        destination: GridScopeId,
        cancel: &CancellationToken,
    ) -> Result<()> {
        let Some(original) = self.geometry.get(&source).cloned() else {
            return Ok(());
        };
        let axes = axes_for(self, destination)?;
        if axes.positional {
            return Err(Error::InvalidTarget);
        }
        let _scratch = self
            .memory
            .reserve(original.owned_bytes() * 3 + 32768)
            .map_err(|_| Error::Limit)?;
        let mut root = Root {
            next_id: 1,
            reviewed_layout: None,
            horizontal: Vec::new(),
            vertical: Vec::new(),
        };
        let mut remapped = std::collections::BTreeMap::new();
        for horizontal in [true, false] {
            let sections = if horizontal {
                &original.horizontal
            } else {
                &original.vertical
            };
            let length = if horizontal {
                axes.rows.length
            } else {
                axes.columns.length
            };
            let opposite = if horizontal {
                axes.columns.length
            } else {
                axes.rows.length
            };
            for s in sections {
                checked(cancel)?;
                let start = match s.start {
                    None => None,
                    Some(Anchor::Ordinal(n)) if n <= length => Some(axes.anchor(horizontal, n)?),
                    Some(Anchor::Ordinal(_)) => break,
                    _ => return Err(Error::InvalidTarget),
                };
                let profile = if let Some(p) = remapped.get(&s.profile.id) {
                    Shared::clone(p)
                } else {
                    let mut entries = Vec::new();
                    for entry in &s.profile.sizes {
                        let (Anchor::Ordinal(first), Anchor::Ordinal(last)) =
                            (entry.first, entry.last)
                        else {
                            return Err(Error::InvalidTarget);
                        };
                        if first <= opposite {
                            entries.push(SizeImage {
                                first: axes.anchor(!horizontal, first)?,
                                last: axes.anchor(!horizontal, last.min(opposite))?,
                                size: entry.size,
                            });
                        }
                    }
                    let id = root.fresh()?;
                    let p = self
                        .memory
                        .alloc(ProfileImage {
                            id,
                            default: s.profile.default,
                            sizes: entries,
                        })
                        .map_err(|_| Error::Limit)?;
                    remapped.insert(s.profile.id, p.clone());
                    p
                };
                let id = root.fresh()?;
                (if horizontal {
                    &mut root.horizontal
                } else {
                    &mut root.vertical
                })
                .push(Section { id, start, profile });
            }
        }
        let _map = root.map(&axes, None, &self.memory, cancel)?;
        self.geometry
            .insert_private(
                destination,
                self.memory.alloc(root).map_err(|_| Error::Limit)?,
            )
            .map_err(|_| Error::Limit)?;
        Ok(())
    }
    pub(crate) fn restore_geometry(
        &mut self,
        images: &[GeometryImage],
        limits: crate::Limits,
        cancel: &CancellationToken,
    ) -> Result<()> {
        use std::collections::{BTreeMap, BTreeSet};
        if images.len() > 262144 {
            return Err(Error::Limit);
        }
        let mut scopes = BTreeSet::new();
        for image in images {
            checked(cancel)?;
            let retired = self
                .sheets
                .as_ref()
                .and_then(|d| d.0.iter().find(|s| s.id == image.scope))
                .ok_or(Error::InvalidTarget)?
                .retired;
            if !scopes.insert(image.scope)
                || (!retired && (image.horizontal.is_empty() || image.vertical.is_empty()))
                || (retired
                    && (!image.horizontal.is_empty()
                        || !image.vertical.is_empty()
                        || !image.profiles.is_empty()))
                || image.horizontal.len() > MAX_BANDS
                || image.vertical.len() > MAX_BANDS
                || (!retired && image.profiles.is_empty())
                || image.profiles.len() > 512
                || image.next_id > limits.allocated_cells as u64 + 1
                || image.next_id < 5
                || image.profiles.iter().map(|p| p.sizes.len()).sum::<usize>() > MAX_RUNS
            {
                return Err(Error::InvalidTarget);
            }
            let _scratch = self
                .memory
                .reserve(
                    image
                        .profiles
                        .iter()
                        .map(|p| p.sizes.len() * std::mem::size_of::<SizeImage>() + 256)
                        .sum::<usize>()
                        + 32768,
                )
                .map_err(|_| Error::Limit)?;
            let mut ids = BTreeSet::new();
            let mut profiles = BTreeMap::new();
            for profile in &image.profiles {
                if profile.id == 0
                    || profile.id >= image.next_id
                    || !ids.insert(profile.id)
                    || !(MIN_SIZE..=MAX_SIZE).contains(&profile.default)
                {
                    return Err(Error::InvalidTarget);
                }
                profiles.insert(
                    profile.id,
                    self.memory
                        .alloc(profile.clone())
                        .map_err(|_| Error::Limit)?,
                );
            }
            let mut used = BTreeSet::new();
            let mut decode = |sections: &[SectionImage]| -> Result<Vec<Section>> {
                sections
                    .iter()
                    .map(|s| {
                        if s.id == 0 || s.id >= image.next_id || !ids.insert(s.id) {
                            return Err(Error::InvalidTarget);
                        }
                        used.insert(s.profile);
                        Ok(Section {
                            id: s.id,
                            start: s.start,
                            profile: profiles
                                .get(&s.profile)
                                .ok_or(Error::InvalidTarget)?
                                .clone(),
                        })
                    })
                    .collect()
            };
            if image.reviewed_layout.is_some() && !axes_for(self, image.scope)?.positional {
                return Err(Error::InvalidTarget);
            }
            let root = Root {
                next_id: image.next_id,
                reviewed_layout: image.reviewed_layout,
                horizontal: decode(&image.horizontal)?,
                vertical: decode(&image.vertical)?,
            };
            if used.len() != profiles.len() {
                return Err(Error::InvalidTarget);
            }
            if !retired {
                let axes = axes_for(self, image.scope)?;
                let _map = root.map(&axes, None, &self.memory, cancel)?;
            }
            self.geometry
                .insert_private(
                    image.scope,
                    self.memory.alloc(root).map_err(|_| Error::Limit)?,
                )
                .map_err(|_| Error::Limit)?;
            self.check_geometry_limits(limits)?;
        }
        Ok(())
    }
    pub(crate) fn transform_geometry(
        &mut self,
        old: &persistent::Data,
        scope: GridScopeId,
        change: crate::structure::Change,
        cancel: &CancellationToken,
    ) -> Result<()> {
        use crate::structure::{Action, AxisKind};
        let Some(stored) = old.geometry.get(&scope) else {
            return Ok(());
        };
        let old_axes = axes_for(old, scope)?;
        let new_axes = axes_for(self, scope)?;
        if old_axes.positional {
            return Err(Error::InvalidTarget);
        }
        let _scratch = self
            .memory
            .reserve(stored.owned_bytes() * 4 + 32768)
            .map_err(|_| Error::Limit)?;
        let mut root = (**stored).clone();
        let row = change.axis == AxisKind::Rows;
        let length = if row {
            old_axes.rows.length
        } else {
            old_axes.columns.length
        };
        let sections = if row {
            &mut root.horizontal
        } else {
            &mut root.vertical
        };
        let mut transformed: Vec<Section> = Vec::with_capacity(sections.len());
        for s in sections.iter() {
            checked(cancel)?;
            let rank = if let Some(a) = s.start {
                old_axes.rank(row, a)?
            } else {
                1
            };
            let new_rank = if s.start.is_none() {
                Some(1)
            } else {
                match change.action {
                    Action::Insert => Some(if rank >= change.start {
                        rank + change.count
                    } else {
                        rank
                    })
                    .filter(|r| *r <= length),
                    Action::Delete => {
                        let end = change.start + change.count;
                        if rank < change.start {
                            Some(rank)
                        } else if rank >= end {
                            Some(rank - change.count)
                        } else if end <= length {
                            Some(change.start)
                        } else {
                            None
                        }
                    }
                }
            };
            if let Some(n) = new_rank {
                let mut s = s.clone();
                s.start = if n == 1 {
                    None
                } else {
                    Some(new_axes.anchor(row, n)?)
                };
                if transformed.last().is_some_and(|p| p.start == s.start) {
                    transformed.pop();
                }
                transformed.push(s);
            }
        }
        *sections = transformed;
        let (widths, heights) = stored.resolved(&old_axes, cancel)?;
        let old_profiles = if row {
            &stored.vertical
        } else {
            &stored.horizontal
        };
        let resolved = if row { heights } else { widths };
        let mut replacements: Vec<(u64, Shared<ProfileImage>)> = Vec::new();
        for (section, runs) in old_profiles.iter().zip(resolved) {
            if replacements.iter().any(|(id, _)| *id == section.profile.id) {
                continue;
            }
            let mut pieces = Vec::with_capacity(runs.len() + 3);
            for run in &runs {
                checked(cancel)?;
                let before_end = run.last.min(change.start - 1);
                if run.first <= before_end {
                    pieces.push(SizeRun {
                        first: run.first,
                        last: before_end,
                        size: run.size,
                    });
                }
                let after_start = run.first.max(
                    change.start
                        + if change.action == Action::Delete {
                            change.count
                        } else {
                            0
                        },
                );
                if after_start <= run.last {
                    let (first, last) = if change.action == Action::Insert {
                        (
                            after_start + change.count,
                            run.last.saturating_add(change.count).min(length),
                        )
                    } else {
                        (after_start - change.count, run.last - change.count)
                    };
                    if first <= last {
                        pieces.push(SizeRun {
                            first,
                            last,
                            size: run.size,
                        });
                    }
                }
            }
            if change.action == Action::Insert {
                let inherited = change.start.saturating_sub(1).max(1);
                let size = runs
                    .iter()
                    .find(|r| r.first <= inherited && inherited <= r.last)
                    .ok_or(Error::InvalidTarget)?
                    .size;
                pieces.push(SizeRun {
                    first: change.start,
                    last: change.start + change.count - 1,
                    size,
                });
            } else {
                pieces.push(SizeRun {
                    first: length - change.count + 1,
                    last: length,
                    size: section.profile.default,
                });
            }
            pieces.sort_unstable_by_key(|r| r.first);
            let mut entries = Vec::with_capacity(pieces.len());
            for run in pieces {
                if run.size != section.profile.default {
                    entries.push(SizeImage {
                        first: new_axes.anchor(row, run.first)?,
                        last: new_axes.anchor(row, run.last)?,
                        size: run.size,
                    });
                }
            }
            let id = root.fresh()?;
            let profile = self
                .memory
                .alloc(ProfileImage {
                    id,
                    default: section.profile.default,
                    sizes: entries,
                })
                .map_err(|_| Error::Limit)?;
            replacements.push((section.profile.id, profile));
        }
        for s in if row {
            &mut root.vertical
        } else {
            &mut root.horizontal
        } {
            s.profile = replacements
                .iter()
                .find(|(id, _)| *id == s.profile.id)
                .ok_or(Error::InvalidTarget)?
                .1
                .clone();
        }
        let _base = root.map(&new_axes, None, &self.memory, cancel)?;
        let shared = self.memory.alloc(root).map_err(|_| Error::Limit)?;
        self.geometry
            .insert_private(scope, shared)
            .map_err(|_| Error::Limit)?;
        Ok(())
    }
    pub(crate) fn check_geometry_limits(&self, limits: crate::Limits) -> Result<()> {
        let mut breaks = 0;
        let mut profiles = 0;
        let mut runs = 0;
        let mut bytes = 0;
        for (_, root) in self.geometry.iter() {
            breaks += (root.horizontal.len() + root.vertical.len()).saturating_sub(2);
            for (i, s) in root.horizontal.iter().chain(&root.vertical).enumerate() {
                if !root
                    .horizontal
                    .iter()
                    .chain(&root.vertical)
                    .take(i)
                    .any(|p| p.profile.id == s.profile.id)
                {
                    profiles += 1;
                    runs += s.profile.sizes.len();
                }
            }
            bytes += root.owned_bytes();
        }
        if breaks > 512
            || profiles > 512
            || runs > MAX_RUNS
            || bytes > 8 * 1024 * 1024
            || self.stats().allocated_nodes > limits.allocated_cells
            || self.stats().estimated_owned_bytes > limits.bytes
        {
            return Err(Error::Limit);
        }
        Ok(())
    }
    pub(crate) fn geometry_bytes(&self) -> usize {
        self.geometry.iter().map(|(_, r)| r.owned_bytes()).sum()
    }
    pub(crate) fn geometry_identities(&self) -> usize {
        self.geometry
            .iter()
            .map(|(_, r)| (r.next_id - 1) as usize)
            .sum()
    }
}
impl ScopeCapture {
    pub fn geometry_map(&self, visible: Option<&[u32]>, cancel: &CancellationToken) -> Result<Map> {
        self.calculation.geometry_map(self.scope, visible, cancel)
    }
}
impl crate::CalculationCapture {
    #[doc(hidden)]
    pub fn geometry_layout_work(&self, scope: GridScopeId) -> Result<usize> {
        crate::producers::layout_work(&self.pinned, scope)
    }
    pub fn geometry_layout_review(
        &self,
        scope: GridScopeId,
        cancel: &CancellationToken,
    ) -> Result<(Option<[u8; 32]>, bool)> {
        let current = crate::producers::layout_stamp(&self.pinned, scope, cancel)?;
        let review = current.is_some()
            && self
                .pinned
                .geometry
                .get(&scope)
                .is_some_and(|root| root.reviewed_layout != current);
        Ok((current, review))
    }
    pub fn positional_geometry(&self, scope: GridScopeId) -> Result<bool> {
        Ok(axes_for(&self.pinned, scope)?.positional)
    }
    pub fn geometry_map(
        &self,
        scope: GridScopeId,
        visible: Option<&[u32]>,
        cancel: &CancellationToken,
    ) -> Result<Map> {
        let data = &self.pinned;
        let axes = axes_for(data, scope)?;
        let mut map = if let Some(root) = data.geometry.get(&scope) {
            root.map(&axes, visible, &data.memory, cancel)
        } else {
            let widths = [SizeRun {
                first: 1,
                last: axes.columns.length,
                size: DEFAULT_WIDTH,
            }];
            let heights = [SizeRun {
                first: 1,
                last: axes.rows.length,
                size: DEFAULT_HEIGHT,
            }];
            Map::build(
                axes.rows.length,
                axes.columns.length,
                &[Band {
                    start: 1,
                    sizes: &widths,
                }],
                &[Band {
                    start: 1,
                    sizes: &heights,
                }],
                visible,
                &data.memory,
                cancel,
            )
            .map_err(failure)
        }?;
        if let Some(directory) = &data.producers
            && let Some(binding) = directory.binding(scope)
        {
            let (rows, columns) = directory.producer(binding.producer)?.extent();
            map.clip_extent(rows, columns).map_err(failure)?;
        }
        Ok(map)
    }
}
