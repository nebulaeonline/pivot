//! Document-owned sparse presentation, independent of calculation identities.
use super::*;
use pivot_core::model::integration::{CancellationToken, resources::Footprint};
use pivot_types::{PivotDecimal, RoundingMode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumericFormat {
    #[default]
    General,
    Number,
    Currency,
    Percentage,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    Auto,
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Currency {
    #[default]
    Dollar,
    Euro,
    Pound,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub numeric: NumericFormat,
    pub places: u8,
    pub rounding: RoundingMode,
    pub grouping: bool,
    pub currency: Currency,
    pub bold: bool,
    pub italic: bool,
    pub alignment: Alignment,
    pub foreground: Option<u32>,
    pub background: Option<u32>,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            numeric: NumericFormat::General,
            places: 2,
            rounding: RoundingMode::default(),
            grouping: false,
            currency: Currency::Dollar,
            bold: false,
            italic: false,
            alignment: Alignment::Auto,
            foreground: None,
            background: None,
        }
    }
}
impl Footprint for Style {}
impl Style {
    pub fn validate(self) -> Result<()> {
        if self.places > 28
            || self.foreground.is_some_and(|c| c > 0xffffff)
            || self.background.is_some_and(|c| c > 0xffffff)
        {
            return Err(Error::InvalidTarget);
        }
        Ok(())
    }
    /// Fixed-size current-format encoding; invalid enum values never silently default.
    pub fn encode(self) -> [u8; 17] {
        let mut out = [0; 17];
        out[..9].copy_from_slice(&[
            self.numeric as u8,
            self.places,
            self.rounding as u8,
            self.grouping as u8,
            self.currency as u8,
            self.bold as u8,
            self.italic as u8,
            self.alignment as u8,
            0,
        ]);
        for (i, c) in [self.foreground, self.background].into_iter().enumerate() {
            out[9 + i * 4..13 + i * 4].copy_from_slice(&c.unwrap_or(u32::MAX).to_le_bytes());
        }
        out
    }
    pub fn decode(b: &[u8]) -> Result<Self> {
        if b.len() != 17 || b[8] != 0 || b[3] > 1 || b[5] > 1 || b[6] > 1 {
            return Err(Error::InvalidTarget);
        }
        let numeric = match b[0] {
            0 => NumericFormat::General,
            1 => NumericFormat::Number,
            2 => NumericFormat::Currency,
            3 => NumericFormat::Percentage,
            _ => return Err(Error::InvalidTarget),
        };
        let rounding = match b[2] {
            0 => RoundingMode::ToEven,
            1 => RoundingMode::MidpointAwayFromZero,
            2 => RoundingMode::MidpointTowardZero,
            3 => RoundingMode::ToZero,
            4 => RoundingMode::AwayFromZero,
            5 => RoundingMode::ToNegativeInfinity,
            6 => RoundingMode::ToPositiveInfinity,
            _ => return Err(Error::InvalidTarget),
        };
        let currency = match b[4] {
            0 => Currency::Dollar,
            1 => Currency::Euro,
            2 => Currency::Pound,
            _ => return Err(Error::InvalidTarget),
        };
        let alignment = match b[7] {
            0 => Alignment::Auto,
            1 => Alignment::Left,
            2 => Alignment::Center,
            3 => Alignment::Right,
            _ => return Err(Error::InvalidTarget),
        };
        let color = |i| {
            let v = u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
            (v != u32::MAX).then_some(v)
        };
        let value = Self {
            numeric,
            places: b[1],
            rounding,
            grouping: b[3] != 0,
            currency,
            bold: b[5] != 0,
            italic: b[6] != 0,
            alignment,
            foreground: color(9),
            background: color(13),
        };
        value.validate()?;
        Ok(value)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Field<T> {
    #[default]
    Keep,
    Set(T),
    Reset,
}
impl<T: Copy> Field<T> {
    fn apply(self, old: T, default: T) -> T {
        match self {
            Self::Keep => old,
            Self::Set(v) => v,
            Self::Reset => default,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct StylePatch {
    pub numeric: Field<NumericFormat>,
    pub places: Field<u8>,
    pub rounding: Field<RoundingMode>,
    pub grouping: Field<bool>,
    pub currency: Field<Currency>,
    pub bold: Field<bool>,
    pub italic: Field<bool>,
    pub alignment: Field<Alignment>,
    pub foreground: Field<Option<u32>>,
    pub background: Field<Option<u32>>,
}
impl StylePatch {
    pub fn apply(self, old: Style) -> Result<Style> {
        let d = Style::default();
        let s = Style {
            numeric: self.numeric.apply(old.numeric, d.numeric),
            places: self.places.apply(old.places, d.places),
            rounding: self.rounding.apply(old.rounding, d.rounding),
            grouping: self.grouping.apply(old.grouping, d.grouping),
            currency: self.currency.apply(old.currency, d.currency),
            bold: self.bold.apply(old.bold, d.bold),
            italic: self.italic.apply(old.italic, d.italic),
            alignment: self.alignment.apply(old.alignment, d.alignment),
            foreground: self.foreground.apply(old.foreground, d.foreground),
            background: self.background.apply(old.background, d.background),
        };
        s.validate()?;
        Ok(s)
    }
}
/// Revisable admission policy, unrelated to graph capacity and encoded identifiers.
#[derive(Clone, Copy, Debug)]
pub struct StyleLimits {
    pub positions: usize,
    pub command_positions: u64,
}
impl Default for StyleLimits {
    fn default() -> Self {
        Self {
            positions: 4096,
            command_positions: 4096,
        }
    }
}

/// Decimal digit arithmetic only: never scales through bounded Decimal96 multiplication.
pub fn render_decimal(value: PivotDecimal, style: Style) -> Result<String> {
    style.validate()?;
    if style.numeric == NumericFormat::General {
        return Ok(value.to_string());
    }
    let (lo, mid, hi, negative, scale) = value.parts();
    let coefficient = u128::from(lo) | (u128::from(mid) << 32) | (u128::from(hi) << 64);
    let shift = if style.numeric == NumericFormat::Percentage {
        2
    } else {
        0
    };
    let cut = scale as i32 - shift - i32::from(style.places);
    let mut digits = coefficient.to_string().into_bytes();
    if cut > 0 {
        let divisor = 10u128.pow(cut as u32);
        let q = coefficient / divisor;
        let r = coefficient % divisor;
        let up = match style.rounding {
            RoundingMode::ToEven => r * 2 > divisor || (r * 2 == divisor && q % 2 == 1),
            RoundingMode::MidpointAwayFromZero => r * 2 >= divisor,
            RoundingMode::MidpointTowardZero => r * 2 > divisor,
            RoundingMode::ToZero => false,
            RoundingMode::AwayFromZero => r != 0,
            RoundingMode::ToNegativeInfinity => negative && r != 0,
            RoundingMode::ToPositiveInfinity => !negative && r != 0,
        };
        digits = (q + u128::from(up)).to_string().into_bytes();
    } else {
        digits.extend(std::iter::repeat_n(b'0', (-cut) as usize));
    }
    let zero = digits.iter().all(|d| *d == b'0');
    let places = usize::from(style.places);
    if digits.len() <= places {
        let mut pad = vec![b'0'; places + 1 - digits.len()];
        pad.extend(digits);
        digits = pad;
    }
    let split = digits.len() - places;
    let mut out = String::with_capacity(96);
    if negative && !zero {
        out.push('-');
    }
    if style.numeric == NumericFormat::Currency {
        out.push(match style.currency {
            Currency::Dollar => '$',
            Currency::Euro => '€',
            Currency::Pound => '£',
        });
    }
    for (i, d) in digits[..split].iter().enumerate() {
        if style.grouping && i > 0 && (split - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(char::from(*d));
    }
    if places > 0 {
        out.push('.');
        for d in &digits[split..] {
            out.push(char::from(*d));
        }
    }
    if style.numeric == NumericFormat::Percentage {
        out.push('%');
    }
    debug_assert!(out.len() <= 128);
    Ok(out)
}

impl Workbook {
    pub fn style_at(&self, sheet: WorksheetId, position: Position) -> Result<Style> {
        let info = self.sheet_info(sheet)?;
        info.rows.at(position.row)?;
        info.columns.at(position.column)?;
        Ok(self
            .persistent
            .styles
            .get(&sheets::at(sheet, position))
            .copied()
            .unwrap_or_default())
    }
    pub fn styled_positions(&self) -> usize {
        self.persistent.styles.len()
    }
    pub fn style_limits(&self) -> StyleLimits {
        self.limits.styles
    }
    pub fn styles(&self) -> impl Iterator<Item = (WorksheetId, Position, Style)> + '_ {
        self.persistent
            .styles
            .iter()
            .map(|(p, s)| (p.worksheet, p.position, *s))
    }
    pub fn format_sheet(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        rectangle: Rectangle,
        patch: StylePatch,
        reset: bool,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        let info = self.sheet_info(sheet)?;
        info.rows.at(rectangle.first.row)?;
        info.rows.at(rectangle.last.row)?;
        info.columns.at(rectangle.first.column)?;
        info.columns.at(rectangle.last.column)?;
        let height = rectangle
            .last
            .row
            .checked_sub(rectangle.first.row)
            .ok_or(Error::Position)? as u64
            + 1;
        let width = rectangle
            .last
            .column
            .checked_sub(rectangle.first.column)
            .ok_or(Error::Position)? as u64
            + 1;
        if height * width > self.limits.styles.command_positions {
            return Err(Error::Limit);
        }
        let scratch = usize::try_from(height * width)
            .ok()
            .and_then(|area| area.checked_mul(64))
            .and_then(|n| n.checked_add(4096))
            .ok_or(Error::Limit)?;
        let _scratch = self
            .persistent
            .memory
            .reserve(scratch)
            .map_err(|_| Error::Limit)?;
        let mut changes = Vec::new();
        for r in rectangle.first.row..=rectangle.last.row {
            for c in rectangle.first.column..=rectangle.last.column {
                if cancel.is_cancelled() {
                    return Err(Error::Graph("cancelled".into()));
                }
                let p = Position::new(r, c);
                let old = self.style_at(sheet, p)?;
                let new = if reset {
                    Style::default()
                } else {
                    patch.apply(old)?
                };
                if old != new {
                    changes.push((sheet, p, new));
                }
            }
        }
        self.install_styles(expected, &changes, cancel)
    }
    /// Transactional style-only publication; graph revision and allocations remain untouched.
    pub(crate) fn install_styles(
        &mut self,
        expected: Expected,
        changes: &[(WorksheetId, Position, Style)],
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active {
            return Err(Error::InvalidTarget);
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let mut changed = false;
        for &(sheet, p, s) in changes {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            self.axes_at_sheet(sheet, p)?;
            s.validate()?;
            let key = sheets::at(sheet, p);
            let old = data.styles.get(&key).copied().unwrap_or_default();
            if old == s {
                continue;
            }
            changed = true;
            if s == Style::default() {
                data.styles.remove(&key).map_err(|_| Error::Limit)?;
            } else {
                data.styles.insert(key, s).map_err(|_| Error::Limit)?;
            }
        }
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        if data.styles.len() > self.limits.styles.positions
            || data.stats().estimated_owned_bytes > self.limits.bytes
        {
            return Err(Error::Limit);
        }
        if changed {
            let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
            self.persistent = data.freeze();
            self.state = next;
        }
        Ok(self.expected())
    }
}
