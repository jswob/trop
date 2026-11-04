//! Load's checker, its calls: the built-in functions' argument types and results, band calls, and the reading forms whose types are M0's later groups' (walked for their reads, typed `Unseen`).
//! Spec: L §4.2 (the date and calendar functions), §5.9 (bands, `band_lo`, `band_hi`,
//! `marginal`), §5.11 (`missing`, `min`, `max`), §7.2 (`round`, its step and mode), §4.6
//! (`since`), §5.14.1 (`next`, running reads), §5.19 (`observed`, `gap`, `covered`), §5.10
//! (`bal`, `value`, prices), §8.3 (`LOAD-UNIT`, `-PARSE`, `-NAME`, `-KEY`, `-LITERAL`,
//! `-MISSING`, `-BAND`), §8.4.
//! Twin: engine/tools/refcore/check.py (`t_call`, `args`, `want`, `tdate`,
//! `tdate_or_period`, `tint`, `c_start_of` to `c_since`, `cal_arg`, `band_ref`, `c_band_lo`,
//! `c_marginal`, `band_call`); check3.py (`m_t_bal`, `m_t_flows`, `m_t_value`, `m_c_next`,
//! `m_c_covered`, `m_c_gap`, `m_c_observed`, `m_legof_read`), their reads.
//! Invariants: every argument is typed in refcore's order and its reads recorded; a call
//! whose checks are a later group's (`exp`, `pow`, `periodic`, `unit`, `lifespan`, …)
//! types its arguments and is `Unseen`, refusing nothing of its own.

use super::check::{
    Checker, Reading, Scope, Stands, input_or_def, move_keys, named_parts, place_target,
    price_inputs,
};
use super::ops::unify;
use super::types::{
    Typed, input_read, is_date, is_period, no_order, numcls, period_kind, refused, text, uerr,
    unit_of,
};
use crate::base::Load;
use crate::model::{
    CodeKind, Mark, NameKind, Node, NodeKind, PeriodWord, Reality, Rel, Shown, Target, Ty, Unit,
};
use crate::syntax::{
    Bal, Call, Expr, Flows, Key, Lit, OfQ, Postfix, Price, Span, Step, Stop, Valuation,
};
use crate::time::PeriodKind;

/// The date functions whose names no band's call shadows (refcore's `DATE_FNS`).
const DATE_FNS: [&str; 21] = [
    "start_of",
    "end_of",
    "year_of",
    "quarter_of",
    "month_of",
    "day_of",
    "days_between",
    "add_days",
    "add_months",
    "add_years",
    "overlap_days",
    "age",
    "age_months",
    "weekday",
    "pos",
    "is_business",
    "add_business",
    "last_business",
    "schedule",
    "window",
    "one_of",
];

/// The built-ins a call may name (refcore's `parser.BUILTINS`, less those parsed as forms of
/// their own: the aggregates, the queries, `prorata`, `flows`, `bal`, `value`, `low`,
/// `covered`, the distributions and their functions).
const BUILTINS: [&str; 46] = [
    "next",
    "round",
    "pow",
    "exp",
    "ln",
    "sqrt",
    "abs",
    "periodic",
    "factor",
    "dcf",
    "accrue",
    "days_between",
    "add_days",
    "add_months",
    "add_years",
    "start_of",
    "end_of",
    "year_of",
    "quarter_of",
    "month_of",
    "day_of",
    "overlap_days",
    "pos",
    "age",
    "age_months",
    "weekday",
    "add_business",
    "last_business",
    "is_business",
    "schedule",
    "calendar",
    "window",
    "one_of",
    "since",
    "band_lo",
    "band_hi",
    "marginal",
    "unit",
    "minor",
    "lifespan",
    "gap",
    "observed",
    "missing",
    "min",
    "max",
    "covered",
];

/// `num u`.
fn num(unit: Unit) -> Ty {
    Ty::Num(unit, Shown::Plain)
}

/// `LOAD-PARSE` for a built-in given the wrong number of arguments (refcore's `args`).
fn arity(call: &Call, n: usize) -> Result<(), Stop> {
    if call.args.len() != n {
        return Err(refused(
            Load::Parse,
            format!("cannot parse: {} takes {n} arguments", call.name),
        ));
    }
    Ok(())
}

impl Checker {
    /// A call: a band's, or a built-in's (refcore's `t_call`). L §5.9, §7.2, §4.2.
    pub(super) fn call(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        if !call.prefix.is_empty() {
            return Err(Stop::NotBuilt("a member's band"));
        }
        let named_band = !DATE_FNS.contains(&call.name.as_str())
            && m.names.get(&call.name) == Some(&NameKind::Band);
        if call.band_key.is_some() || named_band {
            return self.band_call(m, scope, call, depth);
        }
        if !BUILTINS.contains(&call.name.as_str()) {
            if m.names.contains_key(&call.name) {
                return Err(refused(
                    Load::Parse,
                    format!("cannot parse: {} is not called", call.name),
                ));
            }
            return Err(refused(Load::Name, format!("unknown id {}", call.name)));
        }
        self.builtin(m, scope, call, expect, depth)
    }

    /// A built-in, one arm per family (refcore's `c_*`).
    fn builtin(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let args = call.args.as_slice();
        match call.name.as_str() {
            "start_of" | "end_of" => self.of_period(m, scope, call, Ty::Date, depth),
            "year_of" => self.of_period(m, scope, call, Ty::Period(PeriodWord::Year), depth),
            "quarter_of" => {
                self.of_period(m, scope, call, Ty::Period(PeriodWord::Quarter), depth)
            }
            "month_of" => self.of_period(m, scope, call, Ty::Period(PeriodWord::Month), depth),
            "day_of" => self.day_of(m, scope, call, depth),
            "days_between" | "age" | "age_months" => {
                self.dates(m, scope, call, 2, depth)?;
                Ok(Ty::Int)
            }
            "window" => {
                self.dates(m, scope, call, 2, depth)?;
                Ok(Ty::Date)
            }
            "weekday" => {
                self.dates(m, scope, call, 1, depth)?;
                Ok(Ty::Int)
            }
            "add_days" => self.add_to_date(m, scope, call, "day", depth),
            "add_months" => self.add_to_date(m, scope, call, "month", depth),
            "add_years" => self.add_to_date(m, scope, call, "year", depth),
            "overlap_days" => self.overlap_days(m, scope, call, depth),
            "pos" => self.pos(m, scope, call, depth),
            "one_of" => self.one_of(m, scope, call, depth),
            "is_business" => self.is_business(m, scope, call, depth),
            "add_business" | "last_business" => self.business(m, scope, call, depth),
            "round" => self.round(m, scope, call, depth),
            "minor" => minor(m, call),
            "abs" => self.abs(m, scope, call, depth),
            "min" | "max" => self.min_max(m, scope, call, expect, depth),
            "missing" => self.missing(m, scope, call, depth),
            "since" => self.since(m, scope, call, depth),
            "calendar" => Err(refused(
                Load::Parse,
                "cannot parse: calendar(...) is a row value (L §2.3 value), not an atom of an expression"
                    .to_owned(),
            )),
            "band_lo" | "band_hi" => self.band_edge(m, scope, call, depth),
            "marginal" => self.marginal(m, scope, call, depth),
            "next" => self.next(m, scope, args, depth),
            "observed" => self.observed(m, scope, args, depth),
            "gap" => self.gap(m, scope, args, depth),
            "schedule" => self.calendar_call(m, scope, args, 3, depth),
            _ => self.args_only(m, scope, args, depth),
        }
    }

    /// Each argument typed, the call `Unseen` (a built-in a later group checks); a bare name
    /// that is no value (a period word such as `month`, an index, a kind) is that built-in's
    /// own argument, read by its checks, not here (refcore's `period_arg`, `code_arg`).
    fn args_only(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        for arg in args {
            if let Expr::Name(id) = arg
                && !scope.is_var(id)
                && m.names.get(id).is_some_and(|kind| !is_value(*kind))
            {
                continue;
            }
            self.expr(m, scope, arg, None, depth)?;
        }
        Ok(Ty::Unseen)
    }

    /// `LOAD-UNIT` unless `t` is a date (refcore's `tdate`, `want`). L §4.2.
    pub(super) fn date_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        name: &str,
        what: &str,
        depth: u32,
    ) -> Typed {
        let t = self.expr(m, scope, e, None, depth)?;
        if !is_date(m, &t) {
            return Err(uerr(format!("{name}: {what} is {}", t.show())));
        }
        Ok(t)
    }

    /// A date or a period; a four-digit figure is a year (refcore's `tdate_or_period`).
    fn date_or_period(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        name: &str,
        depth: u32,
    ) -> Typed {
        let side = self.operand(m, scope, e, None, depth)?;
        let mut t = side.ty;
        if matches!(t, Ty::Year4) {
            self.decide(side.year4, Reading::Year);
            t = Ty::Period(PeriodWord::Year);
        }
        if !is_date(m, &t) && !is_period(m, &t) {
            return Err(uerr(format!("{name}: the argument is {}", t.show())));
        }
        Ok(t)
    }

    /// A count: an int, a bare figure or a plain num; the duration it may stand for
    /// (refcore's `tint`). L §4.2.
    fn count_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        name: &str,
        duration: Option<&str>,
        depth: u32,
    ) -> Typed {
        let t = self.expr(m, scope, e, Some(&Ty::Int), depth)?;
        if let (Some(word), Ty::Num(unit, _)) = (duration, &t)
            && matches!(unit.terms(), [(base, 1)] if base == word)
        {
            return Ok(t);
        }
        let plain = matches!(&t, Ty::Num(unit, _) if unit.is_one());
        let counted = matches!(numcls(m, &t), Some("int" | "flex")) || plain;
        if !counted && !matches!(t, Ty::Unseen) {
            return Err(uerr(format!("{name}: the count is {}", t.show())));
        }
        Ok(t)
    }

    /// `start_of`, `end_of`, `year_of`, `quarter_of`, `month_of` (refcore's `c_start_of`,
    /// `c_year_of`, …). L §4.2.
    fn of_period(&mut self, m: &Reality, scope: &Scope, call: &Call, out: Ty, depth: u32) -> Typed {
        arity(call, 1)?;
        if let Some(arg) = call.args.first() {
            self.date_or_period(m, scope, arg, &call.name, depth)?;
        }
        Ok(out)
    }

    /// `day_of(m, n)` or `day_of(y, md)` (refcore's `c_day_of`). L §4.2.
    fn day_of(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 2)?;
        let [period, second] = call.args.as_slice() else {
            return Ok(Ty::Date);
        };
        let t = self.date_or_period(m, scope, period, &call.name, depth)?;
        if matches!(t, Ty::Unseen) {
            self.expr(m, scope, second, None, depth)?;
            return Ok(Ty::Date);
        }
        match period_kind(m, &t) {
            Some(PeriodKind::Month) => {
                self.count_arg(m, scope, second, &call.name, None, depth)?;
            }
            Some(PeriodKind::Year) => {
                let t2 = self.expr(m, scope, second, Some(&Ty::MonthDay), depth)?;
                if !matches!(t2, Ty::MonthDay | Ty::Unseen) {
                    return Err(uerr(format!(
                        "day_of(y, md): md is {}, not a monthday",
                        t2.show()
                    )));
                }
            }
            Some(PeriodKind::Day | PeriodKind::Quarter) | None => {
                return Err(uerr(format!(
                    "day_of takes a month or a year, not {}",
                    t.show()
                )));
            }
        }
        Ok(Ty::Date)
    }

    /// `n` date arguments (refcore's `tdate` each).
    fn dates(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        n: usize,
        depth: u32,
    ) -> Result<(), Stop> {
        arity(call, n)?;
        for arg in &call.args {
            self.date_arg(m, scope, arg, &call.name, "the date", depth)?;
        }
        Ok(())
    }

    /// `add_days`, `add_months`, `add_years` (refcore's `c_add_days`, …). L §4.2.
    fn add_to_date(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        word: &str,
        depth: u32,
    ) -> Typed {
        arity(call, 2)?;
        if let [day, count] = call.args.as_slice() {
            self.date_arg(m, scope, day, &call.name, "the date", depth)?;
            self.count_arg(m, scope, count, &call.name, Some(word), depth)?;
        }
        Ok(Ty::Date)
    }

    /// `overlap_days(a, b, p)` (refcore's `c_overlap_days`). L §4.2.
    fn overlap_days(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 3)?;
        if let [a, b, p] = call.args.as_slice() {
            self.date_arg(m, scope, a, &call.name, "the date", depth)?;
            self.date_arg(m, scope, b, &call.name, "the date", depth)?;
            self.date_or_period(m, scope, p, &call.name, depth)?;
        }
        Ok(Ty::Int)
    }

    /// `pos(k)`: a key of a list or range index (refcore's `c_pos`). L §4.2, §5.6.
    fn pos(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 1)?;
        let Some(arg) = call.args.first() else {
            return Ok(Ty::Int);
        };
        let mut t = self.expr(m, scope, arg, None, depth)?;
        if let Ty::KeyLit(written) = &t {
            let hits: Vec<&String> = m
                .indexes
                .entries()
                .filter(|(_, ix)| ix.by_text(written).is_some())
                .map(|(id, _)| id)
                .collect();
            let [hit] = hits.as_slice() else {
                return Err(refused(Load::Key, format!("ambiguous key {written}")));
            };
            t = self.expr(m, scope, arg, Some(&Ty::Key((*hit).clone())), depth)?;
        }
        if matches!(t, Ty::Unseen) {
            return Ok(Ty::Int);
        }
        let Ty::Key(ix) = &t else {
            return Err(uerr(format!("pos of {}, not a key", t.show())));
        };
        if m.kinds.get(ix).is_some() {
            return Err(uerr(format!("pos of a thing of kind {ix}")));
        }
        no_order(m, &t, "pos")?;
        Ok(Ty::Int)
    }

    /// `one_of(list)` (refcore's `c_one_of`). L §4.2.
    fn one_of(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 1)?;
        if let Some(arg) = call.args.first() {
            let t = self.expr(m, scope, arg, None, depth)?;
            if !matches!(t, Ty::Dates | Ty::List(_) | Ty::Unseen) {
                return Err(uerr("one_of takes a list of days".to_owned()));
            }
        }
        Ok(Ty::Date)
    }

    /// The calendar argument: a read of a calendar input, at each day asked about
    /// (refcore's `cal_arg`). L §4.2.
    fn cal_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        name: &str,
        depth: u32,
    ) -> Result<(), Stop> {
        let t = self.expr(m, &scope.with_mode(Rel::Calendar), e, None, depth)?;
        if matches!(t, Ty::Unseen) {
            return Ok(());
        }
        if !matches!(t, Ty::Calendar) {
            return Err(uerr(format!("{name}: the calendar is {}", t.show())));
        }
        if input_read(m, scope, e).is_none() {
            return Err(uerr(format!(
                "{name}: its calendar {} is not a read of a calendar input",
                text(e)
            )));
        }
        Ok(())
    }

    /// `is_business(d, cal)` (refcore's `c_is_business`). L §4.2.
    fn is_business(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 2)?;
        if let [day, cal] = call.args.as_slice() {
            self.date_arg(m, scope, day, &call.name, "the date", depth)?;
            self.cal_arg(m, scope, cal, &call.name, depth)?;
        }
        Ok(Ty::Bool)
    }

    /// `add_business(d, n, cal)`, `last_business(p, n, cal)` (refcore's `c_add_business`,
    /// `c_last_business`). L §4.2.
    fn business(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 3)?;
        let [first, count, cal] = call.args.as_slice() else {
            return Ok(Ty::Date);
        };
        if call.name == "add_business" {
            self.date_arg(m, scope, first, &call.name, "the date", depth)?;
        } else {
            let t = self.date_or_period(m, scope, first, &call.name, depth)?;
            if !is_period(m, &t) {
                return Err(uerr(format!("{} takes a period", call.name)));
            }
        }
        self.count_arg(m, scope, count, &call.name, None, depth)?;
        self.cal_arg(m, scope, cal, &call.name, depth)?;
        Ok(Ty::Date)
    }

    /// `round(x, step, mode)`: a number, a step of its unit (1 with a unit, `minor(CUR)` or a
    /// row in a rule), a rounding mode; money where the step is a whole number of minor
    /// units (refcore's `c_round`). L §7.1, §7.2.
    fn round(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 3)?;
        let [x, step, mode] = call.args.as_slice() else {
            return Ok(Ty::Unseen);
        };
        let xt = self.expr(m, scope, x, None, depth)?;
        let c = numcls(m, &xt);
        if (c.is_none() || c == Some("rate")) && !matches!(xt, Ty::Unseen) {
            return Err(uerr(format!("round of {}", xt.show())));
        }
        let mut step_scope = scope.clone();
        if scope.stands == Stands::Ask {
            step_scope.allow_lit = true;
        }
        let mut ut = self.expr(m, &step_scope, step, Some(&xt), depth)?;
        if let Expr::Lit(Lit::Num(n)) = step
            && n.unit.is_none()
            && !n.fraction
        {
            ut = Ty::Flex(n.value.clone());
        }
        let uc = numcls(m, &ut);
        if uc.is_none() && !matches!(ut, Ty::Unseen) {
            return Err(uerr(format!("round: the step is {}", ut.show())));
        }
        if matches!(scope.stands, Stands::Rule | Stands::At) {
            let one = matches!(step, Expr::Lit(Lit::Num(n)) if n.value == crate::num::Num::int(1));
            let minor = matches!(step, Expr::Call(called) if called.name == "minor");
            if !one && !minor && input_read(m, scope, step).is_none() {
                return Err(refused(
                    Load::Literal,
                    "the step of a round is 1 with a unit, minor(CUR) or a row".to_owned(),
                ));
            }
        }
        if matches!(xt, Ty::Unseen) || matches!(ut, Ty::Unseen) {
            self.mode_arg(m, scope, call, mode, depth)?;
            return Ok(Ty::Unseen);
        }
        round_type(self, m, scope, call, (&xt, &ut), mode, depth)
    }

    /// `round`'s mode: a rounding code (refcore's `mode_arg`). L §7.2.
    pub(super) fn mode_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        mode: &Expr,
        depth: u32,
    ) -> Result<(), Stop> {
        if let Expr::Name(id) = mode
            && !scope.is_var(id)
            && m.names.get(id) == Some(&NameKind::Axis)
        {
            return Err(Stop::NotBuilt("axes"));
        }
        let rounding = Ty::Code(CodeKind::Rounding);
        let t = self.expr(m, scope, mode, Some(&rounding), depth)?;
        if t != rounding && !matches!(t, Ty::Unseen) {
            return Err(uerr(format!(
                "{}: the mode is {}, not a rounding code",
                call.name,
                t.show()
            )));
        }
        Ok(())
    }

    /// `abs(x)` (refcore's `c_abs`). L §7.
    fn abs(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 1)?;
        let Some(arg) = call.args.first() else {
            return Ok(Ty::Unseen);
        };
        let t = self.expr(m, scope, arg, None, depth)?;
        if matches!(t, Ty::Unseen) {
            return Ok(t);
        }
        match numcls(m, &t) {
            None => Err(uerr(format!("abs of {}", t.show()))),
            Some("flex") => Ok(Ty::Int),
            Some(_) if matches!(t, Ty::Key(_)) => Ok(Ty::Int),
            Some(_) => Ok(t),
        }
    }

    /// `min(a, b, …)`, `max(a, b, …)`: one type, ordered (refcore's `c_min`). L §5.11.
    fn min_max(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let mut tys = Vec::new();
        for arg in &call.args {
            tys.push(self.expr(m, scope, arg, expect, depth)?);
        }
        let Some(mut out) = tys.first().cloned() else {
            return Ok(Ty::Unseen);
        };
        for (at, t) in tys.iter().enumerate().skip(1) {
            let mut t = t.clone();
            if matches!(out, Ty::KeyLit(_))
                && let Some(first) = call.args.first()
            {
                out = self.expr(m, scope, first, Some(&t), depth)?;
            }
            if matches!(t, Ty::KeyLit(_))
                && let Some(arg) = call.args.get(at)
            {
                t = self.expr(m, scope, arg, Some(&out), depth)?;
            }
            out = unify(m, &out, &t)?;
        }
        for t in &tys {
            no_order(m, t, &call.name)?;
        }
        if matches!(out, Ty::Flex(_)) {
            let whole = tys
                .iter()
                .all(|t| !matches!(t, Ty::Flex(n) if !n.is_integer()));
            return Ok(if whole { Ty::Int } else { num(Unit::one()) });
        }
        Ok(out)
    }

    /// `missing(r)`: only in the `when` of a cited rule, of a read of an input (refcore's
    /// `c_missing`). L §5.11.
    fn missing(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 1)?;
        if !(scope.cited && scope.in_when) {
            return Err(refused(
                Load::Missing,
                "missing() outside the when of a cited rule".to_owned(),
            ));
        }
        let Some(read) = call.args.first() else {
            return Ok(Ty::Bool);
        };
        self.expr(m, scope, read, None, depth)?;
        if input_read(m, scope, read).is_none() {
            let r = text(read);
            return Err(refused(
                Load::Missing,
                format!("missing({r}): {r} is not a read of an input"),
            ));
        }
        Ok(Ty::Bool)
    }

    /// `since(r, d)`: a reading read at the day stated (refcore's `c_since`). L §4.6.
    fn since(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 2)?;
        let [read, day] = call.args.as_slice() else {
            return Ok(Ty::Int);
        };
        self.expr(m, &scope.with_mode(Rel::Explicit), read, None, depth)?;
        let Some(input) = input_read(m, scope, read) else {
            return Err(refused(
                Load::Parse,
                "cannot parse: since takes a read of a reading".to_owned(),
            ));
        };
        if !m.inputs.get(&input).is_some_and(|i| i.reading) {
            return Err(uerr(format!("since reads a reading; {input} is a state")));
        }
        self.date_arg(m, scope, day, &call.name, "the date", depth)?;
        Ok(Ty::Int)
    }

    /// The band of `band_lo`, `band_hi` and `marginal`: `b` or `b[k]`, read (refcore's
    /// `band_ref`). L §5.9.
    fn band_ref(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        depth: u32,
    ) -> Result<Option<String>, Stop> {
        let Some(band) = call.args.first() else {
            return Ok(None);
        };
        let Some((id, posts)) = named_parts(band) else {
            return Err(refused(
                Load::Parse,
                format!("cannot parse: {} takes a band", call.name),
            ));
        };
        let key = match posts {
            [] => None,
            [Postfix::Keys(keys)] => {
                let [key] = keys.as_slice() else {
                    return Err(refused(Load::Key, "wrong number of keys".to_owned()));
                };
                Some(key)
            }
            [_, ..] => {
                return Err(refused(
                    Load::Parse,
                    format!("cannot parse: {} takes a band", call.name),
                ));
            }
        };
        let info = m
            .bands
            .get(id)
            .filter(|_| m.names.get(id) == Some(&NameKind::Band));
        let Some(info) = info else {
            return Err(refused(Load::Name, format!("unknown band {id}")));
        };
        if info.binds.is_empty() == key.is_some() {
            return Err(refused(
                Load::Key,
                format!("wrong number of keys for band {id}"),
            ));
        }
        if let (Some(Key::Expr(kn)), Some(bind)) = (key, info.binds.first()) {
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        self.edge(&scope.item, Node::new(NodeKind::Band, id), Rel::Any);
        Ok(Some(id.to_owned()))
    }

    /// `band_lo(b, x)`, `band_hi(b, x)`: the band's argument type (refcore's `c_band_lo`).
    /// L §5.9.
    fn band_edge(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 2)?;
        let Some(id) = self.band_ref(m, scope, call, depth)? else {
            return Ok(Ty::Unseen);
        };
        let arg_type = m.bands.get(&id).map_or(Ty::Unseen, |b| b.arg_type.clone());
        if let Some(x) = call.args.get(1) {
            let what = format!("the argument of {}", call.name);
            self.typed_as(m, scope, x, &arg_type, &what, depth)?;
        }
        Ok(arg_type)
    }

    /// `marginal(b, x)`: a band whose first row doesn't start at `-inf`, its argument and rows
    /// numbers; their product's unit (refcore's `c_marginal`). L §5.9.
    fn marginal(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 2)?;
        let Some(id) = self.band_ref(m, scope, call, depth)? else {
            return Ok(Ty::Unseen);
        };
        let Some(info) = m.bands.get(&id) else {
            return Ok(Ty::Unseen);
        };
        for table in &info.tables {
            let open = table
                .rows
                .first()
                .is_some_and(|row| row.lo.is_none() && !row.replace);
            if open {
                return Err(refused(
                    Load::Band,
                    format!("marginal({id}, x) over a first row from -inf"),
                ));
            }
        }
        let Some(x) = call.args.get(1) else {
            return Ok(Ty::Unseen);
        };
        let xt = self.typed_as(
            m,
            scope,
            x,
            &info.arg_type,
            "the argument of marginal",
            depth,
        )?;
        let rt = &info.ty;
        if matches!(xt, Ty::Unseen) {
            return Ok(Ty::Unseen);
        }
        if numcls(m, &xt).is_none() || numcls(m, rt).is_none() {
            return Err(uerr(format!(
                "marginal of {} over rows of {}",
                xt.show(),
                rt.show()
            )));
        }
        let measured = |t: &Ty| matches!(t, Ty::Num(..) | Ty::Money(_));
        let xu = if measured(&xt) {
            unit_of(&xt)?
        } else {
            Unit::one()
        };
        let ru = if measured(rt) {
            unit_of(rt)?
        } else {
            Unit::one()
        };
        let (u, _) = xu
            .mul(&ru)
            .map_err(|_| Stop::NotBuilt("a unit past the 64-bit powers"))?;
        Ok(num(u))
    }

    /// `b(x)` or `b[k](x)`: one argument of the band's argument type (refcore's
    /// `band_call`). L §5.9.
    fn band_call(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        let Some(kind) = m.names.get(&call.name) else {
            return Err(refused(Load::Name, format!("unknown id {}", call.name)));
        };
        let band = m.bands.get(&call.name).filter(|_| *kind == NameKind::Band);
        let Some(band) = band else {
            return Err(refused(
                Load::Parse,
                format!(
                    "cannot parse: {} is not a band, and there are no user functions",
                    call.name
                ),
            ));
        };
        let [arg] = call.args.as_slice() else {
            return Err(refused(
                Load::Parse,
                "cannot parse: a band takes one argument".to_owned(),
            ));
        };
        if band.binds.is_empty() == call.band_key.is_some() {
            return Err(refused(
                Load::Key,
                format!("wrong number of keys for band {}", band.id),
            ));
        }
        if let (Some(Key::Expr(kn)), Some(bind)) = (&call.band_key, band.binds.first()) {
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        let what = format!("the argument of band {}", band.id);
        self.typed_as(m, scope, arg, &band.arg_type, &what, depth)?;
        let rel = if band.binds.is_empty() {
            Rel::Same
        } else {
            Rel::Computed
        };
        self.edge(&scope.item, Node::new(NodeKind::Band, &band.id), rel);
        Ok(band.ty.clone())
    }

    /// A business-day function's calendar argument at `at`, read at each day asked about;
    /// the call `Unseen` (refcore's `cal_arg`; `schedule` is a later group's). L §4.2.
    fn calendar_call(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        at: usize,
        depth: u32,
    ) -> Typed {
        let calendar = scope.with_mode(Rel::Calendar);
        for (position, arg) in args.iter().enumerate() {
            let read_by = if position == at { &calendar } else { scope };
            self.expr(m, read_by, arg, None, depth)?;
        }
        Ok(Ty::Unseen)
    }

    /// `next(K, d)`: every move of K or a subkind (refcore's `m_c_next`). L §5.14.1.
    fn next(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        let Some((kind, rest)) = args.split_first() else {
            return Ok(Ty::Unseen);
        };
        if let Expr::Name(id) = kind
            && m.names.get(id) == Some(&NameKind::Kind)
        {
            let sub = m.subtree(id);
            for (move_id, mv) in m.moves.entries() {
                if sub.contains(&mv.kind) {
                    self.edge(&scope.item, Node::new(NodeKind::Move, move_id), Rel::Next);
                }
            }
        }
        self.args_only(m, scope, rest, depth)
    }

    /// `observed(m[k])`: the planned instance, and every observed input (refcore's
    /// `m_c_observed`). L §5.19.
    fn observed(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        self.args_only(m, scope, args, depth)?;
        let instance = args.first().and_then(named_parts);
        if instance.is_some_and(|(id, _)| m.names.get(id) == Some(&NameKind::Move)) {
            for (id, input) in m.inputs.entries() {
                if input.observed {
                    self.edge(&scope.item, Node::new(NodeKind::Input, id), Rel::Observed);
                }
            }
        }
        Ok(Ty::Unseen)
    }

    /// `gap(x, d)`: a balance read (refcore's `m_c_gap`). L §5.19.
    fn gap(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        self.args_only(m, scope, args, depth)?;
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        Ok(Ty::Unseen)
    }

    /// `m.leg.amount of q`: booking state, read as a balance is (refcore's `m_t_legof`,
    /// `m_legof_read`). L §5.14.1 (R-10).
    pub(super) fn leg_of(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        of: &OfQ,
        depth: u32,
    ) -> Typed {
        self.expr(m, scope, inner, None, depth)?;
        self.of(m, scope, Some(of), depth)?;
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.mark(&scope.item, Mark::LegOf, Target::Unknown);
        Ok(Ty::Unseen)
    }

    /// `of q`: the party, the batch, or the pair.
    fn of(&mut self, m: &Reality, scope: &Scope, of: Option<&OfQ>, depth: u32) -> Result<(), Stop> {
        match of {
            None => Ok(()),
            Some(OfQ::Ref(e)) => self.expr(m, scope, e, None, depth).map(|_| ()),
            Some(OfQ::Pair(party, batch)) => {
                self.expr(m, scope, party, None, depth)?;
                self.expr(m, scope, batch, None, depth).map(|_| ())
            }
        }
    }

    /// `e to u`: the value and a unit written as a reference.
    pub(super) fn to(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        step: &Step,
        depth: u32,
    ) -> Typed {
        self.expr(m, scope, inner, None, depth)?;
        if let Step::Ref(e) = step {
            self.expr(m, scope, e, None, depth)?;
        }
        Ok(Ty::Unseen)
    }

    /// A span's ends, each a date or a period (refcore's `span`). L §5.6.
    pub(super) fn span(
        &mut self,
        m: &Reality,
        scope: &Scope,
        span: &Span,
        depth: u32,
    ) -> Result<(), Stop> {
        for end in [span.from.as_ref(), span.to.as_ref()].into_iter().flatten() {
            let side = self.operand(m, scope, end, None, depth)?;
            let mut t = side.ty.clone();
            if matches!(t, Ty::KeyList)
                && let Some(read) = self.kl_value(m, scope, &side, Reading::Year, None)?
            {
                t = read;
            }
            if matches!(t, Ty::Year4) {
                self.decide(side.year4, Reading::Year);
                t = Ty::Period(PeriodWord::Year);
            }
            if !(is_date(m, &t) || is_period(m, &t)) {
                return Err(uerr(format!(
                    "a span end is {}, not a date or a period",
                    t.show()
                )));
            }
        }
        Ok(())
    }

    /// A balance read: the place, its days, its clauses; marked as a running read, a
    /// projection or `bal(x)` with no day (refcore's `m_t_bal`, `m_check_proj`); its type a
    /// later group's. L §5.14.1; E §3.1 step 7.
    pub(super) fn bal(&mut self, m: &Reality, scope: &Scope, bal: &Bal, depth: u32) -> Typed {
        if bal.proj.random.is_some() {
            return Err(Stop::NotBuilt("a projection's `random at`"));
        }
        self.expr(
            m,
            scope,
            &bal.place,
            Some(&Ty::Key("places".to_owned())),
            depth,
        )?;
        for arg in &bal.args {
            self.expr(m, scope, arg, None, depth)?;
        }
        if let Some(from) = &bal.proj.from {
            self.expr(m, scope, from, None, depth)?;
        }
        if !bal.modelled {
            let mark = self.bal_mark(scope, bal);
            self.mark(&scope.item, mark, place_target(m, scope, &bal.place));
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.of(m, scope, bal.of.as_ref(), depth)?;
        Ok(Ty::Unseen)
    }

    /// A balance read's mark: a projection, `bal(x)` with no day, or a running read at a
    /// day; `bal(x, d) modelled` reads the modelled past and takes none (L §5.19).
    fn bal_mark(&mut self, scope: &Scope, bal: &Bal) -> Mark {
        if bal.proj.from.is_some() {
            self.graph.holders.insert(scope.item.clone());
            return Mark::Proj;
        }
        if bal.args.is_empty() {
            self.graph.balnow.insert(scope.item.clone());
            return Mark::BalNow;
        }
        Mark::Bal
    }

    /// `flows(x, span …)`: of a move (its leg named, or its one leg), or of a place
    /// (refcore's `m_t_flows`); its type a later group's. L §5.14.3, §5.14.1.
    pub(super) fn flows(&mut self, m: &Reality, scope: &Scope, flows: &Flows, depth: u32) -> Typed {
        if let Some((id, keys)) = move_keys(&flows.target)
            && !scope.is_var(id)
            && m.names.get(id) == Some(&NameKind::Move)
        {
            for key in keys {
                if let Key::Expr(e) = key {
                    self.expr(m, scope, e, None, depth)?;
                }
            }
            self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Flows);
            if m.moves.get(id).is_some_and(|mv| mv.decision) {
                self.mark(&scope.item, Mark::Flows, Target::Move(id.to_owned()));
            }
        } else {
            self.expr(m, scope, &flows.target, None, depth)?;
            let target = place_target(m, scope, &flows.target);
            self.mark(&scope.item, Mark::Flows, target);
            self.edge(&scope.item, Node::balances(), Rel::Flows);
        }
        self.span(m, scope, &flows.span, depth)?;
        if let Some(key) = &flows.for_key {
            self.expr(m, scope, key, None, depth)?;
        }
        if let Some((_, rate, mode)) = &flows.convert {
            self.expr(m, scope, rate, None, depth)?;
            self.expr(m, scope, mode, None, depth)?;
        }
        Ok(Ty::Unseen)
    }

    /// `covered(x, span)`: the place and the span (refcore's `m_c_covered`). L §5.19.
    pub(super) fn covered(
        &mut self,
        m: &Reality,
        scope: &Scope,
        place: &Expr,
        span: &Span,
        depth: u32,
    ) -> Typed {
        self.expr(m, scope, place, None, depth)?;
        self.span(m, scope, span, depth)?;
        Ok(Ty::Bool)
    }

    /// `value(x, d) in CUR [by p] [of q]`: the place, the day, the prices its places are
    /// valued by, and a balance read (refcore's `m_t_value`, `m_price_edges`); its type a
    /// later group's. L §5.10.
    pub(super) fn value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        value: &Valuation,
        depth: u32,
    ) -> Typed {
        self.expr(m, scope, &value.place, None, depth)?;
        self.expr(m, scope, &value.day, None, depth)?;
        for price in price_inputs(m, scope, &value.place) {
            self.edge(&scope.item, price, Rel::Explicit);
        }
        if let Some(by) = &value.by {
            self.by(m, scope, by, &value.day, depth)?;
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.mark(&scope.item, Mark::Bal, place_target(m, scope, &value.place));
        self.of(m, scope, value.of.as_ref(), depth)?;
        Ok(Ty::Unseen)
    }

    /// A value's `by p`: a keyed price input or def, or a reading through `last`, read at
    /// the value's day; any other price is an expression read as written (refcore's
    /// `m_t_value`, its `by` branches). L §5.10.
    fn by(
        &mut self,
        m: &Reality,
        scope: &Scope,
        by: &Price,
        day: &Expr,
        depth: u32,
    ) -> Result<(), Stop> {
        if let Expr::Name(id) = &by.input
            && !scope.is_var(id)
            && let Some((node, keyed)) = input_or_def(m, id)
            && (keyed || by.last)
        {
            self.edge(&scope.item, node, Rel::Explicit);
            return Ok(());
        }
        if by.last {
            self.expr(m, &scope.with_mode(Rel::Explicit), &by.input, None, depth)?;
            self.expr(m, scope, day, None, depth)?;
            return Ok(());
        }
        self.expr(m, scope, &by.input, None, depth).map(|_| ())
    }
}

/// Whether a top-level name of this kind is read as a value. L §2.5.
fn is_value(kind: NameKind) -> bool {
    match kind {
        NameKind::Input | NameKind::Def | NameKind::Place | NameKind::Move | NameKind::Cut => true,
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Fill
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source => false,
    }
}

/// `minor(CUR)`: money in a declared currency (refcore's `c_minor`). L §7.2.
fn minor(m: &Reality, call: &Call) -> Typed {
    arity(call, 1)?;
    let Some(Expr::Lit(Lit::Cur(cur))) = call.args.first() else {
        return Err(refused(
            Load::Parse,
            "cannot parse: minor takes a currency code".to_owned(),
        ));
    };
    if !m.currencies.contains_key(cur) {
        return Err(uerr(format!("money in an undeclared currency {cur}")));
    }
    Ok(Ty::Money(cur.clone()))
}

/// `round`'s type from its operand's and its step's (refcore's `c_round`, its end). L §7.2.
fn round_type(
    checker: &mut Checker,
    m: &Reality,
    scope: &Scope,
    call: &Call,
    (xt, ut): (&Ty, &Ty),
    mode: &Expr,
    depth: u32,
) -> Typed {
    let c = numcls(m, xt);
    let uc = numcls(m, ut);
    let measured = |k: Option<&str>| matches!(k, Some("num" | "money"));
    let xu = if measured(c) {
        unit_of(xt)?
    } else {
        Unit::one()
    };
    let su = if measured(uc) {
        unit_of(ut)?
    } else {
        Unit::one()
    };
    if c == Some("dyn") {
        checker.mode_arg(m, scope, call, mode, depth)?;
        return Ok(match (uc, ut) {
            (Some("money"), Ty::Money(cur)) => Ty::Money(cur.clone()),
            (Some("num"), _) => num(su),
            (Some(_) | None, _) => Ty::Dyn,
        });
    }
    let converts = su
        .conv(&xu)
        .map_err(|_| Stop::NotBuilt("a unit past the 64-bit powers"))?
        .is_some();
    if uc != Some("flex") && !converts {
        return Err(uerr(format!(
            "round: the step {} and {} have different units",
            ut.show(),
            xt.show()
        )));
    }
    checker.mode_arg(m, scope, call, mode, depth)?;
    let currency = match xu.terms() {
        [(cur, 1)] if xu.is_currency() => Some(cur.clone()),
        _ => None,
    };
    if let (Some("money"), Some(_), Ty::Money(cur)) = (uc, &currency, ut) {
        return Ok(Ty::Money(cur.clone()));
    }
    if let (Some("flex"), Some(cur), Ty::Flex(n)) = (uc, &currency, ut) {
        let whole = m
            .currencies
            .get(cur)
            .and_then(|minor| n.checked_div(minor).ok())
            .is_some_and(|q| q.is_integer());
        return Ok(if whole {
            Ty::Money(cur.clone())
        } else {
            num(xu)
        });
    }
    let pct = matches!(xt, Ty::Num(_, Shown::Percent));
    if let Ty::Flex(n) = ut
        && *n == crate::num::Num::int(1)
        && xu.is_one()
        && !pct
        && matches!(c, Some("num" | "int" | "flex"))
    {
        return Ok(Ty::Int);
    }
    if matches!(c, Some("int" | "flex")) {
        return Ok(num(Unit::one()));
    }
    Ok(num(xu))
}
