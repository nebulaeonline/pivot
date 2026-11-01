//! Exact equal allocation. Fixed-width scratch; no floating point or group state.
use crate::{DecimalError, PivotDecimal};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationError {
    InvalidArgument,
    Overflow,
}
impl std::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidArgument => "invalid allocation argument",
            Self::Overflow => "allocation cannot reconcile exactly as Decimal96",
        })
    }
}
impl std::error::Error for AllocationError {}
#[derive(Clone, Copy, Default)]
struct Wide([u32; 6]);
#[derive(Default)]
struct Work(u16);
impl Work {
    fn visit(&mut self) {
        self.0 += 1;
    }
}
pub const MAX_LIMB_VISITS: usize = 1024;
// Five simultaneously accounted limb buffers, two results, coefficient and controls.
// Compiler frames and the existing evaluator's argument stack are separate.
pub const FIXED_SCRATCH_BYTES: usize = 5 * std::mem::size_of::<Wide>()
    + 2 * std::mem::size_of::<PivotDecimal>()
    + std::mem::size_of::<u128>()
    + 8 * std::mem::size_of::<u32>()
    + std::mem::size_of::<Work>();
const _: () = assert!(FIXED_SCRATCH_BYTES <= 256);
impl Wide {
    fn coefficient(c: u128) -> Self {
        Self([c as u32, (c >> 32) as u32, (c >> 64) as u32, 0, 0, 0])
    }
    fn zero(&self) -> bool {
        self.0 == [0; 6]
    }
    fn mul10(&mut self, w: &mut Work) -> Result<(), AllocationError> {
        let mut carry = 0u64;
        for limb in &mut self.0 {
            w.visit();
            let n = u64::from(*limb) * 10 + carry;
            *limb = n as u32;
            carry = n >> 32;
        }
        if carry != 0 {
            return Err(AllocationError::Overflow);
        }
        Ok(())
    }
    fn div(&mut self, d: u32, w: &mut Work) -> u32 {
        let mut rem = 0u64;
        for limb in self.0.iter_mut().rev() {
            w.visit();
            let n = (rem << 32) | u64::from(*limb);
            *limb = (n / u64::from(d)) as u32;
            rem = n % u64::from(d);
        }
        rem as u32
    }
    fn increment(&mut self, w: &mut Work) -> Result<(), AllocationError> {
        let mut carry = true;
        for limb in &mut self.0 {
            w.visit();
            let (n, overflow) = limb.overflowing_add(u32::from(carry));
            *limb = n;
            carry = overflow;
        }
        if carry {
            return Err(AllocationError::Overflow);
        }
        Ok(())
    }
    fn subtract(&mut self, other: Self, w: &mut Work) -> Result<(), AllocationError> {
        let mut borrow = false;
        for (limb, other) in self.0.iter_mut().zip(other.0) {
            w.visit();
            let (v, b1) = limb.overflowing_sub(other);
            let (v, b2) = v.overflowing_sub(u32::from(borrow));
            *limb = v;
            borrow = b1 || b2;
        }
        if borrow {
            return Err(AllocationError::Overflow);
        }
        Ok(())
    }
    fn fits96(&self) -> bool {
        self.0[3..] == [0; 3]
    }
    fn pack(
        mut self,
        mut scale: u32,
        negative: bool,
        w: &mut Work,
    ) -> Result<(PivotDecimal, u32), AllocationError> {
        if self.zero() {
            return Ok((PivotDecimal::ZERO, 0));
        }
        while scale > 0 {
            let mut next = self;
            if next.div(10, w) != 0 {
                break;
            }
            self = next;
            scale -= 1;
        }
        if !self.fits96() {
            return Err(AllocationError::Overflow);
        }
        let value = PivotDecimal::from_parts(self.0[0], self.0[1], self.0[2], negative, scale)
            .map_err(|_: DecimalError| AllocationError::Overflow)?;
        Ok((value, scale))
    }
}
fn allocate(
    total: PivotDecimal,
    count: u32,
    index: u32,
    places: u32,
) -> Result<(PivotDecimal, Work), AllocationError> {
    if count == 0 || index == 0 || index > count || places > 28 {
        return Err(AllocationError::InvalidArgument);
    }
    let (lo, mid, hi, negative, scale) = total.parts();
    let mut c = u128::from(lo) | (u128::from(mid) << 32) | (u128::from(hi) << 64);
    let mut work = Work::default();
    if scale > places {
        let divisor = 10u128.pow(scale - places);
        if !c.is_multiple_of(divisor) {
            return Err(AllocationError::InvalidArgument);
        }
        c /= divisor;
    }
    let mut units = Wide::coefficient(c);
    for _ in scale..places {
        units.mul10(&mut work)?;
    }
    let mut quotient = units;
    let remainder = quotient.div(count, &mut work);
    // Validate both actually used shares, independent of the requested recipient.
    let (low, low_scale) = quotient.pack(places, negative, &mut work)?;
    let (high, high_scale) = if remainder > 0 {
        let mut high = quotient;
        high.increment(&mut work)?;
        high.pack(places, negative, &mut work)?
    } else {
        (low, low_scale)
    };
    let common = low_scale.max(high_scale);
    let mut proper = units;
    proper.subtract(quotient, &mut work)?;
    for _ in common..places {
        if proper.div(10, &mut work) != 0 {
            return Err(AllocationError::Overflow);
        }
    }
    if !proper.fits96() {
        return Err(AllocationError::Overflow);
    }
    debug_assert!(usize::from(work.0) <= MAX_LIMB_VISITS);
    Ok((if index <= remainder { high } else { low }, work))
}
impl PivotDecimal {
    /// Return one settled equal share. Every used share and proper partial sum must
    /// be exactly representable; the complete consistent group conserves this total.
    pub fn allocate_equal(
        self,
        count: u32,
        index: u32,
        places: u32,
    ) -> Result<Self, AllocationError> {
        allocate(self, count, index, places).map(|(v, _)| v)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_work_and_width_boundaries() {
        let mut high = Wide::coefficient((1u128 << 96) - 1);
        let mut w = Work::default();
        for _ in 0..28 {
            high.mul10(&mut w).unwrap();
        }
        assert!(!high.zero());
        assert_eq!(w.0, 168);
        for total in [
            PivotDecimal::MAX,
            PivotDecimal::MIN,
            PivotDecimal::ZERO,
            "10".parse().unwrap(),
            "0.0000000000000000000000000001".parse().unwrap(),
        ] {
            for n in [1, 2, 3, 10, 11, u32::MAX] {
                for p in 0..=28 {
                    if let Ok((_, work)) = allocate(total, n, 1, p) {
                        assert!(usize::from(work.0) <= MAX_LIMB_VISITS);
                    }
                }
            }
        }
    }
    #[test]
    fn full_width_division_matches_independent_bitwise_reference() {
        for input in [
            Wide([u32::MAX; 6]),
            Wide([1, 0, 0, 0, 0, 1 << 31]),
            Wide([0, 1, u32::MAX, 123, 0, u32::MAX]),
        ] {
            for d in [1, 2, 3, 5, 10, 11, u32::MAX] {
                let mut expected = [0u32; 6];
                let mut remainder = 0u64;
                for bit in (0..192).rev() {
                    remainder = remainder * 2 + u64::from((input.0[bit / 32] >> (bit % 32)) & 1);
                    if remainder >= u64::from(d) {
                        remainder -= u64::from(d);
                        expected[bit / 32] |= 1 << (bit % 32);
                    }
                }
                let mut actual = input;
                let mut work = Work::default();
                let r = actual.div(d, &mut work);
                assert_eq!(actual.0, expected);
                assert_eq!(u64::from(r), remainder);
                assert_eq!(work.0, 6);
            }
        }
    }
    #[test]
    fn limb_division_carry_and_subtraction() {
        for d in [1, 2, 10, u32::MAX] {
            let original = Wide::coefficient((1u128 << 96) - 1);
            let mut q = original;
            let mut w = Work::default();
            let r = q.div(d, &mut w);
            let c = (1u128 << 96) - 1;
            let expected = c / u128::from(d);
            assert_eq!(q.0, Wide::coefficient(expected).0);
            assert_eq!(u128::from(r), c % u128::from(d));
        }
        let mut w = Work::default();
        let mut v = Wide([u32::MAX; 6]);
        assert_eq!(v.increment(&mut w), Err(AllocationError::Overflow));
        let mut v = Wide::default();
        assert!(v.subtract(Wide::coefficient(1), &mut w).is_err());
    }
}
