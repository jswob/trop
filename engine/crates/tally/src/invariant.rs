//! E §9 invariants 3 and 4: a scenario changes nothing before its from, and asks move no money.
//! Spec: E §9 item 4 ("Asks move no money. The timeline's hash is the same before and after
//! the asks are answered"); E §9's opening (a failure refuses with `ENGINE-INVARIANT`, never an
//! answer). Items 1 and 2 are checked at each day's close (`walk/day.rs`), item 5 where a plan
//! instance is booked (`walk/plan.rs`); item 6 holds by the walk's `RUN-LIFESPAN` refusals
//! (`walk/day.rs`). Item 3 ("A scenario changes nothing before its `from` that it doesn't
//! list"; L §6.4): a main walk in a scenario's world, once walked, against the world under
//! it, entry by entry before `from`; the movements that read the scenario's versions listed
//! as `before from:` lines (ruling P43).
//! Twin: engine/tools/refcore/walk.py (`Walk.signature`; `m_main_walk` keeps `w.sig` once the
//! walk is done, then `m_inv3`) and ask.py (`Asker._answer`'s last step, its text).
//! Invariants: a signature is every entry of every place's log, (place, unit) in text order and
//! each log in walk order, with the entry's point, change and movement; it is taken once, when
//! a main walk is done; a projection's walk and a failed walk have none checked; item 3 walks
//! the world under the scenario's in that world's own timeline, and the scenario's is back
//! in force after it.

use std::collections::{BTreeMap, BTreeSet};

use crate::base::{Code, Refusal};
use crate::eval::{Collected, Engine, Entry, world_named};
use crate::model::{Reality, World};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{Amount, At, Change, Exists, Inst, Progress, Walk};

/// One booked entry as the signature holds it: its point, what it does, its movement.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    /// The entry's point.
    at: At,
    /// What it does to the place's state.
    change: Change,
    /// The movement it books, by id.
    mv: Option<String>,
}

/// A main walk's signature (refcore's `Walk.signature`): each (place, unit)'s booked entries,
/// in walk order. E §9 item 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Signature {
    /// The entries of each (place, unit), the keys in text order.
    logs: BTreeMap<(String, String), Vec<Line>>,
}

impl Signature {
    /// The signature of `walk` as it stands: every entry of every place's log. E §9 item 4.
    fn of(walk: &Walk) -> Signature {
        let mut logs = BTreeMap::new();
        for (key, log) in &walk.logs {
            let lines = log
                .ents
                .iter()
                .map(|e| Line {
                    at: e.at,
                    change: e.change.clone(),
                    mv: e.mv.clone(),
                })
                .collect();
            logs.insert(key.clone(), lines);
        }
        Signature { logs }
    }
}

impl Engine {
    /// Keeps the signature of walk `id` once it is done, when it is a main walk (refcore's
    /// `m_main_walk`, `w.sig`): a projection's walk is a read, not the timeline. E §9 item 4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn keep_signature(&mut self, id: usize) -> Result<(), Stop> {
        let walk = self.walk(id)?;
        if walk.proj.is_some() {
            return Ok(());
        }
        let signature = Signature::of(walk);
        self.signatures.insert(id, signature);
        Ok(())
    }
}

impl Engine {
    /// E §9 item 3 once main walk `id` is walked in a scenario's world: every entry of every
    /// place dated before the scenario's `from` is one the world under it books there, the
    /// same cut and pins, except an instance, a plan's or a decision's as it ran, whose read
    /// set (E §3.2) holds a fact of the scenario, through its defs and the balances it read,
    /// which the walk lists (rulings P45, P46: whatever else ran before `from`), and a reading
    /// or pad at a place such an instance changed earlier; a hole, a hole no reading answers
    /// and a refusal pass, as they answer the read; a base walk that refuses is passed. L §6.4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT scenario s: x on d: … differs from the base's, and nothing it read
    /// is the scenario's (E §9 invariant 3)` for the first such entry, by day; `NotBuilt`
    /// when that entry is a decision's (its projection's reads aren't traced); `Deep` and
    /// `NotBuilt` from the instances and the base world's walk.
    pub(crate) fn scenario_unchanged(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        let world = self.timeline_world();
        let Some(scenario) = m.scenarios.scenario_of(world) else {
            return Ok(());
        };
        let Some(from) = m.scenarios.from.get(scenario).copied() else {
            return Ok(());
        };
        let walk = self.walk(id)?;
        if from <= m.start || walk.proj.is_some() {
            return Ok(());
        }
        let cut = walk.cut;
        let below = m
            .scenarios
            .over
            .get(scenario)
            .and_then(Option::as_deref)
            .unwrap_or(&m.header.id);
        let under = world_named(m, below)?;
        let ours = before(m, self.walk(id)?, from);
        let facts = self.instance_facts(m, (id, cut), under)?;
        let ctx = self.timeline_ctx(None, cut).in_world(under);
        self.enter_timeline(m, ctx)?;
        let base = self.base_before(m, cut, from);
        self.leave_timeline()?;
        let Some(base) = base? else {
            return Ok(());
        };
        let removed = listed(scenario, (&ours, &base), &facts)?;
        let scenario_walk = self.walk_mut(id)?;
        for (place, day, entry) in removed {
            let at_place = scenario_walk.adj_regs.entry(place).or_insert_with(Vec::new);
            if !at_place.contains(&(day, entry.clone())) {
                at_place.push((day, entry));
            }
        }
        Ok(())
    }

    /// The scenario's facts each plan instance of cut `cut` in the world in force read, by
    /// instance id: the items of its read set and its day's (E §3.2) whose sections world
    /// `under` doesn't see, an instance that doesn't exist or is refused included. E §9 item
    /// 3, L §6.4.
    fn instance_facts(
        &mut self,
        m: &Reality,
        (id, cut): (usize, Day),
        under: World,
    ) -> Result<Facts, Stop> {
        let mut out = BTreeMap::new();
        for mv in m.moves_in(self.timeline_world()) {
            if mv.decision {
                continue;
            }
            for inst in self.move_instances(m, mv, cut)?.iter() {
                add_read(&mut out, m, inst, (under, false));
            }
        }
        for run in &self.walk(id)?.runs {
            add_read(&mut out, m, &run.inst, (under, true));
        }
        Ok(out)
    }

    /// The entries before `from` of the world in force's main walk of cut `cut`; none for a
    /// base walk that refuses (refcore's `m_inv3`, its `bw`). E §9 item 3.
    fn base_before(&mut self, m: &Reality, cut: Day, from: Day) -> Result<Option<Before>, Stop> {
        let id = match self.main_walk(m, cut) {
            Ok(id) => id,
            Err(Stop::Refused(_)) => return Ok(None),
            Err(stop) => return Err(stop),
        };
        Ok(Some(before(m, self.walk(id)?, from)))
    }
}

/// Each (place, unit)'s entries before a scenario's `from`, by day, each day's in walk order.
type Before = BTreeMap<(String, String), BTreeMap<Day, Vec<Moved>>>;

/// The scenario's facts each plan instance read, by instance id.
type Facts = BTreeMap<String, Read>;

/// What an instance read of the scenario: the items, whether the instance is a hole or a
/// refusal, which answers the read in its place, and whether it is a decision's as it ran.
struct Read {
    /// The scenario's items it read, sorted.
    items: BTreeSet<String>,
    /// Whether its evaluation, its existence or a leg's amount is a hole or a refusal.
    answers: bool,
    /// Whether it is a decision instance the walk ran (E §4.2 step 3).
    decision: bool,
}

/// Adds instance `inst`'s read of the scenario, the items of its read set and its day's
/// whose sections world `under` doesn't see, to `out`, marked a decision's when `decision`.
/// E §9 item 3, L §6.4.
fn add_read(out: &mut Facts, m: &Reality, inst: &Inst, (under, decision): (World, bool)) {
    let held = out.entry(inst.id.clone()).or_insert_with(|| Read {
        items: BTreeSet::new(),
        answers: false,
        decision,
    });
    held.items.extend(scenario_items(m, &inst.coll, under));
    held.items.extend(scenario_items(m, &inst.day_coll, under));
    held.answers = held.answers || answers_itself(inst);
    held.decision = held.decision || decision;
}

/// An entry as E §9 item 3 compares it: what it does and its movement, not its rank among
/// the day's acts, which a scenario's own moves shift.
#[derive(Debug, Clone)]
struct Moved {
    /// What it does to the place's state.
    change: Change,
    /// The movement it books, by id.
    mv: Option<String>,
}

/// An entry before `from` that only one of the two walks books.
struct Odd {
    /// Its place.
    place: String,
    /// Its day.
    day: Day,
    /// Its movement, by id; none for a reading.
    mv: Option<String>,
    /// Whether the scenario's walk books it.
    ours: bool,
    /// Whether it is a hole, a hole no reading answers or a refusal, which answers the read.
    flagged: bool,
}

/// A movement whose read set holds a scenario fact: its first day that differs, whether the
/// scenario's walk books it, the places it differs at, whether it is flagged.
struct Found {
    /// The first day it differs.
    day: Day,
    /// Whether the scenario's walk books it at all.
    ours: bool,
    /// The places it differs at.
    places: BTreeSet<String>,
    /// Whether an entry of it is a hole or a refusal, which answers the read: not listed.
    flagged: bool,
}

/// The entries of `walk` dated before `from` at each thing's place (`*` is the walk's own
/// row of every place). E §9 item 3, L §6.4.
fn before(m: &Reality, walk: &Walk, from: Day) -> Before {
    let mut out = BTreeMap::new();
    for ((place, unit), log) in &walk.logs {
        if place == "*" || unit == "*" || !m.things.contains(place) {
            continue;
        }
        let mut days: BTreeMap<Day, Vec<Moved>> = BTreeMap::new();
        // a mark changes no balance (refcore compares balances): what decided an instance
        // doesn't exist, an `open` instance
        let changing = log
            .ents
            .iter()
            .filter(|e| e.at.day < from && !matches!(e.change, Change::Mark));
        for ent in changing {
            days.entry(ent.at.day).or_insert_with(Vec::new).push(Moved {
                change: ent.change.clone(),
                mv: ent.mv.clone(),
            });
        }
        out.insert((place.clone(), unit.clone()), days);
    }
    out
}

/// Whether plan instance `inst` is a hole or a refusal: its evaluation refused, its `when` a
/// hole, a leg's amount a hole or a refusal (E §4.2 step 5). E §9 item 3.
fn answers_itself(inst: &Inst) -> bool {
    inst.err.is_some()
        || matches!(inst.exists, Exists::Unknown(_))
        || inst
            .legs
            .iter()
            .any(|lg| matches!(lg.amount, Amount::Hole(_) | Amount::Refused(_)))
}

/// The items of the scenario's facts read set `coll` holds whose sections world `under`
/// doesn't see (L §6.4). E §9 item 3.
fn scenario_items(m: &Reality, coll: &Collected, under: World) -> BTreeSet<String> {
    let unseen = coll
        .scenario
        .iter()
        .filter(|(section, _)| !m.scenarios.sees(under, Some(section.as_str())));
    unseen.map(|(_, item)| item.clone()).collect()
}

/// The scenario's walk against its base's before `from`: each entry that differs is a hole or
/// a refusal, or an instance's that read a fact of the scenario, or at a place such an
/// instance changed on that day or before (ruling P45); then the base's movements the
/// scenario removed, at their places (`removed`). E §9 item 3, L §6.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for the first entry, by day, that differs from the base's, isn't
/// flagged, isn't a listed instance's, and isn't at a place a listed instance changed on that
/// day or before.
fn listed(
    scenario: &str,
    (ours, base): (&Before, &Before),
    facts: &Facts,
) -> Result<Vec<(String, Day, Entry)>, Stop> {
    let mut odds = Vec::new();
    let keys: BTreeSet<&(String, String)> = ours.keys().chain(base.keys()).collect();
    for key in keys {
        let (mine, theirs) = (ours.get(key), base.get(key));
        let days: BTreeSet<&Day> = mine
            .into_iter()
            .chain(theirs)
            .flat_map(BTreeMap::keys)
            .collect();
        for day in days {
            let a = mine.and_then(|d| d.get(day)).map(Vec::as_slice);
            let b = theirs.and_then(|d| d.get(day)).map(Vec::as_slice);
            day_odds((&key.0, *day), (a, b), &mut odds);
        }
    }
    let mut found: BTreeMap<String, Found> = BTreeMap::new();
    for odd in &odds {
        let read = odd.mv.as_deref().and_then(|mv| facts.get_key_value(mv));
        let Some((mv, read)) = read.filter(|(_, r)| !r.items.is_empty()) else {
            continue;
        };
        let held = found.entry(mv.clone()).or_insert_with(|| Found {
            day: odd.day,
            ours: false,
            places: BTreeSet::new(),
            flagged: false,
        });
        held.day = held.day.min(odd.day);
        held.ours = held.ours || odd.ours;
        held.places.insert(odd.place.clone());
        held.flagged = held.flagged || odd.flagged || read.answers;
    }
    let mut first: Option<&Odd> = None;
    for odd in &odds {
        let explained = odd.flagged
            || odd.mv.as_deref().is_some_and(|mv| found.contains_key(mv))
            || follows(&found, odd);
        if !explained && first.is_none_or(|f| odd.day < f.day) {
            first = Some(odd);
        }
    }
    if let Some(odd) = first {
        let decided = odd.mv.as_deref().and_then(|mv| facts.get(mv));
        if decided.is_some_and(|r| r.decision) {
            return Err(Stop::NotBuilt(
                "a decision before a scenario's from that differs from the base's, its read set holding no fact of the scenario (E §9 item 3: its projection's reads untraced)",
            ));
        }
        return Err(unexplained(scenario, odd));
    }
    Ok(removed(scenario, (ours, base)))
}

/// The `before from:` entries of the base's movements booked before `from` that the
/// scenario's walk doesn't book: `<movement> on <day> removed by <scenario>`, at each place
/// the base books it, from that day (refcore's `m_inv3`, its listing; ruling P46). The legs
/// the scenario's walk books list what they read themselves (`walk/day.rs`, `before_from`).
/// L §8.2 `before from:`, E §9 item 3 (tally 2.1.7 R7).
fn removed(scenario: &str, (ours, base): (&Before, &Before)) -> Vec<(String, Day, Entry)> {
    let mut mine: BTreeSet<&str> = BTreeSet::new();
    for moveds in ours.values().flat_map(BTreeMap::values) {
        mine.extend(moveds.iter().filter_map(|moved| moved.mv.as_deref()));
    }
    let mut out: Vec<(String, Day, Entry)> = Vec::new();
    for ((place, _), days) in base {
        for (day, moveds) in days {
            for mv in moveds.iter().filter_map(|moved| moved.mv.as_deref()) {
                if mine.contains(mv) {
                    continue;
                }
                let entry = Entry {
                    word: "before from",
                    text: format!("{mv} on {} removed by {scenario}", day.text()),
                    figure: String::new(),
                };
                let listed = (place.clone(), *day, entry);
                if !out.contains(&listed) {
                    out.push(listed);
                }
            }
        }
    }
    out
}

/// Whether entry `odd` follows from a listed instance: one that changed its place on its day
/// or before (E §9 item 3, "what follows from them").
fn follows(found: &BTreeMap<String, Found>, odd: &Odd) -> bool {
    found
        .values()
        .any(|f| f.day <= odd.day && f.places.contains(&odd.place))
}

/// The entries of one place on one day that only one walk books, the two walks' entries
/// `(ours, base)`, none where a walk books nothing there, matched by what they do and their
/// movement. E §9 item 3.
fn day_odds(
    (place, day): (&str, Day),
    (ours, base): (Option<&[Moved]>, Option<&[Moved]>),
    out: &mut Vec<Odd>,
) {
    let base: Vec<&Moved> = base.into_iter().flatten().collect();
    let mut used = vec![false; base.len()];
    let odd = |moved: &Moved, mine: bool| Odd {
        place: place.to_owned(),
        day,
        mv: moved.mv.clone(),
        ours: mine,
        flagged: matches!(
            moved.change,
            Change::Hole(_) | Change::Perm(_) | Change::Bad(_)
        ),
    };
    for moved in ours.into_iter().flatten() {
        let hit = base
            .iter()
            .zip(used.iter_mut())
            .find(|(b, u)| !**u && b.change == moved.change && b.mv == moved.mv);
        match hit {
            Some((_, u)) => *u = true,
            None => out.push(odd(moved, true)),
        }
    }
    let left = base.iter().zip(&used).filter(|(_, u)| !**u);
    out.extend(left.map(|(b, _)| odd(b, false)));
}

/// The refusal of an entry a scenario changed before its `from` whose read set holds
/// nothing of it. E §9 item 3.
fn unexplained(scenario: &str, odd: &Odd) -> Stop {
    let what = match &odd.mv {
        Some(mv) => format!("movement {mv}"),
        None => "a reading".to_owned(),
    };
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        format!(
            "scenario {scenario}: {} on {}: {what} differs from the base's, and nothing it read is the scenario's (E §9 invariant 3)",
            odd.place,
            odd.day.text()
        ),
    ))
}

/// Asks move no money in one timeline: every main walk of `walks` that kept a signature,
/// unless it failed, has the signature it had when it was done (refcore's `Asker._answer`,
/// its last step). E §9 item 4.
///
/// # Errors
///
/// `ENGINE-INVARIANT asks move no money: …` for the first walk whose entries changed, by
/// the order the walks were made; `ENGINE-INVARIANT` for a signature of a walk not held.
pub(crate) fn unmoved(walks: &[Walk], signatures: &BTreeMap<usize, Signature>) -> Result<(), Stop> {
    for (id, kept) in signatures {
        let walk = walks.get(*id).ok_or_else(|| {
            Stop::Refused(Refusal::new(
                Code::EngineInvariant,
                "a signature of a walk the timeline lacks".to_owned(),
            ))
        })?;
        if matches!(walk.progress, Progress::Failed(..)) {
            continue;
        }
        if Signature::of(walk) != *kept {
            return Err(moved(walk.cut));
        }
    }
    Ok(())
}

/// The refusal of a timeline that changed while an ask was answered, as refcore words it.
/// E §9 item 4.
fn moved(cut: Day) -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        format!(
            "asks move no money: the timeline at cut {} changed while the ask was answered (E §9 invariant 4)",
            cut.text()
        ),
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::Days;
    use crate::api::Source;
    use crate::eval::Engine;
    use crate::load::load;
    use crate::model::Reality;
    use crate::num::Num;
    use crate::syntax::{Stop, parse_reality};
    use crate::walk::{At, Change, Ent, Projected};

    /// A rent and a wage between two places, asked once.
    const REALITY: &str = r#"model sg
  tally 2
  on 2025-02-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

input pay: money DUK

input rent_day: date

input wage_day: date

move rent: transfers
  on rent_day
  leg cash from bank to out amount pay

move wage: transfers
  on wage_day
  leg cash from out to bank amount pay

version facts
  cite card
  governs always
  known 2025-01-01
  pay = 100.00
  rent_day = 2025-01-20
  wage_day = 2025-03-05
  opening[bank] = 1000.00 at 2025-01-01 start

ask bank_end: bal(bank, 2025-03-31)
"#;

    fn reality() -> Reality {
        let files = parse_reality(&[Source {
            path: "sg.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("parses");
        load(files).expect("loads")
    }

    #[test]
    fn a_walk_whose_entries_change_after_it_is_done_refuses_the_ask() {
        let m = reality();
        let mut engine = Engine::new(m.on).with_days(Days::Events);
        let id = engine.main_walk(&m, m.on).expect("walks");
        assert!(engine.signatures.contains_key(&id), "kept once walked");
        assert_eq!(engine.asks_moved_no_money(), Ok(()), "nothing moved");
        let again = engine.main_walk(&m, m.on).expect("walks");
        assert_eq!(again, id, "the same walk");
        assert_eq!(engine.asks_moved_no_money(), Ok(()), "a read moves nothing");
        let walk = engine.walk_mut(id).expect("the walk");
        let log = walk
            .logs
            .get_mut(&("bank".to_owned(), "DUK".to_owned()))
            .expect("bank's log");
        let first = log.ents.first().expect("an entry").at;
        log.ents
            .push(Ent::new(At::end(first.day), Change::Delta(Num::int(1))));
        let Err(Stop::Refused(r)) = engine.asks_moved_no_money() else {
            panic!("a changed timeline is refused");
        };
        assert_eq!(
            r.text(),
            "ENGINE-INVARIANT asks move no money: the timeline at cut 2025-02-10 changed while \
             the ask was answered (E §9 invariant 4)",
            "refcore's text"
        );
    }

    #[test]
    fn a_projection_keeps_no_signature() {
        let m = reality();
        let mut engine = Engine::new(m.on).with_days(Days::Events);
        let id = engine.main_walk(&m, m.on).expect("walks");
        let mut walk = engine.walk(id).expect("the walk").clone();
        walk.proj = Some(Box::new(Projected {
            base: id,
            point: At::start(m.start),
            left_out: BTreeSet::new(),
            counted: BTreeMap::new(),
            levels: Vec::new(),
            root: (m.on, m.start),
            world: crate::model::World::Whole,
        }));
        let projected = engine.walks.len();
        engine.walks.push(walk);
        engine.keep_signature(projected).expect("kept or not");
        assert!(
            !engine.signatures.contains_key(&projected),
            "a projection's walk is a read"
        );
    }
}
