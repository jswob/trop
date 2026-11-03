//! Exact numbers as text: decimal literals read exactly, values printed in the output's form.
//! Spec: L §2.1 (number literals), §5.18 (output: decimals, `a/b`, the 15-digit aid), §7.1.
//! Twin: engine/tools/refcore/values.py (`fmt_dec`, `fmt_q`, `sig15`, `group3`, `dec_exp`).
//! Invariants: a literal is read exactly or refused, never rounded; printing is exact, and the
//! 15-digit display beside an `a/b` is only a reading aid, rounded half to even.
//!
//! Printing works on the big tier's integers whatever the value's tier, uncapped: a value
//! under the cap may need more digits to print than to hold.

use super::big::{Big, BigRat};
use super::money::Money;
use super::rational::{Num, NumError, Repr};
use core::cmp::Ordering;

/// The most decimal digits read as one machine word: 10^18 < 2^63.
const WORD_DIGITS: usize = 18;
/// The significant digits of the reading aid. L §5.18.
const AID_DIGITS: i64 = 15;

impl Num {
    /// The value a decimal literal states (`-12.50`, `0.0976`, `24000`), exactly;
    /// `NumError::Domain` for text that isn't one. L §2.1, §7.1.
    pub fn parse_decimal(text: &str) -> Result<Num, NumError> {
        let (negative, body) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (whole, fraction) = match body.split_once('.') {
            Some((w, f)) if !f.is_empty() => (w, f),
            Some(_) => return Err(NumError::Domain),
            None => (body, ""),
        };
        if whole.is_empty() || !all_digits(whole) || !all_digits(fraction) {
            return Err(NumError::Domain);
        }
        let digits = [whole, fraction].concat();
        let places = u32::try_from(fraction.len()).map_err(|_| NumError::Domain)?;
        let value = if digits.len() <= WORD_DIGITS {
            let n = word_digits(&digits)?;
            let d = 10_i64.checked_pow(places).ok_or(NumError::Invariant)?;
            Num::ratio(n, d)?
        } else {
            let n = Big::from_decimal_digits(&digits).ok_or(NumError::Domain)?;
            Num::from_big(BigRat::new(&n, &Big::pow(10, places)?)?)?
        };
        if negative {
            return value.checked_neg();
        }
        Ok(value)
    }

    /// The value as L §5.18 prints a `num`: the shortest exact decimal, with the 15-digit
    /// aid beside one of more than 15 significant digits; otherwise `a/b` and the aid.
    pub fn to_text(&self) -> Result<String, NumError> {
        let r = self.rational();
        let Some(places) = decimal_places(r.denom())? else {
            let aid = aid(&r)?;
            let sign = if r.numer().is_negative() { "-" } else { "" };
            return Ok(format!(
                "{sign}{}/{} ({aid})",
                r.numer().abs_decimal(),
                r.denom().abs_decimal()
            ));
        };
        let text = fixed(&r, places)?;
        if significant_digits(&r, places)? > AID_DIGITS {
            return Ok(format!("{text} ({})", aid(&r)?));
        }
        Ok(text)
    }

    /// The value with exactly `places` decimals, its whole part grouped by threes
    /// (`6 064.17`): how money and ints print. L §5.18.
    pub fn to_fixed(&self, places: u32) -> Result<String, NumError> {
        fixed(&self.rational(), places)
    }

    /// The decimals of the value's shortest exact decimal, or `None` when it has none (its
    /// denominator has a prime other than 2 and 5). L §5.18.
    pub fn decimal_places(&self) -> Result<Option<u32>, NumError> {
        decimal_places(self.rational().denom())
    }

    /// The value in the big tier's form, uncapped, for printing. L §5.18.
    fn rational(&self) -> BigRat {
        match &self.0 {
            Repr::Small(s) => BigRat::from_reduced_i128(i128::from(s.n), i128::from(s.d.get())),
            Repr::Big(b) => BigRat::clone(b),
        }
    }
}

impl Money {
    /// The amount as L §5.18 prints money: its minor-unit digits (`6 064.17` for a minor unit
    /// of 0.01), without the currency's name. L §5.18.
    pub fn to_text(&self, minor: &Num) -> Result<String, NumError> {
        let places = minor.decimal_places()?.ok_or(NumError::Invariant)?;
        self.to_num(minor)?.to_fixed(places)
    }
}

/// Whether every byte is an ASCII digit. L §2.1.
fn all_digits(text: &str) -> bool {
    text.bytes().all(|b| b.is_ascii_digit())
}

/// The value of at most 18 decimal digits. L §2.1.
fn word_digits(digits: &str) -> Result<i64, NumError> {
    let mut value: i64 = 0;
    for byte in digits.bytes() {
        let digit = i64::from(byte.checked_sub(b'0').ok_or(NumError::Domain)?);
        let shifted = value.checked_mul(10).ok_or(NumError::Invariant)?;
        value = shifted.checked_add(digit).ok_or(NumError::Invariant)?;
    }
    Ok(value)
}

/// The decimals of `1/d`'s shortest exact decimal, the larger power of 2 or 5 in `d`; `None`
/// when `d` has another prime. L §5.18.
fn decimal_places(d: &Big) -> Result<Option<u32>, NumError> {
    let (twos, rest) = strip_factor(d, 2)?;
    let (fives, rest) = strip_factor(&rest, 5)?;
    if rest != Big::from_u128(1) {
        return Ok(None);
    }
    Ok(Some(twos.max(fives)))
}

/// How many times `p` divides `d`, and what is left. L §5.18.
fn strip_factor(d: &Big, p: u64) -> Result<(u32, Big), NumError> {
    let divisor = Big::from_u128(u128::from(p));
    let (mut count, mut rest) = (0_u32, d.clone());
    while !rest.is_zero() && rest.rem_floor(&divisor)?.is_zero() {
        rest = rest.div_exact(&divisor)?;
        count = count.checked_add(1).ok_or(NumError::Invariant)?;
    }
    Ok((count, rest))
}

/// `|r|` with exactly `places` decimals (truncated, as refcore's `fmt_dec`), the whole part
/// grouped by threes, `-` before a value whose digits aren't all 0. L §5.18.
fn fixed(r: &BigRat, places: u32) -> Result<String, NumError> {
    let scale = Big::pow(10, places)?;
    let whole = r.numer().abs().mul(&scale)?.div_floor(r.denom())?;
    let int_part = whole.div_floor(&scale)?;
    let mut text = group3(&int_part.abs_decimal());
    if places > 0 {
        let frac = whole.rem_floor(&scale)?.abs_decimal();
        let width = usize::try_from(places).map_err(|_| NumError::Invariant)?;
        text = format!("{text}.{frac:0>width$}");
    }
    if r.numer().is_negative() && !whole.is_zero() {
        return Ok(format!("-{text}"));
    }
    Ok(text)
}

/// The integer digits grouped by threes from the right, with a space between groups.
/// L §5.18.
fn group3(digits: &str) -> String {
    let mut groups = Vec::new();
    let mut rest = digits;
    while let Some(cut) = rest.len().checked_sub(3).filter(|c| *c > 0) {
        let Some((head, tail)) = rest.split_at_checked(cut) else {
            break;
        };
        groups.push(tail);
        rest = head;
    }
    groups.push(rest);
    groups.reverse();
    groups.join(" ")
}

/// The significant digits of a terminating value with `places` decimals: a whole number's
/// digits without its trailing zeros, else all the digits of `|r| × 10^places`. L §5.18.
fn significant_digits(r: &BigRat, places: u32) -> Result<i64, NumError> {
    let digits = if places == 0 {
        let text = r.numer().abs_decimal();
        text.trim_end_matches('0').len()
    } else {
        let scaled = r
            .numer()
            .abs()
            .mul(&Big::pow(10, places)?)?
            .div_exact(r.denom())?;
        scaled.abs_decimal().len()
    };
    i64::try_from(digits).map_err(|_| NumError::Invariant)
}

/// The 15-significant-digit reading aid, rounded half to even (refcore's `sig15`). L §5.18.
fn aid(r: &BigRat) -> Result<String, NumError> {
    if r.numer().is_zero() {
        return Ok("0".to_owned());
    }
    let mag = BigRat::new(&r.numer().abs(), r.denom())?;
    let exponent = decimal_exponent(&mag)?;
    let shift = AID_DIGITS
        .checked_sub(1)
        .and_then(|v| v.checked_sub(exponent))
        .ok_or(NumError::Invariant)?;
    // mag × 10^shift as numer / denom
    let (numer, denom) = scaled(&mag, shift)?;
    let mut digits = numer.div_floor(&denom)?;
    let twice_rest = numer.rem_floor(&denom)?.mul(&Big::from_u128(2))?;
    let odd = !digits.rem_floor(&Big::from_u128(2))?.is_zero();
    let round_up = match twice_rest.order(&denom) {
        Ordering::Less => false,
        Ordering::Greater => true,
        Ordering::Equal => odd,
    };
    if round_up {
        digits = digits.add(&Big::from_u128(1))?;
    }
    let places = u32::try_from(shift.max(0)).map_err(|_| NumError::Invariant)?;
    let value = if shift >= 0 {
        BigRat::new(&digits, &Big::pow(10, places)?)?
    } else {
        let up = u32::try_from(shift.checked_neg().ok_or(NumError::Invariant)?)
            .map_err(|_| NumError::Invariant)?;
        BigRat::integer(digits.mul(&Big::pow(10, up)?)?)
    };
    let mut text = fixed(&value, places)?;
    if text.contains('.') {
        text = text.trim_end_matches('0').trim_end_matches('.').to_owned();
    }
    let sign = if r.numer().is_negative() { "-" } else { "" };
    Ok(format!("{sign}{text}"))
}

/// `a × 10^shift` as a numerator and a positive denominator. L §5.18.
fn scaled(a: &BigRat, shift: i64) -> Result<(Big, Big), NumError> {
    let power = u32::try_from(shift.unsigned_abs()).map_err(|_| NumError::Invariant)?;
    let p = Big::pow(10, power)?;
    if shift >= 0 {
        return Ok((a.numer().mul(&p)?, a.denom().clone()));
    }
    Ok((a.numer().clone(), a.denom().mul(&p)?))
}

/// The exponent `e` with `10^e ≤ a < 10^(e+1)` for `a > 0`, from the digit counts and then
/// corrected by comparison (refcore's `dec_exp`). L §5.18.
fn decimal_exponent(a: &BigRat) -> Result<i64, NumError> {
    let n_digits = i64::try_from(a.numer().abs_decimal().len()).map_err(|_| NumError::Invariant)?;
    let d_digits = i64::try_from(a.denom().abs_decimal().len()).map_err(|_| NumError::Invariant)?;
    let mut e = n_digits.checked_sub(d_digits).ok_or(NumError::Invariant)?;
    while power_compare(a, e)? == Ordering::Less {
        e = e.checked_sub(1).ok_or(NumError::Invariant)?;
    }
    while power_compare(a, e.checked_add(1).ok_or(NumError::Invariant)?)? != Ordering::Less {
        e = e.checked_add(1).ok_or(NumError::Invariant)?;
    }
    Ok(e)
}

/// How `a` compares with `10^e`. L §5.18.
fn power_compare(a: &BigRat, e: i64) -> Result<Ordering, NumError> {
    let (n, d) = scaled(a, e.checked_neg().ok_or(NumError::Invariant)?)?;
    Ok(n.order(&d))
}

#[cfg(test)]
mod tests {
    use crate::num::{Currency, Money, Num, NumError};

    fn n(s: &str) -> Num {
        Num::parse_decimal(s).expect("a decimal")
    }

    #[test]
    fn literals_are_read_exactly() {
        assert_eq!(n("0.0976"), Num::ratio(976, 10_000).expect("r"), "0.0976");
        assert_eq!(n("-12.50"), Num::ratio(-25, 2).expect("r"), "-12.50");
        assert_eq!(n("24000"), Num::int(24_000), "24000");
        assert_eq!(n("0"), Num::int(0), "0");
        let long = n("123456789012345678901234567890.0000000000000000000001");
        assert_eq!(
            long.to_text().expect("text"),
            "123 456 789 012 345 678 901 234 567 890.0000000000000000000001 \
             (123 456 789 012 346 000 000 000 000 000)",
            "a long literal round-trips"
        );
        for bad in ["", "-", "1.", ".5", "1.2.3", "1e5", "12a", "+1", "1 000"] {
            assert_eq!(
                Num::parse_decimal(bad),
                Err(NumError::Domain),
                "{bad:?} is no literal"
            );
        }
    }

    #[test]
    fn numbers_print_as_the_spec_shows() {
        // L §5.18: `6 536`, `2.7`, a/b with the aid
        assert_eq!(n("6536").to_text().expect("t"), "6 536", "grouped");
        assert_eq!(n("2.7").to_text().expect("t"), "2.7", "decimal");
        assert_eq!(n("-0.0976").to_text().expect("t"), "-0.0976", "negative");
        assert_eq!(n("1000000").to_text().expect("t"), "1 000 000", "millions");
        assert_eq!(n("123").to_text().expect("t"), "123", "three digits");
        assert_eq!(n("0").to_text().expect("t"), "0", "zero");
        assert_eq!(
            Num::ratio(1, 3).expect("r").to_text().expect("t"),
            "1/3 (0.333333333333333)",
            "a/b and the aid"
        );
        assert_eq!(
            Num::ratio(-2, 3).expect("r").to_text().expect("t"),
            "-2/3 (-0.666666666666667)",
            "negative a/b, the aid rounded"
        );
        assert_eq!(
            Num::ratio(200, 3).expect("r").to_text().expect("t"),
            "200/3 (66.6666666666667)",
            "the aid has 15 significant digits"
        );
        assert_eq!(
            n("0.1234567890123456").to_text().expect("t"),
            "0.1234567890123456 (0.123456789012346)",
            "16 significant digits get the aid"
        );
        assert_eq!(
            n("1234567890123456789").to_text().expect("t"),
            "1 234 567 890 123 456 789 (1 234 567 890 123 460 000)",
            "a whole number's aid"
        );
        assert_eq!(
            n("100000000000000000000").to_text().expect("t"),
            "100 000 000 000 000 000 000",
            "trailing zeros aren't significant"
        );
    }

    #[test]
    fn fixed_places_for_money_and_ints() {
        assert_eq!(n("6064.17").to_fixed(2).expect("t"), "6 064.17", "money");
        assert_eq!(n("-3690").to_fixed(2).expect("t"), "-3 690.00", "padded");
        assert_eq!(
            n("-0.001").to_fixed(2).expect("t"),
            "0.00",
            "no sign on zero digits"
        );
        assert_eq!(n("12").to_fixed(0).expect("t"), "12", "int");
        let pln = Currency::new(0);
        let grosz = n("0.01");
        assert_eq!(
            Money::from_minor(606_417, pln).to_text(&grosz).expect("t"),
            "6 064.17",
            "money prints its minor digits"
        );
        assert_eq!(
            Money::from_minor(-459_990, pln).to_text(&grosz).expect("t"),
            "-4 599.90",
            "negative money"
        );
        assert_eq!(
            Money::from_minor(5, pln).to_text(&n("1")).expect("t"),
            "5",
            "a currency with no minor digits"
        );
    }

    #[test]
    fn decimal_places_count_twos_and_fives() {
        assert_eq!(n("0.01").decimal_places().expect("p"), Some(2), "0.01");
        assert_eq!(n("0.125").decimal_places().expect("p"), Some(3), "1/8");
        assert_eq!(n("7").decimal_places().expect("p"), Some(0), "whole");
        assert_eq!(
            Num::ratio(1, 6).expect("r").decimal_places().expect("p"),
            None,
            "1/6"
        );
    }
}
