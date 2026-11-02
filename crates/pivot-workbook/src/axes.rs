//! Compact, charged immutable axis runs; rank is never an axis identity.
use super::*;
use pivot_core::model::integration::resources::{Footprint, Memory, Shared};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxisRun {
    pub first: u32,
    pub count: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxisImage {
    pub next: u32,
    pub runs: Vec<AxisRun>,
}
impl Footprint for AxisImage {
    fn heap_bytes(&self) -> usize {
        self.runs.capacity() * std::mem::size_of::<AxisRun>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AxisLayout {
    pub image: AxisImage,
    ends: Vec<u32>,
}
impl std::ops::Deref for AxisLayout {
    type Target = AxisImage;
    fn deref(&self) -> &AxisImage {
        &self.image
    }
}
impl Footprint for AxisLayout {
    fn heap_bytes(&self) -> usize {
        self.image.heap_bytes() + self.ends.capacity() * std::mem::size_of::<u32>()
    }
}
#[derive(Clone)]
pub(crate) struct Axis {
    pub namespace: Uuid,
    pub length: u32,
    pub layout: Option<Shared<AxisLayout>>,
}
impl std::fmt::Debug for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Axis")
            .field("namespace", &self.namespace)
            .field("length", &self.length)
            .finish()
    }
}
impl PartialEq for Axis {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace
            && self.length == other.length
            && match (&self.layout, &other.layout) {
                (None, None) => true,
                (Some(a), Some(b)) => **a == **b,
                (Some(a), None) | (None, Some(a)) => {
                    a.next == self.length + 1
                        && a.runs.as_slice()
                            == [AxisRun {
                                first: 1,
                                count: self.length,
                            }]
                }
            }
    }
}
impl Eq for Axis {}
impl Axis {
    pub fn at(&self, position: u32) -> Result<AxisId> {
        if position == 0 || position > self.length {
            return Err(Error::Position);
        }
        let sequence = if let Some(layout) = &self.layout {
            let index = layout.ends.partition_point(|end| *end < position);
            let before = if index == 0 {
                0
            } else {
                layout.ends[index - 1]
            };
            layout.runs.get(index).ok_or(Error::InvalidTarget)?.first + position - before - 1
        } else {
            position
        };
        Ok(AxisId {
            namespace: self.namespace,
            sequence,
        })
    }
    pub fn image(&self) -> AxisImage {
        self.layout.as_ref().map_or_else(
            || AxisImage {
                next: self.length + 1,
                runs: vec![AxisRun {
                    first: 1,
                    count: self.length,
                }],
            },
            |p| p.image.clone(),
        )
    }
    pub fn from_image(
        namespace: Uuid,
        length: u32,
        image: &AxisImage,
        memory: &Memory,
    ) -> Result<Self> {
        if namespace.is_nil()
            || length == 0
            || image.runs.is_empty()
            || image.runs.len() > length as usize
        {
            return Err(Error::InvalidTarget);
        }
        if image.next == length + 1
            && image.runs.as_slice()
                == [AxisRun {
                    first: 1,
                    count: length,
                }]
        {
            return Ok(Self {
                namespace,
                length,
                layout: None,
            });
        }
        let _scratch = memory
            .reserve(image.runs.len().saturating_mul(64))
            .map_err(|_| Error::Limit)?;
        let mut ranges = Vec::with_capacity(image.runs.len());
        let mut ends = Vec::with_capacity(image.runs.len());
        let mut total = 0u32;
        for run in &image.runs {
            let end = run.first.checked_add(run.count).ok_or(Error::Exhausted)?;
            if run.first == 0 || run.count == 0 || end > image.next {
                return Err(Error::InvalidTarget);
            }
            total = total.checked_add(run.count).ok_or(Error::Exhausted)?;
            ends.push(total);
            ranges.push((run.first, end));
        }
        ranges.sort_unstable();
        if total != length || ranges.windows(2).any(|r| r[0].1 > r[1].0) {
            return Err(Error::InvalidTarget);
        }
        Ok(Self {
            namespace,
            length,
            layout: Some(
                memory
                    .alloc(AxisLayout {
                        image: image.clone(),
                        ends,
                    })
                    .map_err(|_| Error::Limit)?,
            ),
        })
    }
    pub fn changed(&self, change: structure::Change, memory: &Memory) -> Result<Self> {
        let run_count = self.layout.as_ref().map_or(1, |p| p.runs.len());
        let _scratch = memory
            .reserve((run_count + 3).saturating_mul(64))
            .map_err(|_| Error::Limit)?;
        let image = self.image();
        let next = image
            .next
            .checked_add(change.count)
            .ok_or(Error::Exhausted)?;
        let mut runs: Vec<AxisRun> = Vec::with_capacity(run_count + 3);
        let mut append = |first: u32, count: u32| {
            if count == 0 {
                return;
            }
            if let Some(last) = runs.last_mut()
                && last.first.checked_add(last.count) == Some(first)
            {
                last.count += count;
            } else {
                runs.push(AxisRun { first, count });
            }
        };
        let mut slice = |start: u32, end: u32| {
            let mut rank = 1;
            for run in &image.runs {
                let a = start.max(rank);
                let b = end.min(rank + run.count);
                if b > a {
                    append(run.first + a - rank, b - a);
                }
                rank += run.count;
            }
        };
        slice(1, change.start);
        if change.action == structure::Action::Insert {
            // End the borrow of append held by slice before adding the inserted run.
            drop(slice);
            append(image.next, change.count);
            let mut rank = 1;
            for run in &image.runs {
                let a = change.start.max(rank);
                let b = (self.length - change.count + 1).min(rank + run.count);
                if b > a {
                    append(run.first + a - rank, b - a);
                }
                rank += run.count;
            }
        } else {
            slice(change.start + change.count, self.length + 1);
            drop(slice);
            append(image.next, change.count);
        }
        Self::from_image(
            self.namespace,
            self.length,
            &AxisImage { next, runs },
            memory,
        )
    }
    pub fn admitted_length(&self, length: u32, memory: &Memory) -> Result<Self> {
        if length < self.length {
            return Err(Error::Position);
        }
        if length == self.length {
            return Ok(self.clone());
        }
        let count = self.layout.as_ref().map_or(1, |p| p.runs.len());
        let _scratch = memory
            .reserve((count + 1).saturating_mul(64))
            .map_err(|_| Error::Limit)?;
        let mut image = self.image();
        let added = length - self.length;
        if let Some(last) = image.runs.last_mut()
            && last.first.checked_add(last.count) == Some(image.next)
        {
            last.count = last.count.checked_add(added).ok_or(Error::Exhausted)?;
        } else {
            image.runs.push(AxisRun {
                first: image.next,
                count: added,
            });
        }
        image.next = image.next.checked_add(added).ok_or(Error::Exhausted)?;
        Self::from_image(self.namespace, length, &image, memory)
    }
}
