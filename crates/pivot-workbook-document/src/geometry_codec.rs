//! WB16 geometry authority. Decoding bounds all nested totals before allocation;
//! workbook restore independently resolves axes and proves full topology.
use super::*;
use pivot_workbook::geometry::{Anchor, GeometryImage, ProfileImage, SectionImage, SizeImage};
impl Writer {
    fn anchor(&mut self, a: Option<Anchor>) {
        match a {
            None => self.n(0),
            Some(Anchor::Authored(id)) => {
                self.n(1);
                self.id(id.namespace);
                self.n(id.sequence.into());
            }
            Some(Anchor::Ordinal(n)) => {
                self.n(2);
                self.n(n.into());
            }
        }
    }
    pub(super) fn geometry(&mut self, images: &[GeometryImage]) {
        self.n(images.len() as u64);
        for image in images {
            self.scope(image.scope);
            self.n(image.next_id);
            self.bytes(image.reviewed_layout.as_ref().map_or(&[][..], |s| &s[..]));
            for sections in [&image.horizontal, &image.vertical] {
                self.n(sections.len() as u64);
                for s in sections {
                    self.n(s.id);
                    self.anchor(s.start);
                    self.n(s.profile);
                }
            }
            self.n(image.profiles.len() as u64);
            for p in &image.profiles {
                self.n(p.id);
                self.n(p.default as u64);
                self.n(p.sizes.len() as u64);
                for s in &p.sizes {
                    self.anchor(Some(s.first));
                    self.anchor(Some(s.last));
                    self.n(s.size as u64);
                }
            }
        }
    }
}
impl Reader<'_> {
    fn anchor(&mut self) -> Result<Option<Anchor>> {
        match self.n()? {
            0 => Ok(None),
            1 => Ok(Some(Anchor::Authored(AxisId {
                namespace: self.id()?,
                sequence: self.u32()?,
            }))),
            2 => Ok(Some(Anchor::Ordinal(self.u32()?))),
            _ => Err(malformed()),
        }
    }
    fn geometry_size(&mut self) -> Result<i64> {
        let n = i64::try_from(self.n()?).map_err(|_| malformed())?;
        if !(geometry::MIN_SIZE..=geometry::MAX_SIZE).contains(&n) {
            return Err(malformed());
        }
        Ok(n)
    }
    pub(super) fn geometry(&mut self) -> Result<Vec<GeometryImage>> {
        let mut images = Vec::new();
        let mut breaks = 0;
        let mut profiles = 0;
        let mut runs = 0;
        for _ in 0..self.count(262_144)? {
            let scope = self.scope()?;
            let next_id = self.n()?;
            let stamp = self.bytes(32)?;
            let reviewed_layout = if stamp.is_empty() {
                None
            } else {
                Some(stamp.try_into().map_err(|_| malformed())?)
            };
            let mut bands = Vec::new();
            for _ in 0..2 {
                let count = self.count(geometry::MAX_BANDS)?;
                breaks += count.saturating_sub(1);
                if breaks > 512 {
                    return Err(malformed());
                }
                let mut sections = Vec::with_capacity(count);
                for _ in 0..count {
                    sections.push(SectionImage {
                        id: self.n()?,
                        start: self.anchor()?,
                        profile: self.n()?,
                    });
                }
                bands.push(sections);
            }
            let count = self.count(512)?;
            profiles += count;
            if profiles > 512 {
                return Err(malformed());
            }
            let mut ps = Vec::with_capacity(count);
            for _ in 0..count {
                let id = self.n()?;
                let default = self.geometry_size()?;
                let count = self.count(geometry::MAX_RUNS)?;
                runs += count;
                if runs > geometry::MAX_RUNS {
                    return Err(malformed());
                }
                let mut sizes = Vec::with_capacity(count);
                for _ in 0..count {
                    let first = self.anchor()?.ok_or_else(malformed)?;
                    let last = self.anchor()?.ok_or_else(malformed)?;
                    sizes.push(SizeImage {
                        first,
                        last,
                        size: self.geometry_size()?,
                    });
                }
                ps.push(ProfileImage { id, default, sizes });
            }
            let vertical = bands.pop().unwrap();
            let horizontal = bands.pop().unwrap();
            images.push(GeometryImage {
                scope,
                next_id,
                reviewed_layout,
                horizontal,
                vertical,
                profiles: ps,
            });
        }
        Ok(images)
    }
}
