//! Load's checker, its exact forms: `to u` and its step, and the transcendental and rate built-ins, each refused without `to u` where it needs it.
//! Spec: L §7.3 (`to u`, the transcendental calls), §7.1 (precision steps), §3.3
//! (`periodic`, `factor`, `dcf`, `accrue`), §8.3 (`LOAD-TO`, `-UNIT`, `-DOMAIN`, `-PARSE`).
//! Twin: engine/tools/refcore/check2.py (`m_t_to`, `m_step_check`, `m_under_to`, `m_c_exp`,
//! `m_c_sqrt`, `m_c_pow`, `m_period_arg`, `m_code_arg`, `m_rate_arg`, `m_rate_lit_period`,
//! `m_day_pairing`, `m_c_periodic`, `m_c_factor`, `m_factor_power_int`, `m_c_dcf`,
//! `m_c_accrue`).
//! Invariants: each call is typed in refcore's order with refcore's messages; a type load
//! doesn't see (`Unseen`) refuses nothing; `to u` marks only the form directly under it.

use super::calls::arity;
use super::check::{Checker, Scope, invariant};
use super::random::strip;
use super::texts::text;
use super::types::{
    Typed, converts, key_value_kind, lit_expr, num, numcls, refused, uerr, unit_error, unit_of,
    unit_stop,
};
use crate::base::Load;
use crate::model::{CodeKind, NameKind, Node, NodeKind, Reality, Rel, Ty, Unit};
use crate::num::Num;
use crate::syntax::{Call, CmpOp, Expr, Key, Lit, Postfix, Solve, Step, Stop};

/// The transcendental and rate calls `to u` may stand over (refcore's `TRANS_CALLS`).
pub(super) const TRANS_CALLS: [&str; 7] =
    ["exp", "ln", "sqrt", "pow", "periodic", "factor", "accrue"];

/// The period words a rate built-in takes (refcore's `PERIOD_WORDS`).
const PERIOD_WORDS: [&str; 4] = ["day", "month", "quarter", "year"];

/// Whether a form stands directly under `to u` (refcore's `_under_to`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Under {
    /// Under `to u`.
    To,
    /// Bare.
    Bare,
}

impl Under {
    /// What the checker's mark says, taken: the next form typed is the one under it.
    pub(super) fn take(mark: &mut bool) -> Under {
        if core::mem::take(mark) {
            Under::To
        } else {
            Under::Bare
        }
    }
}

/// `LOAD-TO` unless the form stands under `to u` (refcore's `under_to`). L §7.3.
pub(super) fn under_to(under: Under, what: &str) -> Result<(), Stop> {
    if under == Under::To {
        return Ok(());
    }
    Err(refused(Load::To, format!("{what} needs to u")))
}

/// Whether `t` is a plain number, or a type load doesn't see.
fn plain(m: &Reality, t: &Ty) -> Result<bool, Stop> {
    if matches!(t, Ty::Unseen) {
        return Ok(true);
    }
    let c = numcls(m, t);
    if !matches!(c, Some("num" | "int" | "flex")) {
        return Ok(false);
    }
    Ok(c != Some("num") || unit_of(t)?.is_one())
}

/// The unit of a number type, none for anything else.
fn num_unit(m: &Reality, t: &Ty) -> Result<Unit, Stop> {
    if matches!(numcls(m, t), Some("num" | "money")) {
        return unit_of(t);
    }
    Ok(Unit::one())
}

/// The call's argument at `at`, which `arity` has counted.
fn arg(call: &Call, at: usize) -> Result<&Expr, Stop> {
    call.args
        .get(at)
        .ok_or_else(|| invariant(format!("{} has fewer arguments", call.name)))
}

impl Checker {
    /// `e to u`: a transcendental value, a distribution, a function of one or a query over
    /// one, rounded to a positive step of its unit (refcore's `t_to`). L §7.3, §7.1.
    pub(super) fn to(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        step: &Step,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let affx = core::mem::take(&mut self.affx_hit);
        let core = strip(inner);
        let trans = matches!(core, Expr::Call(call) if TRANS_CALLS.contains(&call.name.as_str()));
        let rounded = matches!(core, Expr::DistFn(_))
            || matches!(core, Expr::Query(q) if q.binds.is_none())
            || matches!(core, Expr::Crps(c) if c.bind.is_none());
        let is_dist = matches!(core, Expr::Dist(_));
        if !(is_dist || trans || rounded) {
            return Err(uerr(format!(
                "to u on {}: `to` states the precision of a transcendental value (L §7.3)",
                text(core)
            )));
        }
        let want = expect.filter(|t| matches!(t, Ty::Dist(_)));
        self.under_to = true;
        let typed = self.expr(m, scope, core, if is_dist { want } else { None }, depth);
        self.under_to = false;
        let t = typed?;
        let vu = if is_dist {
            unit_of(&t)?
        } else {
            num_unit(m, &t)?
        };
        self.step_check(m, scope, step, &vu, depth)?;
        if matches!(t, Ty::Money(_)) {
            return Ok(num(vu));
        }
        if is_dist && affx {
            return Ok(num(unit_of(&t)?));
        }
        Ok(t)
    }

    /// The step of `to u`: positive, of the value's unit; a bare step takes it; its value in
    /// the value's unit where it is a literal (refcore's `step_check`, `u._val`). L §7.1
    /// precision steps.
    pub(super) fn step_check(
        &mut self,
        m: &Reality,
        scope: &Scope,
        step: &Step,
        vu: &Unit,
        depth: u32,
    ) -> Result<Option<Num>, Stop> {
        let (written, t) = match step {
            Step::Number(n) => {
                let value = step_value(m, n, vu)?;
                if value.is_zero() || value.is_negative() {
                    return Err(refused(
                        Load::Domain,
                        format!("to {}: a step that is not positive", n.text),
                    ));
                }
                return Ok(Some(value));
            }
            Step::Minor(cur) => (format!("minor({cur})"), Ty::Money(cur.clone())),
            Step::Ref(e) => {
                let mut stepped = scope.child();
                stepped.allow_lit = true;
                (text(e), self.expr(m, &stepped, e, None, depth)?)
            }
        };
        if matches!(t, Ty::Unseen) {
            return Ok(None);
        }
        let c = numcls(m, &t);
        let tu = if matches!(c, Some("num" | "money")) {
            Some(unit_of(&t)?)
        } else if matches!(c, Some("int" | "flex")) {
            Some(Unit::one())
        } else {
            None
        };
        let fits = match &tu {
            Some(tu) => converts(tu, vu)?,
            None => false,
        };
        if !fits {
            return Err(uerr(format!(
                "to {written}: the step ({}) and the value have different units",
                t.show()
            )));
        }
        Ok(None)
    }

    /// The transcendental and rate built-ins, one arm each (refcore's `c_exp` to
    /// `c_accrue`). L §7.3, §3.3.
    pub(super) fn exact_call(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        depth: u32,
    ) -> Typed {
        let under = Under::take(&mut self.under_to);
        match call.name.as_str() {
            "sqrt" => self.sqrt(m, scope, call, under, depth),
            "pow" => self.pow(m, scope, call, depth),
            "periodic" => self.periodic(m, scope, call, depth),
            "factor" => self.factor(m, scope, call, under, depth),
            "dcf" => self.dcf(m, scope, call, depth),
            "accrue" => self.accrue(m, scope, call, depth),
            _ => self.exp(m, scope, call, under, depth),
        }
    }

    /// `exp(x)`, `ln(x)`: of a plain number, under `to u` (refcore's `c_exp`). L §7.3.
    fn exp(&mut self, m: &Reality, scope: &Scope, call: &Call, under: Under, depth: u32) -> Typed {
        arity(call, 1)?;
        let t = self.expr(m, scope, arg(call, 0)?, None, depth)?;
        if !plain(m, &t)? {
            return Err(uerr(format!(
                "{} of {}: its argument is a plain number",
                call.name,
                t.show()
            )));
        }
        under_to(under, &call.name)?;
        Ok(num(Unit::one()))
    }

    /// `sqrt(x)`: its unit's powers halved, under `to u` (refcore's `c_sqrt`). L §7.3.
    fn sqrt(&mut self, m: &Reality, scope: &Scope, call: &Call, under: Under, depth: u32) -> Typed {
        arity(call, 1)?;
        let t = self.expr(m, scope, arg(call, 0)?, None, depth)?;
        if matches!(t, Ty::Unseen) {
            under_to(under, "sqrt")?;
            return Ok(Ty::Unseen);
        }
        if !matches!(numcls(m, &t), Some("num" | "int" | "flex" | "money")) {
            return Err(uerr(format!("sqrt of {}", t.show())));
        }
        let unit = num_unit(m, &t)?;
        let mut halves = Vec::new();
        for (base, power) in unit.terms() {
            if power.checked_rem_euclid(2) != Some(0) {
                return Err(uerr(format!(
                    "sqrt of {}: its unit has an odd power",
                    t.show()
                )));
            }
            let half = power
                .checked_div_euclid(2)
                .ok_or_else(|| invariant("a unit power that doesn't halve".to_owned()))?;
            halves.push((base.clone(), half));
        }
        under_to(under, "sqrt")?;
        Ok(num(Unit::make(halves).map_err(unit_stop)?))
    }

    /// `pow(x, n)`: an integer power, a literal where x has a unit; a power that may not be
    /// an integer is a real (refcore's `c_pow`). L §7.3, §7.1.
    fn pow(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 2)?;
        let xt = self.expr(m, scope, arg(call, 0)?, None, depth)?;
        let nt = self.expr(m, scope, arg(call, 1)?, None, depth)?;
        if matches!(xt, Ty::Unseen) || matches!(nt, Ty::Unseen) {
            return Ok(Ty::Unseen);
        }
        let c = numcls(m, &xt);
        if c.is_none() || c == Some("rate") {
            return Err(uerr(format!("pow of {}", xt.show())));
        }
        let x_unit = num_unit(m, &xt)?;
        let nc = numcls(m, &nt);
        if !matches!(nc, Some("int" | "flex")) {
            if nc == Some("num") && unit_of(&nt)?.is_one() {
                if !x_unit.is_one() {
                    return Err(uerr(format!(
                        "pow of {} to a power that may not be an integer: its unit would have a fractional power",
                        xt.show()
                    )));
                }
                return Ok(num(Unit::one()));
            }
            return Err(uerr(format!("pow: the power is {}", nt.show())));
        }
        if !x_unit.is_one() {
            let Some(p) = literal_int(arg(call, 1)?)? else {
                return Err(uerr(format!(
                    "pow of {} to a power that is not a literal",
                    xt.show()
                )));
            };
            return Ok(num(x_unit.power(&p).map_err(unit_error)?));
        }
        Ok(num(x_unit))
    }

    /// `periodic(r, P, c)`: a rate over a period word with a compounding code (refcore's
    /// `c_periodic`). L §3.3.
    fn periodic(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 3)?;
        self.rate_arg(m, scope, call, 0, depth)?;
        let period = period_arg(call, 1)?;
        self.code_arg(m, scope, call, 2, CodeKind::Compounding, depth)?;
        day_pairing(call, rate_lit_period(arg(call, 0)?), period)?;
        Ok(num(Unit::one()))
    }

    /// `factor(r, n, P, c)`: exact when load shows its whole power is an integer, else under
    /// `to u` (refcore's `c_factor`). L §3.3.
    fn factor(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        under: Under,
        depth: u32,
    ) -> Typed {
        arity(call, 4)?;
        self.rate_arg(m, scope, call, 0, depth)?;
        let nt = self.expr(m, scope, arg(call, 1)?, None, depth)?;
        if !plain(m, &nt)? {
            return Err(uerr(format!(
                "factor: n is {}, not a plain number",
                nt.show()
            )));
        }
        let period = period_arg(call, 2)?;
        self.code_arg(m, scope, call, 3, CodeKind::Compounding, depth)?;
        let per = rate_lit_period(arg(call, 0)?);
        day_pairing(call, per, period)?;
        if under == Under::Bare && !factor_power_int(m, call, &nt, per, period)? {
            return Err(refused(
                Load::To,
                "factor needs to u: load can't show its power is an integer".to_owned(),
            ));
        }
        Ok(num(Unit::one()))
    }

    /// `dcf(a, b, d)`: two dates and a day count (refcore's `c_dcf`). L §3.3.
    fn dcf(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 3)?;
        self.date_arg(m, scope, arg(call, 0)?, &call.name, "a", depth)?;
        self.date_arg(m, scope, arg(call, 1)?, &call.name, "b", depth)?;
        self.code_arg(m, scope, call, 2, CodeKind::Daycount, depth)?;
        Ok(num(Unit::one()))
    }

    /// `accrue(p, r, a, b, d, c)`: a principal's accrual, of its unit (refcore's
    /// `c_accrue`). L §3.3.
    fn accrue(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Typed {
        arity(call, 6)?;
        let pt = self.expr(m, scope, arg(call, 0)?, None, depth)?;
        let principal = matches!(numcls(m, &pt), Some("num" | "money" | "int" | "flex"));
        if !principal && !matches!(pt, Ty::Unseen) {
            return Err(uerr(format!("accrue: the principal is {}", pt.show())));
        }
        self.rate_arg(m, scope, call, 1, depth)?;
        self.date_arg(m, scope, arg(call, 2)?, &call.name, "a", depth)?;
        self.date_arg(m, scope, arg(call, 3)?, &call.name, "b", depth)?;
        self.code_arg(m, scope, call, 4, CodeKind::Daycount, depth)?;
        self.code_arg(m, scope, call, 5, CodeKind::Compounding, depth)?;
        if matches!(pt, Ty::Unseen) {
            return Ok(Ty::Unseen);
        }
        Ok(num(num_unit(m, &pt)?))
    }

    /// A rate argument (refcore's `rate_arg`). L §3.3.
    fn rate_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        at: usize,
        depth: u32,
    ) -> Result<(), Stop> {
        let a = arg(call, at)?;
        let t = self.expr(m, scope, a, None, depth)?;
        if t != Ty::Rate && !matches!(t, Ty::Unseen) {
            return Err(uerr(format!(
                "{}: {} is {}, not a rate",
                call.name,
                text(a),
                t.show()
            )));
        }
        Ok(())
    }

    /// A code argument of the kind `code` (refcore's `code_arg`). L §3.1.
    fn code_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        at: usize,
        code: CodeKind,
        depth: u32,
    ) -> Result<(), Stop> {
        let a = arg(call, at)?;
        let want = Ty::Code(code);
        let t = self.expr(m, scope, a, Some(&want), depth)?;
        if t != want && !matches!(t, Ty::Unseen) {
            return Err(uerr(format!(
                "{}: {} is not a {} code",
                call.name,
                text(a),
                code.name()
            )));
        }
        Ok(())
    }
}

/// A literal step's value in the value's unit `vu` (refcore's `step_check`, `u._val`).
/// L §7.1.
fn step_value(m: &Reality, n: &crate::syntax::Number, vu: &Unit) -> Result<Num, Stop> {
    let Some(unit) = &n.unit else {
        return Ok(n.value.clone());
    };
    let (su, factor) = step_unit(m, unit)?;
    let Some(conv) = su.conv(vu).map_err(unit_stop)? else {
        let shown = if vu.is_one() {
            "num".to_owned()
        } else {
            vu.text()
        };
        return Err(uerr(format!(
            "to {}: the step and the value ({shown}) have different units",
            n.text
        )));
    };
    n.value
        .checked_mul(&factor)
        .and_then(|v| v.checked_mul(&conv))
        .map_err(unit_stop)
}

/// A precision step's written unit, resolved (refcore's `unit_terms`, `from_syntax`).
fn step_unit(m: &Reality, unit: &crate::syntax::Unit) -> Result<(Unit, Num), Stop> {
    let mut terms = Vec::new();
    for term in &unit.terms {
        let base = term.base.as_str();
        let known = base == "%"
            || m.currencies.contains_key(base)
            || m.units.contains(base)
            || crate::model::PERIODS.contains(&base)
            || m.unit_kind_keys.contains_key(base);
        if !known {
            return Err(refused(Load::Name, format!("unknown id {base} (a unit)")).naming(base));
        }
        terms.push((term.base.clone(), term.power.clone()));
    }
    Unit::from_syntax(terms).map_err(unit_error)
}

/// An integer literal of any size, or a negated one (refcore's `c_pow`, its literal power).
fn literal_int(e: &Expr) -> Result<Option<Num>, Stop> {
    let inner = strip(e);
    let (lit, negative) = if let Expr::Neg(neg) = inner {
        (&**neg, true)
    } else {
        (inner, false)
    };
    let Expr::Lit(Lit::Num(n)) = lit else {
        return Ok(None);
    };
    if !n.whole {
        return Ok(None);
    }
    if negative {
        return n.value.checked_neg().map(Some).map_err(unit_stop);
    }
    Ok(Some(n.value.clone()))
}

/// A period word argument (refcore's `period_arg`). L §3.3.
fn period_arg(call: &Call, at: usize) -> Result<&'static str, Stop> {
    let a = strip(arg(call, at)?);
    if let Expr::Name(word) = a
        && let Some(found) = PERIOD_WORDS.iter().find(|p| *p == word)
    {
        return Ok(found);
    }
    Err(uerr(format!(
        "{}: the period argument is a period word (day, month, quarter or year), not {}",
        call.name,
        text(a)
    )))
}

/// The period of a rate literal load sees, `1 %/month` (refcore's `rate_lit_period`).
fn rate_lit_period(e: &Expr) -> Option<&'static str> {
    let Expr::Lit(Lit::Num(n)) = strip(e) else {
        return None;
    };
    let unit = n.unit.as_ref()?;
    let terms = unit
        .terms
        .iter()
        .map(|t| (t.base.clone(), t.power.clone()))
        .collect();
    let (made, _) = Unit::from_syntax(terms).ok()?;
    let period = made.period_of()?;
    PERIOD_WORDS.iter().find(|p| **p == period).copied()
}

/// A daily rate over a longer period, or another rate over a day, refused where load sees
/// the rate's period (refcore's `day_pairing`). L §3.3.
fn day_pairing(call: &Call, per: Option<&str>, period: &str) -> Result<(), Stop> {
    let Some(per) = per else {
        return Ok(());
    };
    if (period == "day" || per == "day") && !(period == "day" && per == "day") {
        return Err(uerr(format!(
            "{}: a {per} rate over a {period}: a day's share of a longer period is a day count (dcf), never a default",
            call.name
        )));
    }
    Ok(())
}

/// Whether load shows `factor`'s whole power is an integer: t·n (effective, discount),
/// m·t·n (nominal m), n (simple), t the period's length in the rate's periods (refcore's
/// `factor_power_int`). L §3.3.
fn factor_power_int(
    m: &Reality,
    call: &Call,
    nt: &Ty,
    per: Option<&str>,
    period: &str,
) -> Result<bool, Stop> {
    let whole = matches!(nt, Ty::Int)
        || matches!(nt, Ty::Flex(n) if n.is_integer())
        || (matches!(nt, Ty::Key(_)) && key_value_kind(m, nt) == Some("int"));
    if !whole {
        return Ok(false);
    }
    let code = strip(arg(call, 3)?);
    let word = if let Expr::Name(word) | Expr::Lit(Lit::Name(word) | Lit::Key(word)) = code {
        word.as_str()
    } else {
        ""
    };
    if word == "simple" {
        return Ok(true);
    }
    let multiple = if let Expr::Lit(Lit::Nominal(n)) = code {
        n.clone()
    } else if word == "effective" || word == "discount" {
        Num::int(1)
    } else {
        return Ok(false);
    };
    let months = |p: &str| -> i64 {
        match p {
            "year" => 12,
            "quarter" => 3,
            _ => 1,
        }
    };
    let lengths: Vec<Num> = if period == "day" {
        vec![Num::int(1)]
    } else if let Some(per) = per {
        vec![Num::ratio(months(period), months(per)).map_err(unit_stop)?]
    } else {
        let mut all = Vec::new();
        for each in ["year", "quarter", "month"] {
            all.push(Num::ratio(months(period), months(each)).map_err(unit_stop)?);
        }
        all
    };
    for t in lengths {
        if !multiple.checked_mul(&t).map_err(unit_stop)?.is_integer() {
            return Ok(false);
        }
    }
    Ok(true)
}

impl Checker {
    /// `solve f == g for v in [lo, hi] to u`: v an input or a def, pinned at each step, the
    /// bracket and the step of v's type, the bracket on the grid lo + i u (refcore's
    /// `check_solve`). L §5.18.
    pub(super) fn check_solve(
        &mut self,
        m: &Reality,
        scope: &Scope,
        s: &Solve,
        depth: u32,
    ) -> Typed {
        let (name, keys): (&String, &[Key]) = if let Expr::Name(name) = &s.target {
            (name, &[])
        } else if let Expr::Post(base, posts) = &s.target
            && let (Expr::Name(name), [Postfix::Keys(keys)]) = (&**base, posts.as_slice())
        {
            (name, keys.as_slice())
        } else {
            return Err(not_a_target(&s.target));
        };
        let kind = m.names.kind(name);
        let Some(kind) = kind else {
            return Err(refused(Load::Name, format!("unknown id {name}")).naming(name));
        };
        let (target, binds, declared) = if kind == NameKind::Input {
            let input = m
                .inputs
                .get(name)
                .ok_or_else(|| invariant(format!("input {name} isn't built")))?;
            (
                Node::new(NodeKind::Input, name),
                &input.binds,
                input.ty.clone(),
            )
        } else if kind == NameKind::Def {
            let def = m
                .defs
                .get(name)
                .ok_or_else(|| invariant(format!("def {name} isn't built")))?;
            (
                Node::new(NodeKind::Def, name),
                &def.binds,
                Some(def.ty.clone()),
            )
        } else {
            return Err(refused(
                Load::AsIf,
                format!("solve for {name}, which is not an input or a def"),
            ));
        };
        if keys.len() != binds.len() {
            return Err(refused(
                Load::Key,
                format!("wrong number of keys for {name}"),
            ));
        }
        for (key, bind) in keys.iter().zip(binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        let shown = match &declared {
            Some(t) => t.show(),
            None => "None".to_owned(),
        };
        let numeric = |t: &Ty| matches!(t, Ty::Num(..) | Ty::Money(_) | Ty::Int | Ty::Rate);
        let Some(vt) = declared.filter(numeric) else {
            return Err(uerr(format!(
                "solve for {name} of type {shown}: a root is searched over numbers"
            )));
        };
        self.edge(&scope.item, target, Rel::Pin);
        let ft = self.expr(m, scope, &s.left, None, depth)?;
        let gt = self.expr(m, scope, &s.right, Some(&ft), depth)?;
        let ft = if matches!(ft, Ty::Flex(_)) {
            self.expr(m, scope, &s.left, Some(&gt), depth)?
        } else {
            ft
        };
        self.cmp(m, scope, &s.left, CmpOp::Eq, &s.right, depth)?;
        let side = |t: &Ty| numcls(m, t).is_some() || matches!(t, Ty::Hole | Ty::Unseen);
        if !side(&ft) || !side(&gt) {
            return Err(uerr(format!(
                "solve {} == {}: F = f - g is a number",
                ft.show(),
                gt.show()
            )));
        }
        let mut bracketed = scope.child();
        bracketed.allow_lit = true;
        let mut ends = Vec::new();
        for end in [&s.lo, &s.hi] {
            let e = lit_expr(end);
            if !super::pins::pin_literal(m, &bracketed, &e, &vt)? {
                let t = self.expr(m, &bracketed, &e, Some(&vt), depth)?;
                self.assign(m, &bracketed, &vt, &e, &t, "the bracket of solve")?;
            }
            ends.push(lit_value(end, &vt)?);
        }
        let step = if vt == Ty::Rate {
            let Step::Number(n) = &s.unit else {
                return Err(uerr(
                    "solve for a rate: the step states its period (to 0.0001 %/year)".to_owned(),
                ));
            };
            let Some(unit) = &n.unit else {
                return Err(uerr(
                    "solve for a rate: the step states its period (to 0.0001 %/year)".to_owned(),
                ));
            };
            let (su, factor) = step_unit(m, unit)?;
            if su.period_of().is_none() {
                return Err(uerr(format!(
                    "solve for a rate: the step {} is not a rate",
                    n.text
                )));
            }
            Some((n.value.checked_mul(&factor).map_err(unit_stop)?, su))
        } else {
            let vu = num_unit(m, &vt)?;
            self.step_check(m, &bracketed, &s.unit, &vu, depth)?
                .map(|value| (value, vu))
        };
        if let ([Some((lq, lu)), Some((hq, hu))], Some((step, su))) = (ends.as_slice(), &step) {
            if vt == Ty::Rate && (lu != hu || lu != su) {
                return Err(uerr(
                    "solve: the bracket and the step are rates of different periods".to_owned(),
                ));
            }
            if hq.compare(lq).map_err(unit_stop)? == core::cmp::Ordering::Less {
                return Err(refused(
                    Load::Domain,
                    "solve: the bracket [lo, hi] has hi below lo".to_owned(),
                ));
            }
            let steps = hq
                .checked_sub(lq)
                .and_then(|d| d.checked_div(step))
                .map_err(unit_stop)?;
            if !steps.is_integer() {
                return Err(refused(
                    Load::Domain,
                    "solve: hi is not on the grid lo + i u".to_owned(),
                ));
            }
        }
        Ok(vt)
    }
}

/// `LOAD-PARSE`: `solve` names an input or a def, read at keys or not.
fn not_a_target(e: &Expr) -> Stop {
    refused(
        Load::Parse,
        format!(
            "cannot parse: solve names an input or a def, not {}",
            text(e)
        ),
    )
}

/// A bracket literal's value and unit, when it is a number literal; a rate's needs its
/// period (refcore's `_lit_value`). L §5.18.
fn lit_value(end: &Lit, vt: &Ty) -> Result<Option<(Num, Unit)>, Stop> {
    let Lit::Num(n) = end else {
        return Ok(None);
    };
    let Some(unit) = &n.unit else {
        if *vt == Ty::Rate {
            return Ok(None);
        }
        return Ok(Some((n.value.clone(), Unit::one())));
    };
    let terms = unit
        .terms
        .iter()
        .map(|t| (t.base.clone(), t.power.clone()))
        .collect();
    let (made, factor) = Unit::from_syntax(terms).map_err(unit_error)?;
    Ok(Some((
        n.value.checked_mul(&factor).map_err(unit_stop)?,
        made,
    )))
}
