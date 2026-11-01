//! Rounding to a step by the seven modes of L §7.2, written once for every caller.
//! Spec: L §7.2 (rounding).
//! Twin: engine/tools/refcore/evaluate.py (`round_mode`).
//! Invariants: the step is positive; the result is a whole multiple of the step; a value
//! already on the grid is returned as it is, whatever the mode.

use super::big::Big;
use super::rational::{Num, NumError, Repr};
use super::small;
use core::cmp::Ordering;

/// A rounding mode of L §7.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Toward zero (truncation).
    Down,
    /// Away from zero.
    Up,
    /// Toward −∞.
    Floor,
    /// Toward +∞.
    Ceil,
    /// The nearest; a tie goes away from zero.
    HalfUp,
    /// The nearest; a tie goes toward zero.
    HalfDown,
    /// The nearest; a tie goes to the even multiple.
    HalfEven,
}

impl Mode {
    /// The mode a word of the language names (`half-up`), or `None`. L §7.2.
    pub fn from_word(word: &str) -> Option<Mode> {
        match word {
            "down" => Some(Mode::Down),
            "up" => Some(Mode::Up),
            "floor" => Some(Mode::Floor),
            "ceil" => Some(Mode::Ceil),
            "half-up" => Some(Mode::HalfUp),
            "half-down" => Some(Mode::HalfDown),
            "half-even" => Some(Mode::HalfEven),
            _ => None,
        }
    }

    /// The word of the language for the mode. L §7.2.
    pub fn word(self) -> &'static str {
        match self {
            Mode::Down => "down",
            Mode::Up => "up",
            Mode::Floor => "floor",
            Mode::Ceil => "ceil",
            Mode::HalfUp => "half-up",
            Mode::HalfDown => "half-down",
            Mode::HalfEven => "half-even",
        }
    }
}

/// Where a value lies between two whole numbers: its floor, how the rest compares with one
/// half, and whether there is no rest.
struct Parts {
    /// The floor of the value.
    floor: Num,
    /// The fractional part against one half.
    half: Ordering,
    /// Whether the value is a whole number.
    exact: bool,
}

impl Num {
    /// `round(self, step, mode)`: the multiple of `step` that `mode` chooses; `RUN-DOMAIN`
    /// when the step isn't positive. L §7.2.
    pub fn round(&self, step: &Num, mode: Mode) -> Result<Num, NumError> {
        if step.is_zero() || step.is_negative() {
            return Err(NumError::Domain);
        }
        let k = self.checked_div(step)?.round_whole(mode)?;
        k.checked_mul(step)
    }

    /// The whole number `mode` chooses for the value (`round_mode` of refcore). L §7.2.
    fn round_whole(&self, mode: Mode) -> Result<Num, NumError> {
        let p = self.parts()?;
        if p.exact {
            return Ok(p.floor);
        }
        // not whole, so the value is positive exactly when its floor is 0 or more
        let (away, toward) = if p.floor.is_negative() {
            (Pick::Floor, Pick::Above)
        } else {
            (Pick::Above, Pick::Floor)
        };
        let pick = match mode {
            Mode::Floor => Pick::Floor,
            Mode::Ceil => Pick::Above,
            Mode::Down => toward,
            Mode::Up => away,
            Mode::HalfUp => nearest(p.half, away),
            Mode::HalfDown => nearest(p.half, toward),
            Mode::HalfEven => nearest(p.half, p.floor.even_side()?),
        };
        match pick {
            Pick::Floor => Ok(p.floor),
            Pick::Above => p.floor.checked_add(&Num::int(1)),
        }
    }

    /// The floor and the fractional part's place against one half (Python's `//` and `%`).
    /// L §7.2.
    fn parts(&self) -> Result<Parts, NumError> {
        match &self.0 {
            Repr::Small(s) => {
                let (floor, half, exact) = small::floor_parts(*s)?;
                Ok(Parts {
                    floor: Num::int(floor),
                    half,
                    exact,
                })
            }
            Repr::Big(b) => {
                let floor = b.numer().div_floor(b.denom())?;
                let rest = b.numer().rem_floor(b.denom())?;
                let twice = rest.add(&rest)?;
                Ok(Parts {
                    floor: Num::from_big(super::big::BigRat::integer(floor))?,
                    half: twice.order(b.denom()),
                    exact: rest.is_zero(),
                })
            }
        }
    }

    /// For a whole number, the side `half-even` takes at a tie: itself when even, the number
    /// above when odd. L §7.2 (`half-even`).
    fn even_side(&self) -> Result<Pick, NumError> {
        let even = match &self.0 {
            Repr::Small(s) => s.n.checked_rem_euclid(2) == Some(0),
            Repr::Big(b) => b.numer().rem_floor(&Big::from_u128(2))?.is_zero(),
        };
        Ok(if even { Pick::Floor } else { Pick::Above })
    }
}

/// Which of the two whole numbers around a value that isn't whole rounding takes. L §7.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pick {
    /// The floor, the whole number below.
    Floor,
    /// The whole number above the floor.
    Above,
}

/// The pick of a nearest mode: by the fractional part against one half, a tie going to
/// `tie`. L §7.2.
fn nearest(half: Ordering, tie: Pick) -> Pick {
    match half {
        Ordering::Less => Pick::Floor,
        Ordering::Greater => Pick::Above,
        Ordering::Equal => tie,
    }
}

#[cfg(test)]
mod tests {
    use super::Mode;
    use crate::num::Num;

    fn n(s: &str) -> Num {
        Num::parse_decimal(s).expect("a decimal")
    }

    const MODES: [Mode; 7] = [
        Mode::Down,
        Mode::Up,
        Mode::Floor,
        Mode::Ceil,
        Mode::HalfUp,
        Mode::HalfDown,
        Mode::HalfEven,
    ];

    /// Each row: the value, then its rounding to a whole number by the modes in `MODES`'
    /// order. L §7.2's table, on ties and on negatives (BUILDER §2).
    const TABLE: [(&str, [&str; 7]); 10] = [
        ("2.5", ["2", "3", "2", "3", "3", "2", "2"]),
        ("3.5", ["3", "4", "3", "4", "4", "3", "4"]),
        ("-2.5", ["-2", "-3", "-3", "-2", "-3", "-2", "-2"]),
        ("-3.5", ["-3", "-4", "-4", "-3", "-4", "-3", "-4"]),
        ("2.4", ["2", "3", "2", "3", "2", "2", "2"]),
        ("2.6", ["2", "3", "2", "3", "3", "3", "3"]),
        ("-2.4", ["-2", "-3", "-3", "-2", "-2", "-2", "-2"]),
        ("-2.6", ["-2", "-3", "-3", "-2", "-3", "-3", "-3"]),
        ("-0.5", ["0", "-1", "-1", "0", "-1", "0", "0"]),
        ("7", ["7", "7", "7", "7", "7", "7", "7"]),
    ];

    #[test]
    fn every_mode_on_ties_and_negatives() {
        for (value, expected) in TABLE {
            for (mode, want) in MODES.iter().zip(expected) {
                assert_eq!(
                    n(value).round(&Num::int(1), *mode).expect("rounds"),
                    n(want),
                    "{value} {}",
                    mode.word()
                );
            }
        }
    }

    #[test]
    fn steps_other_than_one() {
        // L §7.2: round(x, minor(PLN), half-up); 5 Rappen; a step of 1000
        assert_eq!(
            n("153.333")
                .round(&n("0.01"), Mode::HalfUp)
                .expect("rounds"),
            n("153.33"),
            "to the grosz"
        );
        assert_eq!(
            n("0.125")
                .round(&n("0.05"), Mode::HalfEven)
                .expect("rounds"),
            n("0.1"),
            "0.125 is 2.5 steps of 0.05, so half-even gives 2 steps"
        );
        assert_eq!(
            n("1500").round(&n("1000"), Mode::HalfDown).expect("rounds"),
            n("1000"),
            "a tie toward zero"
        );
        let third = Num::ratio(1, 3).expect("1/3");
        assert_eq!(
            third.round(&n("0.01"), Mode::Up).expect("rounds"),
            n("0.34"),
            "1/3 up"
        );
    }

    #[test]
    fn a_step_must_be_positive() {
        assert!(n("1").round(&n("0"), Mode::Up).is_err(), "step 0");
        assert!(n("1").round(&n("-1"), Mode::Up).is_err(), "step -1");
    }

    #[test]
    fn rounding_is_idempotent_and_big_values_round() {
        let big = Num::int(i64::MAX)
            .checked_mul(&Num::int(i64::MAX))
            .and_then(|v| v.checked_add(&n("0.5")))
            .expect("big");
        for mode in MODES {
            let once = big.round(&Num::int(1), mode).expect("rounds");
            assert_eq!(
                once.round(&Num::int(1), mode).expect("again"),
                once,
                "idempotent"
            );
            assert!(once.is_integer(), "whole");
        }
        let squared = Num::int(i64::MAX)
            .checked_mul(&Num::int(i64::MAX))
            .expect("sq");
        // (2^63 - 1)^2 is odd, so its half above ties to the even one above it
        assert_eq!(
            big.round(&Num::int(1), Mode::HalfEven).expect("rounds"),
            squared.checked_add(&Num::int(1)).expect("+1"),
            "half-even on a big tie"
        );
    }

    #[test]
    fn words_name_the_modes() {
        for mode in MODES {
            assert_eq!(Mode::from_word(mode.word()), Some(mode), "{}", mode.word());
        }
        assert_eq!(Mode::from_word("exact"), None, "not a mode");
    }
}
