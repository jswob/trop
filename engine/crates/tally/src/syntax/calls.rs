//! The built-in calls and the reads with clauses: `bal`, `value`, `flows`, `covered`, the aggregates, `calendar`, the day sets and `price`.
//! Spec: L §2.3 (`call`, `agg`, `AGG`, `balx`, `proj`, `wchoice`, `valuex`, `flowsx`, `MODE`,
//! `coveredx`, `calval`, `cday`, `dayset`, `price`), its parse rules 4 and 7, and its Calls
//! (`min` and `max` with two or more plain arguments; a period word as the period argument of
//! `periodic` and `factor`); L §2.2 (a built-in's name, read as one only directly before `(`).
//! Twin: engine/tools/refcore/parser.py (`builtin`, `period_word_arg`, `agg`, `proj`,
//! `wchoice`, `mode`, `calval`, `_mmdd`, `cday`, `dayset`, `price`).
//! Invariants: each bracket is counted by `open` as refcore counts it; a clause after a read
//! belongs to the nearest read; the error texts are refcore's word for word.

use super::expr::Ends;
use super::lex::{Kind, Read};
use super::parse::{P, Stop};
use super::reads::{dist_kind, query_fn};
use super::tree::{
    Agg, AggFn, Bal, Calendar, CalendarDay, Call, DaySet, DistFnKind, Expr, Flows, Lit, Price,
    Proj, Valuation,
};
use crate::time::Day;

/// The aggregate named `word`, if it is one.
fn agg_fn(word: &str) -> Option<AggFn> {
    match word {
        "sum" => Some(AggFn::Sum),
        "count" => Some(AggFn::Count),
        "min" => Some(AggFn::Min),
        "max" => Some(AggFn::Max),
        "any" => Some(AggFn::Any),
        "all" => Some(AggFn::All),
        "first" => Some(AggFn::First),
        "last" => Some(AggFn::Last),
        "argmin" => Some(AggFn::Argmin),
        "argmax" => Some(AggFn::Argmax),
        _ => None,
    }
}

impl P<'_> {
    /// A projection's clauses: `from`, `windows`, `random at`, `with uncertain`. The `from`
    /// term counts one depth level, so `bal(x, d) from bal(y, e) from …` is bounded by
    /// `LOAD-DEPTH` (L §2.3, tally 2.1.2).
    pub(super) fn proj(&mut self) -> Result<Proj, Stop> {
        let from = if self.accept("from")? {
            self.open()?;
            let term = self.post(Ends::Free)?;
            self.close()?;
            Some(term)
        } else {
            None
        };
        let windows_each = if self.accept("windows")? {
            self.each()?;
            true
        } else {
            false
        };
        let random = if self.at("random", 0) && self.at("at", 1) {
            self.skip(2)?;
            Some(self.rchoice()?)
        } else {
            None
        };
        let uncertain = self.with_uncertain()?;
        Ok(Proj {
            from,
            windows_each,
            random,
            uncertain,
        })
    }

    /// `with uncertain`, taken if next.
    pub(super) fn with_uncertain(&mut self) -> Result<bool, Stop> {
        if self.at("with", 0) && self.at("uncertain", 1) {
            self.skip(2)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// `wchoice = "each"`.
    pub(super) fn each(&mut self) -> Result<(), Stop> {
        if self.at("each", 0) {
            return self.skip(1);
        }
        Err(self.err("each"))
    }

    /// `MODE`: a rounding code's key token, or a reference.
    pub(super) fn mode(&mut self) -> Result<Expr, Stop> {
        if self.kind(0) == Some(&Kind::Key) {
            return Ok(Expr::Lit(Lit::Key(self.next()?.text.clone())));
        }
        self.reference()
    }

    /// A built-in, `word` directly before `(`.
    pub(super) fn builtin(&mut self, word: &str) -> Result<Expr, Stop> {
        if let Some(func) = agg_fn(word) {
            return self.agg(func);
        }
        if let Some(func) = query_fn(word) {
            return self.query(func);
        }
        if let Some(func) = dist_kind(word) {
            return self.dist(func);
        }
        match word {
            "low" => self.low(),
            "crps" => self.crps(),
            "cdf" => self.dist_fn(DistFnKind::Cdf),
            "pdf" => self.dist_fn(DistFnKind::Pdf),
            "pmf" => self.dist_fn(DistFnKind::Pmf),
            "prorata" => self.prorata(),
            "bal" => self.bal(),
            "flows" => self.flows(),
            "covered" => self.covered(),
            "value" => self.valuation(),
            "calendar" => Ok(Expr::Calendar(Box::new(self.calendar()?))),
            _ => {
                self.skip(1)?;
                let open = self.i;
                let args = self.call_args()?;
                match word {
                    "periodic" => self.period_word_arg(open, 1),
                    "factor" => self.period_word_arg(open, 2),
                    _ => {}
                }
                Ok(Expr::Call(Box::new(Call {
                    name: word.to_owned(),
                    prefix: Vec::new(),
                    band_key: None,
                    args,
                })))
            }
        }
    }

    /// The period argument of `periodic` (its second) and `factor` (its third), a lone period
    /// word there, is a word of the language, not an id, so a member's index named `month`
    /// doesn't take it (refcore's `period_word_arg`). L §2.2, §6.1.
    fn period_word_arg(&self, open: usize, pos: usize) {
        let Some(s) = self.lone_arg(open, pos) else {
            return;
        };
        if let Some(t) = self.toks.get(s)
            && t.kind == Kind::Id
            && matches!(t.text.as_str(), "day" | "month" | "quarter" | "year")
        {
            self.mark(Some(s), |mark| mark.read = Read::Word);
        }
    }

    /// The place of the one token argument `pos` of a call is, its `(` at `open`, when it is
    /// one token (refcore's `period_word_arg`, its walk over the arguments). L §2.3.
    fn lone_arg(&self, open: usize, pos: usize) -> Option<usize> {
        let (mut depth, mut k, mut start): (u32, usize, Option<usize>) = (0, 0, None);
        for j in open..self.i {
            let t = self.toks.get(j)?;
            let op = t.kind == Kind::Op;
            if op && (t.text == "(" || t.text == "[") {
                depth = depth.checked_add(1)?;
                if depth == 1 && t.text == "(" && j == open {
                    start = Some(j.checked_add(1)?);
                }
                continue;
            }
            let lone = start.filter(|s| j.checked_sub(*s) == Some(1));
            if op && (t.text == ")" || t.text == "]") {
                if depth == 1 && k == pos && lone.is_some() {
                    return lone;
                }
                depth = depth.checked_sub(1)?;
                continue;
            }
            if depth == 1 && op && t.text == "," {
                if k == pos {
                    return lone;
                }
                k = k.checked_add(1)?;
                start = Some(j.checked_add(1)?);
            }
        }
        None
    }

    /// `bal(x [, [e,] e [start]]) [of q] [proj | modelled]` (L §2.3 `balx`, parse rule 7).
    fn bal(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.reference()?;
        let mut args = Vec::new();
        let mut start = false;
        if self.accept(",")? {
            args.push(self.expr(Ends::Free)?);
            if self.accept(",")? {
                args.push(self.expr(Ends::Free)?);
            }
            start = self.accept("start")?;
        }
        self.expect(")")?;
        self.close()?;
        let of = if self.accept("of")? {
            Some(self.ofq()?)
        } else {
            None
        };
        let modelled = self.accept("modelled")?;
        let proj = if modelled {
            Proj {
                from: None,
                windows_each: false,
                random: None,
                uncertain: false,
            }
        } else if self.at("from", 0) {
            self.proj()?
        } else {
            Proj {
                from: None,
                windows_each: false,
                random: None,
                uncertain: self.with_uncertain()?,
            }
        };
        Ok(Expr::Bal(Box::new(Bal {
            place,
            args,
            start,
            modelled,
            proj,
            of,
        })))
    }

    /// `value(x, d) in CUR [by p | by last(p)] [of q]`: a value's `of` comes after its `by`
    /// (L §2.3 `valuex`, parse rule 7).
    fn valuation(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.reference()?;
        self.expect(",")?;
        let day = self.expr(Ends::Free)?;
        self.expect(")")?;
        self.close()?;
        self.expect("in")?;
        let currency = self.cur()?;
        let by = if self.accept("by")? {
            Some(self.price()?)
        } else {
            None
        };
        let of = if self.accept("of")? {
            Some(self.ofq()?)
        } else {
            None
        };
        Ok(Expr::Value(Box::new(Valuation {
            place,
            day,
            currency,
            by,
            of,
        })))
    }

    /// `flows(x, span [, for k]) [in CUR at fx MODE]` (L §2.3 `flowsx`, parse rule 4).
    fn flows(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let target = self.reference()?;
        self.expect(",")?;
        let span = self.span()?;
        let for_key = if self.accept(",")? {
            self.expect("for")?;
            Some(self.expr(Ends::Free)?)
        } else {
            None
        };
        self.expect(")")?;
        self.close()?;
        let convert = if self.at("in", 0) && self.kind(1) == Some(&Kind::Cur) {
            self.skip(1)?;
            let cur = self.cur()?;
            self.expect("at")?;
            let rate = self.reference()?;
            let mode = self.mode()?;
            Some((cur, rate, mode))
        } else {
            None
        };
        Ok(Expr::Flows(Box::new(Flows {
            target,
            span,
            for_key,
            convert,
        })))
    }

    /// `covered(x, span)` (L §2.3 `coveredx`).
    fn covered(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.expr(Ends::Free)?;
        self.expect(",")?;
        let span = self.span()?;
        self.expect(")")?;
        self.close()?;
        Ok(Expr::Covered(Box::new((place, span))))
    }

    /// An aggregate; `min` and `max` with plain arguments are calls; `last(x, d)` a carry.
    fn agg(&mut self, func: AggFn) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let (term, binds) = if self.is_bind_start() {
            (None, self.binds()?)
        } else {
            let first = self.expr(Ends::Free)?;
            if self.accept("for")? {
                (Some(first), self.binds()?)
            } else if (func == AggFn::Min || func == AggFn::Max) && self.at(",", 0) {
                return self.plain_min_max(func, first);
            } else if func == AggFn::Last && self.accept(",")? {
                let day = self.expr(Ends::Free)?;
                self.expect(")")?;
                self.close()?;
                return Ok(Expr::Last(Box::new((first, day))));
            } else {
                return Err(self.err("\"for\" or a binding"));
            }
        };
        self.expect(")")?;
        self.close()?;
        let ties = if self.at("ties", 0) && self.at("by", 1) {
            Some(self.ties()?)
        } else {
            None
        };
        Ok(Expr::Agg(Box::new(Agg {
            func,
            term,
            binds,
            ties,
        })))
    }

    /// `min(a, b, …)` or `max(a, b, …)`: a call of two or more plain arguments.
    fn plain_min_max(&mut self, func: AggFn, first: Expr) -> Result<Expr, Stop> {
        let mut args = vec![first];
        while self.accept(",")? {
            args.push(self.expr(Ends::Free)?);
        }
        self.expect(")")?;
        self.close()?;
        let name = if func == AggFn::Min { "min" } else { "max" };
        Ok(Expr::Call(Box::new(Call {
            name: name.to_owned(),
            prefix: Vec::new(),
            band_key: None,
            args,
        })))
    }

    /// `calendar(weekends [...], [yearly [...] [from n]], [added [...]])` (L §4.2).
    pub(super) fn calendar(&mut self) -> Result<Calendar, Stop> {
        self.expect("calendar")?;
        self.expect("(")?;
        self.open()?;
        self.expect("weekends")?;
        self.expect("[")?;
        let mut weekends = vec![self.ident("a weekday")?];
        while self.accept(",")? {
            weekends.push(self.ident("a weekday")?);
        }
        self.expect("]")?;
        let mut yearly = Vec::new();
        let mut yearly_from = None;
        if self.at(",", 0) && self.at("yearly", 1) {
            self.skip(2)?;
            self.expect("[")?;
            if !self.at("]", 0) {
                yearly.push(self.month_day()?);
                while self.accept(",")? {
                    yearly.push(self.month_day()?);
                }
            }
            self.expect("]")?;
            if self.accept("from")? {
                yearly_from = Some(self.integer()?);
            }
        }
        let mut added = Vec::new();
        if self.at(",", 0) && self.at("added", 1) {
            self.skip(2)?;
            self.expect("[")?;
            if !self.at("]", 0) {
                added.push(self.calendar_day()?);
                while self.accept(",")? {
                    added.push(self.calendar_day()?);
                }
            }
            self.expect("]")?;
        }
        self.expect(")")?;
        self.close()?;
        Ok(Calendar {
            weekends,
            yearly,
            yearly_from,
            added,
        })
    }

    /// A month-day every year has (L §3.1: never `02-29`).
    fn month_day(&mut self) -> Result<(u8, u8), Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err("a month-day (MM-DD)"));
        };
        let Kind::MonthDay(month, day) = tok.kind else {
            return Err(self.err("a month-day (MM-DD)"));
        };
        if Day::from_ymd(2001, month, day).is_none() {
            return Err(self.cannot(&format!("{} is not a month-day every year has", tok.text)));
        }
        self.skip(1)?;
        Ok((month, day))
    }

    /// An added day: a day, a hole, a window or a choice.
    fn calendar_day(&mut self) -> Result<CalendarDay, Stop> {
        match self.peek(0).map(|tok| (&tok.kind, &tok.text)) {
            Some((Kind::Day(day), _)) => {
                self.skip(1)?;
                Ok(CalendarDay::Day(*day))
            }
            Some((Kind::MonthDay(..), text)) => {
                Err(self.cannot(&format!("a month-day {text} in added; added takes days")))
            }
            _ if self.at("?", 0) && self.at_str(1) => {
                self.skip(1)?;
                Ok(CalendarDay::Hole(self.string()?))
            }
            _ => Ok(CalendarDay::Set(self.day_set()?)),
        }
    }

    /// `window(a, b)` or `one_of([a, …])` (L §4.4).
    pub(super) fn day_set(&mut self) -> Result<DaySet, Stop> {
        if self.at("window", 0) && self.at("(", 1) {
            self.skip(2)?;
            let a = self.day()?;
            self.expect(",")?;
            let b = self.day()?;
            self.expect(")")?;
            return Ok(DaySet::Window(a, b));
        }
        if self.at("one_of", 0) && self.at("(", 1) {
            self.skip(2)?;
            self.expect("[")?;
            let mut days = vec![self.day()?];
            while self.accept(",")? {
                days.push(self.day()?);
            }
            self.expect("]")?;
            self.expect(")")?;
            return Ok(DaySet::OneOf(days));
        }
        Err(self.err("a window or one_of"))
    }

    /// `price = ref | "last" "(" ref ")"`.
    pub(super) fn price(&mut self) -> Result<Price, Stop> {
        if self.at("last", 0) && self.at("(", 1) {
            self.skip(2)?;
            let input = self.reference()?;
            self.expect(")")?;
            return Ok(Price { input, last: true });
        }
        Ok(Price {
            input: self.reference()?,
            last: false,
        })
    }
}
