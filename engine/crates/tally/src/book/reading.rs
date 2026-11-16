//! Balance readings: the `opening` and `stated` rows the cut sees, and what a stated reading does to its place, a pad or an anchor, with its gap.
//! Spec: L §5.19 (balance readings and their gaps: `pad x from y`, `anchor x`; a reading whose
//! value is `none` states nothing; the read rule decides who sees it), §4.6 (a reading's
//! instant), §5.10 (`HOLE-ANCHOR`; the sign rule tests the reading's own place at the
//! reading's point, a pad's in the point's group with its leg, an anchor's through
//! `walk/day.rs`; tally 2.1.2); E §6.1 (Readings: the gap is the reading less the booked
//! balance at its instant; a pad is a movement of `pads`, an anchor sets the balance; holes),
//! §4.2 step 0 and step 4 (where readings apply), step 5 (the sign rule at a reading's point);
//! L §6.4 (the readings a walk's world sees).
//! Twin: engine/tools/refcore/moves.py (`readings`, `instant_text`, `PadMv`) and walk.py
//! (`apply_reading`, `booked_coll`, `sub`, `st_value`, `gtext`).
//! Invariants: a reading applies only before the cut but an opening at the run's start; after
//! a reading the place's balance is the reading; a pad books only its other end, the reading's
//! end is the reset, which carries the pad's flow there; a gap is used where it is met (worth,
//! the register, a pad's amount) and kept per (place, day) for `gap(x, d)`; the ledger's
//! balance line computes its own from the reading (`book/ledger.rs`).

use std::collections::BTreeMap;

use super::observe::{KnownAtCut, Obs, Windows};
use crate::base::{Code, Load, Refusal};
use crate::eval::{Collected, Engine, Entry, Rows};
use crate::hole::{Hole, spread};
use crate::model::{GapRole, Reality, RowInstant, World};
use crate::num::Num;
use crate::read::row_known;
use crate::syntax::{Edge, Stop};
use crate::time::Day;
use crate::walk::{
    Amount, At, Change, Dated, End, Ends, Ent, GapAt, LegVal, Reading, St, invariant, qty_value,
};

/// A reading the cut sees: its day, where it stands in the day, its place, its input, and
/// what it states (none for a row whose value is `none`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    /// Its day: for a reading `after m`, m's day.
    pub day: Day,
    /// Where it stands in its day.
    pub stands: Stands,
    /// The place.
    pub place: String,
    /// The input: `opening` or `stated`.
    pub input: String,
    /// What it states; none for a row whose value is `none`.
    pub reading: Option<Reading>,
}

/// Where a reading stands in its day (L §4.6 Readings): at its start (before the day's
/// movements), at its end (after them), or right after a movement, by its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stands {
    /// `at D start` or `at D end`.
    Edge(Edge),
    /// `after m`: m's instance id, as written (refcore's `_txt`).
    After(String),
}

/// The day each reading `after m` stands on, by m's id and the reading's place, where the
/// plan pass gives m one: the day the walk applies it (E §4.2 step 3, L §4.6).
pub type AfterDays = BTreeMap<(String, String), Day>;

/// The place a reading row names, its one key (L §4.6 Readings); none where its key is no
/// thing.
#[must_use]
pub(crate) fn reading_place(row: &crate::model::Row) -> Option<&str> {
    match row.pattern.first() {
        Some(crate::model::Pat::Key(crate::model::KeyVal::Thing(place))) => Some(place),
        Some(_) | None => None,
    }
}

/// The day of the last leg of the observations `obs` that links `links` names, booked from
/// `start` through `end` before the cut, a leg at `place` first, all where none is: where a
/// reading `after m` with m modelled past stands, just after the observations that fulfil m,
/// "on the day of the last of them" (as the walk's `last_standing_in` places it). L §4.6
/// Readings, E §4.2 step 3.
fn last_standing_day(
    obs: &[Obs],
    links: &[String],
    place: &str,
    (start, end, cut): (Day, Day, Day),
) -> Option<Day> {
    let (mut touching, mut all): (Option<Day>, Option<Day>) = (None, None);
    for o in obs.iter().filter(|o| links.contains(&o.id)) {
        for lg in &o.legs {
            let Some(d) = lg.day.day() else {
                continue;
            };
            if d < start || d > end || d >= cut {
                continue;
            }
            all = all.max(Some(d));
            if lg.from.place() == Some(place) || lg.to.place() == Some(place) {
                touching = touching.max(Some(d));
            }
        }
    }
    touching.or(all)
}

/// A pad: a movement of `pads` that takes up a statement's gap (refcore's `PadMv`). E §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pad {
    /// Its id, `pads:<version>:<place>@<instant>`.
    pub id: String,
    /// Its one leg, from the pad's other place or to it.
    pub leg: LegVal,
    /// What its amount rests on: the booked balance's register and the reading's.
    pub coll: Collected,
}

/// A reading row kept as the latest of its key: when it became known, its day and edge, the
/// row (L §5.19).
type Kept = (Day, Day, Stands, crate::model::Row);

/// A kept reading row with the unit its value states, none for a hole or `none` at a place
/// of unit `any` (pick P22, #173).
type UnitKept = (Option<String>, Kept);

/// The readings cut `cut` sees in world `world`: each `opening` and `stated` row known by
/// the cut, of a version the world sees, a restated one replaced as `replace` says (a
/// scenario's over its base's whatever their `known` dates, as `standing` takes it), sorted
/// by place, instant and input, each with its row among what it met where `rows` says so; a
/// reading `after m` on m's day by `after`, none where m has no day there; one known in a
/// window that holds the cut as `known` decides it: unseen, or with no `windows each` its
/// value `HOLE-IMPRECISE`, it is stated or not (refcore's `readings`, `world_ok`, `visible`).
/// L §5.19, §4.6 step 2, Readings, §6.4, §4.5; E §10.
///
/// # Errors
///
/// A row load left without its version; `NotBuilt` for a reading at a day with no edge.
pub(crate) fn readings(
    m: &Reality,
    (cut, world): (Day, World),
    (rows, after): (Rows, &AfterDays),
    known: &KnownAtCut,
) -> Result<Vec<Seen>, Stop> {
    // a reading row is keyed by its place and instant, its unit in its value: a place of unit
    // `any` holds a reading per unit (L §4.6), and a row that states no unit (a hole, `none`)
    // stands for every unit at its instant (pick P22, #173)
    let mut cands: BTreeMap<(String, String, String), Vec<UnitKept>> = BTreeMap::new();
    for input in ["opening", "stated"] {
        let Some(info) = m.inputs.get(input) else {
            continue;
        };
        for row in &info.rows {
            if !m.scenarios.sees(world, Some(&row.version)) {
                continue;
            }
            let known_on = row_known(m, row)?;
            let sid = row_known_sid(row);
            if known_on > cut || known.unseen.contains(&sid) {
                continue;
            }
            let Some(crate::model::Pat::Key(crate::model::KeyVal::Thing(place))) =
                row.pattern.first()
            else {
                return Err(Stop::NotBuilt("a reading of a place not named"));
            };
            let mut row = row.clone();
            if let Some(question) = known.question(&sid, cut) {
                row.value = crate::syntax::Value::Hole(question);
            }
            let (day, stands) = match &row.instant {
                Some(RowInstant::At(day, Some(edge))) => (*day, Stands::Edge(*edge)),
                Some(RowInstant::After(node)) => {
                    let id = crate::load::expr_text(node);
                    // an m the plan pass gives no day: the reading has no instant
                    let Some(day) = after.get(&(id.clone(), place.clone())) else {
                        continue;
                    };
                    (*day, Stands::After(id))
                }
                Some(RowInstant::At(_, None)) | None => {
                    return Err(Stop::NotBuilt(
                        "a reading at an instant other than a day's start or end",
                    ));
                }
            };
            let unit = m
                .reading_ty(place, &row.value)
                .map_err(crate::eval::num_stop)?
                .map(|(unit, _)| unit);
            let key = (place.clone(), stands_text(day, &stands), input.to_owned());
            cands
                .entry(key)
                .or_insert_with(Vec::new)
                .push((unit, (known_on, day, stands, row)));
        }
    }
    let mut out = Vec::with_capacity(cands.len());
    for ((place, instant, input), rows_at) in cands {
        for unit in stated_units(&rows_at) {
            // the unit's own rows and the rows that state no unit, which stand for it
            let rows_of: Vec<Kept> = rows_at
                .iter()
                .filter(|(u, _)| u.is_none() || *u == unit)
                .map(|(_, kept)| kept.clone())
                .collect();
            let Some((_, day, stands, row)) = standing(m, rows_of) else {
                continue;
            };
            let mut reading = reading_of(m, (&input, &place, &instant), &row, unit.as_deref())?;
            // the row as `inputs-of` lists it, kept by an engine answering one (E §10)
            if rows == Rows::Kept
                && let Some(r) = &mut reading
            {
                r.coll
                    .note_row(&row.sid, &row.version, row.locator.as_ref());
            }
            out.push(Seen {
                day,
                stands: stands.clone(),
                place: place.clone(),
                input: input.clone(),
                reading,
            });
        }
    }
    Ok(out)
}

/// Reading `r`, applied at point `at` of walk `walk`, once per unit it stands for: itself, or
/// for a hole that states no unit (unit `ALL`) one per unit its place holds at that point, in
/// text order, none where it holds none yet; each unit's own next anchor sets that unit's
/// balance again (pick P22, #173; P24). L §4.6 Readings, E §6.1.
#[must_use]
pub fn per_unit(walk: &crate::walk::Walk, r: &Reading, at: At) -> Vec<Reading> {
    if r.unit != crate::walk::ALL {
        return vec![r.clone()];
    }
    let mut out = Vec::new();
    for unit in walk.units_of(&r.place) {
        // a unit the place holds by `at`: its first entry is at or before it
        let held = matches!(walk.entries(&r.place, &unit).first(), Some(e) if e.at <= at);
        if held {
            out.push(Reading { unit, ..r.clone() });
        }
    }
    out
}

/// The units the rows of one place, instant and input state, in text order; one `None` where
/// none states a unit, every unit the place holds then (pick P22, #173). L §4.6 Readings.
fn stated_units(rows_at: &[UnitKept]) -> Vec<Option<String>> {
    let units: std::collections::BTreeSet<&String> =
        rows_at.iter().filter_map(|(u, _)| u.as_ref()).collect();
    if units.is_empty() {
        return vec![None];
    }
    units.into_iter().map(|u| Some(u.clone())).collect()
}

/// The row that stands among the readings `rows_at` of one place, instant and input: of the
/// rows no `replace` row beats, the latest known, the first at a tie (refcore's `choose`,
/// its replacement, as `read/rows.rs` `unreplaced` weighs other rows). L §4.6 step 2, §6.4.
fn standing(m: &Reality, rows_at: Vec<Kept>) -> Option<Kept> {
    let mut best: Option<usize> = None;
    for (at, (known, .., row)) in rows_at.iter().enumerate() {
        if replaced(m, &rows_at, (*known, row)) {
            continue;
        }
        let later = best
            .and_then(|b| rows_at.get(b))
            .is_none_or(|(k, ..)| known > k);
        if later {
            best = Some(at);
        }
    }
    best.and_then(|b| rows_at.into_iter().nth(b))
}

/// Whether a `replace` row among `rows_at` beats row `row`, known on `known`: one of a layer
/// above its own, whatever the two rows' `known` dates (a branch stands above its base), or
/// one of its own layer known after it. L §4.6 step 2, §6.4.
fn replaced(m: &Reality, rows_at: &[Kept], (known, row): (Day, &crate::model::Row)) -> bool {
    let layer = m.scenarios.layer(Some(&row.version));
    for (k, .., other) in rows_at {
        if !other.replace {
            continue;
        }
        let above = m.scenarios.layer(Some(&other.version));
        if above > layer || (above == layer && *k > known) {
            return true;
        }
    }
    false
}

/// A reading's place among the readings `walked` a walk's cut sees, as the walk ranks them on
/// their day: how many come before it by place, instant and input (refcore's walk key, its
/// text). The reading needn't be one of them: one only a later cut sees takes the place it
/// would have. E §4.2 step 0, §6.3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` past four billion readings.
pub fn reading_rank(walked: &[Seen], seen: &Seen) -> Result<u32, Stop> {
    let key = (
        seen.place.as_str(),
        stands_text(seen.day, &seen.stands),
        seen.input.as_str(),
    );
    let mut before: u32 = 0;
    for other in walked {
        let theirs = (
            other.place.as_str(),
            stands_text(other.day, &other.stands),
            other.input.as_str(),
        );
        if theirs < key {
            before = before
                .checked_add(1)
                .ok_or_else(|| invariant("a reading's rank"))?;
        }
    }
    Ok(before)
}

/// Whose `known` a reading row's is: its own (the row's id), else its version's (refcore's
/// `row_known_sid`). L §4.5.
fn row_known_sid(row: &crate::model::Row) -> String {
    if row.own_known.is_some() {
        row.sid.clone()
    } else {
        row.version.clone()
    }
}

/// Each `opening` and `stated` row known in a window, its own or its version's, with whose
/// `known` it is (L §4.5, §5.19).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a row of a version load didn't build.
fn reading_windows(m: &Reality) -> Result<Windows, Stop> {
    let mut out = Windows::new();
    for input in ["opening", "stated"] {
        let Some(info) = m.inputs.get(input) else {
            continue;
        };
        for row in &info.rows {
            let window = match (row.own_known, row.own_window) {
                (Some(lo), Some(hi)) => (lo, hi),
                (Some(_), None) => continue,
                (None, _) => {
                    let version = m
                        .versions
                        .get(&row.version)
                        .ok_or_else(|| invariant("a reading of a version load didn't build"))?;
                    match version.window {
                        Some(hi) => (version.known, hi),
                        None => continue,
                    }
                }
            };
            out.push((window, row_known_sid(row)));
        }
    }
    Ok(out)
}

/// A reading's instant as text, `2026-09-30 end` (refcore's `instant_text`). L §4.6.
pub(crate) fn instant_text(day: Day, edge: Edge) -> String {
    let word = match edge {
        Edge::Start => "start",
        Edge::End => "end",
    };
    format!("{} {word}", day.text())
}

/// Where a reading stands, as text: its instant, `2026-09-30 end`, or `after save[2026-03]`
/// (refcore's `instant_text`). L §4.6.
pub(crate) fn stands_text(day: Day, stands: &Stands) -> String {
    match stands {
        Stands::Edge(edge) => instant_text(day, *edge),
        Stands::After(id) => format!("after {id}"),
    }
}

/// A reading's row as the walk applies it, for unit `stands` among its instant's: its value
/// in its unit, a stated hole, or none for a row whose value is `none`. A hole that states no
/// unit stands for `stands`, or for every unit the place holds at its point (`ALL`) where no
/// row at its instant states one (refcore's `apply_reading`, its value; pick P22, #173).
/// L §5.19, §4.6 Readings.
fn reading_of(
    m: &Reality,
    (input, place, instant): (&str, &str, &str),
    row: &crate::model::Row,
    stands: Option<&str>,
) -> Result<Option<Reading>, Stop> {
    if row.value == crate::syntax::Value::None {
        return Ok(None);
    }
    let figure = format!("{input}[{place}]");
    let stated = m
        .reading_ty(place, &row.value)
        .map_err(crate::eval::num_stop)?;
    let (unit, value) = match (stated, &row.value) {
        (Some((unit, ty)), _) => match crate::eval::convert_row(m, &row.value, &ty)? {
            Err(text) => (unit, Err(Hole::asked(&text, &figure))),
            Ok(crate::model::Value::None) => return Ok(None),
            Ok(v) => (unit, Ok(crate::walk::quantity(m, &v)?.0)),
        },
        (None, crate::syntax::Value::Hole(text)) => (
            stands.unwrap_or(crate::walk::ALL).to_owned(),
            Err(Hole::asked(text, &figure)),
        ),
        (None, _) => {
            return Err(Stop::NotBuilt(
                "a reading whose unit neither its place nor its row states",
            ));
        }
    };
    let mut coll = Collected::default();
    coll.versions.insert(row.version.clone());
    if let Some(reason) = &row.assumed {
        coll.entries.insert(Entry {
            word: "assumed",
            text: reason.clone(),
            figure: String::new(),
        });
    }
    Ok(Some(Reading {
        input: input.to_owned(),
        place: place.to_owned(),
        unit,
        value,
        coll,
        version: row.version.clone(),
        instant: instant.to_owned(),
    }))
}

impl Engine {
    /// The readings cut `cut` and world `world` see ([`readings`]), each known in a window
    /// that holds the cut decided by the line's `windows each`, or undecided without one
    /// (refcore's `readings`, `visible`). L §5.19, §4.5, §4.4.
    ///
    /// # Errors
    ///
    /// As [`readings`]; `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn seen_readings(
        &mut self,
        m: &Reality,
        (cut, world): (Day, World),
        (rows, after): (Rows, &AfterDays),
    ) -> Result<Vec<Seen>, Stop> {
        let known = self.known_at_cut(reading_windows(m)?, cut)?;
        readings(m, (cut, world), (rows, after), &known)
    }

    /// Each place's first reading day among the readings the cut and the world of `by` see,
    /// for `HOLE-ANCHOR` (refcore's `anchor_question`, its days). L §8.1, §6.4.
    ///
    /// # Errors
    ///
    /// As [`Engine::seen_readings`].
    pub(crate) fn first_readings(
        &mut self,
        m: &Reality,
        by: (Day, World),
        after: &AfterDays,
    ) -> Result<BTreeMap<String, Day>, Stop> {
        let mut out: BTreeMap<String, Day> = BTreeMap::new();
        for seen in self.seen_readings(m, by, (Rows::Unkept, after))? {
            let first = out.entry(seen.place).or_insert(seen.day);
            *first = (*first).min(seen.day);
        }
        Ok(out)
    }

    /// The day each reading `after m` of `stated` or `opening` stands on at cut `cut`, by m
    /// and the reading's place ([`Engine::reading_day`]): the walk's whole plan pass. E §4.2
    /// step 3, L §4.6 Readings.
    ///
    /// # Errors
    ///
    /// The refusals of the plan pass.
    pub(crate) fn after_days(&mut self, m: &Reality, cut: Day) -> Result<AfterDays, Stop> {
        let mut out = AfterDays::new();
        for input in ["opening", "stated"] {
            // an input with no reading `after m` plans nothing here (L §4.6 Readings)
            let Some(info) = m.inputs.get(input).filter(|info| info.holds.after) else {
                continue;
            };
            for row in &info.rows {
                let (Some(RowInstant::After(node)), Some(place)) =
                    (&row.instant, reading_place(row))
                else {
                    continue;
                };
                let key = (crate::load::expr_text(node), place.to_owned());
                if out.contains_key(&key) {
                    continue;
                }
                if let Some(day) = self.reading_day(m, node, place, cut)? {
                    out.insert(key, day);
                }
            }
        }
        Ok(out)
    }

    /// The day a reading of `place` `after m` stands on at cut `cut`, m the decision instance
    /// `node` names, as the walk applies it: just after the observations that fulfil m, on
    /// the day of the last of them, where any do; else on m's day as the plan pass lists it;
    /// none for an m it doesn't list or whose day is a hole or a window, so the reading has
    /// no instant. What the plan pass and the links meet is the walk's, kept apart from the
    /// read that asks (refcore's `after`, keyed by `_txt`). L §4.6 Readings, E §4.2 step 3.
    ///
    /// # Errors
    ///
    /// The refusals of the plan pass and of judging m's links.
    pub(crate) fn reading_day(
        &mut self,
        m: &Reality,
        node: &crate::syntax::Expr,
        place: &str,
        cut: Day,
    ) -> Result<Option<Day>, Stop> {
        let id = crate::load::expr_text(node);
        self.push();
        let found = self.standing_day(m, (node, &id), place, cut);
        self.pop_alone();
        found
    }

    /// [`Engine::reading_day`], its reads collected by the caller. L §4.6, E §4.2 step 3.
    fn standing_day(
        &mut self,
        m: &Reality,
        (node, id): (&crate::syntax::Expr, &str),
        place: &str,
        cut: Day,
    ) -> Result<Option<Day>, Stop> {
        let Some(day) = self.instance_day(m, node, id, cut)? else {
            return Ok(None);
        };
        let links = self.obs_links_for(m, id, cut)?;
        if links.is_empty() {
            return Ok(Some(day));
        }
        let obs = self.observations(m, cut)?;
        let last = last_standing_day(obs, &links, place, (m.start, m.end, cut));
        Ok(Some(last.unwrap_or(day)))
    }

    /// The day of the decision instance `id` that `node` names, as the plan pass at cut `cut`
    /// lists it; none for a move that isn't a decision, an instance it doesn't list, or a day
    /// that is a hole or a window (refcore's `after_mv`, its test). E §4.2 step 3.
    fn instance_day(
        &mut self,
        m: &Reality,
        node: &crate::syntax::Expr,
        id: &str,
        cut: Day,
    ) -> Result<Option<Day>, Stop> {
        let base = if let crate::syntax::Expr::Post(inner, _) = node {
            inner.as_ref()
        } else {
            node
        };
        let crate::syntax::Expr::Name(name) = base else {
            return Ok(None);
        };
        let Some(mv) = m.moves.get(name).filter(|mv| mv.decision) else {
            return Ok(None);
        };
        let insts = self.plan_list(m, mv, cut)?;
        Ok(insts
            .iter()
            .find(|inst| inst.id == id)
            .and_then(|inst| inst.day.as_ref().and_then(Dated::day)))
    }

    /// A `stated` reading at its point: its gap against the booked balance, then, by its
    /// version's gap role, an anchor that sets the balance or a pad that books the gap from
    /// the other place (refcore's `apply_reading`). E §6.1, L §5.19.
    ///
    /// # Errors
    ///
    /// `LOAD-GAPROLE` for a version that states the reading with neither pad nor anchor; a
    /// number error.
    pub(crate) fn apply_stated(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        r: &Reading,
    ) -> Result<(), Stop> {
        if at.day >= self.walk(id)?.cut {
            return Ok(());
        }
        let version = m
            .versions
            .get(&r.version)
            .ok_or_else(|| invariant("a reading of a version load didn't build"))?;
        let Some(role) = version
            .gap_roles
            .iter()
            .find(|(p, _)| *p == r.place)
            .map(|(_, g)| g.clone())
        else {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::GapRole),
                format!(
                    "{} states balance readings of {} with neither pad nor anchor",
                    r.version, r.place
                ),
            )));
        };
        let walk = self.walk(id)?;
        let before = walk.pstate(m, &r.place, &r.unit, at);
        let booked = booked_amount(&before, &walk.anchor_question(&r.place));
        let gap = gap_of(&r.value, &booked)?;
        // the gap rests on what the booked balance rests on, and on the reading's row (L §8.2);
        // so does a pad, whose amount it is; kept for `gap(x, d)` (L §5.19)
        let mut bc = self.booked_coll(m, id, &r.place, &r.unit, at)?;
        bc.merge(&r.coll);
        let read = self.walk_mut(id)?;
        read.gaps
            .entry((r.place.clone(), at.day))
            .or_insert_with(Vec::new)
            .push(GapAt {
                at,
                gap: gap.clone(),
                unit: r.unit.clone(),
                coll: bc.clone(),
            });
        // E §9 item 2: an anchor's gap, or a pad's from outside, moves worth; a pad from an
        // inside place moves it between two inside places
        if read.inside(m, &r.place) {
            let moves = match &role {
                GapRole::Anchor => true,
                GapRole::Pad(other) => !read.inside(m, other),
            };
            if moves {
                let q = match &gap {
                    Amount::Qty(g) => Some(g),
                    Amount::Hole(_) | Amount::Refused(_) => None,
                };
                read.worth_add(&r.unit, q)?;
            }
        }
        let (val, qs) = match &r.value {
            Ok(v) => (v.clone(), None),
            Err(h) => (Num::zero(), Some(h.clone())),
        };
        match role {
            GapRole::Anchor => {
                let stated = match &r.value {
                    Ok(v) => qty_text(m, v, &r.unit)?,
                    Err(h) => h.text(),
                };
                let text = format!(
                    "{} {}, stated {stated}, difference {}",
                    r.place,
                    r.instant,
                    gap_text(m, &gap, &r.unit)?
                );
                let mut ent = Ent::new(at, Change::Reset { val, qs });
                ent.coll = Some(r.coll.clone());
                ent.reg = vec![Entry {
                    word: "anchor",
                    text,
                    figure: String::new(),
                }];
                let anchored = self.walk_mut(id)?;
                anchored.add(m, &r.place, &r.unit, ent)?;
                anchored.sign_anchor(m, at, r)
            }
            GapRole::Pad(other) => {
                let pad = make_pad(r, &other, &gap, bc, at)?;
                // a member of a binding over pads from its point (L §5.14.1)
                self.walk_mut(id)?.pads.push((pad.clone(), at));
                let text = format!(
                    "{} {}, {} of {}",
                    r.place,
                    r.instant,
                    pad.id,
                    gap_text(m, &gap, &r.unit)?
                );
                let reg = vec![Entry {
                    word: "pad",
                    text,
                    figure: String::new(),
                }];
                let ends = Ends {
                    from: pad.leg.from.place() == Some(other.as_str()),
                    to: pad.leg.to.place() == Some(other.as_str()),
                    whole: true,
                };
                let grouped = self.walk_mut(id)?.open_group(at).is_some();
                let breaking = match (&gap, &r.value) {
                    (Amount::Qty(_), Ok(v)) => !grouped && crate::walk::breaks(m, &r.place, v),
                    (Amount::Qty(_) | Amount::Hole(_) | Amount::Refused(_), _) => false,
                };
                if breaking {
                    // after a decision, where no group is open, the reading's own place is
                    // tested as it is booked, the pad refused before anything is (L §5.10
                    // point 2, E §4.2 step 5; refcore's `apply_reading`)
                    let err = Refusal::new(
                        Code::Run(crate::base::Run::Place),
                        crate::walk::takes_text(m, &pad.id, &r.place),
                    );
                    self.walk_mut(id)?
                        .refuse_leg(m, (&pad.id, &pad.leg), &err, at)?;
                } else {
                    self.book_leg_ends(m, id, (&pad.id, &pad.leg), at, (&reg, &pad.coll), ends)?;
                }
                let padded = self.walk_mut(id)?;
                let key = (pad.id.clone(), pad.leg.name.clone());
                let refused = padded.bad_legs.contains_key(&key);
                if let (Amount::Qty(g), false) = (&gap, refused) {
                    let sum = padded
                        .conserved
                        .entry(r.unit.clone())
                        .or_insert_with(Num::zero);
                    *sum = sum.checked_add(g).map_err(crate::eval::num_stop)?;
                    // L §5.10, E §4.2 step 5 (tally 2.1.2): the reading's own place is tested
                    // at its point too, in the point's group with the pad's leg; outside every
                    // group it was tested above
                    if let Some(group) = padded.open_group(at) {
                        let at_place = group.entry((r.place.clone(), r.unit.clone()));
                        at_place
                            .or_insert_with(Vec::new)
                            .push((pad.id.clone(), pad.leg.clone()));
                    }
                }
                let sub = at
                    .sub
                    .checked_add(1)
                    .ok_or_else(|| invariant("a reading's point"))?;
                let mut ent = Ent::new(at.with_sub(sub), Change::Reset { val, qs });
                ent.coll = Some(r.coll.clone());
                ent.reg = reg;
                ent.mv = Some(pad.id.clone());
                // the reading's end of the pad, which `flows` of the place counts: the gap, a
                // number or a hole (refcore's reset `flow`)
                ent.flow = match gap {
                    Amount::Refused(_) => None,
                    Amount::Qty(_) | Amount::Hole(_) => Some(gap),
                };
                padded.add(m, &r.place, &r.unit, ent)?;
                match padded.bad_legs.get(&key).filter(|_| breaking).cloned() {
                    // the balance the reading states breaks its place's sign rule: refused from
                    // there to the place's next reading, as an anchor's is, not ended by its own
                    // (L §5.10, E §4.2 step 5; tally 2.1.8, R11)
                    Some(err) => {
                        let mut held = Ent::new(at.with_sub(sub), Change::Bad(err));
                        held.mv = Some(pad.id.clone());
                        padded.add(m, &r.place, &r.unit, held)
                    }
                    None => Ok(()),
                }
            }
        }
    }

    /// The register of a place's booked balance at a point: its log's since the last reset,
    /// and the adjustments with a remainder there (refcore's `booked_coll`). E §6.1, L §8.2.
    fn booked_coll(
        &mut self,
        m: &Reality,
        id: usize,
        place: &str,
        unit: &str,
        at: At,
    ) -> Result<Collected, Stop> {
        let mut out = Collected::default();
        let (_, last) = self.walk(id)?.state(m, place, unit, at);
        if let Some(i) = last {
            let cum = self.walk_mut(id)?.cum(place, unit, i);
            out.merge(&cum);
        }
        let walk = self.walk(id)?;
        for (day, entry) in walk.adj_regs.get(place).into_iter().flatten() {
            if *day <= at.day {
                out.entries.insert(entry.clone());
            }
        }
        Ok(out)
    }
}

/// A pad of gap `gap` at reading `r`, from `other` into the place when the gap is positive or
/// unknown, else out of it (refcore's `PadMv`). E §6.1.
fn make_pad(r: &Reading, other: &str, gap: &Amount, coll: Collected, at: At) -> Result<Pad, Stop> {
    let (amount, into) = match gap {
        Amount::Qty(g) if g.is_negative() => (
            Amount::Qty(g.checked_neg().map_err(crate::eval::num_stop)?),
            false,
        ),
        Amount::Qty(g) => (Amount::Qty(g.clone()), !g.is_zero()),
        Amount::Hole(_) | Amount::Refused(_) => (gap.clone(), true),
    };
    let (from, to) = if into {
        (End::Place(other.to_owned()), End::Place(r.place.clone()))
    } else {
        (End::Place(r.place.clone()), End::Place(other.to_owned()))
    };
    Ok(Pad {
        id: format!("pads:{}:{}@{}", r.version, r.place, r.instant),
        leg: LegVal {
            name: "cash".to_owned(),
            from,
            to,
            amount,
            unit: Some(r.unit.clone()),
            day: Dated::Day(at.day),
            src: None,
        },
        coll,
    })
}

/// A state's value as an amount: the refusal it holds, a hole with every question it rests
/// on (`HOLE-ANCHOR` before the place's first reading), or the balance (refcore's
/// `st_value`). L §5.10, §8.1.
pub(super) fn booked_amount(st: &St, anchor: &Hole) -> Amount {
    if let Some(r) = &st.bad {
        return Amount::Refused(r.clone());
    }
    let missing = st.val.is_none().then_some(anchor);
    if let Some(h) = st.questions(missing) {
        return Amount::Hole(h);
    }
    match &st.val {
        Some(v) => Amount::Qty(v.clone()),
        None => Amount::Hole(anchor.clone()),
    }
}

/// The gap: the reading less the booked balance; the refusal the balance is; the holes of
/// either (refcore's `sub`). E §6.1.
fn gap_of(stated: &Result<Num, Hole>, booked: &Amount) -> Result<Amount, Stop> {
    let stated_hole = stated.as_ref().err();
    match booked {
        Amount::Refused(r) => Ok(Amount::Refused(r.clone())),
        Amount::Hole(h) => Ok(Amount::Hole(
            spread(&[stated_hole, Some(h)]).ok_or_else(|| invariant("a gap's hole"))?,
        )),
        Amount::Qty(b) => match stated {
            Err(h) => Ok(Amount::Hole(h.clone())),
            Ok(s) => Ok(Amount::Qty(
                s.checked_sub(b).map_err(crate::eval::num_stop)?,
            )),
        },
    }
}

/// A quantity of a unit as it prints. L §5.18.
fn qty_text(m: &Reality, q: &Num, unit: &str) -> Result<String, Stop> {
    qty_value(m, q, unit)?
        .text(m)
        .map_err(crate::eval::num_stop)
}

/// A gap as the register prints it: the amount, its hole, or `refused: <code> <message>`
/// (refcore's `gtext`). E §6.1, L §8.2.
pub(crate) fn gap_text(m: &Reality, gap: &Amount, unit: &str) -> Result<String, Stop> {
    match gap {
        Amount::Qty(q) => qty_text(m, q, unit),
        Amount::Hole(h) => Ok(h.text()),
        Amount::Refused(r) => Ok(format!("refused: {}", r.text())),
    }
}
