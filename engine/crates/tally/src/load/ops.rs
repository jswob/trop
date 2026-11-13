//! Load's checker, its operators: arithmetic and its unit algebra, date and key arithmetic, `if`, `and`, `or`, `not` and aggregates, each with refcore's refusals.
//! Spec: L §3.2 (units: a sum of like units, a product's unit; exchange rates apply from B to
//! A), §3.3 (rates), §2.1 (a four-digit figure reads as a year or an int), §4.2 (date and key
//! arithmetic, differences, keys as values), §2.5 (a bare key takes the other operand's
//! index), §5.11 (aggregates), §7.6 item 3 (a key offset's digits), §8.3 (`LOAD-UNIT`,
//! `LOAD-AMBIG`, `LOAD-KEY`, `LOAD-DOMAIN`, `LOAD-PARSE`, `LOAD-LIMIT`), §8.4.
//! Twin: engine/tools/refcore/check.py (`_arith`, `fx_check`, `year4_as_number`, `ambig`,
//! `arith_time`, `t_not`, `t_and` and `t_if` with their guards, `unify`, `key_digits`, `t_agg`,
//! `t_lastx`). The comparisons and membership (`t_cmp`, `kl_value`, `compare`, `t_in`) are
//! `compare.rs`'s.
//! Invariants: the checks run in refcore's order; `Unseen` on either side gives `Unseen` and
//! refuses nothing; a four-digit literal's reading is decided where refcore decides it, and
//! its literal rule waits for the checker's end as refcore's does.

use super::check::{Checker, Reading, Scope};
use super::defs::tested_vars;
use super::texts::covered_spans;
use super::types::{
    Op, Sides, Typed, converts, input_read, int_flex, is_bool, is_date, is_duration, key_pk,
    key_value_kind, no_order, num, numcls, refused, uerr, unit_error, unit_of,
};
use crate::base::{Code, Load};
use crate::eval::{num_stop, periods_of};
use crate::model::{IndexInfo, KeyVal, NameKind, PeriodWord, Reality, Rel, Shown, Ty, Unit};
use crate::num::Num;
use crate::syntax::{Agg, AggFn, Expr, Lit, Stop};
use crate::time::{PeriodKind, Step};

/// `LOAD-LIMIT <n> has more than 18 digits` for an integer written as a literal that names
/// keys: a key offset `k ± n`, a `pos(k)` bound (refcore's `key_digits`). L §7.6 item 3.
pub(super) fn key_digits(n: &crate::syntax::Number) -> Result<(), Stop> {
    if !n.whole || n.unit.is_some() {
        return Ok(());
    }
    let Some(text) = n
        .value
        .integer_text()
        .filter(|t| t.trim_start_matches('-').len() > 18)
    else {
        return Ok(());
    };
    Err(refused(
        Load::Limit,
        format!("{text} has more than 18 digits"),
    ))
}

/// A duration moves a key of a period index by whole keys: a literal one that isn't a whole
/// number of the index's step is refused where load sees both, `LOAD-DOMAIN` in run's words
/// `<k> <op> <d>: not a whole number of its periods (L §4.2)`; else the key's read refuses it
/// at run (refcore's `key_duration_whole`; tally 2.1.7 B5, #98, R5). L §4.1 `by n`, §4.2.
fn key_duration_whole(m: &Reality, ix: &str, op: Op, sides: &Sides) -> Result<(), Stop> {
    let (Some(Lit::Num(n)), Some(KeyVal::Period(first))) = (
        &sides.right.lit,
        m.indexes.get(ix).and_then(|index| index.keys().first()),
    ) else {
        return Ok(());
    };
    let Some([term]) = n.unit.as_ref().map(|unit| unit.terms.as_slice()) else {
        return Ok(());
    };
    let (family, months) = match term.base.as_str() {
        "day" => (PeriodKind::Day, 1),
        "month" => (PeriodKind::Month, 1),
        "quarter" => (PeriodKind::Month, 3),
        "year" => (PeriodKind::Month, 12),
        _ => return Ok(()),
    };
    if !n.whole || term.power != Num::int(1) {
        return Ok(());
    }
    let amount = n.value.checked_mul(&Num::int(months)).map_err(num_stop)?;
    let own = match first.step() {
        Step::Days(_) => PeriodKind::Day,
        Step::Months(_) => PeriodKind::Month,
    };
    if own != family || periods_of(first.step(), family, &amount)?.is_some() {
        return Ok(());
    }
    Err(refused(
        Load::Domain,
        format!(
            "{} {} {}: not a whole number of its periods (L §4.2)",
            sides.left.text,
            sign(op),
            sides.right.text
        ),
    ))
}

/// Whether `t` is `num %`.
fn is_pct(t: &Ty) -> bool {
    matches!(t, Ty::Num(unit, Shown::Percent) if unit.is_one())
}

/// The types no arithmetic takes (refcore's `_arith`, its list).
const NOT_NUMBERS: [&str; 10] = [
    "text", "code", "record", "list", "calendar", "dates", "keylit", "mmdd", "tuple", "dist",
];

/// `<a> <op> <b>` in a message.
fn shown(op: Op, lt: &Ty, rt: &Ty) -> String {
    format!("{} {} {}", lt.show(), sign(op), rt.show())
}

/// The operator as written.
fn sign(op: Op) -> &'static str {
    match op {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Mul => "*",
        Op::Div => "/",
    }
}

/// An exchange rate `A/B` in a unit: its currencies (refcore's `fx_check`, its `fx`).
fn fx(u: &Unit) -> Option<(String, String)> {
    let terms = u.terms();
    let currencies: Vec<&(String, i64)> = terms
        .iter()
        .filter(|(base, _)| crate::model::is_currency_code(base))
        .collect();
    if terms.len() != 2 || currencies.len() != 2 {
        return None;
    }
    let a = currencies.iter().find(|(_, e)| *e == 1)?;
    let b = currencies.iter().find(|(_, e)| *e == -1)?;
    Some((a.0.clone(), b.0.clone()))
}

/// `LOAD-UNIT` for an exchange rate `A/B` applied to anything but `B` (refcore's
/// `fx_check`). L §3.2 (U-05, C-16).
fn fx_check(xu: &Unit, ru: &Unit, op: Op) -> Result<(), Stop> {
    let wrong = |(a, b): (String, String)| {
        uerr(format!(
            "an exchange rate {a}/{b} applied to the wrong currency"
        ))
    };
    if op == Op::Mul {
        for (x, r) in [(xu, ru), (ru, xu)] {
            if let Some(rate) = fx(r)
                && x.terms().iter().any(|(base, e)| *base == rate.0 && *e > 0)
            {
                return Err(wrong(rate));
            }
        }
        return Ok(());
    }
    if let Some(rate) = fx(ru)
        && xu.terms().iter().any(|(base, e)| *base == rate.1 && *e > 0)
    {
        return Err(wrong(rate));
    }
    Ok(())
}

/// Whether a refusal is a load refusal, which `ambig` tries past.
fn is_load(stop: &Stop) -> bool {
    matches!(stop, Stop::Refused(r) if matches!(r.code(), Code::Load(_)))
}

impl Checker {
    /// An operator's type before its last check (refcore's `_arith`). L §3.2, §3.3, §4.2.
    pub(super) fn arith_core(
        &mut self,
        m: &Reality,
        scope: &Scope,
        op: Op,
        (lt, rt): (&Ty, &Ty),
        sides: &Sides,
    ) -> Typed {
        let mut lt = lt.clone();
        let mut rt = rt.clone();
        if matches!(lt, Ty::KeyList)
            && let Some(t) = self.kl_value(m, scope, &sides.left, Reading::Int, None)?
        {
            lt = t;
        }
        if matches!(rt, Ty::KeyList)
            && let Some(t) = self.kl_value(m, scope, &sides.right, Reading::Int, None)?
        {
            rt = t;
        }
        let (lk, rk) = (lt.word(), rt.word());
        if lk == "unseen" || rk == "unseen" {
            return Ok(Ty::Unseen);
        }
        if lk == "hole" || rk == "hole" {
            return Err(refused(
                Load::Parse,
                "cannot parse: a hole stands only as a whole value".to_owned(),
            ));
        }
        let nothing = |k: &str| k == "none" || k == "never";
        if nothing(lk) || nothing(rk) {
            let other = if nothing(lk) { &rt } else { &lt };
            return Ok(if nothing(other.word()) {
                Ty::Int
            } else {
                other.clone()
            });
        }
        if lk == "bool" || rk == "bool" {
            return Err(uerr("bool in arithmetic".to_owned()));
        }
        if lk == "dyn" || rk == "dyn" {
            let other = if lk == "dyn" { &rt } else { &lt };
            if numcls(m, other).is_some() {
                return Ok(Ty::Dyn);
            }
            return Err(uerr(format!(
                "{} and {} in arithmetic",
                lt.show(),
                rt.show()
            )));
        }
        if lk == "year4" || rk == "year4" {
            return self.ambig(m, scope, op, (&lt, &rt), sides);
        }
        let timed = |k: &str| k == "date" || k == "period" || k == "key";
        if timed(lk) || timed(rk) {
            return self.arith_time(m, scope, op, (&lt, &rt), sides);
        }
        if NOT_NUMBERS.contains(&lk) || NOT_NUMBERS.contains(&rk) {
            if lk == "dist" || rk == "dist" {
                return Err(uerr(format!(
                    "{} and {} in arithmetic: a distribution value is no number (L §5.16 The closed list)",
                    lt.show(),
                    rt.show()
                )));
            }
            return Err(uerr(format!(
                "{} and {} in arithmetic",
                lt.show(),
                rt.show()
            )));
        }
        if op == Op::Add || op == Op::Sub {
            return additive(op, &lt, &rt);
        }
        multiplicative(op, lt, rt)
    }

    /// A four-digit literal in arithmetic: a year or an int, whichever reading the operator
    /// takes; both is `LOAD-AMBIG` (refcore's `ambig`). L §2.1, §7.1.
    fn ambig(
        &mut self,
        m: &Reality,
        scope: &Scope,
        op: Op,
        (lt, rt): (&Ty, &Ty),
        sides: &Sides,
    ) -> Typed {
        let mut readings = Vec::new();
        for as_year in [true, false] {
            let read = |t: &Ty| {
                if !matches!(t, Ty::Year4) {
                    return t.clone();
                }
                if as_year {
                    Ty::Period(PeriodWord::Year)
                } else {
                    Ty::Int
                }
            };
            match self.arith(m, scope, op, (&read(lt), &read(rt)), sides) {
                Ok(t) => readings.push((as_year, t)),
                Err(stop) if is_load(&stop) => {}
                Err(stop) => return Err(stop),
            }
        }
        if readings.len() == 2 {
            return Err(refused(
                Load::Ambig,
                format!("{} reads as a year and as an integer here", sides.text),
            ));
        }
        let Some((as_year, t)) = readings.into_iter().next() else {
            for side in [&sides.left, &sides.right] {
                self.year4_now(side.year4, Reading::Int)?;
            }
            return Err(uerr(format!(
                "{} and {} in arithmetic",
                lt.show(),
                rt.show()
            )));
        };
        let reading = if as_year { Reading::Year } else { Reading::Int };
        for side in [&sides.left, &sides.right] {
            self.year4_now(side.year4, reading)?;
        }
        Ok(t)
    }

    /// Date, period and key arithmetic (refcore's `arith_time`). L §4.2.
    fn arith_time(
        &mut self,
        m: &Reality,
        scope: &Scope,
        op: Op,
        (lt, rt): (&Ty, &Ty),
        sides: &Sides,
    ) -> Typed {
        let (lk, rk) = (lt.word(), rt.word());
        let intlike = |k: &str| k == "int" || k == "flex";
        let additive = op == Op::Add || op == Op::Sub;
        if let Ty::Key(ix) = lt {
            let vk = key_value_kind(m, lt);
            let rint = key_value_kind(m, rt) == Some("int");
            if additive && (intlike(rk) || (rk == "key" && rint && vk != Some("int"))) {
                if m.kinds.get(ix).is_some() {
                    return Err(uerr(format!("key arithmetic on things of kind {ix}")));
                }
                if m.names.kind(ix) == Some(NameKind::Axis) {
                    return Err(uerr("key arithmetic on an axis".to_owned()));
                }
                no_order(m, lt, "key arithmetic")?;
                if let Some(Lit::Num(offset)) = &sides.right.lit {
                    key_digits(offset)?;
                }
                return Ok(lt.clone());
            }
            if op == Op::Sub && matches!(rt, Ty::Key(other) if other == ix) {
                no_order(m, lt, "key arithmetic")?;
                return Ok(Ty::Int);
            }
            if vk == Some("int") && ["int", "flex", "num", "money", "rate"].contains(&rk) {
                return self.arith(m, scope, op, (&Ty::Int, rt), sides);
            }
            if vk == Some("int") && rk == "key" && rint {
                return self.arith(m, scope, op, (&Ty::Int, &Ty::Int), sides);
            }
            if vk == Some("date") {
                return self.arith_time(m, scope, op, (&Ty::Date, rt), sides);
            }
            if vk == Some("period")
                && let Some(pk) = key_pk(m, lt)
            {
                key_duration_whole(m, ix, op, sides)?;
                return self.arith_time(m, scope, op, (&period_ty(pk), rt), sides);
            }
            return Err(uerr(shown(op, lt, rt)));
        }
        if rk == "key" && op == Op::Add && intlike(lk) && key_value_kind(m, rt) != Some("int") {
            return Err(uerr(format!("{}: write the key first", shown(op, lt, rt))));
        }
        if rk == "key" {
            return match key_value_kind(m, rt) {
                Some("int") => self.arith(m, scope, op, (lt, &Ty::Int), sides),
                Some("date") => self.arith_time(m, scope, op, (lt, &Ty::Date), sides),
                Some("period") => {
                    let pk = key_pk(m, rt).map_or(Ty::Unseen, period_ty);
                    self.arith_time(m, scope, op, (lt, &pk), sides)
                }
                Some(_) | None => Err(uerr(shown(op, lt, rt))),
            };
        }
        if (lk == "date" || lk == "period") && !additive {
            return Err(uerr(shown(op, lt, rt)));
        }
        if lk == "date" {
            if intlike(rk) || is_duration(rt) {
                return Ok(Ty::Date);
            }
            if op == Op::Sub && rk == "date" {
                return Err(uerr(
                    "date - date has no meaning; days_between(a, b) gives b - a in days".to_owned(),
                ));
            }
            return Err(uerr(shown(op, lt, rt)));
        }
        if let Ty::Period(word) = lt {
            if intlike(rk) {
                return Ok(lt.clone());
            }
            if op == Op::Sub
                && *word != PeriodWord::Day
                && matches!(rt, Ty::Period(other) if other == word)
            {
                return Ok(Ty::Int);
            }
            // a duration of its own family: days for a period of days, months, quarters or
            // years for the others; whether the count is whole shows at run (L §4.2, §8.4)
            if let Ty::Num(unit, _) = rt
                && is_duration(rt)
                && let [(du, _)] = unit.terms()
                && (du == "day") == (*word == PeriodWord::Day)
            {
                return Ok(lt.clone());
            }
            return Err(uerr(format!(
                "{} (a period moves by an int of its periods, or by a duration that is a whole number of them)",
                shown(op, lt, rt)
            )));
        }
        Err(uerr(shown(op, lt, rt)))
    }

    /// `not c` (refcore's `t_not`). L §2.3.
    pub(super) fn not(&mut self, m: &Reality, scope: &Scope, inner: &Expr, depth: u32) -> Typed {
        let t = self.expr(m, scope, inner, None, depth)?;
        if !is_bool(&t) {
            return Err(uerr(format!("not of {}", t.show())));
        }
        Ok(Ty::Bool)
    }

    /// A chain of `and` or `or`, nested to the left (refcore's `t_and`, `t_or`). L §2.3.
    pub(super) fn junction(
        &mut self,
        m: &Reality,
        scope: &Scope,
        terms: &[Expr],
        depth: u32,
    ) -> Typed {
        let Some((first, rest)) = terms.split_first() else {
            return Ok(Ty::Bool);
        };
        let mut lt = self.expr(m, scope, first, None, depth)?;
        let mut right = scope.clone();
        right.guards.extend(tested_vars(first, &scope.keyvars));
        for term in rest {
            let rt = self.expr(m, &right, term, None, depth)?;
            right.guards.extend(tested_vars(term, &scope.keyvars));
            if !is_bool(&lt) || !is_bool(&rt) {
                return Err(uerr(format!("and of {} and {}", lt.show(), rt.show())));
            }
            lt = Ty::Bool;
        }
        Ok(lt)
    }

    /// `if c then a else b`: a bool condition, branches of one type, a bare key or figure
    /// taking the other branch's (refcore's `t_if`). L §2.3, §3.2, §5.19.
    pub(super) fn conditional(
        &mut self,
        m: &Reality,
        scope: &Scope,
        parts: &[Expr; 3],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let [c, a, b] = parts;
        // a kind test of the condition narrows throughout it and the `then` branch (L §5.7)
        let narrowed = scope.narrowed(m, Some(c));
        let ct = self.expr(m, &narrowed, c, None, depth)?;
        if !is_bool(&ct) {
            return Err(uerr(format!("if of {}", ct.show())));
        }
        let mut yes = narrowed;
        let mut no = scope.clone();
        for branch in [&mut yes, &mut no] {
            covered_spans(Some(c), &mut branch.covered);
            branch.guards.extend(tested_vars(c, &scope.keyvars));
        }
        let mut at = self.expr(m, &yes, a, expect, depth)?;
        let mut bt = self.expr(m, &no, b, expect, depth)?;
        let open = |t: &Ty| matches!(t, Ty::KeyLit(_) | Ty::Flex(_) | Ty::Year4);
        if expect.is_none() {
            if open(&at) && !open(&bt) && !matches!(bt, Ty::None | Ty::Hole) {
                at = self.expr(m, &yes, a, Some(&bt), depth)?;
            }
            if open(&bt) && !open(&at) && !matches!(at, Ty::None | Ty::Hole) {
                bt = self.expr(m, &no, b, Some(&at), depth)?;
            }
        }
        unify(m, &at, &bt)
    }

    /// An aggregate: its bindings, its term and its ties, under the bindings (refcore's
    /// `t_agg`). L §5.11.
    pub(super) fn aggregate(&mut self, m: &Reality, scope: &Scope, agg: &Agg, depth: u32) -> Typed {
        let mut inner = scope.in_aggregate();
        let mut tys = Vec::new();
        for bind in &agg.binds {
            tys.push(self.bind(m, &mut inner, bind, depth)?);
        }
        let ordered = matches!(agg.func, AggFn::First | AggFn::Last);
        if ordered && agg.ties.is_none() {
            for bind in &agg.binds {
                if let Some(what) = unordered_binding(m, &inner, &bind.var) {
                    return Err(uerr(format!(
                        "{} over {what} has no ties by",
                        agg.func.word()
                    )));
                }
            }
        }
        let Some(term) = &agg.term else {
            return self.bare_aggregate(m, &inner, agg, &tys, depth);
        };
        if ordered {
            return Err(refused(
                Load::Parse,
                format!(
                    "cannot parse: {} takes no term (a bare binding)",
                    agg.func.word()
                ),
            ));
        }
        let tt = self.expr(m, &inner, term, None, depth)?;
        if let Some(ties) = &agg.ties {
            for tie in ties {
                self.expr(m, &inner, tie, None, depth)?;
            }
        }
        aggregate_type(m, agg.func, &tt, &tys)
    }

    /// An aggregate with no term: `count`, or `first`, `last`, `min`, `max` of the binding
    /// itself (refcore's `t_agg`, its bare arm). L §5.11.
    fn bare_aggregate(
        &mut self,
        m: &Reality,
        inner: &Scope,
        agg: &Agg,
        tys: &[Ty],
        depth: u32,
    ) -> Typed {
        let word = agg.func.word();
        match agg.func {
            AggFn::Count => return Ok(Ty::Int),
            AggFn::First | AggFn::Last | AggFn::Min | AggFn::Max => {}
            AggFn::Sum | AggFn::Any | AggFn::All | AggFn::Argmin | AggFn::Argmax => {
                return Err(refused(
                    Load::Parse,
                    format!("cannot parse: {word} takes a term"),
                ));
            }
        }
        let min_max = matches!(agg.func, AggFn::Min | AggFn::Max);
        if min_max && tys.len() != 1 {
            return Err(refused(
                Load::Domain,
                format!("{word} over a bare binding of several variables"),
            ));
        }
        if min_max && let Some(t) = tys.first() {
            no_order(m, t, word)?;
        }
        if let Some(ties) = &agg.ties {
            for tie in ties {
                self.expr(m, inner, tie, None, depth)?;
            }
        }
        Ok(binding_type(tys))
    }

    /// `last(x[k], d)`: a read of an input at a day the reader states (refcore's `t_lastx`).
    /// L §4.6.
    pub(super) fn last(
        &mut self,
        m: &Reality,
        scope: &Scope,
        read: &Expr,
        day: &Expr,
        depth: u32,
    ) -> Typed {
        let t = self.expr(m, &scope.with_mode(Rel::Explicit), read, None, depth)?;
        if input_read(m, scope, read).is_none() {
            return Err(refused(
                Load::Parse,
                "cannot parse: last(x[k], d) takes a read of an input".to_owned(),
            ));
        }
        self.date_arg(m, scope, day, "last", "the date", depth)?;
        Ok(t)
    }
}

/// `a + b`, `a - b` of numbers (refcore's `_arith`, its `+ -` arm). L §3.2, §3.3.
fn additive(op: Op, lt: &Ty, rt: &Ty) -> Typed {
    let (lk, rk) = (lt.word(), rt.word());
    if lk == "flex" && rk == "flex" {
        return Ok(if int_flex(lt) && int_flex(rt) {
            Ty::Int
        } else {
            num(Unit::one())
        });
    }
    if lk == "flex" {
        return Ok(rt.clone());
    }
    if rk == "flex" {
        return Ok(lt.clone());
    }
    if is_pct(lt) && is_pct(rt) {
        return Ok(Ty::Num(Unit::one(), Shown::Percent));
    }
    if lk == "rate" || rk == "rate" {
        if lk == rk {
            return Ok(Ty::Rate);
        }
        return Err(uerr(format!(
            "{}: rate arithmetic outside L §3.3",
            shown(op, lt, rt)
        )));
    }
    if lk == "int" && rk == "int" {
        return Ok(Ty::Int);
    }
    if let (Ty::Money(a), Ty::Money(b)) = (lt, rt) {
        if a != b {
            return Err(uerr(format!(
                "{} and {} have different units (money in two currencies)",
                lt.show(),
                rt.show()
            )));
        }
        return Ok(lt.clone());
    }
    if lk == "money" || rk == "money" {
        let (money, other) = if lk == "money" { (lt, rt) } else { (rt, lt) };
        let mu = unit_of(money)?;
        if let Ty::Num(ou, _) = other
            && converts(ou, &mu)?
        {
            return Ok(num(mu));
        }
        return Err(uerr(format!(
            "{} and {} have different units",
            lt.show(),
            rt.show()
        )));
    }
    let lu = if lk == "num" {
        unit_of(lt)?
    } else {
        Unit::one()
    };
    let ru = if rk == "num" {
        unit_of(rt)?
    } else {
        Unit::one()
    };
    if !converts(&ru, &lu)? {
        return Err(uerr(format!(
            "{} and {} have different units",
            lt.show(),
            rt.show()
        )));
    }
    Ok(num(lu))
}

/// `a * b`, `a / b` of numbers: rates, exchange rates and the product's unit (refcore's
/// `_arith`, its `* /` arm). L §3.2, §3.3.
fn multiplicative(op: Op, lt: Ty, rt: Ty) -> Typed {
    let bare = |t: Ty| {
        if matches!(t, Ty::Flex(_)) {
            if int_flex(&t) {
                Ty::Int
            } else {
                num(Unit::one())
            }
        } else {
            t
        }
    };
    let lt = bare(lt);
    let rt = bare(rt);
    let (lk, rk) = (lt.word(), rt.word());
    let plain = |t: &Ty| matches!(t, Ty::Num(unit, _) if unit.is_one());
    if lk == "rate" || rk == "rate" {
        let scaled = (lk == "rate" && (rk == "int" || plain(&rt)))
            || (rk == "rate" && (lk == "int" || plain(&lt)));
        if op == Op::Mul && scaled {
            return Ok(Ty::Rate);
        }
        if op == Op::Mul {
            let (duration, ok) = if lk == "rate" {
                (&rt, rk == "num")
            } else {
                (&lt, lk == "num")
            };
            if ok && is_duration(duration) {
                return Ok(num(Unit::one()));
            }
        }
        return Err(uerr(format!(
            "{}: rate arithmetic outside L §3.3",
            shown(op, &lt, &rt)
        )));
    }
    let measured = |k: &str| k == "num" || k == "money";
    let lu = if measured(lk) {
        unit_of(&lt)?
    } else {
        Unit::one()
    };
    let ru = if measured(rk) {
        unit_of(&rt)?
    } else {
        Unit::one()
    };
    fx_check(&lu, &ru, op)?;
    if op == Op::Mul {
        if lk == "int" && rk == "int" {
            return Ok(Ty::Int);
        }
        if lk == "money" && rk == "int" {
            return Ok(lt);
        }
        if rk == "money" && lk == "int" {
            return Ok(rt);
        }
        let (u, _) = lu.mul(&ru).map_err(unit_error)?;
        return Ok(num(u));
    }
    let (u, _) = lu.div(&ru).map_err(unit_error)?;
    Ok(num(u))
}

/// A period kind as a type (refcore's `T.period`): a key of a day index `by n` is a period
/// of days. L §4.1 `by n`, §4.2.
fn period_ty(pk: PeriodKind) -> Ty {
    match pk {
        PeriodKind::Month => Ty::Period(PeriodWord::Month),
        PeriodKind::Quarter => Ty::Period(PeriodWord::Quarter),
        PeriodKind::Year => Ty::Period(PeriodWord::Year),
        PeriodKind::Day => Ty::Period(PeriodWord::Day),
    }
}

/// The type of two branches, or of a value and its counterpart (refcore's `unify`).
/// L §3.2, §3.1.
pub(super) fn unify(m: &Reality, a: &Ty, b: &Ty) -> Typed {
    if matches!(a, Ty::None | Ty::Hole) {
        return Ok(b.clone());
    }
    if matches!(b, Ty::None | Ty::Hole) {
        return Ok(a.clone());
    }
    if matches!(a, Ty::Unseen) || matches!(b, Ty::Unseen) {
        return Ok(Ty::Unseen);
    }
    if a == b {
        return Ok(a.clone());
    }
    if let (Some(ka), Some(kb)) = (numcls(m, a), numcls(m, b)) {
        return unify_numbers(a, b, ka, kb);
    }
    if (matches!(a, Ty::Never) && matches!(b, Ty::Date))
        || (matches!(b, Ty::Never) && matches!(a, Ty::Date))
    {
        return Ok(Ty::Date);
    }
    if let (Ty::Key(x), Ty::Key(y)) = (a, b)
        && m.kinds.get(x).is_some()
        && m.kinds.get(y).is_some()
    {
        return Ok(a.clone());
    }
    if is_date(m, a) && is_date(m, b) {
        return Ok(Ty::Date);
    }
    Err(uerr(format!(
        "{} and {} have different types",
        a.show(),
        b.show()
    )))
}

/// Two numbers unified (refcore's `unify`, its number arm).
fn unify_numbers(a: &Ty, b: &Ty, ka: &str, kb: &str) -> Typed {
    if ka == "dyn" || kb == "dyn" {
        return Ok(Ty::Dyn);
    }
    if ka == "flex" {
        return Ok(b.clone());
    }
    if kb == "flex" {
        return Ok(a.clone());
    }
    if ka == "int" && kb == "int" {
        return Ok(Ty::Int);
    }
    let differ = || {
        uerr(format!(
            "{} and {} have different units",
            a.show(),
            b.show()
        ))
    };
    if let (Ty::Money(x), Ty::Money(y)) = (a, b) {
        return if x == y { Ok(a.clone()) } else { Err(differ()) };
    }
    if ka == "rate" || kb == "rate" {
        return if ka == kb {
            Ok(a.clone())
        } else {
            Err(differ())
        };
    }
    let measured = |k: &str| k == "num" || k == "money";
    let au = if measured(ka) {
        unit_of(a)?
    } else {
        Unit::one()
    };
    let bu = if measured(kb) {
        unit_of(b)?
    } else {
        Unit::one()
    };
    if !converts(&au, &bu)? {
        return Err(differ());
    }
    Ok(num(au))
}

/// What a binding ranges over when it has no order a rule can read: things, movements or
/// an open index's keys (refcore's `t_agg`, its ties check). L §5.11.
fn unordered_binding(m: &Reality, scope: &Scope, var: &str) -> Option<&'static str> {
    let var = scope.vars.get(var)?;
    match &var.dom {
        super::check::VarDom::Kind(_) => Some("things"),
        super::check::VarDom::Event(_) | super::check::VarDom::Obs(_) => Some("movements"),
        super::check::VarDom::Index(ix) => m
            .indexes
            .get(ix)
            .filter(|index: &&IndexInfo| index.form == crate::model::IndexForm::Open)
            .map(|_| "open-index keys"),
        super::check::VarDom::Axis(_)
        | super::check::VarDom::List
        | super::check::VarDom::Range
        | super::check::VarDom::Arg => None,
    }
}

/// The key or the tuple of keys a bare binding gives.
fn binding_type(tys: &[Ty]) -> Ty {
    match tys {
        [one] => one.clone(),
        many => Ty::Tuple(many.to_vec()),
    }
}

/// An aggregate's type from its term's (refcore's `t_agg`, its term arms). L §5.11.
fn aggregate_type(m: &Reality, func: AggFn, tt: &Ty, tys: &[Ty]) -> Typed {
    if matches!(tt, Ty::Unseen) {
        return Ok(match func {
            AggFn::Count | AggFn::Any | AggFn::All => Ty::Bool,
            AggFn::Sum
            | AggFn::Min
            | AggFn::Max
            | AggFn::First
            | AggFn::Last
            | AggFn::Argmin
            | AggFn::Argmax => Ty::Unseen,
        })
        .map(|t| if func == AggFn::Count { Ty::Int } else { t });
    }
    let word = func.word();
    match func {
        AggFn::Sum => match numcls(m, tt) {
            None => Err(uerr(format!("sum of {}", tt.show()))),
            Some("flex" | "year4") => Ok(Ty::Int),
            Some(_) if matches!(tt, Ty::Key(_)) => Ok(Ty::Int),
            Some(_) => Ok(tt.clone()),
        },
        AggFn::Count => {
            if !is_bool(tt) {
                return Err(uerr(format!("count of {}", tt.show())));
            }
            Ok(Ty::Int)
        }
        AggFn::Min | AggFn::Max => {
            no_order(m, tt, word)?;
            Ok(if matches!(tt, Ty::Flex(_)) {
                Ty::Int
            } else {
                tt.clone()
            })
        }
        AggFn::Argmin | AggFn::Argmax => {
            no_order(m, tt, word)?;
            Ok(binding_type(tys))
        }
        AggFn::Any | AggFn::All => {
            if !is_bool(tt) {
                return Err(uerr(format!("{word} of {}", tt.show())));
            }
            Ok(Ty::Bool)
        }
        AggFn::First | AggFn::Last => Err(refused(
            Load::Parse,
            format!("cannot parse: {word} takes a bare binding"),
        )),
    }
}
