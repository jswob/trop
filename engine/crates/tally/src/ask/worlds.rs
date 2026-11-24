//! Scenarios' asks, each in the world it names; refused scenarios, warnings, `check`'s lines.
//! Spec: L §6.4 (asking a scenario: `in s`, `diff … in s against b`, a `{scenario s}` pin; two
//! scenarios over one base each declare their own asks), §5.18, §8 (a scenario's refusal
//! refuses only that scenario and those over it), §8.5 (an item is unread only where every
//! world that declares it leaves it unread); E §11 (`REQ-ASK`, tally 2.1.3; `REQ-CLOCK`);
//! OPERATIONS §2.3 (`check`).
//! Twin: engine/tools/refcore/cli.py (`run_worlds`, from its `REQ-ASK` check on, its
//! `unread_everywhere` and `asif_worlds`; `check_worlds`) and ask.py (`Asker.load_check`).
//! Invariants: every id the run names is checked before any ask is answered, then every turn's
//! literal cut against `on` in the run's order (`REQ-CLOCK`), a member's ask as it resolves, so
//! no member runs alone after a turn that refuses the request; refused scenarios print first,
//! by their files' full paths, in `run` as in `check` (R20); `check`'s asks refused alone go
//! by id, then file, then line; the asks go by id, then by the full path
//! of the file that states each, then its line, the world breaking no tie (P49); an ask
//! that names scenarios is answered on the reality of the deepest, whose stack holds the
//! others, each world under it seen with the layers over it hidden; the base's diff whose two
//! worlds are on two stacks is answered side by side, each side in the reality holding its
//! stack (refcore's `cross_stack_diff`); its axes over both worlds were counted at load
//! (`LOAD-FORKS`, `load/worlds.rs`); any other ask whose worlds are on two stacks stops the run
//! `NotBuilt` (refcore's cross evaluation).

use super::answer::{Answered, Turn, answer_turns, no_ask, no_turn, reality_turns, req_clock};
use super::members;
use super::register::warning_lines;
use crate::Asks;
use crate::base::{Code, Refusal, Warn};
use crate::eval::How;
use crate::load::{Loaded, Worlds};
use crate::model::Reality;
use crate::oracle::Days;
use crate::syntax::{Ask, AskBody, PinForm, Stop};
use crate::walk::invariant;
use std::collections::{BTreeMap, BTreeSet};

/// The lines a run of a reality with scenarios prints (refcore's `run_worlds`, its asks): each
/// refused scenario's line, then each ask's lines in the world it names, a joint model's
/// member's answered by the member alone, then the warnings. L §6.4, §6.1, §5.18, §8.5.
///
/// # Errors
///
/// `REQ-ASK` for the first id the run names that no world's ask has, then `REQ-CLOCK` for the
/// first ask whose literal cut is after `on` (E §11); `NotBuilt` for an ask evaluated in two
/// worlds, a load refusal met at run, and a construct not built yet.
pub fn answer_worlds(w: &Worlds, asks: &Asks, days: Days, how: How) -> Result<Answered, Stop> {
    let Parts {
        refused,
        turns: asked,
        warnings,
    } = Parts::worlds(w, asks)?;
    let mut turns: Vec<Turn> = refused
        .into_iter()
        .map(|(id, r)| Turn::Scenario(scenario_line(&id, &r)))
        .collect();
    let realities = world_realities(w);
    turns.extend(members::resolve(&realities, asked, (days, how))?);
    answer_turns(&realities, &turns, warnings, days, how)
}

/// What a run answers its asks from, beside its realities, before any is answered (refcore's
/// `run_worlds` up to its first ask, `Asker.run`'s): its refused scenarios, in `check`'s
/// order; its turns; its warnings, load's. L §6.4, §8, §8.5; E §11 (`REQ-ASK`).
pub(super) struct Parts {
    /// Each refused scenario's id and refusal, by its file's full path (R20).
    pub(super) refused: Vec<(String, Refusal)>,
    /// The asks' turns, in the run's order.
    pub(super) turns: Vec<Turn>,
    /// Load's warnings, an item unread only where every world leaves it unread.
    pub(super) warnings: Vec<(Warn, String)>,
}

impl Parts {
    /// The parts of a loaded reality, one world or several.
    ///
    /// # Errors
    ///
    /// `REQ-ASK` for the first id the run names that no world's ask has (E §11).
    pub(super) fn of(loaded: &Loaded, asks: &Asks) -> Result<Parts, Stop> {
        match loaded {
            Loaded::One(m) => Ok(Parts {
                refused: Vec::new(),
                turns: reality_turns(m, asks)?,
                warnings: m.warnings.clone(),
            }),
            Loaded::Worlds(w) => Parts::worlds(w, asks),
        }
    }

    /// The parts of a reality with scenarios.
    ///
    /// # Errors
    ///
    /// `REQ-ASK` for the first id the run names that no world's ask has (E §11).
    fn worlds(w: &Worlds, asks: &Asks) -> Result<Parts, Stop> {
        Ok(Parts {
            refused: refused_in_order(w)?
                .into_iter()
                .map(|(id, r)| (id.clone(), r.clone()))
                .collect(),
            turns: ask_turns(w, asks)?,
            warnings: merged_warnings(w),
        })
    }
}

/// A loaded reality's realities as a run's turns count them: the base's first, then each
/// loaded scenario's by id (refcore's `run_worlds`, its worlds). L §6.4.
pub(super) fn realities(loaded: &Loaded) -> Vec<&Reality> {
    match loaded {
        Loaded::One(m) => vec![m],
        Loaded::Worlds(w) => world_realities(w),
    }
}

/// [`realities`] of a reality with scenarios. L §6.4.
fn world_realities(w: &Worlds) -> Vec<&Reality> {
    core::iter::once(&w.base).chain(w.loaded.values()).collect()
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
    turns
        .iter()
        .try_for_each(|turn| req_clock_turn(realities, turn))
}

/// [`req_clocks`] of one turn: each side of a diff on two stacks in order. E §11.
pub(super) fn req_clock_turn(realities: &[&Reality], turn: &Turn) -> Result<(), Stop> {
    match turn {
        Turn::Ask(world, at) => req_clock_at(realities, None, (*world, *at)),
        Turn::Cross([(w0, a0, _), (w1, a1, _)]) => req_clock_at(realities, None, (*w0, *a0))
            .and_then(|()| req_clock_at(realities, None, (*w1, *a1))),
        Turn::OfMember(aliases, at) => req_clock_at(realities, Some(aliases), (0, *at)),
        Turn::Stopped(Stop::Refused(refusal)) if matches!(refusal.code(), Code::Req(_)) => {
            Err(Stop::Refused(refusal.clone()))
        }
        Turn::Line(_)
        | Turn::Scenario(_)
        | Turn::Lifted(..)
        | Turn::Member(_)
        | Turn::Stopped(_) => Ok(()),
    }
}

/// `REQ-CLOCK` for the ask at `at` of the reality at `world` among `realities`, or of the
/// member alone these aliases name from the base down. E §11.
fn req_clock_at(
    realities: &[&Reality],
    member: Option<&[String]>,
    (world, at): (usize, usize),
) -> Result<(), Stop> {
    let mut m: &Reality = realities.get(world).ok_or_else(no_turn)?;
    if let Some(aliases) = member {
        m = &m.member_alone(aliases).ok_or_else(no_turn)?.reality;
    }
    req_clock(m, m.asks.get(at).ok_or_else(no_turn)?)
}

/// The turns of a run of a reality with scenarios, after its refused scenarios' lines: each
/// ask in the world it names, by id, then the full path of the file that states it, then its
/// line (P49), or in the order the run names them, a member's ask (`a.q`) as `Turn::Member`
/// (refcore's `run_worlds`, its `order`). L §6.4, §6.1; OPERATIONS §2.1, §2.3.
///
/// # Errors
///
/// `REQ-ASK` for the first id the run names that no world's ask has (E §11).
fn ask_turns(w: &Worlds, asks: &Asks) -> Result<Vec<Turn>, Stop> {
    if let Asks::Named(ids) = asks
        && let Some(id) = ids.iter().find(|id| !knows(w, id))
    {
        return Err(Stop::Refused(no_ask(id)));
    }
    let model = w.base.header.id.as_str();
    // the base's asks from the base, a scenario's from its own reality, by (id, the full path
    // of its file, its line), the world last (P49)
    let mut keyed = Vec::new();
    for (at, ask) in w.base.asks.iter().enumerate() {
        let Some(file) = w.base.graph.ask_files.get(&ask.id) else {
            return Err(invariant("an ask with no file"));
        };
        keyed.push(((ask.id.as_str(), file, ask.line, ""), (Some(at), ask)));
    }
    for (id, reality) in &w.loaded {
        for ask in &reality.asks {
            if reality.scenarios.of(&ask.id) != Some(id.as_str()) {
                continue;
            }
            let Some(file) = reality.graph.ask_files.get(&ask.id) else {
                return Err(invariant("an ask with no file"));
            };
            keyed.push(((ask.id.as_str(), file, ask.line, id.as_str()), (None, ask)));
        }
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    let mut turns = Vec::new();
    match asks {
        Asks::Every => {
            for ((_, _, _, world), (at, ask)) in &keyed {
                turns.push(ask_turn(w, (*at, world), (ask, model))?);
            }
        }
        Asks::Named(ids) => {
            for id in ids {
                if member_ask(w, id) {
                    // answered by the member alone, where the run names it (L §6.1)
                    turns.push(Turn::Member(id.clone()));
                    continue;
                }
                let of_id = keyed.iter().filter(|((a, ..), _)| *a == id.as_str());
                for ((_, _, _, world), (at, ask)) in of_id {
                    turns.push(ask_turn(w, (*at, world), (ask, model))?);
                }
            }
        }
    }
    Ok(turns)
}

/// The turn of an ask in the world `own` (`""` for the base's): an ask refused alone, the
/// base's or a loaded scenario's file's own, prints its line; an ask in a refused scenario's
/// world prints that refusal; any other is answered in the deepest world it names (refcore's
/// `run_worlds`, its loop over `order`). L §6.4, §8; OPERATIONS §2.1 (an ask refused alone).
///
/// # Errors
///
/// As [`world_turn`].
fn ask_turn(
    w: &Worlds,
    (at, own): (Option<usize>, &str),
    (ask, model): (&Ask, &str),
) -> Result<Turn, Stop> {
    if let Some(base_at) = at.filter(|_| w.base.asks_alone.contains_key(&ask.id)) {
        return Ok(Turn::Ask(0, base_at));
    }
    // a scenario's file's own ask refused alone prints its line, as `check` does
    // (OPERATIONS §2.1, §2.3; the gate review's B-4)
    let alone = w
        .loaded
        .get(own)
        .filter(|reality| reality.scenarios.of(&ask.id) == Some(own))
        .and_then(|reality| reality.asks_alone.get(&ask.id));
    if let Some(refusal) = alone {
        return Ok(Turn::Line(refused_line(&ask.id, refusal)));
    }
    let named = worlds_named(w, ask, own, model);
    if let Some((_, refusal)) = w.refused.iter().find(|(id, _)| named.contains(*id)) {
        return Ok(Turn::Line(refused_line(&ask.id, refusal)));
    }
    world_turn(w, &named, (at, own), (ask, model))
}

/// The turn of an ask: the base's ask at `at` in the base when it names no world besides the
/// model's, else the ask `id` in the reality of the deepest world it names, the one whose stack
/// holds every other, as that reality holds it (refcore's `run_worlds`: `need`, `cands`, `ra`).
/// L §6.4, §5.18.
///
/// # Errors
///
/// `NotBuilt` for an ask other than the base's diff whose worlds are on two stacks of
/// scenarios (refcore's `make_cross`).
fn world_turn(
    w: &Worlds,
    named: &BTreeSet<String>,
    (at, own): (Option<usize>, &str),
    (ask, model): (&Ask, &str),
) -> Result<Turn, Stop> {
    let id = ask.id.as_str();
    if named.is_empty() {
        let base_at = at.ok_or_else(|| invariant("a scenario's ask that names no world"))?;
        return Ok(Turn::Ask(0, base_at));
    }
    let deepest = named.iter().find(|world| stack_holds(w, world, named));
    let Some(deepest) = deepest else {
        if let (AskBody::Diff(diff), true) = (&ask.body, own.is_empty()) {
            return cross_turn(w, id, [&diff.world, &diff.against], model);
        }
        return Err(Stop::NotBuilt(
            "an ask whose worlds are on two stacks of scenarios (refcore's cross evaluation)",
        ));
    };
    let found = w
        .loaded
        .iter()
        .enumerate()
        .find(|(_, (world, _))| *world == deepest);
    let Some((place, (_, reality))) = found else {
        return Err(invariant("the deepest world an ask names isn't loaded"));
    };
    let ask_at = ask_in(reality, id, own)
        .ok_or_else(|| invariant("a scenario's reality holds no ask its world names"))?;
    let place = place
        .checked_add(1)
        .ok_or_else(|| invariant("a count of worlds"))?;
    Ok(Turn::Ask(place, ask_at))
}

/// The turn of the base's diff `id` whose two worlds `sides` sit on two stacks: each side in
/// the reality of its own world, else of the first scenario by id whose stack holds it, the
/// model's in the base's, each as that reality holds the ask (refcore's `run_worlds`, its
/// `cross_stack_diff`). L §6.4, §5.18.
///
/// # Errors
///
/// `NotBuilt` where the reality holding a side's stack holds no ask `id` (refcore's own limit
/// there). The axes both sides reach were counted at load (`load/worlds.rs`).
fn cross_turn(w: &Worlds, id: &str, sides: [&String; 2], model: &str) -> Result<Turn, Stop> {
    let mut out = Vec::with_capacity(sides.len());
    for world in sides {
        let (place, reality) = if world == model {
            (0, &w.base)
        } else {
            let own = w.loaded.iter().position(|(s, _)| s == world);
            let holding = w
                .loaded
                .values()
                .position(|m| m.scenarios.over.contains_key(world));
            let at = own
                .or(holding)
                .ok_or_else(|| invariant("a diff's world no loaded stack holds"))?;
            let reality = w
                .loaded
                .values()
                .nth(at)
                .ok_or_else(|| invariant("a diff's side past the realities"))?;
            let place = at
                .checked_add(1)
                .ok_or_else(|| invariant("a count of worlds"))?;
            (place, reality)
        };
        let Some(at) = reality.asks.iter().position(|ask| ask.id == id) else {
            return Err(Stop::NotBuilt(
                "a diff whose worlds sit on different stacks, asked where one is not loaded",
            ));
        };
        out.push((place, at, world.clone()));
    }
    let mut out = out.into_iter();
    let (Some(ours), Some(theirs), None) = (out.next(), out.next(), out.next()) else {
        return Err(invariant("a diff with other than two sides"));
    };
    Ok(Turn::Cross([ours, theirs]))
}

/// Whether world `world` is a loaded scenario whose stack holds every world of `named`
/// (refcore's `cands`: need ⊆ chain(w)). L §6.4.
fn stack_holds(w: &Worlds, world: &str, named: &BTreeSet<String>) -> bool {
    let Some(reality) = w.loaded.get(world) else {
        return false;
    };
    named.iter().all(|n| reality.scenarios.over.contains_key(n))
}

/// The place of ask `id` in `reality` as the file of world `own` states it, a base's for none,
/// else the first ask of that id (refcore's `ra`). L §6.4.
fn ask_in(reality: &Reality, id: &str, own: &str) -> Option<usize> {
    let file = (!own.is_empty()).then_some(own);
    let stated = |ask: &Ask| ask.id == id && reality.scenarios.of(&ask.id) == file;
    if let Some(at) = reality.asks.iter().position(stated) {
        return Some(at);
    }
    reality.asks.iter().position(|ask| ask.id == id)
}

/// The lines a load prints when no ask is answered: a reality's warnings, or each refused
/// scenario's line, in `check`'s order, and the worlds' warnings merged. L §8, §8.5.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a refused scenario with no file.
pub fn report_lines(loaded: &Loaded) -> Result<Vec<String>, Stop> {
    match loaded {
        Loaded::One(m) => Ok(warning_lines(&m.warnings)),
        Loaded::Worlds(w) => {
            let mut lines: Vec<String> = refused_in_order(w)?
                .into_iter()
                .map(|(id, r)| scenario_line(id, r))
                .collect();
            lines.extend(warning_lines(&merged_warnings(w)));
            Ok(lines)
        }
    }
}

/// The refused scenarios in `check`'s order: by the full path of the file that declares each
/// (a scenario declared twice by its first; one whose refusal names no file of its own by its
/// header's), then by id (OPERATIONS §2.1, §2.3; R20 clauses 1 and 2, picks P12, P13).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a refused scenario with no file.
fn refused_in_order(w: &Worlds) -> Result<Vec<(&String, &Refusal)>, Stop> {
    let mut refused = Vec::with_capacity(w.refused.len());
    for (id, refusal) in &w.refused {
        let file = w
            .files
            .get(id)
            .ok_or_else(|| invariant("a refused scenario with no file"))?;
        refused.push((file, id, refusal));
    }
    refused.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    Ok(refused
        .into_iter()
        .map(|(_, id, refusal)| (id, refusal))
        .collect())
}

/// The lines `check` prints for a run's files that load (OPERATIONS §2.3; refcore's
/// `load_check`, `check_worlds`): each refused scenario's line in the full-path order of its
/// file (pick P4), then each ask refused alone, by id, then its file's full path, then its
/// line (pick P5), the base's and each loaded scenario file's own; `ok` alone first when there
/// is neither; then load's warnings, as `run` prints them. An ask in a refused scenario's
/// world is covered by that scenario's line. L §6.4, §8.3, §8.5.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an ask refused alone that its reality places in no file, or a
/// refused scenario with no file.
pub fn check_lines(loaded: &Loaded) -> Result<Vec<String>, Stop> {
    let (mut lines, alone, warnings) = match loaded {
        Loaded::One(m) => (Vec::new(), alone_asks(m, None)?, m.warnings.clone()),
        Loaded::Worlds(w) => {
            let lines = refused_in_order(w)?
                .into_iter()
                .map(|(id, refusal)| scenario_line(id, refusal))
                .collect();
            let mut alone = alone_asks(&w.base, None)?;
            for (id, reality) in &w.loaded {
                alone.extend(alone_asks(reality, Some(id))?);
            }
            (lines, alone, merged_warnings(w))
        }
    };
    lines.extend(alone.into_values());
    if lines.is_empty() {
        lines.push("ok".to_owned());
    }
    lines.extend(warning_lines(&warnings));
    Ok(lines)
}

/// The asks of `m` refused alone that its world's own file states, the base's for none, each
/// keyed by its id, its file's full path and its line, with the line it prints (refcore's
/// `alone_line`). L §6.4, §8.3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for such an ask that `m` places in no file.
fn alone_asks(
    m: &Reality,
    world: Option<&str>,
) -> Result<BTreeMap<(String, String, u32), String>, Stop> {
    let mut out = BTreeMap::new();
    for ask in &m.asks {
        let Some(refusal) = m.asks_alone.get(&ask.id) else {
            continue;
        };
        if m.scenarios.of(&ask.id) != world {
            continue;
        }
        let file = m
            .graph
            .ask_files
            .get(&ask.id)
            .ok_or_else(|| invariant("an ask refused alone with no file"))?;
        let line = refused_line(&ask.id, refusal);
        out.insert((ask.id.clone(), file.clone(), ask.line), line);
    }
    Ok(out)
}

/// A refused scenario's line, `scenario <s>: refused: CODE message`. L §6.4, §8.
fn scenario_line(id: &str, refusal: &Refusal) -> String {
    format!("scenario {id}: refused: {}", refusal.text())
}

/// A refused ask's line, or a tuple's, `<head>: refused: CODE message`: one form for `run` and
/// `check`, whose lines are the same bytes. L §8.3, §8.4.
pub(super) fn refused_line(head: &str, refusal: &Refusal) -> String {
    format!("{head}: refused: {}", refusal.text())
}

/// Whether `id` names an ask of a world: the base's, a loaded scenario's, or one a refused
/// scenario's file declares (refcore's `known` and `knows_ask`). E §11.
fn knows(w: &Worlds, id: &str) -> bool {
    w.refused_asks.contains(id)
        || has_ask(&w.base, id)
        || w.loaded.values().any(|m| has_ask(m, id))
        || member_ask(w, id)
}

/// Whether `id` names a joint model's member's ask, `a.q` (refcore's `knows_ask`). E §11.
fn member_ask(w: &Worlds, id: &str) -> bool {
    w.base.joint.as_ref().is_some_and(|j| j.knows(id))
}

/// Whether the reality has the ask `id`.
fn has_ask(m: &Reality, id: &str) -> bool {
    m.asks.iter().any(|ask| ask.id == id)
}

/// The worlds an ask is evaluated in besides the model's: its file's scenario, its `in`, a
/// diff's two worlds, its `{scenario s}` pins, and each scenario of the run an `as if
/// {scenario s}` names in its line or in a def it reaches, as the base's item graph tells
/// (refcore's `need`, `asif_worlds`). L §6.4, §5.18.
fn worlds_named(w: &Worlds, ask: &Ask, own: &str, model: &str) -> BTreeSet<String> {
    let mut named = BTreeSet::new();
    if !own.is_empty() {
        named.insert(own.to_owned());
    }
    if let Some(world) = &ask.world {
        named.insert(world.clone());
    }
    if let AskBody::Diff(diff) = &ask.body {
        named.insert(diff.world.clone());
        named.insert(diff.against.clone());
    }
    for pin in &ask.pins {
        if let PinForm::Scenario(id) = &pin.form {
            named.insert(id.clone());
        }
    }
    // `x as if {scenario s}` is x in s (L §6.4 Asking)
    for id in crate::load::ask_worlds(&w.base, &w.base.graph, ask) {
        if w.loaded.contains_key(&id) || w.refused.contains_key(&id) {
            named.insert(id);
        }
    }
    named.remove(model);
    named
}

/// The worlds' warnings, the base's first, then each loaded scenario's by id, each once; an
/// unread item kept only where every world that declares it leaves it unread (refcore's
/// `run_worlds`, `unread_everywhere`). L §8.5.
fn merged_warnings(w: &Worlds) -> Vec<(Warn, String)> {
    let worlds: Vec<&Reality> = core::iter::once(&w.base).chain(w.loaded.values()).collect();
    let mut merged: Vec<(Warn, String)> = Vec::new();
    for reality in &worlds {
        for warning in &reality.warnings {
            let (code, message) = warning;
            let kept = *code != Warn::Unread || unread_everywhere(message, &worlds);
            if kept && !merged.contains(warning) {
                merged.push(warning.clone());
            }
        }
    }
    merged
}

/// Whether every world that declares the item a `WARN-UNREAD` message names warns it (refcore's
/// `unread_everywhere`): `<sort> <id> …`, for a source, an input, a def or a place. L §8.5.
fn unread_everywhere(message: &str, worlds: &[&Reality]) -> bool {
    let mut words = message.split(' ');
    let (Some(sort), Some(id)) = (words.next(), words.next()) else {
        return true;
    };
    let declares: fn(&Reality, &str) -> bool = match sort {
        "source" => |m, item| m.sources.contains(item),
        "input" => |m, item| m.inputs.contains(item),
        "def" => |m, item| m.defs.contains(item),
        "place" => |m, item| m.places.contains(item),
        _ => return true,
    };
    worlds
        .iter()
        .all(|m| !declares(m, id) || warns_unread(m, message))
}

/// Whether the reality warns `WARN-UNREAD` with this message.
fn warns_unread(m: &Reality, message: &str) -> bool {
    m.warnings
        .iter()
        .any(|(code, text)| *code == Warn::Unread && text == message)
}

#[cfg(test)]
mod tests {
    use super::check_lines;
    use crate::eval::How;
    use crate::load::{load_run, split_files};
    use crate::{Pin, Source};
    use std::collections::BTreeMap;

    const BASE: &str = "model plain\n  tally 2\n  on 2026-03-01\n  start 2026-01-01\n  end 2027-12-31\n\nsource card \"a card\"\n\nunit PLN: currency minor 0.01\n\ninput amt: money PLN\n\ndef doubled: money PLN = amt + amt at reader\n\nask q: doubled law 2026-03-01\n\nversion card_2026\n  cite card\n  governs always\n  known 2026-01-01\n  amt = 100.00 PLN \"the card\"\n";

    const ALONE: &str = "part alone\n  tally 2\n\nask q_zz: doubled in nowhere_b law 2026-03-01\n\nask q_aa: doubled in nowhere_a law 2026-03-01\n";

    const ZULU: &str = "scenario zulu over plain\n  tally 2\n  from 2027-01-01\n\nversion zulu_rows\n  cite card\n  governs 2027..\n  known 2026-02-01\n  nothing_here = 1 \"no file declares it\"\n";

    fn checked(files: &[(&str, &str)]) -> Vec<String> {
        let sources = files
            .iter()
            .map(|(path, text)| Source {
                path: (*path).to_owned(),
                text: (*text).to_owned(),
            })
            .collect();
        let split = split_files(sources).expect("the files split");
        let loaded = load_run(&split, &BTreeMap::new(), Pin::Canonical, How::FENCES)
            .expect("the base loads");
        check_lines(&loaded).expect("check's lines")
    }

    #[test]
    fn a_clean_base_is_ok() {
        assert_eq!(checked(&[("m.tally", BASE)]), ["ok"], "nothing refused");
    }

    #[test]
    fn refused_scenarios_then_asks_refused_alone_by_id() {
        let lines = checked(&[
            ("m.tally", BASE),
            ("alone.tally", ALONE),
            ("s_zulu.tally", ZULU),
        ]);
        assert_eq!(
            lines,
            [
                "scenario zulu: refused: LOAD-NAME unknown id nothing_here (s_zulu.tally:9)",
                "q_aa: refused: LOAD-SCENARIO ask q_aa names nowhere_a, neither the model nor a \
                 loaded scenario (alone.tally:6)",
                "q_zz: refused: LOAD-SCENARIO ask q_zz names nowhere_b, neither the model nor a \
                 loaded scenario (alone.tally:4)",
            ],
            "the scenario first, then each ask by id, not by line; no `ok`"
        );
    }
}
