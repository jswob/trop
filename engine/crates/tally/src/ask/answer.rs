//! Answering the asks: each ask's cut, law date and pins, its binding tuples, each tuple's line of any form, read seen and unseen, and its register.
//! Spec: L §5.18 (asks: `as known`, `law`, `in`, `with`, `for`, `check`, `solve`, `diff`, one line
//! per binding tuple, a tuple whose `where` is a hole printed as that hole; tally 2.1.8 R13.1:
//! every clause on every form, a diff's own `for` per tuple, on one stack or two); L §4.3 (the
//! law date), §4.5 (the cut; tally 2.1.8 R12: a line that reads a `same` line known in a
//! window that holds the cut is answered with it seen and unseen, `HOLE-IMPRECISE` where the
//! two differ, a ledger's, diff's, `explain`'s or `inputs-of`'s block by every line, P27 on
//! #196; at most 20 000 choices, L §7.6 item 6, P27 on #174), §5.11 (an ask's `law`,
//! `as known` or pin that is a hole makes the ask
//! line that hole), §8.2 (the register), §8.4 (a run-time refusal prints as the tuple's line),
//! §8.5 (warnings last), §6.4 (the line's world: its `in`, else the model's; `law` and
//! `as known` in the model's), §5.5 (tally 2.1.7 R8: an ask of any form fans out over the
//! axes it reaches, `axes.rs`; a `diff`'s and a ledger's results are their sets of lines,
//! `diff.rs`, `residuals.rs`); E §11 (`REQ-ASK`, tally 2.1.3; `REQ-CLOCK`).
//! Twin: engine/tools/refcore/ask.py (`Asker.run`, `req_ask`, `answer`, `_answer`,
//! `answer_same_undecided`, `_answer_in`, `evaluate`, `one`, `one_solve`, `print_line`,
//! `fmt_value`, `fmt_bind`) and
//! evaluate.py (`date_of`, `apply_pins`); refcore answers an `as known` after `on` where E §11
//! refuses it (codes/m1 held.tsv, P22).
//! Invariants: asks print in id order unless the run names them; every id the run names is
//! checked before any ask is answered; each ask has its own engine and memo; a load refusal met
//! at run refuses the run in the base's world and the ask alone in a scenario's (refcore's
//! `LoadAtRun`); every construct not built yet stops the run `NotBuilt` at the ask that meets
//! it, never a guessed line.

use std::collections::{BTreeMap, BTreeSet};

use super::axes::{Axes, fanned_lines};
use super::check::{Printed, carried_of, check_line};
use super::diff::{Cells, cells_lines, crossed, diff_cells, diff_result};
use super::explain::{explain_line, inputs_line, readers_line};
use super::register::{check_register_lines, register_lines, warning_lines};
use super::residuals::residual_result;
use crate::base::{Code, Load, Place, Reach, Refusal, Req, Run, Warn};
use crate::book::SameMet;
use crate::eval::{
    Collected, Ctx, Engine, Entry, Env, Item, Member, Memo, after_on, combinations, known_question,
    names_in, num_stop, one_day, past_days, world_named,
};
use crate::model::{Reality, Value};
use crate::syntax::{
    Ask, AskBody, Bind, Diff, Domain, Expr, Key, Lit, OfQ, Pin, PinForm, Step, Stop,
};
use crate::time::Day;
use crate::walk::{Bookings, Forks, Lows, invariant};
use crate::{Asks, Days};

/// What a run's asks print, and, in the `counters` build, what their engines did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered {
    /// The lines: each ask's, then the run's warnings.
    pub lines: Vec<String>,
    /// The asks' engines' counts together (BUILDER §13.2).
    #[cfg(feature = "counters")]
    pub counts: crate::Counters,
}

/// What the asks gather across their engines: the warnings, each once, in the order met, and,
/// in the `counters` build, the engines' counts.
#[derive(Debug)]
struct Seen {
    /// The warnings, in the order met (L §8.5).
    warnings: Vec<(Warn, String)>,
    /// The booked plan amounts the asks met, each ask's engine checked against the earlier
    /// asks' (E §9 item 5).
    booked: Bookings,
    /// How each ask's engine keeps its cells across law dates (E §3.2).
    memo: Memo,
    /// How each ask's engine reads a written projection's common case (E §4.3, P §2.3).
    lows: Lows,
    /// Whether each ask's engine continues a projection's walk read again (P §2.5).
    forks: Forks,
    /// The side of a diff across two stacks the next ask answers, its cells once answered.
    side: Option<Side>,
    /// The fault planted in the run while it waits for its day (INTERFACE §4).
    #[cfg(feature = "fault-injection")]
    plant: Option<crate::walk::Fault>,
    /// The engines' counts together.
    #[cfg(feature = "counters")]
    counts: crate::Counters,
}

/// One side of a diff whose two worlds sit on two stacks of scenarios: the world this
/// reality's stack holds, and the diff's cells there once its ask is answered (refcore's
/// `_diff_only`, `_diff_side`). L §6.4.
#[derive(Debug)]
struct Side {
    /// The world.
    world: String,
    /// The cells per binding tuple, once answered.
    cells: Option<SideCells>,
    /// The choice of seen and unseen this side is answered under; none for the first run,
    /// which sees every `same` line (L §4.5, tally 2.1.8 R12).
    pick: Option<BTreeMap<String, bool>>,
    /// The `same` lines known in a window that holds the cut the side's run met, and that
    /// cut, once answered.
    met: Option<(SameMet, Day)>,
}

/// One side of a diff over two stacks: per binding tuple of the ask's `for`, its binding
/// part, ` (k = a)`, and its cells or the hole its line prints; one tuple with no part where
/// the ask binds nothing.
type SideCells = Vec<(String, Result<Cells, String>)>;

/// Both sides of a diff over two stacks answered: each side's cells, and the `same` lines
/// known in a window that holds the cut the two met, with that cut (L §6.4, §4.5).
type Crossing = (Vec<SideCells>, Option<(SameMet, Day)>);

/// How a run's engines compute: keeping their cells as `memo` says, reading a written
/// projection's common case as `lows` says and continuing a projection's walk read again as
/// `forks` says, the fences' forms or their plain twins (E §3.2, §4.3, P §2.3, §2.5, §3.3);
/// with the `fault-injection` feature, the fault planted in the run (INTERFACE §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct How {
    /// How the engines keep their cells across law dates.
    pub memo: Memo,
    /// How the engines read a written projection's common case.
    pub lows: Lows,
    /// Whether the engines continue a projection's walk read again through a later day.
    pub forks: Forks,
    /// The fault planted, if any.
    #[cfg(feature = "fault-injection")]
    pub fault: Option<crate::walk::Fault>,
}

impl How {
    /// The engines computing as `memo`, `lows` and `forks` say, no fault planted.
    #[must_use]
    pub const fn new(memo: Memo, lows: Lows, forks: Forks) -> How {
        How {
            memo,
            lows,
            forks,
            #[cfg(feature = "fault-injection")]
            fault: None,
        }
    }
}

impl Answered {
    /// Lines no ask's engine computed: a load's report (its warnings, a scenario's refusal).
    #[must_use]
    pub(crate) fn printed(lines: Vec<String>) -> Answered {
        Answered {
            lines,
            #[cfg(feature = "counters")]
            counts: crate::Counters::default(),
        }
    }
}

/// One step of a run's answer: an ask of a world, by the world's place among the run's
/// realities (the base's first) and the ask's among that reality's asks, or a line printed in
/// its stead (an ask whose scenario was refused, L §6.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Turn {
    /// The ask at this place of the asks of the reality at this place of the run's.
    Ask(usize, usize),
    /// A diff whose two worlds sit on two stacks of scenarios: each side, the world compared
    /// first, by the place of the reality whose stack holds it, its ask's place there and
    /// the world (refcore's `cross_stack_diff`, L §6.4).
    Cross([(usize, usize, String); 2]),
    /// A line, printed as it is.
    Line(String),
}

/// The lines a run prints for its asks: each ask's lines, the asks in id order or in the order
/// the run names them, then the run's warnings (refcore's `Asker.run`), each ask's engine
/// walking `days`, keeping its cells as `memo` says and reading a projection's common case
/// as `lows` says. L §5.18, §8.5.
///
/// # Errors
///
/// `REQ-ASK` for the first ask the run names that the reality lacks, before any is answered
/// (E §11); a load refusal met at run; `NotBuilt` for a construct not built yet.
pub fn answer(m: &Reality, asks: &Asks, days: Days, how: How) -> Result<Answered, Stop> {
    let turns = match asks {
        Asks::Every => {
            let mut all: Vec<(&str, usize)> = m
                .asks
                .iter()
                .enumerate()
                .map(|(at, ask)| (ask.id.as_str(), at))
                .collect();
            all.sort_by_key(|&(id, _)| id);
            all.into_iter().map(|(_, at)| Turn::Ask(0, at)).collect()
        }
        Asks::Named(ids) => {
            let mut named = Vec::with_capacity(ids.len());
            let mut member = false;
            for id in ids {
                match m.asks.iter().position(|ask| ask.id == *id) {
                    Some(at) => named.push(Turn::Ask(0, at)),
                    None if m.joint.as_ref().is_some_and(|j| j.knows(id)) => member = true,
                    None => return Err(Stop::Refused(no_ask(id))),
                }
            }
            if member {
                return Err(Stop::NotBuilt("a member's ask at run"));
            }
            named
        }
    };
    for turn in &turns {
        if let Turn::Ask(_, at) = turn {
            req_clock(m, m.asks.get(*at).ok_or_else(no_turn)?)?;
        }
    }
    answer_turns(&[m], &turns, m.warnings.clone(), days, how)
}

/// `REQ-ASK no ask <q>`: the request names an ask that doesn't exist. E §11 (tally 2.1.3).
pub(crate) fn no_ask(id: &str) -> Refusal {
    Refusal::new(Code::Req(Req::Ask), format!("no ask {id}"))
}

/// The lines of `turns` in order, each ask's engine walking `days`, keeping its cells as
/// `memo` says and reading a projection's common case as `lows` says, then `warnings` with the
/// walks' own (refcore's `Asker.run`, `run_worlds`).
/// L §5.18, §8.5.
///
/// # Errors
///
/// A load refusal met at run in the base's world; `NotBuilt` for a construct not built yet.
pub(crate) fn answer_turns(
    realities: &[&Reality],
    turns: &[Turn],
    warnings: Vec<(Warn, String)>,
    days: Days,
    how: How,
) -> Result<Answered, Stop> {
    let mut out = Vec::new();
    // each world's asks share what their engines gather, apart from another world's
    // (refcore's one `Asker` per world)
    let mut worlds: Vec<World> = realities.iter().map(|m| World::of(m, how)).collect();
    for turn in turns {
        match turn {
            Turn::Ask(world, at) => {
                let m = realities.get(*world).ok_or_else(no_turn)?;
                let gathered = worlds.get_mut(*world).ok_or_else(no_turn)?;
                match gathered.answer(m, *at, days) {
                    // a load refusal met at run refuses the run in the base's world, the ask
                    // alone in a scenario's (refcore's `run_worlds`, `LoadAtRun`)
                    Err(Stop::Refused(r)) if *world > 0 && matches!(r.code(), Code::Load(_)) => {
                        let ask = m.asks.get(*at).ok_or_else(no_turn)?;
                        out.push(format!("{}: refused: {}", ask.id, r.text()));
                    }
                    lines => out.extend(lines?),
                }
            }
            Turn::Cross(sides) => out.extend(cross_lines(realities, &mut worlds, sides, days)?),
            Turn::Line(line) => out.push(line.clone()),
        }
        // the fault is planted once per run, whichever world's walk met its day
        #[cfg(feature = "fault-injection")]
        if worlds.iter().any(|w| w.seen.plant.is_none()) {
            for w in &mut worlds {
                w.seen.plant = None;
            }
        }
    }
    // the realities' warnings, then each world's walks' own, the base's first, each once
    let mut warned = warnings;
    for world in &worlds {
        for w in &world.seen.warnings {
            if !warned.contains(w) {
                warned.push(w.clone());
            }
        }
    }
    out.extend(warning_lines(&warned));
    #[cfg(feature = "counters")]
    let mut counts = crate::Counters::default();
    #[cfg(feature = "counters")]
    for world in &worlds {
        counts.merge(&world.seen.counts);
    }
    Ok(Answered {
        lines: out,
        #[cfg(feature = "counters")]
        counts,
    })
}

/// The lines of a diff whose two worlds sit on two stacks of scenarios: each side answered by
/// the world of the reality whose stack holds it, as that reality holds the ask, then printed
/// as one diff, cell by cell over the axes either side reads; a side whose ask is refused or
/// a hole before its cells prints that line alone (refcore's `cross_stack_diff`). Where the
/// sides met a `same` line known in a window that holds the cut, both sides are answered again
/// under each other choice of seen and unseen, their warnings and bookings not kept, and the
/// blocks compared by every line (L §4.5, tally 2.1.8 R12; P27 on #196). L §6.4, §5.5, §5.18.
///
/// # Errors
///
/// A load refusal met at run; `RUN-LIMIT` past [`SAME_LIMIT`] choices; `NotBuilt` for a
/// construct not built yet.
fn cross_lines(
    realities: &[&Reality],
    worlds: &mut [World],
    sides: &[(usize, usize, String); 2],
    days: Days,
) -> Result<Vec<String>, Stop> {
    let [(place, at, _), _] = sides;
    let m = realities.get(*place).ok_or_else(no_turn)?;
    let ask = m.asks.get(*at).ok_or_else(no_turn)?;
    let AskBody::Diff(diff) = &ask.body else {
        return Err(no_turn());
    };
    let (got, met) = match side_cells(realities, worlds, (sides, days), None)? {
        Ok(answered) => answered,
        Err(lines) => return Ok(lines),
    };
    let first = crossed_tuples(m, (ask, diff), &got)?;
    let Some((met, cut)) = met else {
        return Ok(first);
    };
    let mut others = Vec::new();
    for pick in same_picks(&met)? {
        let kept: Vec<_> = worlds
            .iter()
            .map(|w| (w.seen.booked.clone(), w.seen.warnings.clone()))
            .collect();
        let picked = side_cells(realities, worlds, (sides, days), Some(&pick));
        for (w, (booked, warnings)) in worlds.iter_mut().zip(kept) {
            w.seen.booked = booked;
            w.seen.warnings = warnings;
        }
        others.push(match picked? {
            Ok((cells, _)) => crossed_tuples(m, (ask, diff), &cells)?,
            Err(lines) => lines,
        });
    }
    let hole = same_hole(&met, cut)?;
    Ok(undecided_lines((&ask.id, true), &first, &others, &hole))
}

/// Both sides of a diff over two stacks answered under choice `pick` of seen and unseen, none
/// for the first run: each side's cells, and the `same` lines known in a window that holds the
/// cut the two met, with that cut; or the lines of a side whose ask is refused or a hole
/// before its cells (refcore's `cross_stack_diff`). L §6.4, §4.5.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
fn side_cells(
    realities: &[&Reality],
    worlds: &mut [World],
    (sides, days): (&[(usize, usize, String); 2], Days),
    pick: Option<&BTreeMap<String, bool>>,
) -> Result<Result<Crossing, Vec<String>>, Stop> {
    let mut got = Vec::with_capacity(sides.len());
    let mut all: Option<(SameMet, Day)> = None;
    for (place, at, world) in sides {
        let m = realities.get(*place).ok_or_else(no_turn)?;
        let gathered = worlds.get_mut(*place).ok_or_else(no_turn)?;
        gathered.seen.side = Some(Side {
            world: world.clone(),
            cells: None,
            pick: pick.cloned(),
            met: None,
        });
        let lines = gathered.answer(m, *at, days);
        let side = gathered.seen.side.take();
        let lines = lines?;
        let Some(Side {
            cells: Some(cells),
            met,
            ..
        }) = side
        else {
            return Ok(Err(lines));
        };
        got.push(cells);
        if let Some((met, cut)) = met {
            all.get_or_insert_with(|| (SameMet::new(), cut))
                .0
                .extend(met);
        }
    }
    Ok(Ok((got, all)))
}

/// The blocks of a diff over two stacks from its sides' cells `got`: one per binding tuple,
/// each side's at that tuple; a side's hole, or a run-time refusal met crossing them, that
/// tuple's line (L §5.18, tally 2.1.8 R13.1; §8.4).
///
/// # Errors
///
/// A load refusal; `NotBuilt` for sides that bind other tuples.
fn crossed_tuples(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    got: &[SideCells],
) -> Result<Vec<String>, Stop> {
    let [ours, theirs] = got else {
        return Err(no_turn());
    };
    let mut out = Vec::new();
    for (part, left) in ours {
        let head = format!("{}{part}", ask.id);
        let right = theirs
            .iter()
            .find(|(other, _)| other == part)
            .map(|(_, cells)| cells)
            .ok_or(Stop::NotBuilt(
                "a diff over two stacks whose sides bind other tuples",
            ))?;
        match (left, right) {
            (Err(hole), _) | (_, Err(hole)) => out.push(format!("{head}: {hole}")),
            (Ok(l), Ok(r)) => match crossed(m, (l, r))
                .and_then(|cells| cells_lines(m, (ask, diff, &head), &cells))
            {
                Ok(lines) => out.extend(lines),
                // a run-time refusal, past S7's runs say, prints as the tuple's line (L §8.4)
                Err(Stop::Refused(refusal)) if !matches!(refusal.code(), Code::Load(_)) => {
                    out.push(format!("{head}: refused: {}", refusal.text()));
                }
                Err(stop) => return Err(stop),
            },
        }
    }
    Ok(out)
}

/// One side of a diff whose worlds sit on two stacks, in context `ctx` with the ask's pins
/// `pinned`: its cells in world `world` per binding tuple of the ask's `for`, each with what
/// its tuple brought; a tuple whose `where` is a hole that hole; or the one line of a domain
/// that is a hole (L §5.18, tally 2.1.8 R13; §6.4).
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` past what the engine builds.
fn side_tuples(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
    world: &str,
) -> Result<Result<SideCells, String>, Stop> {
    if ask.binds.is_empty() {
        let cells = diff_cells(m, (ask, diff), engine, (&Env::empty(), ctx, pinned), world)?;
        return Ok(Ok(vec![(String::new(), Ok(cells))]));
    }
    let tuples = match engine.tuples(m, &ask.binds, (&Env::empty(), ctx))? {
        Err(hole) => return Ok(Err(format!("{}: {}", ask.id, hole.text()))),
        Ok(tuples) => tuples,
    };
    let mut out = Vec::with_capacity(tuples.len());
    for tuple in tuples {
        let part = env_text(m, &ask.binds, &tuple.env)?;
        if let Some(hole) = tuple.unknown {
            out.push((part, Err(hole.text())));
            continue;
        }
        let mut cells = diff_cells(m, (ask, diff), engine, (&tuple.env, ctx, pinned), world)?;
        cells.bring(&tuple.brought);
        out.push((part, Ok(cells)));
    }
    Ok(Ok(out))
}

/// What a world's asks' engines gather, and what its reality holds that the run doesn't take
/// yet (refcore's `Asker` of a world).
struct World {
    /// What its asks' engines gather.
    seen: Seen,
    /// Whether it holds a known window the run doesn't take yet.
    unbuilt: bool,
    /// Whether it keeps parts, which the run doesn't take yet.
    parts: bool,
}

impl World {
    /// The world of reality `m` before any of its asks, its engines keeping cells as `memo`
    /// says and reading a projection's common case as `lows` says.
    fn of(m: &Reality, how: How) -> World {
        World {
            seen: Seen {
                warnings: Vec::new(),
                booked: BTreeMap::new(),
                memo: how.memo,
                lows: how.lows,
                forks: how.forks,
                side: None,
                #[cfg(feature = "fault-injection")]
                plant: how.fault,
                #[cfg(feature = "counters")]
                counts: crate::Counters::default(),
            },
            unbuilt: known_windows_unbuilt(m),
            parts: keeps_parts(m),
        }
    }

    /// The lines of the ask at `at` of the world's reality `m`: refused alone, or answered
    /// (refcore's `Asker.answer`). L §5.18, §6.4.
    fn answer(&mut self, m: &Reality, at: usize, days: Days) -> Result<Vec<String>, Stop> {
        let ask = m.asks.get(at).ok_or_else(no_turn)?;
        if let Some(refusal) = m.asks_alone.get(&ask.id) {
            return Ok(vec![format!("{}: refused: {}", ask.id, refusal.text())]);
        }
        if self.unbuilt {
            return Err(Stop::NotBuilt(
                "a moves line's, a rule's, a band table's, a key's or an entry's known \
                 window at run",
            ));
        }
        if self.parts {
            return Err(Stop::NotBuilt("parts at run"));
        }
        if let Some(form) = unbuilt_at_run(m, ask) {
            return Err(Stop::NotBuilt(form));
        }
        answer_one(m, ask, days, &mut self.seen)
    }
}

/// Whether the reality holds a known window the run doesn't take yet (L §4.5, §5.3), so an
/// ask of it stops `NotBuilt`: a `moves` line's, or the window of a version that brings a
/// rule, a band table, an index key or a thing's entry. A version's or a row's own window on
/// an input's or a def's rows, on a reading, on an observation and on a coverage is taken
/// (`eval/read.rs`, `book/reading.rs`, `book/observe.rs`), and so is a `moves` line known on
/// a day.
fn known_windows_unbuilt(m: &Reality) -> bool {
    let mut windowed = BTreeSet::new();
    for (_, v) in m.versions.entries() {
        if v.moved_by.iter().any(|mv| mv.window.is_some()) {
            return true;
        }
        if v.window.is_some() {
            windowed.insert(v.id.as_str());
        }
    }
    !windowed.is_empty() && brought_windowed(m, &windowed)
}

/// Whether one of `windowed`, the versions known in a window, brings a rule, a band table,
/// an index key or a thing's entry (L §4.5).
fn brought_windowed(m: &Reality, windowed: &BTreeSet<&str>) -> bool {
    for (_, d) in m.defs.entries() {
        for (_, vers) in &d.rules {
            for r in vers {
                if in_windowed(windowed, r.version.as_deref())
                    || in_windowed(windowed, r.v.as_deref())
                {
                    return true;
                }
            }
        }
    }
    for (_, b) in m.bands.entries() {
        if b.tables
            .iter()
            .any(|t| windowed.contains(t.version.as_str()))
        {
            return true;
        }
    }
    for (_, ix) in m.indexes.entries() {
        if ix
            .keys()
            .iter()
            .any(|k| in_windowed(windowed, ix.added_by(k)))
        {
            return true;
        }
    }
    for (_, t) in m.things.entries() {
        if t.entries
            .iter()
            .any(|e| in_windowed(windowed, e.version.as_deref()))
        {
            return true;
        }
    }
    false
}

/// Whether `version` is one of `windowed`.
fn in_windowed(windowed: &BTreeSet<&str>, version: Option<&str>) -> bool {
    match version {
        Some(v) => windowed.contains(v),
        None => false,
    }
}

/// Whether a kind of the reality keeps parts, or a reading states parts: load checks them
/// (L §3.6, §5.7, §5.19), the walk doesn't book them yet (M5b's: a place that keeps parts
/// carries them in every walk), so an ask stops `NotBuilt`.
fn keeps_parts(m: &Reality) -> bool {
    m.kinds.entries().any(|(_, kind)| !kind.parts.is_empty())
        || m.inputs
            .entries()
            .any(|(_, input)| !input.part_rows.is_empty())
}

/// A form load checks (M0) that the run doesn't evaluate yet, where the ask meets it: a random
/// ask, `orient`, a fill, or a joint model's merged reality (L §5.12, §5.16, §5.18, §6.1). A
/// version's rows and movements stated `same` are built (`eval/read.rs`, `book/same.rs`).
fn unbuilt_at_run(m: &Reality, ask: &Ask) -> Option<&'static str> {
    if ask.random.is_some() || ask.paths.is_some() || ask.path.is_some() {
        return Some("a random ask at run");
    }
    if ask.orient.is_some() {
        return Some("orient at run");
    }
    if !m.fills.is_empty() {
        return Some("fills at run");
    }
    if m.joint.is_some() {
        return Some("a joint model's asks at run");
    }
    None
}

/// A turn whose ask the reality doesn't hold: the turns are built from its asks.
fn no_turn() -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "a turn names an ask the reality doesn't hold".to_owned(),
    ))
}

/// One ask's lines; a run-time refusal its reading meets outside a line's values is its one
/// line, and a load refusal met at run is returned to stop its world (refcore's `answer`,
/// `LoadAtRun`: a load refusal load can't see, such as a rate per year times days read
/// through rows, L §3.3); a request's refusal is the run's one result (E §11). L §8.3, §8.4.
fn answer_one(m: &Reality, ask: &Ask, days: Days, seen: &mut Seen) -> Result<Vec<String>, Stop> {
    if formed_per_tuple(ask) {
        return tuple_asks(m, ask, days, seen);
    }
    answer_in(m, ask, &Env::empty(), days, seen)
}

/// [`answer_one`] of ask `ask` in scope `env`, its own binding tuple's where it has one.
fn answer_in(
    m: &Reality,
    ask: &Ask,
    env: &Env,
    days: Days,
    seen: &mut Seen,
) -> Result<Vec<String>, Stop> {
    refusal_line(ask, ask_lines(m, ask, env, days, seen))
}

/// Ask `ask`'s lines `lines`, or the one line of the run-time refusal its reading met; a load
/// or request refusal stays the run's (L §8.3, §8.4; E §11).
fn refusal_line(ask: &Ask, lines: Result<Vec<String>, Stop>) -> Result<Vec<String>, Stop> {
    match lines {
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_) | Code::Req(_)) => {
            Ok(vec![format!("{}: refused: {}", ask.id, r.text())])
        }
        other => other,
    }
}

/// One ask's lines: its cut by `as known`, its law date by `law`, then one line per binding
/// tuple with the register under it; once answered, the timelines its engines walked are
/// checked unchanged (refcore's `_answer`, `_answer_in`). L §5.18, E §9 item 4.
fn ask_lines(
    m: &Reality,
    ask: &Ask,
    env: &Env,
    days: Days,
    seen: &mut Seen,
) -> Result<Vec<String>, Stop> {
    let mut cut = m.on;
    let mut known_engine = None;
    if let Some(known) = &ask.known {
        let mut engine = ask_engine(m.on, (ask, env), days, seen);
        let v =
            model_ctx(m, &mut engine, (None, cut)).and_then(|ctx| engine.value(m, known, env, ctx));
        gather(seen, &mut engine);
        let v = v?;
        if let Value::Hole(_) = v {
            return Ok(vec![format!("{}: {}", ask.id, shown(m, &v)?)]);
        }
        cut = match one_day(m, &v, "as known")? {
            Ok(known_day) => known_day,
            Err(h) => return Ok(vec![format!("{}: {}", ask.id, shown(m, &Value::Hole(h))?)]),
        };
        cut = after_on(m, cut)?;
        known_engine = Some(engine);
    }
    let mut engine = ask_engine(cut, (ask, env), days, seen);
    // a side of a diff over two stacks is answered under the choice its crossing gives
    if let Some(side) = &seen.side {
        engine.same_picks.pick.clone_from(&side.pick);
    }
    let out = ask_body(m, ask, &mut engine, (cut, seen.side.as_mut()));
    gather(seen, &mut engine);
    let mut met = core::mem::take(&mut engine.same_picks.met);
    // and its first run's `same` lines go to the crossing, which reads the choices
    // (`cross_lines`)
    if let Some(side) = seen.side.as_mut() {
        if side.pick.is_none() && !met.is_empty() {
            side.met = Some((core::mem::take(&mut met), engine.ask_cut));
        }
        met.clear();
    }
    let lines = if met.is_empty() {
        out?
    } else {
        refusal_line(ask, out)?
    };
    // E §9 item 4: asks move no money; the walks the ask made are as they were when walked
    if let Some(first) = &known_engine {
        first.asks_moved_no_money()?;
    }
    engine.asks_moved_no_money()?;
    if met.is_empty() {
        return Ok(lines);
    }
    let hole = same_hole(&met, engine.ask_cut)?;
    let others = picked_lines(m, (ask, env), (cut, days), seen, &met)?;
    Ok(undecided_lines(
        (&ask.id, by_every_line(ask)),
        &lines,
        &others,
        &hole,
    ))
}

/// Ask `ask`'s lines in scope `env` at cut `cut` under each choice of seen and unseen for
/// the `same` lines `met` but the first run's, each on an engine of its own, its warnings and
/// bookings not kept (refcore's `answer_same_undecided`, on fresh caches). L §4.5 (tally
/// 2.1.8 R12).
///
/// # Errors
///
/// A load or request refusal a run meets; `RUN-LIMIT` past [`SAME_LIMIT`] choices. A diff
/// over two stacks reads its choices where its sides are crossed (`cross_lines`).
fn picked_lines(
    m: &Reality,
    (ask, env): (&Ask, &Env),
    (cut, days): (Day, Days),
    seen: &mut Seen,
    met: &SameMet,
) -> Result<Vec<Vec<String>>, Stop> {
    let mut others = Vec::new();
    for pick in same_picks(met)? {
        let booked = seen.booked.clone();
        let mut engine = ask_engine(cut, (ask, env), days, seen);
        engine.same_picks.pick = Some(pick);
        let out = ask_body(m, ask, &mut engine, (cut, None));
        seen.booked = booked;
        #[cfg(feature = "counters")]
        seen.counts.merge(engine.counters());
        others.push(refusal_line(ask, out)?);
        engine.asks_moved_no_money()?;
    }
    Ok(others)
}

/// Every choice of seen and unseen for the `same` lines `met` but the first run's, which saw
/// them all: the versions in id order, the first slowest, seen before unseen (refcore's
/// `answer_same_undecided`, its `picks`). L §4.5 (tally 2.1.8 R12).
///
/// # Errors
///
/// `RUN-LIMIT` past [`SAME_LIMIT`] choices, before any is read (S7, pick P27 on #174).
fn same_picks(met: &SameMet) -> Result<Vec<BTreeMap<String, bool>>, Stop> {
    let counts = vec![2; met.len()];
    let Some(combos) = combinations(&counts, SAME_LIMIT) else {
        return Err(Stop::Refused(Refusal::new(
            Code::Run(Run::Limit),
            SAME_LIMIT_TEXT.to_owned(),
        )));
    };
    let mut out = Vec::new();
    for combo in combos.into_iter().skip(1) {
        let pick = met
            .keys()
            .zip(combo)
            .map(|(id, at)| (id.clone(), at == 0))
            .collect();
        out.push(pick);
    }
    Ok(out)
}

/// The most choices of seen and unseen one line is read under: S7's limit, which reaches a
/// line's seen and unseen choices as it reaches its day combinations (pick P27, on #174;
/// L §7.6 item 6). Past it the line is `RUN-LIMIT`, [`SAME_LIMIT_TEXT`].
const SAME_LIMIT: usize = 20_000;

/// The message of a line past [`SAME_LIMIT`]: the builder's words, until the spec's commit
/// gives the bytes (`spec(ask): a line's seen and unseen choices fall under its limit`).
const SAME_LIMIT_TEXT: &str = "seen and unseen over more than 20 000 choices";

/// The `HOLE-IMPRECISE` of the `same` lines `met` under cut `cut`: for each line's version,
/// `date within window(a, b) not precise enough for the cut K`, asked of the rows or
/// movements its lines join, then of the version (refcore's `answer_same_undecided`). L §4.5
/// (tally 2.1.8 R12), §8.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for no line met.
fn same_hole(met: &SameMet, cut: Day) -> Result<crate::hole::Hole, Stop> {
    let mut asked = Vec::new();
    for (version, (window, sides)) in met {
        let question = known_question(*window, cut);
        for side in sides {
            asked.push(crate::hole::Hole::asked(&question, side));
        }
        asked.push(crate::hole::Hole::asked(&question, version));
    }
    let operands: Vec<_> = asked.iter().map(Some).collect();
    crate::hole::spread(&operands).ok_or_else(|| invariant("a same line's hole with no line"))
}

/// The lines of ask `id` where its first run, `first`, saw every `same` line known in a
/// window that holds the cut and the runs `others` took the other choices: the lines where
/// every run prints them; where the runs' heads differ, the one line of `hole` with the first
/// run's last cut or law date line (refcore's `answer_same_undecided`). Else block by block (a
/// line and the lines under it): a block of a form that prints lines of its own (`whole`: a
/// ledger, a diff, an `explain`, an `inputs-of`) that differs in any line becomes its head
/// and `hole`, with its last cut or law date line, as where heads differ (pick P27, on #196);
/// a value, `check` or `solve` block whose first line differs becomes its head and `hole`,
/// the first run's register under it, its register free to differ. L §4.5 (tally 2.1.8 R12: a
/// read on which seen and unseen differ is `HOLE-IMPRECISE`), §8.2.
fn undecided_lines(
    (id, whole): (&str, bool),
    first: &[String],
    others: &[Vec<String>],
    hole: &crate::hole::Hole,
) -> Vec<String> {
    if others.iter().all(|lines| lines.as_slice() == first) {
        return first.to_vec();
    }
    let own = heads(first);
    if others.iter().any(|lines| heads(lines) != own) {
        let mut out = vec![format!("{id}: {}", hole.text())];
        out.extend(last_clock(first).cloned());
        return out;
    }
    let theirs: Vec<Vec<Vec<&str>>> = others.iter().map(|lines| blocks(lines)).collect();
    let mut out = Vec::new();
    for (at, block) in blocks(first).into_iter().enumerate() {
        let Some((lead, under)) = block.split_first() else {
            continue;
        };
        let agree = theirs.iter().all(|bs| match bs.get(at) {
            Some(other) if whole => *other == block,
            Some(other) => other.first() == Some(lead),
            None => false,
        });
        if agree || !lead.contains(": ") {
            out.extend(block.iter().map(|l| (*l).to_owned()));
            continue;
        }
        out.push(format!("{}: {}", line_head(lead), hole.text()));
        if whole {
            out.extend(block_clock(under).map(|l| (*l).to_owned()));
        } else {
            out.extend(under.iter().map(|l| (*l).to_owned()));
        }
    }
    out
}

/// Whether seen and unseen compare ask `ask`'s blocks by every line: a ledger, a diff, an
/// `explain` or an `inputs-of` prints its result in lines under its head; a value, `check`,
/// `solve` or `readers-of` block by its first line, its register free to differ (pick P27,
/// on #196). L §4.5 (tally 2.1.8 R12), §5.18.
fn by_every_line(ask: &Ask) -> bool {
    match ask.body {
        AskBody::Residuals(_) | AskBody::Diff(_) | AskBody::Explain(_) | AskBody::InputsOf(_) => {
            true
        }
        AskBody::Exprs(_) | AskBody::Check(..) | AskBody::Solve(_) | AskBody::ReadersOf(_) => false,
    }
}

/// The last cut or law date line among a block's lines `under`.
fn block_clock<'a>(under: &[&'a str]) -> Option<&'a str> {
    under
        .iter()
        .rev()
        .find(|l| l.starts_with("  cut") || l.starts_with("  law date"))
        .copied()
}

/// Lines `lines` in blocks: a line and the indented lines under it.
fn blocks(lines: &[String]) -> Vec<Vec<&str>> {
    let mut out: Vec<Vec<&str>> = Vec::new();
    for line in lines {
        match out.last_mut() {
            Some(block) if line.starts_with(' ') => block.push(line),
            _ => out.push(vec![line.as_str()]),
        }
    }
    out
}

/// The first line of each block of `lines`.
fn leads(lines: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    for block in blocks(lines) {
        out.extend(block.first());
    }
    out
}

/// The head of each block of `lines`, its first line's.
fn heads(lines: &[String]) -> Vec<&str> {
    leads(lines).into_iter().map(line_head).collect()
}

/// The last cut or law date line under the last block of `lines`.
fn last_clock(lines: &[String]) -> Option<&String> {
    let mut found = None;
    for line in lines {
        if !line.starts_with(' ') {
            found = None;
        } else if line.starts_with("  cut") || line.starts_with("  law date") {
            found = Some(line);
        }
    }
    found
}

/// A line's head: its text before its first `: `, the whole line where it has none.
fn line_head(line: &str) -> &str {
    line.split_once(": ").map_or(line, |(head, _)| head)
}

/// The cuts the request's asks state as literals, `as known D` or `with known D` (`what`), on or
/// before the model's `on`: past it, the request is refused after load, as `REQ-ASK` is, the
/// run's one result, `REQ-CLOCK <what> <D> is after the model's on <on>` (E §11; R5, #112;
/// refcore's `req_clock`). A computed cut is refused at its value (`after_on`).
fn req_clock(m: &Reality, ask: &Ask) -> Result<(), Stop> {
    let as_known = ask.known.iter().map(|e| ("as known", e));
    let with_known = ask.pins.iter().filter_map(|pin| match &pin.form {
        PinForm::Known(e) => Some(("with known", e)),
        PinForm::Law(_) | PinForm::Wording(..) | PinForm::Scenario(_) | PinForm::Set { .. } => None,
    });
    for (what, e) in as_known.chain(with_known) {
        if let Expr::Lit(Lit::Day(cut)) = e
            && *cut > m.on
        {
            return Err(Stop::Refused(Refusal::new(
                Code::Req(Req::Clock),
                format!(
                    "{what} {} is after the model's on {}",
                    cut.text(),
                    m.on.text()
                ),
            )));
        }
    }
    Ok(())
}

/// The walk warnings an ask's engine met, each once, in the order met (refcore's
/// `Asker.walk_warnings`); the booked plan amounts, for the next ask (E §9 item 5); and, in
/// the `counters` build, its counts. L §8.5.
fn gather(seen: &mut Seen, engine: &mut Engine) {
    for w in engine.walk_warnings() {
        if !seen.warnings.contains(w) {
            seen.warnings.push(w.clone());
        }
    }
    seen.booked = engine.take_booked();
    #[cfg(feature = "fault-injection")]
    seen.plant.clone_from(&engine.plant);
    #[cfg(feature = "counters")]
    seen.counts.merge(engine.counters());
}

/// One ask's lines from its cut on: its law date, then a line per binding tuple with the
/// register under it, or the residual ledger; for a diff's `side` on one stack, no line, its
/// cells kept there (refcore's `_answer`, `_answer_in`). L §5.18, E §6.3.
fn ask_body(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (cut, side): (Day, Option<&mut Side>),
) -> Result<Vec<String>, Stop> {
    let mut law = None;
    if let Some(e) = &ask.law {
        let in_model = model_ctx(m, engine, (None, cut))?;
        let env = engine.ask_env.clone();
        let v = engine.value(m, e, &env, in_model)?;
        if let Value::Hole(_) = v {
            return Ok(vec![format!("{}: {}", ask.id, shown(m, &v)?)]);
        }
        law = match one_day(m, &v, "the law of an ask")? {
            Ok(day) => Some(day),
            Err(h) => return Ok(vec![format!("{}: {}", ask.id, shown(m, &Value::Hole(h))?)]),
        };
    }
    let mut pinned = Collected::default();
    // `in s` names the line's world, the model's when it names none; a world under the
    // reality's own hides the layers over it (L §5.18, §6.4)
    let world = match &ask.world {
        Some(id) => {
            pinned.entries.insert(Entry {
                word: "world",
                text: id.clone(),
                figure: String::new(),
            });
            world_named(m, id)?
        }
        None => m.scenarios.model_world(),
    };
    // the line's own reads take its `with uncertain`; `as known` and `law` don't (L §5.18)
    let plain = Ctx::new(law, cut).in_world(world);
    let ctx = if ask.uncertain {
        plain.with_uncertain()
    } else {
        plain
    };
    // and its `partial` (L §5.18 Partial answers)
    let ctx = if ask.partial { ctx.partial() } else { ctx };
    engine.enter_timeline(m, ctx)?;
    let ctx = if ask.pins.is_empty() {
        ctx
    } else {
        // the ask's `with` pins: evaluated once, outside the line, their register under every
        // line (refcore's `apply_pins` with `ask=True`); a pin whose key or date is a hole makes
        // the ask line that hole (L §5.11)
        engine.push();
        let read = engine.with_pins(m, &ask.pins, ctx);
        pinned.merge(&engine.pop_alone());
        let inside = match read? {
            Err(hole) => return Ok(vec![format!("{}: {}", ask.id, hole.text())]),
            Ok(inside) => inside,
        };
        engine.ask_cut = inside.cut;
        engine.unpinned = Some(ctx);
        engine.pins_read.keeping = true;
        engine.pins_read.ask = pinned
            .entries
            .iter()
            .filter(|e| e.word == "pinned")
            .map(|e| e.text.clone())
            .collect();
        engine.open_layer(m, inside)?;
        inside
    };
    match (&ask.body, side) {
        // `readers-of` is static: its tuples are read for a domain that is a hole, and it
        // fans out over no axis (refcore's `_answer_in`, its readers-of return)
        (AskBody::ReadersOf(reference), _) => {
            if !ask.binds.is_empty()
                && let Err(hole) = engine.tuples(m, &ask.binds, (&Env::empty(), ctx))?
            {
                return Ok(vec![format!("{}: {}", ask.id, hole.text())]);
            }
            Ok(vec![readers_line(m, ask, reference)?])
        }
        // one side of a diff whose worlds sit on two stacks: its cells, under `windows each`
        // each cell's runs, kept for the turn that prints both (refcore's `_diff_only`,
        // `_diff_side`; tally 2.1.8 R13, #186)
        (AskBody::Diff(diff), Some(side)) => {
            match side_tuples(m, (ask, diff), engine, (ctx, &pinned), &side.world)? {
                Ok(cells) => side.cells = Some(cells),
                Err(line) => return Ok(vec![line]),
            }
            Ok(Vec::new())
        }
        // every other form: per binding tuple, axis combination and sub-window, its result
        // what the form prints (L §5.18, tally 2.1.8 R13)
        _ => tuple_lines(m, ask, engine, (ctx, &pinned)),
    }
}

/// A context of the model's world, its timeline entered: where an ask's `law` and `as known`
/// are evaluated (refcore's `Ctx(None, K)` in the base's world). L §5.18, §6.4.
fn model_ctx(
    m: &Reality,
    engine: &mut Engine,
    (law, cut): (Option<Day>, Day),
) -> Result<Ctx, Stop> {
    let ctx = Ctx::new(law, cut).in_world(m.scenarios.model_world());
    engine.enter_timeline(m, ctx)?;
    Ok(ctx)
}

/// An ask's lines: a line per binding tuple with the register under it, fanned out over the
/// axes the ask reaches and, under `windows each`, split by sub-window, each line the result
/// its form prints: values, a check, a solve, an `explain`, an `inputs-of`, a residual ledger
/// or a diff (refcore's `_answer_in`, `print_line`). L §5.18 (tally 2.1.8 R13: every clause
/// stands on every form), §5.5, §4.4.
fn tuple_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    clause_lines(m, ask, engine, (ctx, pinned))
}

/// [`tuple_lines`] of an ask whose binding its form takes: its `windows each` clause, a
/// read's, or neither. L §4.4, §5.18.
fn clause_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    if ask.windows_each {
        // the line's clause: one run per sub-window (L §4.4)
        return super::windows::each_lines(m, ask, engine, (ctx, pinned));
    }
    if m.graph.read_each {
        return read_each_lines(m, ask, engine, (ctx, pinned));
    }
    plain_lines(m, ask, engine, (ctx, pinned))
}

/// The lines of ask `ask` in a reality where a read states `windows each`, which holds for
/// that read (L §4.4, §8.2): the line runs plain, such a read taking its windows' first days;
/// only where one met a window does the line run once per sub-window, so a line no such read
/// reaches answers as it would without the clause (refcore's `proj_clauses`).
///
/// # Errors
///
/// As [`tuple_lines`].
fn read_each_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let probe = engine.with_read_windows(ctx, &Vec::new())?;
    let outer = engine.each_met.replace(Vec::new());
    engine.open_layer(m, probe)?;
    let lines = plain_lines(m, ask, engine, (probe, pinned));
    engine.end_layer(probe.layer, lines.as_ref().err())?;
    let met = core::mem::replace(&mut engine.each_met, outer)
        .ok_or_else(|| invariant("a line's windows met taken while it ran"))?;
    let lines = lines?;
    if met.is_empty() {
        return Ok(lines);
    }
    super::windows::each_lines(m, ask, engine, (ctx, pinned))
}

/// The lines of ask `ask` with no `windows each` to run: per binding tuple, or fanned out
/// over the axes it reaches (L §5.5, §5.18).
///
/// # Errors
///
/// As [`tuple_lines`].
fn plain_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    if let Some(axes) = Axes::of(m, ask)? {
        return fanned_lines(m, ask, engine, (ctx, pinned), &axes);
    }
    let tuples = if ask.binds.is_empty() {
        vec![Member {
            env: engine.ask_env.clone(),
            unknown: None,
            brought: Collected::default(),
        }]
    } else {
        match engine.tuples(m, &ask.binds, (&Env::empty(), ctx))? {
            Err(hole) => return Ok(vec![format!("{}: {}", ask.id, hole.text())]),
            Ok(tuples) => tuples,
        }
    };
    let mut out = Vec::new();
    for tuple in tuples {
        let head = format!("{}{}", ask.id, env_text(m, &ask.binds, &tuple.env)?);
        if let Some(hole) = tuple.unknown {
            out.push(format!("{head}: {}", hole.text()));
            continue;
        }
        let mut line = form_line(m, ask, engine, (&tuple.env, ctx, pinned))?;
        out.extend(line.lines(&head));
        if line.alone {
            continue;
        }
        line.met.merge(&tuple.brought);
        line.met.merge(&ask_pins_listed(pinned, &line.met));
        let register = match line.laws {
            Some(laws) => check_register_lines(&line.met, ctx, laws)?,
            None => register_lines(&line.met, ctx)?,
        };
        out.extend(register.into_iter().map(|l| format!("  {l}")));
    }
    Ok(out)
}

/// The ask's `with` pins and what they read, as a line lists them: an ask pin the line's
/// reads didn't take, which its own read set records as met under an inner or narrower claim
/// of its cell (`pin met`: an `as if`'s pin, a `solve`'s variable; the innermost claim wins,
/// L §4.6), is in no read of it and isn't listed; a pin no read reaches stays listed, as
/// refcore's walk reads don't reach a refused result yet (L §8.2 One read set; tally 2.1.7
/// R7, P45; refcore's `print_line`, `pin_value`).
pub(super) fn ask_pins_listed(pinned: &Collected, line: &Collected) -> Collected {
    let mut kept = pinned.clone();
    kept.entries.retain(|e| {
        let met = Entry {
            word: "pin met",
            text: e.text.clone(),
            figure: String::new(),
        };
        e.word != "pinned" || line.entries.contains(e) || !line.entries.contains(&met)
    });
    kept
}

/// One tuple's line after its head in scope `env` and context `ctx`, with the register
/// entries it met: the result its form prints, the values of `e, f, …`, a `check`'s, a
/// `solve`'s, an `explain`'s, an `inputs-of`'s, a residual ledger or a diff, whose register
/// holds the ask's pins `pinned` (refcore's `evaluate`, `answer_residuals`, `answer_diff`). A
/// line holding a bare key literal meets its `LOAD-KEY ambiguous key` where it is evaluated
/// (L §2.5, §8.3; P39). L §5.18 (tally 2.1.8 R13).
pub(super) fn form_line(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
) -> Result<Printed, Stop> {
    match &ask.body {
        AskBody::Exprs(exprs) => {
            let (items, met) = engine.line(m, exprs, env, ctx)?;
            if let Some(place) = m.asks_unresolved.get(&ask.id) {
                unresolved_word(&items, place)?;
            }
            let carried = carried_of(&items.iter().collect::<Vec<_>>());
            Ok(Printed {
                text: line_values(m, &items)?,
                body: Vec::new(),
                bare: false,

                whole: false,
                alone: false,
                met,
                laws: None,
                carried,
                causes: None,
            })
        }
        AskBody::Check(left, right) => check_line(m, engine, (left, right), env, ctx),
        AskBody::Solve(solve) => {
            let (item, met) = engine.solve_line(m, solve, env, ctx)?;
            Ok(Printed {
                text: item_text(m, &item)?,
                body: Vec::new(),
                bare: false,

                whole: false,
                alone: false,
                met,
                laws: None,
                carried: carried_of(&[&item]),
                causes: None,
            })
        }
        AskBody::Explain(reference) => explain_line(m, (ask, reference), engine, env, ctx),
        AskBody::InputsOf(reference) => inputs_line(m, (ask, reference), engine, env, ctx),
        AskBody::Residuals(r) => residual_result(m, r, engine, (env, ctx, pinned)),
        AskBody::Diff(diff) => diff_result(m, diff, engine, (env, ctx, pinned)),
        AskBody::ReadersOf(_) => Err(invariant("readers-of read as a tuple's line")),
    }
}

/// `LOAD-KEY ambiguous key <k>` at the ask's line `place` for the first of a line's values
/// that holds a key literal no position resolved, bare (an `if` may have chosen it) or as a
/// list's item: refcore refuses the literal where it is evaluated (its `e_name`, the `keylit`
/// arm; pick P39). L §2.5, §8.3.
pub(super) fn unresolved_word(items: &[Item], place: &Place) -> Result<(), Stop> {
    let Some(word) = items.iter().find_map(item_word) else {
        return Ok(());
    };
    let message = format!("ambiguous key {word}");
    let refusal = Refusal::at(Code::Load(Load::Key), message, place.clone());
    Err(Stop::Refused(refusal.reaching(Reach::Id(word.clone()))))
}

/// The key literal a line's value is, or the first a list holds. L §2.5.
fn item_word(item: &Item) -> Option<&String> {
    match item {
        Item::Value(Value::List(values)) => values.iter().find_map(value_word),
        Item::Value(value) => value_word(value),
        Item::Refused(_) => None,
    }
}

/// The key literal a value is, unresolved. L §2.5.
fn value_word(value: &Value) -> Option<&String> {
    let Value::Word(word) = value else {
        return None;
    };
    Some(word)
}

/// A line's values: one alone, several in parentheses (refcore's `print_line`). L §5.18.
fn line_values(m: &Reality, items: &[Item]) -> Result<String, Stop> {
    let mut texts = Vec::with_capacity(items.len());
    for item in items {
        texts.push(item_text(m, item)?);
    }
    match texts.as_slice() {
        [one] => Ok(one.clone()),
        _ => Ok(format!("({})", texts.join(", "))),
    }
}

/// A value as a line prints it: by its type, a key outside its index and a refusal as
/// `refused: <code> <message>` (refcore's `fmt_value`). L §5.18, §8.4.
pub(super) fn item_text(m: &Reality, item: &Item) -> Result<String, Stop> {
    if let Item::Value(Value::Hole(h)) = item
        && let Some(text) = crate::eval::partial_text(m, h)?
    {
        // a `partial` line's hole: its known part and the holes it is linear in (L §5.18)
        return Ok(text);
    }
    match item {
        Item::Refused(r) => Ok(format!("refused: {}", r.text())),
        Item::Value(Value::OutKey { index, text }) => Ok(format!(
            "refused: {} {text} is outside {index}",
            Code::Run(Run::Key).as_str()
        )),
        Item::Value(Value::Day(d)) if d.is_end() => Err(past_days(1)),
        Item::Value(Value::Window(w)) if !w.of_days() => {
            // a window prints as a date; any other interval needs its one value (L §4.4)
            let window = w.question_text(m).map_err(num_stop)?;
            let text = format!("date within {window} not precise enough for a print");
            Ok(crate::hole::Hole::asked(&text, w.figure())
                .carrying(&w.carry)
                .text())
        }
        Item::Value(v) => v.typed_text(m).map_err(num_stop),
    }
}

/// A value as an ask's head line prints it when it stands alone: a hole, a law date's or an
/// `as known`'s.
fn shown(m: &Reality, v: &Value) -> Result<String, Stop> {
    v.text(m).map_err(num_stop)
}

/// A tuple's variables as its line's head prints them, ` (v = x, w = y)`: an integer key of a
/// key list as written (refcore's `fmt_bind`). L §5.18, §2.5.
pub(super) fn env_text(m: &Reality, binds: &[Bind], env: &Env) -> Result<String, Stop> {
    if binds.is_empty() {
        return Ok(String::new());
    }
    Ok(format!(" {}", tuple_key(m, binds, env)?))
}

/// A tuple's variables in parentheses, `(v = x, w = y)`: its line's head, a `diff`'s key
/// (refcore's `fmt_bind`). L §5.18, §2.5.
pub(super) fn tuple_key(m: &Reality, binds: &[Bind], env: &Env) -> Result<String, Stop> {
    let mut parts = Vec::with_capacity(binds.len());
    for bind in binds {
        let var = env.get(&bind.var).ok_or_else(|| {
            Stop::Refused(Refusal::new(
                Code::EngineInvariant,
                format!("an ask's tuple without {}", bind.var),
            ))
        })?;
        parts.push(format!(
            "{} = {}",
            bind.var,
            bound_text(m, bind, &var.value)?
        ));
    }
    Ok(format!("({})", parts.join(", ")))
}

/// A bound value as a line's head prints it: an integer key of a key list as written, any
/// other value as it prints (refcore's `fmt_bind`). L §2.5.
fn bound_text(m: &Reality, bind: &Bind, v: &Value) -> Result<String, Stop> {
    if let (Domain::List(_), Value::Number(n)) = (&bind.domain, v)
        && let Some(whole) = n.q.to_i64()
    {
        return Ok(whole.to_string());
    }
    shown(m, v)
}

/// Whether ask `ask`'s `as known`, `law` or a `with` pin names a variable of its own `for`:
/// each is then formed per binding tuple, so each tuple is a line of its own, with its own
/// cut, law date and pins (L §5.18, §5.11 A pin's keys).
fn formed_per_tuple(ask: &Ask) -> bool {
    if ask.binds.is_empty() {
        return false;
    }
    for e in [&ask.known, &ask.law].into_iter().flatten() {
        if names_a_for(ask, e) {
            return true;
        }
    }
    pins_name_for(ask)
}

/// An engine answering ask `ask` at cut `cut` in scope `env`, the world's memo, lows, forks
/// and the bookings the asks before it made taken from `seen` (refcore's `Asker`, its engine per
/// ask). L §5.18.
fn ask_engine(cut: Day, (ask, env): (&Ask, &Env), days: Days, seen: &mut Seen) -> Engine {
    let engine = Engine::new(cut)
        .with_days(days)
        .with_how((seen.memo, seen.lows, seen.forks));
    let mut engine = engine
        .for_ask(ask)
        .in_tuple(env)
        .with_booked(core::mem::take(&mut seen.booked));
    #[cfg(feature = "fault-injection")]
    engine.plant.clone_from(&seen.plant);
    engine
}

/// The lines of ask `ask` whose cut, law date or pins are formed per its own binding tuple:
/// the binding's tuples read in the model's world at its `on`, then each tuple's line as an
/// ask of its own, its head the tuple's (`q (k = a)`), in the tuple's scope; a tuple whose
/// membership is a hole prints that hole (L §5.18, §5.11, §2.5).
fn tuple_asks(m: &Reality, ask: &Ask, days: Days, seen: &mut Seen) -> Result<Vec<String>, Stop> {
    let mut engine = ask_engine(m.on, (ask, &Env::empty()), days, seen);
    let read = model_ctx(m, &mut engine, (None, m.on))
        .and_then(|ctx| engine.tuples(m, &ask.binds, (&Env::empty(), ctx)));
    gather(seen, &mut engine);
    let members = match read {
        Ok(Ok(members)) => members,
        Ok(Err(hole)) => return Ok(vec![format!("{}: {}", ask.id, hole.text())]),
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_) | Code::Req(_)) => {
            return Ok(vec![format!("{}: refused: {}", ask.id, r.text())]);
        }
        Err(stop) => return Err(stop),
    };
    engine.asks_moved_no_money()?;
    let mut out = Vec::new();
    for member in members {
        let head = env_text(m, &ask.binds, &member.env)?;
        if let Some(hole) = member.unknown {
            out.push(format!("{}{head}: {}", ask.id, hole.text()));
            continue;
        }
        let one = Ask {
            id: format!("{}{head}", ask.id),
            binds: Vec::new(),
            ..ask.clone()
        };
        out.extend(answer_in(m, &one, &member.env, days, seen)?);
    }
    Ok(out)
}

/// Whether one of ask `ask`'s `with` pins names a variable of its own `for`, by its value, a
/// key or a date: a pin formed per binding tuple (L §8.2), which the ask's pins, evaluated
/// once outside the line, can't take. A form whose names aren't walked keeps its path.
fn pins_name_for(ask: &Ask) -> bool {
    for pin in &ask.pins {
        let named = match &pin.form {
            PinForm::Set { target, value, .. } => {
                names_a_for(ask, value)
                    || target
                        .keys
                        .iter()
                        .flatten()
                        .any(|key| matches!(key, Key::Expr(e) if names_a_for(ask, e)))
            }
            PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => names_a_for(ask, e),
            PinForm::Scenario(_) => false,
        };
        if named {
            return true;
        }
    }
    false
}

/// Whether expression `e` names a variable of ask `ask`'s own `for`, by the walk of the names
/// it reads; a form whose names that walk doesn't follow (an aggregate, a walk's read) is
/// walked by [`mentions`]. L §8.2.
pub(super) fn names_a_for(ask: &Ask, e: &Expr) -> bool {
    let Some(names) = names_in(e) else {
        return ask.binds.iter().any(|b| mentions(e, &b.var));
    };
    for name in names {
        if ask.binds.iter().any(|b| b.var == name) {
            return true;
        }
    }
    false
}

/// Whether expression `e` may name variable `var`: every part walked, those [`extra_parts`]
/// adds to the walk's children included, a binding of its own of that name too; a form whose
/// parts aren't walked here may (a conservative test: such a pin stops "not implemented"
/// rather than read the variable as a key literal). L §8.2.
fn mentions(e: &Expr, var: &str) -> bool {
    let mut work = vec![e];
    while let Some(x) = work.pop() {
        if let Expr::Name(n) = x
            && n == var
        {
            return true;
        }
        if matches!(
            x,
            Expr::Calendar(_)
                | Expr::Query(_)
                | Expr::Low(_)
                | Expr::Crps(_)
                | Expr::DistFn(_)
                | Expr::Dist(_)
                | Expr::Prorata(_)
        ) {
            return true;
        }
        work.extend(extra_parts(x));
        work.extend(crate::load::expr_children(x));
    }
    false
}

/// The parts of read form `x` the walk's children leave out: an aggregate's bindings' ranges,
/// `on` and `where` and its `ties by`; a balance's `from` and `of`; a value's `by` and `of`;
/// a leg's `of`; a step read from an expression; a span's ends; an `as if`'s pins. L §8.2.
fn extra_parts(x: &Expr) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = Vec::new();
    match x {
        Expr::Agg(agg) => {
            for bind in &agg.binds {
                if let Domain::Range(ends) = &bind.domain {
                    out.extend([&ends.0, &ends.1]);
                    out.extend(ends.2.iter());
                }
                out.extend(bind.on.iter());
                out.extend(bind.filter.iter());
            }
            out.extend(agg.ties.iter().flatten());
        }
        Expr::Bal(bal) => {
            out.extend(bal.proj.from.iter());
            out.extend(bal.of.iter().flat_map(of_parts));
        }
        Expr::Value(v) => {
            out.extend(v.by.iter().map(|p| &p.input));
            out.extend(v.of.iter().flat_map(of_parts));
        }
        Expr::LegOf(_, of) => out.extend(of_parts(of)),
        Expr::To(_, step) => {
            if let Step::Ref(e) = step.as_ref() {
                out.push(e);
            }
        }
        Expr::Flows(flows) => {
            out.extend([&flows.span.from, &flows.span.to].into_iter().flatten());
            out.extend(flows.for_key.iter());
            out.extend(flows.convert.iter().flat_map(|(_, a, b)| [a, b]));
        }
        Expr::Covered(pair) => out.extend([&pair.1.from, &pair.1.to].into_iter().flatten()),
        Expr::AsIf(_, pins) => {
            for pin in pins {
                out.extend(pin_parts(pin));
            }
        }
        Expr::Last(_)
        | Expr::Calendar(_)
        | Expr::Query(_)
        | Expr::Low(_)
        | Expr::Crps(_)
        | Expr::DistFn(_)
        | Expr::Dist(_)
        | Expr::Prorata(_)
        | Expr::Lit(_)
        | Expr::Str(_)
        | Expr::Name(_)
        | Expr::None
        | Expr::Never
        | Expr::Cut
        | Expr::List(_)
        | Expr::If(_)
        | Expr::Or(_)
        | Expr::And(_)
        | Expr::Not(_)
        | Expr::Neg(_)
        | Expr::Paren(_)
        | Expr::Cmp(_)
        | Expr::In(_)
        | Expr::Sum(..)
        | Expr::Prod(..)
        | Expr::Post(..)
        | Expr::Call(_)
        | Expr::Record(_) => {}
    }
    out
}

/// The expressions of an `of q`: one reference, or a pair's two.
fn of_parts(of: &OfQ) -> Vec<&Expr> {
    match of {
        OfQ::Ref(e) => vec![e],
        OfQ::Pair(a, b) => vec![a, b],
    }
}

/// The expressions of pin `pin`: a value pin's value and keys, a date pin's date.
fn pin_parts(pin: &Pin) -> Vec<&Expr> {
    match &pin.form {
        PinForm::Set { target, value, .. } => {
            let mut out = vec![value];
            for key in target.keys.iter().flatten() {
                if let Key::Expr(e) = key {
                    out.push(e);
                }
            }
            out
        }
        PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => vec![e],
        PinForm::Scenario(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::undecided_lines;
    use crate::hole::Hole;

    fn owned(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|l| (*l).to_owned()).collect()
    }

    const HEAD: &str = "r: residuals konto 2026-04";
    const GROUP: &str =
        "  2026-04 unexplained: model 0.00 PLN, observed -100.00 PLN, difference -100.00 PLN";

    /// Seen and unseen accounts of one block under one first line (the review of group 1's
    /// N5; pick P27, on #196): a ledger's, a diff's, an `explain`'s or an `inputs-of`'s block
    /// that differs in any line is the hole, with its last cut or law date line; a value's
    /// block compares by its first line, its register free to differ.
    #[test]
    fn a_block_of_lines_is_compared_by_every_line() {
        let hole = Hole::asked(
            "date within window(2026-05-01, 2026-05-20) not precise enough for the cut 2026-05-10",
            "link1",
        );
        let hole_line = format!("r: {}", hole.text());
        let seen = owned(&[
            HEAD,
            GROUP,
            "    unexplained bank[a1], bank[b1] on 2026-04-10: observed -100.00 PLN",
            "  law date 2026-04-01, cut 2026-05-10",
            "q: 3",
            "  cut 2026-05-10",
        ]);
        let unseen = owned(&[
            HEAD,
            GROUP,
            "    unexplained bank[a1] on 2026-04-10: observed -100.00 PLN",
            "  law date 2026-04-01, cut 2026-05-10",
            "q: 3",
            "  cut 2026-05-10",
        ]);
        let others = [unseen];
        assert_eq!(
            undecided_lines(("r", true), &seen, &others, &hole),
            [
                hole_line.as_str(),
                "  law date 2026-04-01, cut 2026-05-10",
                "q: 3",
                "  cut 2026-05-10"
            ],
            "a ledger's line under one head moves: the hole and its clock"
        );
        assert_eq!(
            undecided_lines(("r", false), &seen, &others, &hole),
            seen,
            "a value's block: its first line agrees"
        );
        let moved = owned(&["q: 3", "  open: two", "  cut 2026-05-10"]);
        let still = owned(&["q: 3", "  cut 2026-05-10"]);
        assert_eq!(
            undecided_lines(("q", false), &moved, &[still], &hole),
            moved,
            "a value's register may differ"
        );
    }
}
