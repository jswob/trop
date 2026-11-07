//! Load's checker, its calls: every built-in's argument types and result, band calls, `to` and spans.
//! Spec: L §4.2 (the date and calendar functions), §5.7 (`lifespan`), §5.9 (bands, `band_lo`,
//! `band_hi`, `marginal`), §5.10 (`unit`), §5.11 (`missing`, `min`, `max`), §7.2 (`round`, its
//! step and mode), §4.6 (`since`), §5.6 (spans), §6.4 (a scenario row's base prefix), §8.3
//! (`LOAD-UNIT`, `-PARSE`, `-NAME`, `-KEY`, `-LITERAL`, `-MISSING`, `-BAND`), §8.4.
//! Twin: engine/tools/refcore/check.py (`t_call`, `args`, `want`, `tdate`,
//! `tdate_or_period`, `tint`, `c_start_of` to `c_since`, `c_lifespan`, `cal_arg`, `band_ref`,
//! `c_band_lo`, `c_marginal`, `band_call`, `mode_arg`, `span`, `prefixed_call`); check3.py
//! (`m_c_covered`, its call form `reads.rs`'s); the reads of the ledger are `reads.rs`'s, the
//! exact and transcendental calls `exact.rs`'s.
//! Invariants: every argument is typed in refcore's order and its reads recorded; a call is a
//! band's, a built-in's with its arm, or a name looked up (`unknown id`, `is not called`), as
//! refcore's `t_call`: no call is left untyped.

use super::check::{Checker, Reading, Scope, Stands};
use super::ops::unify;
use super::steps::num_stop;
use super::targets::named_parts;
use super::texts::text;
use super::types::{
    Typed, input_read, is_date, is_period, no_order, num, numcls, period_kind, refused, uerr,
    unit_error, unit_of, unit_stop,
};
use crate::base::Load;
use crate::model::{CodeKind, NameKind, Node, NodeKind, PeriodWord, Reality, Rel, Shown, Ty, Unit};
use crate::syntax::{Call, Expr, Key, Lit, Postfix, Span, Stop};
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

/// A call naming no built-in with a typing arm: `LOAD-PARSE … is not called` for a name the
/// model declares, else `LOAD-NAME unknown id` (refcore's `t_call` with no `c_<name>`). The
/// names refcore's `parser.BUILTINS` holds with no arm are parsed as forms of their own (the
/// aggregates, the queries, `prorata`, `flows`, `bal`, `value`, `low`, the distributions and
/// their functions) and reach a call only as a scenario row's `household.sum(x)`, its base
/// prefix dropped. L §2.3 Calls, §6.4, §8.3.
fn no_builtin(m: &Reality, call: &Call) -> Stop {
    if m.names.contains_key(&call.name) {
        return refused(
            Load::Parse,
            format!("cannot parse: {} is not called", call.name),
        );
    }
    refused(Load::Name, format!("unknown id {}", call.name)).naming(&call.name)
}

/// `LOAD-PARSE` for a built-in given the wrong number of arguments (refcore's `args`).
pub(super) fn arity(call: &Call, n: usize) -> Result<(), Stop> {
    if call.args.len() != n {
        return Err(refused(
            Load::Parse,
            format!("cannot parse: {} takes {n} arguments", call.name),
        ));
    }
    Ok(())
}

/// `LOAD-DOMAIN` for `day_of` or `schedule` of a key of an anchored index, where load sees
/// the index (refcore's `anchored_key`). L §4.2, §8.3, §8.4.
fn anchored_key(m: &Reality, t: &Ty, what: &str) -> Result<(), Stop> {
    let Ty::Key(ix) = t else {
        return Ok(());
    };
    if m.indexes.get(ix).is_some_and(|index| index.anchored) {
        let message = format!("{what} of a key of the anchored index {ix} has no meaning");
        return Err(refused(Load::Domain, message));
    }
    Ok(())
}

/// A call whose prefixes name neither the base (in a scenario's expression row) nor a member
/// that declares the band: a joint model's merge has made each member's band one name, so a
/// member's prefix left over names its ask (no band) or nothing (refcore's `prefixed_call`).
/// L §2.3 Calls, §6.1, §8.3.
fn prefixed_call(m: &Reality, first: &str, call: &Call) -> Stop {
    let mut full = call.prefix.join(".");
    full.push('.');
    full.push_str(&call.name);
    let asked = call.prefix.len() == 1
        && m.joint
            .as_ref()
            .is_some_and(|joint| joint.names_ask(first, &call.name));
    if asked {
        let text = format!("cannot parse: {full} is not a band, and there are no user functions");
        return refused(Load::Parse, text);
    }
    if m.names.contains_key(first) {
        return refused(
            Load::Name,
            format!("cannot parse: {full} names no band (a prefix is a member or the base)"),
        );
    }
    refused(Load::Name, format!("unknown id {full}")).naming(&full)
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
        if let [prefix] = call.prefix.as_slice()
            && self.base_world.as_ref().is_some_and(|w| w.id == *prefix)
        {
            // `household.b(x)` in a scenario's expression row: the base's band, called bare
            // (refcore's `m_check_scen_rows` rewrite). L §2.3 Calls, §6.4.
            let bare = Call {
                prefix: Vec::new(),
                ..call.clone()
            };
            return self.call(m, scope, &bare, expect, depth);
        }
        if let Some(first) = call.prefix.first() {
            return Err(prefixed_call(m, first, call));
        }
        let named_band = !DATE_FNS.contains(&call.name.as_str())
            && m.names.kind(&call.name) == Some(NameKind::Band);
        if call.band_key.is_some() || named_band {
            return self.band_call(m, scope, call, depth);
        }
        self.builtin(m, scope, call, expect, depth)
    }

    /// A built-in, one arm per family (refcore's `c_*`); any other name is looked up.
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
            "lifespan" => self.lifespan(m, scope, call, depth),
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
            "schedule" => self.schedule(m, scope, call, depth),
            "unit" => self.unit(m, scope, args, depth),
            "exp" | "ln" | "sqrt" | "pow" | "periodic" | "factor" | "dcf" | "accrue" => {
                self.exact_call(m, scope, call, depth)
            }
            "covered" => self.covered_call(m, scope, args, depth),
            _ => Err(no_builtin(m, call)),
        }
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
        anchored_key(m, &t, &call.name)?;
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

    /// `schedule(p, e, roll, cal)`: p a month, not a key of an anchored index; e a count; a
    /// roll code; a calendar input (refcore's `c_schedule`). L §4.2.
    fn schedule(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 4)?;
        let [period, day, roll, cal] = call.args.as_slice() else {
            return Ok(Ty::Date);
        };
        let t = self.date_or_period(m, scope, period, &call.name, depth)?;
        anchored_key(m, &t, &call.name)?;
        if matches!(t, Ty::Unseen) {
            self.expr(m, scope, day, None, depth)?;
        } else {
            if period_kind(m, &t) != Some(PeriodKind::Month) {
                return Err(uerr(format!("schedule takes a month, not {}", t.show())));
            }
            self.count_arg(m, scope, day, &call.name, None, depth)?;
        }
        let rolls = Ty::Code(CodeKind::Roll);
        let t3 = self.expr(m, scope, roll, Some(&rolls), depth)?;
        if t3 != rolls && !matches!(t3, Ty::Unseen) {
            return Err(uerr(format!("schedule: the roll is {}", t3.show())));
        }
        self.cal_arg(m, scope, cal, &call.name, depth)?;
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
                return Err(refused(Load::Key, format!("ambiguous key {written}")).naming(written));
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

    /// `lifespan(t)`, t a thing: the record `(from date, to date)` (refcore's `c_lifespan`).
    /// A bare key naming one thing is read as a key of its kind. L §5.7, §2.5.
    fn lifespan(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 1)?;
        let span = || {
            Ty::Record(vec![
                ("from".to_owned(), Ty::Date),
                ("to".to_owned(), Ty::Date),
            ])
        };
        let Some(arg) = call.args.first() else {
            return Ok(span());
        };
        let mut t = self.expr(m, scope, arg, None, depth)?;
        if let Ty::KeyLit(written) = &t
            && let Some(base) = m.things.get(written).and_then(|thing| thing.base.clone())
        {
            t = self.expr(m, scope, arg, Some(&Ty::Key(base)), depth)?;
        }
        // `Unseen` is a type refcore knows and load doesn't see yet (a bound name's field a
        // kind test narrows, or read over `events`), so it passes; `dyn` is refused.
        let of_a_kind = matches!(&t, Ty::Key(ix) if m.kinds.get(ix).is_some());
        if !of_a_kind && !matches!(t, Ty::Unseen) {
            return Err(uerr(format!("lifespan of {}, not a thing", t.show())));
        }
        Ok(span())
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
            self.mode_arg(m, scope, &call.name, mode, depth)?;
            return Ok(Ty::Unseen);
        }
        round_type(self, m, scope, call, (&xt, &ut), mode, depth)
    }

    /// `round`'s mode: a rounding code, or an axis whose options are all mode names, read
    /// as an axis (refcore's `mode_arg`). L §7.2, §5.5.
    pub(super) fn mode_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        name: &str,
        mode: &Expr,
        depth: u32,
    ) -> Result<(), Stop> {
        if let Expr::Name(id) = mode
            && !scope.is_var(id)
            && let Some(axis) = m.axes.get(id)
        {
            for (option, _) in &axis.options {
                if CodeKind::of_word(option) != Some(CodeKind::Rounding) {
                    return Err(uerr(format!(
                        "axis {id} is used as a mode, but {option} is not a mode name"
                    )));
                }
            }
            self.axis_read(&scope.item, id, super::check::AxisRead::ById);
            return Ok(());
        }
        let rounding = Ty::Code(CodeKind::Rounding);
        let t = self.expr(m, scope, mode, Some(&rounding), depth)?;
        if t != rounding && !matches!(t, Ty::Unseen) {
            return Err(uerr(format!(
                "{name}: the mode is {}, not a rounding code",
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
            .filter(|_| m.names.kind(id) == Some(NameKind::Band));
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
        let target = Node::new(NodeKind::Band, id);
        self.direct(&target);
        self.edge(&scope.item, target, Rel::Any);
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
        let (u, _) = xu.mul(&ru).map_err(unit_error)?;
        Ok(num(u))
    }

    /// `b(x)` or `b[k](x)`: one argument of the band's argument type (refcore's
    /// `band_call`). L §5.9.
    fn band_call(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        let Some(kind) = m.names.kind(&call.name) else {
            return Err(refused(Load::Name, format!("unknown id {}", call.name)).naming(&call.name));
        };
        let band = m.bands.get(&call.name).filter(|_| kind == NameKind::Band);
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
        let target = Node::new(NodeKind::Band, &band.id);
        self.direct(&target);
        self.edge(&scope.item, target, rel);
        Ok(band.ty.clone())
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
        checker.mode_arg(m, scope, &call.name, mode, depth)?;
        return Ok(match (uc, ut) {
            (Some("money"), Ty::Money(cur)) => Ty::Money(cur.clone()),
            (Some("num"), _) => num(su),
            (Some(_) | None, _) => Ty::Dyn,
        });
    }
    let converts = su.conv(&xu).map_err(unit_stop)?.is_some();
    if uc != Some("flex") && !converts {
        return Err(uerr(format!(
            "round: the step {} and {} have different units",
            ut.show(),
            xt.show()
        )));
    }
    checker.mode_arg(m, scope, &call.name, mode, depth)?;
    let currency = match xu.terms() {
        [(cur, 1)] if xu.is_currency() => Some(cur.clone()),
        _ => None,
    };
    if let (Some("money"), Some(_), Ty::Money(cur)) = (uc, &currency, ut) {
        return Ok(Ty::Money(cur.clone()));
    }
    if let (Some("flex"), Some(cur), Ty::Flex(n)) = (uc, &currency, ut) {
        let whole = match m.currencies.get(cur) {
            Some(minor) => n.checked_div(minor).map_err(num_stop)?.is_integer(),
            None => false,
        };
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
