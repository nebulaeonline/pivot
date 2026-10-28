//! Private fixed-width arithmetic for aggregate reduction; no public decimal type.
use crate::{
    evaluation::EvaluationError,
    expression::AggregateKind,
    model::{NodeId, selection::SelectionId},
};
use pivot_types::{DecimalError, PivotDecimal};

pub(crate) struct Reduction {
    pub kind: AggregateKind,
    count: u64,
    total: PivotDecimal,
    extremum: Option<PivotDecimal>,
    mean: Mean,
    pub selection: Option<SelectionId>,
}
impl Reduction {
    pub fn new(kind: AggregateKind) -> Self {
        Self {
            kind,
            count: 0,
            total: PivotDecimal::ZERO,
            extremum: None,
            mean: Mean::default(),
            selection: None,
        }
    }
    pub fn push(&mut self, value: PivotDecimal) -> Result<(), DecimalError> {
        self.count = self.count.checked_add(1).ok_or(DecimalError::Overflow)?;
        match self.kind {
            AggregateKind::Sum => self.total = self.total.checked_add(value)?,
            AggregateKind::Average => self.mean.push(value)?,
            AggregateKind::Min => {
                self.extremum = Some(self.extremum.map_or(value, |v| v.min(value)))
            }
            AggregateKind::Max => {
                self.extremum = Some(self.extremum.map_or(value, |v| v.max(value)))
            }
            AggregateKind::Count => (),
        }
        Ok(())
    }
    pub fn finish(self, node: NodeId) -> Result<PivotDecimal, EvaluationError> {
        if self.count == 0 && !matches!(self.kind, AggregateKind::Sum | AggregateKind::Count) {
            return Err(match self.selection {
                Some(selection) => EvaluationError::NoNumericValues {
                    node,
                    operation: self.kind,
                    selection,
                },
                None => EvaluationError::NoNumericArguments {
                    node,
                    operation: self.kind,
                },
            });
        }
        match self.kind {
            AggregateKind::Sum => Ok(self.total),
            AggregateKind::Count => {
                i64::try_from(self.count)
                    .map(PivotDecimal::from)
                    .map_err(|_| EvaluationError::Arithmetic {
                        node,
                        error: DecimalError::Overflow,
                    })
            }
            AggregateKind::Min | AggregateKind::Max => Ok(self.extremum.unwrap().normalize()),
            AggregateKind::Average => self
                .mean
                .finish()
                .map_err(|error| EvaluationError::Arithmetic { node, error }),
        }
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Wide([u64; 4]);
impl Ord for Wide {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}
impl PartialOrd for Wide {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Wide {
    fn zero(self) -> bool {
        self.0 == [0; 4]
    }
    fn add(self, other: Self) -> Option<Self> {
        let mut out = [0; 4];
        let mut carry = 0u128;
        for (i, v) in out.iter_mut().enumerate() {
            let n = self.0[i] as u128 + other.0[i] as u128 + carry;
            *v = n as u64;
            carry = n >> 64;
        }
        (carry == 0).then_some(Self(out))
    }
    fn sub(self, other: Self) -> Self {
        debug_assert!(self >= other);
        let mut out = [0; 4];
        let mut borrow = false;
        for (i, v) in out.iter_mut().enumerate() {
            let (n, a) = self.0[i].overflowing_sub(other.0[i]);
            let (n, b) = n.overflowing_sub(u64::from(borrow));
            *v = n;
            borrow = a || b;
        }
        debug_assert!(!borrow);
        Self(out)
    }
    fn mul(self, rhs: u64) -> Option<Self> {
        let mut out = [0; 4];
        let mut carry = 0u128;
        for (i, v) in out.iter_mut().enumerate() {
            let n = self.0[i] as u128 * rhs as u128 + carry;
            *v = n as u64;
            carry = n >> 64;
        }
        (carry == 0).then_some(Self(out))
    }
    fn div(self, rhs: Self) -> (Self, Self) {
        debug_assert!(!rhs.zero());
        let mut q = Self::default();
        let mut r = Self::default();
        for bit in (0..256).rev() {
            debug_assert!(r.0[3] >> 63 == 0);
            r = r.mul(2).expect("bounded remainder");
            r.0[0] |= (self.0[bit / 64] >> (bit % 64)) & 1;
            if r >= rhs {
                r = r.sub(rhs);
                q.0[bit / 64] |= 1u64 << (bit % 64);
            }
        }
        (q, r)
    }
}
#[derive(Default)]
pub(crate) struct Mean {
    sum: Wide,
    negative: bool,
    count: u64,
}
impl Mean {
    pub(crate) fn push(&mut self, value: PivotDecimal) -> Result<(), DecimalError> {
        let (lo, mid, hi, negative, scale) = value.parts();
        let mut magnitude = Wide([lo as u64 | ((mid as u64) << 32), hi as u64, 0, 0]);
        for _ in scale..28 {
            magnitude = magnitude.mul(10).ok_or(DecimalError::Overflow)?;
        }
        self.count = self.count.checked_add(1).ok_or(DecimalError::Overflow)?;
        if self.negative == negative {
            self.sum = self.sum.add(magnitude).ok_or(DecimalError::Overflow)?;
        } else if self.sum >= magnitude {
            self.sum = self.sum.sub(magnitude);
        } else {
            self.sum = magnitude.sub(self.sum);
            self.negative = negative;
        }
        Ok(())
    }
    pub(crate) fn finish(&self) -> Result<PivotDecimal, DecimalError> {
        if self.count == 0 {
            return Err(DecimalError::DivisionByZero);
        }
        let mut denominator = Wide([self.count, 0, 0, 0]);
        for scale in (0..=28).rev() {
            let (mut q, r) = self.sum.div(denominator);
            if r >= denominator.sub(r) {
                q = q.add(Wide([1, 0, 0, 0])).ok_or(DecimalError::Overflow)?;
            }
            if q.0[3] == 0 && q.0[2] == 0 && q.0[1] <= u32::MAX as u64 {
                return PivotDecimal::from_parts(
                    q.0[0] as u32,
                    (q.0[0] >> 32) as u32,
                    q.0[1] as u32,
                    self.negative,
                    scale,
                )
                .map(PivotDecimal::normalize);
            }
            if scale > 0 {
                denominator = denominator.mul(10).ok_or(DecimalError::Overflow)?;
            }
        }
        Err(DecimalError::Overflow)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_extremes_midpoints_and_cancellation() {
        for (values, expected) in [
            (
                vec![PivotDecimal::MAX, PivotDecimal::MAX],
                PivotDecimal::MAX,
            ),
            (
                vec![PivotDecimal::MIN, PivotDecimal::MIN],
                PivotDecimal::MIN,
            ),
            (
                vec![PivotDecimal::MAX, PivotDecimal::MIN, 1.into()],
                "0.3333333333333333333333333333".parse().unwrap(),
            ),
            (
                vec![1.into(), 2.into(), 2.into()],
                "1.6666666666666666666666666667".parse().unwrap(),
            ),
            (
                vec![
                    PivotDecimal::ZERO,
                    "0.0000000000000000000000000001".parse().unwrap(),
                ],
                "0.0000000000000000000000000001".parse().unwrap(),
            ),
            (
                vec![
                    PivotDecimal::ZERO,
                    "-0.0000000000000000000000000001".parse().unwrap(),
                ],
                "-0.0000000000000000000000000001".parse().unwrap(),
            ),
        ] {
            for input in [values.clone(), values.into_iter().rev().collect()] {
                let mut mean = Mean::default();
                for v in input {
                    mean.push(v).unwrap();
                }
                assert_eq!(mean.finish(), Ok(expected));
            }
        }
    }
}

#[cfg(test)]
mod rational_reference {
    use super::*;
    #[test]
    fn independent_fraction_corpus() {
        for line in include_str!("../tests/fixtures/average-rational.tsv").lines() {
            let (inputs, expected) = line.split_once('\t').unwrap();
            let mut mean = Mean::default();
            for input in inputs.split(',') {
                mean.push(input.parse().unwrap()).unwrap();
            }
            assert_eq!(mean.finish().unwrap().to_string(), expected, "{inputs}");
        }
    }
    #[test]
    fn full_point_bound() {
        let mut mean = Mean::default();
        for _ in 0..4096 {
            mean.push(PivotDecimal::MAX).unwrap();
        }
        assert_eq!(mean.finish(), Ok(PivotDecimal::MAX));
    }
}
