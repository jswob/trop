//! Load's checker, its keyed reads: an input, a def, a place or a move read at keys, each key at its position, and a postfix's index or field.
//! Spec: L §2.5 (names, keys by position: a key literal resolved against the index, kind or
//! axis at its position), §4.6 (a period at a key of another granularity), §5.10 (a place by
//! its id or at keys), §5.7 and §5.14.3 (a movement's fields, `a.adjusts`), §6.4 (a scenario's
//! rows), §8.3 (`LOAD-KEY`, `-FIELD`, `-PARSE`, `-UNIT`); E §3.1 step 5 (each read an edge
//! with its keys' relation).
//! Twin: engine/tools/refcore/check.py (`read`, `key_pos`, `key_conv`, `period_conv`,
//! `t_index`, `t_field`, `member_read`, `dom_of`); check3.py (`m_place_value`,
//! `m_move_value`, `m_movement_field`); model.py (`kind_attrs`).
//! Invariants: each read is typed in refcore's order and recorded as an edge as it is typed; a
//! name the namespace proves is found by its typed id, a miss the engine's invariant; a field
//! load can't narrow is `Unseen`, refused by no rule; every recursion is counted (R-16).

use super::check::{AxisRead, Checker, Figure, Scope, Var, VarDom, invariant};
use super::keys::resolve_key;
use super::random::DrawKey;
use super::reads::leg_type;
use super::srows::base_name;
use super::texts::{fill_ref, post_text, text};
use super::types::{
    Typed, granularity, is_keylit_node, key_lit_of, lit_rule, name_word, num, pk_name, refused,
    unit_error, unit_of, word_pk,
};
use crate::base::Load;
use crate::model::{
    Domain, KeyVal, KindSpec, NameKind, Node, NodeKind, PeriodWord, Reality, Rel, Ty,
};
use crate::syntax::{Expr, Key, Lit, Postfix, Stop};
use crate::time::PeriodKind;

impl Checker {
    /// A read of an input or a def at keys (refcore's `read`). L §2.5, §4.6; E §3.1 step 5.
    pub(super) fn read(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        kind: NameKind,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        let affx = if core::mem::take(&mut self.affx_hit) {
            super::random::SampleOf::AffineX
        } else {
            super::random::SampleOf::Scope
        };
        let (binds, declared, observed, priced) = if kind == NameKind::Input {
            let input = m.inputs.at(m.names.input_id(id)?)?;
            let priced = input.prices.is_some() && input.prices_by.is_some();
            (&input.binds, input.ty.clone(), input.observed, priced)
        } else {
            let def = m.defs.at(m.names.def_id(id)?)?;
            (&def.binds, Some(def.ty.clone()), false, false)
        };
        let node_kind = if kind == NameKind::Input {
            NodeKind::Input
        } else {
            NodeKind::Def
        };
        if keys.len() != binds.len() && !observed {
            return Err(refused(
                Load::Key,
                format!(
                    "wrong number of keys for {id}: {}, not {}",
                    keys.len(),
                    binds.len()
                ),
            ));
        }
        if observed {
            let by_id = match keys {
                [Key::Expr(key)] => is_keylit_node(m, scope, key),
                _ => false,
            };
            if !by_id {
                return Err(refused(
                    Load::Key,
                    format!("an observation is read by its id: {id}[<id>]"),
                ));
            }
            self.edge(&scope.item, Node::new(NodeKind::Input, id), Rel::Any);
            return Ok(Ty::Key("events".to_owned()));
        }
        let mut env = Vec::new();
        for (key, bind) in keys.iter().zip(binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            let lit = self.key_pos(m, scope, kn, &bind.domain, depth)?;
            let mut value = "$?".to_owned();
            if let Expr::Name(name) = kn
                && scope.is_var(name)
            {
                value = format!("${name}");
            }
            if let Some(named) = lit {
                value = named.text();
            }
            env.push((bind.var.clone(), value));
        }
        let target = Node::new(node_kind, id);
        let rel = super::check::relation(scope, keys);
        self.direct(&target);
        self.shifted(scope, &target, &rel, keys);
        self.edge(&scope.item, target.clone(), rel);
        let mut ty = declared.unwrap_or(Ty::Unseen);
        if priced {
            ty = num(unit_of(&ty)?);
        }
        if let Ty::Num(unit, shown) = &ty
            && unit.terms().iter().any(|(base, _)| base.starts_with('$'))
        {
            ty = Ty::Num(unit.subst(&env).map_err(unit_error)?, *shown);
        }
        if let Ty::Dist(unit) = &ty
            && unit.terms().iter().any(|(base, _)| base.starts_with('$'))
        {
            ty = Ty::Dist(unit.subst(&env).map_err(unit_error)?);
        }
        if matches!(ty, Ty::Dist(_)) {
            return self.dist_read(m, scope, &target, &ty, affx);
        }
        self.note_read(scope, &target);
        Ok(ty)
    }

    /// A place read by its id, or a keyed place at keys (refcore's `m_place_value`).
    /// L §5.10.
    pub(super) fn place_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        let place = m.places.at(m.names.place_id(id)?)?;
        if keys.len() != place.binds.len() {
            return Err(refused(
                Load::Key,
                format!(
                    "wrong number of keys for place {id}: {}, not {}",
                    keys.len(),
                    place.binds.len()
                ),
            ));
        }
        for (key, bind) in keys.iter().zip(&place.binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        self.edge(&scope.item, Node::new(NodeKind::Place, id), Rel::Any);
        Ok(Ty::Key(place.kind.clone()))
    }

    /// A movement instance named by its move and keys; a decision's is a running read of its
    /// legs (refcore's `m_move_value`). L §2.5, §5.14.1.
    pub(super) fn move_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        self.move_instance(m, scope, id, keys, depth).map(Ty::Key)
    }

    /// The kind of a movement instance named by its move and keys, its keys checked and its
    /// reads noted (refcore's `m_move_value`, whose type is always that kind's key). L §2.5,
    /// §5.14.1.
    pub(super) fn move_instance(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Result<String, Stop> {
        let mv = m.moves.at(m.names.move_id(id)?)?;
        if keys.len() != mv.binds.len() {
            return Err(refused(
                Load::Key,
                format!(
                    "wrong number of keys for {id}: {}, not {}",
                    keys.len(),
                    mv.binds.len()
                ),
            ));
        }
        for (key, bind) in keys.iter().zip(&mv.binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Any);
        if mv.decision {
            self.mark(
                &scope.item,
                crate::model::Mark::Legs,
                crate::model::Target::Move(id.to_owned()),
            );
        }
        Ok(mv.kind.clone())
    }

    /// A key at a key position whose domain is `domain`: typed there, and checked against
    /// it; the key it names when written as a literal (refcore's `key_pos`). L §2.5, §4.2,
    /// §4.6.
    pub(super) fn key_pos(
        &mut self,
        m: &Reality,
        scope: &Scope,
        kn: &Expr,
        domain: &Domain,
        depth: u32,
    ) -> Result<Option<KeyVal>, Stop> {
        if let (Domain::Axis(axis), Expr::Name(name)) = (domain, kn)
            && !scope.is_var(name)
        {
            if name == axis {
                self.axis_read(&scope.item, axis, AxisRead::AtKey);
                return Ok(None);
            }
            if m.axes.contains(name) {
                return Err(refused(
                    Load::Key,
                    format!("{name} is used as a key of {axis}"),
                ));
            }
        }
        if let Domain::Index(ix) = domain
            && let Some(pk) = m.indexes.get(ix).and_then(|index| index.period)
        {
            let gran = granularity(kn);
            if gran.is_some_and(|g| g != pk) {
                let t = self.expr(m, scope, kn, None, depth)?;
                self.key_conv(m, scope, &t, domain, kn)?;
                return Ok(None);
            }
            let four = matches!(kn, Expr::Lit(Lit::Num(n)) if n.four);
            if four && pk != PeriodKind::Year {
                let t = self.expr(m, scope, kn, Some(&Ty::Period(PeriodWord::Year)), depth)?;
                self.key_conv(m, scope, &t, domain, kn)?;
                return Ok(None);
            }
        }
        let mut range_lit = false;
        if is_keylit_node(m, scope, kn) {
            match domain {
                Domain::Index(_) | Domain::Kind(_) | Domain::Axis(_) => {
                    let dated = matches!(
                        kn,
                        Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..))
                    );
                    let year = matches!(kn, Expr::Lit(Lit::Num(n)) if n.four)
                        && matches!(domain, Domain::Index(ix) if m.indexes.get(ix).is_some_and(|i| i.period.is_some()));
                    if dated || year {
                        lit_rule(scope, Figure::Date, "", None)?;
                    }
                    return resolve_lit(m, scope, kn, domain).map(Some);
                }
                Domain::List(_) => return resolve_lit(m, scope, kn, domain).map(Some),
                Domain::Range => range_lit = true,
            }
        }
        let mut inner = scope.clone();
        if matches!(kn, Expr::Sum(..)) || range_lit {
            inner.allow_lit = true;
        }
        let expect = match domain {
            Domain::Index(ix) | Domain::Kind(ix) | Domain::Axis(ix) => Some(Ty::Key(ix.clone())),
            Domain::List(_) | Domain::Range => None,
        };
        let t = self.expr(m, &inner, kn, expect.as_ref(), depth)?;
        if let (Domain::Index(ix), Expr::Name(name)) = (domain, kn)
            && let Some(var) = scope.vars.get(name)
            && var.dom == VarDom::Range
            && let Some(end) = &var.end
        {
            self.draws.push(DrawKey {
                end: end.clone(),
                index: ix.clone(),
                at: scope.file_line(),
            });
        }
        self.key_conv(m, scope, &t, domain, kn)?;
        Ok(None)
    }

    /// A key's type checked against its position's domain (refcore's `key_conv`). L §2.5,
    /// §4.2 Keys as values, §4.6.
    fn key_conv(
        &mut self,
        m: &Reality,
        scope: &Scope,
        t: &Ty,
        domain: &Domain,
        kn: &Expr,
    ) -> Result<(), Stop> {
        let var = if let Expr::Name(name) = kn {
            scope.vars.get(name)
        } else {
            None
        };
        let listed = var.is_some_and(|v| matches!(v.ty, Ty::KeyList));
        if matches!(t, Ty::Unseen) {
            return Ok(());
        }
        if matches!(t, Ty::KeyList) || (listed && matches!(domain, Domain::Index(_))) {
            if let Some(var) = var {
                self.fix_list(m, scope, var, domain)?;
            }
            return Ok(());
        }
        let not_a_key = |dom: &str| {
            refused(
                Load::Key,
                format!("{} ({}) is not a key of {dom}", text(kn), t.show()),
            )
        };
        match domain {
            Domain::Index(ix) => {
                let index = m.indexes.at(m.names.index_id(ix)?)?;
                if let Ty::Key(other) = t {
                    if key_of_index(m, (ix, index), other, kn)? == Conv::Period {
                        self.period_read(scope, ix, kn);
                    }
                    return Ok(());
                }
                if index.is_int && matches!(t, Ty::Int | Ty::Flex(_) | Ty::Year4) {
                    return Ok(());
                }
                if index.is_int && matches!(t, Ty::Num(unit, _) if unit.is_one()) {
                    return Ok(());
                }
                if index.period.is_some() {
                    if matches!(t, Ty::Date | Ty::Never) {
                        return Ok(());
                    }
                    if let Ty::Period(word) = t {
                        period_conv(index.period, ix, word_pk(*word))?;
                        self.period_read(scope, ix, kn);
                        return Ok(());
                    }
                }
                Err(not_a_key(ix))
            }
            Domain::Kind(kind) => {
                if let Ty::Key(other) = t
                    && m.kinds.get(other).is_some()
                    && (m.subtree(kind).contains(other) || m.subtree(other).contains(kind))
                {
                    return Ok(());
                }
                let unit_kind = m.kinds.get(kind).is_some_and(|k| k.spec == KindSpec::Unit);
                if matches!(t, Ty::UnitKey) && (unit_kind || kind == "currencies") {
                    return Ok(());
                }
                Err(not_a_key(kind))
            }
            Domain::Axis(axis) => {
                if matches!(t, Ty::Key(other) if other == axis) {
                    return Ok(());
                }
                Err(not_a_key(axis))
            }
            Domain::List(_) | Domain::Range => Ok(()),
        }
    }

    /// A key-list variable read at a key position: the first position fixes its index, a
    /// second index is `LOAD-KEY`, and each of its keys must be a key there (refcore's
    /// `key_conv`, its key-list arm). L §2.5.
    pub(super) fn fix_list(
        &mut self,
        m: &Reality,
        scope: &Scope,
        var: &Var,
        domain: &Domain,
    ) -> Result<(), Stop> {
        let name = domain_name(domain);
        match self.kl_fixed.get(&var.id) {
            Some(fixed) if fixed != domain => {
                return Err(refused(
                    Load::Key,
                    format!(
                        "{} is used as a key of {} and {}",
                        var.name,
                        domain_name(fixed),
                        name
                    ),
                ));
            }
            Some(_) => {}
            None => {
                self.kl_fixed.insert(var.id, domain.clone());
            }
        }
        for key in &var.keylist {
            resolve_lit(m, scope, &Expr::Lit(key.clone()), domain)?;
        }
        Ok(())
    }

    /// A base with its postfixes: a name read at keys, then fields (refcore's `t_index`,
    /// `t_field`). L §2.5, §3.1, §5.14.
    pub(super) fn post(
        &mut self,
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        posts: &[Postfix],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let Some(first) = posts.first() else {
            return self.expr(m, scope, base, expect, depth);
        };
        if let (Expr::Name(prefix), Postfix::Field(name)) = (base, first)
            && self.base_world.as_ref().is_some_and(|w| w.id == *prefix)
        {
            let rest = posts.get(1..).unwrap_or(&[]);
            return self.base_read(m, scope, name, rest, expect, depth);
        }
        if let Some((fill, fkeys, keys, used)) = fill_ref(m, scope, base, posts) {
            let ty = self.fill_read(m, scope, &fill, (&fkeys, &keys), depth)?;
            return Self::post_rest(m, scope, base, posts, (ty, used));
        }
        let ty = match first {
            Postfix::Keys(keys) => self.index(m, scope, base, keys, depth)?,
            Postfix::Field(field) => {
                let t = self.expr(m, scope, base, None, depth)?;
                Self::field(m, scope, Some(base), &t, field)?
            }
        };
        Self::post_rest(m, scope, base, posts, (ty, 1))
    }

    /// `<base>.x` and its postfixes `rest` in a scenario's expression row: x read in the base,
    /// typed as refcore types the name its rewrite leaves (refcore's `m_check_scen_rows`):
    /// alone or under a field, as `t_name`'s base-world arm, after a fill's `taken`; at keys,
    /// as `t_index`, which names the base no more. L §6.4.
    fn base_read(
        &mut self,
        m: &Reality,
        scope: &Scope,
        name: &str,
        rest: &[Postfix],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let bare = Expr::Name(name.to_owned());
        let keyed = matches!(rest.first(), Some(Postfix::Keys(_)));
        if !keyed && fill_ref(m, scope, &bare, rest).is_none() {
            let Some(world) = &self.base_world else {
                return Err(invariant("a base read outside a scenario row".to_owned()));
            };
            base_name(m, world, name)?;
        }
        if rest.is_empty() {
            return self.name(m, scope, name, expect, depth);
        }
        self.post(m, scope, &bare, rest, expect, depth)
    }

    /// The postfixes after the first `done` of `posts`, the value so far of type `ty`: fields
    /// of it; a key there is no read (refcore's `t_index`, `t_field`). L §2.5, §3.1.
    fn post_rest(
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        posts: &[Postfix],
        (mut ty, mut done): (Ty, usize),
    ) -> Typed {
        for post in posts.get(done..).unwrap_or(&[]) {
            let prefix = post_text(base, posts.get(..done).unwrap_or(&[]));
            match post {
                Postfix::Keys(_) => {
                    return Err(refused(
                        Load::Parse,
                        format!("cannot parse: {prefix} is not read with keys"),
                    ));
                }
                Postfix::Field(field) => ty = Self::field(m, scope, None, &ty, field)?,
            }
            done = done
                .checked_add(1)
                .ok_or(Stop::NotBuilt("a long postfix chain"))?;
        }
        Ok(ty)
    }

    /// `x[k]` with `x` a name (refcore's `t_index`). L §2.5.
    fn index(
        &mut self,
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        if let Expr::Name(name) = base
            && !scope.is_var(name)
        {
            let Some(kind) = m.names.kind(name) else {
                return Err(refused(Load::Name, format!("unknown id {name}")).naming(name));
            };
            return match kind {
                NameKind::Input | NameKind::Def => self.read(m, scope, name, kind, keys, depth),
                NameKind::Place => self.place_value(m, scope, name, keys, depth),
                NameKind::Move => self.move_value(m, scope, name, keys, depth),
                NameKind::Unit
                | NameKind::Kind
                | NameKind::Index
                | NameKind::Axis
                | NameKind::Band
                | NameKind::Fill
                | NameKind::Version
                | NameKind::Ask
                | NameKind::Source
                | NameKind::Cut => Err(refused(
                    Load::Parse,
                    format!(
                        "cannot parse: {} {name} is not read with keys",
                        name_word(kind)
                    ),
                )),
            };
        }
        Err(refused(
            Load::Parse,
            format!("cannot parse: {} is not read with keys", text(base)),
        ))
    }

    /// `x.f` of a value of type `t` (refcore's `t_field`). L §3.1, §5.7, §5.14. A prefix that
    /// names no item: a joint model's member whose namespace holds the ask `f` (no value,
    /// refcore's `member_read`), else an unknown id.
    pub(super) fn field(
        m: &Reality,
        scope: &Scope,
        base: Option<&Expr>,
        t: &Ty,
        field: &str,
    ) -> Typed {
        if let (Some(Expr::Name(name)), Ty::KeyLit(_)) = (base, t)
            && !scope.is_var(name)
            && !m.names.contains_key(name)
        {
            if m.joint.as_ref().is_some_and(|j| j.names_ask(name, field)) {
                let text = format!("cannot parse: ask {name}.{field} is not a value");
                return Err(refused(Load::Parse, text));
            }
            let id = format!("{name}.{field}");
            return Err(refused(Load::Name, format!("unknown id {id}")).naming(&id));
        }
        if matches!(t, Ty::Unseen) {
            return Ok(Ty::Unseen);
        }
        if let Ty::Record(_) = t {
            return t.field(field).cloned().ok_or_else(|| {
                refused(
                    Load::Field,
                    format!("{field} is not a field of {}", t.show()),
                )
            });
        }
        if let Ty::Key(kind) = t
            && let Some(info) = m.kinds.get(kind)
        {
            if info.spec == KindSpec::Event {
                return movement_field(m, kind, field, base);
            }
            return Err(refused(
                Load::Field,
                format!("{field} is not a field of a thing of {kind}"),
            ));
        }
        Err(refused(
            Load::Field,
            format!("{field} is not a field of {}", t.show()),
        ))
    }
}

/// The domain a key type's index, kind or axis is (refcore's `dom_of`).
pub(super) fn domain_of(m: &Reality, ix: &str) -> Option<Domain> {
    if m.indexes.get(ix).is_some() {
        return Some(Domain::Index(ix.to_owned()));
    }
    if m.kinds.get(ix).is_some() {
        return Some(Domain::Kind(ix.to_owned()));
    }
    (m.names.kind(ix) == Some(NameKind::Axis)).then(|| Domain::Axis(ix.to_owned()))
}

/// A domain's name, as a message names it.
fn domain_name(domain: &Domain) -> String {
    match domain {
        Domain::Index(id) | Domain::Kind(id) | Domain::Axis(id) => id.clone(),
        Domain::List(_) => "list".to_owned(),
        Domain::Range => "range".to_owned(),
    }
}

/// A key literal's key in `domain`, refused at the scope's line (refcore's
/// `resolve_key_literal`). L §2.5.
pub(super) fn resolve_lit(
    m: &Reality,
    scope: &Scope,
    node: &Expr,
    domain: &Domain,
) -> Result<KeyVal, Stop> {
    let Some(key) = key_lit_of(node) else {
        return Err(refused(
            Load::Key,
            format!("{} is not a key of {}", text(node), domain_name(domain)),
        ));
    };
    let (file, line) = scope.file_line();
    resolve_key(m, &key, domain, &file, line)
}

/// How a key stands at its key position: as a key there, or as a period read at a key of a
/// time index, which load folds at the end (refcore's `key_conv`, its `period` result).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Conv {
    /// A key of the position's index, an integer, or a day.
    Plain,
    /// A period no wider than the index's keys.
    Period,
}

/// A key of index `other` at a key of `ix`: the same index, two integer indexes, or two
/// period indexes where the period isn't wider (refcore's `key_conv`, its key arm). L §2.5,
/// §4.6.
fn key_of_index(
    m: &Reality,
    (ix, index): (&str, &crate::model::IndexInfo),
    other: &str,
    kn: &Expr,
) -> Result<Conv, Stop> {
    if other == ix {
        return Ok(Conv::Plain);
    }
    if let Some(theirs) = m.indexes.get(other) {
        if index.is_int && theirs.is_int {
            return Ok(Conv::Plain);
        }
        if let (Some(_), Some(pk)) = (index.period, theirs.period) {
            period_conv(index.period, ix, pk)?;
            return Ok(Conv::Period);
        }
    }
    Err(refused(
        Load::Key,
        format!("{} is used as a key of {other} and {ix}", text(kn)),
    ))
}

/// `LOAD-KEY` when a period is wider than the index's keys (refcore's `period_conv`).
/// L §4.6.
fn period_conv(index_pk: Option<PeriodKind>, ix: &str, pk: PeriodKind) -> Result<(), Stop> {
    let order = |p: Option<PeriodKind>| match p {
        Some(PeriodKind::Day) => 0,
        Some(PeriodKind::Month) => 1,
        Some(PeriodKind::Quarter) => 2,
        Some(PeriodKind::Year) => 3,
        None => 9,
    };
    if order(Some(pk)) > order(index_pk) {
        let keys = index_pk.map_or("?", pk_name);
        return Err(refused(
            Load::Key,
            format!(
                "a {} holds no key of {ix}, whose keys are {keys}s",
                pk_name(pk)
            ),
        ));
    }
    Ok(())
}

/// Whether a field's object is a name, parentheses aside: the one object a kind test narrows
/// (refcore's `sc.narrow`, read for a name only). `None` is a field read after another
/// postfix, never a name.
fn may_narrow(base: Option<&Expr>) -> bool {
    let mut object = base;
    while let Some(Expr::Paren(inner)) = object {
        object = Some(inner.as_ref());
    }
    matches!(object, Some(Expr::Name(_)))
}

/// A field of a movement of event kind `kind`: a leg, an attribute, `day`, `fulfils`; any
/// other, of a movement of `events` (`a.adjusts.f`), is read by name over every event kind;
/// else `LOAD-FIELD` (refcore's `m_movement_field`). Of a name a kind test may narrow it is
/// `Unseen`: load keeps no kind test's narrowing. L §5.7, §5.14, §5.14.3, §8.3.
fn movement_field(m: &Reality, kind: &str, field: &str, object: Option<&Expr>) -> Typed {
    for (leg, unit, _) in m.kind_legs(kind) {
        if leg == field {
            return leg_type(m, &unit);
        }
    }
    let chain = m.chain(kind);
    // Root first, as refcore's `kind_attrs`: a parent's attribute before a subkind's.
    for info in chain.iter().rev() {
        if !info.attrs.iter().any(|(attr, _)| attr == field) {
            continue;
        }
        let declared = info.attr_types.iter().find(|(attr, _)| attr == field);
        if let Some((_, Some(ty))) = declared {
            return Ok(ty.clone());
        }
        // An attribute the prelude declares has no written type: refcore's `DYN`, but an
        // adjustment's `adjusts`, the movement it adjusts (L §5.14.3).
        let adjustment = chain.iter().any(|k| k.id == "adjustments");
        return Ok(if adjustment && field == "adjusts" {
            Ty::Key("events".to_owned())
        } else {
            Ty::Dyn
        });
    }
    match field {
        "day" => return Ok(Ty::Date),
        "fulfils" => return Ok(Ty::Key("events".to_owned())),
        _ => {}
    }
    if may_narrow(object) {
        return Ok(Ty::Unseen);
    }
    if kind == "events"
        && let Some(ty) = event_field(m, field)?
    {
        return Ok(ty);
    }
    Err(refused(
        Load::Field,
        format!("{field} is not a field of {kind}"),
    ))
}

/// A field of a movement whose kind load doesn't know (`a.adjusts`, a key of `events`): the
/// first event kind, in the order declared, with a leg of that name gives a leg's record of
/// unit `any`, or with an attribute of that name its declared type, else `dyn` (refcore's
/// `m_movement_field`, its `events` arm, read by name at run). L §5.14.3.
fn event_field(m: &Reality, field: &str) -> Result<Option<Ty>, Stop> {
    for (id, kind) in m.kinds.entries() {
        if kind.spec != KindSpec::Event {
            continue;
        }
        if m.kind_legs(id).iter().any(|(leg, _, _)| leg == field) {
            return leg_type(m, "any").map(Some);
        }
        for info in m.chain(id).into_iter().rev() {
            if info.attrs.iter().any(|(attr, _)| attr == field) {
                let declared = info.attr_types.iter().find(|(attr, _)| attr == field);
                let ty = declared.and_then(|(_, ty)| ty.clone());
                return Ok(Some(ty.unwrap_or(Ty::Dyn)));
            }
        }
    }
    Ok(None)
}
