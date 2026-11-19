//! A member's ask at run, `a.q` or `a.b.q`: answered by the member alone, under its prefix.
//! Spec: L §6.1 (a member's asks are asked by name in a run of the joint model, as `a.q`, and
//! answered by the member alone; joining changes no member's answer), §6.4 (a member's
//! scenarios: an ask its scenario files declare, or one of its base's that names one of them,
//! is answered by the member with all its files), §8 (a scenario's refusal refuses that
//! scenario alone); E §3.1 step 1 (a member's files), §11 (`REQ-ASK`, `REQ-CLOCK`);
//! OPERATIONS §2.1 (`--ask a.q`).
//! Twin: engine/tools/refcore/ask.py (`Asker.member_ask`, `Asker.member_alone`,
//! `member_names_world`); refcore refuses a member's ask refused alone as `REQ-ASK`, gives its
//! `REQ-CLOCK` as the ask's line and prefixes no line starting `warning` (#233).
//! Invariants: a member is found by its aliases from the run's model down, a loop over them,
//! never a recursion (R-16); a member's base ask that names none of its scenarios, refused
//! alone or not, is answered in the member's own world, whose walks' warnings print nowhere;
//! any other is a run of the member alone with all its files, made before the run answers its
//! first ask and printed at its turn, a stop it meets stopping the run there and a request
//! refusal refusing the run's request; the member's lines take its aliases, each with its dot,
//! before any warning is added, every line but an indented one; a member that is itself a
//! joint model answers none of its own asks yet (M5c).

use super::answer::{Turn, answer_turns_apart, reality_turns, req_clocks};
use super::register::warning_lines;
use super::worlds::{merged_warnings, refused_line};
use crate::base::{Code, Refusal};
use crate::eval::How;
use crate::load::{Loaded, load_run, split_files};
use crate::model::{MemberAlone, Reality};
use crate::oracle::Days;
use crate::syntax::{Ask, Stop};
use crate::{Asks, Pin};
use std::collections::BTreeMap;

/// The run's turns with each member's ask resolved (refcore's `member_ask`): a base ask of the
/// member that names none of its scenarios, refused alone or not, is answered at its turn in
/// the member's own world; any other is the lines of the member alone, or the stop its run
/// met, a request refusal among them (`REQ-CLOCK`) raised before any ask is answered
/// (`req_clocks`). The ids were checked before, so no run of a member alone precedes a
/// `REQ-ASK`. L §6.1, §6.4; E §11.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an ask the run's check knew that no member holds.
pub(super) fn resolve(
    base: &Reality,
    turns: Vec<Turn>,
    (days, how): (Days, How),
) -> Result<Vec<Turn>, Stop> {
    let mut out = Vec::with_capacity(turns.len());
    for turn in turns {
        let Turn::Member(id) = turn else {
            out.push(turn);
            continue;
        };
        let (aliases, own) = found(base, &id)?;
        let member = base.member_alone(&aliases).ok_or_else(unknown)?;
        let m = &member.reality;
        let at = m.asks.iter().position(|ask| ask.id == own);
        match (at, at.and_then(|at| m.asks.get(at))) {
            (Some(at), Some(ask)) if !names_world(member, ask) => {
                out.push(Turn::OfMember(aliases, at));
            }
            _ => match alone_lines(member, (&id, &aliases, &own), (days, how)) {
                Ok(lines) => out.extend(lines.into_iter().map(Turn::Line)),
                Err(stop) => out.push(Turn::Stopped(stop)),
            },
        }
    }
    Ok(out)
}

/// A member's ask's lines under its prefix, its aliases each with its dot (`a.b.`), but an
/// indented line, which belongs to the line above it: given before any warning is added, so
/// no line is told by its text (refcore's `member_ask`, each level's). L §6.1.
pub(super) fn prefixed(aliases: &[String], lines: Vec<String>) -> Vec<String> {
    let prefix = aliases.join(".");
    lines
        .into_iter()
        .map(|line| {
            if line.starts_with(' ') {
                line
            } else {
                format!("{prefix}.{line}")
            }
        })
        .collect()
}

/// The aliases, from the run's model down, of the member a member's ask `id` names, and the
/// ask's own id there (refcore's `member_ask`, its descent: a member's own ask, else one of its
/// members' when it is a joint model). L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an id no member's asks hold: the run's check knew it.
fn found(base: &Reality, id: &str) -> Result<(Vec<String>, String), Stop> {
    let mut m = base;
    let mut aliases = Vec::new();
    let mut rest = id;
    while let Some((alias, own)) = rest.split_once('.') {
        let joint = m.joint.as_ref().ok_or_else(unknown)?;
        let (Some(asks), Some(member)) = (joint.members.get(alias), joint.alone.get(alias)) else {
            return Err(unknown());
        };
        aliases.push(alias.to_owned());
        if asks.asks.contains(own) {
            return Ok((aliases, own.to_owned()));
        }
        m = &member.reality;
        rest = own;
    }
    Err(unknown())
}

/// Whether a member's base ask names one of the member's scenarios: its `in`, a diff's
/// worlds, a scenario pin, an `as if {scenario s}` in its line or a def it reaches (refcore's
/// `member_names_world`). L §6.4, §5.18.
fn names_world(member: &MemberAlone, ask: &Ask) -> bool {
    let m = &member.reality;
    !member.worlds.is_empty()
        && crate::load::ask_worlds(m, &m.graph, ask)
            .iter()
            .any(|world| member.worlds.contains(world))
}

/// The lines of a member's ask `own`, asked as `id`, answered by the member alone with all its
/// files, its scenarios loaded, as a run of them answers it (refcore's `member_alone`): a
/// refusal of that run as the ask's line, `a.q: refused: …`, but a request refusal
/// (`REQ-CLOCK`), which refuses the run's request (E §11). L §6.1, §6.4, §8.
///
/// # Errors
///
/// `NotBuilt` for a member that is itself a joint model, whose own asks the run doesn't
/// answer yet (M5c); the request refusal; any other stop the run meets but a refusal.
fn alone_lines(
    member: &MemberAlone,
    (id, aliases, own): (&str, &[String], &str),
    (days, how): (Days, How),
) -> Result<Vec<String>, Stop> {
    if member.reality.joint.is_some() {
        return Err(Stop::NotBuilt("a joint model's asks at run"));
    }
    // the run's fault is planted in the run's own worlds, once
    #[cfg(feature = "fault-injection")]
    let how = How { fault: None, ..how };
    let mut files = member.files.clone();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let asks = Asks::Named(vec![own.to_owned()]);
    // a member is no joint model here, so it reads no member and checks no pin
    let run = split_files(files)
        .map_err(Stop::Refused)
        .and_then(|split| load_run(&split, &BTreeMap::new(), Pin::Canonical, how))
        .and_then(|loaded| alone_run(&loaded, &asks, aliases, (days, how)));
    match run {
        Err(Stop::Refused(refusal)) if matches!(refusal.code(), Code::Req(_)) => {
            Err(Stop::Refused(refusal))
        }
        Err(Stop::Refused(refusal)) => Ok(vec![refused_line(id, &refusal)]),
        other => other,
    }
}

/// A run of the member alone, loaded, on `asks`: its refused scenarios' lines, each under the
/// member's prefix, `scenario a.s: refused: …`; its asks' lines under the prefix; then its
/// warnings, as they are (refcore's `member_alone`, over `run_worlds`). L §6.1, §6.4, §8, §8.5.
///
/// # Errors
///
/// `REQ-CLOCK` (E §11), a load refusal met at run, and a construct not built yet.
fn alone_run(
    loaded: &Loaded,
    asks: &Asks,
    aliases: &[String],
    (days, how): (Days, How),
) -> Result<Vec<String>, Stop> {
    let prefix = aliases.join(".");
    let (realities, turns, warnings, mut out) = match loaded {
        Loaded::One(m) => {
            let m: &Reality = m;
            (
                vec![m],
                reality_turns(m, asks)?,
                m.warnings.clone(),
                Vec::new(),
            )
        }
        Loaded::Worlds(w) => {
            let refused = w
                .refused
                .iter()
                .map(|(s, r)| format!("scenario {prefix}.{s}: refused: {}", r.text()))
                .collect();
            let turns = super::worlds::ask_turns(w, asks)?;
            (
                super::worlds::realities(w),
                turns,
                merged_warnings(w),
                refused,
            )
        }
    };
    req_clocks(&realities, &turns)?;
    let (answered, warned) = answer_turns_apart(&realities, &turns, warnings, days, how)?;
    out.extend(prefixed(aliases, answered.lines));
    out.extend(warning_lines(&warned));
    Ok(out)
}

/// A member's ask the run's check knew that no member holds.
fn unknown() -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "a member's ask no member holds".to_owned(),
    ))
}
