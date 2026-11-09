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
use crate::eval::{Engine, world_named};
use crate::model::{Reality, World};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{At, Change, Ent, Progress, Walk};

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
    /// same cut and pins, except a movement whose evaluation read a version of the scenario
    /// (a law date from `from`, a binding reaching it), which the walk lists as `before from:
    /// <movement> on <day> reads <versions>` (ruling P43), and what differs from that day on,
    /// which follows from it; a walk in which a decision ran before `from` is passed (refcore's
    /// `m_inv3`), and so is a base walk that refuses. L §6.4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT scenario s: x on d: … differs from the base's, reading nothing of
    /// the scenario (E §9 invariant 3)` for the first such entry, by day; `Deep` and
    /// `NotBuilt` from the base world's walk.
    pub(crate) fn scenario_unchanged(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        let world = self.timeline_world();
        let Some(scenario) = m.scenarios.scenario_of(world) else {
            return Ok(());
        };
        let Some(from) = m.scenarios.from.get(scenario).copied() else {
            return Ok(());
        };
        let walk = self.walk(id)?;
        if from <= m.start || walk.proj.is_some() || walk.runs.iter().any(|r| r.point.day < from) {
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
        let ours = before(m, walk, (from, Some(under)));
        let ctx = self.timeline_ctx(None, cut).in_world(under);
        self.enter_timeline(m, ctx)?;
        let base = self.base_before(m, cut, from);
        self.leave_timeline()?;
        let Some(base) = base? else {
            return Ok(());
        };
        let listed = listed(scenario, &ours, &base)?;
        self.walk_mut(id)?.listed = listed;
        Ok(())
    }

    /// The entries before `from` of the world in force's main walk of cut `cut`; none for a
    /// base walk that refuses (refcore's `m_inv3`, its `bw`). E §9 item 3.
    fn base_before(&mut self, m: &Reality, cut: Day, from: Day) -> Result<Option<Before>, Stop> {
        let id = match self.main_walk(m, cut) {
            Ok(id) => id,
            Err(Stop::Refused(_)) => return Ok(None),
            Err(stop) => return Err(stop),
        };
        Ok(Some(before(m, self.walk(id)?, (from, None))))
    }
}

/// Each (place, unit)'s entries before a scenario's `from`, by day, each day's in walk order.
type Before = BTreeMap<(String, String), BTreeMap<Day, Vec<Moved>>>;

/// An entry as E §9 item 3 compares it: what it does and its movement, not its rank among
/// the day's acts, which a scenario's own moves shift; and the versions its movement's
/// evaluation read that the world under the scenario doesn't see.
#[derive(Debug, Clone)]
struct Moved {
    /// What it does to the place's state.
    change: Change,
    /// The movement it books, by id.
    mv: Option<String>,
    /// The scenario's versions its movement read (the scenario's walk's only).
    reads: BTreeSet<String>,
}

/// An entry before `from` that only one of the two walks books.
struct Odd {
    /// Its place.
    place: String,
    /// Its day.
    day: Day,
    /// Its movement, by id; none for a reading.
    mv: Option<String>,
}

/// The entries of `walk` dated before `from` at each thing's place (`*` is the walk's own
/// row of every place), with the versions of the scenario each read where `under`, the
/// world under it, is given. E §9 item 3, L §6.4.
fn before(m: &Reality, walk: &Walk, (from, under): (Day, Option<World>)) -> Before {
    let mut out = BTreeMap::new();
    for ((place, unit), log) in &walk.logs {
        if place == "*" || unit == "*" || !m.things.contains(place) {
            continue;
        }
        let mut days: BTreeMap<Day, Vec<Moved>> = BTreeMap::new();
        for ent in log.ents.iter().filter(|e| e.at.day < from) {
            days.entry(ent.at.day).or_insert_with(Vec::new).push(Moved {
                change: ent.change.clone(),
                mv: ent.mv.clone(),
                reads: unseen(m, ent, under),
            });
        }
        out.insert((place.clone(), unit.clone()), days);
    }
    out
}

/// The versions entry `ent`'s movement read that world `under` doesn't see: the scenario's
/// own (L §6.4); none without a world. E §9 item 3.
fn unseen(m: &Reality, ent: &Ent, under: Option<World>) -> BTreeSet<String> {
    let (Some(under), Some(coll)) = (under, &ent.coll) else {
        return BTreeSet::new();
    };
    coll.versions
        .iter()
        .filter(|v| !m.scenarios.sees(under, Some(v.as_str())))
        .cloned()
        .collect()
}

/// The `before from:` texts of the scenario's walk `ours` against its base's, sorted: one
/// per movement that differs and read a version of the scenario, `<movement> on <day> reads
/// <versions>`, its first day that differs (ruling P43). E §9 item 3, L §6.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for the first entry, by day, that differs from the base's before every
/// listed movement and read nothing of the scenario.
fn listed(scenario: &str, ours: &Before, base: &Before) -> Result<Vec<String>, Stop> {
    let mut reads: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for moved in ours.values().flat_map(BTreeMap::values).flatten() {
        if let Some(mv) = &moved.mv {
            let held = reads.entry(mv.as_str()).or_insert_with(BTreeSet::new);
            held.extend(moved.reads.iter().map(String::as_str));
        }
    }
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
    let mut found: BTreeMap<&str, (Day, &BTreeSet<&str>)> = BTreeMap::new();
    let mut first_odd: Option<&Odd> = None;
    for odd in &odds {
        let read = odd.mv.as_deref().and_then(|mv| reads.get_key_value(mv));
        match read.filter(|(_, versions)| !versions.is_empty()) {
            Some((mv, versions)) => {
                let held = found.entry(mv).or_insert((odd.day, versions));
                held.0 = held.0.min(odd.day);
            }
            None if first_odd.is_none_or(|f| odd.day < f.day) => first_odd = Some(odd),
            None => {}
        }
    }
    let first_listed = found.values().map(|(day, _)| *day).min();
    if let Some(odd) = first_odd
        && first_listed.is_none_or(|day| odd.day < day)
    {
        return Err(unexplained(scenario, odd));
    }
    let mut out: Vec<String> = found
        .iter()
        .map(|(mv, (day, versions))| {
            let names: Vec<&str> = versions.iter().copied().collect();
            format!("{mv} on {} reads {}", day.text(), names.join(", "))
        })
        .collect();
    out.sort();
    Ok(out)
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
    let odd = |moved: &Moved| Odd {
        place: place.to_owned(),
        day,
        mv: moved.mv.clone(),
    };
    for moved in ours.into_iter().flatten() {
        let hit = base
            .iter()
            .zip(used.iter_mut())
            .find(|(b, u)| !**u && b.change == moved.change && b.mv == moved.mv);
        match hit {
            Some((_, u)) => *u = true,
            None => out.push(odd(moved)),
        }
    }
    let left = base.iter().zip(&used).filter(|(_, u)| !**u);
    out.extend(left.map(|(b, _)| odd(b)));
}

/// The refusal of an entry a scenario changed before its `from` that read nothing of it.
/// E §9 item 3.
fn unexplained(scenario: &str, odd: &Odd) -> Stop {
    let what = match &odd.mv {
        Some(mv) => format!("movement {mv}"),
        None => "a reading".to_owned(),
    };
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        format!(
            "scenario {scenario}: {} on {}: {what} differs from the base's, reading nothing of the scenario (E §9 invariant 3)",
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
        if matches!(walk.progress, Progress::Failed(_)) {
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
