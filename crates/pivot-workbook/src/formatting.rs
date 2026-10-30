//! Document-owned sparse presentation, independent of calculation identities.
use super::*;
use pivot_core::model::integration::{CancellationToken, resources::Footprint};
use pivot_types::{PivotDecimal, RoundingMode};
pub(crate) mod regions;
pub use regions::Region;
pub(crate) use regions::Store;

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
    Inherit,
}
impl<T: Copy> Field<T> {
    fn apply(self, old: T, default: T) -> T {
        match self {
            Self::Keep => old,
            Self::Set(v) => v,
            Self::Reset | Self::Inherit => default,
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
    pub bytes: usize,
    pub command_work: usize,
    pub query_work: usize,
}
impl Default for StyleLimits {
    fn default() -> Self {
        Self {
            positions: 65_536,
            command_positions: 4096,
            bytes: 8 * 1024 * 1024,
            command_work: 2_097_152,
            query_work: 1_048_576,
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

impl StylePatch {
    fn intents(self) -> [u8; 10] {
        fn intent<T>(f: Field<T>) -> u8 {
            match f {
                Field::Keep => 0,
                Field::Inherit => 2,
                _ => 1,
            }
        }
        [
            intent(self.numeric),
            intent(self.places),
            intent(self.rounding),
            intent(self.grouping),
            intent(self.currency),
            intent(self.bold),
            intent(self.italic),
            intent(self.alignment),
            intent(self.foreground),
            intent(self.background),
        ]
    }
    pub fn inherit() -> Self {
        Self {
            numeric: Field::Inherit,
            places: Field::Inherit,
            rounding: Field::Inherit,
            grouping: Field::Inherit,
            currency: Field::Inherit,
            bold: Field::Inherit,
            italic: Field::Inherit,
            alignment: Field::Inherit,
            foreground: Field::Inherit,
            background: Field::Inherit,
        }
    }
}
impl persistent::Data {
    pub(crate) fn paint_style(
        &mut self,
        sheet: WorksheetId,
        r: Rectangle,
        patch: StylePatch,
        reset: bool,
        limits: StyleLimits,
        cancel: &CancellationToken,
    ) -> Result<bool> {
        let available = limits
            .command_work
            .checked_sub(self.styles.work)
            .ok_or(Error::Limit)?;
        let mut work = regions::Work::new(available, cancel);
        let v = regions::values(if reset {
            Style::default()
        } else {
            patch.apply(Style::default())?
        });
        let intents = if reset { [1; 10] } else { patch.intents() };
        let mut changed = false;
        for property in 0..10 {
            let intent = intents[property];
            if intent != 0 {
                changed |= self.styles.paint(
                    sheet,
                    r,
                    property as u8,
                    if intent == 2 { None } else { Some(v[property]) },
                    limits,
                    &mut work,
                )?;
            }
        }
        self.styles.work += work.used();
        Ok(changed)
    }
    pub(crate) fn resolved_style(
        &self,
        sheet: WorksheetId,
        p: Position,
        limits: StyleLimits,
        cancel: &CancellationToken,
    ) -> Result<Style> {
        self.styles
            .style_at(sheet, p, &mut regions::Work::new(limits.query_work, cancel))
    }
    pub(crate) fn snapshot_style(
        &mut self,
        sheet: WorksheetId,
        p: Position,
        s: Style,
        limits: StyleLimits,
        cancel: &CancellationToken,
    ) -> Result<bool> {
        s.validate()?;
        self.paint_style(
            sheet,
            Rectangle { first: p, last: p },
            StylePatch {
                numeric: Field::Set(s.numeric),
                places: Field::Set(s.places),
                rounding: Field::Set(s.rounding),
                grouping: Field::Set(s.grouping),
                currency: Field::Set(s.currency),
                bold: Field::Set(s.bold),
                italic: Field::Set(s.italic),
                alignment: Field::Set(s.alignment),
                foreground: Field::Set(s.foreground),
                background: Field::Set(s.background),
            },
            false,
            limits,
            cancel,
        )
    }
}
impl Workbook {
    fn check_style_rectangle(&self, sheet: WorksheetId, r: Rectangle) -> Result<()> {
        if r.first.row > r.last.row || r.first.column > r.last.column {
            return Err(Error::Position);
        }
        self.axes_at_sheet(sheet, r.first)?;
        self.axes_at_sheet(sheet, r.last)?;
        Ok(())
    }
    pub fn style_at(&self, sheet: WorksheetId, p: Position) -> Result<Style> {
        self.axes_at_sheet(sheet, p)?;
        self.persistent
            .resolved_style(sheet, p, self.limits.styles, &Default::default())
    }
    /// Stored property rectangles, not the number of positions they cover.
    pub fn style_regions(&self) -> usize {
        self.persistent.styles.len()
    }
    pub fn style_command_work(&self) -> usize {
        self.persistent.styles.work
    }
    pub fn style_bytes(&self) -> usize {
        self.persistent.styles.bytes()
    }
    pub fn style_limits(&self) -> StyleLimits {
        self.limits.styles
    }
    pub fn regions(&self) -> impl Iterator<Item = Region> + '_ {
        self.persistent.styles.regions()
    }
    pub fn worksheet_defaults(&self) -> impl Iterator<Item = (WorksheetId, Style)> + '_ {
        self.persistent.styles.defaults()
    }
    pub fn worksheet_default(&self, sheet: WorksheetId) -> Result<Style> {
        self.sheet_info(sheet)?;
        Ok(self.persistent.styles.default_style(sheet))
    }
    pub fn style_summary(&self, sheet: WorksheetId, r: Rectangle) -> Result<(Style, u32)> {
        self.check_style_rectangle(sheet, r)?;
        self.persistent.styles.summary(
            sheet,
            r,
            &mut regions::Work::new(self.limits.styles.query_work, &Default::default()),
        )
    }
    pub fn style_viewport(
        &self,
        sheet: WorksheetId,
        r: Rectangle,
    ) -> Result<Vec<(Position, Style)>> {
        self.check_style_rectangle(sheet, r)?;
        let area =
            u64::from(r.last.row - r.first.row + 1) * u64::from(r.last.column - r.first.column + 1);
        if area > self.limits.styles.command_positions {
            return Err(Error::Limit);
        }
        // Resolve each map once for the complete viewport under one shared work budget.
        self.persistent
            .styles
            .viewport(sheet, r, self.limits.styles)
    }
    pub fn set_worksheet_default(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        patch: StylePatch,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        self.sheet_info(sheet)?;
        self.formatting_allowed(cancel)?;
        if patch.intents().contains(&2) {
            return Err(Error::InvalidTarget);
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let style = patch.apply(data.styles.default_style(sheet))?;
        let changed = data.styles.set_default(sheet, style)?;
        self.publish_styles(data, changed, cancel)
    }
    fn formatting_allowed(&self, cancel: &CancellationToken) -> Result<()> {
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active {
            return Err(Error::InvalidTarget);
        }
        Ok(())
    }
    pub fn format_sheet(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        r: Rectangle,
        patch: StylePatch,
        reset: bool,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        self.check_style_rectangle(sheet, r)?;
        self.formatting_allowed(cancel)?;
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        data.styles.work = 0;
        let changed = data.paint_style(sheet, r, patch, reset, self.limits.styles, cancel)?;
        self.publish_styles(data, changed, cancel)
    }
    fn publish_styles(
        &mut self,
        data: pivot_core::model::integration::resources::Reclaimable<persistent::Data>,
        changed: bool,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.formatting_allowed(cancel)?;
        if data.styles.bytes() > self.limits.styles.bytes
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

impl CalculationCapture {
    pub fn style_at(&self, sheet: WorksheetId, p: Position) -> Result<Style> {
        let info = self
            .pinned
            .sheets
            .as_ref()
            .and_then(|s| s.0.iter().find(|s| s.id == sheet))
            .ok_or(Error::InvalidTarget)?;
        info.rows.at(p.row)?;
        info.columns.at(p.column)?;
        self.pinned
            .resolved_style(sheet, p, self.style_limits, &Default::default())
    }
}
