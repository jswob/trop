//! Greatest common divisors for reducing exact fractions: machine words, big by small, Lehmer.
//! Spec: L §7.1 (values are reduced fractions); P §6 (Lehmer's gcd, big × small).
//! Twin: engine/tools/refcore/values.py (Python's `math.gcd`, inside `Fraction`).
//! Invariants: every result is the gcd of the absolute values, gcd(0, 0) = 0; no step can
//! crash or wrap: each division is a checked call.
//!
//! Lehmer's algorithm is Knuth, TAOCP vol. 2, 4.5.2, Algorithm L, with 64-bit leading parts
//! and `i128` cofactors. Collins's test (the two quotients agree) keeps every quotient
//! exact, so the cofactor matrix applied to the big values is the one the Euclidean steps
//! would apply, and the values stay non-negative.

use super::NumError;
use super::big::Big;
use core::cmp::Ordering;

/// gcd of two machine words by Euclid; `checked_rem` is `None` exactly when the divisor is
/// 0, which ends the loop. L §7.1.
pub(crate) fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while let Some(r) = a.checked_rem(b) {
        a = b;
        b = r;
    }
    a
}

/// gcd of two double words, for the small tier's 128-bit intermediates. L §7.1.
pub(crate) fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while let Some(r) = a.checked_rem(b) {
        a = b;
        b = r;
    }
    a
}

/// gcd of the absolute values of two big integers: Lehmer's steps while both are big, then
/// one big-by-small reduction and machine words (P §6). L §7.1.
pub(crate) fn gcd_big(a: &Big, b: &Big) -> Result<Big, NumError> {
    let (mut u, mut v) = (a.abs(), b.abs());
    if u.order(&v) == Ordering::Less {
        core::mem::swap(&mut u, &mut v);
    }
    loop {
        if v.is_zero() {
            return Ok(u);
        }
        if let Some(small) = v.to_u64() {
            let rest = u.rem_floor(&v)?.to_u64().ok_or(NumError::Invariant)?;
            return Ok(Big::from_u128(u128::from(gcd_u64(small, rest))));
        }
        (u, v) = lehmer_step(&u, &v)?;
    }
}

/// One step of Algorithm L on `u ≥ v > 2^64`: the cofactors found from the leading 64 bits,
/// applied to the big values; or one Euclidean division when the leading bits decide no
/// quotient (Knuth's step L4 with B = 0). P §6.
fn lehmer_step(u: &Big, v: &Big) -> Result<(Big, Big), NumError> {
    let shift = u.bits().checked_sub(64).ok_or(NumError::Invariant)?;
    let u_hat = leading_bits(&u.limbs(), shift)?;
    let v_hat = leading_bits(&v.limbs(), shift)?;
    let cof = cofactors(u_hat, v_hat)?;
    if cof.b == 0 {
        // no step agreed on: one plain Euclidean step on the full values
        return Ok((v.clone(), u.rem_floor(v)?));
    }
    Ok((combine(cof.a, u, cof.b, v)?, combine(cof.c, u, cof.d, v)?))
}

/// Knuth's cofactors A, B, C, D (TAOCP 4.5.2, Algorithm L).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cofactors {
    /// A
    a: i128,
    /// B
    b: i128,
    /// C
    c: i128,
    /// D
    d: i128,
}

/// Steps L2–L3 on the leading parts `û`, `v̂`: the Euclidean steps both quotient bounds agree
/// on, as a cofactor matrix. P §6.
fn cofactors(u_hat: u64, v_hat: u64) -> Result<Cofactors, NumError> {
    let mut m = Cofactors {
        a: 1,
        b: 0,
        c: 0,
        d: 1,
    };
    let (mut uh, mut vh) = (i128::from(u_hat), i128::from(v_hat));
    loop {
        let (Some(q), Some(q2)) = (
            quotient(uh.checked_add(m.a), vh.checked_add(m.c)),
            quotient(uh.checked_add(m.b), vh.checked_add(m.d)),
        ) else {
            return Ok(m);
        };
        if q != q2 {
            return Ok(m);
        }
        m = Cofactors {
            a: m.c,
            b: m.d,
            c: sub_mul(m.a, q, m.c)?,
            d: sub_mul(m.b, q, m.d)?,
        };
        let next = sub_mul(uh, q, vh)?;
        uh = vh;
        vh = next;
    }
}

/// One quotient bound of step L2, `top ÷ bottom`; `None` when a part overflowed or the
/// divisor is 0, which ends the steps. Both parts are non-negative here (Knuth), so the
/// truncating division is the floor. P §6.
fn quotient(top: Option<i128>, bottom: Option<i128>) -> Option<i128> {
    top?.checked_div(bottom?)
}

/// `x - q × y`, checked. P §6.
fn sub_mul(x: i128, q: i128, y: i128) -> Result<i128, NumError> {
    q.checked_mul(y)
        .and_then(|p| x.checked_sub(p))
        .ok_or(NumError::Invariant)
}

/// `p × u + q × v` on big values with machine cofactors. P §6.
fn combine(p: i128, u: &Big, q: i128, v: &Big) -> Result<Big, NumError> {
    Big::from_i128(p).mul(u)?.add(&Big::from_i128(q).mul(v)?)
}

/// The 64 bits of a magnitude starting at bit `shift` (its limbs least significant first):
/// `⌊x / 2^shift⌋` when that fits a word. P §6.
fn leading_bits(limbs: &[u64], shift: u64) -> Result<u64, NumError> {
    let word = usize::try_from(shift.checked_div(64).ok_or(NumError::Invariant)?)
        .map_err(|_| NumError::Invariant)?;
    let offset = u32::try_from(shift.checked_rem(64).ok_or(NumError::Invariant)?)
        .map_err(|_| NumError::Invariant)?;
    let low = limb(limbs, word);
    let high = limb(limbs, word.checked_add(1).ok_or(NumError::Invariant)?);
    let both = u128::from(high)
        .checked_shl(64)
        .map(|h| h | u128::from(low))
        .ok_or(NumError::Invariant)?;
    let shifted = both.checked_shr(offset).ok_or(NumError::Invariant)?;
    u64::try_from(shifted & u128::from(u64::MAX)).map_err(|_| NumError::Invariant)
}

/// Limb `i` of a magnitude: a limb past the most significant one is 0, since the magnitude
/// has no more bits. P §6.
fn limb(limbs: &[u64], i: usize) -> u64 {
    match limbs.get(i) {
        Some(&x) => x,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{Big, gcd_big, gcd_u64, gcd_u128, leading_bits};

    /// The plain twin: Euclid on big values, one division a step.
    fn euclid(a: &Big, b: &Big) -> Big {
        let (mut u, mut v) = (a.abs(), b.abs());
        while !v.is_zero() {
            let rest = u.rem_floor(&v).expect("v is not 0");
            u = v;
            v = rest;
        }
        u
    }

    /// xorshift64, a fixed-seed generator for test inputs.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0.checked_shl(13).expect("shift");
            self.0 ^= self.0.checked_shr(7).expect("shift");
            self.0 ^= self.0.checked_shl(17).expect("shift");
            self.0
        }
        /// A big value of `limbs` random 64-bit limbs.
        fn big(&mut self, limbs: usize) -> Big {
            let base = Big::pow(2, 64).expect("2^64");
            let mut v = Big::zero();
            for _ in 0..limbs {
                v = v
                    .mul(&base)
                    .expect("mul")
                    .add(&Big::from_u128(u128::from(self.next())))
                    .expect("add");
            }
            v
        }
    }

    #[test]
    fn machine_gcds() {
        assert_eq!(gcd_u64(0, 0), 0, "gcd(0, 0)");
        assert_eq!(gcd_u64(0, 7), 7, "gcd(0, 7)");
        assert_eq!(gcd_u64(12, 18), 6, "gcd(12, 18)");
        assert_eq!(gcd_u64(u64::MAX, 3), 3, "u64::MAX is divisible by 3");
        let (a, b) = (
            u128::MAX.checked_shr(27).expect("shift"),
            1_u128.checked_shl(70).expect("shift"),
        );
        assert_eq!(gcd_u128(a, b), 1, "odd and a power of 2");
        assert_eq!(
            gcd_u128(b, b.checked_shl(30).expect("shift")),
            b,
            "powers of 2"
        );
    }

    #[test]
    fn leading_bits_reads_across_limbs() {
        let limbs = [0xFFFF_0000_0000_0000, 0x0000_0000_0000_ABCD];
        assert_eq!(
            leading_bits(&limbs, 48).expect("bits"),
            0xABCD_FFFF,
            "two limbs"
        );
        assert_eq!(leading_bits(&limbs, 64).expect("bits"), 0xABCD, "one limb");
        assert_eq!(leading_bits(&limbs, 200).expect("bits"), 0, "past the end");
    }

    #[test]
    fn lehmer_agrees_with_euclid() {
        // BUILDER §2: Lehmer tested against a reference on random and boundary values
        let mut rng = Rng(0x5EED_2026_1101);
        for case in 0..200_usize {
            let common = rng.big(
                case.checked_rem_euclid(3)
                    .and_then(|k| k.checked_add(1))
                    .expect("n"),
            );
            let a = rng
                .big(
                    case.checked_rem_euclid(7)
                        .and_then(|k| k.checked_add(1))
                        .expect("n"),
                )
                .mul(&common)
                .expect("a");
            let b = rng
                .big(
                    case.checked_rem_euclid(5)
                        .and_then(|k| k.checked_add(1))
                        .expect("n"),
                )
                .mul(&common)
                .expect("b");
            assert_eq!(gcd_big(&a, &b).expect("gcd"), euclid(&a, &b), "case {case}");
        }
    }

    #[test]
    fn gcd_big_edges() {
        let x = Big::pow(2, 300).expect("2^300");
        let y = Big::pow(6, 100).expect("6^100");
        assert_eq!(
            gcd_big(&x, &y).expect("gcd"),
            Big::pow(2, 100).expect("2^100"),
            "2^100"
        );
        assert_eq!(gcd_big(&x, &Big::zero()).expect("gcd"), x, "gcd(x, 0)");
        assert_eq!(
            gcd_big(&x.neg().expect("-x"), &x).expect("gcd"),
            x,
            "signs ignored"
        );
        let p = Big::pow(10, 60)
            .expect("10^60")
            .add(&Big::from_i128(1))
            .expect("p");
        assert_eq!(
            gcd_big(&p, &Big::from_i128(7)).expect("gcd"),
            euclid(&p, &Big::from_i128(7)),
            "big by small"
        );
    }
}
