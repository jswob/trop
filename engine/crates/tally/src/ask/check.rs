//! An ask's `check e == f`: `ok`, `differs` with the difference, or both sides where one is a hole or refused.
//! Spec: L §5.18 (`check e == f` prints `ok`, or `differs: e = …, f = …` and, where e − f is
//! defined, the difference e − f; a hole or refused side prints in its place with the other
//! side's value beside it; sides read at different law dates list each), §8.2 (the register's
//! law date line), §3.2 and §4.2 (where e − f is defined).
//! Twin: engine/tools/refcore/ask.py (`check`, `side_law`, `_check`, `CheckOut`).
//! Invariants: each side is read once, as an ask line reads a value; a load refusal met at
//! run refuses the run; a side's text is refcore's `_txt` of the expression.

use crate::base::{Code, Refusal, Run};
use crate::eval::{
    Collected, Ctx, Engine, Env, Item, Op, arith, equal_sides, keys_differ, num_stop, partial_text,
    resolved_sides,
};
use crate::hole::Unknown;
use crate::model::{Reality, Value};
use crate::syntax::{Expr, PinForm, Stop};
use crate::time::Day;

/// What an ask's line prints after its head: its text, the lines under it, the register
/// entries it met, and, for a `check`, each side's law date where they differ.
#[derive(Debug, Clone)]
pub(super) struct Printed {
    /// The line's values, or a check's `ok`, `differs: …` or both sides.
    pub text: String,
    /// The lines under the head, before the register: an `explain`'s derivation, an
    /// `inputs-of`'s rows (L §5.18).
    pub body: Vec<String>,
    /// Whether the head stands alone and the lines under it are the result: `inputs-of`'s
    /// (refcore's `print_line`, its `RowSet`).
    pub bare: bool,
    /// Whether the lines under the head are part of the result, as a `diff`'s are (L §5.5).
    pub whole: bool,
    /// Whether the line prints alone, no register under it: a residual ledger whose place or
    /// span is a hole (refcore's `answer_residuals`, L §8.1).
    pub alone: bool,
    /// The register entries the line met.
    pub met: Collected,
    /// A check's sides' law dates, where a side's own `as if {law D}` makes them differ.
    pub laws: Option<(Option<Day>, Option<Day>)>,
    /// The unknown days the holes among the values it prints carry: what its register's
    /// `unknown day:` lines name (L §4.4, §8.2; refcore's `hole_days`).
    pub unknown: Vec<Unknown>,
}

impl Printed {
    /// The line's head `head` with its text, then the lines under it (refcore's
    /// `print_line`). L §5.18.
    pub(super) fn lines(&self, head: &str) -> Vec<String> {
        let first = if self.bare {
            format!("{head}:")
        } else {
            format!("{head}: {}", self.text)
        };
        let mut out = vec![first];
        out.extend(self.body.iter().cloned());
        out
    }

    /// Whether `other` gives the same result: the same text, and the same lines under a bare
    /// head or a diff's (refcore's `Result.key`, `rkey`, `diff_print`). L §5.5.
    pub(super) fn same(&self, other: &Printed) -> bool {
        self.text == other.text
            && self.bare == other.bare
            && self.whole == other.whole
            && (!(self.bare || self.whole) || self.body == other.body)
    }
}

/// The unknown days the holes among the values `items` carry, in order, each once (refcore's
/// `hole_days`). L §4.4, §8.2.
pub(super) fn hole_days(items: &[&Item]) -> Vec<Unknown> {
    let mut out = Vec::new();
    for item in items {
        if let Item::Value(v) = item {
            v.hole_days_into(&mut out);
        }
    }
    out
}

/// The line of `check left == right` of ask `ask` in scope `env` and context `ctx` (refcore's
/// `check`, `_check`). L §5.18.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a form not built yet.
pub(super) fn check_line(
    m: &Reality,
    engine: &mut Engine,
    (left, right): (&Expr, &Expr),
    ask: &str,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    engine.push();
    let read = sides_text(m, engine, (left, right), ask, env, ctx).and_then(|text| {
        let laws = (
            side_law(m, engine, left, env, ctx)?,
            side_law(m, engine, right, env, ctx)?,
        );
        Ok((text, laws))
    });
    let met = engine.pop_alone();
    let ((text, unknown), laws) = read?;
    Ok(Printed {
        text,
        body: Vec::new(),
        bare: false,
        whole: false,
        alone: false,
        met,
        laws: (laws.0 != laws.1).then_some(laws),
        unknown,
    })
}

/// The check's text: both sides read, then `ok`, `differs` with the difference, or both sides
/// where one is a hole or refused, with the unknown days those carry, the open comparison's
/// too (refcore's `_check`, its `CheckOut` values). L §5.18, §4.4.
fn sides_text(
    m: &Reality,
    engine: &mut Engine,
    (left, right): (&Expr, &Expr),
    ask: &str,
    env: &Env,
    ctx: Ctx,
) -> Result<(String, Vec<Unknown>), Stop> {
    let lv = side(engine.value(m, left, env, ctx))?;
    let rv = side(engine.value(m, right, env, ctx))?;
    let (lt, rt) = (crate::load::expr_text(left), crate::load::expr_text(right));
    let both = |open: Option<&Value>| -> Result<(String, Vec<Unknown>), Stop> {
        let text = format!(
            "{lt} = {}, {rt} = {}",
            side_text(m, &lv)?,
            side_text(m, &rv)?
        );
        let mut unknown = hole_days(&[&lv, &rv]);
        if let Some(v) = open {
            v.hole_days_into(&mut unknown);
        }
        Ok((text, unknown))
    };
    let (Item::Value(a), Item::Value(b)) = (&lv, &rv) else {
        return both(None);
    };
    if a.as_hole().is_some() || b.as_hole().is_some() {
        return both(None);
    }
    let same = match equal_sides(m, (left, a), (right, b)) {
        Ok(v) => v,
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
            return Ok((format!("refused: {}", r.text()), Vec::new()));
        }
        Err(stop) => return Err(stop),
    };
    // an open comparison (a hole) prints both sides (refcore's `open`)
    let Value::Bool(same) = same else {
        return both(Some(&same));
    };
    if same {
        return Ok(("ok".to_owned(), Vec::new()));
    }
    // each side as the comparison took it: a key literal the other side's key, a four-digit
    // literal beside a year that year (refcore's sides, evaluated as load resolved them)
    let (a, b) = resolved_sides(m, (left, a), (right, b))?;
    let text = format!(
        "differs: {lt} = {}, {rt} = {}{}",
        plain(m, &a)?,
        plain(m, &b)?,
        difference(m, (&a, &b), ask)?
    );
    Ok((text, Vec::new()))
}

/// A side read as an ask line reads a value: a run-time refusal stands in its place; a load
/// refusal met at run refuses the run (refcore's `one`). L §5.18, §8.3.
fn side(read: Result<Value, Stop>) -> Result<Item, Stop> {
    match read {
        Ok(v) => Ok(Item::Value(v)),
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => Ok(Item::Refused(r)),
        Err(stop) => Err(stop),
    }
}

/// `, difference e − f` where e − f is defined, of two numbers or two money values; nothing
/// for two dates, texts or codes, or where a side is `none`; `, difference refused: …` where
/// the subtraction is refused (refcore's `_check`). Two keys differ only where load typed both
/// sides as keys of one ordered index (`keyed_checks`): the int of positions, of the integers
/// for an integer index, or of the periods for a period index (L §4.2 "Differences"); a period
/// typed as no key prints none, as on refcore. L §5.18, §3.2, §3.1.
fn difference(m: &Reality, (a, b): (&Value, &Value), ask: &str) -> Result<String, Stop> {
    let numbers = |v: &Value| matches!(v, Value::Number(_) | Value::Money(_));
    if *a == Value::None || *b == Value::None {
        // `none` in arithmetic is `RUN-NONE`: e − f isn't defined (L §3.1)
        return Ok(String::new());
    }
    if !(numbers(a) && numbers(b) || m.keyed_checks.contains(ask)) {
        // dates, periods load typed as no key, things, texts, codes: no difference
        return Ok(String::new());
    }
    let keyed = if m.keyed_checks.contains(ask) {
        keys_differ(m, a, b).transpose()
    } else {
        None
    };
    match keyed.unwrap_or_else(|| arith(m, Op::Sub, a, b)) {
        Ok(d) => Ok(format!(", difference {}", plain(m, &d)?)),
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
            Ok(format!(", difference refused: {}", r.text()))
        }
        Err(stop) => Err(stop),
    }
}

/// A side as a check prints it in place of a decided comparison: a refusal as `refused: …`, a
/// key outside its index as its `RUN-KEY`, under `partial` a hole in its linear form (tally
/// 2.1.8 R15), any other
/// value as L §5.18 prints it (refcore's `fmt_value` with no static type and the line's
/// `partial`).
fn side_text(m: &Reality, item: &Item) -> Result<String, Stop> {
    if let Item::Value(Value::Hole(h)) = item
        && let Some(text) = partial_text(m, h)?
    {
        return Ok(text);
    }
    match item {
        Item::Refused(r) => Ok(format!("refused: {}", r.text())),
        Item::Value(Value::OutKey { index, text }) => Ok(format!(
            "refused: {} {text} is outside {index}",
            Code::Run(Run::Key).as_str()
        )),
        Item::Value(v) => plain(m, v),
    }
}

/// A value as refcore's `fmt` prints it.
fn plain(m: &Reality, v: &Value) -> Result<String, Stop> {
    v.text(m).map_err(num_stop)
}

/// The law date a side is read at: its own `as if {law D}`'s D where it states one, else the
/// line's; none where D is a hole or refused (refcore's `side_law`). L §5.18.
///
/// # Errors
///
/// `NotBuilt` for a form not built yet.
fn side_law(
    m: &Reality,
    engine: &mut Engine,
    e: &Expr,
    env: &Env,
    ctx: Ctx,
) -> Result<Option<Day>, Stop> {
    let Expr::AsIf(_, pins) = e.unbracketed() else {
        return Ok(ctx.law);
    };
    let Some(law) = pins.iter().find_map(|pin| match &pin.form {
        PinForm::Law(d) => Some(d),
        PinForm::Known(_) | PinForm::Wording(..) | PinForm::Scenario(_) | PinForm::Set { .. } => {
            None
        }
    }) else {
        return Ok(ctx.law);
    };
    // a refusal leaves the side with no law date; a load refusal and an engine invariant are
    // the run's, as in `check_line` (R-66)
    let v = match engine.value(m, law, env, ctx) {
        Ok(v) => v,
        Err(Stop::Refused(r)) if side_keeps(&r) => return Ok(None),
        Err(stop) => return Err(stop),
    };
    if v.as_hole().is_some() {
        return Ok(None);
    }
    match crate::eval::one_day(m, &v, "the law pin") {
        Ok(Ok(day)) => Ok(Some(day)),
        Ok(Err(_)) => Ok(None),
        Err(Stop::Refused(r)) if side_keeps(&r) => Ok(None),
        Err(stop) => Err(stop),
    }
}

/// Whether refusal `r` of a side's law pin leaves the side with no law date: every refusal
/// but a load refusal met at run and an engine invariant, which are the run's (L §8.3, R-66).
fn side_keeps(r: &Refusal) -> bool {
    !matches!(r.code(), Code::Load(_) | Code::EngineInvariant)
}
