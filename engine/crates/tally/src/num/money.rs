//! Money: a whole number of a currency's minor units, held in 64 bits or big.
//! Spec: L §3.1 (`money CUR`), §5.4 (a currency's minor unit), §7.1, §7.2 (back to money).
//! Twin: engine/tools/refcore/values.py (`Num` with `money`, `Money`).
//! Invariants: always the smallest tier that fits; two amounts meet only in one currency
//! (a mismatch is an invariant break, since load checks units); no operator, `Default`,
//! `Sum` or `From`, so the way from a number to money is a `round` (R-28).

use super::big::{Big, BigRat};
use super::rational::{Num, NumError, Repr, check_cap};
use super::round::Mode;
use core::cmp::Ordering;

/// A currency, by the id load gives its declaration; its minor unit is kept with that
/// declaration (L §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Currency(u32);

impl Currency {
    /// The currency with declaration id `id`. L §5.4.
    pub fn new(id: u32) -> Currency {
        Currency(id)
    }

    /// The declaration id. L §5.4.
    pub fn id(self) -> u32 {
        self.0
    }
}

/// A whole number of minor units: one machine word, or big when it doesn't fit (K8-3).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Units {
    /// Fits an `i64` (never `i64::MIN`, so its negation fits too).
    Word(i64),
    /// Too large for a word.
    Big(Box<Big>),
}

/// An amount of money: a whole number of minor units of one currency (L §3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    /// The minor units.
    units: Units,
    /// The currency.
    cur: Currency,
}

impl Money {
    /// `units` minor units of `cur`. L §3.1.
    pub fn from_minor(units: i64, cur: Currency) -> Money {
        Money {
            units: Units::from_i128(i128::from(units)),
            cur,
        }
    }

    /// The currency. L §3.1.
    pub fn currency(&self) -> Currency {
        self.cur
    }

    /// Whether the amount is 0. L §3.1.
    pub fn is_zero(&self) -> bool {
        match &self.units {
            Units::Word(v) => *v == 0,
            Units::Big(b) => b.is_zero(),
        }
    }

    /// Whether the amount is below 0. L §3.1.
    pub fn is_negative(&self) -> bool {
        match &self.units {
            Units::Word(v) => *v < 0,
            Units::Big(b) => b.is_negative(),
        }
    }

    /// `self + o`: one checked add of minor units, no gcd (D2-2). L §7.1.
    pub fn checked_add(&self, o: &Money) -> Result<Money, NumError> {
        let cur = self.same_currency(o)?;
        if let (Units::Word(a), Units::Word(b)) = (&self.units, &o.units) {
            let sum = i128::from(*a)
                .checked_add(i128::from(*b))
                .ok_or(NumError::Invariant)?;
            return Ok(Money {
                units: Units::from_i128(sum),
                cur,
            });
        }
        Money::from_big(self.units.to_big().add(&o.units.to_big())?, cur)
    }

    /// `self - o`. L §7.1.
    pub fn checked_sub(&self, o: &Money) -> Result<Money, NumError> {
        self.checked_add(&o.checked_neg()?)
    }

    /// `-self`. L §7.1.
    pub fn checked_neg(&self) -> Result<Money, NumError> {
        match &self.units {
            Units::Word(v) => Ok(Money {
                units: Units::Word(v.checked_neg().ok_or(NumError::Invariant)?),
                cur: self.cur,
            }),
            Units::Big(b) => Money::from_big(b.neg()?, self.cur),
        }
    }

    /// The order of two amounts of one currency. L §7.1.
    pub fn compare(&self, o: &Money) -> Result<Ordering, NumError> {
        self.same_currency(o)?;
        match (&self.units, &o.units) {
            (Units::Word(a), Units::Word(b)) => Ok(a.cmp(b)),
            (Units::Word(_) | Units::Big(_), Units::Big(_)) | (Units::Big(_), Units::Word(_)) => {
                Ok(self.units.to_big().order(&o.units.to_big()))
            }
        }
    }

    /// The amount as a number: its minor units times the currency's minor unit `minor`.
    /// L §3.2 (money is a `num CUR` too).
    pub fn to_num(&self, minor: &Num) -> Result<Num, NumError> {
        self.minor_units().checked_mul(minor)
    }

    /// The minor units as a whole number. L §3.1.
    pub fn minor_units(&self) -> Num {
        match &self.units {
            Units::Word(v) => Num::int(*v),
            Units::Big(b) => Num(Repr::Big(Box::new(BigRat::integer(Big::clone(b))))),
        }
    }

    /// The one currency of two amounts; a mismatch is an invariant break, since load checks
    /// units (L §3.2).
    fn same_currency(&self, o: &Money) -> Result<Currency, NumError> {
        if self.cur == o.cur {
            return Ok(self.cur);
        }
        Err(NumError::Invariant)
    }

    /// Money from big minor units, capped and held in a word when it fits. L §7.1.
    fn from_big(units: Big, cur: Currency) -> Result<Money, NumError> {
        check_cap(&units)?;
        let units = match units.to_i128() {
            Some(v) => Units::from_i128(v),
            None => Units::Big(Box::new(units)),
        };
        Ok(Money { units, cur })
    }
}

impl Units {
    /// The smallest tier for `v`. L §3.1.
    fn from_i128(v: i128) -> Units {
        match i64::try_from(v) {
            Ok(w) if w != i64::MIN => Units::Word(w),
            Ok(_) | Err(_) => Units::Big(Box::new(Big::from_i128(v))),
        }
    }

    /// The units as a big integer. L §3.1.
    fn to_big(&self) -> Big {
        match self {
            Units::Word(v) => Big::from_i128(i128::from(*v)),
            Units::Big(b) => Big::clone(b),
        }
    }
}

impl Num {
    /// `round(self, step, mode)` as money of `cur`, whose minor unit is `minor`: the only way
    /// from a number to money (L §7.2). The step is a whole number of minor units, which
    /// load checks; a result that isn't is an invariant break.
    pub fn round_to_money(
        &self,
        step: &Num,
        mode: Mode,
        cur: Currency,
        minor: &Num,
    ) -> Result<Money, NumError> {
        let units = self.round(step, mode)?.checked_div(minor)?;
        units.to_money(cur)
    }

    /// A whole number of minor units as money of `cur`; a fraction is an invariant break.
    /// L §3.1.
    pub fn to_money(&self, cur: Currency) -> Result<Money, NumError> {
        if !self.is_integer() {
            return Err(NumError::Invariant);
        }
        match &self.0 {
            Repr::Small(s) => Ok(Money {
                units: Units::Word(s.n),
                cur,
            }),
            Repr::Big(b) => Money::from_big(b.numer().clone(), cur),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Currency, Money, Units};
    use crate::num::{Mode, Num, NumError};
    use core::cmp::Ordering;

    const PLN: Currency = Currency(0);
    const EUR: Currency = Currency(1);

    fn grosz() -> Num {
        Num::parse_decimal("0.01").expect("0.01")
    }

    #[test]
    fn money_adds_in_minor_units() {
        let a = Money::from_minor(150, PLN);
        let b = Money::from_minor(-275, PLN);
        assert_eq!(
            a.checked_add(&b).expect("sum"),
            Money::from_minor(-125, PLN),
            "sum"
        );
        assert_eq!(
            a.checked_sub(&b).expect("difference"),
            Money::from_minor(425, PLN),
            "difference"
        );
        assert_eq!(
            a.to_num(&grosz()).expect("num"),
            Num::parse_decimal("1.5").expect("1.5"),
            "1.50 PLN as a number"
        );
        assert_eq!(a.compare(&b).expect("cmp"), Ordering::Greater, "order");
    }

    #[test]
    fn currencies_never_mix() {
        let a = Money::from_minor(1, PLN);
        let b = Money::from_minor(1, EUR);
        assert_eq!(a.checked_add(&b), Err(NumError::Invariant), "PLN + EUR");
        assert_eq!(a.compare(&b), Err(NumError::Invariant), "PLN vs EUR");
    }

    #[test]
    fn money_widens_and_demotes() {
        let max = Money::from_minor(i64::MAX, PLN);
        let one = Money::from_minor(1, PLN);
        let past = max.checked_add(&one).expect("past a word");
        assert!(matches!(past.units, Units::Big(_)), "big");
        let back = past.checked_sub(&one).expect("back");
        assert_eq!(back, max, "demoted to a word");
        assert_eq!(
            Money::from_minor(i64::MIN, PLN)
                .checked_neg()
                .expect("neg")
                .compare(&max)
                .expect("cmp"),
            Ordering::Greater,
            "-i64::MIN is past i64::MAX"
        );
    }

    #[test]
    fn rounding_is_the_way_to_money() {
        // L §7.2: round(x, minor(PLN), half-up) gives money; 153.335 is a tie
        let x = Num::parse_decimal("153.335").expect("x");
        let m = x
            .round_to_money(&grosz(), Mode::HalfUp, PLN, &grosz())
            .expect("money");
        assert_eq!(m, Money::from_minor(15_334, PLN), "153.34 PLN");
        let whole = x
            .round_to_money(&Num::int(1), Mode::Down, PLN, &grosz())
            .expect("money");
        assert_eq!(whole, Money::from_minor(15_300, PLN), "whole złoty");
        assert_eq!(
            Num::ratio(1, 3).expect("1/3").to_money(PLN),
            Err(NumError::Invariant),
            "a fraction of a minor unit is no money"
        );
    }
}
