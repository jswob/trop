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

use std::collections::{BTreeMap, BTreeSet};

use super::observe::Obs;
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Engine, Entry, num_stop};
use crate::hole::Hole;
use crate::model::{Reality, SamePair, Value};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{
    ALL, Amount, At, Change, Dated, Ends, Ent, Exists, Inst, LegVal, Status, Step, invariant,
    qty_value,
};

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

/// What one end of a leg books under the `same` lines (refcore's `consol`). L §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EndBook {
    /// As written: no other member of its group books that place.
    Written,
    /// Nothing here: the group's latest member there books it.
    Skip,
    /// At the point of the decision named, which hasn't run yet.
    Wait(String),
    /// The group's disagreement there, a hole movement (`HOLE-SAME`).
    Hole(Hole),
}

/// A member of a `same` group as the walk meets it (refcore's `movement`, `decs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Member {
    /// An observed row.
    Obs(Obs),
    /// A plan's instance.
    Plan(Inst),
    /// A decision's instance: its point once it has run, none before.
    Decision(Inst, Option<At>),
}

impl Member {
    /// Its stable id. L §2.5.
    fn id(&self) -> &str {
        match self {
            Member::Obs(o) => &o.id,
            Member::Plan(i) | Member::Decision(i, _) => &i.id,
        }
    }

    /// Its legs. L §5.14 Legs.
    fn legs(&self) -> &[LegVal] {
        match self {
            Member::Obs(o) => &o.legs,
            Member::Plan(i) | Member::Decision(i, _) => &i.legs,
        }
    }

    /// Its day, a window's first (refcore's `day_lo`). L §4.4.
    fn day(&self) -> Option<Day> {
        match self {
            Member::Obs(o) => Some(o.day),
            Member::Plan(i) | Member::Decision(i, _) => i.day.as_ref().and_then(Dated::lo),
        }
    }
}

/// An end waiting for a decision's point: the movement, its leg, the side, what it carries.
pub(crate) type Waiting = (String, LegVal, usize, Vec<Entry>, Collected);

/// Whether movement `id` is named by a `same` line. L §6.1.
#[must_use]
pub(crate) fn stated_same(m: &Reality, id: &str) -> bool {
    m.sames
        .iter()
        .any(|pair| matches!(pair, SamePair::Moves(a, b) if a == id || b == id))
}

/// The first `same` line naming movement `id`, its two sides (refcore's `same_pairs`, its
/// lines). L §6.1.
fn first_line(m: &Reality, id: &str) -> Option<(String, String)> {
    m.sames.iter().find_map(|pair| match pair {
        SamePair::Moves(a, b) if a == id || b == id => Some((a.clone(), b.clone())),
        SamePair::Moves(..) | SamePair::Rows(..) => None,
    })
}

/// A leg's amount as a `HOLE-SAME` question prints it: the amount, `refused`, or the hole.
/// L §8.1.
fn amount_text(m: &Reality, lg: &LegVal) -> Result<String, Stop> {
    match (&lg.amount, &lg.unit) {
        (Amount::Qty(q), Some(unit)) => shown(m, q, unit),
        (Amount::Qty(_), None) => Err(invariant("an amount with no unit")),
        (Amount::Refused(_), _) => Ok("refused".to_owned()),
        (Amount::Hole(h), _) => Value::Hole(h.clone()).text(m).map_err(num_stop),
    }
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
    /// Of the observations `good` whose links to instance `tid` stand, those two unjoined
    /// rows of one transfer make: each refused `RUN-RECORD <r> and <r'> state one transfer
    /// and both fulfil <instance>: state them same` (refcore's `unjoined_refused`). L §5.19
    /// One movement on two statements, §8.4.
    ///
    /// # Errors
    ///
    /// As [`Engine::covering`].
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
        let groups = same_groups(m);
        for (i, a) in good.iter().enumerate() {
            for b in good.iter().skip(i).skip(1) {
                if !self.one_transfer(m, &groups, (a, b), cut)? {
                    continue;
                }
                let err = Refusal::new(
                    Code::Run(Run::Record),
                    format!(
                        "{} and {} state one transfer and both fulfil {tid}: state them same",
                        a.id, b.id
                    ),
                );
                // a row keeps the first refusal it meets
                let fresh: Vec<&Obs> = [a, b]
                    .into_iter()
                    .filter(|o| !bad.iter().any(|(id, _)| *id == o.id))
                    .collect();
                bad.extend(fresh.into_iter().map(|o| (o.id.clone(), err.clone())));
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
    ) -> Result<bool, Stop> {
        if a.version == b.version || groups.get(&a.id).is_some_and(|g| g.contains(&b.id)) {
            return Ok(false);
        }
        let places = row_places(a);
        if places.len() != 2 || row_places(b) != places {
            return Ok(false);
        }
        let (in_a, in_b) = (
            self.row_covers(m, a, &places, cut)?,
            self.row_covers(m, b, &places, cut)?,
        );
        Ok(in_a.iter().any(|x| in_b.iter().any(|y| x != y)))
    }

    /// Of places `places`, those row `o`'s own version covers on its legs' days (refcore's
    /// `unjoined_refused`, `covered`). L §5.19 Coverage.
    fn row_covers(
        &mut self,
        m: &Reality,
        o: &Obs,
        places: &BTreeSet<&str>,
        cut: Day,
    ) -> Result<BTreeSet<String>, Stop> {
        let mut out = BTreeSet::new();
        for lg in &o.legs {
            let day = lg.day.day().unwrap_or(o.day);
            for p in places {
                if self.covering(m, p, day, cut)?.contains(&o.version) {
                    out.insert((*p).to_owned());
                }
            }
        }
        Ok(out)
    }

    /// The plan instances and decisions stated `same` as an observation the cut sees: the
    /// group is one movement, observed, so none of them is unfulfilled or open (refcore's
    /// `same_observed`). L §6.1, §5.19.
    ///
    /// # Errors
    ///
    /// As [`Engine::observations`].
    pub(crate) fn same_observed_ids(
        &mut self,
        m: &Reality,
        cut: Day,
    ) -> Result<BTreeSet<String>, Stop> {
        let groups = same_groups(m);
        if groups.is_empty() {
            return Ok(BTreeSet::new());
        }
        let observed: BTreeSet<String> = self
            .observations(m, cut)?
            .iter()
            .map(|o| o.id.clone())
            .collect();
        Ok(groups
            .into_iter()
            .filter(|(x, group)| group.iter().any(|y| y != x && observed.contains(y)))
            .map(|(x, _)| x)
            .collect())
    }

    /// The ends of leg `lg` of `mover` its booking keeps under the `same` lines: an end the
    /// group's latest member books instead, or one waiting for a decision's point, is dropped;
    /// one where the members disagree takes the `HOLE-SAME` hole (refcore's `book_leg`, its
    /// `consol`). L §6.1 (a place two of them book takes it once), §5.19, E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// `RUN-RECORD` for a `same` side that doesn't exist; a refusal reading a waiting
    /// decision's places.
    pub(crate) fn same_mask(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg): (&Member, &LegVal),
        at: At,
        (reg, coll): (&[Entry], &Collected),
        mut mask: Ends,
    ) -> Result<Ends, Stop> {
        let books = self.same_ends(m, id, (mover, lg))?;
        for (side, book) in books.into_iter().enumerate() {
            let (on, end) = if side == 0 {
                (&mut mask.from, &lg.from)
            } else {
                (&mut mask.to, &lg.to)
            };
            if !*on {
                continue;
            }
            match book {
                EndBook::Written => continue,
                EndBook::Skip => {}
                EndBook::Wait(decision) => {
                    let waiting = (
                        mover.id().to_owned(),
                        lg.clone(),
                        side,
                        reg.to_vec(),
                        coll.clone(),
                    );
                    let walk = self.walk_mut(id)?;
                    walk.same_waiting.entry(decision).or_default().push(waiting);
                }
                EndBook::Hole(h) => {
                    let Some(p) = end.place() else {
                        continue;
                    };
                    let mut ent = Ent::new(at, Change::Hole(h));
                    ent.mv = Some(mover.id().to_owned());
                    ent.leg = Some(lg.name.clone());
                    ent.reg = reg.to_vec();
                    ent.coll = Some(coll.clone());
                    let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
                    let walk = self.walk_mut(id)?;
                    walk.worth_add(&unit, None)?;
                    walk.add(m, p, &unit, ent)?;
                }
            }
            *on = false;
        }
        Ok(mask)
    }

    /// Instance `inst` as a member of its `same` group: a decision with its point once this
    /// walk ran it, else a plan's instance (refcore's `consol`, its `mv`). L §6.1.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn mover_of(&self, m: &Reality, id: usize, inst: &Inst) -> Result<Member, Stop> {
        if !m.moves.get(&inst.move_id).is_some_and(|mv| mv.decision) {
            return Ok(Member::Plan(inst.clone()));
        }
        let walk = self.walk(id)?;
        let point = walk
            .run_ids
            .get(&inst.id)
            .and_then(|&ix| walk.runs.get(ix))
            .map(|run| run.point);
        Ok(Member::Decision(inst.clone(), point))
    }

    /// What each end of leg `lg` of `mover` books under the `same` lines (refcore's
    /// `consol`). L §6.1 What `same` does, §8.4 (`same <p> = <q>: <p> doesn't exist`).
    fn same_ends(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg): (&Member, &LegVal),
    ) -> Result<[EndBook; 2], Stop> {
        let groups = same_groups(m);
        let Some(group) = groups.get(mover.id()) else {
            return Ok([EndBook::Written, EndBook::Written]);
        };
        let decides = matches!(mover, Member::Decision(..));
        let (mut others, mut waiting) = (Vec::new(), Vec::new());
        for x in group.iter().filter(|x| x.as_str() != mover.id()) {
            let Some(found) = self.walk_member(m, id, x)? else {
                let (left, right) =
                    first_line(m, x).unwrap_or_else(|| (x.clone(), mover.id().to_owned()));
                return Err(Stop::Refused(Refusal::new(
                    Code::Run(Run::Record),
                    format!("same {left} = {right}: {x} doesn't exist"),
                )));
            };
            if matches!(found, Member::Decision(_, None)) && !decides {
                waiting.push(found);
            } else {
                others.push(found);
            }
        }
        let mut out = [EndBook::Written, EndBook::Written];
        for (side, (end, slot)) in [&lg.from, &lg.to]
            .into_iter()
            .zip(out.iter_mut())
            .enumerate()
        {
            let Some(p) = end.place() else {
                continue;
            };
            *slot = match self.same_wait_for(m, id, (mover, lg), p, &waiting)? {
                Some(wait) => wait,
                None => self.same_end(m, id, (mover, lg, side), p, (&others, group))?,
            };
        }
        Ok(out)
    }

    /// A member of a `same` group by id: a decision as this walk ran it, else a plan's or a
    /// decision's instance, else an observation; none where none is (refcore's `decs`,
    /// `movement`). L §6.1.
    fn walk_member(&mut self, m: &Reality, id: usize, x: &str) -> Result<Option<Member>, Stop> {
        let walk = self.walk(id)?;
        if let Some(run) = walk.run_ids.get(x).and_then(|&ix| walk.runs.get(ix)) {
            return Ok(Some(Member::Decision(run.inst.clone(), Some(run.point))));
        }
        let cut = walk.cut;
        let name = x.split_once('[').map_or(x, |(n, _)| n);
        if let Some(mv) = m.moves.get(name) {
            let insts = self.plan_list(m, mv, cut)?;
            if let Some(inst) = insts.iter().find(|i| i.id == x) {
                let found = if mv.decision {
                    Member::Decision(inst.clone(), None)
                } else {
                    Member::Plan(inst.clone())
                };
                return Ok(Some(found));
            }
        }
        let rows = self.observations(m, cut)?;
        Ok(rows.iter().find(|o| o.id == x).cloned().map(Member::Obs))
    }

    /// An end at place `p` of a movement stated `same` as a decision not yet run: waiting for
    /// the decision's point where it books `p` that day, the `HOLE-SAME` hole where their days
    /// differ, none where no waiting decision books `p` (refcore's `same_wait_for`). L §6.1
    /// (a decision's amount is known only at its point), §5.14.2.
    fn same_wait_for(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg): (&Member, &LegVal),
        p: &str,
        waiting: &[Member],
    ) -> Result<Option<EndBook>, Stop> {
        for dec in waiting {
            let Member::Decision(inst, _) = dec else {
                continue;
            };
            let Some(dday) = inst.day.as_ref().and_then(Dated::lo) else {
                return Ok(None);
            };
            if !self.decision_books(m, id, inst, (dday, p))? {
                continue;
            }
            if lg.day == Dated::Day(dday) {
                return Ok(Some(EndBook::Wait(inst.id.clone())));
            }
            let text = format!(
                "{} and {} are one, but the leg at {p} differs on {}: {} ({}) and its day {} ({})",
                mover.id(),
                inst.id,
                lg.day.text(),
                amount_text(m, lg)?,
                mover.id(),
                dday.text(),
                inst.id
            );
            let figure = format!("{}, {}", mover.id(), inst.id);
            return Ok(Some(EndBook::Hole(Hole::asked(&text, &figure))));
        }
        Ok(None)
    }

    /// Whether decision instance `inst` books place `p` on its day: the places its legs give
    /// there, every place where one can't be read (refcore's `inst_places`, its changes).
    /// L §5.14.2, §6.1.
    fn decision_books(
        &mut self,
        m: &Reality,
        id: usize,
        inst: &Inst,
        (day, p): (Day, &str),
    ) -> Result<bool, Stop> {
        let mv = m
            .moves
            .get(&inst.move_id)
            .ok_or_else(|| invariant("a decision of a move load didn't build"))?;
        let cut = self.walk(id)?.cut;
        let ctx = self.timeline_ctx(inst.law.first(), day.min(cut));
        self.push();
        let read = self.leg_places(m, mv, inst, ctx);
        self.pop_alone();
        match read {
            Ok(Some(places)) => Ok(places.contains(p)),
            // a place that can't be read here meets every place
            Ok(None) | Err(Stop::Refused(_)) => Ok(true),
            Err(stop) => Err(stop),
        }
    }

    /// What place `p` takes from `mover`'s end there: as written where no other member
    /// books it; once, by the latest member in the day's order, where they agree; the
    /// `HOLE-SAME` hole where they give it different amounts or days (refcore's `consol`).
    /// L §6.1 What `same` does, §8.1.
    fn same_end(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg, side): (&Member, &LegVal, usize),
        p: &str,
        (others, group): (&[Member], &[String]),
    ) -> Result<EndBook, Stop> {
        let mut touch: Vec<(&Member, &LegVal, usize)> = vec![(mover, lg, side)];
        for o in others {
            if matches!(o, Member::Plan(i) if i.status == Status::Modelled) {
                // a plan instance modelled past is never booked (L §5.19 Coverage)
                continue;
            }
            for (li, sj) in self.touching(m, id, o, (p, lg.unit.as_deref()))? {
                let g = o
                    .legs()
                    .get(li)
                    .ok_or_else(|| invariant("a leg the member lacks"))?;
                touch.push((o, g, sj));
            }
        }
        let mut movers: Vec<&str> = touch.iter().map(|(x, _, _)| x.id()).collect();
        movers.dedup();
        if movers.len() < 2 {
            return Ok(EndBook::Written);
        }
        let agree = touch
            .iter()
            .skip(1)
            .all(|(_, g, sj)| *sj == side && g.amount == lg.amount && g.day == lg.day);
        if agree {
            let mut latest: Option<(Option<At>, &str)> = None;
            for (x, g, _) in &touch {
                let key = self.book_key(id, x, g)?;
                if latest.as_ref().is_none_or(|(best, _)| key > *best) {
                    latest = Some((key, x.id()));
                }
            }
            let mine = latest.is_some_and(|(_, x)| x == mover.id());
            return Ok(if mine {
                EndBook::Written
            } else {
                EndBook::Skip
            });
        }
        // the members in the order the same lines write them
        touch.sort_by_key(|(x, _, _)| group.iter().position(|y| y.as_str() == x.id()));
        let ids: Vec<&str> = touch.iter().map(|(x, _, _)| x.id()).collect();
        let days: BTreeSet<Day> = touch.iter().filter_map(|(_, g, _)| g.day.day()).collect();
        let days: Vec<String> = days.into_iter().map(Day::text).collect();
        let mut parts = Vec::with_capacity(touch.len());
        for (x, g, _) in &touch {
            parts.push(format!("{} ({})", amount_text(m, g)?, x.id()));
        }
        let text = format!(
            "{} are one, but the leg at {p} differs on {}: {}",
            ids.join(" and "),
            days.join(", "),
            parts.join(" and ")
        );
        Ok(EndBook::Hole(Hole::asked(&text, &ids.join(", "))))
    }

    /// The legs of member `o` that touch place `p` in unit `unit`, by position, each with
    /// its side; an observed row's only where it books that end (refcore's `consol`, its
    /// `touch`). L §6.1, §5.19 Booking an observation.
    fn touching(
        &mut self,
        m: &Reality,
        id: usize,
        o: &Member,
        (p, unit): (&str, Option<&str>),
    ) -> Result<Vec<(usize, usize)>, Stop> {
        let cut = self.walk(id)?.cut;
        let mut out = Vec::new();
        for (li, g) in o.legs().iter().enumerate() {
            if g.unit.as_deref() != unit {
                continue;
            }
            for (sj, end) in [&g.from, &g.to].into_iter().enumerate() {
                if end.place() != Some(p) {
                    continue;
                }
                if let Member::Obs(row) = o
                    && !self.obs_end_on(m, row, p, g.day.day().unwrap_or(row.day), cut)?
                {
                    continue;
                }
                out.push((li, sj));
            }
        }
        Ok(out)
    }

    /// Where member `x`'s leg `lg` is booked in the day's order: observations before plans,
    /// plans before decisions, a decision at its point (refcore's `same_book_key`). E §4.2
    /// steps 1–3, L §6.1.
    fn book_key(&self, id: usize, x: &Member, lg: &LegVal) -> Result<Option<At>, Stop> {
        let Some(day) = lg.day.day().or_else(|| x.day()) else {
            return Ok(None);
        };
        let walk = self.walk(id)?;
        Ok(Some(match x {
            Member::Obs(o) => At {
                rank: walk.obs_rank(&format!("{} {}", o.id, lg.name))?,
                ..At::of(day, Step::Observations)
            },
            Member::Decision(_, Some(point)) if point.day == day => *point,
            Member::Decision(..) => At::of(day, Step::Decisions),
            Member::Plan(i) => At {
                rank: walk.plan_rank(&i.order)?,
                ..At::of(day, Step::Plans)
            },
        }))
    }

    /// At decision run `ix`'s point, the ends that waited for it: refused where it doesn't
    /// exist; nothing where it books their place (its booking took the end once, or found the
    /// hole); else booked by their own movement now (refcore's `same_release`). L §6.1, §8.4.
    ///
    /// # Errors
    ///
    /// As [`Engine::book_leg_ends`].
    pub(crate) fn same_release(&mut self, m: &Reality, id: usize, ix: usize) -> Result<(), Stop> {
        let walk = self.walk_mut(id)?;
        if walk.same_waiting.is_empty() {
            return Ok(());
        }
        let run = walk
            .runs
            .get(ix)
            .ok_or_else(|| invariant("a run the walk lacks"))?;
        let Some(pending) = walk.same_waiting.remove(&run.inst.id) else {
            return Ok(());
        };
        let (run_id, point) = (run.inst.id.clone(), run.point);
        let none = run.inst.exists == Exists::No;
        let legs = if run.inst.status == Status::Modelled {
            Vec::new()
        } else {
            run.inst.legs.clone()
        };
        let after = point.with_sub(
            point
                .sub
                .checked_add(1)
                .ok_or_else(|| invariant("a point past the counted subpoints"))?,
        );
        for (mover, lg, side, reg, coll) in pending {
            if none {
                let err = Refusal::new(
                    Code::Run(Run::Record),
                    format!("same {mover} = {run_id}: {run_id} doesn't exist"),
                );
                self.walk_mut(id)?.leg_bad(m, &mover, &lg, &err, point)?;
                continue;
            }
            let end = if side == 0 { &lg.from } else { &lg.to };
            let Some(p) = end.place() else {
                continue;
            };
            let hit = legs.iter().any(|g| {
                g.unit == lg.unit && (g.from.place() == Some(p) || g.to.place() == Some(p))
            });
            if hit {
                continue;
            }
            let ends = Ends {
                from: side == 0,
                to: side == 1,
                whole: false,
            };
            self.book_leg_ends(m, id, (&mover, &lg), after, (&reg, &coll), ends)?;
        }
        Ok(())
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
