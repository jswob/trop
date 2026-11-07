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

    /// The keys of the integer range `self .. hi step step`, as refcore's Python integers:
    /// the keys are counted on the big tier, uncapped, so ends near the 4 096-digit cap count
    /// as any others, and each key made lies between the ends, under the cap; no key when
    /// `hi < self`, `Ok(None)` past `limit` keys. `RUN-DOMAIN` unless all three are whole and
    /// the step positive. L §5.6, §7.1, §7.6 item 2.
    pub fn range_keys(
        &self,
        hi: &Num,
        step: &Num,
        limit: u64,
    ) -> Result<Option<Vec<Num>>, NumError> {
        let whole = self.is_integer() && hi.is_integer() && step.is_integer();
        if !whole || step.is_negative() || step.is_zero() {
            return Err(NumError::Domain);
        }
        let span = hi.to_big().numer().sub(self.to_big().numer())?;
        if span.is_negative() {
            return Ok(Some(Vec::new()));
        }
        // the keys after the first, ⌊(hi − lo) / step⌋
        let after = span.div_floor(step.to_big().numer())?.to_u64();
        let Some(mut left) = after.filter(|n| *n < limit) else {
            return Ok(None);
        };
        let mut keys = Vec::new();
        let mut key = self.clone();
        loop {
            if left == 0 {
                keys.push(key);
                return Ok(Some(keys));
            }
            let next = key.checked_add(step)?;
            keys.push(key);
            key = next;
            left = left.checked_sub(1).ok_or(NumError::Invariant)?;
        }
    }

    /// Whether the range `self .. hi step step` holds more than `limit` keys, counted on the
    /// big tier, uncapped, as refcore's Python numbers count them, and never stepped:
    /// `⌊(hi − self) ÷ step⌋ + 1 > limit`, that is `hi − self ≥ limit × step` (no key when
    /// `hi < self`). `RUN-DOMAIN` unless the step is positive. L §5.6, §7.1, §7.6 item 2.
    pub fn range_past(&self, hi: &Num, step: &Num, limit: u64) -> Result<bool, NumError> {
        if step.is_negative() || step.is_zero() {
            return Err(NumError::Domain);
        }
        let span = hi.to_big().add(&self.to_big().neg()?)?;
        let reach = step
            .to_big()
            .mul(&BigRat::integer(Big::from_u128(u128::from(limit))))?;
        Ok(span.compare(&reach)? != Ordering::Less)
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

    /// xorshift64, a fixed-seed generator of 64-bit words, so the cases repeat.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0.checked_shl(13).expect("shift");
            self.0 ^= self.0.checked_shr(7).expect("shift");
            self.0 ^= self.0.checked_shl(17).expect("shift");
            self.0
        }

        /// A small value whose numerator and denominator have at most `bits` bits each: short
        /// decimals and whole numbers at 20 bits, values near the tier's edge at 63.
        fn small(&mut self, bits: u32) -> Num {
            let shift = 64_u32.checked_sub(bits).expect("at most 64 bits");
            let mask = u64::MAX.checked_shr(shift).expect("a mask");
            let top = i64::try_from(self.next() & mask).expect("at most 63 bits");
            let n = if self.next() & 1 == 1 {
                top.checked_neg().expect("negated")
            } else {
                top
            };
            let whole = self.next().checked_rem_euclid(3) == Some(0);
            let d = if whole {
                1
            } else {
                i64::try_from((self.next() & mask).max(1)).expect("at most 63 bits")
            };
            Num::ratio(n, d).expect("a small value")
        }
    }

    #[test]
    fn the_tiers_agree_on_fixed_seed_values() {
        // P §6, the F4 fence's equivalence test: each operation on two small values, directly
        // and through the big tier (scaled by 2^70 and back), gives one answer
        let k = power_of_two(71);
        let mut rng = Rng(0x5EED_2026_1102);
        for case in 0..600_u32 {
            let bits = [20, 40, 63]
                .get(usize::try_from(case.checked_rem_euclid(3).expect("3")).expect("an index"))
                .copied()
                .expect("bits");
            let x = rng.small(bits);
            let y = rng.small(bits);
            let product = x.checked_mul(&y).expect("x × y");
            let through = x
                .checked_mul(&k)
                .and_then(|v| v.checked_mul(&y))
                .and_then(|v| v.checked_div(&k))
                .expect("x × 2^70 × y / 2^70");
            assert_eq!(product, through, "multiply, case {case}");
            let sum = x.checked_add(&y).expect("x + y");
            let sum_big = x
                .checked_add(&k)
                .and_then(|v| v.checked_add(&y))
                .and_then(|v| v.checked_sub(&k))
                .expect("x + 2^70 + y - 2^70");
            assert_eq!(sum, sum_big, "add, case {case}");
            if !y.is_zero() {
                let quotient = x.checked_div(&y).expect("x / y");
                let quotient_big = x
                    .checked_mul(&k)
                    .and_then(|v| v.checked_div(&y))
                    .and_then(|v| v.checked_div(&k))
                    .expect("x × 2^70 / y / 2^70");
                assert_eq!(quotient, quotient_big, "divide, case {case}");
            }
            let scaled_x = x.checked_mul(&k).expect("x × 2^70");
            let scaled_y = y.checked_mul(&k).expect("y × 2^70");
            assert_eq!(
                x.compare(&y).expect("small"),
                scaled_x.compare(&scaled_y).expect("big"),
                "compare, case {case}"
            );
        }
    }

    #[test]
    fn a_range_counts_its_keys_uncapped_and_makes_only_keys_under_the_cap() {
        let (one, two) = (Num::int(1), Num::int(2));
        let nines = Big::pow(10, 4_096)
            .expect("pow")
            .sub(&Big::from_i128(1))
            .expect("sub");
        let top = Num::from_big(BigRat::integer(nines)).expect("10^4096 - 1");
        let ntop = top.checked_neg().expect("-top");
        let at = |lo: &Num, hi: &Num, step: &Num, limit| lo.range_keys(hi, step, limit);
        assert_eq!(
            at(&top, &top, &one, 9),
            Ok(Some(vec![top.clone()])),
            "top .. top"
        );
        let three = vec![ntop.clone(), Num::zero(), top.clone()];
        assert_eq!(
            at(&ntop, &top, &top, 9),
            Ok(Some(three)),
            "ntop .. top step top"
        );
        assert_eq!(at(&ntop, &top, &one, 9), Ok(None), "ntop .. top: too many");
        let odd = vec![Num::int(1), Num::int(3), Num::int(5)];
        assert_eq!(
            at(&one, &Num::int(6), &two, 3),
            Ok(Some(odd)),
            "1 .. 6 step 2, at the limit"
        );
        assert_eq!(
            at(&one, &Num::int(7), &two, 3),
            Ok(None),
            "1 .. 7 step 2, past it"
        );
        assert_eq!(
            at(&two, &one, &one, 3),
            Ok(Some(Vec::new())),
            "2 .. 1: no key"
        );
        assert_eq!(
            at(&one, &two, &Num::zero(), 3),
            Err(NumError::Domain),
            "step 0"
        );
        assert_eq!(
            at(&one, &two, &Num::int(-1), 3),
            Err(NumError::Domain),
            "step -1"
        );
        let half = Num::ratio(1, 2).expect("1/2");
        assert_eq!(
            at(&half, &two, &one, 3),
            Err(NumError::Domain),
            "an end not whole"
        );
        assert_eq!(
            at(&one, &half, &one, 3),
            Err(NumError::Domain),
            "the other end"
        );
        assert_eq!(
            at(&one, &two, &half, 3),
            Err(NumError::Domain),
            "a step not whole"
        );
    }

    #[test]
    fn a_range_is_past_its_limit_counted_uncapped_and_its_step_is_positive() {
        // L §7.6 item 2: ⌊(hi − lo) ÷ step⌋ + 1 keys, counted on the big tier, never stepped
        let (one, two, six) = (Num::int(1), Num::int(2), Num::int(6));
        let past = |lo: &Num, hi: &Num, step: &Num, limit| lo.range_past(hi, step, limit);
        assert_eq!(
            past(&one, &six, &two, 3),
            Ok(false),
            "1 .. 6 step 2: 3 keys"
        );
        assert_eq!(
            past(&one, &Num::int(7), &two, 3),
            Ok(true),
            "1 .. 7 step 2: 4"
        );
        assert_eq!(past(&two, &one, &one, 0), Ok(false), "2 .. 1: no key");
        let nines = Big::pow(10, 4_096)
            .expect("pow")
            .sub(&Big::from_i128(1))
            .expect("sub");
        let top = Num::from_big(BigRat::integer(nines)).expect("10^4096 - 1");
        let ntop = top.checked_neg().expect("-top");
        assert_eq!(
            past(&ntop, &top, &one, u64::MAX),
            Ok(true),
            "-top .. top: past every limit, uncapped"
        );
        assert_eq!(
            past(&one, &two, &Num::zero(), 3),
            Err(NumError::Domain),
            "step 0"
        );
        assert_eq!(
            past(&one, &two, &Num::int(-1), 3),
            Err(NumError::Domain),
            "step -1"
        );
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
