//! Movements stated `same`: one movement on two statements, counted once, and where its rows disagree a hole naming both.
//! Spec: L §6.1 (What `same` does: `same` is an equivalence, lines that share a side make one
//! group, a group of movements is one movement, bound once, counted once by `flows` and
//! `observed`), §5.19 (One movement on two statements: a read of the movement away from its
//! places is the disagreement's hole, `HOLE-SAME`, naming both rows), §8.1 (`HOLE-SAME`'s form
//! `<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)`), §4.5 (tally
//! 2.1.8 R12: a `same` line is seen under a cut as its version's rows are, one known in a
//! window that holds the cut read as seen and as unseen).
//! Twin: engine/tools/refcore/moves.py (`same_lines`, `same_groups`, `once_obs`,
//! `same_obs_differ`), model.py (`same_seen`) and mreads.py (`once_by_same`); refcore's
//! walk.py `same_observed` (an instance stated `same` as an observed row) serves only a joint
//! model's lines since load refuses the pair in one model (tally 2.1.8), and is built here from
//! it when joint models run (M5c).
//! Invariants: a group's members are in the order of their ids (tally 2.1.8 S6); a group is
//! counted by its first member a read meets, its other members beside it; rows disagree
//! only on a leg two of them state, never where any side is a hole; `HOLE-SAME` names every
//! row of the class that states the fact, in id order.

use std::collections::{BTreeMap, BTreeSet};

use super::observe::Obs;
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, num_stop};
use crate::hole::Hole;
use crate::model::{KeyVal, MvRef, Reality, SamePair, Value};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{
    ALL, Amount, At, Change, Dated, End, Ends, Ent, Exists, Inst, LegVal, Status, Step, invariant,
    qty_value,
};

/// The versions whose `same` line a line's first run read under a cut their `known` window
/// holds, no `windows each` deciding it: each with its window and the sides its lines name
/// (refcore's `_SAMEMET`). L §4.5 (tally 2.1.8 R12).
pub(crate) type SameMet = BTreeMap<String, ((Day, Day), BTreeSet<String>)>;

/// How one run of an ask's line reads the `same` lines known in a window that holds the
/// cut, which no `windows each` decides (L §4.5, tally 2.1.8 R12: such a line is read as
/// seen and as unseen): the line's first run takes each seen and notes it; a later run takes
/// each as its pick says (refcore's `_SAMEMET`, `_SAMEPICK`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SamePicks {
    /// Each version's line seen or not in this run; none in the line's first run.
    pub(crate) pick: Option<BTreeMap<String, bool>>,
    /// What the line's first run met.
    pub(crate) met: SameMet,
}

impl SamePicks {
    /// Whether this run sees the line of version `id`, known in `window`, which holds the cut
    /// and no `windows each` decides: as the pick says, a line it names no choice for seen;
    /// in the line's first run seen, noted with its window and the sides `sides` it names
    /// (refcore's `same_seen`). L §4.5 (tally 2.1.8 R12).
    fn seen(&mut self, id: &str, window: (Day, Day), sides: [&str; 2]) -> bool {
        if let Some(pick) = &self.pick {
            return pick.get(id) != Some(&false);
        }
        let (_, named) = self
            .met
            .entry(id.to_owned())
            .or_insert_with(|| (window, BTreeSet::new()));
        named.extend(sides.iter().map(|side| (*side).to_owned()));
        true
    }
}

/// Each movement stated `same`, by id, with its group's members in the order the same lines
/// first name them (L §6.1).
pub type Groups = BTreeMap<String, Vec<String>>;

impl Engine {
    /// Whether the cut of `ctx` sees a `same` line of version `version`, which names `sides`:
    /// a line in a joint model's header always; else only in the worlds its version's file is
    /// in, its version's `known` on or before the cut, a window that holds the cut decided by
    /// the line's `windows each`, else as the run's pick says, read as seen and as unseen
    /// (refcore's `same_seen`, `same_lines`). L §4.5 (tally 2.1.8 R12: whatever a version
    /// states is read as its rows are), §4.4, §6.4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a version load didn't build; as [`Engine::known_seen`].
    pub(crate) fn same_line_seen(
        &mut self,
        m: &Reality,
        version: Option<&str>,
        ctx: Ctx,
        sides: [&str; 2],
    ) -> Result<bool, Stop> {
        let Some(id) = version else {
            return Ok(true);
        };
        if !m.scenarios.sees(ctx.world, Some(id)) {
            return Ok(false);
        }
        let info = m
            .versions
            .get(id)
            .ok_or_else(|| invariant("a same line's version load didn't build"))?;
        let (lo, cut) = (info.known, ctx.cut);
        // a window that ends on or before the cut is seen, one that starts after it unseen
        let Some(hi) = info.window.filter(|hi| lo <= cut && cut < *hi) else {
            return Ok(lo <= cut);
        };
        if let Some(seen) = self.known_seen(ctx, (lo, hi), &info.id)? {
            return Ok(seen);
        }
        if self.chooses(ctx)? {
            // under `windows each`, a line not yet taken is an unknown day the line's answer
            // rests on: the run takes it as seen, and the line takes its days next (L §4.5,
            // §4.4; tally 2.1.8 R12, R13; refcore's `same_seen`, its `forced`)
            let forced = crate::eval::known_unknown((lo, hi), &info.id)?.id;
            if !self.each_forced.contains(&forced) {
                self.each_forced.push(forced);
            }
            return Ok(true);
        }
        Ok(self.same_picks.seen(&info.id, (lo, hi), sides))
    }

    /// Whether cut `cut` sees each `same` line of the reality, in the order written: a line
    /// of movements as [`Engine::same_line_seen`] decides, a line of rows never (its read is
    /// the row's, `eval/read.rs`). L §6.1, §4.5.
    ///
    /// # Errors
    ///
    /// As [`Engine::same_line_seen`].
    fn moves_seen(&mut self, m: &Reality, cut: Day) -> Result<Vec<bool>, Stop> {
        let ctx = self.timeline_ctx(None, cut);
        let mut out = Vec::with_capacity(m.sames.len());
        for pair in &m.sames {
            let seen = match pair {
                SamePair::Moves(left, right, version) => {
                    self.same_line_seen(m, version.as_deref(), ctx, [left, right])?
                }
                SamePair::Rows(..) => false,
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
    /// As [`Engine::same_line_seen`].
    pub(crate) fn same_groups(&mut self, m: &Reality, cut: Day) -> Result<Groups, Stop> {
        // a class joined by lines in two files has no written order: its members go by their
        // ids, compared by code point (L §2.5; tally 2.1.8 S6)
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

    /// Whether movement `id` is named by a `same` line cut `cut` sees. L §6.1, §4.5.
    ///
    /// # Errors
    ///
    /// As [`Engine::same_line_seen`].
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
    /// As [`Engine::same_line_seen`].
    fn first_line(
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

/// The `same` lines of movements of `m` that `seen` flags (one flag per line, as
/// [`Engine::moves_seen`] gives them), each its two sides, in the order written. L §6.1, §4.5.
fn move_lines(m: &Reality, seen: Vec<bool>) -> Vec<(&String, &String)> {
    let mut out = Vec::new();
    for (pair, flag) in m.sames.iter().zip(seen) {
        if let (SamePair::Moves(left, right, _), true) = (pair, flag) {
            out.push((left, right));
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
    for mine in &row.legs {
        if leg.is_some_and(|name| mine.name != name) {
            continue;
        }
        let mut class: Vec<(&Obs, &LegVal)> = vec![(row, mine)];
        for other in others {
            for theirs in other.legs.iter().filter(|lg| lg.name == mine.name) {
                class.push((other, theirs));
            }
        }
        if class.len() < 2 {
            continue;
        }
        class.sort_by(|a, b| a.0.id.cmp(&b.0.id));
        let (what, figures) = if let Some(figures) = amounts_differ(m, &class)? {
            (format!("the amount of {}", mine.name), figures)
        } else if let Some(figures) = days_differ(&class) {
            (format!("the day of {}", mine.name), figures)
        } else {
            continue;
        };
        let ids: Vec<String> = class.iter().map(|(o, _)| o.id.clone()).collect();
        let parts: Vec<String> = figures
            .iter()
            .zip(&ids)
            .map(|(figure, id)| format!("{figure} ({id})"))
            .collect();
        let day = mine.day.day().unwrap_or(row.day);
        let text = format!(
            "{} are one, but {what} differs on {}: {}",
            and_list(&ids),
            day.text(),
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
fn amounts_differ(m: &Reality, class: &[(&Obs, &LegVal)]) -> Result<Option<Vec<String>>, Stop> {
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
fn shown(m: &Reality, amount: &Num, unit: &str) -> Result<String, Stop> {
    qty_value(m, amount, unit)?.text(m).map_err(num_stop)
}

/// The legs' days as they print where every one is a day and they don't all agree. L §5.19.
fn days_differ(class: &[(&Obs, &LegVal)]) -> Option<Vec<String>> {
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
            if !self.books_ends_at(m, o, place, cut)? {
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
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_end_on`].
    fn books_ends_at(&mut self, m: &Reality, o: &Obs, place: &str, cut: Day) -> Result<bool, Stop> {
        for lg in &o.legs {
            let day = lg.day.day().unwrap_or(o.day);
            for end in [&lg.from, &lg.to] {
                if end.place() == Some(place) && !self.obs_end_on(m, o, place, day, cut)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
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
        let groups = self.same_groups(m, cut)?;
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
    /// As [`Engine::same_line_seen`], and the observations'.
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
        let others: Vec<(String, String)> = self
            .observations(m, cut)?
            .iter()
            .filter(|x| !x.shared)
            .filter_map(|x| x.target_id().map(|t| (x.id.clone(), t)))
            .filter(|(_, t)| t != tid)
            .collect();
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
        Ok(some_differ(&in_a, &in_b))
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

    /// The ends of leg `lg` of `mover` its booking keeps under the `same` lines: an end the
    /// group's latest member books instead, or one waiting for a decision's point, is dropped;
    /// one where the members disagree takes the `HOLE-SAME` hole (refcore's `book_leg`, its
    /// `consol`). L §6.1 (a place two of them book takes it once), §5.19, E §6.1 (one
    /// movement books each end once).
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
                    walk.same_waiting
                        .entry(decision)
                        .or_insert_with(Vec::new)
                        .push(waiting);
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
        let cut = self.walk(id)?.cut;
        let groups = self.same_groups(m, cut)?;
        let Some(group) = groups.get(mover.id()) else {
            return Ok([EndBook::Written, EndBook::Written]);
        };
        let decides = matches!(mover, Member::Decision(..));
        let (mut others, mut waiting) = (Vec::new(), Vec::new());
        for x in group.iter().filter(|x| x.as_str() != mover.id()) {
            let Some(found) = self.walk_member(m, id, x)? else {
                let (left, right) = self
                    .first_line(m, x, cut)?
                    .unwrap_or_else(|| (x.clone(), mover.id().to_owned()));
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
                None => self.same_end(m, id, (mover, lg, side), p, &others)?,
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
        others: &[Member],
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
        // the members in the order of their ids, every one named (tally 2.1.8 S6)
        let mut ordered = touch;
        ordered.sort_by(|a, b| a.0.id().cmp(b.0.id()));
        let ids: Vec<String> = ordered.iter().map(|(x, _, _)| x.id().to_owned()).collect();
        let days: BTreeSet<Day> = ordered.iter().filter_map(|(_, g, _)| g.day.day()).collect();
        let days: Vec<String> = days.into_iter().map(Day::text).collect();
        let mut parts = Vec::with_capacity(ordered.len());
        for (x, g, _) in &ordered {
            parts.push(format!("{} ({})", amount_text(m, g)?, x.id()));
        }
        let text = format!(
            "{} are one, but the leg at {p} differs on {}: {}",
            and_list(&ids),
            days.join(", "),
            and_list(&parts)
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
