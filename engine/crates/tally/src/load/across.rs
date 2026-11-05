//! What load sees, folded at every key: a period lying across two keys, a decision's balance of its own day or later, a schedule giving no date.
//! Spec: L §4.6 (a period no longer than the keys that no key's period holds is refused at
//! load where load sees it), §5.14.1 (a decision's plain `bal(x, d)` of its own day or later
//! is `LOAD-CYCLE` where load sees the day), §4.2 (a schedule that gives no date in the run
//! is `LOAD-SCHEDULE` where load sees it), §8.3 (`LOAD-KEY` `<period> lies across keys <a>
//! and <b> of <index>`; item 5, the keys of a declared index; item 9, what load sees).
//! Twin: engine/tools/refcore/check.py (`period_conv`, its `period_cands`); check3.py
//! (`m_check_period_keys`, `m_seen_day`, `FOLD_FNS`, `m_check_seen_days`); moves.py
//! (`m_check_schedule`, at load by issue #72); evaluate.py (`c_schedule`, the day before
//! its roll).
//! Invariants: an expression is folded only where load sees it, from literals and the
//! reader's own keys of declared indexes through built-ins that read no fact; a fold the
//! evaluator refuses is left to the value, as refcore leaves it.

use super::check::{DayRead, Files, invariant};
use super::steps::refuse;
use super::types::text;
use crate::base::{Code, Load};
use crate::eval::{Ctx, Engine, Env, int_of};
use crate::model::{
    BindInfo, Domain, IndexForm, IndexInfo, KeyVal, Node, NodeKind, Reality, Value,
};
use crate::syntax::{Expr, Lit, Stop};
use crate::time::{Day, Period};
use std::collections::BTreeSet;

/// The built-ins a seen day is folded through: they read no fact (check3.py's `FOLD_FNS`).
const FOLD_FNS: [&str; 9] = [
    "end_of",
    "start_of",
    "day_of",
    "add_days",
    "add_months",
    "add_years",
    "month_of",
    "year_of",
    "quarter_of",
];

/// A period read at a key of a time index, folded once the walk is done (refcore's
/// `period_cands` entry).
#[derive(Debug, Clone)]
pub(super) struct PeriodRead {
    /// The time index.
    pub(super) index: String,
    /// The key as written.
    pub(super) key: Expr,
    /// The item that reads it.
    pub(super) reader: Node,
    /// The file and line it is written on.
    pub(super) at: (String, u32),
}

/// `LOAD-KEY` for each period load sees that lies across two keys of its index, folded at
/// every key of the reader's bindings in their order (refcore's `m_check_period_keys`).
/// L §4.6, §8.3.
///
/// The fold reads only the bindings the period names (`seen_day` admits no other name), so
/// it is folded at each choice of their keys alone, in the bindings' order: the product's
/// first choice that refuses holds every other binding at its first key, and the refusal
/// names the period and the index's keys, never the reader's. A reader over several indexes
/// has millions of choices, a period one or two of its bindings.
///
/// # Errors
///
/// The first such period's `LOAD-KEY`; an invariant or `NotBuilt` the fold meets.
pub(super) fn check_period_keys(m: &Reality, reads: &[PeriodRead]) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for read in reads {
        let Some(index) = m.indexes.get(&read.index) else {
            continue;
        };
        let Some(keys) = period_keys(index) else {
            continue;
        };
        let Some(binds) = reader_binds(m, &read.reader) else {
            continue;
        };
        if !seen_day(m, &read.key, &binds) {
            continue;
        }
        let Some(domains) = bind_keys(m, &binds) else {
            continue;
        };
        if first_choice(&domains).is_none() {
            continue;
        }
        let names = day_names(&read.key);
        let mut named = Vec::new();
        let mut named_keys = Vec::new();
        for (bind, bound) in binds.into_iter().zip(domains) {
            if names.contains(bind.var.as_str()) {
                named.push(bind);
                named_keys.push(bound);
            }
        }
        let mut at = first_choice(&named_keys);
        while let Some(counters) = at {
            let (_, env) = choice(&named, &named_keys, &counters)?;
            if let Some(period) = fold(&mut engine, m, &read.key, &env)? {
                across(&keys, period, read)?;
            }
            at = next_choice(&counters, &named_keys)?;
        }
    }
    Ok(())
}

/// The names a day or period load sees reads, its bound variables, on `seen_day`'s walk.
/// L §8.3 item 9.
fn day_names(e: &Expr) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    let mut stack = vec![e];
    while let Some(node) = stack.pop() {
        if let Expr::Name(name) = node {
            names.insert(name.as_str());
        } else if let Expr::Call(call) = node {
            stack.extend(call.args.iter());
        } else if let Expr::Sum(first, rest) = node {
            stack.push(first);
            stack.extend(rest.iter().map(|(_, term)| term));
        } else if let Expr::Neg(inner) = node {
            stack.push(inner);
        }
    }
    names
}

/// `LOAD-CYCLE` for a decision's plain `bal(x, d)` whose day load sees: folded at every key
/// of the decision's bindings, `d` is the decision's own day without `start`, or a later
/// one (refcore's `m_check_seen_days`). L §5.14.1, §8.3 item 9.
///
/// # Errors
///
/// The first such read's `LOAD-CYCLE`, at the move's line; an invariant or `NotBuilt` the
/// fold meets.
pub(super) fn check_seen_days(m: &Reality, reads: &[DayRead], files: &Files) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for read in reads {
        let Some(mv) = m.moves.get(&read.id) else {
            continue;
        };
        let on = &mv.decl.on;
        if !seen_day(m, on, &mv.binds) || !seen_day(m, &read.day, &mv.binds) {
            continue;
        }
        let Some(domains) = index_keys(m, &mv.binds) else {
            continue;
        };
        let mut at = first_choice(&domains);
        while let Some(counters) = at {
            let (keys, env) = choice(&mv.binds, &domains, &counters)?;
            at = next_choice(&counters, &domains)?;
            let (Some(day), Some(d)) = (
                fold_day(&mut engine, m, on, &env)?,
                fold_day(&mut engine, m, &read.day, &env)?,
            ) else {
                continue;
            };
            if d < day || (d == day && read.start) {
                continue;
            }
            let shown: Vec<String> = keys.iter().map(KeyVal::text).collect();
            let at_keys = if shown.is_empty() {
                String::new()
            } else {
                format!("[{}]", shown.join(", "))
            };
            let start = if read.start { " start" } else { "" };
            let when = if d == day {
                "its own day"
            } else {
                "a later day"
            };
            let message = format!(
                "{}{at_keys} on {} reads bal({}, {}{start}) = {}, {when}: its own result or its future (write the projection)",
                read.id,
                day.text(),
                text(&read.place),
                text(&read.day),
                d.text()
            );
            return Err(refuse(
                Load::Cycle,
                message,
                files.get("move", &read.id)?,
                mv.decl.line,
            ));
        }
    }
    Ok(())
}

/// `LOAD-SCHEDULE` for a plan whose day is `schedule(p, e, …)` with `p` and `e` seen, when at
/// no key of its bindings does day `e` of `p` exist. Load refuses it whatever the asks
/// (issue #72); refcore meets it where an ask reaches the plan's instances (moves.py's
/// `m_check_schedule`). A hole or a refused fold might give a day, so it refuses nothing.
/// L §4.2, §8.3.
///
/// # Errors
///
/// The first such plan's `LOAD-SCHEDULE`, at its line; an invariant or `NotBuilt` the fold
/// meets.
pub(super) fn check_schedules(m: &Reality, files: &Files) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for (id, mv) in m.moves.entries() {
        let on = bare(&mv.decl.on);
        let Expr::Call(call) = on else {
            continue;
        };
        let [p, e, ..] = call.args.as_slice() else {
            continue;
        };
        let seen =
            call.name == "schedule" && seen_day(m, p, &mv.binds) && seen_day(m, e, &mv.binds);
        let unfiltered = mv.binds.iter().all(|bind| bind.node.filter.is_none());
        if mv.decision || !seen || !unfiltered {
            continue;
        }
        let Some(domains) = bind_keys(m, &mv.binds) else {
            continue;
        };
        let mut at = first_choice(&domains);
        if at.is_none() {
            continue;
        }
        let mut dated = false;
        while let Some(counters) = at {
            let (_, env) = choice(&mv.binds, &domains, &counters)?;
            if scheduled_day(&mut engine, m, p, e, &env)? != Some(false) {
                dated = true;
                break;
            }
            at = next_choice(&counters, &domains)?;
        }
        if !dated {
            let message = format!(
                "schedule {} (the day of {id}) gives no date in the run",
                text(on)
            );
            return Err(refuse(
                Load::Schedule,
                message,
                files.get("move", id)?,
                mv.decl.line,
            ));
        }
    }
    Ok(())
}

/// Whether day `e` of period `p` exists at one choice of keys: `Some(false)` where it
/// doesn't (31 February, day 0), `None` where the fold gives no period and int (a hole, a
/// refusal), which might give one (refcore's `c_schedule`, before its roll). L §4.2.
fn scheduled_day(
    engine: &mut Engine,
    m: &Reality,
    p: &Expr,
    e: &Expr,
    env: &Env,
) -> Result<Option<bool>, Stop> {
    let (Some(pv), Some(ev)) = (
        fold_value(engine, m, p, env)?,
        fold_value(engine, m, e, env)?,
    ) else {
        return Ok(None);
    };
    let (Value::Period(period) | Value::Key(KeyVal::Period(period))) = pv else {
        return Ok(None);
    };
    let k = match int_of(m, &ev) {
        Ok(k) => k,
        Err(Stop::Refused(r)) if r.code() != Code::EngineInvariant => return Ok(None),
        Err(stop) => return Err(stop),
    };
    let start = period.start();
    let Ok(k) = u8::try_from(k) else {
        return Ok(Some(false));
    };
    let day = Day::from_ymd(start.year(), start.month(), k);
    Ok(Some(day.is_some_and(|day| period.contains(day))))
}

/// An expression with its outer brackets taken off (refcore's `strip`).
fn bare(e: &Expr) -> &Expr {
    let mut e = e;
    while let Expr::Paren(inner) = e {
        e = inner;
    }
    e
}

/// The keys of an index that isn't open, when each is a period (refcore's `isinstance(k,
/// Period)` over `ix.keys`). L §4.1.
fn period_keys(index: &IndexInfo) -> Option<Vec<Period>> {
    if is_open(index) || index.keys.is_empty() {
        return None;
    }
    let mut keys = Vec::new();
    for key in &index.keys {
        let KeyVal::Period(period) = key else {
            return None;
        };
        keys.push(*period);
    }
    Some(keys)
}

/// Whether an index's keys come from rows, which load doesn't see: an open index, or the
/// built-in `bank_account`, the one built-in index of no time and no integers (refcore's
/// `form == 'open'`). L §4.1, §8.3 item 5.
fn is_open(index: &IndexInfo) -> bool {
    index.form == IndexForm::Open
        || (index.form == IndexForm::Builtin && index.period.is_none() && !index.is_int)
}

/// The reader's own bindings: an ask has none, a def's and a move's are declared; another
/// reader isn't folded (refcore's `holder` cases).
fn reader_binds(m: &Reality, reader: &Node) -> Option<Vec<BindInfo>> {
    match reader.kind {
        NodeKind::Ask => Some(Vec::new()),
        NodeKind::Def => m.defs.get(&reader.id).map(|d| d.binds.clone()),
        NodeKind::Move | NodeKind::MoveOn => m.moves.get(&reader.id).map(|mv| mv.binds.clone()),
        NodeKind::Input
        | NodeKind::Band
        | NodeKind::Place
        | NodeKind::Price
        | NodeKind::Balances
        | NodeKind::Parts
        | NodeKind::Axis
        | NodeKind::Fill => None,
    }
}

/// Whether load sees the day or period `e`: literals, the reader's own keys of a declared
/// index, built-ins that read no fact, and `+` and `-` over these (refcore's `m_seen_day`).
/// L §8.3 item 9.
pub(super) fn seen_day(m: &Reality, e: &Expr, binds: &[BindInfo]) -> bool {
    let mut stack = vec![e];
    while let Some(node) = stack.pop() {
        if let Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) | Lit::Num(_)) = node {
            continue;
        }
        if let Expr::Name(name) = node {
            if !declared_key(m, binds, name) {
                return false;
            }
            continue;
        }
        if let Expr::Call(call) = node
            && FOLD_FNS.contains(&call.name.as_str())
            && call.prefix.is_empty()
            && call.band_key.is_none()
        {
            stack.extend(call.args.iter());
            continue;
        }
        if let Expr::Sum(first, rest) = node {
            stack.push(first);
            stack.extend(rest.iter().map(|(_, term)| term));
            continue;
        }
        let Expr::Neg(inner) = node else {
            return false;
        };
        stack.push(inner);
    }
    true
}

/// Whether `name` is one of the reader's bound variables over a declared index that isn't
/// open. L §8.3 items 5 and 9.
fn declared_key(m: &Reality, binds: &[BindInfo], name: &str) -> bool {
    binds
        .iter()
        .any(|bind| bind.var == name && declared_keys(m, bind).is_some())
}

/// The keys of the index a binding ranges over, when it is declared and not open.
fn declared_keys(m: &Reality, bind: &BindInfo) -> Option<Vec<KeyVal>> {
    let Domain::Index(ix) = &bind.domain else {
        return None;
    };
    m.indexes
        .get(ix)
        .filter(|index| !is_open(index))
        .map(|index| index.keys.clone())
}

/// The keys of each binding's index, in the bindings' order, or `None` when one ranges over
/// what load doesn't see.
fn bind_keys(m: &Reality, binds: &[BindInfo]) -> Option<Vec<Vec<KeyVal>>> {
    binds.iter().map(|bind| declared_keys(m, bind)).collect()
}

/// The keys of each binding's index, open or not, in the bindings' order, or `None` when one
/// ranges over no index (refcore's `m_check_seen_days`, its `doms`).
fn index_keys(m: &Reality, binds: &[BindInfo]) -> Option<Vec<Vec<KeyVal>>> {
    let mut out = Vec::new();
    for bind in binds {
        let Domain::Index(ix) = &bind.domain else {
            return None;
        };
        out.push(m.indexes.get(ix)?.keys.clone());
    }
    Some(out)
}

/// The first choice of one key per binding, a counter per binding at its first key, or
/// `None` when a binding ranges over no key, so there is no choice (Python's
/// `itertools.product`). The product is walked one choice at a time, never held whole: a
/// reader over several indexes has millions of choices.
fn first_choice(domains: &[Vec<KeyVal>]) -> Option<Vec<usize>> {
    if domains.iter().any(Vec::is_empty) {
        return None;
    }
    Some(vec![0; domains.len()])
}

/// The choice after `at`, the last binding's key turning fastest (Python's
/// `itertools.product`), or `None` after the last.
fn next_choice(at: &[usize], domains: &[Vec<KeyVal>]) -> Result<Option<Vec<usize>>, Stop> {
    let mut next = at.to_vec();
    for (slot, keys) in next.iter_mut().zip(domains).rev() {
        let turned = slot
            .checked_add(1)
            .ok_or_else(|| invariant("a key counter past usize".to_owned()))?;
        if turned < keys.len() {
            *slot = turned;
            return Ok(Some(next));
        }
        *slot = 0;
    }
    Ok(None)
}

/// The keys of choice `at`, one per binding, and the scope they bind.
fn choice(
    binds: &[BindInfo],
    domains: &[Vec<KeyVal>],
    at: &[usize],
) -> Result<(Vec<KeyVal>, Env), Stop> {
    let mut keys = Vec::new();
    let mut env = Env::empty();
    for ((bind, domain), index) in binds.iter().zip(domains).zip(at) {
        let key = domain
            .get(*index)
            .ok_or_else(|| invariant("a key counter past its index".to_owned()))?;
        env = env.with(&bind.var, Value::of_key(key), Some(bind.domain.clone()));
        keys.push(key.clone());
    }
    Ok((keys, env))
}

/// An expression's value at one choice of keys; a refusal leaves it to the value at run
/// (refcore's `except TallyError: continue`). E §3.2.
fn fold_value(
    engine: &mut Engine,
    m: &Reality,
    e: &Expr,
    env: &Env,
) -> Result<Option<Value>, Stop> {
    match engine.ev(m, e, env, Ctx::new(None, m.on)) {
        Ok(value) => Ok(Some(value)),
        Err(Stop::Refused(r)) if r.code() != Code::EngineInvariant => Ok(None),
        Err(stop) => Err(stop),
    }
}

/// The key's value at one choice of keys, when it is a period. E §3.2.
fn fold(engine: &mut Engine, m: &Reality, key: &Expr, env: &Env) -> Result<Option<Period>, Stop> {
    let value = fold_value(engine, m, key, env)?;
    Ok(if let Some(Value::Period(period)) = value {
        Some(period)
    } else {
        None
    })
}

/// The day's value at one choice of keys, when it is a date. E §3.2.
fn fold_day(engine: &mut Engine, m: &Reality, day: &Expr, env: &Env) -> Result<Option<Day>, Stop> {
    let value = fold_value(engine, m, day, env)?;
    Ok(if let Some(Value::Day(d)) = value {
        Some(d)
    } else {
        None
    })
}

/// `LOAD-KEY` when no key holds `period`, two keys meet it, and some key is no shorter
/// (refcore's `m_check_period_keys`, its test). L §4.6.
fn across(keys: &[Period], period: Period, read: &PeriodRead) -> Result<(), Stop> {
    let held = keys
        .iter()
        .any(|k| k.start() <= period.start() && period.end() <= k.end());
    if held {
        return Ok(());
    }
    let over: Vec<&Period> = keys
        .iter()
        .filter(|k| k.start() < period.end() && period.start() < k.end())
        .collect();
    let [first, second, ..] = over.as_slice() else {
        return Ok(());
    };
    let long = length(period)?;
    let mut shorter = true;
    for key in keys {
        shorter = shorter && length(*key)? < long;
    }
    if shorter {
        return Ok(());
    }
    let (file, line) = &read.at;
    Err(refuse(
        Load::Key,
        format!(
            "{} lies across keys {} and {} of {}",
            period.label(),
            first.label(),
            second.label(),
            read.index
        ),
        file,
        *line,
    ))
}

/// A period's days, `end - start`.
fn length(period: Period) -> Result<i64, Stop> {
    period
        .start()
        .days_to(period.end())
        .ok_or_else(|| invariant("a period's days past an i64".to_owned()))
}
