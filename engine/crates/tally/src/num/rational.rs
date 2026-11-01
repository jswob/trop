//! The exact number `Num`: a reduced fraction, held small or big, never a float.
//! Spec: L §7.1 (exact numbers, the cap); P §6 (the tiers).
//! Twin: engine/tools/refcore/values.py (`Num` over Python's `Fraction`).
//! Invariants: a `Num` is reduced with a positive denominator; it is small whenever both
//! parts fit (demoted after every big result); a big result past 4 096 digits is refused.
//!
//! `Num` has named checked methods and no operator traits, `Default`, `Sum` or `From`
//! (R-20, R-28). Comparison is `compare`, which returns a `Result` like every other
//! operation, so `Num` has no `Ord`.

use super::big::{Big, BigRat};
use super::small::{self, Small, Wide};
use core::cmp::Ordering;

/// Why a number operation gives no value; each maps to a code of L §8.4 at its key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumError {
    /// Division by zero (`RUN-DIV`).
    Div,
    /// A numerator or denominator past 4 096 decimal digits (`RUN-DIGITS`).
    Digits,
    /// An argument outside the operation's domain, such as a rounding step that isn't
    /// positive, or text that isn't a decimal literal (`RUN-DOMAIN`).
    Domain,
    /// A broken invariant of the number kernel (`ENGINE-INVARIANT`).
    Invariant,
}

/// An exact rational number (L §7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num(pub(super) Repr);

/// The two tiers (D2-1): no middle one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Repr {
    /// Two 64-bit integers, reduced.
    Small(Small),
    /// Two big integers, reduced, at least one too large for the small tier.
    Big(Box<BigRat>),
}

/// The largest bit length a part may have and pass the cap without a comparison:
/// 10^4096 has 13 607 bits, so 13 606 bits are always fewer than 4 097 digits. L §7.1.
const CAP_SURE_BITS: u64 = 13_606;
/// The exponent of the cap, 4 096 decimal digits. L §7.1.
const CAP_DIGITS: u32 = 4_096;

impl Num {
    /// The whole number `n`. L §7.1.
    pub fn int(n: i64) -> Num {
        match Small::whole(n) {
            Some(s) => Num(Repr::Small(s)),
            None => Num(Repr::Big(Box::new(BigRat::integer(Big::from_i128(
                i128::from(n),
            ))))),
        }
    }

    /// The fraction `n / d`, reduced; `RUN-DIV` when `d` is 0. L §7.1.
    pub fn ratio(n: i64, d: i64) -> Result<Num, NumError> {
        let (n, d) = if d < 0 {
            (i128::from(n).checked_neg(), i128::from(d).checked_neg())
        } else {
            (Some(i128::from(n)), Some(i128::from(d)))
        };
        let (Some(n), Some(d)) = (n, d) else {
            return Err(NumError::Invariant);
        };
        Ok(Num::from_wide(small::reduce(n, d)?))
    }

    /// 0. L §7.1.
    pub fn zero() -> Num {
        Num::int(0)
    }

    /// Whether the value is 0. L §7.1.
    pub fn is_zero(&self) -> bool {
        match &self.0 {
            Repr::Small(s) => s.n == 0,
            Repr::Big(b) => b.numer().is_zero(),
        }
    }

    /// Whether the value is below 0. L §7.1.
    pub fn is_negative(&self) -> bool {
        match &self.0 {
            Repr::Small(s) => s.n < 0,
            Repr::Big(b) => b.numer().is_negative(),
        }
    }

    /// Whether the value is a whole number. L §7.1.
    pub fn is_integer(&self) -> bool {
        match &self.0 {
            Repr::Small(s) => s.is_integer(),
            Repr::Big(b) => b.denom() == &Big::from_u128(1),
        }
    }

    /// The value as an `i64`, when it is a whole number that fits. L §7.1.
    pub fn to_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        match &self.0 {
            Repr::Small(s) => Some(s.n),
            Repr::Big(b) => b.numer().to_i128().and_then(|v| i64::try_from(v).ok()),
        }
    }

    /// `self + o`. L §7.1.
    pub fn checked_add(&self, o: &Num) -> Result<Num, NumError> {
        if let Some((a, b)) = self.small_pair(o) {
            return Ok(Num::from_wide(small::add(a, b)?));
        }
        Num::from_big(self.to_big().add(&o.to_big())?)
    }

    /// `self - o`. L §7.1.
    pub fn checked_sub(&self, o: &Num) -> Result<Num, NumError> {
        self.checked_add(&o.checked_neg()?)
    }

    /// `-self`. L §7.1.
    pub fn checked_neg(&self) -> Result<Num, NumError> {
        match &self.0 {
            Repr::Small(s) => Ok(Num(Repr::Small(small::neg(*s)?))),
            Repr::Big(b) => Num::from_big(b.neg()?),
        }
    }

    /// `self × o`. L §7.1.
    pub fn checked_mul(&self, o: &Num) -> Result<Num, NumError> {
        if let Some((a, b)) = self.small_pair(o) {
            return Ok(Num::from_wide(small::mul_small(a, b)?));
        }
        Num::from_big(self.to_big().mul(&o.to_big())?)
    }

    /// `self ÷ o`; `RUN-DIV` when `o` is 0. L §7.1.
    pub fn checked_div(&self, o: &Num) -> Result<Num, NumError> {
        if let Some((a, b)) = self.small_pair(o) {
            return Ok(Num::from_wide(small::mul_small(a, small::recip(b)?)?));
        }
        Num::from_big(self.to_big().mul(&o.to_big().recip()?)?)
    }

    /// The order of two values. L §7.1.
    pub fn compare(&self, o: &Num) -> Result<Ordering, NumError> {
        if let Some((a, b)) = self.small_pair(o) {
            return small::compare(a, b);
        }
        self.to_big().compare(&o.to_big())
    }

    /// Both values, when both are small. P §6 (the small tier first).
    fn small_pair(&self, o: &Num) -> Option<(Small, Small)> {
        match (&self.0, &o.0) {
            (Repr::Small(a), Repr::Small(b)) => Some((*a, *b)),
            (Repr::Small(_), Repr::Big(_)) | (Repr::Big(_), Repr::Small(_) | Repr::Big(_)) => None,
        }
    }

    /// The value in the big tier's form. P §6.
    pub(super) fn to_big(&self) -> BigRat {
        match &self.0 {
            Repr::Small(s) => BigRat::from_reduced_i128(i128::from(s.n), i128::from(s.d.get())),
            Repr::Big(b) => BigRat::clone(b),
        }
    }

    /// A small tier's result: small, or a 128-bit pair held big (under the cap by its size).
    /// L §7.1.
    fn from_wide(w: Wide) -> Num {
        match w {
            Wide::Fits(s) => Num(Repr::Small(s)),
            Wide::Pair(n, d) => Num(Repr::Big(Box::new(BigRat::from_reduced_i128(n, d)))),
        }
    }

    /// A big tier's result: refused past the cap, demoted to small when it fits (D2-1).
    /// L §7.1.
    pub(super) fn from_big(r: BigRat) -> Result<Num, NumError> {
        check_cap(r.numer())?;
        check_cap(r.denom())?;
        if let (Some(n), Some(d)) = (r.numer().to_i128(), r.denom().to_i128()) {
            return Ok(Num::from_wide(small::reduce(n, d)?));
        }
        Ok(Num(Repr::Big(Box::new(r))))
    }
}

/// `RUN-DIGITS` when a part of a reduced value has more than 4 096 decimal digits, screened
/// by its bit length: only a part of 13 607 bits needs the comparison with 10^4096. L §7.1.
pub(super) fn check_cap(part: &Big) -> Result<(), NumError> {
    let bits = part.bits();
    if bits <= CAP_SURE_BITS {
        return Ok(());
    }
    let limit_bits = CAP_SURE_BITS.checked_add(1).ok_or(NumError::Invariant)?;
    if bits > limit_bits {
        return Err(NumError::Digits);
    }
    if part.abs().order(&Big::pow(10, CAP_DIGITS)?) != Ordering::Less {
        return Err(NumError::Digits);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Big, BigRat, Num, NumError, Repr};
    use core::cmp::Ordering;

    fn n(s: &str) -> Num {
        Num::parse_decimal(s).expect("a decimal")
    }

    /// A value with `bits` bits: 2^(bits - 1).
    fn power_of_two(bits: u32) -> Num {
        let r = BigRat::integer(Big::pow(2, bits.checked_sub(1).expect("bits")).expect("pow"));
        Num::from_big(r).expect("under the cap")
    }

    #[test]
    fn num_is_sixteen_bytes() {
        // BUILDER §2, U-11: the niche in the denominator keeps Num at 16 bytes
        assert_eq!(size_of::<Num>(), 16, "size of Num");
    }

    #[test]
    fn values_demote_to_the_small_tier() {
        let big = Num::int(i64::MAX).checked_add(&Num::int(1)).expect("2^63");
        assert!(matches!(big.0, Repr::Big(_)), "2^63 is big");
        let back = big.checked_sub(&Num::int(1)).expect("back");
        assert!(matches!(back.0, Repr::Small(_)), "demoted");
        assert_eq!(back, Num::int(i64::MAX), "equal after the round trip");
        assert!(
            matches!(Num::int(i64::MIN).0, Repr::Big(_)),
            "i64::MIN is held big"
        );
        assert_eq!(
            Num::int(i64::MIN).checked_neg().expect("neg"),
            Num::int(i64::MAX).checked_add(&Num::int(1)).expect("2^63"),
            "-i64::MIN"
        );
    }

    #[test]
    fn arithmetic_is_exact() {
        // L §7.1: 0.1 + 0.2 is exactly 0.3; 1/3 × 3 is 1
        assert_eq!(
            n("0.1").checked_add(&n("0.2")).expect("sum"),
            n("0.3"),
            "0.1 + 0.2"
        );
        let third = Num::ratio(1, 3).expect("1/3");
        assert_eq!(
            third.checked_mul(&Num::int(3)).expect("product"),
            Num::int(1),
            "1/3 × 3"
        );
        assert_eq!(
            Num::int(1).checked_div(&Num::int(0)),
            Err(NumError::Div),
            "RUN-DIV"
        );
        assert_eq!(Num::ratio(1, 0), Err(NumError::Div), "RUN-DIV");
        assert_eq!(
            Num::ratio(3, -6).expect("-1/2"),
            n("-0.5"),
            "the sign moves up"
        );
        assert_eq!(
            n("-0.5").compare(&third).expect("cmp"),
            Ordering::Less,
            "order"
        );
    }

    #[test]
    fn small_and_big_agree() {
        // the tiers give one answer: the same operation through 2^70 and back
        let k = power_of_two(71);
        let a = n("1234.5678");
        let b = Num::ratio(-7, 13).expect("b");
        for (x, y) in [(&a, &b), (&b, &a), (&a, &a)] {
            let direct = x.checked_mul(y).expect("small");
            let through = x
                .checked_mul(&k)
                .and_then(|v| v.checked_mul(y))
                .and_then(|v| v.checked_div(&k))
                .expect("big");
            assert_eq!(direct, through, "multiply");
            let sum = x.checked_add(y).expect("small");
            let sum_big = x
                .checked_add(&k)
                .and_then(|v| v.checked_add(y))
                .and_then(|v| v.checked_sub(&k))
                .expect("big");
            assert_eq!(sum, sum_big, "add");
            let scaled_x = x.checked_mul(&k).expect("x");
            let scaled_y = y.checked_mul(&k).expect("y");
            assert_eq!(
                x.compare(y).expect("small"),
                scaled_x.compare(&scaled_y).expect("big"),
                "compare"
            );
        }
    }

    #[test]
    fn the_cap_is_4096_digits() {
        // BUILDER §10: fixtures at 13 606, 13 607 and 13 608 bits; 10^4096 - 1 passes,
        // 10^4096 is refused
        let sure = power_of_two(13_606);
        assert!(sure.checked_add(&Num::int(1)).is_ok(), "13 606 bits pass");
        let tenth = Num::from_big(BigRat::integer(Big::pow(10, 4_095).expect("pow")))
            .expect("10^4095 passes");
        assert_eq!(
            tenth.checked_mul(&Num::int(10)),
            Err(NumError::Digits),
            "10^4096 is refused"
        );
        let below = BigRat::integer(
            Big::pow(10, 4_096)
                .expect("pow")
                .sub(&Big::from_i128(1))
                .expect("sub"),
        );
        assert!(Num::from_big(below).is_ok(), "10^4096 - 1 passes");
        let mid = power_of_two(13_607);
        assert!(
            mid.checked_add(&Num::int(0)).is_ok(),
            "2^13606 < 10^4096 passes"
        );
        assert_eq!(
            mid.checked_mul(&Num::int(2)),
            Err(NumError::Digits),
            "13 608 bits are refused"
        );
        assert!(
            Num::int(1).checked_div(&mid).is_ok(),
            "a denominator of 13 607 bits under 10^4096 passes"
        );
        let tiny_over = Num::int(1)
            .checked_div(&tenth)
            .and_then(|v| v.checked_div(&Num::int(10)));
        assert_eq!(
            tiny_over,
            Err(NumError::Digits),
            "a denominator of 10^4096 is refused"
        );
    }

    #[test]
    fn to_i64_reads_whole_numbers() {
        assert_eq!(Num::int(-5).to_i64(), Some(-5), "small");
        assert_eq!(n("2.5").to_i64(), None, "not whole");
        assert_eq!(
            Num::int(i64::MIN).to_i64(),
            Some(i64::MIN),
            "held big, fits i64"
        );
        assert_eq!(power_of_two(80).to_i64(), None, "too large");
        assert!(n("0").is_zero(), "zero");
        assert!(n("-0.01").is_negative(), "negative");
        assert!(n("12").is_integer(), "integer");
    }
}
