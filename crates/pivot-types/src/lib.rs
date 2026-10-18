//! Transport-independent Pivot numeric types.

use rust_decimal::{Decimal, RoundingStrategy};
use std::{fmt, str::FromStr};

/// Extensible value boundary. Additional numeric types require explicit coercion rules.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Decimal,
}
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PivotValue {
    Decimal(PivotDecimal),
}
impl PivotValue {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Decimal(_) => ValueType::Decimal,
        }
    }
}
impl From<PivotDecimal> for PivotValue {
    fn from(value: PivotDecimal) -> Self {
        Self::Decimal(value)
    }
}

/// Decimal arithmetic errors, independent of the backing implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalError {
    InvalidLiteral,
    Overflow,
    DivisionByZero,
    InvalidScale,
}

impl fmt::Display for DecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLiteral => "invalid or unrepresentable decimal literal",
            Self::Overflow => "decimal overflow",
            Self::DivisionByZero => "division by zero",
            Self::InvalidScale => "decimal scale must be between 0 and 28",
        })
    }
}
impl std::error::Error for DecimalError {}

/// Explicit rounding of a value to a decimal place. Independent of the
/// precision reduction that can happen inside arithmetic operations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    ToEven,
    #[default]
    MidpointAwayFromZero,
    MidpointTowardZero,
    ToZero,
    AwayFromZero,
    ToNegativeInfinity,
    ToPositiveInfinity,
}

impl From<RoundingMode> for RoundingStrategy {
    fn from(mode: RoundingMode) -> Self {
        match mode {
            RoundingMode::ToEven => Self::MidpointNearestEven,
            RoundingMode::MidpointAwayFromZero => Self::MidpointAwayFromZero,
            RoundingMode::MidpointTowardZero => Self::MidpointTowardZero,
            RoundingMode::ToZero => Self::ToZero,
            RoundingMode::AwayFromZero => Self::AwayFromZero,
            RoundingMode::ToNegativeInfinity => Self::ToNegativeInfinity,
            RoundingMode::ToPositiveInfinity => Self::ToPositiveInfinity,
        }
    }
}

/// A 96-bit significand with scale 0..=28. No floating-point conversions.
/// Arithmetic results are normalized; precision reduction inside arithmetic
/// retains backend behavior, independently of explicit rounding modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PivotDecimal(Decimal);

impl PivotDecimal {
    /// Lossless significand boundary, independent of backend memory layout.
    pub fn from_parts(
        lo: u32,
        mid: u32,
        hi: u32,
        negative: bool,
        scale: u32,
    ) -> Result<Self, DecimalError> {
        if scale > 28 {
            return Err(DecimalError::InvalidScale);
        }
        Ok(Self(Decimal::from_parts(lo, mid, hi, negative, scale)))
    }
    pub fn parts(self) -> (u32, u32, u32, bool, u32) {
        let parts = self.0.unpack();
        (parts.lo, parts.mid, parts.hi, parts.negative, parts.scale)
    }
    pub const ZERO: Self = Self(Decimal::ZERO);
    pub const ONE: Self = Self(Decimal::ONE);
    pub const MAX: Self = Self(Decimal::MAX);
    pub const MIN: Self = Self(Decimal::MIN);

    pub fn scale(self) -> u32 {
        self.0.scale()
    }
    /// Round to 0..=28 decimal places using nearest, ties away from zero.
    pub fn round(self, places: u32) -> Result<Self, DecimalError> {
        self.round_with_mode(places, RoundingMode::default())
    }
    pub fn round_with_mode(self, places: u32, mode: RoundingMode) -> Result<Self, DecimalError> {
        if places > 28 {
            return Err(DecimalError::InvalidScale);
        }
        Ok(Self(self.0.round_dp_with_strategy(places, mode.into())))
    }
    /// Remove insignificant fractional zeros, canonicalizing numeric zero.
    pub fn normalize(self) -> Self {
        Self(self.0.normalize())
    }
    pub fn checked_add(self, rhs: Self) -> Result<Self, DecimalError> {
        self.0
            .checked_add(rhs.0)
            .map(|value| Self(value.normalize()))
            .ok_or(DecimalError::Overflow)
    }
    pub fn checked_sub(self, rhs: Self) -> Result<Self, DecimalError> {
        self.0
            .checked_sub(rhs.0)
            .map(|value| Self(value.normalize()))
            .ok_or(DecimalError::Overflow)
    }
    pub fn checked_mul(self, rhs: Self) -> Result<Self, DecimalError> {
        self.0
            .checked_mul(rhs.0)
            .map(|value| Self(value.normalize()))
            .ok_or(DecimalError::Overflow)
    }
    pub fn checked_div(self, rhs: Self) -> Result<Self, DecimalError> {
        if rhs == Self::ZERO {
            return Err(DecimalError::DivisionByZero);
        }
        self.0
            .checked_div(rhs.0)
            .map(|value| Self(value.normalize()))
            .ok_or(DecimalError::Overflow)
    }
    pub fn checked_neg(self) -> Result<Self, DecimalError> {
        Self::ZERO.checked_sub(self)
    }
}
impl From<i64> for PivotDecimal {
    fn from(value: i64) -> Self {
        Self(Decimal::from(value))
    }
}
impl FromStr for PivotDecimal {
    type Err = DecimalError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
        let mut parts = digits.split('.');
        let whole = parts.next().unwrap_or_default();
        let fraction = parts.next();
        if whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || fraction.is_some_and(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
            || parts.next().is_some()
        {
            return Err(DecimalError::InvalidLiteral);
        }
        Decimal::from_str_exact(value)
            .map(Self)
            .map_err(|_| DecimalError::InvalidLiteral)
    }
}
impl fmt::Display for PivotDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(s: &str) -> PivotDecimal {
        s.parse().unwrap()
    }
    #[test]
    fn exact_decimal_arithmetic() {
        assert_eq!(d("0.1").checked_add(d("0.2")), Ok(d("0.3")));
        let net = d("250000").checked_sub(d("85000")).unwrap();
        assert_eq!(net, d("165000"));
        assert_eq!(net.checked_div(d("250000")), Ok(d("0.66")));
        assert_eq!(d("1.25").checked_mul(d("4")), Ok(d("5")));
        assert_eq!(d("-2").checked_neg(), Ok(d("2")));
    }
    #[test]
    fn limits_and_errors() {
        assert_eq!(
            PivotDecimal::MAX.checked_add(PivotDecimal::ONE),
            Err(DecimalError::Overflow)
        );
        assert_eq!(
            PivotDecimal::MIN.checked_sub(PivotDecimal::ONE),
            Err(DecimalError::Overflow)
        );
        assert_eq!(
            PivotDecimal::MAX.checked_mul(d("2")),
            Err(DecimalError::Overflow)
        );
        assert_eq!(
            PivotDecimal::MAX.checked_div(d("0.1")),
            Err(DecimalError::Overflow)
        );
        assert_eq!(
            d("1").checked_div(d("-0.00")),
            Err(DecimalError::DivisionByZero)
        );
        assert_eq!(d("1.2300").scale(), 4);
        assert_eq!(d("1.2300").to_string(), "1.2300");
        assert_eq!(d("1.0"), d("1"));
        assert_eq!(d("0.0000000000000000000000000001").scale(), 28);
        for s in [
            "",
            "1e3",
            "1_000",
            ".5",
            "1.",
            " 1",
            "NaN",
            "79228162514264337593543950336",
            "0.00000000000000000000000000001",
        ] {
            assert!(s.parse::<PivotDecimal>().is_err(), "{s}");
        }
    }
    #[test]
    fn provisional_backend_precision() {
        assert_eq!(
            d("1").checked_div(d("3")).unwrap().to_string(),
            "0.3333333333333333333333333333"
        );
        assert_eq!(
            d("0.0000000000000000000000000001").checked_mul(d("0.1")),
            Ok(PivotDecimal::ZERO)
        );
    }
    #[test]
    fn rounding_modes_and_default() {
        use RoundingMode::*;
        for (mode, positive, negative) in [
            (ToEven, "2", "-2"),
            (MidpointAwayFromZero, "3", "-3"),
            (MidpointTowardZero, "2", "-2"),
            (ToZero, "2", "-2"),
            (AwayFromZero, "3", "-3"),
            (ToNegativeInfinity, "2", "-3"),
            (ToPositiveInfinity, "3", "-2"),
        ] {
            assert_eq!(d("2.5").round_with_mode(0, mode), Ok(d(positive)));
            assert_eq!(d("-2.5").round_with_mode(0, mode), Ok(d(negative)));
        }
        for (input, output) in [
            ("2.499999", "2"),
            ("2.5", "3"),
            ("-2.5", "-3"),
            ("1.245", "1.25"),
        ] {
            let places = if input == "1.245" { 2 } else { 0 };
            assert_eq!(d(input).round(places), Ok(d(output)));
        }
        assert_eq!(d("1").round(29), Err(DecimalError::InvalidScale));
        assert_eq!(d("1.2300").normalize().to_string(), "1.23");
        assert_eq!(d("1.00").checked_add(d("0.00")).unwrap().scale(), 0);
        assert_eq!(d("1.00").checked_mul(d("0.00")).unwrap().scale(), 0);
        assert_eq!(d("1.00").checked_div(d("2")).unwrap().to_string(), "0.5");
    }
}
