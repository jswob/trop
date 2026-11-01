//! The small tier of the exact numbers: a reduced fraction of two 64-bit integers.
//! Spec: L §7.1 (exact numbers); P §6 (the small tier, the whole-number shortcut).
//! Twin: engine/tools/refcore/values.py (Python's `Fraction`).
//! Invariants: reduced; `0 < d ≤ i64::MAX`; `n ≠ i64::MIN`; every result is computed exactly in
//! 128 bits, and one that doesn't fit the small tier is handed back as a 128-bit pair.
//!
//! The intermediates can't overflow 128 bits: each product of two values under 2^63 is under
//! 2^126, and a sum of two such is under 2^127. The checked calls stay anyway (R-19).

use super::NumError;
use super::gcd::{gcd_u64, gcd_u128};
use core::cmp::Ordering;
use core::num::NonZeroU64;

/// A small value `n / d`: reduced, `d > 0`, `n ≠ i64::MIN`, `d ≤ i64::MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Small {
    /// The numerator, its sign the value's.
    pub(crate) n: i64,
    /// The denominator, positive; non-zero by its type, which gives `Num` its niche.
    pub(crate) d: NonZeroU64,
}

/// A result of the small tier: a small value, or a reduced pair of 128-bit integers with a
/// positive denominator, which the caller widens to the big tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wide {
    /// The result fits the small tier.
    Fits(Small),
    /// The result, reduced, with `d > 0`, too large for the small tier.
    Pair(i128, i128),
}

impl Small {
    /// The whole number `n`, or `None` at `i64::MIN`, which the small tier excludes. L §7.1.
    pub(crate) fn whole(n: i64) -> Option<Small> {
        if n == i64::MIN {
            return None;
        }
        Some(Small {
            n,
            d: NonZeroU64::MIN,
        })
    }

    /// The denominator as a 128-bit integer. L §7.1.
    fn d128(self) -> i128 {
        i128::from(self.d.get())
    }

    /// Whether the value is a whole number. L §7.1.
    pub(crate) fn is_integer(self) -> bool {
        self.d == NonZeroU64::MIN
    }
}

/// `n / d` reduced, for `d > 0`: small when it fits, else a 128-bit pair. L §7.1.
pub(crate) fn reduce(n: i128, d: i128) -> Result<Wide, NumError> {
    let d_mag = u128::try_from(d).map_err(|_| NumError::Invariant)?;
    if d_mag == 0 {
        return Err(NumError::Div);
    }
    let g = gcd_u128(n.unsigned_abs(), d_mag);
    let g = i128::try_from(g).map_err(|_| NumError::Invariant)?;
    let n = n.checked_div(g).ok_or(NumError::Invariant)?;
    let d = d.checked_div(g).ok_or(NumError::Invariant)?;
    Ok(fit(n, d))
}

/// A reduced pair with `d > 0`, as a small value when both parts fit. L §7.1.
fn fit(n: i128, d: i128) -> Wide {
    if let (Some(n), Some(d)) = (small_numerator(n), small_denominator(d)) {
        return Wide::Fits(Small { n, d });
    }
    Wide::Pair(n, d)
}

/// A numerator the small tier holds: an `i64` other than `i64::MIN`. L §7.1.
fn small_numerator(n: i128) -> Option<i64> {
    match i64::try_from(n) {
        Ok(v) if v != i64::MIN => Some(v),
        Ok(_) | Err(_) => None,
    }
}

/// A denominator the small tier holds: positive and at most `i64::MAX`. L §7.1.
fn small_denominator(d: i128) -> Option<NonZeroU64> {
    if d > i128::from(i64::MAX) {
        return None;
    }
    match u64::try_from(d) {
        Ok(v) => NonZeroU64::new(v),
        Err(_) => None,
    }
}

/// `a + b`: one checked machine add for two whole numbers, else one gcd of the denominators
/// (Knuth, TAOCP 4.5.1). L §7.1.
pub(crate) fn add(a: Small, b: Small) -> Result<Wide, NumError> {
    if a.is_integer() && b.is_integer() {
        let n = i128::from(a.n)
            .checked_add(i128::from(b.n))
            .ok_or(NumError::Invariant)?;
        return Ok(fit(n, 1));
    }
    let common = i128::from(gcd_u64(a.d.get(), b.d.get()));
    let ad_g = div(a.d128(), common)?;
    let bd_g = div(b.d128(), common)?;
    let numer = mul(i128::from(a.n), bd_g)?
        .checked_add(mul(i128::from(b.n), ad_g)?)
        .ok_or(NumError::Invariant)?;
    reduce(numer, mul(ad_g, b.d128())?)
}

/// `-a`, exact since `n ≠ i64::MIN`. L §7.1.
pub(crate) fn neg(a: Small) -> Result<Small, NumError> {
    Ok(Small {
        n: a.n.checked_neg().ok_or(NumError::Invariant)?,
        d: a.d,
    })
}

/// `a × b`, cross-reduced (Knuth, TAOCP 4.5.1); one checked machine multiply for two whole
/// numbers. L §7.1.
pub(crate) fn mul_small(a: Small, b: Small) -> Result<Wide, NumError> {
    if a.is_integer() && b.is_integer() {
        return Ok(fit(mul(i128::from(a.n), i128::from(b.n))?, 1));
    }
    let g1 = i128::from(gcd_u64(a.n.unsigned_abs(), b.d.get()));
    let g2 = i128::from(gcd_u64(b.n.unsigned_abs(), a.d.get()));
    let n = mul(div(i128::from(a.n), g1)?, div(i128::from(b.n), g2)?)?;
    let d = mul(div(a.d128(), g2)?, div(b.d128(), g1)?)?;
    Ok(fit(n, d))
}

/// `1 / a`; `RUN-DIV` at 0. L §7.1.
pub(crate) fn recip(a: Small) -> Result<Small, NumError> {
    let d = i64::try_from(a.d.get()).map_err(|_| NumError::Invariant)?;
    let n_mag = NonZeroU64::new(a.n.unsigned_abs()).ok_or(NumError::Div)?;
    let n = if a.n < 0 {
        d.checked_neg().ok_or(NumError::Invariant)?
    } else {
        d
    };
    Ok(Small { n, d: n_mag })
}

/// The order of two values, cross-multiplied in 128 bits. L §7.1.
pub(crate) fn compare(a: Small, b: Small) -> Result<Ordering, NumError> {
    Ok(mul(i128::from(a.n), b.d128())?.cmp(&mul(i128::from(b.n), a.d128())?))
}

/// The floor of the value, and how its fractional part compares with one half: what
/// rounding needs (Python's `//` and `%`). L §7.2.
pub(crate) fn floor_parts(a: Small) -> Result<(i64, Ordering, bool), NumError> {
    let d = i64::try_from(a.d.get()).map_err(|_| NumError::Invariant)?;
    // Python: a.n // d and a.n % d, with d > 0
    let k = a.n.checked_div_euclid(d).ok_or(NumError::Invariant)?;
    let r = a.n.checked_rem_euclid(d).ok_or(NumError::Invariant)?;
    let twice_r = i128::from(r).checked_mul(2).ok_or(NumError::Invariant)?;
    Ok((k, twice_r.cmp(&i128::from(d)), r == 0))
}

/// `x × y` in 128 bits, checked. L §7.1.
fn mul(x: i128, y: i128) -> Result<i128, NumError> {
    x.checked_mul(y).ok_or(NumError::Invariant)
}

/// `x ÷ y` in 128 bits for a `y` that divides `x` (a gcd), checked. L §7.1.
fn div(x: i128, y: i128) -> Result<i128, NumError> {
    x.checked_div(y).ok_or(NumError::Invariant)
}

#[cfg(test)]
mod tests {
    use super::{Small, Wide, add, compare, floor_parts, mul_small, recip, reduce};
    use core::cmp::Ordering;
    use core::num::NonZeroU64;

    fn s(n: i64, d: u64) -> Small {
        match reduce(i128::from(n), i128::from(d)).expect("a value") {
            Wide::Fits(v) => v,
            Wide::Pair(..) => panic!("{n}/{d} is small"),
        }
    }

    #[test]
    fn reduce_keeps_the_invariants() {
        assert_eq!(
            s(6, 4),
            Small {
                n: 3,
                d: NonZeroU64::new(2).expect("2")
            },
            "6/4 = 3/2"
        );
        assert_eq!(
            reduce(i128::from(i64::MIN), 1).expect("pair"),
            Wide::Pair(i128::from(i64::MIN), 1),
            "i64::MIN is not small"
        );
        assert_eq!(
            reduce(1, i128::from(i64::MAX).checked_add(1).expect("d")).expect("pair"),
            Wide::Pair(1, i128::from(i64::MAX).checked_add(1).expect("d")),
            "a denominator past i64::MAX is not small"
        );
        assert!(reduce(1, 0).is_err(), "RUN-DIV");
    }

    #[test]
    fn whole_numbers_take_the_shortcut() {
        assert_eq!(
            add(s(2, 1), s(3, 1)).expect("sum"),
            Wide::Fits(s(5, 1)),
            "2 + 3"
        );
        assert_eq!(
            add(s(i64::MAX, 1), s(1, 1)).expect("sum"),
            Wide::Pair(i128::from(i64::MAX).checked_add(1).expect("n"), 1),
            "widens past i64::MAX"
        );
        assert_eq!(
            mul_small(s(i64::MAX, 1), s(2, 1)).expect("product"),
            Wide::Pair(i128::from(i64::MAX).checked_mul(2).expect("n"), 1),
            "widens"
        );
    }

    #[test]
    fn fractions() {
        assert_eq!(
            add(s(1, 6), s(1, 10)).expect("sum"),
            Wide::Fits(s(4, 15)),
            "1/6 + 1/10"
        );
        assert_eq!(
            add(s(1, 2), s(-1, 2)).expect("sum"),
            Wide::Fits(s(0, 1)),
            "to 0"
        );
        assert_eq!(
            mul_small(s(2, 3), s(9, 4)).expect("product"),
            Wide::Fits(s(3, 2)),
            "2/3 × 9/4"
        );
        assert_eq!(recip(s(-2, 3)).expect("recip"), s(-3, 2), "1/(-2/3)");
        assert!(recip(s(0, 1)).is_err(), "1/0");
        assert_eq!(
            compare(s(-1, 2), s(1, 3)).expect("cmp"),
            Ordering::Less,
            "order"
        );
        assert_eq!(
            compare(s(2, 4), s(1, 2)).expect("cmp"),
            Ordering::Equal,
            "equal"
        );
    }

    #[test]
    fn floor_parts_follow_python() {
        // -7/2: -7 // 2 == -4, the fraction 1/2 is a tie
        assert_eq!(
            floor_parts(s(-7, 2)).expect("parts"),
            (-4, Ordering::Equal, false),
            "-3.5"
        );
        assert_eq!(
            floor_parts(s(7, 3)).expect("parts"),
            (2, Ordering::Less, false),
            "2.33"
        );
        assert_eq!(
            floor_parts(s(-7, 3)).expect("parts"),
            (-3, Ordering::Greater, false),
            "-2.33"
        );
        assert_eq!(
            floor_parts(s(4, 1)).expect("parts"),
            (4, Ordering::Less, true),
            "4"
        );
    }
}
