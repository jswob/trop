//! Load's checker, its moves and their targets: a move's day, legs and attributes against its kind's, and the place or read each target names.
//! Spec: L §5.14 (a move's binds, day, `at`, `for`, `when`, legs and attributes against its
//! kind's), §5.14.1 (`reads`), §5.14.2 (what a leg's place changes), §5.10 (a place written
//! literally), §5.7 (an event kind's attributes), §2.5 (a key literal at a key position),
//! §8.3 (`LOAD-LEG`, `-FIELD`, `-READS`); E §3.1 step 7 (a running or declared read's target).
//! Twin: engine/tools/refcore/check3.py (`m_check_move`, `m_read_target`, `m_literal_place`,
//! `m_place_target`, `m_move_changes`, and the `_r` of `place` that `m_place_value` sets);
//! check.py (`is_keylit_node`); model.py (`kind_attrs`).
//! Invariants: a move is checked in refcore's order, each refusal at its line; a target is a
//! place written, every place of a keyed place or of a kind, or a place load can't see, each
//! once in the order met.

use super::check::{Checker, MoveFacts, Scope, Stands, invariant, law_date};
use super::keys::{KeyLit, keyed_place_thing};
use super::moves::own_day_reads;
use super::reads::{is_move_type, is_place_type};
use super::types::{is_date, refused, uerr};
use crate::base::{Code, Load};
use crate::model::{KindSpec, MoveInfo, NameKind, Node, NodeKind, Reality, Rel, Target, Ty};
use crate::syntax::{Expr, Key, Lit, Order, Postfix, Stop};
use std::collections::BTreeSet;

impl Checker {
    /// A move: its binds, its day (read by its own day node), `at`, `for`, `when`, its legs
    /// and attributes against its kind's, `fulfils`, `ties by`, what it declares it reads,
    /// and its written order (refcore's `m_check_move`, `m_move_changes`). L §5.14, §5.14.1,
    /// §5.14.2.
    pub(super) fn check_move(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        file: &str,
    ) -> Result<MoveFacts, Stop> {
        let decl = &mv.decl;
        let lines = &decl.lines;
        let mut scope = Scope::new(
            Node::new(NodeKind::Move, &mv.id),
            Stands::Rule,
            file,
            decl.line,
        );
        for bind in &mv.binds {
            self.bind(m, &mut scope, &bind.node, 0)?;
        }
        scope.keyvars = mv.binds.iter().map(|bind| bind.var.clone()).collect();
        let day = scope.for_item(Node::new(NodeKind::MoveOn, &mv.id));
        self.edge(&scope.item, day.item.clone(), Rel::Any);
        let t = self.expr(m, &day.on_line(lines.on), &decl.on, None, 0)?;
        if !(is_date(m, &t) || matches!(t, Ty::Hole)) {
            let message = format!("on of move {} is {}, not a date", mv.id, t.show());
            return Err(scope.place(uerr(message)));
        }
        if let Some(at) = &decl.at {
            self.direct = Some(Vec::new());
            let law = self.expr(m, &day.on_line(lines.at), at, None, 0);
            if let Some(reads) = self.direct.take() {
                self.move_at.insert(mv.id.clone(), reads);
            }
            law_date(m, &law?, at, &format!("at of move {}", mv.id)).map_err(|s| scope.place(s))?;
        }
        if let Some(period) = &decl.for_period {
            self.expr(m, &scope.on_line(lines.for_period), period, None, 0)?;
        }
        if let Some(when) = &decl.when {
            let what = format!("when of move {}", mv.id);
            self.when(m, &scope.on_line(lines.when), &scope, when, &what)?;
        }
        leg_names(m, mv).map_err(|s| scope.place(s))?;
        let legs = m.kind_legs(&mv.kind);
        let mut changes = Vec::new();
        for leg in &decl.legs {
            if let Some((_, unit, _)) = legs.iter().find(|(name, ..)| *name == leg.name) {
                let lt = self.check_leg(m, &scope.on_line(leg.line), mv, leg, unit)?;
                self.leg_types.insert((mv.id.clone(), leg.name.clone()), lt);
            }
            let on_leg = scope.on_line(leg.line);
            for end in [&leg.from, &leg.to] {
                let change = change_of(m, &scope, end).map_err(|s| on_leg.place(s))?;
                push_once(&mut changes, change);
            }
        }
        self.attrs(m, &scope, mv)?;
        if let Some(fulfils) = &decl.fulfils {
            let ft = self.expr(m, &scope.on_line(lines.fulfils), fulfils, None, 0)?;
            if !is_move_type(m, &ft) {
                let message = format!("fulfils of {} names {}, not a movement", mv.id, ft.show());
                return Err(scope.place(uerr(message)));
            }
        }
        if let Some(ties) = &decl.ties {
            for tie in ties {
                self.expr(m, &scope.on_line(lines.ties), tie, None, 0)?;
            }
        }
        let mut read_targets = Vec::new();
        for read in &decl.reads {
            read_targets.push(self.read_target(m, &scope.on_line(lines.reads), mv, read)?);
        }
        for (order, other) in &decl.order {
            if m.names.kind(other) != Some(NameKind::Move) {
                let word = match order {
                    Order::Before => "before",
                    Order::After => "after",
                };
                let message = format!("unknown move {other} ({word} {other})");
                return Err(scope.place(refused(Load::Name, message)));
            }
        }
        if !decl.reads.is_empty() {
            let found = own_day_reads(mv).map_err(|stop| scope.place(stop))?;
            self.day_reads.extend(found);
        }
        Ok(MoveFacts {
            id: mv.id.clone(),
            read_targets,
            changes,
        })
    }

    /// A move's attributes: each its kind declares, once, of its type, and none missing
    /// (refcore's `m_check_move`, its attributes). L §5.14.
    fn attrs(&mut self, m: &Reality, scope: &Scope, mv: &MoveInfo) -> Result<(), Stop> {
        let declared = attr_types(m, &mv.kind);
        let lines = &mv.decl.lines.attrs;
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (at, (attr, value)) in mv.decl.attrs.iter().enumerate() {
            let Some((_, ty)) = declared.iter().find(|(name, _)| name == attr) else {
                let message = format!("attribute {attr} is missing or unknown ({})", mv.id);
                return Err(scope.place(refused(Load::Leg, message)));
            };
            if !seen.insert(attr) {
                let message = format!("attribute {attr} twice in {}", mv.id);
                return Err(scope.place(refused(Load::Leg, message)));
            }
            let line = lines
                .get(at)
                .copied()
                .ok_or_else(|| invariant(format!("an attr of move {} with no line", mv.id)))?;
            let Some(ty) = ty else {
                return Err(invariant(format!(
                    "attribute {attr} of {} has no type",
                    mv.id
                )));
            };
            let t = self.expr(m, &scope.on_line(line), value, Some(ty), 0)?;
            let what = format!("attribute {attr} of {}", mv.id);
            self.assign(m, scope, ty, value, &t, &what)
                .map_err(|stop| scope.place(stop))?;
        }
        if let Some((attr, _)) = declared
            .iter()
            .find(|(name, _)| !seen.contains(name.as_str()))
        {
            let message = format!("attribute {attr} is missing or unknown ({})", mv.id);
            return Err(scope.place(refused(Load::Leg, message)));
        }
        Ok(())
    }

    /// What a `reads` line names: a move, all of a keyed place, an event kind, or the place
    /// an expression gives, `LOAD-READS` for anything else (refcore's `m_read_target`).
    /// L §5.14.1.
    fn read_target(
        &mut self,
        m: &Reality,
        scope: &Scope,
        mv: &MoveInfo,
        read: &Expr,
    ) -> Result<Target, Stop> {
        if let Expr::Name(id) = read
            && !scope.is_var(id)
        {
            let named = m.names.kind(id);
            if named == Some(NameKind::Move) {
                return Ok(Target::Move(id.clone()));
            }
            let keyed = m
                .places
                .get(id)
                .is_some_and(|place| !place.binds.is_empty());
            if named == Some(NameKind::Place) && keyed {
                return Ok(Target::PlaceSec(id.clone()));
            }
            let event = m
                .kinds
                .get(id)
                .is_some_and(|kind| kind.spec == KindSpec::Event);
            if named == Some(NameKind::Kind) && event {
                return Ok(Target::Kind(id.clone()));
            }
        }
        let t = self.expr(m, scope, read, Some(&Ty::Key("places".to_owned())), 0)?;
        if !is_place_type(m, &t) {
            let message = format!(
                "{} declares reads {}, which is not a place or a move",
                mv.id,
                super::texts::text(read)
            );
            return Err(scope.place(refused(Load::Reads, message)));
        }
        place_target(m, scope, read).map_err(|s| scope.place(s))
    }
}

/// A move's legs: each its kind declares, once, and none missing (refcore's
/// `m_check_move`, its legs). L §5.14.
fn leg_names(m: &Reality, mv: &MoveInfo) -> Result<(), Stop> {
    let declared = m.kind_legs(&mv.kind);
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for leg in &mv.decl.legs {
        if !declared.iter().any(|(name, ..)| *name == leg.name) {
            return Err(refused(
                Load::Leg,
                format!("leg {} is not a leg of {}", leg.name, mv.kind),
            ));
        }
        if !seen.insert(&leg.name) {
            return Err(refused(
                Load::Leg,
                format!("leg {} twice in {}", leg.name, mv.id),
            ));
        }
    }
    if let Some((name, ..)) = declared
        .iter()
        .find(|(name, ..)| !seen.contains(name.as_str()))
    {
        return Err(refused(Load::Leg, format!("{} lacks leg {name}", mv.id)));
    }
    Ok(())
}

/// An event kind's attributes with their types, its parents' first (refcore's
/// `kind_attrs`). L §5.7.
fn attr_types(m: &Reality, kind: &str) -> Vec<(String, Option<Ty>)> {
    let mut out = Vec::new();
    for info in m.chain(kind).into_iter().rev() {
        out.extend(info.attr_types.iter().cloned());
    }
    out
}

/// A name with its postfixes, `x` or `x[k].f`.
pub(super) fn named_parts(expr: &Expr) -> Option<(&str, &[Postfix])> {
    if let Expr::Name(id) = expr {
        return Some((id, &[]));
    }
    if let Expr::Post(base, posts) = expr
        && let Expr::Name(id) = base.as_ref()
    {
        return Some((id, posts));
    }
    None
}

/// A `flows` target written as a move's may be: `m`, `m[k]`, `m.leg` or `m[k].leg`, with
/// its name and keys.
pub(super) fn move_keys(target: &Expr) -> Option<(&str, &[Key])> {
    let (id, posts) = named_parts(target)?;
    match posts {
        [] | [Postfix::Field(_)] => Some((id, &[])),
        [Postfix::Keys(keys)] | [Postfix::Keys(keys), Postfix::Field(_)] => Some((id, keys)),
        _ => None,
    }
}

/// The thing a place expression names literally: an unkeyed place's id, a thing of a place
/// kind, or a keyed place at literal keys (refcore's `m_literal_place`). L §5.10.
pub(super) fn literal_place(
    m: &Reality,
    scope: &Scope,
    expr: &Expr,
) -> Result<Option<String>, Stop> {
    match literal_form(m, scope, expr) {
        None => Ok(None),
        Some(Literal::Thing(thing)) => Ok(Some(thing)),
        Some(Literal::Keyed(id, lits)) => match keyed_place_thing(m, &id, &keyed(lits), "", 0) {
            Ok(thing) => Ok(thing),
            // A key no index or kind holds is refused where the key is typed (`LOAD-KEY`);
            // here it names no literal place. `LOAD-KEY` is the one refusal of a key.
            Err(Stop::Refused(r)) if r.code() == Code::Load(Load::Key) => Ok(None),
            Err(stop) => Err(stop),
        },
    }
}

/// Literal keys as keys written at a place's positions.
fn keyed(lits: Vec<Lit>) -> Vec<KeyLit> {
    lits.into_iter().map(KeyLit::Lit).collect()
}

/// A place expression read literally, before a keyed place's keys are looked up.
enum Literal {
    /// The thing it names.
    Thing(String),
    /// A keyed place, at keys written as literals.
    Keyed(String, Vec<Lit>),
}

/// What a place expression names literally, its keys not yet looked up (refcore's
/// `m_literal_place`, up to its `key_thing`). L §5.10.
fn literal_form(m: &Reality, scope: &Scope, expr: &Expr) -> Option<Literal> {
    let (id, posts) = named_parts(expr)?;
    if scope.is_var(id) {
        return None;
    }
    let named = m.names.kind(id);
    if named.is_none() && posts.is_empty() {
        let thing = m.things.get(id)?;
        return m.is_place(thing).then(|| Literal::Thing(id.to_owned()));
    }
    let place = m
        .places
        .get(id)
        .filter(|_| named == Some(NameKind::Place))?;
    if posts.is_empty() {
        let unkeyed = place.things.iter().find(|(keys, _)| keys.is_empty());
        return unkeyed.map(|(_, thing)| Literal::Thing(thing.clone()));
    }
    let [Postfix::Keys(keys)] = posts else {
        return None;
    };
    let mut lits = Vec::new();
    for key in keys {
        lits.push(key_literal(m, scope, key)?);
    }
    Some(Literal::Keyed(id.to_owned(), lits))
}

/// A key written as a literal at a key position: a word that is no variable and no id, a
/// key, a day, a month, a month-day, a currency, or a whole number with no unit (refcore's
/// `is_keylit_node`). L §2.5.
fn key_literal(m: &Reality, scope: &Scope, key: &Key) -> Option<Lit> {
    let Key::Expr(expr) = key else {
        return None;
    };
    if let Expr::Name(name) = expr {
        let literal = !scope.is_var(name) && !m.names.contains_key(name);
        return literal.then(|| Lit::Name(name.clone()));
    }
    let Expr::Lit(lit) = expr else {
        return None;
    };
    let keylike = match lit {
        Lit::Key(_) | Lit::Day(_) | Lit::Month(..) | Lit::MonthDay(..) | Lit::Cur(_) => true,
        Lit::Num(number) => number.whole && number.unit.is_none(),
        Lit::Quarter(..) | Lit::Name(_) | Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) => false,
    };
    keylike.then(|| lit.clone())
}

/// The keyed place section a place expression reads, `p` or `p[k]` (refcore's `_r` of
/// `place`).
pub(super) fn place_section(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let (id, posts) = named_parts(expr)?;
    let read = posts.is_empty() || matches!(posts, [Postfix::Keys(_)]);
    let place = !scope.is_var(id) && m.names.kind(id) == Some(NameKind::Place);
    (read && place).then(|| id.to_owned())
}

/// What a running read or a declared read of a place names (refcore's `m_place_target`).
/// E §3.1 step 7.
pub(super) fn place_target(m: &Reality, scope: &Scope, expr: &Expr) -> Result<Target, Stop> {
    if let Some(thing) = literal_place(m, scope, expr)? {
        return Ok(Target::Thing(thing));
    }
    Ok(place_section(m, scope, expr).map_or(Target::Unknown, Target::PlaceSec))
}

/// What a leg's place changes: the place written, every place of a keyed place, every
/// place of the kind its type names, or a place load can't see (refcore's
/// `m_move_changes`). L §5.14.2.
fn change_of(m: &Reality, scope: &Scope, end: &Expr) -> Result<Target, Stop> {
    if let Some(thing) = literal_place(m, scope, end)? {
        return Ok(Target::Thing(thing));
    }
    if let Some(section) = place_section(m, scope, end) {
        return Ok(Target::PlaceSec(section));
    }
    Ok(place_kind(m, scope, end).map_or(Target::Unknown, Target::Kind))
}

/// The kind a place expression's type names, where load sees it: a variable bound over a
/// kind, a place, a thing, or an input or def of type `key K` (refcore's `_t.ix`).
fn place_kind(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let mut inner = expr;
    while let Expr::Paren(nested) = inner {
        inner = nested;
    }
    let (id, posts) = named_parts(inner)?;
    if posts.len() > 1 || posts.iter().any(|post| matches!(post, Postfix::Field(_))) {
        return None;
    }
    if let Some(var) = scope.vars.get(id) {
        return var.kind().map(str::to_owned);
    }
    let named = m.names.kind(id)?;
    let ty = match named {
        NameKind::Place => return m.places.get(id).map(|place| place.kind.clone()),
        NameKind::Input => m.inputs.get(id).and_then(|input| input.ty.as_ref()),
        NameKind::Def => m.defs.get(id).map(|def| &def.ty),
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Fill
        | NameKind::Move
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source
        | NameKind::Cut => None,
    };
    if let Some(Ty::Key(kind)) = ty
        && m.kinds.contains(kind)
    {
        return Some(kind.clone());
    }
    None
}

/// Adds `target` to `targets` unless it is there.
fn push_once(targets: &mut Vec<Target>, target: Target) {
    if !targets.contains(&target) {
        targets.push(target);
    }
}
