//! Movements stated `same`: one movement on two statements, counted once, and where its rows disagree a hole naming both.
//! Spec: L §6.1 (What `same` does: `same` is an equivalence, lines that share a side make one
//! group, a group of movements is one movement, bound once, counted once by `flows` and
//! `observed`), §5.19 (One movement on two statements: a read of the movement away from its
//! places is the disagreement's hole, `HOLE-SAME`, naming both rows), §8.1 (`HOLE-SAME`'s form
//! `<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)`).
//! Twin: engine/tools/refcore/moves.py (`same_groups`, `once_obs`, `same_obs_differ`) and
//! mreads.py (`once_by_same`).
//! Invariants: a group's members are in the order the same lines first name them; a group is
//! counted by its first member a read meets, its other members beside it; two rows disagree
//! only on a leg both state, never where either side is a hole.

use std::collections::BTreeMap;

use super::observe::Obs;
use crate::eval::num_stop;
use crate::hole::Hole;
use crate::model::{Reality, SamePair};
use crate::num::Num;
use crate::syntax::Stop;
use crate::walk::{Amount, Dated, LegVal, invariant, qty_value};

/// Each movement stated `same` with its group, the group's members in the order the same
/// lines first name them (refcore's `same_groups`). L §6.1.
#[must_use]
pub fn same_groups(m: &Reality) -> BTreeMap<String, Vec<String>> {
    let mut order: Vec<&str> = Vec::new();
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    for pair in &m.sames {
        let SamePair::Moves(left, right) = pair else {
            continue;
        };
        for side in [left.as_str(), right.as_str()] {
            if !order.contains(&side) {
                order.push(side);
            }
        }
        let (top, under) = (root(&parent, left), root(&parent, right));
        if top != under {
            parent.insert(under, top);
        }
    }
    let mut by_root: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for side in &order {
        by_root
            .entry(root(&parent, side))
            .or_default()
            .push((*side).to_owned());
    }
    let mut out = BTreeMap::new();
    for group in by_root.into_values() {
        for member in &group {
            out.insert(member.clone(), group.clone());
        }
    }
    out
}

/// The root of `start` among the links `parent`, a union-find (refcore's `find`); a walk up
/// is at most as long as the links. L §6.1.
fn root(parent: &BTreeMap<String, String>, start: &str) -> String {
    let mut at = start.to_owned();
    for _ in 0..=parent.len() {
        let Some(up) = parent.get(&at).filter(|up| **up != at) else {
            break;
        };
        at.clone_from(up);
    }
    at
}

/// The observations `obs`, a group stated `same` counted once: each group's first row met,
/// with the group's other rows among `obs`, by position, in the order given (refcore's
/// `once_obs`). L §5.19 One movement on two statements, §6.1.
#[must_use]
pub fn once_obs(groups: &BTreeMap<String, Vec<String>>, obs: &[Obs]) -> Vec<(usize, Vec<usize>)> {
    let mut out: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut slots: BTreeMap<&str, usize> = BTreeMap::new();
    for (pos, row) in obs.iter().enumerate() {
        let lead = groups
            .get(&row.id)
            .and_then(|group| group.first())
            .map_or(row.id.as_str(), String::as_str);
        if let Some(slot) = slots.get(lead).and_then(|&k| out.get_mut(k)) {
            slot.1.push(pos);
            continue;
        }
        slots.insert(lead, out.len());
        out.push((pos, Vec::new()));
    }
    out
}

/// Which of the movements `ids` a binding keeps: of each group stated `same`, the member
/// the same lines name first among them, so the group is bound once (refcore's
/// `once_by_same`). L §6.1, §5.19.
#[must_use]
pub fn kept_once(groups: &BTreeMap<String, Vec<String>>, ids: &[&str]) -> Vec<bool> {
    let mut kept: Vec<&str> = Vec::new();
    for group in groups.values() {
        if let Some(first) = group.iter().find(|member| ids.contains(&member.as_str())) {
            kept.push(first.as_str());
        }
    }
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(!groups.contains_key(*id) || kept.contains(id));
    }
    out
}

/// Where row `row` and the rows `others` stated `same` with it disagree on a leg's amount
/// or day (on leg `leg` alone, where given), the disagreement's hole in L §8.1's form,
/// naming both rows; none where they agree or either side is a hole (refcore's
/// `same_obs_differ`). L §5.19, §6.1, §8.1.
///
/// # Errors
///
/// A number error printing an amount.
pub fn same_differ(
    m: &Reality,
    row: &Obs,
    others: &[&Obs],
    leg: Option<&str>,
) -> Result<Option<Hole>, Stop> {
    for other in others {
        for mine in row
            .legs
            .iter()
            .filter(|lg| leg.is_none_or(|name| lg.name == name))
        {
            let Some(theirs) = other.legs.iter().find(|lg| lg.name == mine.name) else {
                continue;
            };
            let (what, (one, two)) = if let Some(pair) = amounts_differ(m, mine, theirs)? {
                (format!("the amount of {}", mine.name), pair)
            } else if let Some(pair) = days_differ(mine, theirs) {
                (format!("the day of {}", mine.name), pair)
            } else {
                continue;
            };
            let day = mine.day.day().unwrap_or(row.day);
            let text = format!(
                "{} and {} are one, but {what} differs on {}: {one} ({}) and {two} ({})",
                row.id,
                other.id,
                day.text(),
                row.id,
                other.id
            );
            return Ok(Some(Hole::asked(
                &text,
                &format!("{}, {}", row.id, other.id),
            )));
        }
    }
    Ok(None)
}

/// Two legs' amounts as they print where both are numbers that differ (refcore's `vkey`
/// compare). L §5.19.
fn amounts_differ(
    m: &Reality,
    mine: &LegVal,
    theirs: &LegVal,
) -> Result<Option<(String, String)>, Stop> {
    let (Amount::Qty(one), Amount::Qty(two)) = (&mine.amount, &theirs.amount) else {
        return Ok(None);
    };
    if one == two && mine.unit == theirs.unit {
        return Ok(None);
    }
    let (Some(one_unit), Some(two_unit)) = (&mine.unit, &theirs.unit) else {
        return Err(invariant("an observed leg's amount with no unit"));
    };
    Ok(Some((shown(m, one, one_unit)?, shown(m, two, two_unit)?)))
}

/// An amount in its unit as L §2.4 prints it. L §2.4 Values.
fn shown(m: &Reality, amount: &Num, unit: &str) -> Result<String, Stop> {
    qty_value(m, amount, unit)?.text(m).map_err(num_stop)
}

/// Two legs' days as they print where both are days that differ. L §5.19.
fn days_differ(mine: &LegVal, theirs: &LegVal) -> Option<(String, String)> {
    let (Dated::Day(one), Dated::Day(two)) = (&mine.day, &theirs.day) else {
        return None;
    };
    (one != two).then(|| (one.text(), two.text()))
}

#[cfg(test)]
mod tests {
    use super::{kept_once, root};
    use std::collections::BTreeMap;

    #[test]
    fn a_group_is_bound_once_by_its_first_member_met() {
        let group = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        let mut groups = BTreeMap::new();
        for member in &group {
            groups.insert(member.clone(), group.clone());
        }
        assert_eq!(
            kept_once(&groups, &["c", "b", "z"]),
            vec![false, true, true],
            "the first member the same lines name is kept"
        );
        assert_eq!(
            kept_once(&groups, &["c"]),
            vec![true],
            "a lone member is kept"
        );
        let mut parent = BTreeMap::new();
        parent.insert("b".to_owned(), "a".to_owned());
        parent.insert("c".to_owned(), "b".to_owned());
        assert_eq!(root(&parent, "c"), "a", "the root of a chain");
    }
}
