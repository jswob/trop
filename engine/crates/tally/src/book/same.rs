//! Movements stated `same`: one movement on two statements, counted once, and where its rows disagree a hole naming both.
//! Spec: L §6.1 (What `same` does: `same` is an equivalence, lines that share a side make one
//! group, a group of movements is one movement, bound once, counted once by `flows` and
//! `observed`), §5.19 (One movement on two statements: a read of the movement away from its
//! places is the disagreement's hole, `HOLE-SAME`, naming both rows), §8.1 (`HOLE-SAME`'s form
//! `<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)`), §4.5 (tally
//! 2.1.8 R12: a `same` line is seen under a cut as its version's rows are, but a version's
//! line between two observed rows, which has no clock of its own: it holds at every cut that
//! sees both rows, in the worlds its file is in; tally 2.2, S-5).
//! Twin: engine/tools/refcore/moves.py (`same_lines`, `same_groups`, `once_obs`,
//! `same_obs_differ`, `obs_row_seen`), model.py (`same_seen`) and mreads.py
//! (`once_by_same`); refcore's walk.py `same_observed` (an instance stated `same` as an
//! observed row) serves only a joint model's lines since load refuses the pair in one model
//! (tally 2.1.8), and is built here from it when joint models run (M5c).
//! Invariants: a group's members are in the order of their ids, an id by its item's name and
//! then each key by its place in its index, an open index's by code point (tally 2.1.8 S6, pick
//! P50); a group is counted by its first member a read meets, its other members beside it;
//! rows disagree only on a leg two of them state, never where any side is a hole; `HOLE-SAME`
//! names every row of the class that states the fact, in that order; rows stated `same` that
//! state different `fulfils` are `RUN-RECORD` for every read that rests on them (S1, P42).
//! The walk's booking of a movement stated `same` is `book/same_booking.rs`'s.

use std::collections::{BTreeMap, BTreeSet};

use super::observe::{Obs, observed_row_seen};
use super::same_booking::{EndBook, Member};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Ctx, Engine, num_stop};
use crate::hole::Hole;
use crate::model::{KeyVal, MvRef, Reality, Value, World};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{Amount, Dated, End, Inst, LegVal, invariant, qty_value};

/// Each movement stated `same`, by id, with its group's members in the order the same lines
/// first name them (L §6.1).
pub type Groups = BTreeMap<String, Vec<String>>;

/// Whether cut `cut` in world `world` sees a `same` line of version `version`: a line in a
/// joint model's header always; else only in the worlds its version's file is in, its
/// version's `known` on or before the cut (refcore's `same_seen`, `same_lines`). L §4.5
/// (tally 2.1.8 R12: whatever a version states is read as its rows are), §6.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a version load didn't build.
fn same_line_seen(
    m: &Reality,
    version: Option<&str>,
    (cut, world): (Day, World),
) -> Result<bool, Stop> {
    let Some(id) = version else {
        return Ok(true);
    };
    if !m.scenarios.sees(world, Some(id)) {
        return Ok(false);
    }
    let info = m
        .versions
        .get(id)
        .ok_or_else(|| invariant("a same line's version load didn't build"))?;
    Ok(info.known <= cut)
}

/// Whether cut `cut` in world `world` sees a version's `same` line between observed rows
/// `left` and `right`: it has no day of its own, so in the worlds its version's file is in it
/// holds at every cut that sees both rows, whichever version states it (refcore's
/// `same_lines`, `obs_row_seen`). L §5.19 One movement on two statements, §6.4 (tally 2.2,
/// S-5).
///
/// # Errors
///
/// As [`observed_row_seen`].
fn rows_line_seen(
    m: &Reality,
    (left, right): (&str, &str),
    version: &str,
    (cut, world): (Day, World),
) -> Result<bool, Stop> {
    Ok(m.scenarios.sees(world, Some(version))
        && observed_row_seen(m, left, (cut, world))?
        && observed_row_seen(m, right, (cut, world))?)
}

impl Engine {
    /// Whether cut `cut` sees each `same` line of the reality, in the order written: a
    /// version's line between two observed rows as [`rows_line_seen`] decides, any other as
    /// [`same_line_seen`] does. L §6.1, §5.19, §4.5.
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`] and [`rows_line_seen`].
    fn moves_seen(&self, m: &Reality, cut: Day) -> Result<Vec<bool>, Stop> {
        let world = self.timeline_world();
        let mut out = Vec::with_capacity(m.sames.len());
        for pair in &m.sames {
            let seen = match (&pair.version, pair.observed) {
                (Some(version), true) => {
                    rows_line_seen(m, (&pair.left, &pair.right), version, (cut, world))?
                }
                (version, _) => same_line_seen(m, version.as_deref(), (cut, world))?,
            };
            out.push(seen);
        }
        Ok(out)
    }

    /// Each movement stated `same` by a line cut `cut` sees, with its group, the group's
    /// members in the order of their ids (refcore's `same_groups`). L §6.1, §2.5, §4.5
    /// (tally 2.1.8 S6).
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`].
    pub(crate) fn same_groups(&mut self, m: &Reality, cut: Day) -> Result<Groups, Stop> {
        // a class joined by lines in two files has no written order: its members go by their
        // ids in id order (L §2.5; tally 2.1.8 S6, P50)
        let mut order: BTreeSet<&str> = BTreeSet::new();
        let mut parent: BTreeMap<String, String> = BTreeMap::new();
        for (left, right) in move_lines(m, self.moves_seen(m, cut)?) {
            order.insert(left.as_str());
            order.insert(right.as_str());
            let (top, under) = (root(&parent, left), root(&parent, right));
            if top != under {
                parent.insert(under, top);
            }
        }
        let mut by_root: BTreeMap<String, Vec<String>> = BTreeMap::new();
        // in id order, a list or range index keeping its own (pick P50)
        let mut order: Vec<&str> = order.into_iter().collect();
        order.sort_by_cached_key(|side| m.id_order(side));
        for side in &order {
            by_root
                .entry(root(&parent, side))
                .or_insert_with(Vec::new)
                .push((*side).to_owned());
        }
        let mut out = BTreeMap::new();
        for group in by_root.into_values() {
            for member in &group {
                out.insert(member.clone(), group.clone());
            }
        }
        Ok(out)
    }

    /// The sides of the `same` lines cut `cut` sees, each once, in id order, a list or range
    /// index keeping its own order: section order changes nothing, and S6 names a class's
    /// rows in id order (refcore's `once_by_same`, `id_order`; picks P42 and P50, group 4's
    /// review N2 and its fix's F1). L §6.1, §2.5, §4.5.
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`].
    pub(crate) fn same_sides(&mut self, m: &Reality, cut: Day) -> Result<Vec<String>, Stop> {
        let mut out: BTreeSet<String> = BTreeSet::new();
        for (left, right) in move_lines(m, self.moves_seen(m, cut)?) {
            out.insert(left.clone());
            out.insert(right.clone());
        }
        let mut out: Vec<String> = out.into_iter().collect();
        out.sort_by_cached_key(|side| m.id_order(side));
        Ok(out)
    }

    /// Whether movement `id` is named by a `same` line cut `cut` sees. L §6.1, §4.5.
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`].
    pub(crate) fn stated_same(&mut self, m: &Reality, id: &str, cut: Day) -> Result<bool, Stop> {
        Ok(move_lines(m, self.moves_seen(m, cut)?)
            .iter()
            .any(|(a, b)| *a == id || *b == id))
    }

    /// The first `same` line cut `cut` sees naming movement `id`, its two sides (refcore's
    /// `same_pairs`, its lines). L §6.1, §4.5.
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`].
    pub(super) fn first_line(
        &mut self,
        m: &Reality,
        id: &str,
        cut: Day,
    ) -> Result<Option<(String, String)>, Stop> {
        Ok(move_lines(m, self.moves_seen(m, cut)?)
            .into_iter()
            .find(|(a, b)| *a == id || *b == id)
            .map(|(a, b)| (a.clone(), b.clone())))
    }
}

/// The `same` lines of `m` that `seen` flags (one flag per line, as [`Engine::moves_seen`]
/// gives them), each its two sides, in the order written. L §6.1, §4.5.
fn move_lines(m: &Reality, seen: Vec<bool>) -> Vec<(&String, &String)> {
    let mut out = Vec::new();
    for (pair, flag) in m.sames.iter().zip(seen) {
        if flag {
            out.push((&pair.left, &pair.right));
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

/// Row `o` with each leg that touches `place` taking the hole `hole`: a `same` group whose
/// members disagree there, as the walk books it, the ledger's one line there (E §6.3; L §5.19
/// One movement on two statements).
fn holed_at(o: &Obs, place: &str, hole: &Hole) -> Obs {
    let mut row = o.clone();
    for lg in &mut row.legs {
        if lg.from.place() == Some(place) || lg.to.place() == Some(place) {
            lg.amount = Amount::Hole(hole.clone());
        }
    }
    row
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
/// first among them in `sides`, the same lines' sides in id order, so the group is bound
/// once, by its first id (refcore's `once_by_same`; pick P42). L §6.1, §5.19.
#[must_use]
pub fn kept_once(
    groups: &BTreeMap<String, Vec<String>>,
    sides: &[String],
    ids: &[&str],
) -> Vec<bool> {
    let mut kept: Vec<&str> = Vec::new();
    // the groups met so far, each by its first member
    let mut met: BTreeSet<&str> = BTreeSet::new();
    for side in sides {
        if !ids.contains(&side.as_str()) {
            continue;
        }
        let Some(lead) = groups.get(side).and_then(|group| group.first()) else {
            continue;
        };
        if met.insert(lead.as_str()) {
            kept.push(side.as_str());
        }
    }
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(!groups.contains_key(*id) || kept.contains(id));
    }
    out
}

/// An observed row as `HOLE-SAME` reads one side of a movement stated `same`: its id, its
/// day, its legs (L §5.19 One movement on two statements, §8.1).
pub(crate) fn obs_side(o: &Obs) -> (&str, Option<Day>, &[LegVal]) {
    (&o.id, Some(o.day), &o.legs)
}

/// A planned instance as `HOLE-SAME` reads one side of a movement stated `same`: its id as
/// the register names it (`in1`, `in2[k]`), its day where it is one, its legs (L §6.1, §8.1;
/// tally 2.2.1's 219.1).
pub(crate) fn inst_side(i: &Inst) -> (&str, Option<Day>, &[LegVal]) {
    (&i.id, i.day.as_ref().and_then(Dated::day), &i.legs)
}

/// Where row `row` and the rows `others` stated `same` with it disagree on a leg's amount
/// or day (on leg `leg` alone, where given), the disagreement's hole in L §8.1's form,
/// naming every row of the class that states the leg, in id order; none where they agree or
/// any side is a hole (refcore's `same_obs_differ`). L §5.19, §6.1, §8.1 (tally 2.1.8 S6).
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
    let others: Vec<(&str, Option<Day>, &[LegVal])> = others.iter().map(|o| obs_side(o)).collect();
    sides_differ(m, obs_side(row), &others, leg)
}

/// [`same_differ`] over sides, each an id, a day where it is one and legs ([`obs_side`],
/// [`inst_side`]), a planned one named by its instance (tally 2.2.1's 219.1): where side
/// `row` and the sides `others` stated `same` with it disagree on a leg's amount or day (on
/// leg `leg` alone, where given), the disagreement's hole naming every side of the class that
/// states the leg, in id order (refcore's `same_obs_differ`). L §5.19, §6.1, §8.1.
///
/// # Errors
///
/// A number error printing an amount or a day.
pub(crate) fn sides_differ(
    m: &Reality,
    (row_id, row_day, legs): (&str, Option<Day>, &[LegVal]),
    others: &[(&str, Option<Day>, &[LegVal])],
    leg: Option<&str>,
) -> Result<Option<Hole>, Stop> {
    for mine in legs {
        if leg.is_some_and(|name| mine.name != name) {
            continue;
        }
        let mut class: Vec<(&str, &LegVal)> = vec![(row_id, mine)];
        for (other, _, their_legs) in others {
            for theirs in their_legs.iter().filter(|lg| lg.name == mine.name) {
                class.push((other, theirs));
            }
        }
        if class.len() < 2 {
            continue;
        }
        // the rows in id order (L §8.1 HOLE-SAME, §2.5; P50)
        class.sort_by_cached_key(|(o, _)| m.id_order(o));
        let (what, figures) = if let Some(figures) = amounts_differ(m, &class)? {
            (format!("the amount of {}", mine.name), figures)
        } else if let Some(figures) = days_differ(&class) {
            (format!("the day of {}", mine.name), figures)
        } else {
            continue;
        };
        let ids: Vec<String> = class.iter().map(|(o, _)| (*o).to_owned()).collect();
        let parts: Vec<String> = figures
            .iter()
            .zip(&ids)
            .map(|(figure, id)| format!("{figure} ({id})"))
            .collect();
        let day = match mine.day.day().or(row_day) {
            Some(day) => day.text(),
            None => mine.day.value().text(m).map_err(num_stop)?,
        };
        let text = format!(
            "{} are one, but {what} differs on {day}: {}",
            and_list(&ids),
            and_list(&parts)
        );
        return Ok(Some(Hole::asked(&text, &ids.join(", "))));
    }
    Ok(None)
}

/// `a`, `a and b`, `a, b and c`: how `HOLE-SAME` names a class's rows and values (refcore's
/// `and_list`). L §8.1 (tally 2.1.8 S6).
#[must_use]
pub(crate) fn and_list(items: &[String]) -> String {
    match items.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        Some((last, _)) => last.clone(),
        None => String::new(),
    }
}

/// The legs' amounts as they print where every one is a number and they don't all agree
/// (refcore's `vkey` compare). L §5.19.
fn amounts_differ(m: &Reality, class: &[(&str, &LegVal)]) -> Result<Option<Vec<String>>, Stop> {
    let mut figures: Vec<(&Num, &str)> = Vec::with_capacity(class.len());
    for (_, lg) in class {
        let Amount::Qty(amount) = &lg.amount else {
            return Ok(None);
        };
        let Some(unit) = &lg.unit else {
            return Err(invariant("an observed leg's amount with no unit"));
        };
        figures.push((amount, unit));
    }
    if figures.iter().all(|f| Some(f) == figures.first()) {
        return Ok(None);
    }
    let mut texts = Vec::with_capacity(figures.len());
    for (amount, unit) in figures {
        texts.push(shown(m, amount, unit)?);
    }
    Ok(Some(texts))
}

/// An amount in its unit as L §2.4 prints it. L §2.4 Values.
pub(super) fn shown(m: &Reality, amount: &Num, unit: &str) -> Result<String, Stop> {
    qty_value(m, amount, unit)?.text(m).map_err(num_stop)
}

/// The legs' days as they print where every one is a day and they don't all agree. L §5.19.
fn days_differ(class: &[(&str, &LegVal)]) -> Option<Vec<String>> {
    let mut days = Vec::with_capacity(class.len());
    for (_, lg) in class {
        let Dated::Day(day) = &lg.day else {
            return None;
        };
        days.push(*day);
    }
    if days.iter().all(|d| Some(d) == days.first()) {
        return None;
    }
    Some(days.into_iter().map(Day::text).collect())
}

/// One leg of a realised movement as its rows build it (refcore's `m_c_observed`, a leg).
#[derive(Debug, Default)]
struct Realised {
    /// Its `from`: the first row's, none where their legs name different places.
    from: Option<Value>,
    /// Its `to`, likewise.
    to: Option<Value>,
    /// The sum of its amounts, in its unit.
    amount: Option<(Num, String)>,
    /// A hole amount among them.
    hole: Option<Hole>,
    /// The last of its days.
    day: Option<Day>,
}

impl Realised {
    /// Adds one row's leg `lg`. L §5.19 Realised amounts.
    fn add(&mut self, lg: &LegVal) -> Result<(), Stop> {
        let end = |e: &End| match e {
            End::Place(p) => Value::Key(KeyVal::Thing(p.clone())),
            End::Hole(h) => Value::Hole(h.clone()),
        };
        let (from, to) = (end(&lg.from), end(&lg.to));
        let first = self.from.is_none() && self.to.is_none();
        if first || self.from.as_ref().is_some_and(|f| *f != from) {
            self.from = Some(if first { from } else { Value::None });
        }
        if first || self.to.as_ref().is_some_and(|t| *t != to) {
            self.to = Some(if first { to } else { Value::None });
        }
        match (&lg.amount, &lg.unit, &self.amount) {
            (Amount::Refused(r), _, _) => return Err(Stop::Refused(r.clone())),
            (Amount::Hole(h), _, _) => self.hole = self.hole.take().or_else(|| Some(h.clone())),
            (Amount::Qty(q), Some(unit), None) => self.amount = Some((q.clone(), unit.clone())),
            (Amount::Qty(q), Some(_), Some((sum, unit))) => {
                let total = sum.checked_add(q).map_err(num_stop)?;
                self.amount = Some((total, unit.clone()));
            }
            (Amount::Qty(_), None, _) => return Err(invariant("an observed amount with no unit")),
        }
        if let Some(d) = lg.day.day() {
            self.day = Some(self.day.map_or(d, |last| last.max(d)));
        }
        Ok(())
    }
}

/// Leg `leg` of instance `tid`'s realised movement, over its observations `obs` grouped
/// `once` by the `same` groups (refcore's `m_c_observed`, a leg). L §5.19 Realised amounts,
/// One movement on two statements.
fn realised_leg(
    m: &Reality,
    tid: &str,
    (obs, once): (&[Obs], &[(usize, Vec<usize>)]),
    leg: &str,
) -> Result<Value, Stop> {
    let mut built = Realised::default();
    let mut differs = None;
    let mut met = false;
    for (first, rest) in once {
        let row = obs
            .get(*first)
            .ok_or_else(|| invariant("a same group with no first row"))?;
        let others: Vec<&Obs> = rest.iter().filter_map(|&k| obs.get(k)).collect();
        if differs.is_none() {
            differs = same_differ(m, row, &others, Some(leg))?;
        }
        for lg in row.legs.iter().filter(|lg| lg.name == leg) {
            met = true;
            built.add(lg)?;
        }
    }
    if !met {
        // an observation that lacks a leg of the instance it fulfils is refused before here
        return Err(Stop::Refused(Refusal::new(
            Code::Run(Run::Record),
            format!("the observations fulfilling {tid} lack its leg {leg}"),
        )));
    }
    let amount = match (differs.or(built.hole), built.amount) {
        (Some(h), _) => Value::Hole(h),
        (None, Some((sum, unit))) => qty_value(m, &sum, &unit)?,
        (None, None) => return Err(invariant("a realised leg with no amount")),
    };
    let day = built
        .day
        .map(Value::Day)
        .ok_or_else(|| invariant("a realised leg with no day"))?;
    Ok(Value::Record(vec![
        ("from".to_owned(), built.from.unwrap_or(Value::None)),
        ("to".to_owned(), built.to.unwrap_or(Value::None)),
        ("amount".to_owned(), amount),
        ("day".to_owned(), day),
    ]))
}

/// Row `o` refused `err` unless it was refused already: a row keeps the first refusal it
/// meets (refcore's `unjoined_refused`, `o.err is None`). L §5.19.
fn refuse_once(bad: &mut Vec<(String, Refusal)>, o: &Obs, err: &Refusal) {
    if !bad.iter().any(|(id, _)| *id == o.id) {
        bad.push((o.id.clone(), err.clone()));
    }
}

/// Whether some place of `one` and some place of `two` differ: each version covers a
/// different one of them. L §5.19.
fn some_differ(one: &BTreeSet<String>, two: &BTreeSet<String>) -> bool {
    for x in one {
        if two.iter().any(|y| y != x) {
            return true;
        }
    }
    false
}

/// The places an observed row's legs name. L §5.19.
fn row_places(o: &Obs) -> BTreeSet<&str> {
    o.legs
        .iter()
        .flat_map(|lg| [lg.from.place(), lg.to.place()])
        .flatten()
        .collect()
}

impl Engine {
    /// `observed(m[k])`: the realised movement of planned instance `v`, a record whose legs
    /// sum the legs of the observations that fulfil it, rows stated `same` counted once and
    /// their disagreement a leg's `HOLE-SAME` amount, a `from` or `to` none where their legs
    /// name different places, and whose `day` is the last of their days; none when nothing
    /// fulfils it (refcore's `m_c_observed`). L §5.19 Realised amounts, One movement on two
    /// statements.
    ///
    /// # Errors
    ///
    /// The refusal of a record its realised movement rests on; `RUN-RECORD` where its
    /// observations lack one of its legs.
    pub(crate) fn observed_of(&mut self, m: &Reality, v: &MvRef, ctx: Ctx) -> Result<Value, Stop> {
        let cut = ctx.cut;
        let links = self.obs_links_for(m, &v.id, cut)?;
        if let Some(bad) = self.bad_links.get(&(cut, v.id.clone())) {
            return Err(Stop::Refused(bad.clone()));
        }
        if links.is_empty() {
            return Ok(Value::None);
        }
        let all = self.observations(m, cut)?;
        // in the observations' order, as the links are
        let obs: Vec<Obs> = all
            .iter()
            .filter(|o| links.contains(&o.id))
            .cloned()
            .collect();
        for o in &obs {
            self.absorb(&o.coll);
        }
        let name = v.id.split_once('[').map_or(v.id.as_str(), |(n, _)| n);
        let mv = m
            .moves
            .get(name)
            .ok_or_else(|| invariant("an observed movement of a move load didn't build"))?;
        let groups = self.same_groups(m, ctx.cut)?;
        let once = once_obs(&groups, &obs);
        let mut fields = Vec::new();
        for (leg, _, _) in m.kind_legs(&mv.kind) {
            let value = realised_leg(m, &v.id, (&obs, &once), &leg)?;
            fields.push((leg, value));
        }
        let day = obs
            .iter()
            .map(|o| o.day)
            .max()
            .ok_or_else(|| invariant("a realised movement with no observation"))?;
        fields.push(("day".to_owned(), Value::Day(day)));
        Ok(Value::Record(fields))
    }

    /// Of the observations `obs` a ledger at `place` lists, the row of a `same` group whose
    /// end there another member books is left out, so a movement whose rows agree there is
    /// listed once; where they disagree there, the group is listed once, by its first row
    /// met, as the disagreement's hole; each row kept that is stated `same`, with its group's
    /// ids among `obs`, sorted, as its line's evidence (refcore's `m_residuals`, its
    /// `also`). E §6.3, L §6.1.
    ///
    /// # Errors
    ///
    /// As [`Engine::main_walk`].
    pub(crate) fn same_once_at(
        &mut self,
        m: &Reality,
        place: &str,
        obs: Vec<Obs>,
        cut: Day,
    ) -> Result<(Vec<Obs>, BTreeMap<String, String>), Stop> {
        let mut also = BTreeMap::new();
        let groups = self.same_groups(m, cut)?;
        if groups.is_empty() {
            return Ok((obs, also));
        }
        let id = self.main_walk(m, cut)?;
        let present: BTreeSet<String> = obs.iter().map(|o| o.id.clone()).collect();
        let mut keep = Vec::with_capacity(obs.len());
        // the groups listed already as their disagreement's hole, by their first member
        let mut holed: BTreeSet<&str> = BTreeSet::new();
        for o in &obs {
            let Some(group) = groups.get(&o.id) else {
                keep.push(o.clone());
                continue;
            };
            // a row stated same shows at the place only where its own statement books its
            // end there; where it doesn't, another row's statement does, or none (L §5.19:
            // each end is booked once, from the statement that covers it)
            if !self.books_ends_at(m, o, place, cut) {
                continue;
            }
            let row = match self.end_book_at(m, id, o, place)? {
                EndBook::Skip => continue,
                EndBook::Written | EndBook::Wait(_) => o.clone(),
                // the walk books the movement there once, as the hole: listed once with it
                EndBook::Hole(hole) => {
                    let lead = group.first().map_or(o.id.as_str(), String::as_str);
                    if !holed.insert(lead) {
                        continue;
                    }
                    holed_at(o, place, &hole)
                }
            };
            // sorted by code point (refcore's `sorted`)
            let ids: BTreeSet<&str> = group
                .iter()
                .filter(|x| present.contains(*x))
                .map(String::as_str)
                .collect();
            let ids: Vec<&str> = ids.into_iter().collect();
            also.insert(o.id.clone(), ids.join(", "));
            keep.push(row);
        }
        Ok((keep, also))
    }

    /// Whether row `o` books each of its ends at `place` by its own coverage: each on a day
    /// no version covers the place, or its own version does (refcore's `m_residuals`, its
    /// `obs_end_on` test). E §6.3, §6.1; L §5.19 Coverage, One movement on two statements.
    fn books_ends_at(&mut self, m: &Reality, o: &Obs, place: &str, cut: Day) -> bool {
        for lg in &o.legs {
            for end in [&lg.from, &lg.to] {
                if end.place() == Some(place) && !self.obs_end_on(m, o, place, o.day, cut) {
                    return false;
                }
            }
        }
        true
    }

    /// What row `o`'s end at `place` books under the `same` lines, as walk `id` books it:
    /// nothing where another member of its group books it, the group's `HOLE-SAME` where the
    /// members disagree there, else as written; a refusal there leaves it as written
    /// (refcore's `m_residuals`, its `consol`). E §6.3, §6.1; L §6.1.
    fn end_book_at(
        &mut self,
        m: &Reality,
        id: usize,
        o: &Obs,
        place: &str,
    ) -> Result<EndBook, Stop> {
        let mover = Member::Obs(o.clone());
        for lg in &o.legs {
            let books = match self.same_ends(m, id, (&mover, lg)) {
                Ok(books) => books,
                Err(Stop::Refused(_)) => continue,
                Err(stop) => return Err(stop),
            };
            for (end, book) in [&lg.from, &lg.to].into_iter().zip(books) {
                if end.place() != Some(place) {
                    continue;
                }
                if let EndBook::Skip | EndBook::Hole(_) = book {
                    return Ok(book);
                }
            }
        }
        Ok(EndBook::Written)
    }

    /// Of the observations `good` whose links to instance `tid` stand, those two unjoined
    /// rows of one transfer make: each refused `RUN-RECORD <r> and <r'> state one transfer
    /// and both fulfil <instance>: state them same` (refcore's `unjoined_refused`). L §5.19
    /// One movement on two statements, §8.4.
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`].
    pub(crate) fn unjoined(
        &mut self,
        m: &Reality,
        tid: &str,
        good: &[Obs],
        cut: Day,
    ) -> Result<Vec<(String, Refusal)>, Stop> {
        let mut bad: Vec<(String, Refusal)> = Vec::new();
        if good.len() < 2 {
            return Ok(bad);
        }
        let groups = self.same_groups(m, cut)?;
        for (i, a) in good.iter().enumerate() {
            for b in good.iter().skip(i).skip(1) {
                if !self.one_transfer(m, &groups, (a, b), cut) {
                    continue;
                }
                let err = Refusal::new(
                    Code::Run(Run::Record),
                    format!(
                        "{} and {} state one transfer and both fulfil {tid}: state them same",
                        a.id, b.id
                    ),
                );
                refuse_once(&mut bad, a, &err);
                refuse_once(&mut bad, b, &err);
            }
        }
        Ok(bad)
    }

    /// Of rows `good`, which fulfil `tid`, those stated `same` as a row whose own `fulfils`
    /// names another instance, each refused `RUN-RECORD` naming both rows in id order: a
    /// movement on two statements fulfils what is stated on either row or on both alike, and
    /// what reads either instance's realised movement rests on the refusal (refcore's
    /// `same_fulfils_differ`). L §5.19 One movement on two statements, §8.4 (tally 2.1.8 S1).
    ///
    /// # Errors
    ///
    /// As [`same_line_seen`], and the observations'.
    pub(crate) fn fulfils_differ(
        &mut self,
        m: &Reality,
        tid: &str,
        good: &[Obs],
        cut: Day,
    ) -> Result<Vec<(String, Refusal)>, Stop> {
        let mut bad: Vec<(String, Refusal)> = Vec::new();
        let groups = self.same_groups(m, cut)?;
        if !good.iter().any(|o| groups.contains_key(&o.id)) {
            return Ok(bad);
        }
        // the rows with a `fulfils` of their own that link another instance
        let seen = self.observations(m, cut)?;
        let mut others: Vec<(String, String)> = Vec::new();
        for x in seen.iter().filter(|x| !x.shared) {
            if let Some(t) = x.target_id()
                && t.as_str() != tid
            {
                others.push((x.id.clone(), t));
            }
        }
        for o in good {
            let Some(group) = groups.get(&o.id) else {
                continue;
            };
            for (x, t2) in others.iter().filter(|(x, _)| group.contains(x)) {
                let ((a, ta), (b, tb)) = if o.id <= *x {
                    ((o.id.as_str(), tid), (x.as_str(), t2.as_str()))
                } else {
                    ((x.as_str(), t2.as_str()), (o.id.as_str(), tid))
                };
                let err = Refusal::new(
                    Code::Run(Run::Record),
                    format!("{a} and {b} are stated same and fulfil {ta} and {tb}"),
                );
                self.obs_errs
                    .entry((cut, x.clone()))
                    .or_insert_with(|| err.clone());
                for t in [tid, t2.as_str()] {
                    self.bad_links
                        .entry((cut, t.to_owned()))
                        .or_insert_with(|| err.clone());
                }
                // S1's reach: every read resting on the class or on either instance (P42)
                for k in [tid, t2.as_str(), o.id.as_str(), x.as_str()] {
                    self.s1_links
                        .entry((cut, k.to_owned()))
                        .or_insert_with(|| err.clone());
                }
                refuse_once(&mut bad, o, &err);
            }
        }
        Ok(bad)
    }

    /// Whether rows `a` and `b` of different versions, not stated `same`, name the same two
    /// places, each version covering a different one of them: each would book one end and
    /// count the whole (refcore's `unjoined_refused`, its test). L §5.19.
    fn one_transfer(
        &mut self,
        m: &Reality,
        groups: &BTreeMap<String, Vec<String>>,
        (a, b): (&Obs, &Obs),
        cut: Day,
    ) -> bool {
        if a.version == b.version || groups.get(&a.id).is_some_and(|g| g.contains(&b.id)) {
            return false;
        }
        let places = row_places(a);
        if places.len() != 2 || row_places(b) != places {
            return false;
        }
        let (in_a, in_b) = (
            self.row_covers(m, a, &places, cut),
            self.row_covers(m, b, &places, cut),
        );
        some_differ(&in_a, &in_b)
    }

    /// Of places `places`, those row `o`'s own version covers on its day, where it has a leg
    /// (refcore's `unjoined_refused`, `covered`). L §5.19 Coverage.
    fn row_covers(
        &mut self,
        m: &Reality,
        o: &Obs,
        places: &BTreeSet<&str>,
        cut: Day,
    ) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        if o.legs.is_empty() {
            return out;
        }
        for p in places {
            if self.covering(m, p, o.day, cut).contains(&o.version) {
                out.insert((*p).to_owned());
            }
        }
        out
    }
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
            kept_once(&groups, &group, &["c", "b", "z"]),
            vec![false, true, true],
            "the first side among the ids is kept"
        );
        let sides = vec!["c".to_owned(), "a".to_owned(), "b".to_owned()];
        assert_eq!(
            kept_once(&groups, &sides, &["a", "b", "c"]),
            vec![false, false, true],
            "the first in the sides' order, which the caller gives by id"
        );
        assert_eq!(
            kept_once(&groups, &group, &["c"]),
            vec![true],
            "a lone member is kept"
        );
        let mut parent = BTreeMap::new();
        parent.insert("b".to_owned(), "a".to_owned());
        parent.insert("c".to_owned(), "b".to_owned());
        assert_eq!(root(&parent, "c"), "a", "the root of a chain");
    }
}
