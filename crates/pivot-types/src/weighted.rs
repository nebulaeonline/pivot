//! Exact bounded weighted settlement. Arithmetic buffers never depend on worksheet size.
use crate::PivotDecimal;
use std::cmp::Ordering;

pub const MAX_WEIGHTS: usize = 64;
pub const SCRATCH_BYTES: usize = 16 * 1024;
pub const fn limb_limit(n: usize) -> usize {
    4096 + 16384 * n
}
pub const fn visit_charge(n: usize) -> usize {
    4 + 16 * n
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    Overflow,
    Cancelled,
    WorkBudget,
}
#[derive(Clone, Copy)]
struct Int<const N: usize>([u32; N]);
impl<const N: usize> Default for Int<N> {
    fn default() -> Self {
        Self([0; N])
    }
}
struct Work<'a> {
    used: usize,
    limit: usize,
    cancelled: &'a dyn Fn() -> bool,
}
impl Work<'_> {
    fn tick(&mut self, n: usize) -> Result<(), Error> {
        let before = self.used;
        self.used = self.used.checked_add(n).ok_or(Error::WorkBudget)?;
        if self.used > self.limit {
            return Err(Error::WorkBudget);
        }
        if before / 128 != self.used / 128 && (self.cancelled)() {
            return Err(Error::Cancelled);
        }
        Ok(())
    }
}
impl<const N: usize> Int<N> {
    fn coefficient(c: u128) -> Self {
        let mut result = Self::default();
        for (k, limb) in result.0.iter_mut().take(3).enumerate() {
            *limb = (c >> (k * 32)) as u32;
        }
        result
    }
    fn zero(&self, w: &mut Work<'_>) -> Result<bool, Error> {
        w.tick(N)?;
        Ok(self.0.iter().all(|&v| v == 0))
    }
    fn compare(&self, other: &Self, w: &mut Work<'_>) -> Result<Ordering, Error> {
        w.tick(N)?;
        Ok(self.0.iter().rev().cmp(other.0.iter().rev()))
    }
    fn add(&mut self, other: Self, w: &mut Work<'_>) -> Result<(), Error> {
        w.tick(N)?;
        let mut carry = 0u64;
        for (a, b) in self.0.iter_mut().zip(other.0) {
            let sum = u64::from(*a) + u64::from(b) + carry;
            *a = sum as u32;
            carry = sum >> 32;
        }
        if carry != 0 {
            return Err(Error::Overflow);
        }
        Ok(())
    }
    fn subtract(&mut self, other: Self, w: &mut Work<'_>) -> Result<(), Error> {
        w.tick(N)?;
        let mut borrow = false;
        for (a, b) in self.0.iter_mut().zip(other.0) {
            let (v, b1) = a.overflowing_sub(b);
            let (v, b2) = v.overflowing_sub(u32::from(borrow));
            *a = v;
            borrow = b1 || b2;
        }
        if borrow {
            return Err(Error::Overflow);
        }
        Ok(())
    }
    fn mul10(&mut self, w: &mut Work<'_>) -> Result<(), Error> {
        w.tick(N)?;
        let mut carry = 0u64;
        for a in &mut self.0 {
            let v = u64::from(*a) * 10 + carry;
            *a = v as u32;
            carry = v >> 32;
        }
        if carry != 0 {
            return Err(Error::Overflow);
        }
        Ok(())
    }
    fn div10(&mut self, w: &mut Work<'_>) -> Result<u32, Error> {
        w.tick(N)?;
        let mut rem = 0u64;
        for a in self.0.iter_mut().rev() {
            let v = (rem << 32) | u64::from(*a);
            *a = (v / 10) as u32;
            rem = v % 10;
        }
        Ok(rem as u32)
    }
    fn pack(
        mut self,
        mut scale: u32,
        negative: bool,
        w: &mut Work<'_>,
    ) -> Result<(PivotDecimal, u32), Error> {
        if self.zero(w)? {
            return Ok((PivotDecimal::ZERO, 0));
        }
        while scale > 0 {
            let mut next = self;
            if next.div10(w)? != 0 {
                break;
            }
            self = next;
            scale -= 1;
        }
        w.tick(N)?;
        if self.0[3..].iter().any(|&v| v != 0) {
            return Err(Error::Overflow);
        }
        Ok((
            PivotDecimal::from_parts(self.0[0], self.0[1], self.0[2], negative, scale)
                .map_err(|_| Error::Overflow)?,
            scale,
        ))
    }
}
fn parts(value: PivotDecimal) -> (u128, bool, u32) {
    let (lo, mid, hi, negative, scale) = value.parts();
    (
        u128::from(lo) | (u128::from(mid) << 32) | (u128::from(hi) << 64),
        negative,
        scale,
    )
}
fn product(a: Int<6>, b: Int<6>, w: &mut Work<'_>) -> Result<Int<12>, Error> {
    let mut out = Int::<12>::default();
    for i in 0..6 {
        let mut carry = 0u64;
        for j in 0..6 {
            w.tick(1)?;
            let v = u64::from(a.0[i]) * u64::from(b.0[j]) + u64::from(out.0[i + j]) + carry;
            out.0[i + j] = v as u32;
            carry = v >> 32;
        }
        w.tick(1)?;
        out.0[i + 6] = carry as u32;
    }
    Ok(out)
}
fn divide(
    value: Int<12>,
    denominator: Int<7>,
    w: &mut Work<'_>,
) -> Result<(Int<6>, Int<7>), Error> {
    let mut quotient = Int::<6>::default();
    let mut remainder = Int::<7>::default();
    for bit in (0..384).rev() {
        w.tick(8)?;
        let mut carry = (value.0[bit / 32] >> (bit % 32)) & 1;
        for a in &mut remainder.0 {
            let next = *a >> 31;
            *a = (*a << 1) | carry;
            carry = next;
        }
        if carry != 0 {
            return Err(Error::Overflow);
        }
        if remainder.compare(&denominator, w)? != Ordering::Less {
            remainder.subtract(denominator, w)?;
            w.tick(1)?;
            if bit >= 192 {
                return Err(Error::Overflow);
            }
            quotient.0[bit / 32] |= 1 << (bit % 32);
        }
    }
    Ok((quotient, remainder))
}
/// Caller-owned fixed storage. Canonical callers reserve SCRATCH_BYTES before construction.
pub struct Scratch {
    weights: [Int<6>; MAX_WEIGHTS],
    quotients: [Int<6>; MAX_WEIGHTS],
    remainders: [Int<7>; MAX_WEIGHTS],
    values: [PivotDecimal; MAX_WEIGHTS],
    order: [usize; MAX_WEIGHTS],
}
impl Default for Scratch {
    fn default() -> Self {
        Self {
            weights: [Int::default(); MAX_WEIGHTS],
            quotients: [Int::default(); MAX_WEIGHTS],
            remainders: [Int::default(); MAX_WEIGHTS],
            values: [PivotDecimal::ZERO; MAX_WEIGHTS],
            order: [0; MAX_WEIGHTS],
        }
    }
}
// Includes the caller's at-most-64 Decimal input copies and conservative fixed local
// integer/control buffers. Compiler frames and pre-existing argument stacks are separate.
pub const EXPLICIT_BYTES: usize =
    std::mem::size_of::<Scratch>() + MAX_WEIGHTS * std::mem::size_of::<PivotDecimal>() + 2048;
const _: () = assert!(EXPLICIT_BYTES <= SCRATCH_BYTES);
impl Scratch {
    fn rank(
        &mut self,
        n: usize,
        w: &mut Work<'_>,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(), Error> {
        for j in 1..n {
            if cancelled() {
                return Err(Error::Cancelled);
            }
            let current = self.order[j];
            let mut k = j;
            while k > 0 {
                let prior = self.order[k - 1];
                let order = self.remainders[current].compare(&self.remainders[prior], w)?;
                if order == Ordering::Less || (order == Ordering::Equal && current > prior) {
                    break;
                }
                self.order[k] = prior;
                k -= 1;
            }
            self.order[k] = current;
        }
        Ok(())
    }
    /// Pure numeric boundary; execution/resource admission belongs to the caller.
    /// Every arithmetic loop is bounded and checks the same limb ceiling/cancellation.
    pub fn allocate(
        &mut self,
        total: PivotDecimal,
        index: u32,
        places: u32,
        weights: &[PivotDecimal],
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(PivotDecimal, usize), Error> {
        let n = weights.len();
        if !(1..=MAX_WEIGHTS).contains(&n) || index == 0 || index as usize > n || places > 28 {
            return Err(Error::InvalidArgument);
        }
        if cancelled() {
            return Err(Error::Cancelled);
        }
        let mut w = Work {
            used: 0,
            limit: limb_limit(n),
            cancelled,
        };
        let mut common_weight_scale = 0;
        for value in weights {
            let (c, negative, scale) = parts(*value);
            if negative && c != 0 {
                return Err(Error::InvalidArgument);
            }
            common_weight_scale = common_weight_scale.max(scale);
        }
        let mut denominator = Int::<7>::default();
        for (j, value) in weights.iter().enumerate() {
            if cancelled() {
                return Err(Error::Cancelled);
            }
            let (c, _, scale) = parts(*value);
            let mut aligned = Int::<6>::coefficient(c);
            for _ in scale..common_weight_scale {
                aligned.mul10(&mut w)?;
            }
            self.weights[j] = aligned;
            let mut extended = Int::<7>::default();
            extended.0[..6].copy_from_slice(&aligned.0);
            w.tick(6)?;
            denominator.add(extended, &mut w)?;
            self.order[j] = j;
        }
        if denominator.zero(&mut w)? {
            return Err(Error::InvalidArgument);
        }
        let (mut c, negative, scale) = parts(total);
        if scale > places {
            let d = 10u128.pow(scale - places);
            if !c.is_multiple_of(d) {
                return Err(Error::InvalidArgument);
            }
            c /= d;
        }
        let mut units = Int::<6>::coefficient(c);
        for _ in scale..places {
            units.mul10(&mut w)?;
        }
        let mut floor_sum = Int::<6>::default();
        for j in 0..n {
            if cancelled() {
                return Err(Error::Cancelled);
            }
            let (q, r) = divide(
                product(units, self.weights[j], &mut w)?,
                denominator,
                &mut w,
            )?;
            self.quotients[j] = q;
            self.remainders[j] = r;
            floor_sum.add(q, &mut w)?;
        }
        let mut residual = units;
        residual.subtract(floor_sum, &mut w)?;
        w.tick(6)?;
        if residual.0[1..].iter().any(|&v| v != 0) || residual.0[0] as usize >= n {
            return Err(Error::Overflow);
        }
        self.rank(n, &mut w, cancelled)?;
        for &j in &self.order[..residual.0[0] as usize] {
            if self.weights[j].zero(&mut w)? {
                return Err(Error::Overflow);
            }
            self.quotients[j].add(Int::coefficient(1), &mut w)?;
        }
        let mut common = 0;
        let mut minimum: Option<Int<6>> = None;
        for j in 0..n {
            if cancelled() {
                return Err(Error::Cancelled);
            }
            let (value, scale) = self.quotients[j].pack(places, negative, &mut w)?;
            self.values[j] = value;
            common = common.max(scale);
            if !self.quotients[j].zero(&mut w)?
                && match minimum {
                    None => true,
                    Some(m) => self.quotients[j].compare(&m, &mut w)? == Ordering::Less,
                }
            {
                minimum = Some(self.quotients[j]);
            }
        }
        if let Some(minimum) = minimum {
            let mut proper = units;
            proper.subtract(minimum, &mut w)?;
            for _ in common..places {
                if proper.div10(&mut w)? != 0 {
                    return Err(Error::Overflow);
                }
            }
            w.tick(6)?;
            if proper.0[3..].iter().any(|&v| v != 0) {
                return Err(Error::Overflow);
            }
        }
        Ok((self.values[index as usize - 1], w.used))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_weights_match_equal_allocation_and_work_is_bounded() {
        let mut scratch = Scratch::default();
        for n in [1, 3, 10, 64] {
            let weights = vec![PivotDecimal::from(1); n];
            for total in [
                PivotDecimal::ZERO,
                PivotDecimal::MAX,
                "10".parse().unwrap(),
                "0.0000000000000000000000000001".parse().unwrap(),
            ] {
                for p in [0, 2, 28] {
                    for i in [1, n as u32] {
                        let actual = scratch.allocate(total, i, p, &weights, &|| false);
                        let expected = total.allocate_equal(n as u32, i, p);
                        match (actual, expected) {
                            (Ok((a, work)), Ok(b)) => {
                                assert_eq!(a, b);
                                assert!(work <= limb_limit(n));
                            }
                            (Err(Error::Overflow), Err(crate::AllocationError::Overflow))
                            | (
                                Err(Error::InvalidArgument),
                                Err(crate::AllocationError::InvalidArgument),
                            ) => {}
                            pair => panic!("{pair:?}"),
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn ranking_is_cancellable_and_charged() {
        use std::cell::Cell;
        let mut scratch = Scratch::default();
        for j in 0..64 {
            scratch.order[j] = j;
            scratch.remainders[j] = Int::coefficient(j as u128);
        }
        let calls = Cell::new(0);
        let stop = || {
            calls.set(calls.get() + 1);
            calls.get() > 10
        };
        let mut work = Work {
            used: 0,
            limit: limb_limit(64),
            cancelled: &stop,
        };
        assert_eq!(scratch.rank(64, &mut work, &stop), Err(Error::Cancelled));
        assert!(work.used > 0 && work.used < limb_limit(64));
    }
    #[test]
    fn fixed_width_multiply_divide_and_inner_cancellation() {
        let mut w = Work {
            used: 0,
            limit: limb_limit(1),
            cancelled: &|| false,
        };
        for a in [1, u64::MAX as u128, (1u128 << 96) - 1] {
            for b in [1, 10, u32::MAX as u128] {
                w.used = 0;
                let product = product(Int::coefficient(a), Int::coefficient(b), &mut w).unwrap();
                let (q, r) = divide(product, Int::coefficient(b), &mut w).unwrap();
                assert_eq!(q.0, Int::<6>::coefficient(a).0);
                assert!(r.zero(&mut w).unwrap());
            }
        }
        let mut stopped = Work {
            used: 0,
            limit: limb_limit(1),
            cancelled: &|| true,
        };
        assert!(matches!(
            divide(Int::coefficient(1), Int::coefficient(3), &mut stopped),
            Err(Error::Cancelled)
        ));
        let mut exhausted = Work {
            used: 0,
            limit: 0,
            cancelled: &|| false,
        };
        assert!(matches!(
            product(Int::coefficient(1), Int::coefficient(1), &mut exhausted),
            Err(Error::WorkBudget)
        ));
    }
}
