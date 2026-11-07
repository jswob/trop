//! Load's checker, its comparisons: two sides typed and compared, `pos(k)` guards, key-list variables and period literals as keys, and membership.
//! Spec: L §7.1 and §5.11 (a comparison's sides, `pos(k) > n` a guard of the def's key
//! arithmetic), §2.5 (a bare key takes the other side's index; a period literal a key of a
//! period index of its granularity), §2.1 and §4.2 (a key-list variable as an int or a year,
//! keys as values), §3.1 and §3.2 (what compares: numbers of like units, bools, texts, codes,
//! month-days, records), §5.7 (membership; things compare only by == and !=), §8.3
//! (`LOAD-UNIT`, `LOAD-KEY`, `LOAD-LIMIT`), §8.4.
//! Twin: engine/tools/refcore/check.py (`t_cmp` with its guard, `kl_value`, `compare` with its
//! literal, number and last arms, `t_in`).
//! Invariants: the checks run in refcore's order; `Unseen` on either side gives `Unseen` and
//! refuses nothing; a four-digit literal's reading is decided where refcore decides it.

use super::check::{Checker, Figure, Reading, Scope};
use super::ops::key_digits;
use super::texts::cmp_sign;
use super::types::{
    Operand, Typed, converts, is_date, key_pk, key_value_kind, lit_expr, lit_rule, no_order,
    numcls, period_kind, refused, uerr, unit_of,
};
use crate::base::Load;
use crate::model::{Domain, KindSpec, NameKind, PeriodWord, Reality, Ty, Unit};
use crate::syntax::{CmpOp, Expr, In, Lit, Stop};
use crate::time::PeriodKind;

impl Checker {
    /// A comparison: its sides typed, a bare key or figure taking the other side's type,
    /// then compared (refcore's `t_cmp`). L §7.1, §5.11.
    pub(super) fn cmp(
        &mut self,
        m: &Reality,
        scope: &Scope,
        l: &Expr,
        op: CmpOp,
        r: &Expr,
        depth: u32,
    ) -> Typed {
        let mut left = self.operand(m, scope, l, None, depth)?;
        let mut right_scope = scope.clone();
        if let Expr::Lit(Lit::Num(n)) = r
            && n.whole
            && n.unit.is_none()
            && let Expr::Call(call) = l
            && call.name == "pos"
        {
            key_digits(n)?;
            self.pos_guard(scope, call.args.first(), n)?;
            right_scope.allow_lit = true;
        }
        let open = |t: &Ty| matches!(t, Ty::KeyLit(_) | Ty::Flex(_) | Ty::Year4);
        let expect = (!open(&left.ty)).then(|| left.ty.clone());
        let right = self.operand(m, &right_scope, r, expect.as_ref(), depth)?;
        if open(&left.ty) && !matches!(right.ty, Ty::KeyLit(_)) {
            left = self.operand(m, scope, l, Some(&right.ty), depth)?;
        }
        self.compare(m, scope, op, &left, &right)?;
        Ok(Ty::Bool)
    }

    /// `pos(k) > n` in a def: `n` an offset of the def's key arithmetic on `k` (refcore's
    /// `t_cmp`, its guard). L §7.1, §5.11.
    fn pos_guard(
        &self,
        scope: &Scope,
        arg: Option<&Expr>,
        n: &crate::syntax::Number,
    ) -> Result<(), Stop> {
        let Some(offsets) = self.offsets_of(&scope.item) else {
            return Ok(());
        };
        let ruled = matches!(
            scope.stands,
            super::check::Stands::Rule | super::check::Stands::At
        );
        if !ruled || super::types::is_inline(&n.value) {
            return Ok(());
        }
        let held = match (arg, n.value.to_i64()) {
            (Some(Expr::Name(var)), Some(v)) => {
                offsets.get(var).is_some_and(|set| set.contains(&v))
            }
            (Some(_) | None, Some(_) | None) => false,
        };
        if held {
            return Ok(());
        }
        Err(refused(
            Load::Literal,
            format!(
                "figure {} in a rule; make it a row (a guard holds only an offset of the key arithmetic it protects)",
                n.text
            ),
        ))
    }

    /// A key-list variable outside a key position: a key of the index the other side names,
    /// else an int or a year by its keys (refcore's `kl_value`). L §2.5, §4.2, §2.1.
    pub(super) fn kl_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        side: &Operand,
        use_as: Reading,
        other: Option<&Ty>,
    ) -> Result<Option<Ty>, Stop> {
        let Some(var) = side.var.as_ref().and_then(|v| scope.vars.get(v)) else {
            return Ok(None);
        };
        if !matches!(var.ty, Ty::KeyList) {
            return Ok(None);
        }
        if let Some(Ty::Key(ix)) = other
            && m.indexes.get(ix).is_some()
        {
            let domain = Domain::Index(ix.clone());
            let var = var.clone();
            self.fix_list(m, scope, &var, &domain)?;
            return Ok(Some(Ty::Key(ix.clone())));
        }
        let all_int = var
            .keylist
            .iter()
            .all(|k| matches!(k, Lit::Num(n) if n.whole && n.unit.is_none()));
        if var.keylist.is_empty() || !all_int {
            return Ok(None);
        }
        let ty = match use_as {
            Reading::Year => {
                if !var
                    .keylist
                    .iter()
                    .all(|k| matches!(k, Lit::Num(n) if n.four))
                {
                    return Ok(None);
                }
                Ty::Period(PeriodWord::Year)
            }
            Reading::Int => Ty::Int,
        };
        self.kl_uses.push(super::check::KeyListUse {
            var: var.id,
            name: var.name.clone(),
            reading: use_as,
            at: scope.at.clone(),
        });
        Ok(Some(ty))
    }

    /// Two typed sides compared with `op` (refcore's `compare`). L §2.5, §3.2, §4.2, §5.7.
    fn compare(
        &mut self,
        m: &Reality,
        scope: &Scope,
        op: CmpOp,
        left: &Operand,
        right: &Operand,
    ) -> Result<(), Stop> {
        let mut lt = left.ty.clone();
        let mut rt = right.ty.clone();
        if matches!(lt, Ty::KeyList)
            && let Some(t) = self.kl_value(m, scope, left, Reading::Int, Some(&rt))?
        {
            lt = t;
        }
        if matches!(rt, Ty::KeyList)
            && let Some(t) = self.kl_value(m, scope, right, Reading::Int, Some(&lt))?
        {
            rt = t;
        }
        let (lk, rk) = (lt.word(), rt.word());
        let eq = matches!(op, CmpOp::Eq | CmpOp::Ne);
        if lk == "unseen" || rk == "unseen" || lk == "none" || rk == "none" {
            return Ok(());
        }
        if !eq {
            no_order(m, &lt, "an order")?;
            no_order(m, &rt, "an order")?;
        }
        if lk == "never" || rk == "never" {
            let other = if lk == "never" { &rt } else { &lt };
            if matches!(other, Ty::Date | Ty::Never) || key_value_kind(m, other) == Some("date") {
                return Ok(());
            }
            return Err(uerr(format!(
                "never is a date; compared with {}",
                other.show()
            )));
        }
        for (side, t, other) in [(left, &lt, &rt), (right, &rt, &lt)] {
            if self.period_literal(m, scope, side, t, other)? {
                return Ok(());
            }
        }
        if lk == "year4" || rk == "year4" {
            let (side, other) = if lk == "year4" {
                (left, &rt)
            } else {
                (right, &lt)
            };
            let year = matches!(other, Ty::Period(PeriodWord::Year))
                || key_pk(m, other) == Some(PeriodKind::Year);
            if year {
                self.decide(side.year4, Reading::Year);
                return Ok(());
            }
            if matches!(numcls(m, other), Some("int" | "num" | "flex")) {
                self.decide(side.year4, Reading::Int);
                return Ok(());
            }
            self.year4_now(side.year4, Reading::Int)?;
            return Err(uerr(format!("{} compared with {}", lt.show(), rt.show())));
        }
        compare_rest(m, op, &lt, &rt)
    }

    /// A period literal compared with a key of a period index of its granularity is that
    /// index's key (refcore's `compare`, its literal arm). L §2.5, §7.1.
    fn period_literal(
        &mut self,
        m: &Reality,
        scope: &Scope,
        side: &Operand,
        t: &Ty,
        other: &Ty,
    ) -> Result<bool, Stop> {
        let Ty::Key(ix) = other else {
            return Ok(false);
        };
        let Some(index) = m.indexes.get(ix) else {
            return Ok(false);
        };
        let Some(lit) = &side.lit else {
            return Ok(false);
        };
        let gran = match lit {
            Lit::Num(_) if matches!(t, Ty::Year4) => PeriodKind::Year,
            Lit::Month(..) if matches!(t, Ty::Period(_)) => PeriodKind::Month,
            Lit::Quarter(..) if matches!(t, Ty::Period(_)) => PeriodKind::Quarter,
            Lit::Num(_)
            | Lit::Month(..)
            | Lit::Quarter(..)
            | Lit::Day(_)
            | Lit::MonthDay(..)
            | Lit::Key(_)
            | Lit::Cur(_)
            | Lit::Bool(_)
            | Lit::Inf { .. }
            | Lit::Nominal(_)
            | Lit::Name(_) => return Ok(false),
        };
        let periodic = matches!(
            index.period,
            Some(PeriodKind::Year | PeriodKind::Quarter | PeriodKind::Month)
        );
        if !periodic || index.period != Some(gran) {
            return Ok(false);
        }
        if let Some(entry) = side.year4 {
            let at = self.year4.get(entry).map(|y| y.scope.clone());
            if let Some(at) = at {
                lit_rule(&at, Figure::Date, "", None)?;
            }
            if let Some(y) = self.year4.get_mut(entry) {
                y.keyed = true;
            }
        }
        super::keyed::resolve_lit(m, scope, &lit_expr(lit), &Domain::Index(ix.clone()))?;
        Ok(true)
    }

    /// `x in K`, `x in [a, b]`, `d in dates` (refcore's `t_in`). L §5.7.
    pub(super) fn membership(
        &mut self,
        m: &Reality,
        scope: &Scope,
        test: &In,
        depth: u32,
    ) -> Typed {
        let mut lt = self.expr(m, scope, &test.member, None, depth)?;
        if let Some(on) = &test.on {
            let ot = self.expr(m, scope, on, None, depth)?;
            if !is_date(m, &ot) {
                return Err(uerr(format!(
                    "on of a membership test is {}, not a date",
                    ot.show()
                )));
            }
        }
        if let Expr::Name(id) = &test.set
            && !scope.is_var(id)
        {
            match m.names.kind(id) {
                Some(NameKind::Kind) => {
                    if matches!(lt, Ty::KeyLit(_)) {
                        lt =
                            self.expr(m, scope, &test.member, Some(&Ty::Key(id.clone())), depth)?;
                    }
                    let thing = matches!(&lt, Ty::Key(k) if m.kinds.get(k).is_some());
                    if !thing && !matches!(lt, Ty::Unseen) {
                        return Err(uerr(format!("membership of {} in kind {id}", lt.show())));
                    }
                    return Ok(Ty::Bool);
                }
                Some(NameKind::Index) => {
                    return Err(uerr(format!(
                        "membership in index {id}: x in K is defined for a kind or a list (§5.7)"
                    )));
                }
                Some(_) | None => {}
            }
        }
        if let Expr::List(items) = &test.set {
            if matches!(lt, Ty::Key(_) | Ty::Code(_)) {
                for item in items {
                    self.expr(m, scope, &lit_expr(item), Some(&lt), depth)?;
                }
                return Ok(Ty::Bool);
            }
            if let Ty::KeyLit(written) = &lt {
                return Err(refused(
                    Load::Key,
                    format!("ambiguous key {written}: nothing names its index"),
                )
                .naming(written));
            }
        }
        let rt = self.expr(m, scope, &test.set, None, depth)?;
        if matches!(rt, Ty::Dates) && is_date(m, &lt) {
            return Ok(Ty::Bool);
        }
        if matches!(rt, Ty::List(_) | Ty::Unseen) || matches!(lt, Ty::Unseen) {
            return Ok(Ty::Bool);
        }
        Err(uerr(format!(
            "membership of {} in {}",
            lt.show(),
            rt.show()
        )))
    }
}

/// The comparison of two typed sides past the four-digit and the period-literal arms
/// (refcore's `compare`, its rest). L §3.1, §3.2, §4.2, §5.7.
fn compare_rest(m: &Reality, op: CmpOp, lt: &Ty, rt: &Ty) -> Result<(), Stop> {
    let (lk, rk) = (lt.word(), rt.word());
    let eq = matches!(op, CmpOp::Eq | CmpOp::Ne);
    if let (Some(ka), Some(kb)) = (numcls(m, lt), numcls(m, rt)) {
        return compare_numbers(lt, rt, ka, kb);
    }
    if is_date(m, lt) && is_date(m, rt) {
        return Ok(());
    }
    let pa = if lk == "period" || lk == "key" {
        period_kind(m, lt)
    } else {
        None
    };
    let pb = if rk == "period" || rk == "key" {
        period_kind(m, rt)
    } else {
        None
    };
    let periodic = lk == "period"
        || rk == "period"
        || key_value_kind(m, lt) == Some("period")
        || key_value_kind(m, rt) == Some("period");
    if periodic {
        if pa.is_some() && pa == pb {
            return Ok(());
        }
        return Err(uerr(format!("{} and {} compared", lt.show(), rt.show())));
    }
    if (lk == "unitkey" || rk == "unitkey") && eq {
        let other = if lk == "unitkey" { rt } else { lt };
        let unit_kind =
            matches!(other, Ty::Key(k) if m.kinds.get(k).is_some_and(|k| k.spec == KindSpec::Unit));
        if matches!(other, Ty::UnitKey) || unit_kind {
            return Ok(());
        }
    }
    if let (Ty::Key(a), Ty::Key(b)) = (lt, rt) {
        if a == b {
            return Ok(());
        }
        if m.kinds.get(a).is_some()
            && m.kinds.get(b).is_some()
            && (m.subtree(b).contains(a) || m.subtree(a).contains(b))
        {
            return Ok(());
        }
        return Err(refused(Load::Key, format!("keys of {a} and {b} compared")));
    }
    compare_plain(op, lt, rt)
}

/// Two numbers compared (refcore's `compare`, its number arm). L §3.2.
fn compare_numbers(lt: &Ty, rt: &Ty, ka: &str, kb: &str) -> Result<(), Stop> {
    if ka == "dyn" || kb == "dyn" {
        return Ok(());
    }
    if let (Ty::Key(a), Ty::Key(b)) = (lt, rt) {
        if a != b {
            return Err(refused(Load::Key, format!("keys of {a} and {b} compared")));
        }
        return Ok(());
    }
    if ka == "flex" || kb == "flex" {
        return Ok(());
    }
    if ka == "rate" || kb == "rate" {
        if ka == kb {
            return Ok(());
        }
        return Err(uerr(format!("{} and {} compared", lt.show(), rt.show())));
    }
    let differ = || {
        uerr(format!(
            "{} and {} have different units",
            lt.show(),
            rt.show()
        ))
    };
    if let (Ty::Money(a), Ty::Money(b)) = (lt, rt)
        && a != b
    {
        return Err(differ());
    }
    let measured = |k: &str| k == "num" || k == "money";
    let au = if measured(ka) {
        unit_of(lt)?
    } else {
        Unit::one()
    };
    let bu = if measured(kb) {
        unit_of(rt)?
    } else {
        Unit::one()
    };
    if !converts(&au, &bu)? {
        return Err(differ());
    }
    Ok(())
}

/// Bools, texts, codes, month-days, key literals and records compared (refcore's `compare`,
/// its last arms). L §3.1, §2.5, §8.4.
fn compare_plain(op: CmpOp, lt: &Ty, rt: &Ty) -> Result<(), Stop> {
    let eq = matches!(op, CmpOp::Eq | CmpOp::Ne);
    let (lk, rk) = (lt.word(), rt.word());
    if lk == "bool" && rk == "bool" {
        return Ok(());
    }
    if lk == "text" && rk == "text" {
        if !eq {
            return Err(uerr(
                "text is ordered by nothing; only == and != compare it".to_owned(),
            ));
        }
        return Ok(());
    }
    if let (Ty::Code(a), Ty::Code(b)) = (lt, rt)
        && a == b
    {
        if !eq {
            return Err(uerr("codes compare only with == and !=".to_owned()));
        }
        return Ok(());
    }
    if lk == "mmdd" && rk == "mmdd" {
        if !eq {
            return Err(uerr(format!(
                "{} orders month-days: they compare with == and != only",
                cmp_sign(op)
            )));
        }
        return Ok(());
    }
    for side in [lt, rt] {
        if let Ty::KeyLit(written) = side {
            return Err(refused(
                Load::Key,
                format!("ambiguous key {written}: nothing names its index"),
            )
            .naming(written));
        }
    }
    if lk == "record" || rk == "record" {
        return Err(uerr(
            "a comparison of two records (compare their fields)".to_owned(),
        ));
    }
    Err(uerr(format!("{} and {} compared", lt.show(), rt.show())))
}
