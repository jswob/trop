//! The asks of a reality with scenarios: each ask in the world it names, an ask whose
//! scenario was refused printed refused with it, the worlds' warnings merged.
//! Spec: L §6.4 (asking a scenario: `in s`, `diff … in s against b`, a `{scenario s}` pin; two
//! scenarios over one base each declare their own asks), §5.18, §8 (a scenario's refusal
//! refuses only that scenario and those over it), §8.5 (an item is unread only where every
//! world that declares it leaves it unread); E §11 (`REQ-ASK`, tally 2.1.3).
//! Twin: engine/tools/refcore/cli.py (`run_worlds`, from its `REQ-ASK` check on, and its
//! `unread_everywhere`).
//! Invariants: every id the run names is checked before any ask is answered; the refused
//! scenarios print first, by id; the asks go by id, then by world, the base's first; an ask that
//! would be evaluated in a scenario's world stops the run `NotBuilt` (M1's).

use super::answer::{Answered, Turn, answer_turns, no_ask};
use super::register::warning_lines;
use crate::Asks;
use crate::base::Warn;
use crate::eval::Memo;
use crate::load::{Loaded, Worlds};
use crate::model::Reality;
use crate::oracle::Days;
use crate::syntax::{Ask, AskBody, PinForm, Stop};
use std::collections::{BTreeMap, BTreeSet};

/// The lines a run of a reality with scenarios prints (refcore's `run_worlds`, its asks): each
/// refused scenario's line, then each ask's lines in the world it names, then the warnings.
/// L §6.4, §5.18, §8.5.
///
/// # Errors
///
/// `REQ-ASK` for the first id the run names that no world's ask has (E §11); `NotBuilt` for an
/// ask evaluated in a scenario's world, a load refusal met at run, and a construct not built
/// yet.
pub fn answer_worlds(w: &Worlds, asks: &Asks, days: Days, memo: Memo) -> Result<Answered, Stop> {
    if let Asks::Named(ids) = asks
        && let Some(id) = ids.iter().find(|id| !knows(w, id))
    {
        return Err(Stop::Refused(no_ask(id)));
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
            turns.push(Turn::Ask(base_at));
            continue;
        }
        let named = worlds_named(ask, world, model);
        if let Some((_, refusal)) = w.refused.iter().find(|(id, _)| named.contains(*id)) {
            turns.push(Turn::Line(format!(
                "{}: refused: {}",
                ask.id,
                refusal.text()
            )));
            continue;
        }
        let Some(base_at) = at.filter(|_| named.is_empty()) else {
            return Err(Stop::NotBuilt("an ask in a scenario's world"));
        };
        turns.push(Turn::Ask(base_at));
    }
    answer_turns(&w.base, &turns, merged_warnings(w), days, memo)
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
    w.refused_asks.contains(id) || has_ask(&w.base, id) || w.loaded.values().any(|m| has_ask(m, id))
}

/// Whether the reality has the ask `id`.
fn has_ask(m: &Reality, id: &str) -> bool {
    m.asks.iter().any(|ask| ask.id == id)
}

/// The worlds an ask is evaluated in besides the model's: its file's scenario, its `in`, a
/// diff's two worlds and its `{scenario s}` pins (refcore's `need`; an `as if {scenario s}` is
/// evaluated `NotBuilt` until M1). L §6.4, §5.18.
fn worlds_named(ask: &Ask, own: &str, model: &str) -> BTreeSet<String> {
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
