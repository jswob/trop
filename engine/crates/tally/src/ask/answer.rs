//! Answering the asks: each ask's cut and law date, its binding tuples, its line's values and the register under each line, as L §5.18 prints them.
//! Spec: L §5.18 (asks: `as known`, `law`, `in`, `for`, one line per binding tuple, a tuple
//! whose `where` is a hole printed as that hole); L §4.3 (the law date), §4.5 (the cut),
//! §5.11 (an ask's `law` or `as known` that is a hole makes the ask line that hole), §8.2 (the
//! register), §8.4 (a run-time refusal prints as the ask's line), §8.5 (warnings last).
//! Twin: engine/tools/refcore/ask.py (`Asker.run`, `answer`, `_answer`, `_answer_in`,
//! `evaluate`, `one`, `print_line`, `fmt_value`, `fmt_bind`) and evaluate.py (`date_of`).
//! Invariants: asks print in id order unless the run names them; each ask has its own engine
//! and memo; a load refusal met at run and every construct S doesn't build stop the run
//! `NotBuilt`, never a guessed line.

use std::collections::BTreeMap;

use super::register::{register_lines, warning_lines};
use super::residuals::residual_lines;
use crate::base::{Code, Load, Refusal, Run, Warn};
use crate::eval::{Collected, Ctx, Engine, Entry, Env, Item, Member, Memo, num_stop};
use crate::model::{Reality, Value};
use crate::syntax::{Ask, AskBody, Bind, Domain, Expr, Stop};
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

/// The lines a run prints for its asks: each ask's lines, the asks in id order or in the order
/// the run names them, then the run's warnings (refcore's `Asker.run`), each ask's engine
/// walking `days` and keeping its cells as `memo` says. L §5.18, §8.5.
///
/// # Errors
///
/// `LOAD-NAME` for an ask the run names that the reality lacks; `NotBuilt` for a load refusal
/// met at run and for a construct S doesn't build.
pub fn answer(m: &Reality, asks: &Asks, days: Days, memo: Memo) -> Result<Answered, Stop> {
    let order: Vec<&Ask> = match asks {
        Asks::Every => {
            let mut all: Vec<&Ask> = m.asks.iter().collect();
            all.sort_by(|a, b| a.id.cmp(&b.id));
            all
        }
        Asks::Named(ids) => {
            let mut named = Vec::with_capacity(ids.len());
            for id in ids {
                let ask = m.asks.iter().find(|a| a.id == *id).ok_or_else(|| {
                    Stop::Refused(Refusal::new(Code::Load(Load::Name), format!("no ask {id}")))
                })?;
                named.push(ask);
            }
            named
        }
    };
    let mut out = Vec::new();
    let mut seen = Seen {
        warnings: m.warnings.clone(),
        booked: BTreeMap::new(),
        memo,
        #[cfg(feature = "counters")]
        counts: crate::Counters::default(),
    };
    for ask in order {
        out.extend(answer_one(m, ask, days, &mut seen)?);
    }
    out.extend(warning_lines(&seen.warnings));
    Ok(Answered {
        lines: out,
        #[cfg(feature = "counters")]
        counts: seen.counts,
    })
}

/// One ask's lines; a run-time refusal its reading meets outside a line's values is its one
/// line (refcore's `answer`). L §8.4.
fn answer_one(m: &Reality, ask: &Ask, days: Days, seen: &mut Seen) -> Result<Vec<String>, Stop> {
    match ask_lines(m, ask, days, seen) {
        Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(load_at_run()),
        Err(Stop::Refused(r)) => Ok(vec![format!("{}: refused: {}", ask.id, r.text())]),
        other => other,
    }
}

/// A load refusal met at run: refcore refuses the run with it, where load's checks (M0's)
/// would have met it first, so S stops `NotBuilt` rather than print another line.
fn load_at_run() -> Stop {
    Stop::NotBuilt("a load refusal met at run (load's checks are M0's)")
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
        if *world != m.header.id {
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
    match &ask.body {
        AskBody::Exprs(exprs) => expr_lines(m, ask, exprs, engine, (ctx, &pinned)),
        AskBody::Residuals(r) => residual_lines(m, ask, r, engine, (ctx, &pinned)),
    }
}

/// An ask's lines of values: a line per binding tuple with the register under it (refcore's
/// `_answer_in`). L §5.18.
fn expr_lines(
    m: &Reality,
    ask: &Ask,
    exprs: &[Expr],
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
        let (items, mut met) = engine.line(m, exprs, &tuple.env, ctx)?;
        out.push(format!("{head}: {}", line_values(m, &items)?));
        met.merge(&tuple.brought);
        met.merge(pinned);
        out.extend(
            register_lines(&met, ctx)?
                .into_iter()
                .map(|l| format!("  {l}")),
        );
    }
    Ok(out)
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
        return Ok(*day);
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
    Err(load_at_run())
}
