//! The split of a total by weights to a step: `prorata`'s three rests, the largest-remainder split written once for every split that takes it.
//! Spec: L §5.13 (`prorata`: `raw_j = total × w_j / Σw`, the weights ≥ 0 with Σw > 0 or
//! `RUN-WEIGHTS`; `rest none` rounds each raw share by the mode; `rest last` rounds each but
//! the last, which takes what remains; `rest largest` rounds each toward zero and gives the
//! leftover steps one each to the largest remainders, Hamilton's method kept symmetric in
//! sign, a total that isn't a whole number of steps `RUN-PRORATA`, the remainders' tie by the
//! written `ties by` only), §3.6 (an arrival's split among owners and a `prorata` take are
//! this same split, R-114).
//! Twin: engine/tools/refcore/eval2.py (`m_e_prorata`); partswalk.py (`w_psplit`).
//! Invariants: the weights are checked before any share is made; a share is a whole multiple
//! of the step; under `rest largest` and `rest last` the shares sum to the total (`rest
//! largest` checks it: else `ENGINE-INVARIANT`); a tie among the remainders that changes
//! which keys take a step is never broken here: the caller passes the keys its `ties by`
//! chose, and nothing else (no index order, no id, no map's order) decides it (R-114).

use super::rational::{Num, NumError};
use super::round::Mode;
use core::cmp::Ordering;

/// Why a split gives no shares; each maps to a code of L §8.4 at its key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    /// A weight below 0, or weights summing to 0 (`RUN-WEIGHTS`).
    Weights,
    /// Under `rest largest`, a total that isn't a whole number of the step (`RUN-PRORATA`).
    NotWhole,
    /// A number error: the cap, or a broken invariant (`ENGINE-INVARIANT`).
    Num(NumError),
}

impl From<NumError> for SplitError {
    /// A number error as a split's (L §7.1, §5.13).
    fn from(error: NumError) -> SplitError {
        SplitError::Num(error)
    }
}

/// The weights of a split, in key order, checked: each at least 0, their sum above 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weights {
    /// Each key's weight.
    each: Vec<Num>,
    /// Their sum, above 0.
    sum: Num,
}

impl Weights {
    /// The weights `each`, checked: `RUN-WEIGHTS` for one below 0 or a sum of 0, no keys
    /// included. L §5.13 step 1.
    ///
    /// # Errors
    ///
    /// `SplitError::Weights`, or a number error of the sum.
    pub fn new(each: Vec<Num>) -> Result<Weights, SplitError> {
        let mut sum = Num::zero();
        for w in &each {
            if w.is_negative() {
                return Err(SplitError::Weights);
            }
            sum = sum.checked_add(w)?;
        }
        if sum.is_zero() {
            return Err(SplitError::Weights);
        }
        Ok(Weights { each, sum })
    }

    /// `raw_j = total × w_j / Σw` of key `j`. L §5.13 step 1.
    fn raw(&self, total: &Num, j: usize) -> Result<Num, NumError> {
        let w = self.each.get(j).ok_or(NumError::Invariant)?;
        total.checked_mul(w)?.checked_div(&self.sum)
    }

    /// `rest none`: key `j`'s raw share rounded to `step` by `mode`, on its own. L §5.13
    /// step 2.
    ///
    /// # Errors
    ///
    /// A number error; `RUN-DOMAIN` for a step that isn't positive.
    pub fn rounded(
        &self,
        total: &Num,
        j: usize,
        step: &Num,
        mode: Mode,
    ) -> Result<Num, SplitError> {
        Ok(self.raw(total, j)?.round(step, mode)?)
    }

    /// `rest last`: key `j`'s raw share rounded to `step` by `mode`, but the last key's share
    /// is what the others leave of the total (the instalment that absorbs the odd unit,
    /// N-213). L §5.13 step 2.
    ///
    /// # Errors
    ///
    /// A number error; `RUN-DOMAIN` for a step that isn't positive.
    pub fn last_rest(
        &self,
        total: &Num,
        j: usize,
        step: &Num,
        mode: Mode,
    ) -> Result<Num, SplitError> {
        let last = self.each.len().checked_sub(1).ok_or(NumError::Invariant)?;
        if j != last {
            return self.rounded(total, j, step, mode);
        }
        let mut rest = total.clone();
        for other in 0..last {
            rest = rest.checked_sub(&self.rounded(total, other, step, mode)?)?;
        }
        Ok(rest)
    }

    /// `rest largest`: the split of `total` to `step` before a tie among the remainders is
    /// broken. Each key takes its raw share's whole steps rounded toward zero, of the total's
    /// absolute value; the steps left over go one each to the keys with the largest
    /// remainders, so a negative total's shares are the negatives of its absolute value's
    /// (Hamilton's method, kept symmetric in sign). Where keys whose remainders tie at the
    /// edge can't all take one, the split holds that tie for the caller's `ties by` (D-5).
    /// L §5.13 step 2, §3.6.
    ///
    /// # Errors
    ///
    /// `RUN-PRORATA` for a total that isn't a whole number of `step`; `RUN-DOMAIN` for a step
    /// that isn't positive (`prorata` refuses one first, with its text); a number error.
    pub fn largest(&self, total: &Num, step: &Num) -> Result<Largest, SplitError> {
        if step.compare(&Num::zero())? != Ordering::Greater {
            return Err(SplitError::Num(NumError::Domain));
        }
        let signed_steps = total.checked_div(step)?;
        if !signed_steps.is_integer() {
            return Err(SplitError::NotWhole);
        }
        let negative = signed_steps.is_negative();
        let steps = if negative {
            signed_steps.checked_neg()?
        } else {
            signed_steps
        };
        let n = self.each.len();
        let (mut units, mut rems) = (Vec::with_capacity(n), Vec::with_capacity(n));
        let mut handed = Num::zero();
        for j in 0..n {
            // the raw share in steps, at least 0: its floor is its whole steps toward zero
            let x = self.raw(&steps, j)?;
            let down = x.round(&Num::int(1), Mode::Floor)?;
            rems.push(x.checked_sub(&down)?);
            handed = handed.checked_add(&down)?;
            units.push(down);
        }
        let left = steps.checked_sub(&handed)?;
        let left = left
            .to_i64()
            .and_then(|l| usize::try_from(l).ok())
            .ok_or(NumError::Invariant)?;
        let tie = edge_tie(&rems, left, &mut units)?;
        Ok(Largest {
            negative,
            step: step.clone(),
            steps,
            units,
            tie,
        })
    }
}

/// The steps left over given to the keys whose remainders are above the edge, the `left`-th
/// largest remainder, and the keys at the edge: all given one when they can all take one,
/// else held as the tie (L §5.13 step 2: "keys whose remainders tie go by `ties by e`").
fn edge_tie(rems: &[Num], left: usize, units: &mut [Num]) -> Result<Option<Tie>, NumError> {
    let Some(at) = left.checked_sub(1) else {
        return Ok(None);
    };
    let order = largest_first(rems)?;
    let edge_key = order.get(at).ok_or(NumError::Invariant)?;
    let edge = rems.get(*edge_key).ok_or(NumError::Invariant)?;
    let (mut above, mut tied) = (0_usize, Vec::new());
    for ((j, rem), unit) in rems.iter().enumerate().zip(units.iter_mut()) {
        match rem.compare(edge)? {
            Ordering::Greater => {
                *unit = unit.checked_add(&Num::int(1))?;
                above = above.checked_add(1).ok_or(NumError::Invariant)?;
            }
            Ordering::Equal => tied.push(j),
            Ordering::Less => {}
        }
    }
    let take = left.checked_sub(above).ok_or(NumError::Invariant)?;
    if take == tied.len() {
        for j in &tied {
            let unit = units.get_mut(*j).ok_or(NumError::Invariant)?;
            *unit = unit.checked_add(&Num::int(1))?;
        }
        return Ok(None);
    }
    Ok(Some(Tie { tied, take }))
}

/// The keys `0..n` by their remainders, the largest first, equal remainders in key order: a
/// merge sort, since a comparison of two numbers can fail (the cap), and none is assumed.
/// L §5.13 step 2 ("the keys with the largest remainders").
fn largest_first(rems: &[Num]) -> Result<Vec<usize>, NumError> {
    let mut runs: Vec<Vec<usize>> = (0..rems.len()).map(|j| vec![j]).collect();
    while runs.len() > 1 {
        let mut merged = Vec::with_capacity(runs.len());
        let mut pairs = runs.into_iter();
        while let Some(a) = pairs.next() {
            let Some(b) = pairs.next() else {
                merged.push(a);
                break;
            };
            merged.push(merge(rems, a, b)?);
        }
        runs = merged;
    }
    Ok(runs.into_iter().flatten().collect())
}

/// Two runs, each the largest remainder first, as one: `b`'s key goes first only when its
/// remainder is strictly larger, so equal remainders keep key order. L §5.13 step 2.
fn merge(rems: &[Num], a: Vec<usize>, b: Vec<usize>) -> Result<Vec<usize>, NumError> {
    let size = a.len().checked_add(b.len()).ok_or(NumError::Invariant)?;
    let mut out = Vec::with_capacity(size);
    let (mut a, mut b) = (a.into_iter().peekable(), b.into_iter().peekable());
    while let (Some(&x), Some(&y)) = (a.peek(), b.peek()) {
        let rx = rems.get(x).ok_or(NumError::Invariant)?;
        let ry = rems.get(y).ok_or(NumError::Invariant)?;
        if ry.compare(rx)? == Ordering::Greater {
            out.push(y);
            b.next();
        } else {
            out.push(x);
            a.next();
        }
    }
    out.extend(a);
    out.extend(b);
    Ok(out)
}

/// The keys whose remainders tie at the edge where only some of them can take the last
/// steps: who they are, in key order, and how many of them take one (L §5.13, D-5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tie {
    /// The tied keys, by position, in key order.
    pub tied: Vec<usize>,
    /// How many of them take a step: at least 1, fewer than all.
    pub take: usize,
}

/// A `rest largest` split before its tie is broken (L §5.13 step 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Largest {
    /// Whether the total is negative: every share is then the negative of its absolute
    /// value's.
    negative: bool,
    /// The step.
    step: Num,
    /// The total's absolute value in steps.
    steps: Num,
    /// Each key's steps: its whole steps toward zero, one more above the edge, and one more
    /// for each key at the edge when they all take one.
    units: Vec<Num>,
    /// The tie at the edge that changes which keys take a step, if there is one.
    tie: Option<Tie>,
}

impl Largest {
    /// The tie the caller's `ties by` breaks, if the split has one. L §5.13 step 2.
    #[must_use]
    pub fn tie(&self) -> Option<&Tie> {
        self.tie.as_ref()
    }

    /// The shares, in key order, the tie broken by `chosen`: the tied keys, by position,
    /// that the `ties by` gave a step (none where the split has no tie). The shares sum to
    /// the total. L §5.13 step 2 ("The shares then sum to the total").
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` where `chosen` isn't as many tied keys as take a step, each once,
    /// or the shares don't sum to the total; a number error.
    pub fn shares(&self, chosen: &[usize]) -> Result<Vec<Num>, SplitError> {
        let mut units = self.units.clone();
        let (tied, take): (&[usize], usize) = match &self.tie {
            Some(t) => (&t.tied, t.take),
            None => (&[], 0),
        };
        if chosen.len() != take {
            return Err(SplitError::Num(NumError::Invariant));
        }
        // the keys tied at the edge that no choice has taken yet
        let mut open = vec![false; units.len()];
        for j in tied {
            *open.get_mut(*j).ok_or(NumError::Invariant)? = true;
        }
        for j in chosen {
            let slot = open.get_mut(*j).ok_or(NumError::Invariant)?;
            if !*slot {
                return Err(SplitError::Num(NumError::Invariant));
            }
            *slot = false;
            let unit = units.get_mut(*j).ok_or(NumError::Invariant)?;
            *unit = unit.checked_add(&Num::int(1))?;
        }
        let mut sum = Num::zero();
        let mut out = Vec::with_capacity(units.len());
        for unit in units {
            sum = sum.checked_add(&unit)?;
            let share = unit.checked_mul(&self.step)?;
            out.push(if self.negative {
                share.checked_neg()?
            } else {
                share
            });
        }
        if sum != self.steps {
            return Err(SplitError::Num(NumError::Invariant));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::{Largest, SplitError, Weights};
    use crate::num::{Mode, Num};

    fn nums(each: &[i64]) -> Vec<Num> {
        each.iter().map(|n| Num::int(*n)).collect()
    }

    fn cents(n: i64) -> Num {
        Num::ratio(n, 100).expect("cents")
    }

    fn split(total: &Num, weights: &[i64], step: &Num) -> Largest {
        Weights::new(nums(weights))
            .expect("weights")
            .largest(total, step)
            .expect("split")
    }

    #[test]
    fn weights_below_zero_or_summing_to_zero_are_refused() {
        assert_eq!(
            Weights::new(nums(&[1, -1])),
            Err(SplitError::Weights),
            "negative"
        );
        assert_eq!(
            Weights::new(nums(&[0, 0])),
            Err(SplitError::Weights),
            "sum 0"
        );
        assert_eq!(
            Weights::new(Vec::new()),
            Err(SplitError::Weights),
            "no keys"
        );
        assert!(Weights::new(nums(&[0, 2])).is_ok(), "a 0 beside a positive");
    }

    #[test]
    fn thirds_leave_one_step_to_a_tie() {
        // 100.00 by thirds: 33.33 each, one cent left, a three-way tie at the edge
        let s = split(&Num::int(100), &[1, 1, 1], &cents(1));
        let tie = s.tie().expect("a tie");
        assert_eq!((tie.tied.clone(), tie.take), (vec![0, 1, 2], 1), "tie");
        let to_first = s.shares(&[0]).expect("shares");
        assert_eq!(
            to_first,
            vec![cents(3334), cents(3333), cents(3333)],
            "first"
        );
        let to_last = s.shares(&[2]).expect("shares");
        assert_eq!(to_last, vec![cents(3333), cents(3333), cents(3334)], "last");
        assert!(s.shares(&[]).is_err(), "too few chosen");
        assert!(s.shares(&[0, 1]).is_err(), "too many chosen");
    }

    #[test]
    fn largest_remainders_take_the_leftover_steps() {
        // 10 by 1, 2, 4: raw 10/7, 20/7, 40/7; floors 1, 2, 5; remainders 3/7, 6/7, 5/7
        let sevenths = split(&Num::int(10), &[1, 2, 4], &Num::int(1));
        assert_eq!(sevenths.tie(), None, "no tie");
        let shares = sevenths.shares(&[]).expect("shares");
        assert_eq!(shares, nums(&[1, 3, 6]), "sevenths");
        // 3 by 1, 1, 2: floors 0, 0, 1; two steps left, the two remainders of 3/4 tied at the
        // edge: both take one, no tie to break
        let quarters = split(&Num::int(3), &[1, 1, 2], &Num::int(1));
        assert_eq!(quarters.tie(), None, "both tied keys take a step");
        let both = quarters.shares(&[]).expect("shares");
        assert_eq!(both, nums(&[1, 1, 1]), "quarters");
        // 4 by 1, 1, 1, 0: floors 1, 1, 1, 0; one step left, three remainders of 1/3 tied
        let thirds = split(&Num::int(4), &[1, 1, 1, 0], &Num::int(1));
        let tie = thirds.tie().expect("three tied for one step");
        assert_eq!((tie.tied.clone(), tie.take), (vec![0, 1, 2], 1), "tie");
        let to_second = thirds.shares(&[1]).expect("shares");
        assert_eq!(to_second, nums(&[1, 2, 1, 0]), "thirds");
    }

    #[test]
    fn a_negative_total_splits_as_its_absolute_value() {
        let s = split(&Num::int(-10), &[1, 2, 4], &Num::int(1));
        assert_eq!(
            s.shares(&[]).expect("shares"),
            nums(&[-1, -3, -6]),
            "negated"
        );
    }

    #[test]
    fn a_total_off_the_step_is_refused() {
        let w = Weights::new(nums(&[1, 3])).expect("weights");
        assert_eq!(
            w.largest(&Num::int(100), &cents(3)),
            Err(SplitError::NotWhole),
            "100 by 0.03"
        );
        assert!(
            w.largest(&Num::int(100), &Num::zero()).is_err(),
            "a step of 0"
        );
    }

    #[test]
    fn rest_none_and_rest_last_round_by_the_mode() {
        let w = Weights::new(nums(&[1, 1, 1])).expect("weights");
        let total = Num::int(100);
        let none: Vec<Num> = (0..3)
            .map(|j| w.rounded(&total, j, &cents(1), Mode::HalfUp).expect("none"))
            .collect();
        assert_eq!(none, vec![cents(3333), cents(3333), cents(3333)], "none");
        let last: Vec<Num> = (0..3)
            .map(|j| {
                w.last_rest(&total, j, &cents(1), Mode::HalfUp)
                    .expect("last")
            })
            .collect();
        assert_eq!(last, vec![cents(3333), cents(3333), cents(3334)], "last");
    }
}
