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
    Collected, Ctx, Engine, Env, Item, Op, arith, equal_sides, num_stop, partial_text,
};
use crate::hole::Unknown;
use crate::model::{KeyVal, Reality, Value};
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
    /// The unknown days the values it prints carry, in the order met: what `windows each`
    /// takes next (L §4.4; tally 2.1.8 R13; refcore's `carried` of a result).
    pub carried: Vec<Unknown>,
    /// An `explain`'s causes as two runs of `windows each` compare them: each cause with its
    /// value and its rules, whatever law date it ran at, a block printing its first
    /// sub-window's (refcore's `each_key`, `trace_key`; tally 2.1.8 R13, #191 Q4).
    pub causes: Option<String>,
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

    /// Whether `other`, another run of a `windows each` line, gives the same answer: the same
    /// result, and an `explain`'s same causes (refcore's `each_key`). L §4.4.
    pub(super) fn same_run(&self, other: &Printed) -> bool {
        self.same(other) && self.causes == other.causes
    }

    /// The answer of a run of a `windows each` line as one text, equal for two runs exactly
    /// where `same_run` holds: the text, the two kinds, the lines under a bare head or a
    /// diff's, and an `explain`'s causes (refcore's `each_key`, `rkey`). L §5.5, §4.4.
    pub(super) fn key(&self) -> String {
        let under = if self.bare || self.whole {
            self.body.join("\u{0}")
        } else {
            String::new()
        };
        format!(
            "{}\u{0}{}{}\u{0}{under}\u{0}{}",
            self.text,
            u8::from(self.bare),
            u8::from(self.whole),
            self.causes.as_deref().unwrap_or("")
        )
    }
}

/// The unknown days the values `items` carry, in order, each once (refcore's `carried`).
/// L §4.4; tally 2.1.8 R13.
pub(super) fn carried_of(items: &[&Item]) -> Vec<Unknown> {
    let mut out = Vec::new();
    for item in items {
        if let Item::Value(v) = item {
            v.carried_into(&mut out);
        }
    }
    out
}

/// A walk explanation's lines `body` as two runs of `windows each` compare them: its causes and
/// their values whatever days they fall on, each date written `D`, a line of a date alone left
/// out, sorted; a block prints its first sub-window's (refcore's `each_key` of its
/// `ExplainLines`). L §4.4; tally 2.1.8 R13.
pub(super) fn undated_causes(body: &[String]) -> String {
    let mut out: Vec<String> = Vec::with_capacity(body.len());
    for line in body {
        let undated = dates_written_d(line);
        if undated.trim() == "D:" {
            continue;
        }
        out.push(undated);
    }
    out.sort();
    out.join("\u{0}")
}

/// `line` with each `yyyy-mm-dd` in it written `D` (refcore's `re.sub(r'\d{4}-\d{2}-\d{2}',
/// 'D', ln)`).
fn dates_written_d(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut rest: &[char] = &chars;
    while let [first, tail @ ..] = rest {
        if let [y1, y2, y3, y4, '-', m1, m2, '-', d1, d2, after @ ..] = rest
            && [y1, y2, y3, y4, m1, m2, d1, d2]
                .iter()
                .all(|c| c.is_ascii_digit())
        {
            out.push('D');
            rest = after;
            continue;
        }
        out.push(*first);
        rest = tail;
    }
    out
}

/// The line of `check left == right` in scope `env` and context `ctx` (refcore's `check`,
/// `_check`). L §5.18.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a form not built yet.
pub(super) fn check_line(
    m: &Reality,
    engine: &mut Engine,
    (left, right): (&Expr, &Expr),
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    engine.push();
    let read = sides_text(m, engine, (left, right), env, ctx).and_then(|text| {
        let laws = (
            side_law(m, engine, left, env, ctx)?,
            side_law(m, engine, right, env, ctx)?,
        );
        Ok((text, laws))
    });
    let met = engine.pop_alone();
    let ((text, carried), laws) = read?;
    Ok(Printed {
        text,
        body: Vec::new(),
        bare: false,

        whole: false,
        alone: false,
        met,
        laws: (laws.0 != laws.1).then_some(laws),
        carried,
        causes: None,
    })
}

/// The check's text: both sides read, then `ok`, `differs` with the difference, or both sides
/// where one is a hole or refused, with the unknown days those carry, the open comparison's
/// too (refcore's `_check`, its `CheckOut` values). L §5.18, §4.4.
fn sides_text(
    m: &Reality,
    engine: &mut Engine,
    (left, right): (&Expr, &Expr),
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
        let mut carried = carried_of(&[&lv, &rv]);
        if let Some(v) = open {
            v.carried_into(&mut carried);
        }
        Ok((text, carried))
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
    let text = format!(
        "differs: {lt} = {}, {rt} = {}{}",
        plain(m, a)?,
        plain(m, b)?,
        difference(m, a, b)?
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
/// for two dates, texts or codes; `, difference refused: …` where the subtraction is refused
/// (refcore's `_check`). L §5.18, §3.2.
fn difference(m: &Reality, a: &Value, b: &Value) -> Result<String, Stop> {
    let numbers = |v: &Value| matches!(v, Value::Number(_) | Value::Money(_));
    if numbers(a) && numbers(b) || keys_of_one_index(m, a, b) {
        return match arith(m, Op::Sub, a, b) {
            Ok(d) => Ok(format!(", difference {}", plain(m, &d)?)),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                Ok(format!(", difference refused: {}", r.text()))
            }
            Err(stop) => Err(stop),
        };
    }
    if matches!(a, Value::Key(KeyVal::Thing(_))) && matches!(b, Value::Key(KeyVal::Thing(_))) {
        // things of a kind: a kind is no index, so no difference (refcore's `_check`)
        return Ok(String::new());
    }
    if matches!(a, Value::Key(_) | Value::Period(_))
        && matches!(b, Value::Key(_) | Value::Period(_))
    {
        // two keys of one ordered index differ by an int (L §4.2); which index a key's
        // static type names isn't kept at run
        return Err(Stop::NotBuilt("a check's difference of two keys"));
    }
    Ok(String::new())
}

/// Whether `a` and `b` are keys of one list index, which a key carries at run: their
/// difference is the int of positions between them (L §4.2, refcore's `key_diff`).
fn keys_of_one_index(m: &Reality, a: &Value, b: &Value) -> bool {
    let (Value::Key(KeyVal::Key { index, .. }), Value::Key(KeyVal::Key { index: other, .. })) =
        (a, b)
    else {
        return false;
    };
    index == other && m.indexes.get(index).is_some()
}

/// A side as a check prints it in place of a decided comparison: a refusal as `refused: …`, a
/// key outside its index as its `RUN-KEY`, an interval that isn't a window of days as its
/// `HOLE-IMPRECISE`, under `partial` a hole in its linear form (tally 2.1.8 R15), any other
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
        Item::Value(Value::Window(w)) if !w.of_days() => {
            let window = w.question_text(m).map_err(num_stop)?;
            let text = format!("date within {window} not precise enough for a print");
            Ok(crate::hole::Hole::asked(&text, w.figure())
                .carrying(&w.carry)
                .text())
        }
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
