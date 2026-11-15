//! The residual ledger's groups: its lines grouped by month and name, each group's totals,
//! the groups and their lines in order.
//! Spec: E §6.3 (Grouping: one group per month and name; Totals: the model's, the
//! observations' and the difference, a balance group's the booked and stated balances; Order:
//! by the size of the difference, holes first, then month, group name and the movement
//! order), L §8.1 (holes sort first, by their first question).
//! Twin: engine/tools/refcore/resid.py (`m_residuals`' groups, totals and sorts, `sk`, `_add`,
//! `_sub`).
//! Invariants: groups and lines are ordered by explicit keys (size, day, group, movement
//! order), ties in the order they were made; a hole on either side makes a total or a
//! difference a hole, never 0; no figure counts as 0.

use core::cmp::Ordering;

use super::ledger::{Fig, Line};
use crate::eval::num_stop;
use crate::hole::{Hole, spread};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::{Day, Period};
use crate::walk::{Term, invariant};

/// A group's key: its month, none for the `?` group, and its name (`move rent_pay`,
/// `unexplained`, `balance`, `?`). E §6.3 Grouping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupKey {
    /// The month; none for the group of instances whose day is a hole.
    pub month: Option<Period>,
    /// The name.
    pub name: String,
}

/// A group of the ledger: its lines and its totals. E §6.3 Totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// Its key.
    pub key: GroupKey,
    /// The model's total (the booked balances, for a balance group).
    pub model: Fig,
    /// The observations' total (the stated balances, for a balance group).
    pub observed: Fig,
    /// The difference, observed less model.
    pub diff: Fig,
    /// Its lines, in order.
    pub lines: Vec<Line>,
}

/// The group of a line on `day` named `name`: its month's. E §6.3 Grouping.
pub(super) fn month_group(day: Day, name: String) -> Result<GroupKey, Stop> {
    let month = Period::month_of(day).ok_or_else(|| invariant("a month past the calendar"))?;
    Ok(GroupKey {
        month: Some(month),
        name,
    })
}

/// `a + b`, where `a` may be no figure yet: holes spread (refcore's `_add`). E §6.3 Totals.
pub(super) fn fig_add(a: Option<Fig>, b: &Fig) -> Result<Fig, Stop> {
    let Some(a) = a else {
        return Ok(b.clone());
    };
    if let (Fig::Qty(x), Fig::Qty(y)) = (&a, b) {
        return Ok(Fig::Qty(x.checked_add(y).map_err(num_stop)?));
    }
    spread(&[a.hole(), b.hole()])
        .map(Fig::Hole)
        .ok_or_else(|| invariant("a sum of holes with no hole"))
}

/// `a − b`, where no figure is 0: a hole on either side makes the difference a hole
/// (refcore's `_sub`). E §6.3 Signs.
pub(super) fn fig_sub(a: Option<&Fig>, b: Option<&Fig>) -> Result<Fig, Stop> {
    let holes = [a.and_then(Fig::hole), b.and_then(Fig::hole)];
    if let Some(h) = spread(&holes) {
        return Ok(Fig::Hole(h));
    }
    let x = qty_or_zero(a);
    let y = qty_or_zero(b);
    Ok(Fig::Qty(x.checked_sub(&y).map_err(num_stop)?))
}

/// A figure's quantity, where no figure is 0 (refcore's `(a or 0)`); a hole is never one
/// here, `fig_sub` having taken it. E §6.3.
fn qty_or_zero(f: Option<&Fig>) -> Num {
    match f {
        Some(Fig::Qty(q)) => q.clone(),
        Some(Fig::Hole(_)) | None => Num::zero(),
    }
}

/// `|q|`. E §6.3 Order.
pub(super) fn abs(q: &Num) -> Result<Num, Stop> {
    if q.is_negative() {
        return q.checked_neg().map_err(num_stop);
    }
    Ok(q.clone())
}

/// What a line or a group sorts by (refcore's sort keys). E §6.3 Order.
struct SortKey {
    /// A group's: whether it has a month, so the `?` group comes first; false for a line.
    dated: bool,
    /// The difference, by size: holes first, then the biggest.
    size: Fig,
    /// A line's day, a group's month's first day; none sorts first.
    day: Option<Day>,
    /// The group's name.
    name: String,
    /// A line's movement order; the empty tuple for a group, whose order no tie reaches.
    order: Term,
}

/// Two figures by size: holes first, among themselves by their first question, then by
/// absolute value, the biggest first (refcore's `sk`). E §6.3 Order, L §8.1.
fn size_cmp(a: &Fig, b: &Fig) -> Result<Ordering, Stop> {
    match (a, b) {
        (Fig::Hole(x), Fig::Hole(y)) => Ok(first_question(x).cmp(&first_question(y))),
        (Fig::Hole(_), Fig::Qty(_)) => Ok(Ordering::Less),
        (Fig::Qty(_), Fig::Hole(_)) => Ok(Ordering::Greater),
        (Fig::Qty(x), Fig::Qty(y)) => abs(y)?.compare(&abs(x)?).map_err(num_stop),
    }
}

/// A hole's first question, by code point; the empty text for one with none. L §8.1.
fn first_question(h: &Hole) -> String {
    match h.question_texts().into_iter().next() {
        Some(q) => q,
        None => String::new(),
    }
}

/// Two sort keys: a group's month first, then the size, the day, the group's name and the
/// movement order (refcore's `sort` keys). E §6.3 Order.
fn key_cmp(a: &SortKey, b: &SortKey) -> Result<Ordering, Stop> {
    let first = a.dated.cmp(&b.dated).then(size_cmp(&a.size, &b.size)?);
    let rest = a.day.cmp(&b.day).then_with(|| a.name.cmp(&b.name));
    Ok(first.then(rest).then_with(|| a.order.cmp(&b.order)))
}

/// The positions of `keys` in sorted order, ties in their order: a merge sort from runs of
/// one, since comparing two sizes can refuse at the digits' cap (refcore's stable `sort`).
/// E §6.3 Order.
fn sorted_positions(keys: &[SortKey]) -> Result<Vec<usize>, Stop> {
    let mut runs: Vec<Vec<usize>> = (0..keys.len()).map(|i| vec![i]).collect();
    while runs.len() > 1 {
        let mut next = Vec::new();
        let mut pairs = runs.into_iter();
        while let Some(left) = pairs.next() {
            match pairs.next() {
                Some(right) => next.push(merged(keys, left, right)?),
                None => next.push(left),
            }
        }
        runs = next;
    }
    match runs.into_iter().next() {
        Some(order) => Ok(order),
        None => Ok(Vec::new()),
    }
}

/// Two sorted runs merged, the left one's first on a tie, so the sort is stable. E §6.3
/// Order.
fn merged(keys: &[SortKey], left: Vec<usize>, right: Vec<usize>) -> Result<Vec<usize>, Stop> {
    let mut out = Vec::new();
    let mut lefts = left.into_iter().peekable();
    let mut rights = right.into_iter().peekable();
    loop {
        let take_right = match (lefts.peek(), rights.peek()) {
            (Some(a), Some(b)) => {
                key_cmp(key_at(keys, *a)?, key_at(keys, *b)?)? == Ordering::Greater
            }
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (None, None) => return Ok(out),
        };
        out.extend(if take_right {
            rights.next()
        } else {
            lefts.next()
        });
    }
}

/// The sort key at position `i`. E §6.3 Order.
fn key_at(keys: &[SortKey], i: usize) -> Result<&SortKey, Stop> {
    keys.get(i).ok_or_else(|| invariant("a ledger's sort"))
}

/// The lines in groups, each group's lines and totals, the groups in order (refcore's
/// `groups`, `out`). E §6.3 Grouping, Totals, Order.
pub(super) fn grouped(lines: Vec<Line>) -> Result<Vec<Group>, Stop> {
    let mut made: Vec<(GroupKey, Vec<Line>)> = Vec::new();
    for line in lines {
        match made.iter_mut().find(|(k, _)| *k == line.group) {
            Some((_, ls)) => ls.push(line),
            None => made.push((line.group.clone(), vec![line])),
        }
    }
    let mut groups = Vec::with_capacity(made.len());
    for (key, ls) in made {
        groups.push(group_of(key, ls)?);
    }
    let keys: Vec<SortKey> = groups
        .iter()
        .map(|g| SortKey {
            dated: g.key.month.is_some(),
            size: g.diff.clone(),
            day: g.key.month.map(Period::start),
            name: g.key.name.clone(),
            order: Term::Tuple(Vec::new()),
        })
        .collect();
    let order = sorted_positions(&keys)?;
    let mut slots: Vec<Option<Group>> = groups.into_iter().map(Some).collect();
    let mut sorted = Vec::with_capacity(order.len());
    for i in &order {
        let group = slots.get_mut(*i).and_then(Option::take);
        sorted.push(group.ok_or_else(|| invariant("a ledger's group sorted twice"))?);
    }
    Ok(sorted)
}

/// One group: its totals, the model's and the observations' (none counting 0), the
/// difference, its lines in order. E §6.3 Totals, Order.
fn group_of(key: GroupKey, lines: Vec<Line>) -> Result<Group, Stop> {
    let mut model: Option<Fig> = None;
    let mut observed: Option<Fig> = None;
    for line in &lines {
        if let Some(f) = &line.model {
            model = Some(fig_add(model, f)?);
        }
        if let Some(f) = &line.observed {
            observed = Some(fig_add(observed, f)?);
        }
    }
    let model = model.unwrap_or_else(|| Fig::Qty(Num::zero()));
    let observed = observed.unwrap_or_else(|| Fig::Qty(Num::zero()));
    let diff = fig_sub(Some(&observed), Some(&model))?;
    let keys: Vec<SortKey> = lines
        .iter()
        .map(|l| SortKey {
            dated: false,
            size: l.diff.clone(),
            day: l.day,
            name: l.group.name.clone(),
            order: l.order.clone(),
        })
        .collect();
    let order = sorted_positions(&keys)?;
    let mut slots: Vec<Option<Line>> = lines.into_iter().map(Some).collect();
    let mut sorted = Vec::with_capacity(order.len());
    for i in &order {
        let line = slots.get_mut(*i).and_then(Option::take);
        sorted.push(line.ok_or_else(|| invariant("a ledger's line sorted twice"))?);
    }
    Ok(Group {
        key,
        model,
        observed,
        diff,
        lines: sorted,
    })
}
