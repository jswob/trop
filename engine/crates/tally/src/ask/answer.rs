//! Answering the asks: each ask's cut, law date and `with` pins, its binding tuples, its line's values, check or solve, and each line's register.
//! Spec: L §5.18 (asks: `as known`, `law`, `in`, `with`, `for`, `check`, `solve`, one line
//! per binding tuple, a tuple whose `where` is a hole printed as that hole); L §4.3 (the law
//! date), §4.5 (the cut), §5.11 (an ask's `law`, `as known` or pin that is a hole makes the ask
//! line that hole), §8.2 (the register), §8.4 (a run-time refusal prints as the ask's line),
//! §8.5 (warnings last); E §11 (`REQ-ASK`, tally 2.1.3; `REQ-CLOCK`).
//! Twin: engine/tools/refcore/ask.py (`Asker.run`, `req_ask`, `answer`, `_answer`,
//! `_answer_in`, `evaluate`, `one`, `one_solve`, `print_line`, `fmt_value`, `fmt_bind`) and
//! evaluate.py (`date_of`, `apply_pins`); refcore answers an `as known` after `on` where E §11
//! refuses it (codes/m1 held.tsv, P22).
//! Invariants: asks print in id order unless the run names them; every id the run names is
//! checked before any ask is answered; each ask has its own engine and memo; a load refusal met
//! at run refuses the run in the base's world and the ask alone in a scenario's (refcore's
//! `LoadAtRun`); every construct not built yet stops the run `NotBuilt` at the ask that meets
//! it, never a guessed line.

use std::collections::BTreeMap;

use super::check::{Printed, check_line};
use super::register::{check_register_lines, register_lines, warning_lines};
use super::residuals::residual_lines;
use crate::base::{Code, Refusal, Req, Run, Warn};
use crate::eval::{
    Collected, Ctx, Engine, Entry, Env, Item, Member, Memo, a_day, num_stop, past_days,
};
use crate::model::{Reality, Value};
use crate::syntax::{Ask, AskBody, Bind, Domain, Expr, Solve, Stop};
use crate::time::{Day, PeriodKind};
use crate::walk::Booked;
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
    booked: BTreeMap<(String, Day), Booked>,
    /// How each ask's engine keeps its cells across law dates (E §3.2).
    memo: Memo,
    /// The engines' counts together.
    #[cfg(feature = "counters")]
    counts: crate::Counters,
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
    /// A line, printed as it is.
    Line(String),
}

/// The lines a run prints for its asks: each ask's lines, the asks in id order or in the order
/// the run names them, then the run's warnings (refcore's `Asker.run`), each ask's engine
/// walking `days` and keeping its cells as `memo` says. L §5.18, §8.5.
///
/// # Errors
///
/// `REQ-ASK` for the first ask the run names that the reality lacks, before any is answered
/// (E §11); a load refusal met at run; `NotBuilt` for a construct not built yet.
pub fn answer(m: &Reality, asks: &Asks, days: Days, memo: Memo) -> Result<Answered, Stop> {
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
    answer_turns(&[m], &turns, m.warnings.clone(), days, memo)
}

/// `REQ-ASK no ask <q>`: the request names an ask that doesn't exist. E §11 (tally 2.1.3).
pub(crate) fn no_ask(id: &str) -> Refusal {
    Refusal::new(Code::Req(Req::Ask), format!("no ask {id}"))
}

/// The lines of `turns` in order, each ask's engine walking `days` and keeping its cells as
/// `memo` says, then `warnings` with the walks' own (refcore's `Asker.run`, `run_worlds`).
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
    memo: Memo,
) -> Result<Answered, Stop> {
    let mut out = Vec::new();
    // each world's asks share what their engines gather, apart from another world's
    // (refcore's one `Asker` per world)
    let mut worlds: Vec<World> = realities.iter().map(|m| World::of(m, memo)).collect();
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
            Turn::Line(line) => out.push(line.clone()),
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

/// What a world's asks' engines gather, and what its reality holds that the run doesn't take
/// yet (refcore's `Asker` of a world).
struct World {
    /// What its asks' engines gather.
    seen: Seen,
    /// Whether it holds a known window or a `moves` line, which the run doesn't take yet.
    unbuilt: bool,
    /// Whether it keeps parts, which the run doesn't take yet.
    parts: bool,
}

impl World {
    /// The world of reality `m` before any of its asks, its engines keeping cells as `memo`
    /// says.
    fn of(m: &Reality, memo: Memo) -> World {
        World {
            seen: Seen {
                warnings: Vec::new(),
                booked: BTreeMap::new(),
                memo,
                #[cfg(feature = "counters")]
                counts: crate::Counters::default(),
            },
            unbuilt: windows_or_moves(m),
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
            return Err(Stop::NotBuilt("a known window or a moves line at run"));
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

/// Whether the reality holds a known window (a version's, a row's or an observation's own) or
/// a `moves` line: load checks them (L §4.5, §5.3), the run doesn't take them yet (M1's), so
/// an ask that would read them stops `NotBuilt`.
fn windows_or_moves(m: &Reality) -> bool {
    let versions = m
        .versions
        .entries()
        .any(|(_, v)| v.window.is_some() || !v.moved_by.is_empty());
    if versions {
        return true;
    }
    for (_, input) in m.inputs.entries() {
        let rows = input.rows.iter().any(|r| r.own_window.is_some());
        let unread = |o: &crate::model::Observation| o.own_window.is_some();
        if rows || input.observations.iter().any(unread) {
            return true;
        }
    }
    for (_, def) in m.defs.entries() {
        if def.givens.iter().any(|r| r.own_window.is_some()) {
            return true;
        }
    }
    false
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
/// ask, `orient`, an axis it reaches, a fill or a `same` line of the reality, or a joint
/// model's merged reality (L §5.5, §5.12, §5.16, §5.18, §6.1).
fn unbuilt_at_run(m: &Reality, ask: &Ask) -> Option<&'static str> {
    if ask.random.is_some() || ask.paths.is_some() || ask.path.is_some() {
        return Some("a random ask at run");
    }
    if ask.orient.is_some() {
        return Some("orient at run");
    }
    if m.graph.ask_axes.get(&ask.id).is_some_and(|a| !a.is_empty()) {
        return Some("an axis at run");
    }
    if !m.fills.is_empty() {
        return Some("fills at run");
    }
    if !m.sames.is_empty() {
        return Some("same lines at run");
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
    match ask_lines(m, ask, days, seen) {
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_) | Code::Req(_)) => {
            Ok(vec![format!("{}: refused: {}", ask.id, r.text())])
        }
        other => other,
    }
}

/// One ask's lines: its cut by `as known`, its law date by `law`, then one line per binding
/// tuple with the register under it; once answered, the timelines its engines walked are
/// checked unchanged (refcore's `_answer`, `_answer_in`). L §5.18, E §9 item 4.
fn ask_lines(m: &Reality, ask: &Ask, days: Days, seen: &mut Seen) -> Result<Vec<String>, Stop> {
    if ask.windows_each {
        return Err(Stop::NotBuilt("an ask's `windows each`"));
    }
    if ask.partial {
        return Err(Stop::NotBuilt("a partial answer"));
    }
    let mut cut = m.on;
    let mut known_engine = None;
    if let Some(known) = &ask.known {
        let mut engine = Engine::new(m.on)
            .with_days(days)
            .with_memo(seen.memo)
            .for_ask(ask)
            .with_booked(core::mem::take(&mut seen.booked));
        let v = engine.value(m, known, &Env::empty(), Ctx::new(None, cut));
        gather(seen, &mut engine);
        let v = v?;
        if let Value::Hole(_) = v {
            return Ok(vec![format!("{}: {}", ask.id, shown(m, &v)?)]);
        }
        cut = date_of(&v, "as known")?;
        if cut > m.on {
            // E §11: an `as known` after the model's `on` refuses the request (P22's line)
            return Err(Stop::Refused(Refusal::new(
                Code::Req(Req::Clock),
                format!(
                    "as known {} is after the model's on {}",
                    cut.text(),
                    m.on.text()
                ),
            )));
        }
        known_engine = Some(engine);
    }
    let mut engine = Engine::new(cut)
        .with_days(days)
        .with_memo(seen.memo)
        .for_ask(ask)
        .with_booked(core::mem::take(&mut seen.booked));
    let out = ask_body(m, ask, &mut engine, cut);
    gather(seen, &mut engine);
    let lines = out?;
    // E §9 item 4: asks move no money; the walks the ask made are as they were when walked
    if let Some(first) = &known_engine {
        first.asks_moved_no_money()?;
    }
    engine.asks_moved_no_money()?;
    Ok(lines)
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
    #[cfg(feature = "counters")]
    seen.counts.merge(engine.counters());
}

/// One ask's lines from its cut on: its law date, then a line per binding tuple with the
/// register under it, or the residual ledger (refcore's `_answer`, `_answer_in`). L §5.18,
/// E §6.3.
fn ask_body(m: &Reality, ask: &Ask, engine: &mut Engine, cut: Day) -> Result<Vec<String>, Stop> {
    let mut law = None;
    if let Some(e) = &ask.law {
        let v = engine.value(m, e, &Env::empty(), Ctx::new(None, cut))?;
        if let Value::Hole(_) = v {
            return Ok(vec![format!("{}: {}", ask.id, shown(m, &v)?)]);
        }
        law = Some(date_of(&v, "the law of an ask")?);
    }
    let mut pinned = Collected::default();
    if let Some(world) = &ask.world {
        // the reality's own world, its whole stack seen: a world under it (the model's, a
        // lower scenario's) hides the layers over it, M1's overlays
        let own = m.scenarios.own().unwrap_or(m.header.id.as_str());
        if world != own {
            return Err(Stop::NotBuilt("an ask in a scenario's world"));
        }
        pinned.entries.insert(Entry {
            word: "world",
            text: world.clone(),
            figure: String::new(),
        });
    }
    // the line's own reads take its `with uncertain`; `as known` and `law` don't (L §5.18)
    let plain = Ctx::new(law, cut);
    let ctx = if ask.uncertain {
        plain.with_uncertain()
    } else {
        plain
    };
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
        engine.open_layer(inside.layer);
        inside
    };
    match &ask.body {
        AskBody::Exprs(exprs) => form_lines(m, ask, Form::Values(exprs), engine, (ctx, &pinned)),
        AskBody::Check(left, right) => {
            form_lines(m, ask, Form::Check(left, right), engine, (ctx, &pinned))
        }
        AskBody::Solve(form) => form_lines(m, ask, Form::Solve(form), engine, (ctx, &pinned)),
        AskBody::Residuals(r) => residual_lines(m, ask, r, engine, (ctx, &pinned)),
        AskBody::Explain(_) | AskBody::InputsOf(_) | AskBody::ReadersOf(_) | AskBody::Diff(_) => {
            Err(Stop::NotBuilt(
                "an ask's form beyond values, check, solve and residuals",
            ))
        }
    }
}

/// What an ask's line reads for each binding tuple (refcore's `evaluate`, its forms). L §5.18.
#[derive(Debug, Clone, Copy)]
enum Form<'a> {
    /// Its values, `e, f, …`.
    Values(&'a [Expr]),
    /// `check e == f`.
    Check(&'a Expr, &'a Expr),
    /// `solve f == g for v in [lo, hi] to u`.
    Solve(&'a Solve),
}

/// An ask's lines: a line per binding tuple with the register under it (refcore's
/// `_answer_in`, `print_line`). L §5.18.
fn form_lines(
    m: &Reality,
    ask: &Ask,
    form: Form<'_>,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let tuples = if ask.binds.is_empty() {
        vec![Member {
            env: Env::empty(),
            unknown: None,
            brought: Collected::default(),
        }]
    } else {
        match engine.tuples(m, &ask.binds, ctx)? {
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
        let mut line = form_line(m, form, engine, &tuple.env, ctx)?;
        out.push(format!("{head}: {}", line.text));
        line.met.merge(&tuple.brought);
        line.met.merge(pinned);
        let register = match line.laws {
            Some(laws) => check_register_lines(&line.met, ctx, laws)?,
            None => register_lines(&line.met, ctx)?,
        };
        out.extend(register.into_iter().map(|l| format!("  {l}")));
    }
    Ok(out)
}

/// One tuple's line after its head, with the register entries it met (refcore's `evaluate`).
/// L §5.18.
fn form_line(
    m: &Reality,
    form: Form<'_>,
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    match form {
        Form::Values(exprs) => {
            let (items, met) = engine.line(m, exprs, env, ctx)?;
            Ok(Printed {
                text: line_values(m, &items)?,
                met,
                laws: None,
            })
        }
        Form::Check(left, right) => check_line(m, engine, (left, right), env, ctx),
        Form::Solve(solve) => {
            let (item, met) = engine.solve_line(m, solve, env, ctx)?;
            Ok(Printed {
                text: item_text(m, &item)?,
                met,
                laws: None,
            })
        }
    }
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
fn item_text(m: &Reality, item: &Item) -> Result<String, Stop> {
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
            Ok(crate::hole::Hole::asked(&text, w.figure()).text())
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
fn env_text(m: &Reality, binds: &[Bind], env: &Env) -> Result<String, Stop> {
    if binds.is_empty() {
        return Ok(String::new());
    }
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
    Ok(format!(" ({})", parts.join(", ")))
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

/// The day a value names, for an ask's `law` or `as known` (refcore's `date_of`). L §4.3, §4.5.
pub(crate) fn date_of(v: &Value, what: &str) -> Result<Day, Stop> {
    if let Value::Day(day) = v {
        return a_day(*day);
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(p.start());
    }
    if *v == Value::None {
        return Err(Stop::Refused(Refusal::new(
            Code::Run(Run::None),
            format!("a value is none here: {what}"),
        )));
    }
    if *v == Value::Never {
        return Err(Stop::Refused(Refusal::new(
            Code::Run(Run::None),
            format!("never in {what}"),
        )));
    }
    // refcore's `LOAD-UNIT <what> is <v>, not a date`, which load's types meet first
    Err(Stop::NotBuilt(
        "a law date or a cut that isn't a date, met at run",
    ))
}
