//! The asks of a reality with scenarios: each ask in the world it names, an ask whose
//! scenario was refused printed refused with it, the worlds' warnings merged.
//! Spec: L §6.4 (asking a scenario: `in s`, `diff … in s against b`, a `{scenario s}` pin; two
//! scenarios over one base each declare their own asks), §5.18, §8 (a scenario's refusal
//! refuses only that scenario and those over it), §8.5 (an item is unread only where every
//! world that declares it leaves it unread); E §11 (`REQ-ASK`, tally 2.1.3).
//! Twin: engine/tools/refcore/cli.py (`run_worlds`, from its `REQ-ASK` check on, its
//! `unread_everywhere` and `asif_worlds`).
//! Invariants: every id the run names is checked before any ask is answered; the refused
//! scenarios print first, by id; the asks go by id, then by world, the base's first; an ask
//! that names scenarios is answered on the reality of the deepest, whose stack holds the
//! others, each world under it seen with the layers over it hidden; one whose worlds are on two
//! stacks stops the run `NotBuilt` (refcore's cross evaluation).

use super::answer::{Answered, Turn, answer_turns, no_ask};
use super::register::warning_lines;
use crate::Asks;
use crate::base::Warn;
use crate::eval::Memo;
use crate::load::{Loaded, Worlds};
use crate::model::Reality;
use crate::oracle::Days;
use crate::syntax::{Ask, AskBody, PinForm, Stop};
use crate::walk::invariant;
use std::collections::{BTreeMap, BTreeSet};

/// The lines a run of a reality with scenarios prints (refcore's `run_worlds`, its asks): each
/// refused scenario's line, then each ask's lines in the world it names, then the warnings.
/// L §6.4, §5.18, §8.5.
///
/// # Errors
///
/// `REQ-ASK` for the first id the run names that no world's ask has (E §11); `NotBuilt` for an
/// ask evaluated in two worlds, a load refusal met at run, and a construct not built yet.
pub fn answer_worlds(w: &Worlds, asks: &Asks, days: Days, memo: Memo) -> Result<Answered, Stop> {
    if let Asks::Named(ids) = asks
        && let Some(id) = ids.iter().find(|id| !knows(w, id))
    {
        return Err(Stop::Refused(no_ask(id)));
    }
    if let Asks::Named(ids) = asks
        && ids.iter().any(|id| member_ask(w, id))
    {
        return Err(Stop::NotBuilt("a member's ask at run"));
    }
    let model = w.base.header.id.as_str();
    // the base's asks from the base, a scenario's from its own reality, by (id, world)
    let mut keyed: BTreeMap<(&str, &str), (Option<usize>, &Ask)> = BTreeMap::new();
    for (at, ask) in w.base.asks.iter().enumerate() {
        keyed.insert((&ask.id, ""), (Some(at), ask));
    }
    for (id, reality) in &w.loaded {
        for ask in &reality.asks {
            if reality.scenarios.of(&ask.id) == Some(id.as_str()) {
                keyed.insert((&ask.id, id), (None, ask));
            }
        }
    }
    let mut order = Vec::new();
    match asks {
        Asks::Every => order.extend(keyed.iter()),
        Asks::Named(ids) => {
            for id in ids {
                order.extend(keyed.iter().filter(|((ask, _), _)| *ask == id.as_str()));
            }
        }
    }
    let mut turns: Vec<Turn> = w
        .refused
        .iter()
        .map(|(id, r)| Turn::Line(format!("scenario {id}: refused: {}", r.text())))
        .collect();
    for ((_, world), (at, ask)) in order {
        if let Some(base_at) = at.filter(|_| w.base.asks_alone.contains_key(&ask.id)) {
            turns.push(Turn::Ask(0, base_at));
            continue;
        }
        let named = worlds_named(w, ask, world, model);
        if let Some((_, refusal)) = w.refused.iter().find(|(id, _)| named.contains(*id)) {
            turns.push(Turn::Line(format!(
                "{}: refused: {}",
                ask.id,
                refusal.text()
            )));
            continue;
        }
        turns.push(world_turn(w, &named, (*at, world), &ask.id)?);
    }
    // the base's reality first, then each loaded scenario's by id, as `world_turn` counts them
    let realities: Vec<&Reality> = core::iter::once(&w.base).chain(w.loaded.values()).collect();
    answer_turns(&realities, &turns, merged_warnings(w), days, memo)
}

/// The turn of an ask: the base's ask at `at` in the base when it names no world besides the
/// model's, else the ask `id` in the reality of the deepest world it names, the one whose stack
/// holds every other, as that reality holds it (refcore's `run_worlds`: `need`, `cands`, `ra`).
/// L §6.4, §5.18.
///
/// # Errors
///
/// `NotBuilt` for an ask whose worlds are on two stacks of scenarios (refcore's
/// `cross_stack_diff`, `make_cross`).
fn world_turn(
    w: &Worlds,
    named: &BTreeSet<String>,
    (at, own): (Option<usize>, &str),
    id: &str,
) -> Result<Turn, Stop> {
    if named.is_empty() {
        let base_at = at.ok_or_else(|| invariant("a scenario's ask that names no world"))?;
        return Ok(Turn::Ask(0, base_at));
    }
    let deepest = named.iter().find(|world| stack_holds(w, world, named));
    let Some(deepest) = deepest else {
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
/// scenario's line and the worlds' warnings merged. L §8, §8.5.
#[must_use]
pub fn report_lines(loaded: &Loaded) -> Vec<String> {
    match loaded {
        Loaded::One(m) => warning_lines(&m.warnings),
        Loaded::Worlds(w) => {
            let mut lines: Vec<String> = w
                .refused
                .iter()
                .map(|(id, r)| format!("scenario {id}: refused: {}", r.text()))
                .collect();
            lines.extend(warning_lines(&merged_warnings(w)));
            lines
        }
    }
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
