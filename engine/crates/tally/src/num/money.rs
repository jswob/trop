//! Money: a number of a currency's minor units, whole but at `solve`'s centre, in 64 bits or big.
//! Spec: L §3.1 (`money CUR`), §5.4 (a currency's minor unit), §7.1, §7.2 (back to money).
//! Twin: engine/tools/refcore/values.py (`Num` with `money`, `Money`).
//! Invariants: always the smallest tier that fits, a whole number never a `Part`; only
//! `to_money_part` makes a `Part` (`solve`'s centre, L §5.18 step 3); two amounts meet only in one currency
//! (a mismatch is an invariant break, since load checks units); the cap is the value's, its
//! reduced fraction's digits, never the minor units' (L §7.1); no operator, `Default`, `Sum`
//! or `From`, so the way from a number to money is a `round` (R-28).

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

/// A number of minor units: a whole one in a machine word, or big when it doesn't fit (K8-3);
/// or, at `solve`'s centre alone, a fraction of one (L §5.18 step 3; pick P36).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Units {
    /// Fits an `i64` (never `i64::MIN`, so its negation fits too).
    Word(i64),
    /// Too large for a word.
    Big(Box<Big>),
    /// Not a whole number: money that holds a fraction of its minor unit, as `solve`'s
    /// centre a + u/2 does, its value under the cap (L §5.18 step 3, §7.1).
    Part(Box<Num>),
}

/// An amount of money: a number of minor units of one currency, whole but at `solve`'s centre
/// (L §3.1, §5.18 step 3).
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
            Units::Part(_) => false,
        }
    }

    /// Whether the amount is below 0. L §3.1.
    pub fn is_negative(&self) -> bool {
        match &self.units {
            Units::Word(v) => *v < 0,
            Units::Big(b) => b.is_negative(),
            Units::Part(q) => q.is_negative(),
        }
    }

    /// `self + o`, in a currency whose minor unit is `minor`: one checked add of minor units,
    /// no gcd (D2-2), the cap counted on the sum's value. L §7.1.
    pub fn checked_add(&self, o: &Money, minor: &Num) -> Result<Money, NumError> {
        let cur = self.same_currency(o)?;
        if matches!(self.units, Units::Part(_)) || matches!(o.units, Units::Part(_)) {
            let sum = self.minor_units().checked_add(&o.minor_units())?;
            return sum.checked_mul(minor)?.to_money_part(minor, cur);
        }
        if let (Units::Word(a), Units::Word(b)) = (&self.units, &o.units) {
            let sum = i128::from(*a)
                .checked_add(i128::from(*b))
                .ok_or(NumError::Invariant)?;
            return Ok(Money {
                units: Units::from_i128(sum),
                cur,
            });
        }
        Money::from_big(self.units.to_big()?.add(&o.units.to_big()?)?, cur, minor)
    }

    /// `self - o`, in a currency whose minor unit is `minor`. L §7.1.
    pub fn checked_sub(&self, o: &Money, minor: &Num) -> Result<Money, NumError> {
        self.checked_add(&o.checked_neg()?, minor)
    }

    /// `-self`: the same digits, so no cap to meet. L §7.1.
    pub fn checked_neg(&self) -> Result<Money, NumError> {
        match &self.units {
            Units::Word(v) => Ok(Money {
                units: Units::Word(v.checked_neg().ok_or(NumError::Invariant)?),
                cur: self.cur,
            }),
            Units::Big(b) => Ok(Money::held(b.neg()?, self.cur)),
            Units::Part(q) => Ok(Money {
                units: Units::Part(Box::new(q.checked_neg()?)),
                cur: self.cur,
            }),
        }
    }

    /// The order of two amounts of one currency. L §7.1.
    pub fn compare(&self, o: &Money) -> Result<Ordering, NumError> {
        self.same_currency(o)?;
        match (&self.units, &o.units) {
            (Units::Part(_), _) | (_, Units::Part(_)) => {
                self.minor_units().compare(&o.minor_units())
            }
            (Units::Word(a), Units::Word(b)) => Ok(a.cmp(b)),
            (Units::Word(_) | Units::Big(_), Units::Big(_)) | (Units::Big(_), Units::Word(_)) => {
                Ok(self.units.to_big()?.order(&o.units.to_big()?))
            }
        }
    }

    /// The amount as a number: its minor units times the currency's minor unit `minor`.
    /// L §3.2 (money is a `num CUR` too).
    pub fn to_num(&self, minor: &Num) -> Result<Num, NumError> {
        self.minor_units().checked_mul(minor)
    }

    /// The minor units: a whole number, but at `solve`'s centre. L §3.1, §5.18 step 3.
    pub fn minor_units(&self) -> Num {
        match &self.units {
            Units::Word(v) => Num::int(*v),
            Units::Big(b) => Num(Repr::Big(Box::new(BigRat::integer(Big::clone(b))))),
            Units::Part(q) => Num::clone(q),
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

    /// Money from big minor units of a currency whose minor unit is `minor`, capped as its
    /// value: "the digits are counted on the reduced fraction" (L §7.1), not on the minor
    /// units, which may have more.
    fn from_big(units: Big, cur: Currency, minor: &Num) -> Result<Money, NumError> {
        let value = BigRat::integer(Big::clone(&units)).mul(&minor.to_big())?;
        check_cap(value.numer())?;
        check_cap(value.denom())?;
        Ok(Money::held(units, cur))
    }

    /// Money from big minor units already under the cap, held in a word when it fits. L §3.1.
    fn held(units: Big, cur: Currency) -> Money {
        let units = match units.to_i128() {
            Some(v) => Units::from_i128(v),
            None => Units::Big(Box::new(units)),
        };
        Money { units, cur }
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

    /// The whole units as a big integer; a `Part`, which isn't whole, is an invariant break
    /// here, since its callers take it first. L §3.1.
    fn to_big(&self) -> Result<Big, NumError> {
        match self {
            Units::Word(v) => Ok(Big::from_i128(i128::from(*v))),
            Units::Big(b) => Ok(Big::clone(b)),
            Units::Part(_) => Err(NumError::Invariant),
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
        match self.round(step, mode)?.to_money_in(minor, cur) {
            Err(NumError::Domain) => Err(NumError::Invariant),
            money => money,
        }
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
            Repr::Big(b) => Ok(Money::held(b.numer().clone(), cur)),
        }
    }

    /// The value `self` as money of `cur`, whose minor unit is `minor`: its whole number of
    /// minor units, with no cap of their own, since the cap counts the value's reduced
    /// fraction (L §7.1), as `self` is; `Domain` where it isn't a whole number of them
    /// (L §3.1, §7.2).
    pub fn to_money_in(&self, minor: &Num, cur: Currency) -> Result<Money, NumError> {
        let units = match self.checked_div(minor) {
            Ok(units) if units.is_integer() => return units.to_money(cur),
            Ok(_) => return Err(NumError::Domain),
            // minor units past the cap, of a value under it
            Err(NumError::Digits) => self.to_big().mul(&minor.to_big().recip()?)?,
            Err(other) => return Err(other),
        };
        if !whole(&units) {
            return Err(NumError::Domain);
        }
        Ok(Money::held(units.numer().clone(), cur))
    }

    /// The value `self` as money of `cur`, whose minor unit is `minor`, exactly: whole minor
    /// units as `to_money_in` gives them, else a `Part` holding the fraction of one. Only
    /// `solve`'s centre a + u/2 takes a value off the minor grid (L §5.18 step 3; pick P36).
    ///
    /// # Errors
    ///
    /// A number error of the division.
    pub fn to_money_part(&self, minor: &Num, cur: Currency) -> Result<Money, NumError> {
        match self.to_money_in(minor, cur) {
            Err(NumError::Domain) => Ok(Money {
                units: Units::Part(Box::new(self.checked_div(minor)?)),
                cur,
            }),
            money => money,
        }
    }

    /// Whether the value `self` is a whole number of minor units `minor`, those units counted
    /// with no cap of their own (L §3.1, §7.1).
    pub fn is_whole_in(&self, minor: &Num) -> Result<bool, NumError> {
        match self.checked_div(minor) {
            Ok(units) => Ok(units.is_integer()),
            Err(NumError::Digits) => Ok(whole(&self.to_big().mul(&minor.to_big().recip()?)?)),
            Err(other) => Err(other),
        }
    }
}

/// Whether a big fraction is a whole number. L §7.1.
fn whole(r: &BigRat) -> bool {
    r.denom().order(&Big::from_u128(1)) == Ordering::Equal
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
            a.checked_add(&b, &grosz()).expect("sum"),
            Money::from_minor(-125, PLN),
            "sum"
        );
        assert_eq!(
            a.checked_sub(&b, &grosz()).expect("difference"),
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
        assert_eq!(
            a.checked_add(&b, &grosz()),
            Err(NumError::Invariant),
            "PLN + EUR"
        );
        assert_eq!(a.compare(&b), Err(NumError::Invariant), "PLN vs EUR");
    }

    #[test]
    fn money_widens_and_demotes() {
        let max = Money::from_minor(i64::MAX, PLN);
        let one = Money::from_minor(1, PLN);
        let past = max.checked_add(&one, &grosz()).expect("past a word");
        assert!(matches!(past.units, Units::Big(_)), "big");
        let back = past.checked_sub(&one, &grosz()).expect("back");
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
    fn the_cap_counts_the_value_not_its_minor_units() {
        // L §7.1: 6·10^4094 PLN has 4 095 digits, its minor units 4 097
        let times_ten_to = |k: i64, n: i64| {
            Num::int(n)
                .checked_mul(&Num::int(10).checked_pow(k).expect("a power"))
                .expect("a value under the cap")
        };
        let big = times_ten_to(4094, 6);
        let money = big.to_money_in(&grosz(), PLN).expect("under the cap");
        assert_eq!(
            money.to_num(&grosz()),
            Ok(big.clone()),
            "the same value back"
        );
        assert!(money.checked_neg().is_ok(), "its negation too");
        let nine = times_ten_to(4095, 9);
        let most = nine
            .to_money_in(&grosz(), PLN)
            .expect("9e4095: 4 096 digits");
        let sum = most
            .checked_add(&money, &grosz())
            .expect("9.06e4095: 4 096 digits");
        assert!(sum.to_num(&grosz()).is_ok(), "9.06e4095 PLN as a number");
        assert_eq!(
            most.checked_add(&most, &grosz()),
            Err(NumError::Digits),
            "1.8e4096 PLN: 4 097 digits"
        );
    }

    #[test]
    fn solves_centre_holds_part_of_a_minor_unit() {
        // L §5.18 step 3: the centre a + u/2 of 1.04 and 1.05 PLN is 1.045 PLN, exactly
        let half = Num::parse_decimal("1.045").expect("1.045");
        let centre = half.to_money_part(&grosz(), PLN).expect("a part");
        assert!(matches!(centre.units, Units::Part(_)), "a part of a grosz");
        assert!(!centre.is_zero(), "not 0");
        assert!(!centre.is_negative(), "not below 0");
        assert_eq!(centre.to_num(&grosz()), Ok(half.clone()), "1.045 back");
        assert_eq!(
            centre.to_text(&grosz()).expect("text"),
            "1.045",
            "printed exactly"
        );
        let tax = Money::from_minor(19, PLN);
        let net = centre.checked_sub(&tax, &grosz()).expect("net");
        assert_eq!(
            net.to_num(&grosz()),
            Ok(Num::parse_decimal("0.855").expect("0.855")),
            "1.045 - 0.19"
        );
        assert_eq!(
            net.compare(&tax).expect("cmp"),
            Ordering::Greater,
            "0.855 > 0.19"
        );
        assert_eq!(
            tax.compare(&net).expect("cmp"),
            Ordering::Less,
            "0.19 < 0.855"
        );
        let back = centre.checked_add(&centre, &grosz()).expect("2.09");
        assert_eq!(
            back,
            Money::from_minor(209, PLN),
            "whole again: a word, not a part"
        );
        let neg = centre.checked_neg().expect("neg");
        assert!(neg.is_negative(), "-1.045 is below 0");
        assert_eq!(
            Num::int(2).to_money_part(&grosz(), PLN),
            Ok(Money::from_minor(200, PLN)),
            "a whole value is never a part"
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
