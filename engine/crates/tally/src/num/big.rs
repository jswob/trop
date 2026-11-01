//! The big tier of the exact numbers: an integer of any size, and a reduced fraction of two.
//! Spec: L §7.1 (exact numbers, the 4 096-digit cap); P §6 (the two tiers).
//! Twin: engine/tools/refcore/values.py (Python's int and Fraction).
//! Invariants: the one file that names the big-integer type (R-29, R-108); a `BigRat` is
//! reduced, with a positive denominator; nothing here is capped, the caller checks the cap.
//!
//! Every operation goes through a checked call of the big-integer library, so no operator
//! can wrap or crash (R-19, R-20). Addition, subtraction and multiplication of big integers
//! can't fail; their `None` maps to `NumError::Invariant`, which no input reaches.

use super::NumError;
use super::gcd;
use core::cmp::Ordering;
use num_traits::{CheckedEuclid, Signed, ToPrimitive, Zero};

/// An integer of any size: the big tier's one wrapper around the library type.
#[expect(
    clippy::disallowed_types,
    reason = "R-29: the number kernel's one wrapper around the big-integer type"
)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Big(num_bigint::BigInt);

// Written out, not derived: a derived `Eq` names the field's type in a generated assertion,
// outside the struct's one expected exception (R-29).
impl Eq for Big {}

impl Big {
    /// The numeric order of two integers, a named call rather than a derived `Ord` (R-118).
    /// L §7.1.
    pub(crate) fn order(&self, o: &Big) -> Ordering {
        self.0.cmp(&o.0)
    }

    /// The integer `v`. L §7.1.
    pub(crate) fn from_i128(v: i128) -> Big {
        Big(v.into())
    }

    /// The integer `v`. L §7.1.
    pub(crate) fn from_u128(v: u128) -> Big {
        Big(v.into())
    }

    /// 0. L §7.1.
    pub(crate) fn zero() -> Big {
        Big(Zero::zero())
    }

    /// The integer whose decimal digits are `digits` (ASCII digits only), or `None`. L §7.1.
    pub(crate) fn from_decimal_digits(digits: &str) -> Option<Big> {
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        match num_traits::Num::from_str_radix(digits, 10) {
            Ok(v) => Some(Big(v)),
            Err(_) => None,
        }
    }

    /// The value as an `i128`, when it fits. L §7.1.
    pub(crate) fn to_i128(&self) -> Option<i128> {
        self.0.to_i128()
    }

    /// The value as a `u64`, when it fits. L §7.1.
    pub(crate) fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }

    /// Whether the value is 0. L §7.1.
    pub(crate) fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Whether the value is below 0. L §7.1.
    pub(crate) fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// The absolute value. L §7.1.
    pub(crate) fn abs(&self) -> Big {
        Big(self.0.abs())
    }

    /// The number of bits of the absolute value (0 for 0): what the cap screens by. L §7.1.
    pub(crate) fn bits(&self) -> u64 {
        self.0.bits()
    }

    /// The absolute value's 64-bit limbs, least significant first. P §6 (Lehmer's gcd).
    pub(crate) fn limbs(&self) -> Vec<u64> {
        self.0.to_u64_digits().1
    }

    /// `self + o`. L §7.1.
    pub(crate) fn add(&self, o: &Big) -> Result<Big, NumError> {
        self.0.checked_add(&o.0).map(Big).ok_or(NumError::Invariant)
    }

    /// `self - o`. L §7.1.
    pub(crate) fn sub(&self, o: &Big) -> Result<Big, NumError> {
        self.0.checked_sub(&o.0).map(Big).ok_or(NumError::Invariant)
    }

    /// `self × o`. L §7.1.
    pub(crate) fn mul(&self, o: &Big) -> Result<Big, NumError> {
        self.0.checked_mul(&o.0).map(Big).ok_or(NumError::Invariant)
    }

    /// `-self`. L §7.1.
    pub(crate) fn neg(&self) -> Result<Big, NumError> {
        Big::zero().sub(self)
    }

    /// `self ÷ o` truncated, for an `o` known to divide `self` (a gcd); `RUN-DIV` on 0. L §7.1.
    pub(crate) fn div_exact(&self, o: &Big) -> Result<Big, NumError> {
        self.0.checked_div(&o.0).map(Big).ok_or(NumError::Div)
    }

    /// The floor of `self ÷ o` for a positive `o` (Python's `//`); `RUN-DIV` on 0. L §7.2.
    pub(crate) fn div_floor(&self, o: &Big) -> Result<Big, NumError> {
        self.0
            .checked_div_euclid(&o.0)
            .map(Big)
            .ok_or(NumError::Div)
    }

    /// `self mod o` in `0 .. o` for a positive `o` (Python's `%`); `RUN-DIV` on 0. L §7.2.
    pub(crate) fn rem_floor(&self, o: &Big) -> Result<Big, NumError> {
        self.0
            .checked_rem_euclid(&o.0)
            .map(Big)
            .ok_or(NumError::Div)
    }

    /// `base` to the power `exp`, by squaring; uncapped, for the cap's own constant and for
    /// printing (R-26: never the library's `pow`). L §7.1.
    pub(crate) fn pow(base: u64, exp: u32) -> Result<Big, NumError> {
        let mut result = Big::from_u128(1);
        let mut square = Big::from_u128(u128::from(base));
        let mut rest = exp;
        while rest > 0 {
            if rest & 1 == 1 {
                result = result.mul(&square)?;
            }
            rest = rest.checked_shr(1).ok_or(NumError::Invariant)?;
            if rest > 0 {
                square = square.mul(&square)?;
            }
        }
        Ok(result)
    }

    /// The decimal digits of the absolute value. L §5.18 (output).
    pub(crate) fn abs_decimal(&self) -> String {
        self.0.abs().to_str_radix(10)
    }
}

/// A reduced fraction of big integers with a positive denominator: the big tier of `Num`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BigRat {
    /// The numerator, its sign the value's.
    n: Big,
    /// The denominator, positive.
    d: Big,
}

impl BigRat {
    /// The fraction `n / d` reduced, with the sign on the numerator; `RUN-DIV` when `d` is 0.
    /// L §7.1.
    pub(crate) fn new(n: &Big, d: &Big) -> Result<BigRat, NumError> {
        if d.is_zero() {
            return Err(NumError::Div);
        }
        let g = gcd::gcd_big(n, d)?;
        let (mut n, mut d) = (n.div_exact(&g)?, d.div_exact(&g)?);
        if d.is_negative() {
            n = n.neg()?;
            d = d.neg()?;
        }
        Ok(BigRat { n, d })
    }

    /// The integer `n`. L §7.1.
    pub(crate) fn integer(n: Big) -> BigRat {
        BigRat {
            n,
            d: Big::from_u128(1),
        }
    }

    /// A fraction already reduced, with `d > 0`, as the small tier's values are. L §7.1.
    pub(crate) fn from_reduced_i128(n: i128, d: i128) -> BigRat {
        BigRat {
            n: Big::from_i128(n),
            d: Big::from_i128(d),
        }
    }

    /// The numerator. L §7.1.
    pub(crate) fn numer(&self) -> &Big {
        &self.n
    }

    /// The denominator, positive. L §7.1.
    pub(crate) fn denom(&self) -> &Big {
        &self.d
    }

    /// `self + o`, with one gcd of the denominators (Knuth, TAOCP 4.5.1). L §7.1.
    pub(crate) fn add(&self, o: &BigRat) -> Result<BigRat, NumError> {
        let g = gcd::gcd_big(&self.d, &o.d)?;
        let self_d_g = self.d.div_exact(&g)?;
        let o_d_g = o.d.div_exact(&g)?;
        let t = self.n.mul(&o_d_g)?.add(&o.n.mul(&self_d_g)?)?;
        let g2 = gcd::gcd_big(&t, &g)?;
        Ok(BigRat {
            n: t.div_exact(&g2)?,
            d: self_d_g.mul(&o.d.div_exact(&g2)?)?,
        })
    }

    /// `-self`. L §7.1.
    pub(crate) fn neg(&self) -> Result<BigRat, NumError> {
        Ok(BigRat {
            n: self.n.neg()?,
            d: self.d.clone(),
        })
    }

    /// `self × o`, cross-reduced (Knuth, TAOCP 4.5.1). L §7.1.
    pub(crate) fn mul(&self, o: &BigRat) -> Result<BigRat, NumError> {
        let g1 = gcd::gcd_big(&self.n, &o.d)?;
        let g2 = gcd::gcd_big(&o.n, &self.d)?;
        Ok(BigRat {
            n: self.n.div_exact(&g1)?.mul(&o.n.div_exact(&g2)?)?,
            d: self.d.div_exact(&g2)?.mul(&o.d.div_exact(&g1)?)?,
        })
    }

    /// `1 / self`; `RUN-DIV` at 0. L §7.1.
    pub(crate) fn recip(&self) -> Result<BigRat, NumError> {
        if self.n.is_zero() {
            return Err(NumError::Div);
        }
        if self.n.is_negative() {
            return Ok(BigRat {
                n: self.d.neg()?,
                d: self.n.neg()?,
            });
        }
        Ok(BigRat {
            n: self.d.clone(),
            d: self.n.clone(),
        })
    }

    /// The order of two values, by cross-multiplying. L §7.1.
    pub(crate) fn compare(&self, o: &BigRat) -> Result<Ordering, NumError> {
        Ok(self.n.mul(&o.d)?.order(&o.n.mul(&self.d)?))
    }
}

#[cfg(test)]
mod tests {
    use super::{Big, BigRat};
    use core::cmp::Ordering;

    fn rat(n: i128, d: i128) -> BigRat {
        BigRat::new(&Big::from_i128(n), &Big::from_i128(d)).expect("a fraction")
    }

    #[test]
    fn a_fraction_is_reduced_with_its_sign_on_the_numerator() {
        // L §7.1: values are exact rationals; 6 / -4 is -3/2
        let r = rat(6, -4);
        assert_eq!(r.numer(), &Big::from_i128(-3), "numerator");
        assert_eq!(r.denom(), &Big::from_i128(2), "denominator");
    }

    #[test]
    fn division_by_zero_is_refused() {
        // L §7.1: RUN-DIV
        assert!(
            BigRat::new(&Big::from_i128(1), &Big::zero()).is_err(),
            "1/0 refused"
        );
        assert!(BigRat::integer(Big::zero()).recip().is_err(), "1/0 refused");
    }

    #[test]
    fn floor_division_follows_python() {
        // BUILDER §2: Python floors where Rust truncates; -7 // 2 == -4, -7 % 2 == 1
        let a = Big::from_i128(-7);
        let b = Big::from_i128(2);
        assert_eq!(a.div_floor(&b).expect("div"), Big::from_i128(-4), "floor");
        assert_eq!(a.rem_floor(&b).expect("rem"), Big::from_i128(1), "mod");
    }

    #[test]
    fn powers_are_exact() {
        assert_eq!(
            Big::pow(10, 20).expect("10^20"),
            Big::from_u128(100_000_000_000_000_000_000),
            "10^20"
        );
        assert_eq!(Big::pow(7, 0).expect("7^0"), Big::from_u128(1), "x^0");
    }

    #[test]
    fn decimal_digits_round_trip() {
        let digits = "123456789012345678901234567890123456789";
        let v = Big::from_decimal_digits(digits).expect("digits");
        assert_eq!(v.abs_decimal(), digits, "round trip");
        assert!(Big::from_decimal_digits("12a").is_none(), "not digits");
        assert!(Big::from_decimal_digits("").is_none(), "empty");
        assert!(Big::from_decimal_digits("-1").is_none(), "no sign");
    }

    #[test]
    fn fractions_add_multiply_and_compare() {
        // 1/6 + 1/10 = 4/15; 2/3 * 9/4 = 3/2; -1/2 < 1/3
        assert_eq!(rat(1, 6).add(&rat(1, 10)).expect("sum"), rat(4, 15), "sum");
        assert_eq!(
            rat(2, 3).mul(&rat(9, 4)).expect("product"),
            rat(3, 2),
            "product"
        );
        assert_eq!(
            rat(-1, 2).compare(&rat(1, 3)).expect("cmp"),
            Ordering::Less,
            "-1/2 < 1/3"
        );
        assert_eq!(
            rat(-1, 2).recip().expect("recip"),
            BigRat::integer(Big::from_i128(-2)),
            "1/(-1/2)"
        );
    }
}
