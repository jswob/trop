//! Load's checker, its bindings: a binding's variable, what it ranges over and its type, its `on`, `over` and `where`; a stepped range's type; a value assigned to a declared type.
//! Spec: L §5.6 (a binding is over an index, a kind, a key list or a stepped range; `on` and
//! `over` state when a thing of a kind is alive, so no other binding has them; a range of
//! quantities states its step), §2.5 (a bound variable never takes the name of a top-level
//! id, of a variable in scope or of a key of its own domain), §5.7 and §5.14.3 (a binding over
//! a kind or over movements needs `on` or `over`), §5.19 (an aggregate of observations is
//! guarded by `covered()`), §3.1–§3.2 and §7.1 (a value of a declared type, a bare figure's
//! precision), §8.3 (`LOAD-SHADOW`, `LOAD-NAME`, `LOAD-ALIVE`, `LOAD-DOMAIN`, `LOAD-KEY`,
//! `LOAD-COVERED`, `LOAD-UNIT`, `LOAD-PRECISION`, `LOAD-FIELD`).
//! Twin: engine/tools/refcore/check.py (`bind`, `bind_finish`, `range_type`, `assign`);
//! check3.py (`m_bind_movements`, its `LOAD-ALIVE` and `LOAD-COVERED`).
//! Invariants: the checks run in refcore's order, each refusal with no place (the walk puts
//! it at its line); a type `Unseen` is assigned to anything and its range refuses nothing.

use super::check::{Checker, Scope, Stands, VarDom};
use super::ops::unify;
use super::steps::num_stop;
use super::types::{
    Typed, converts, int_flex, is_bool, is_date, is_duration, key_value_kind, no_order,
    period_kind, span_text, text, uerr, unit_of,
};
use crate::base::{Code, Load, Refusal};
use crate::model::{KindSpec, NameKind, Node, NodeKind, Reality, Rel, Shown, Ty};
use crate::syntax::{Bind, Domain as Written, Expr, Lit, Stop};
use crate::time::PeriodKind;

/// How a binding is checked: as any binding, or as a fill's key bind, which is defined for
/// every key and so needs no `on` or `over` (refcore's `bind`, its `key_bind`). L §5.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BindAs {
    /// Any binding.
    Plain,
    /// A fill's key bind.
    KeyBind,
}

/// What a binding ranges over, as refcore's `bind` resolves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Over {
    /// The keys of an index.
    Index(String),
    /// The things of a kind that isn't an event kind.
    Things(String),
    /// The movements of an event kind.
    Movements(String),
    /// The rows of an observed input.
    Observed(String),
    /// A key list's keys.
    List,
    /// A stepped range's values.
    Range,
}

impl Over {
    /// Whether things or movements are bound, which are alive on some days: only then may a
    /// binding state `on` or `over`. L §5.6.
    fn is_alive(&self) -> bool {
        match self {
            Over::Things(_) | Over::Movements(_) | Over::Observed(_) => true,
            Over::Index(_) | Over::List | Over::Range => false,
        }
    }
}

/// `LOAD-SHADOW` for a binding whose variable is already bound around it, refcore's first
/// check of a binding; the walk, which holds the variables in scope, makes it. L §2.5.
pub(super) fn bound_twice(var: &str) -> Stop {
    refused(
        Load::Shadow,
        format!("bound variable {var} has the name of a bound variable in scope"),
    )
}

/// What `bind` ranges over, its variable no top-level id and no key of its domain (refcore's
/// `bind`, after its scope check, up to its `bind_finish`). L §5.6, §2.5.
///
/// # Errors
///
/// `LOAD-SHADOW`, `LOAD-NAME` (a prefixed name included), `LOAD-ALIVE`, `LOAD-DOMAIN` or
/// `LOAD-KEY`, with no place.
pub(super) fn over(m: &Reality, bind: &Bind, key_bind: BindAs) -> Result<Over, Stop> {
    let var = &bind.var;
    if m.names.contains_key(var) {
        return Err(refused(
            Load::Shadow,
            format!("bound variable {var} has the name of a top-level id"),
        ));
    }
    match &bind.domain {
        // A joint model's merge has made each member's index or kind one name; a prefix left
        // names nothing (refcore's `qualified`, GD-1).
        Written::Name { id, qual: Some(_) } => {
            Err(refused(Load::Name, format!("unknown id {id}")).naming(id))
        }
        Written::Name { id, qual: None } => over_name(m, bind, id, key_bind),
        Written::List(keys) => {
            if keys.iter().any(|key| key_text(key) == *var) {
                return Err(refused(
                    Load::Shadow,
                    format!("bound variable {var} has the spelling of a key of its list"),
                ));
            }
            Ok(Over::List)
        }
        Written::Range(_) => Ok(Over::Range),
    }
}

/// A binding over a name: an index, a kind, an event kind's movements or an observed input's
/// rows (refcore's `bind`, its `dname` arm, and `m_bind_movements`). L §5.6, §5.7, §5.14.3.
fn over_name(m: &Reality, bind: &Bind, id: &str, key_bind: BindAs) -> Result<Over, Stop> {
    let var = &bind.var;
    let alive = bind.on.is_some() || bind.over.is_some() || key_bind == BindAs::KeyBind;
    let needs_alive = || refused(Load::Alive, format!("binding over {id} needs on or over"));
    match m.names.get(id) {
        None => Err(refused(Load::Name, format!("unknown id {id}")).naming(id)),
        Some(NameKind::Index) => {
            if m.indexes
                .get(id)
                .is_some_and(|ix| ix.by_text(var).is_some())
            {
                return Err(key_spelling(var, id));
            }
            Ok(Over::Index(id.to_owned()))
        }
        Some(NameKind::Kind) => {
            let event = m.kinds.get(id).is_some_and(|k| k.spec == KindSpec::Event);
            if event {
                return if alive {
                    Ok(Over::Movements(id.to_owned()))
                } else {
                    Err(needs_alive())
                };
            }
            if m.thing_in(var, id) {
                return Err(key_spelling(var, id));
            }
            if !alive {
                return Err(needs_alive());
            }
            Ok(Over::Things(id.to_owned()))
        }
        Some(NameKind::Input) if m.inputs.get(id).is_some_and(|i| i.observed) => {
            if bind.on.is_some() || bind.over.is_some() {
                Ok(Over::Observed(id.to_owned()))
            } else {
                Err(needs_alive())
            }
        }
        Some(NameKind::Axis) => Err(refused(
            Load::Domain,
            format!(
                "a binding over the axis {id}: a bind is over an index, a kind, a key list or a stepped range (L §5.6)"
            ),
        )),
        Some(
            NameKind::Unit
            | NameKind::Input
            | NameKind::Def
            | NameKind::Band
            | NameKind::Place
            | NameKind::Fill
            | NameKind::Move
            | NameKind::Version
            | NameKind::Ask
            | NameKind::Source
            | NameKind::Cut,
        ) => Err(refused(
            Load::Key,
            format!("{id} is not an index, a kind, a key list or a range"),
        )),
    }
}

/// `on` or `over` (`word`) only on a binding over things or movements (refcore's
/// `bind_finish`, its domain checks, each after its clause's own check). L §5.6.
///
/// # Errors
///
/// `LOAD-DOMAIN`, with no place.
pub(super) fn alive_clause(word: &str, over: &Over) -> Result<(), Stop> {
    if over.is_alive() {
        return Ok(());
    }
    Err(refused(
        Load::Domain,
        format!("{word} on a binding that is not over a kind"),
    ))
}

impl Checker {
    /// A binding: its checks and what it ranges over, then its `on` (a date) and `over` (a
    /// span), each checked as allowed, then its `where` (a bool), its variable in scope; the
    /// variable's type (refcore's `bind`, `bind_finish`). L §5.6, §5.19.
    pub(super) fn bind(
        &mut self,
        m: &Reality,
        scope: &mut Scope,
        bind: &Bind,
        depth: u32,
    ) -> Typed {
        self.bind_as(m, scope, bind, BindAs::Plain, depth)
    }

    /// A binding, as [`Checker::bind`]; a fill's key bind (`key_bind`) is defined for every
    /// key, so its binding over a kind or movements needs no `on` or `over` (refcore's
    /// `bind`, its `key_bind`). L §5.6, §5.12.
    pub(super) fn bind_as(
        &mut self,
        m: &Reality,
        scope: &mut Scope,
        bind: &Bind,
        key_bind: BindAs,
        depth: u32,
    ) -> Typed {
        if scope.is_var(&bind.var) {
            return Err(scope.place(bound_twice(&bind.var)));
        }
        let over_what = over(m, bind, key_bind).map_err(|stop| scope.place(stop))?;
        let mut keylist = Vec::new();
        let (ty, dom) = match &over_what {
            Over::Index(ix) => (Ty::Key(ix.clone()), VarDom::Index(ix.clone())),
            Over::Things(kind) => {
                self.bind_places(m, scope, kind);
                (Ty::Key(kind.clone()), VarDom::Kind(kind.clone()))
            }
            Over::Movements(kind) => {
                self.bind_movements(m, scope, kind);
                (Ty::Key(kind.clone()), VarDom::Event(kind.clone()))
            }
            Over::Observed(id) => {
                let input = Node::new(NodeKind::Input, id);
                self.edge(&scope.item, input.clone(), Rel::Bind);
                self.edge(&scope.item, input, Rel::Named);
                covered(scope, bind).map_err(|stop| scope.place(stop))?;
                (Ty::Key("events".to_owned()), VarDom::Obs(id.clone()))
            }
            Over::List => {
                if let Written::List(keys) = &bind.domain {
                    keylist.clone_from(keys);
                }
                (Ty::KeyList, VarDom::List)
            }
            Over::Range => {
                let Written::Range(range) = &bind.domain else {
                    return Ok(Ty::Unseen);
                };
                (self.range_type(m, scope, range, depth)?, VarDom::Range)
            }
        };
        let mut var = self.new_var(&bind.var, ty.clone(), dom, keylist)?;
        if let Written::Range(range) = &bind.domain {
            var.end = Some(range.1.clone());
        }
        if matches!(over_what, Over::Things(_)) {
            scope.vars.insert(bind.var.clone(), var.clone());
        }
        if let Some(on) = &bind.on {
            let t = self.expr(m, scope, on, None, depth)?;
            if !is_date(m, &t) {
                let message = format!("on of a binding is {}, not a date", t.show());
                return Err(scope.place(uerr(message)));
            }
            alive_clause("on", &over_what).map_err(|stop| scope.place(stop))?;
        }
        if let Some(span) = &bind.over {
            self.span(m, scope, span, depth)?;
            alive_clause("over", &over_what).map_err(|stop| scope.place(stop))?;
        }
        scope.vars.insert(bind.var.clone(), var);
        if let Some(filter) = &bind.filter {
            let t = self.expr(m, scope, filter, None, depth)?;
            if !is_bool(&t) {
                let message = format!("where of a binding is {}, not bool", t.show());
                return Err(scope.place(uerr(message)));
            }
        }
        Ok(ty)
    }

    /// A binding over an event kind reads every move of it or a subkind, and every observed
    /// input; one over a decision's movements or over `pads` is a running read (refcore's
    /// `m_bind_movements`). L §5.14.1.
    fn bind_movements(&mut self, m: &Reality, scope: &Scope, kind: &str) {
        let sub = m.subtree(kind);
        for (id, mv) in m.moves.entries() {
            if !sub.contains(&mv.kind) {
                continue;
            }
            self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Bind);
            if mv.decision {
                self.mark(
                    &scope.item,
                    crate::model::Mark::Legs,
                    crate::model::Target::Move(id.clone()),
                );
            }
        }
        if sub.contains("pads") {
            self.mark(
                &scope.item,
                crate::model::Mark::Legs,
                crate::model::Target::Kind("pads".to_owned()),
            );
        }
        for (id, input) in m.inputs.entries() {
            if input.observed {
                self.edge(&scope.item, Node::new(NodeKind::Input, id), Rel::Bind);
            }
        }
    }

    /// A binding over a place kind reads every place section of it or a subkind (refcore's
    /// `bind`, its kind arm). E §3.1 step 5.
    fn bind_places(&mut self, m: &Reality, scope: &Scope, kind: &str) {
        if m.kinds.get(kind).is_none_or(|k| k.spec != KindSpec::Place) {
            return;
        }
        let sub = m.subtree(kind);
        for (id, place) in m.places.entries() {
            if sub.contains(&place.kind) {
                self.edge(&scope.item, Node::new(NodeKind::Place, id), Rel::Bind);
            }
        }
    }

    /// A stepped range's type: its ends of one type, ordered; a range of quantities states a
    /// step of their unit, a range of dates a duration (refcore's `range_type`). L §5.6.
    pub(super) fn range_type(
        &mut self,
        m: &Reality,
        scope: &Scope,
        range: &(Expr, Expr, Option<Expr>),
        depth: u32,
    ) -> Typed {
        let (a, b, step) = range;
        let mut ends = scope.clone();
        if scope.stands == Stands::Ask {
            ends.allow_lit = true;
        }
        let open = |t: &Ty| matches!(t, Ty::Flex(_) | Ty::Year4);
        let mut at = self.expr(m, &ends, a, None, depth)?;
        let b_expect = (!open(&at)).then(|| at.clone());
        let bt = self.expr(m, &ends, b, b_expect.as_ref(), depth)?;
        if open(&at) {
            let a_expect = if matches!(bt, Ty::Flex(_)) {
                Ty::Int
            } else {
                bt.clone()
            };
            at = self.expr(m, &ends, a, Some(&a_expect), depth)?;
        }
        let st = match step {
            Some(step) => Some(self.expr(m, &ends, step, Some(&at), depth)?),
            None => None,
        };
        no_order(m, &at, "a range")?;
        no_order(m, &bt, "a range")?;
        if [&at, &bt].iter().any(|t| matches!(t, Ty::Unseen)) {
            return Ok(Ty::Unseen);
        }
        let int =
            |t: &Ty| matches!(t, Ty::Int | Ty::Flex(_)) || key_value_kind(m, t) == Some("int");
        if int(&at) && int(&bt) {
            return Ok(Ty::Int);
        }
        let Some(st) = st else {
            return Err(refused(
                Load::Domain,
                format!("range of quantities {} .. {} has no step", text(a), text(b)),
            ));
        };
        if is_date(m, &at) {
            if !is_date(m, &bt) || !is_duration(&st) {
                return Err(uerr("a range of dates steps by a duration".to_owned()));
            }
            return Ok(Ty::Date);
        }
        let u = unify(m, &at, &bt)?;
        let as_num = |t: &Ty| -> Result<Ty, Stop> {
            if matches!(t, Ty::Money(_)) {
                return Ok(Ty::Num(unit_of(t)?, Shown::Plain));
            }
            Ok(t.clone())
        };
        unify(m, &as_num(&u)?, &as_num(&st)?)?;
        Ok(u)
    }

    /// An expression of type `target`: typed where `target` is expected, then assigned to it.
    pub(super) fn typed_as(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        target: &Ty,
        what: &str,
        depth: u32,
    ) -> Typed {
        let t = self.expr(m, scope, e, Some(target), depth)?;
        self.assign(m, scope, target, e, &t, what)
            .map_err(|stop| scope.place(stop))?;
        Ok(t)
    }

    /// A value of type `t`, written as `node`, where a `target` is declared: a key literal
    /// resolved there, a bare figure of a currency's precision, a number of the unit, a key
    /// of the index (refcore's `assign`). L §3.1, §3.2, §2.5, §4.2, §7.1.
    pub(super) fn assign(
        &mut self,
        m: &Reality,
        scope: &Scope,
        target: &Ty,
        node: &Expr,
        t: &Ty,
        what: &str,
    ) -> Result<(), Stop> {
        if matches!(t, Ty::Unseen | Ty::None | Ty::Hole | Ty::Year4) || matches!(target, Ty::Unseen)
        {
            return Ok(());
        }
        if matches!(t, Ty::Dyn) && matches!(target, Ty::Money(_) | Ty::Num(..) | Ty::Int | Ty::Dyn)
        {
            return Ok(());
        }
        if let Ty::KeyLit(written) = t {
            if matches!(target, Ty::Key(_) | Ty::Code(_)) {
                let mut retyped = scope.clone();
                retyped.stands = Stands::Ask;
                self.expr(m, &retyped, node, Some(target), 0)?;
                return Ok(());
            }
            return Err(
                refused(Load::Key, format!("ambiguous key {written} for {what}")).naming(written),
            );
        }
        assign_type(m, target, t, what)
    }
}

/// `LOAD-COVERED`: a def's binding over observations names its span in a `covered()` guard
/// of its rule's `when` or an `if` (refcore's `m_bind_movements`, its end). L §5.19.
fn covered(scope: &Scope, bind: &Bind) -> Result<(), Stop> {
    if scope.item.kind != NodeKind::Def {
        return Ok(());
    }
    let span = match (&bind.over, &bind.on) {
        (Some(span), _) => span_text(span),
        (None, Some(on)) => text(on),
        (None, None) => String::new(),
    };
    if scope.covered.contains(&span) {
        return Ok(());
    }
    Err(refused(
        Load::Covered,
        format!(
            "{} aggregates observations over a span with no covered() guard ({} in {span})",
            scope.item.id, bind.var
        ),
    ))
}

/// A type assigned to a declared one, the cases that need no retyping (refcore's `assign`,
/// from its money arm on, by the kinds' words as refcore's `k` and `tk`). L §3.1, §3.2,
/// §4.2.
fn assign_type(m: &Reality, target: &Ty, t: &Ty, what: &str) -> Result<(), Stop> {
    let mismatch = || uerr(format!("{what} is {}, not {}", t.show(), target.show()));
    let fits = match (target, target.word()) {
        (Ty::Money(cur), _) => to_money(m, cur, t, what)?,
        (Ty::Num(unit, _), _) => to_num(m, unit, t)?,
        (Ty::Key(ix), _) => return to_key(m, ix, t, what),
        (Ty::Period(_), _) => {
            if period_kind(m, t).is_some_and(|pk| Some(pk) == period_kind(m, target)) {
                return Ok(());
            }
            return Err(uerr(format!(
                "{what} is {}, not {}",
                t.show(),
                target.show()
            )));
        }
        (Ty::Record(fields), _) => {
            let Ty::Record(have) = t else {
                return Err(mismatch());
            };
            return to_record(fields, have, what);
        }
        (_, "int") => return to_int(m, t, what),
        (_, "rate") => match t.word() {
            "rate" => true,
            "flex" => {
                return Err(uerr(format!(
                    "{what}: a bare figure as a rate states no period (L §3.3)"
                )));
            }
            _ => return Err(uerr(format!("{what} is {}, not rate", t.show()))),
        },
        (_, "date") => {
            if is_date(m, t) {
                return Ok(());
            }
            return Err(uerr(format!("{what} is {}, not date", t.show())));
        }
        (_, "bool" | "text" | "calendar" | "mmdd" | "code") => t == target,
        (_, "dates") => matches!(t.word(), "dates" | "list"),
        (Ty::Dist(want), _) => {
            if let Ty::Dist(unit) = t {
                converts(unit, want)?
            } else {
                // A distribution over keys has no unit: refcore's own message for it needs
                // a unit that converts to none, which a declared unit never is here.
                matches!(t, Ty::DistKeys(_)) && want.is_one()
            }
        }
        (_, "dist") => true,
        (_, _) => false,
    };
    if fits {
        return Ok(());
    }
    Err(mismatch())
}

/// A value assigned to `int` (refcore's `assign`, its int arm). L §3.1, §2.1.
fn to_int(m: &Reality, t: &Ty, what: &str) -> Result<(), Stop> {
    let fits = match (t, t.word()) {
        (Ty::Num(unit, _), _) => unit.is_one(),
        (_, "int") => true,
        (_, "flex") => int_flex(t),
        (_, "key") => key_value_kind(m, t) == Some("int"),
        (_, _) => false,
    };
    if fits {
        return Ok(());
    }
    Err(uerr(format!("{what} is {}, not int", t.show())))
}

/// A value assigned to `num unit`: a bare figure, a number or money of its unit, an int or an
/// integer key where it has none (refcore's `assign`, its num arm). L §3.2.
fn to_num(m: &Reality, unit: &crate::model::Unit, t: &Ty) -> Result<bool, Stop> {
    match t.word() {
        "flex" => Ok(true),
        "num" | "money" => {
            let from = unit_of(t)?;
            Ok(keyed_unit(unit) || keyed_unit(&from) || converts(&from, unit)?)
        }
        "int" => Ok(unit.is_one()),
        "key" => Ok(key_value_kind(m, t) == Some("int") && unit.is_one()),
        _ => Ok(false),
    }
}

/// Whether a unit takes a key's unit (`$k`), which only the read's keys give. L §3.2.
fn keyed_unit(unit: &crate::model::Unit) -> bool {
    unit.terms().iter().any(|(base, _)| base.starts_with('$'))
}

/// A record assigned to a record type: every field the type names (refcore's `assign`, its
/// record arm). L §3.1.
fn to_record(fields: &[(String, Ty)], have: &[(String, Ty)], what: &str) -> Result<(), Stop> {
    for (field, _) in fields {
        if !have.iter().any(|(name, _)| name == field) {
            return Err(refused(Load::Field, format!("{what} lacks field {field}")));
        }
    }
    Ok(())
}

/// A value assigned to `money cur`: money in it, or a bare figure of its precision; a num
/// of its unit needs `round` (refcore's `assign`, its money arm). L §3.2, §7.1.
fn to_money(m: &Reality, cur: &str, t: &Ty, what: &str) -> Result<bool, Stop> {
    if let Ty::Money(c) = t {
        return Ok(c == cur);
    }
    if let Ty::Flex(n) = t {
        let Some(minor) = m.currencies.get(cur) else {
            return Ok(false);
        };
        let whole = n.checked_div(minor).map_err(num_stop)?.is_integer();
        if !whole {
            let figure = n.to_text().map_err(|_| {
                Stop::Refused(Refusal::new(
                    Code::EngineInvariant,
                    "a figure with no decimal text".to_owned(),
                ))
            })?;
            return Err(refused(
                Load::Precision,
                format!("{figure} is finer than the minor unit"),
            ));
        }
        return Ok(true);
    }
    if let Ty::Num(unit, _) = t {
        let money = unit_of(&Ty::Money(cur.to_owned()))?;
        if converts(unit, &money)? {
            return Err(uerr(format!(
                "{what} gives a num {}; a money value needs round or prorata (a num CUR stored as money)",
                unit.text()
            )));
        }
    }
    Ok(false)
}

/// A value assigned to `key ix`: a key of it, of a kind under it, of another integer index,
/// an integer for an integer index, a period or a date for a period index (refcore's
/// `assign`, its key arm). L §2.5, §4.2 Keys as values.
fn to_key(m: &Reality, ix: &str, t: &Ty, what: &str) -> Result<(), Stop> {
    let index = m.indexes.get(ix);
    if let Ty::Key(other) = t {
        if other == ix {
            return Ok(());
        }
        let kinds = m.kinds.get(other).is_some() && m.kinds.get(ix).is_some();
        if kinds && m.subtree(ix).contains(other) {
            return Ok(());
        }
        let ints =
            index.is_some_and(|i| i.is_int) && m.indexes.get(other).is_some_and(|i| i.is_int);
        if ints {
            return Ok(());
        }
    }
    if matches!(t, Ty::Int | Ty::Flex(_)) && index.is_some_and(|i| i.is_int) {
        return Ok(());
    }
    if let Some(pk) = index.and_then(|i| i.period)
        && matches!(t, Ty::Period(_) | Ty::Date | Ty::Key(_))
        && (period_kind(m, t) == Some(pk) || (matches!(t, Ty::Date) && pk == PeriodKind::Day))
    {
        return Ok(());
    }
    if matches!(t, Ty::Tuple(_)) {
        return Err(refused(
            Load::Key,
            format!("{what} gives a tuple of keys, not one key of {ix}"),
        ));
    }
    Err(refused(
        Load::Key,
        format!("{what} is {}, not a key of {ix}", t.show()),
    ))
}

/// `LOAD-SHADOW`: a bound variable spelled as a key of its domain.
fn key_spelling(var: &str, id: &str) -> Stop {
    refused(
        Load::Shadow,
        format!("bound variable {var} has the spelling of a key of {id}"),
    )
}

/// A key list's key as its text compares (refcore's `resolve_text`). L §2.5.
fn key_text(key: &Lit) -> String {
    key.text()
}

/// A load refusal with no place yet.
fn refused(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

#[cfg(test)]
mod tests {
    use super::{BindAs, Over, alive_clause, bound_twice, over};
    use crate::api::Source;
    use crate::load::load;
    use crate::model::Reality;
    use crate::syntax::{Bind, Domain, Expr, Lit, Stop, parse_reality};

    /// A reality with an index, an input, a place kind and an observed input.
    fn reality() -> Reality {
        let text = "model m\n  tally 2\n  on 2026-03-01\n  start 2026-01-01\n  end 2026-12-31\n\n\
                    unit PLN: currency minor 0.01\n\nindex qs = [q1, q2]\n\ninput amt: money PLN\n\n\
                    input bank_rows: observed\n\nplace konto: accounts PLN lives 2026-01-01..\n";
        let files = parse_reality(&[Source {
            path: "a.tally".to_owned(),
            text: text.to_owned(),
        }])
        .expect("it parses");
        load(files).expect("it loads")
    }

    /// `var in id`.
    fn bind(var: &str, id: &str) -> Bind {
        Bind {
            var: var.to_owned(),
            domain: Domain::Name {
                id: id.to_owned(),
                qual: None,
            },
            on: None,
            over: None,
            filter: None,
            text: String::new(),
        }
    }

    /// `var in id on cut`.
    fn alive(var: &str, id: &str) -> Bind {
        Bind {
            on: Some(Expr::Cut),
            ..bind(var, id)
        }
    }

    /// The text of a refusal, or what else came back.
    fn message(result: Result<Over, Stop>) -> String {
        match result {
            Err(Stop::Refused(refusal)) => refusal.text(),
            other => format!("{other:?}"),
        }
    }

    /// Each binding with the refusal it gets, or what it ranges over.
    fn check_all(m: &Reality, cases: &[(Bind, &str)]) {
        for (bind, want) in cases {
            assert_eq!(
                message(over(m, bind, BindAs::Plain)),
                *want,
                "{} in {:?}",
                bind.var,
                bind.domain
            );
        }
    }

    #[test]
    fn a_bound_variable_takes_no_name_already_in_use() {
        let listed = Bind {
            domain: Domain::List(vec![Lit::Name("a".to_owned()), Lit::Name("b".to_owned())]),
            ..bind("b", "qs")
        };
        check_all(
            &reality(),
            &[
                (
                    bind("amt", "qs"),
                    "LOAD-SHADOW bound variable amt has the name of a top-level id",
                ),
                (
                    bind("q1", "qs"),
                    "LOAD-SHADOW bound variable q1 has the spelling of a key of qs",
                ),
                (
                    listed,
                    "LOAD-SHADOW bound variable b has the spelling of a key of its list",
                ),
            ],
        );
        assert_eq!(
            message(Err(bound_twice("k"))),
            "LOAD-SHADOW bound variable k has the name of a bound variable in scope",
            "a variable bound around it"
        );
    }

    #[test]
    fn a_binding_names_what_it_ranges_over() {
        check_all(
            &reality(),
            &[
                (bind("k", "qs"), "Ok(Index(\"qs\"))"),
                (bind("e", "event"), "LOAD-NAME unknown id event"),
                (
                    bind("x", "amt"),
                    "LOAD-KEY amt is not an index, a kind, a key list or a range",
                ),
            ],
        );
    }

    #[test]
    fn a_binding_over_things_or_movements_says_when() {
        check_all(
            &reality(),
            &[
                (
                    bind("p", "accounts"),
                    "LOAD-ALIVE binding over accounts needs on or over",
                ),
                (
                    bind("t", "transfers"),
                    "LOAD-ALIVE binding over transfers needs on or over",
                ),
                (
                    bind("o", "bank_rows"),
                    "LOAD-ALIVE binding over bank_rows needs on or over",
                ),
                (alive("p", "accounts"), "Ok(Things(\"accounts\"))"),
                (alive("o", "bank_rows"), "Ok(Observed(\"bank_rows\"))"),
            ],
        );
        let observed = Over::Observed("bank_rows".to_owned());
        assert!(alive_clause("on", &observed).is_ok(), "on on observed rows");
        let refused = match alive_clause("over", &Over::Index("qs".to_owned())) {
            Err(Stop::Refused(refusal)) => refusal.text(),
            other => format!("{other:?}"),
        };
        assert_eq!(
            refused, "LOAD-DOMAIN over on a binding that is not over a kind",
            "over on an index's keys"
        );
    }
}
