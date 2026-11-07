//! What load sees, folded at every key: periods across keys, decisions' days and projections, moves' days and literal ranges at Tally's limits, schedules, scenarios' moves.
//! Spec: L §4.6 (a period no longer than the keys that no key's period holds is refused at
//! load where load sees it), §5.14.1 (a decision's plain `bal(x, d)` of its own day or later
//! is `LOAD-CYCLE` where load sees the day), §4.2 (a schedule that gives no date in the run
//! is `LOAD-SCHEDULE` where load sees it), §5.16 Decisions and projections (a projection a
//! decision reaches that reads a random amount after its t, with no random clause or query,
//! is `LOAD-RANDOM` where load sees it), §6.4 (a move a scenario declares books nothing
//! before D), §7.6 items 1 and 2 (a move's day past the calendar, a literal range past
//! 1 000 000 keys, `LOAD-LIMIT` where load sees them), §8.3 (`LOAD-KEY` `<period> lies
//! across keys <a> and <b> of <index>`; the forms of `LOAD-RANDOM`, `LOAD-SCENARIO` and
//! `LOAD-LIMIT`; item 5, the keys of a declared index; item 9, what load sees).
//! Twin: engine/tools/refcore/check.py (`period_conv`, its `period_cands`); check3.py
//! (`m_check_period_keys`, `m_seen_day`, `FOLD_FNS`, `m_check_seen_days`,
//! `m_check_seen_day_limits`, `m_check_literal_ranges`,
//! `m_check_proj_seen`, where a move with no `at` dates its amount by its day, issue #75);
//! moves.py (`m_check_schedule`, at load by issue #72); evaluate.py (`c_schedule`, the day
//! before its roll); a scenario's move before its `from` has no twin (issue #75).
//! Invariants: an expression is folded only where load sees it, from literals and the
//! reader's own keys of declared indexes through built-ins that read no fact; a fold the
//! evaluator refuses is left to the value, as refcore leaves it.

use super::check::{DayRead, Files, ProjRead, Scope, Stands, Var, VarDom, invariant, place_target};
use super::cone::all_children;
use super::steps::refuse;
use super::types::text;
use crate::base::{Code, Load, Run};
use crate::eval::{Ctx, Engine, Env, int_of};
use crate::model::{
    BindInfo, Domain, IndexForm, IndexInfo, KeyVal, MoveInfo, NameKind, Node, NodeKind, Reality,
    Target, Ty, Value,
};
use crate::syntax::{
    Ask, AskBody, Bind, Domain as Written, Expr, Key, LawDate, Lit, PinForm, Postfix, Stop,
};
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

/// `LOAD-LIMIT move <m>[<keys>] on <e>: a day passes 9999-12-31` (or `comes before
/// 0001-01-01`) for a move whose day load sees, folded at every key of its bindings, outside
/// the calendar: the end of time read as a day (`on end_of(y)` for the year 9999), or a day
/// the fold computes past it (refcore's `m_check_seen_day_limits`). A day computed from a
/// row's value is refused at its key at run. L §7.6 item 1, §8.3 item 9.
///
/// # Errors
///
/// The first such move's `LOAD-LIMIT`, the moves in the reality's order, at the move's line;
/// an invariant or `NotBuilt` the fold meets.
pub(super) fn check_seen_day_limits(m: &Reality, files: &Files) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for (id, mv) in m.moves.entries() {
        let on = &mv.decl.on;
        if !seen_day(m, on, &mv.binds) {
            continue;
        }
        let Some(domains) = index_keys(m, &mv.binds) else {
            continue;
        };
        let mut at = first_choice(&domains);
        while let Some(counters) = at {
            let (keys, env) = choice(&mv.binds, &domains, &counters)?;
            at = next_choice(&counters, &domains)?;
            let Some(limit) = day_limit(&mut engine, m, on, &env)? else {
                continue;
            };
            let shown: Vec<String> = keys.iter().map(KeyVal::text).collect();
            let message = format!("move {id}{} on {}: {limit}", bracketed(&shown), text(on));
            return Err(refuse(
                Load::Limit,
                message,
                files.get("move", id)?,
                mv.decl.line,
            ));
        }
    }
    Ok(())
}

/// The calendar's limit a move's day meets at one choice of keys: the end of time read as a
/// day, or the `RUN-LIMIT` the fold refuses with; `None` for a day in the calendar, any
/// other value, or another refusal, which the value meets at run. L §7.6 item 1.
fn day_limit(
    engine: &mut Engine,
    m: &Reality,
    on: &Expr,
    env: &Env,
) -> Result<Option<String>, Stop> {
    match engine.ev(m, on, env, Ctx::new(None, m.on)) {
        Ok(Value::Day(day)) if day.is_end() => Ok(Some("a day passes 9999-12-31".to_owned())),
        Err(Stop::Refused(r)) if r.code() == Code::Run(Run::Limit) => {
            Ok(Some(r.message().to_owned()))
        }
        Ok(_) => Ok(None),
        Err(Stop::Refused(r)) if r.code() != Code::EngineInvariant => Ok(None),
        Err(stop) => Err(stop),
    }
}

/// `LOAD-LIMIT <a> .. <b>[ step <s>] holds more than 1 000 000 keys` for a range an ask's
/// binding steps through whose ends and step are literals, which load sees: refused at load
/// whatever the asks, at the ask's line, with the bytes the asked path prints (refcore's
/// `m_check_literal_ranges`). A range the evaluator refuses otherwise, or doesn't build, is
/// left to the ask that steps through it. L §7.6 item 2, §8.3 `LOAD-LIMIT`; E §3.1.
///
/// # Errors
///
/// The first such range's `LOAD-LIMIT`, the asks in the reality's order; an invariant the
/// range meets.
pub(super) fn check_literal_ranges(m: &Reality, files: &Files) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for ask in &m.asks {
        for parts in literal_ranges(ask) {
            match engine.literal_range(m, parts) {
                Err(Stop::Refused(r)) if r.code() == Code::Load(Load::Limit) => {
                    let file = files.get("ask", &ask.id)?;
                    return Err(refuse(Load::Limit, r.message().to_owned(), file, ask.line));
                }
                Err(Stop::Refused(r)) if r.code() == Code::EngineInvariant => {
                    return Err(Stop::Refused(r));
                }
                Err(Stop::Deep) => return Err(Stop::Deep),
                Ok(()) | Err(Stop::Refused(_) | Stop::NotBuilt(_)) => {}
            }
        }
    }
    Ok(())
}

/// A range's ends and step as written, `a .. b [step s]` (L §2.3 `bind`).
type RangeParts = (Expr, Expr, Option<Expr>);

/// The ranges an ask's bindings step through whose ends and step are literals: the ask's own
/// bindings first, then those its expressions hold (refcore's `m_check_literal_ranges`, its
/// walk of the ask). L §2.5.
fn literal_ranges(ask: &Ask) -> Vec<&RangeParts> {
    let mut binds: Vec<&Bind> = ask.binds.iter().collect();
    if let AskBody::Diff(diff) = &ask.body {
        binds.extend(&diff.binds);
    }
    let mut stack = ask_exprs(ask);
    let mut out = Vec::new();
    loop {
        for bind in core::mem::take(&mut binds) {
            let (range, inner) = bind_parts(bind);
            out.extend(range);
            stack.extend(inner);
        }
        let Some(e) = stack.pop() else {
            return out;
        };
        binds.extend(expr_binds(e));
        stack.extend(all_children(e));
    }
}

/// A binding's range, when its ends and step are literals, and the expressions it holds for
/// the walk: its range's, its `on`, its `where`.
fn bind_parts(bind: &Bind) -> (Option<&RangeParts>, Vec<&Expr>) {
    let mut inner: Vec<&Expr> = bind.on.iter().chain(bind.filter.iter()).collect();
    let Written::Range(parts) = &bind.domain else {
        return (None, inner);
    };
    inner.extend([&parts.0, &parts.1]);
    inner.extend(parts.2.as_ref());
    let literal =
        literal_end(&parts.0) && literal_end(&parts.1) && parts.2.as_ref().is_none_or(literal_end);
    (literal.then_some(&**parts), inner)
}

/// An ask's expressions: its body's, its `law` and `as known`, its pins' values.
fn ask_exprs(ask: &Ask) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = match &ask.body {
        AskBody::Exprs(exprs) => exprs.iter().collect(),
        AskBody::Residuals(residuals) => vec![&residuals.place],
        AskBody::Explain(e) | AskBody::InputsOf(e) | AskBody::ReadersOf(e) => vec![e],
        AskBody::Check(a, b) => vec![a, b],
        AskBody::Solve(solve) => vec![&solve.left, &solve.right, &solve.target],
        AskBody::Diff(diff) => vec![&diff.expr],
    };
    out.extend(ask.law.iter().chain(ask.known.iter()));
    for pin in &ask.pins {
        match &pin.form {
            PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => out.push(e),
            PinForm::Set { value, .. } => out.push(value),
            PinForm::Scenario(_) => {}
        }
    }
    out
}

/// The bindings an expression holds itself: an aggregate's, a query's, a distribution's, a
/// `crps`'s, a `prorata`'s; and a `diff`'s, which its ask holds.
fn expr_binds(e: &Expr) -> Vec<&Bind> {
    if let Expr::Agg(agg) = e {
        return agg.binds.iter().collect();
    }
    if let Expr::Query(query) = e {
        return query.binds.iter().flatten().collect();
    }
    if let Expr::Dist(dist) = e {
        return dist.bind.iter().collect();
    }
    if let Expr::Crps(crps) = e {
        return crps.bind.iter().collect();
    }
    if let Expr::Prorata(prorata) = e {
        return prorata.binds.iter().collect();
    }
    Vec::new()
}

/// Whether a range's end or step is a literal: a number (a duration included), a day, a
/// month or a quarter, negated or in parentheses (refcore's `range_is_literal`). L §2.5.
fn literal_end(e: &Expr) -> bool {
    let mut node = e;
    while let Expr::Neg(inner) | Expr::Paren(inner) = node {
        node = inner;
    }
    matches!(
        node,
        Expr::Lit(Lit::Num(_) | Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..))
    )
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

/// `LOAD-SCENARIO` for a move a scenario's file declares that books before the scenario's
/// `from` where load sees the day: at a choice of its keys over declared indexes, its `on`
/// (a window's first day) or a leg's own day falls before it (L §6.4: such a move books
/// nothing before D; issue #75, where refcore loads it). A fold that gives no day refuses
/// nothing. L §8.3.
///
/// # Errors
///
/// The first such move's `LOAD-SCENARIO`, by id, at its line or its leg's; an invariant or
/// `NotBuilt` the fold meets.
pub(super) fn check_scenario_moves(m: &Reality, files: &Files) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for (id, mv) in m.moves.entries() {
        let Some(scenario) = m.scenarios.of(id) else {
            continue;
        };
        let Some(from) = m.scenarios.from.get(scenario).copied() else {
            continue;
        };
        let mut days = vec![(&mv.decl.on, mv.decl.line)];
        for leg in &mv.decl.legs {
            if let Some(on) = &leg.on {
                days.push((on, leg.line));
            }
        }
        days.retain(|(day, _)| seen_day(m, day, &mv.binds));
        let Some(domains) = bind_keys(m, &mv.binds) else {
            continue;
        };
        let mut at = first_choice(&domains);
        while let Some(counters) = at {
            let (_, env) = choice(&mv.binds, &domains, &counters)?;
            at = next_choice(&counters, &domains)?;
            if let Some((first, line)) = early_day(&mut engine, m, &days, &env, from)? {
                let message = format!(
                    "move {id} of {scenario} books on {}, before {}",
                    first.text(),
                    from.text()
                );
                return Err(refuse(
                    Load::Scenario,
                    message,
                    files.get("move", id)?,
                    line,
                ));
            }
        }
    }
    Ok(())
}

/// The first of a move's days, at one choice of keys, that falls before `from`, with its
/// line. L §6.4.
fn early_day(
    engine: &mut Engine,
    m: &Reality,
    days: &[(&Expr, u32)],
    env: &Env,
    from: Day,
) -> Result<Option<(Day, u32)>, Stop> {
    for (day, line) in days {
        if let Some(first) = first_day(engine, m, day, env)?
            && first < from
        {
            return Ok(Some((first, *line)));
        }
    }
    Ok(None)
}

/// A day's value at one choice of keys, or a window's first day. L §4.4, §5.14.
fn first_day(engine: &mut Engine, m: &Reality, day: &Expr, env: &Env) -> Result<Option<Day>, Stop> {
    Ok(match fold_value(engine, m, day, env)? {
        Some(Value::Day(d)) => Some(d),
        Some(Value::Period(period) | Value::Key(KeyVal::Period(period))) => Some(period.start()),
        Some(_) | None => None,
    })
}

/// A read of a random input or def in a random move's leg amount, its keys as written
/// (refcore's `rreads` entry).
struct RandomRead {
    /// The input or def read.
    id: String,
    /// Whether it is a def.
    def: bool,
    /// Its keys, as written.
    keys: Vec<Expr>,
}

/// `LOAD-RANDOM` for a decision's own projection `bal(x, d) from t` that load sees reach a
/// random amount dated after t: its day, t and d seen at a choice of its keys; a random
/// movement booking x on a seen day in (t, d], with no `when` and no `if` in its amount; a
/// random fact its amount reads, at a law date after t (refcore's `m_check_proj_seen`; a move
/// with no `at` dates its amount by its day, issue #75, where refcore leaves it to the run).
/// L §5.14.1, §5.16 Decisions and projections, §8.3.
///
/// # Errors
///
/// `LOAD-RANDOM <decision>[<keys>] on <day>: <projection> reaches random <x>[<keys>] after
/// <t> with no random clause or query`, at the projection's line; an invariant or `NotBuilt`
/// a fold meets.
pub(super) fn check_proj_seen(
    m: &Reality,
    projs: &[ProjRead],
    random: &BTreeSet<Node>,
) -> Result<(), Stop> {
    let rmoves: Vec<&MoveInfo> = random
        .iter()
        .filter(|node| node.kind == NodeKind::Move)
        .filter_map(|node| m.moves.get(&node.id))
        .collect();
    if rmoves.is_empty() {
        return Ok(());
    }
    let mut engine = Engine::new(m.on);
    for proj in projs {
        if let Some(message) = proj_reaches(&mut engine, m, proj, &rmoves)? {
            return Err(refuse(Load::Random, message, &proj.at.0, proj.at.1));
        }
    }
    Ok(())
}

/// The refusal's message when load sees `proj` reach a random amount after its t.
fn proj_reaches(
    engine: &mut Engine,
    m: &Reality,
    proj: &ProjRead,
    rmoves: &[&MoveInfo],
) -> Result<Option<String>, Stop> {
    let Some(mi) = m.moves.get(&proj.holder) else {
        return Ok(None);
    };
    let (Target::Thing(_), Some(from), Some(dn)) =
        (&proj.target, &proj.bal.proj.from, proj.bal.args.last())
    else {
        return Ok(None);
    };
    let seen = [&mi.decl.on, from, dn]
        .iter()
        .all(|e| seen_day(m, e, &mi.binds));
    if !mi.decision || mi.decl.when.is_some() || !seen {
        return Ok(None);
    }
    let Some(domains) = bind_keys(m, &mi.binds) else {
        return Ok(None);
    };
    for rm in rmoves {
        if rm.id == mi.id || !seen_random_move(m, rm) {
            continue;
        }
        let (Some(reads), Some(rdomains)) =
            (random_reads(m, rm, &proj.target)?, bind_keys(m, &rm.binds))
        else {
            continue;
        };
        if reads.is_empty() {
            continue;
        }
        let mut at = first_choice(&domains);
        while let Some(counters) = at {
            let (keys, env) = choice(&mi.binds, &domains, &counters)?;
            at = next_choice(&counters, &domains)?;
            let days = (
                fold_day(engine, m, &mi.decl.on, &env)?,
                fold_day(engine, m, from, &env)?,
                fold_day(engine, m, dn, &env)?,
            );
            let (Some(day), Some(t), Some(d)) = days else {
                continue;
            };
            if day < m.start || day > m.end {
                continue;
            }
            let window = (t, d);
            let Some((id, rkeys)) = late_read(engine, m, rm, &rdomains, &reads, window)? else {
                continue;
            };
            let shown: Vec<String> = keys.iter().map(KeyVal::text).collect();
            let message = format!(
                "{}{} on {}: {} reaches random {id}{} after {} with no random clause or query",
                mi.id,
                bracketed(&shown),
                day.text(),
                text(&Expr::Bal(Box::new(proj.bal.clone()))),
                bracketed(&rkeys),
                t.text()
            );
            return Ok(Some(message));
        }
    }
    Ok(None)
}

/// Keys shown in brackets, or nothing for none.
fn bracketed(keys: &[String]) -> String {
    if keys.is_empty() {
        String::new()
    } else {
        format!("[{}]", keys.join(", "))
    }
}

/// Whether a random move is one load sees whole: no `when`, its day seen, and its `at`, when
/// written, seen.
fn seen_random_move(m: &Reality, rm: &MoveInfo) -> bool {
    let at_seen = rm
        .decl
        .at
        .as_ref()
        .is_none_or(|at| seen_day(m, at, &rm.binds));
    rm.decl.when.is_none() && at_seen && seen_day(m, &rm.decl.on, &rm.binds)
}

/// The first of `reads`, at a choice of the random move's keys whose day falls in `(t, d]`
/// within the run, read at a law date after t: its id and its keys' values.
fn late_read(
    engine: &mut Engine,
    m: &Reality,
    rm: &MoveInfo,
    rdomains: &[Vec<KeyVal>],
    reads: &[RandomRead],
    (t, d): (Day, Day),
) -> Result<Option<(String, Vec<String>)>, Stop> {
    let mut at = first_choice(rdomains);
    while let Some(counters) = at {
        let (_, renv) = choice(&rm.binds, rdomains, &counters)?;
        at = next_choice(&counters, rdomains)?;
        let Some(rday) = fold_day(engine, m, &rm.decl.on, &renv)? else {
            continue;
        };
        let rlaw = match &rm.decl.at {
            Some(e) => fold_day(engine, m, e, &renv)?,
            None => Some(rday),
        };
        let Some(rlaw) = rlaw else {
            continue;
        };
        if rday <= t || rday > d || rday < m.start || rday > m.end {
            continue;
        }
        for read in reads {
            let law = law_of(engine, m, read, rlaw, &renv)?;
            if law.is_none_or(|law| law <= t) {
                continue;
            }
            let mut keys = Vec::new();
            for key in &read.keys {
                keys.push(match fold_value(engine, m, key, &renv)? {
                    Some(value) => value
                        .text(m)
                        .map_err(|_| invariant("a key's text".to_owned()))?,
                    None => text(key),
                });
            }
            return Ok(Some((read.id.clone(), keys)));
        }
    }
    Ok(None)
}

/// The law date a random read is read at, from its move's law date `rlaw`: a def's `at
/// reader` takes it, a def's `at e` is e at the read's keys under it, an input takes it; a
/// def with no `at`, or whose `at` load doesn't see, gives none (refcore's `rreads` laws).
/// L §4.3.
fn law_of(
    engine: &mut Engine,
    m: &Reality,
    read: &RandomRead,
    rlaw: Day,
    renv: &Env,
) -> Result<Option<Day>, Stop> {
    if !read.def {
        return Ok(Some(rlaw));
    }
    let Some(info) = m.defs.get(&read.id) else {
        return Ok(None);
    };
    let e = match &info.at {
        None => return Ok(None),
        Some(LawDate::Reader) => return Ok(Some(rlaw)),
        Some(LawDate::Expr(e)) => e,
    };
    if !seen_day(m, e, &info.binds) {
        return Ok(None);
    }
    let mut env = Env::empty();
    for (bind, key) in info.binds.iter().zip(&read.keys) {
        let Some(value) = fold_value(engine, m, key, renv)? else {
            return Ok(None);
        };
        env = env.with(&bind.var, value, Some(bind.domain.clone()));
    }
    match engine.ev(m, e, &env, Ctx::new(Some(rlaw), m.on)) {
        Ok(Value::Day(law)) => Ok(Some(law)),
        Ok(_) => Ok(None),
        Err(Stop::Refused(r)) if r.code() != Code::EngineInvariant => Ok(None),
        Err(stop) => Err(stop),
    }
}

/// The reads of random inputs and defs in the amounts of a random move's legs that book
/// `target`, or `None` where load doesn't see them whole: an `if` in the amount, or a key it
/// doesn't see (refcore's `rreads`). A read as a distribution, `cdf`'s value, is no sample.
fn random_reads(
    m: &Reality,
    rm: &MoveInfo,
    target: &Target,
) -> Result<Option<Vec<RandomRead>>, Stop> {
    let mut scope = Scope::new(Node::new(NodeKind::Move, &rm.id), Stands::Rule, "", 0);
    for bind in &rm.binds {
        let var = Var {
            name: bind.var.clone(),
            ty: Ty::Unseen,
            dom: VarDom::Arg,
            keylist: Vec::new(),
            id: 0,
            end: None,
        };
        scope.vars.insert(bind.var.clone(), var);
    }
    let mut out = Vec::new();
    for leg in &rm.decl.legs {
        let books = place_target(m, &scope, &leg.from)? == *target
            || place_target(m, &scope, &leg.to)? == *target;
        if !books {
            continue;
        }
        let mut stack = vec![&leg.amount];
        while let Some(x) = stack.pop() {
            if matches!(x, Expr::If(_)) {
                return Ok(None);
            }
            if let Expr::DistFn(f) = x
                && matches!(dist_read(m, &scope, &f.value), AmountNode::Random(_))
            {
                stack.push(&f.at);
                continue;
            }
            match dist_read(m, &scope, x) {
                AmountNode::Unseen => return Ok(None),
                AmountNode::Random(read) => {
                    if !read.keys.iter().all(|key| seen_day(m, key, &rm.binds)) {
                        return Ok(None);
                    }
                    stack.extend(key_exprs(x).into_iter().rev());
                    out.push(read);
                }
                AmountNode::Other => stack.extend(all_children(x).into_iter().rev()),
            }
        }
    }
    Ok(Some(out))
}

/// The key expressions of a read `x[k, …]`, none for a bare name.
fn key_exprs(x: &Expr) -> Vec<&Expr> {
    let Expr::Post(_, posts) = x else {
        return Vec::new();
    };
    let [Postfix::Keys(keys)] = posts.as_slice() else {
        return Vec::new();
    };
    keys.iter()
        .filter_map(|key| match key {
            Key::Expr(e) => Some(e),
            Key::Star => None,
        })
        .collect()
}

/// What a node of a leg's amount is to the projection check.
enum AmountNode {
    /// A read of a random input or def.
    Random(RandomRead),
    /// A read at `*`, which load doesn't see.
    Unseen,
    /// Any other node.
    Other,
}

/// A node of a leg's amount: a read of a random input or def, `x` or `x[k, …]`, or not.
fn dist_read(m: &Reality, scope: &Scope, x: &Expr) -> AmountNode {
    let (name, keys) = if let Expr::Name(name) = x {
        (name, Vec::new())
    } else if let Expr::Post(base, posts) = x
        && let (Expr::Name(name), [Postfix::Keys(keys)]) = (&**base, posts.as_slice())
    {
        let mut exprs = Vec::new();
        for key in keys {
            let Key::Expr(e) = key else {
                return AmountNode::Unseen;
            };
            exprs.push(e.clone());
        }
        (name, exprs)
    } else {
        return AmountNode::Other;
    };
    if scope.is_var(name) {
        return AmountNode::Other;
    }
    let kind = m.names.get(name.as_str()).copied();
    let ty = if kind == Some(NameKind::Input) {
        m.inputs.get(name).and_then(|input| input.ty.as_ref())
    } else if kind == Some(NameKind::Def) {
        m.defs.get(name).map(|def| &def.ty)
    } else {
        None
    };
    if !matches!(ty, Some(Ty::Dist(_) | Ty::DistKeys(_))) {
        return AmountNode::Other;
    }
    AmountNode::Random(RandomRead {
        id: name.clone(),
        def: kind == Some(NameKind::Def),
        keys,
    })
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
        | NodeKind::Fill
        | NodeKind::Srow => None,
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
