//! Units and the unit rules: products of bases to whole powers, with the period words of one dimension.
//! Spec: L §3.2 (units, `%`, year = 4 quarters = 12 months, `day` its own dimension, `$v` bases).
//! Twin: engine/tools/refcore/units.py (`make` to `day_and_period`).
//! Invariants: a unit's terms are sorted by base, by code point, each base once, no zero
//! power; every factor is exact.

use crate::num::{Num, NumError};

/// The period words: year, quarter and month are one dimension, day its own.
pub const PERIODS: [&str; 4] = ["year", "quarter", "month", "day"];

/// The months in a period word of the one dimension, or `None` (`day`, any other base).
fn period_months(base: &str) -> Option<i64> {
    match base {
        "year" => Some(12),
        "quarter" => Some(3),
        "month" => Some(1),
        _ => None,
    }
}

/// A unit: bases to whole powers. The empty unit is 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// The terms, sorted by base, no zero power.
    terms: Vec<(String, i64)>,
}

/// A unit with the factor that brings a value into it.
pub type Scaled = (Unit, Num);

/// `base^power` as a factor: `base` to a whole power, exactly.
fn power_of(base: &Num, power: i64) -> Result<Num, NumError> {
    let mut out = Num::int(1);
    for _ in 0..power.unsigned_abs() {
        out = out.checked_mul(base)?;
    }
    if power < 0 {
        return Num::int(1).checked_div(&out);
    }
    Ok(out)
}

impl Unit {
    /// The unit 1.
    #[must_use]
    pub fn one() -> Unit {
        Unit { terms: Vec::new() }
    }

    /// The unit of these terms: powers of one base summed, zero powers dropped, sorted by
    /// base (refcore's `make`). L §3.2.
    ///
    /// # Errors
    ///
    /// `Domain` if a power passes the 64-bit range.
    pub fn make(terms: Vec<(String, i64)>) -> Result<Unit, NumError> {
        let mut out: Vec<(String, i64)> = Vec::new();
        for (base, power) in terms {
            match out.iter_mut().find(|(b, _)| *b == base) {
                Some(term) => term.1 = term.1.checked_add(power).ok_or(NumError::Domain)?,
                None => out.push((base, power)),
            }
        }
        out.retain(|(_, power)| *power != 0);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Unit { terms: out })
    }

    /// The terms, sorted by base.
    #[must_use]
    pub fn terms(&self) -> &[(String, i64)] {
        &self.terms
    }

    /// Whether it is 1.
    #[must_use]
    pub fn is_one(&self) -> bool {
        self.terms.is_empty()
    }

    /// Unit syntax to a unit and a factor: `%` gives 1/100 per power (refcore's
    /// `from_syntax`). L §3.2.
    ///
    /// # Errors
    ///
    /// A number error of the factor.
    pub fn from_syntax(terms: Vec<(String, i64)>) -> Result<Scaled, NumError> {
        let mut factor = Num::int(1);
        let mut rest = Vec::new();
        let hundredth = Num::ratio(1, 100)?;
        for (base, power) in terms {
            if base == "%" {
                factor = factor.checked_mul(&power_of(&hundredth, power)?)?;
            } else {
                rest.push((base, power));
            }
        }
        let (unit, more) = Unit::make(rest)?.simplify()?;
        Ok((unit, factor.checked_mul(&more)?))
    }

    /// The period bases of the one dimension, in the terms' order.
    fn period_bases(&self) -> Vec<&str> {
        self.terms
            .iter()
            .filter(|(base, _)| period_months(base).is_some())
            .map(|(base, _)| base.as_str())
            .collect()
    }

    /// The period bases of the one dimension brought to one base, the first (refcore's
    /// `simplify`). L §3.2.
    ///
    /// # Errors
    ///
    /// A number error of the factor.
    pub fn simplify(self) -> Result<Scaled, NumError> {
        let first = match self.period_bases().as_slice() {
            [] | [_] => None,
            [first, ..] => Some((*first).to_owned()),
        };
        match first {
            None => Ok((self, Num::int(1))),
            Some(target) => self.to_base(&target),
        }
    }

    /// Every period base of the one dimension as `target` (refcore's `_to_base`).
    fn to_base(&self, target: &str) -> Result<Scaled, NumError> {
        let target_months = period_months(target).ok_or(NumError::Invariant)?;
        let mut factor = Num::int(1);
        let mut out = Vec::new();
        for (base, power) in &self.terms {
            match period_months(base) {
                Some(months) if base != target => {
                    let ratio = Num::ratio(months, target_months)?;
                    factor = factor.checked_mul(&power_of(&ratio, *power)?)?;
                    out.push((target.to_owned(), *power));
                }
                Some(_) | None => out.push((base.clone(), *power)),
            }
        }
        Ok((Unit::make(out)?, factor))
    }

    /// The product of two units, and the factor its periods bring (refcore's `mul`). L §3.2.
    ///
    /// # Errors
    ///
    /// A number error of the factor.
    pub fn mul(&self, other: &Unit) -> Result<Scaled, NumError> {
        let mine = self.period_bases();
        let theirs = other.period_bases();
        let mut terms = self.terms.clone();
        terms.extend(other.terms.iter().cloned());
        let unit = Unit::make(terms)?;
        if let (Some(target), Some(_)) = (mine.first(), theirs.first()) {
            let one_base = mine.iter().chain(theirs.iter()).all(|b| b == target);
            if !one_base {
                return unit.to_base(target);
            }
        }
        unit.simplify()
    }

    /// The reciprocal (refcore's `inv`).
    ///
    /// # Errors
    ///
    /// `Domain` for a power of `i64::MIN`, which no literal writes.
    pub fn inv(&self) -> Result<Unit, NumError> {
        let mut terms = Vec::with_capacity(self.terms.len());
        for (base, power) in &self.terms {
            terms.push((base.clone(), power.checked_neg().ok_or(NumError::Domain)?));
        }
        Ok(Unit { terms })
    }

    /// The quotient (refcore's `div`). L §3.2.
    ///
    /// # Errors
    ///
    /// A number error of the factor.
    pub fn div(&self, other: &Unit) -> Result<Scaled, NumError> {
        self.mul(&other.inv()?)
    }

    /// The unit to the `n`-th power (refcore's `power`). L §3.2.
    ///
    /// # Errors
    ///
    /// `Domain` if a power passes the 64-bit range.
    pub fn power(&self, n: i64) -> Result<Unit, NumError> {
        let mut terms = Vec::with_capacity(self.terms.len());
        for (base, power) in &self.terms {
            terms.push((base.clone(), power.checked_mul(n).ok_or(NumError::Domain)?));
        }
        Unit::make(terms)
    }

    /// The unit with its periods of the one dimension as months, and the factor (refcore's
    /// `canon`).
    fn canon(&self) -> Result<Scaled, NumError> {
        let mut factor = Num::int(1);
        let mut out = Vec::new();
        for (base, power) in &self.terms {
            match period_months(base) {
                Some(months) => {
                    factor = factor.checked_mul(&power_of(&Num::int(months), *power)?)?;
                    out.push(("month".to_owned(), *power));
                }
                None => out.push((base.clone(), *power)),
            }
        }
        Ok((Unit::make(out)?, factor))
    }

    /// The factor `f` with `x [self] = x * f [to]`, or `None` when the units differ
    /// (refcore's `conv`). L §3.2.
    ///
    /// # Errors
    ///
    /// A number error of the factor.
    pub fn conv(&self, to: &Unit) -> Result<Option<Num>, NumError> {
        if self == to {
            return Ok(Some(Num::int(1)));
        }
        let (from_canon, from_factor) = self.canon()?;
        let (to_canon, to_factor) = to.canon()?;
        if from_canon != to_canon {
            return Ok(None);
        }
        Ok(Some(from_factor.checked_div(&to_factor)?))
    }

    /// Whether it is one currency to the first power (refcore's `is_currency`).
    #[must_use]
    pub fn is_currency(&self) -> bool {
        matches_one_currency(&self.terms)
    }

    /// Its `$v` bases replaced by the keys `env` gives (refcore's `subst`). L §3.2.
    ///
    /// # Errors
    ///
    /// `Domain` if a power passes the 64-bit range.
    pub fn subst(&self, env: &[(String, String)]) -> Result<Unit, NumError> {
        let mut terms = Vec::with_capacity(self.terms.len());
        for (base, power) in &self.terms {
            let key = base.strip_prefix('$').and_then(|var| key_of(env, var));
            terms.push((key.unwrap_or_else(|| base.clone()), *power));
        }
        Unit::make(terms)
    }

    /// For a rate unit `1/P`: the period word `P` (refcore's `period_of`). L §3.3.
    #[must_use]
    pub fn period_of(&self) -> Option<&str> {
        match self.terms.as_slice() {
            [(base, -1)] => PERIODS.into_iter().find(|p| p == base),
            _ => None,
        }
    }

    /// The unit as it prints: `PLN*etf_unit/month^2`, `1/year` (refcore's `fmt`). L §3.2.
    #[must_use]
    pub fn text(&self) -> String {
        if self.terms.is_empty() {
            return String::new();
        }
        let term = |base: &str, power: u64| {
            if power == 1 {
                base.to_owned()
            } else {
                format!("{base}^{power}")
            }
        };
        let positive: Vec<String> = self
            .terms
            .iter()
            .filter(|(_, power)| *power > 0)
            .map(|(base, power)| term(base, power.unsigned_abs()))
            .collect();
        let mut out = if positive.is_empty() {
            "1".to_owned()
        } else {
            positive.join("*")
        };
        for (base, power) in self.terms.iter().filter(|(_, power)| *power < 0) {
            out.push('/');
            out.push_str(&term(base, power.unsigned_abs()));
        }
        out
    }

    /// Whether it mixes `day` with year, quarter or month (refcore's `day_and_period`).
    /// L §3.2.
    #[must_use]
    pub fn day_and_period(&self) -> bool {
        let day = self.terms.iter().any(|(base, _)| base == "day");
        day && self
            .terms
            .iter()
            .any(|(base, _)| period_months(base).is_some())
    }
}

/// The key `env` gives the bound variable `var`.
fn key_of(env: &[(String, String)], var: &str) -> Option<String> {
    env.iter()
        .find(|(v, _)| v == var)
        .map(|(_, key)| key.clone())
}

/// Whether the terms are one currency to the first power: three capital letters.
fn matches_one_currency(terms: &[(String, i64)]) -> bool {
    match terms {
        [(base, 1)] => is_currency_code(base),
        _ => false,
    }
}

/// Whether `base` is written as a currency code: three capital letters (L §2.1).
#[must_use]
pub fn is_currency_code(base: &str) -> bool {
    base.len() == 3 && base.bytes().all(|b| b.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::Unit;
    use crate::num::Num;

    fn unit(terms: &[(&str, i64)]) -> Unit {
        let terms = terms.iter().map(|(b, p)| ((*b).to_owned(), *p)).collect();
        Unit::make(terms).expect("a unit")
    }

    #[test]
    fn units_sort_and_cancel() {
        assert_eq!(
            unit(&[("month", -1), ("PLN", 1)]).text(),
            "PLN/month",
            "sorted"
        );
        assert_eq!(unit(&[("PLN", 1), ("PLN", -1)]), Unit::one(), "cancels");
        assert_eq!(unit(&[("year", -1)]).text(), "1/year", "a rate");
        assert_eq!(
            unit(&[("year", -1)]).period_of(),
            Some("year"),
            "its period"
        );
        assert_eq!(Unit::one().text(), "", "1 prints empty");
    }

    #[test]
    fn periods_are_one_dimension() {
        let per_year = unit(&[("PLN", 1), ("year", -1)]);
        let per_month = unit(&[("PLN", 1), ("month", -1)]);
        assert_eq!(
            per_year.conv(&per_month),
            Ok(Some(Num::ratio(1, 12).expect("1/12"))),
            "a year is 12 months"
        );
        let (u, f) = unit(&[("year", 1)])
            .mul(&unit(&[("month", -1)]))
            .expect("mul");
        assert_eq!((u, f), (Unit::one(), Num::int(12)), "year/month is 12");
        let (pct, hundredth) = Unit::from_syntax(vec![("%".to_owned(), 1)]).expect("%");
        assert_eq!(
            (pct, hundredth),
            (Unit::one(), Num::ratio(1, 100).expect("1/100")),
            "% folds"
        );
        assert!(
            unit(&[("day", -1), ("year", 1)]).day_and_period(),
            "day and year"
        );
        assert_eq!(
            unit(&[("PLN", 1)]).conv(&unit(&[("EUR", 1)])),
            Ok(None),
            "differ"
        );
    }
}
