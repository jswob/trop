//! Answering the asks: each ask's cut, law date and pins, its binding tuples, each tuple's line of any form, and its register.
//! Spec: L §5.18 (asks: `as known`, `law`, `in`, `with`, `for`, `check`, `solve`, `diff`, one line
//! per binding tuple, a tuple whose `where` is a hole printed as that hole; tally 2.1.8 R13.1:
//! every clause on every form, a diff's own `for` per tuple, on one stack or two); L §4.3 (the
//! law date), §4.5 (the cut), §5.11 (an ask's `law`, `as known` or pin that is a hole makes the
//! ask line that hole), §8.2 (the register), §8.4 (a run-time refusal prints as the tuple's
//! line),
//! §8.5 (warnings last), §6.4 (the line's world: its `in`, else the model's; `law` and
//! `as known` in the model's), §5.5 (tally 2.1.7 R8: an ask of any form fans out over the
//! axes it reaches, `axes.rs`; a `diff`'s and a ledger's results are their sets of lines,
//! `diff.rs`, `residuals.rs`); E §11 (`REQ-ASK`, tally 2.1.3; `REQ-CLOCK`).
//! Twin: engine/tools/refcore/ask.py (`Asker.run`, `req_ask`, `answer`, `_answer`,
//! `_answer_in`, `evaluate`, `one`, `one_solve`, `print_line`, `fmt_value`, `fmt_bind`) and
//! evaluate.py (`date_of`, `apply_pins`); refcore answers an `as known` after `on` where E §11
//! refuses it (codes/m1 held.tsv, P22).
//! Invariants: asks print in id order unless the run names them; every id the run names is
//! checked before any ask is answered; each ask has its own engine and memo; a load refusal met
//! at run refuses the run in the base's world and the ask alone in a scenario's (refcore's
//! `LoadAtRun`); every construct not built yet stops the run `NotBuilt` at the ask that meets
//! it, never a guessed line; a joint model's member's ask, `a.q`, is answered by the member
//! alone at its place in the run's order (`members.rs`).

use std::collections::BTreeMap;

use super::axes::{Axes, fanned_lines};
use super::check::{Printed, check_line, hole_days};
use super::diff::{Cells, cells_lines, crossed, diff_cells, diff_result};
use super::explain::{explain_line, inputs_line, readers_line};
use super::members;
use super::register::{check_register_lines, name_unknown_days, register_lines, warning_lines};
use super::residuals::residual_result;
use super::worlds::refused_line;
use crate::base::{Code, Load, Place, Reach, Refusal, Req, Run, Warn};
use crate::eval::{
    Collected, Ctx, Engine, Entry, Env, How, Item, Member, Memo, after_on, names_in, num_stop,
    one_day, past_days, world_named,
};
use crate::model::{Reality, Value, Window};
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
    /// The binding part of the tuple answered now, ` (k = a)`, where the ask's law date, cut
    /// or pins are formed per tuple and each tuple is answered as an ask of its own
    /// (`tuple_asks`); empty otherwise.
    part: String,
}

/// One side of a diff over two stacks: per binding tuple of the ask's `for`, its binding
/// part, ` (k = a)`, and its cells or the hole its line prints; one tuple with no part where
/// the ask binds nothing.
type SideCells = Vec<(String, Result<Cells, String>)>;

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
    /// A member's ask the run names, `a.q` or `a.b.q`, before `members::resolve` turns it
    /// into the turns that answer it (L §6.1).
    Member(String),
    /// The ask at this place of the asks of the member these aliases name, from the run's
    /// model down, answered by that member alone, its lines under its prefix (L §6.1).
    OfMember(Vec<String>, usize),
    /// A stop met where the member alone answered its ask, which stops the run at this turn.
    Stopped(Stop),
}

/// The lines a run prints for its asks: each ask's lines, the asks in id order or in the order
/// the run names them, a joint model's member's answered by the member alone, then the run's
/// warnings (refcore's `Asker.run`), each ask's engine walking `days`, keeping its cells as
/// `memo` says and reading a projection's common case as `lows` says. L §5.18, §6.1, §8.5.
///
/// # Errors
///
/// `REQ-ASK` for the first ask the run names that the reality lacks, before any is answered
/// (E §11); a load refusal met at run; `NotBuilt` for a construct not built yet.
pub fn answer(m: &Reality, asks: &Asks, days: Days, how: How) -> Result<Answered, Stop> {
    let turns = members::resolve(m, reality_turns(m, asks)?, (days, how))?;
    req_clocks(&[m], &turns)?;
    answer_turns(&[m], &turns, m.warnings.clone(), days, how)
}

/// The turns of a reality with no scenario file: its asks in id order, or in the order the
/// run names them, a member's ask (`a.q`) as `Turn::Member`. E §11 (`REQ-ASK`).
///
/// # Errors
///
/// `REQ-ASK` for the first ask the run names that the reality lacks.
pub(super) fn reality_turns(m: &Reality, asks: &Asks) -> Result<Vec<Turn>, Stop> {
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
            for id in ids {
                match m.asks.iter().position(|ask| ask.id == *id) {
                    Some(at) => named.push(Turn::Ask(0, at)),
                    None if m.joint.as_ref().is_some_and(|j| j.knows(id)) => {
                        named.push(Turn::Member(id.clone()));
                    }
                    None => return Err(Stop::Refused(no_ask(id))),
                }
            }
            named
        }
    };
    Ok(turns)
}

/// `REQ-CLOCK` before any ask is answered, in the run's order: the first ask whose literal cut
/// is after its model's `on` refuses the request (E §11), whether it is an ask of the run's
/// realities (`realities`, the base's first), a side of a diff on two stacks, a member's ask
/// answered in the member's world, or one whose member alone refused the request.
///
/// # Errors
///
/// `REQ-CLOCK`, or the member alone's request refusal, as the run's one result.
pub(super) fn req_clocks(realities: &[&Reality], turns: &[Turn]) -> Result<(), Stop> {
    let base = realities.first().ok_or_else(no_turn)?;
    for turn in turns {
        match turn {
            Turn::Ask(world, at) => req_clock_at(realities, *world, *at)?,
            Turn::Cross(sides) => {
                for (world, at, _) in sides {
                    req_clock_at(realities, *world, *at)?;
                }
            }
            Turn::OfMember(aliases, at) => {
                let m = &base.member_alone(aliases).ok_or_else(no_turn)?.reality;
                req_clock(m, m.asks.get(*at).ok_or_else(no_turn)?)?;
            }
            Turn::Stopped(Stop::Refused(refusal)) if matches!(refusal.code(), Code::Req(_)) => {
                return Err(Stop::Refused(refusal.clone()));
            }
            Turn::Line(_) | Turn::Member(_) | Turn::Stopped(_) => {}
        }
    }
    Ok(())
}

/// `REQ-CLOCK` for the ask at `at` of the reality at `world` among `realities`. E §11.
fn req_clock_at(realities: &[&Reality], world: usize, at: usize) -> Result<(), Stop> {
    let m = realities.get(world).ok_or_else(no_turn)?;
    req_clock(m, m.asks.get(at).ok_or_else(no_turn)?)
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
    let (mut answered, warned) = answer_turns_apart(realities, turns, warnings, days, how)?;
    answered.lines.extend(warning_lines(&warned));
    Ok(answered)
}

/// The lines of `turns` in order, as [`answer_turns`] prints them, and apart from them the
/// warnings it prints after them: `warnings`, then the walks' own, each once. L §5.18, §8.5.
///
/// # Errors
///
/// As [`answer_turns`].
pub(super) fn answer_turns_apart(
    realities: &[&Reality],
    turns: &[Turn],
    warnings: Vec<(Warn, String)>,
    days: Days,
    how: How,
) -> Result<(Answered, Vec<(Warn, String)>), Stop> {
    let mut out = Vec::new();
    // each world's asks share what their engines gather, apart from another world's
    // (refcore's one `Asker` per world); each member's, by its aliases, likewise
    let mut worlds: Vec<World> = realities.iter().map(|m| World::of(m, how)).collect();
    let mut member_worlds: BTreeMap<&[String], World> = BTreeMap::new();
    for turn in turns {
        match turn {
            Turn::OfMember(aliases, at) => {
                let base = realities.first().ok_or_else(no_turn)?;
                let m = &base.member_alone(aliases).ok_or_else(no_turn)?.reality;
                let gathered = member_worlds
                    .entry(aliases.as_slice())
                    .or_insert_with(|| World::of(m, how));
                #[cfg(feature = "fault-injection")]
                if worlds.iter().any(|w| w.seen.plant.is_none()) {
                    gathered.seen.plant = None;
                }
                // a load refusal met at run refuses the run, as in the base's world
                let lines = gathered.answer(m, *at, days)?;
                out.extend(members::prefixed(aliases, lines));
            }
            Turn::Member(_) => return Err(invariant("a member's ask the run didn't resolve")),
            Turn::Stopped(stop) => return Err(stop.clone()),
            Turn::Ask(world, at) => {
                let m = realities.get(*world).ok_or_else(no_turn)?;
                let gathered = worlds.get_mut(*world).ok_or_else(no_turn)?;
                match gathered.answer(m, *at, days) {
                    // a load refusal met at run refuses the run in the base's world, the ask
                    // alone in a scenario's (refcore's `run_worlds`, `LoadAtRun`)
                    Err(Stop::Refused(r)) if *world > 0 && matches!(r.code(), Code::Load(_)) => {
                        let ask = m.asks.get(*at).ok_or_else(no_turn)?;
                        out.push(refused_line(&ask.id, &r));
                    }
                    lines => out.extend(lines?),
                }
            }
            Turn::Cross(sides) => out.extend(cross_lines(realities, &mut worlds, sides, days)?),
            Turn::Line(line) => out.push(line.clone()),
        }
        // the fault is planted once per run, whichever world's walk met its day
        #[cfg(feature = "fault-injection")]
        if worlds
            .iter()
            .chain(member_worlds.values())
            .any(|w| w.seen.plant.is_none())
        {
            for w in worlds.iter_mut().chain(member_worlds.values_mut()) {
                w.seen.plant = None;
            }
        }
    }
    // the realities' warnings, then each world's walks' own, the base's first, each once; a
    // member's walks warn only where the member runs alone (refcore's `Asker.member_ask`)
    let mut warned = warnings;
    for world in &worlds {
        for w in &world.seen.warnings {
            if !warned.contains(w) {
                warned.push(w.clone());
            }
        }
    }
    #[cfg(feature = "counters")]
    let mut counts = crate::Counters::default();
    #[cfg(feature = "counters")]
    for world in worlds.iter().chain(member_worlds.values()) {
        counts.merge(&world.seen.counts);
    }
    let answered = Answered {
        lines: out,
        #[cfg(feature = "counters")]
        counts,
    };
    Ok((answered, warned))
}

/// The lines of a diff whose two worlds sit on two stacks of scenarios: each side answered by
/// the world of the reality whose stack holds it, as that reality holds the ask, then printed
/// as one diff, cell by cell over the axes either side reads; a side whose ask is refused or
/// a hole before its cells prints that line alone (refcore's `cross_stack_diff`). L §6.4,
/// §5.5, §5.18.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
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
    match side_cells(realities, worlds, (sides, days))? {
        Ok(got) => crossed_tuples(m, (ask, diff), &got),
        Err(lines) => Ok(lines),
    }
}

/// Both sides of a diff over two stacks answered: each side's cells; or the lines of a side
/// whose ask is refused or a hole before its cells (refcore's `cross_stack_diff`). L §6.4.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
fn side_cells(
    realities: &[&Reality],
    worlds: &mut [World],
    (sides, days): (&[(usize, usize, String); 2], Days),
) -> Result<Result<Vec<SideCells>, Vec<String>>, Stop> {
    let mut got = Vec::with_capacity(sides.len());
    for (place, at, world) in sides {
        let m = realities.get(*place).ok_or_else(no_turn)?;
        let gathered = worlds.get_mut(*place).ok_or_else(no_turn)?;
        gathered.seen.side = Some(Side {
            world: world.clone(),
            cells: None,
            part: String::new(),
        });
        let lines = gathered.answer(m, *at, days);
        let side = gathered.seen.side.take();
        let lines = lines?;
        let Some(Side {
            cells: Some(cells), ..
        }) = side
        else {
            return Ok(Err(lines));
        };
        got.push(cells);
    }
    Ok(Ok(got))
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
            (Ok(l), Ok(r)) => {
                match crossed(m, (l, r)).and_then(|cells| cells_lines(m, (diff, &head), &cells)) {
                    Ok(lines) => out.extend(lines),
                    // a run-time refusal, past S7's runs say, prints as the tuple's line (L §8.4)
                    Err(Stop::Refused(refusal)) if !matches!(refusal.code(), Code::Load(_)) => {
                        out.push(refused_line(&head, &refusal));
                    }
                    Err(stop) => return Err(stop),
                }
            }
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
        // the tuple's scope where it is answered as an ask of its own (`tuple_asks`)
        let env = engine.ask_env.clone();
        let cells = diff_cells(m, (ask, diff), engine, (&env, ctx, pinned), world)?;
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
            parts: keeps_parts(m),
        }
    }

    /// The lines of the ask at `at` of the world's reality `m`: refused alone, or answered
    /// (refcore's `Asker.answer`). L §5.18, §6.4.
    fn answer(&mut self, m: &Reality, at: usize, days: Days) -> Result<Vec<String>, Stop> {
        let ask = m.asks.get(at).ok_or_else(no_turn)?;
        if let Some(refusal) = m.asks_alone.get(&ask.id) {
            return Ok(vec![refused_line(&ask.id, refusal)]);
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
            Ok(vec![refused_line(&ask.id, &r)])
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
    let out = ask_body(m, ask, &mut engine, (cut, seen.side.as_mut()));
    gather(seen, &mut engine);
    let lines = out?;
    // E §9 item 4: asks move no money; the walks the ask made are as they were when walked
    if let Some(first) = &known_engine {
        first.asks_moved_no_money()?;
    }
    engine.asks_moved_no_money()?;
    Ok(lines)
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
        // one side of a diff whose worlds sit on two stacks: its cells, kept for the turn
        // that prints both (refcore's `_diff_only`, `_diff_side`; tally 2.1.8 R13, #186)
        (AskBody::Diff(diff), Some(side)) => {
            // a tuple answered as an ask of its own adds its cells to the side's
            match side_tuples(m, (ask, diff), engine, (ctx, &pinned), &side.world)? {
                Ok(cells) => side.cells.get_or_insert_with(Vec::new).extend(
                    cells
                        .into_iter()
                        .map(|(part, cell)| (format!("{}{part}", side.part), cell)),
                ),
                Err(line) => return Ok(vec![line]),
            }
            Ok(Vec::new())
        }
        // every other form: per binding tuple and axis combination, its result what the
        // form prints (L §5.18, tally 2.1.8 R13)
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

/// An ask's lines: a line per binding tuple with the register under it, or fanned out over
/// the axes the ask reaches, each line the result its form prints: values, a check, a solve,
/// an `explain`, an `inputs-of`, a residual ledger or a diff (refcore's `_answer_in`,
/// `print_line`). L §5.18 (tally 2.1.8 R13: every clause stands on every form), §5.5.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
fn tuple_lines(
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
        name_unknown_days(&mut line.met, &line.unknown);
        let register = match line.laws {
            Some(laws) => check_register_lines(&line.met, ctx, laws)?,
            None => register_lines(&line.met, ctx)?,
        };
        out.extend(register.into_iter().map(|l| format!("  {l}")));
    }
    Ok(out)
}

/// The ask's `with` pins and what they read, as a line with read set `line` lists them: an
/// entry is listed exactly when a read in its set gives it, so an ask's pin of a cell (noted
/// `pin cell`) that no read of the line took isn't listed; a pin of an axis or a clock stays
/// listed (L §8.2 One read set; tally 2.1.7 R7, P45; tally 2.1.8 §4, #161; refcore's
/// `print_line`, `pin_value`).
pub(super) fn ask_pins_listed(pinned: &Collected, line: &Collected) -> Collected {
    let mut kept = pinned.clone();
    kept.entries.retain(|e| {
        e.word != "pinned"
            || !pinned.entries.contains(&pin_entry("pin cell", &e.text))
            || line.entries.contains(e)
    });
    kept
}

/// The ask's pins of a cell, `pinned`, that a line whose every value is refused takes: a
/// refusal carries no read set of its own, so what it rests on took each pin, but one its
/// read set `line` records as met under an inner or narrower claim of its cell (`pin met`: an
/// `as if`'s pin, a `solve`'s variable; the innermost claim wins, L §4.6) (refcore's
/// `print_line`, its `refused`; tally 2.1.8 §4, #161). L §8.2 One read set.
fn refused_pins(pinned: &Collected, line: &Collected) -> Collected {
    let mut kept = Collected::default();
    for e in &pinned.entries {
        let cell = pinned.entries.contains(&pin_entry("pin cell", &e.text));
        if e.word == "pinned" && cell && !line.entries.contains(&pin_entry("pin met", &e.text)) {
            kept.entries.insert(e.clone());
        }
    }
    kept
}

/// The register entry `word` of pin `text`, as a read set notes it (L §8.2).
fn pin_entry(word: &'static str, text: &str) -> Entry {
    Entry {
        word,
        text: text.to_owned(),
        figure: String::new(),
    }
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
            let (items, mut met) = engine.line(m, exprs, env, ctx)?;
            if let Some(place) = m.asks_unresolved.get(&ask.id) {
                unresolved_word(&items, place)?;
            }
            if !items.is_empty() && items.iter().all(|i| matches!(i, Item::Refused(_))) {
                // a refused line rests on what refused it, which took the ask's pins
                let taken = refused_pins(pinned, &met);
                met.merge(&taken);
            }
            let unknown = hole_days(&items.iter().collect::<Vec<_>>());
            Ok(Printed {
                text: line_values(m, &items)?,
                body: Vec::new(),
                bare: false,
                whole: false,
                alone: false,
                met,
                laws: None,
                unknown,
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
                unknown: hole_days(&[&item]),
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
            print_hole(m, w)
        }
        Item::Value(Value::List(items)) if computed_entry(items) => entries_text(m, items),
        Item::Value(v) => v.typed_text(m).map_err(num_stop),
    }
}

/// The hole an interval that is no window of days prints as, where a print needs its one
/// value: `?"date within <w> not precise enough for a print" (<its read>)` (refcore's
/// `fmt_value` of an `Iv`). L §4.4 l.669, §8.1.
pub(super) fn print_hole(m: &Reality, w: &Window) -> Result<String, Stop> {
    let window = w.question_text(m).map_err(num_stop)?;
    let text = format!("date within {window} not precise enough for a print");
    Ok(crate::hole::Hole::asked(&text, w.figure())
        .carrying(&w.carry)
        .text())
}

/// Whether a list holds an entry computed from a window that is no window of days, in a list
/// it holds too (refcore's `computed_iv`). L §4.4.
pub(super) fn computed_entry(items: &[Value]) -> bool {
    let mut todo: Vec<&Value> = items.iter().collect();
    while let Some(v) = todo.pop() {
        if let Value::Window(w) = v
            && !w.of_days()
        {
            return true;
        }
        if let Value::List(more) = v {
            todo.extend(more);
        }
    }
    false
}

/// A list that holds an entry computed from a window, as a line prints it: an entry is where
/// one value is needed, so such an entry prints as its hole, a list of them likewise, every
/// other entry as it prints in a list (refcore's `fmt_value` of a `Lst`; #221, 221.C2).
/// L §4.4 l.669 ("wherever one value is needed (a print but a date's …)").
///
/// # Errors
///
/// A number error printing an entry.
pub(super) fn entries_text(m: &Reality, items: &[Value]) -> Result<String, Stop> {
    // the lists open, outermost first: each with its entries not yet printed and the texts
    // of those printed, so a list inside a list is a worklist step, never a recursion (R-16)
    let mut open: Vec<(core::slice::Iter<'_, Value>, Vec<String>)> =
        vec![(items.iter(), Vec::new())];
    loop {
        let Some((rest, _)) = open.last_mut() else {
            return Err(invariant("a list printed with no list open"));
        };
        let text = match rest.next() {
            Some(Value::List(more)) if computed_entry(more) => {
                open.push((more.iter(), Vec::new()));
                continue;
            }
            Some(Value::Window(w)) if !w.of_days() => print_hole(m, w)?,
            Some(other) => other.text(m).map_err(num_stop)?,
            None => {
                let Some((_, done)) = open.pop() else {
                    return Err(invariant("a list printed with no list open"));
                };
                let list = format!("[{}]", done.join(", "));
                if open.is_empty() {
                    return Ok(list);
                }
                list
            }
        };
        let Some((_, done)) = open.last_mut() else {
            return Err(invariant("a list printed with no list open"));
        };
        done.push(text);
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
    let mut engine = Engine::new(cut)
        .with_days(days)
        .with_how((seen.memo, seen.lows, seen.forks))
        .for_ask(ask)
        .in_tuple(env);
    engine.booked = core::mem::take(&mut seen.booked);
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
            return Ok(vec![refused_line(&ask.id, &r)]);
        }
        Err(stop) => return Err(stop),
    };
    engine.asks_moved_no_money()?;
    let mut out = Vec::new();
    for member in members {
        let head = env_text(m, &ask.binds, &member.env)?;
        // one side of a diff over two stacks: each tuple's cells under its part (`cross_lines`)
        if let Some(side) = seen.side.as_mut() {
            side.part.clone_from(&head);
            if let Some(hole) = &member.unknown {
                let cells = side.cells.get_or_insert_with(Vec::new);
                cells.push((head.clone(), Err(hole.text())));
                continue;
            }
        }
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
