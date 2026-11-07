//! Load's checker, its reads of the ledger: `bal`, `low`, `flows`, `gap`, `covered`, `observed`, `next`, `unit`, a leg's `of`, projections, each typed and marked.
//! Spec: L §5.10 (`bal`, `unit`, a place's unit), §5.14 (The leg read), §5.14.1 (running
//! reads, projections, `next`), §5.14.3 (`flows` of a place or a move), §5.19 (`observed`,
//! `gap`, `covered`), §3.6 (`of q`), §8.3 (`LOAD-UNIT`, `-PARSE`, `-LEG`, `-KEY`, `-NAME`,
//! `-DOMAIN`, `-FIELD`, `-PARTS`); E §3.1 step 7 (the marks).
//! Twin: engine/tools/refcore/check3.py (`m_place_arg`, `m_place_type_unit`, `m_t_bal`,
//! `m_check_of`, `m_t_legof`, `m_legof_read`, `m_check_proj`, `m_t_low`, `m_t_flows`,
//! `m_c_next`, `m_c_covered`, `m_c_gap`, `m_c_observed`, `m_c_unit`, `m_unit_of_quantity`,
//! `m_leg_type`).
//! Invariants: each read is typed in refcore's order, its marks and edges recorded as refcore
//! records them; a type load doesn't see yet (`Unseen`) passes every one of these checks, and
//! `Dyn`, refcore's `DYN`, is checked as refcore checks it (no place, no movement);
//! an `of q` reads the labels its place's kind keeps (the parts clauses, `parts.rs`) and is
//! `LOAD-PARTS` where load sees a place that keeps none of q's sort.

use super::check::{
    Checker, ProjRead, ReadIn, Scope, invariant, literal_place, move_keys, named_parts,
    place_section, place_target,
};
use super::types::{Typed, is_date, numcls, refused, text, uerr, unit_type};
use crate::base::Load;
use crate::model::{
    KindInfo, KindSpec, Labels, Mark, NameKind, Node, NodeKind, Reality, Rel, Target, Ty,
};
use crate::syntax::{Bal, Expr, Flows, Key, Lit, Low, OfQ, Postfix, Proj, Span, Stop};
use std::collections::BTreeSet;

/// Which sort an `of q` asks for at a position (refcore's `sort_of`'s `want`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Want {
    /// Either.
    Any,
    /// A party: the pair's first.
    Owner,
    /// A batch: the pair's second.
    Batch,
}

/// The sort of label an `of q` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sort {
    /// A party's.
    Owner,
    /// A batch's.
    Batch,
}

/// Whether `t` is a place's type: a key of a place kind (refcore's `m_is_place_type`); a type
/// load doesn't see passes.
pub(super) fn is_place_type(m: &Reality, t: &Ty) -> bool {
    key_of_spec(m, t, KindSpec::Place)
}

/// Whether `t` is a movement's type: a key of an event kind (refcore's `m_is_move_type`); a
/// type load doesn't see passes.
pub(super) fn is_move_type(m: &Reality, t: &Ty) -> bool {
    key_of_spec(m, t, KindSpec::Event)
}

/// Whether `t` is a key of a kind of `spec`, or a type load doesn't see.
fn key_of_spec(m: &Reality, t: &Ty, spec: KindSpec) -> bool {
    if matches!(t, Ty::Unseen) {
        return true;
    }
    let Ty::Key(ix) = t else {
        return false;
    };
    m.kinds.get(ix).is_some_and(|k| k.spec == spec)
}

/// Whether `leg` is a leg of a movement of type `t`: of its kind, or of any event kind for a
/// movement whose kind load doesn't know (`events`), or a type load doesn't see (refcore's
/// `m_movement_field`, its `('mvfield', 'leg')` reading).
fn movement_leg(m: &Reality, t: &Ty, leg: &str) -> bool {
    if matches!(t, Ty::Unseen) {
        return true;
    }
    let Ty::Key(kind) = t else {
        return false;
    };
    if kind == "events" {
        return m
            .kinds
            .entries()
            .filter(|(_, k)| k.spec == KindSpec::Event)
            .any(|(id, _)| kind_has_leg(m, id, leg));
    }
    kind_has_leg(m, kind, leg)
}

/// Whether the event kind `kind` declares, or inherits, the leg `leg`.
fn kind_has_leg(m: &Reality, kind: &str, leg: &str) -> bool {
    m.kind_legs(kind).iter().any(|(name, ..)| name == leg)
}

/// A span's text, as a message names it (refcore's `_txt` of a span).
pub(super) fn span_text(span: &Span) -> String {
    let a = span.from.as_ref().map_or_else(String::new, text);
    if !span.dots {
        return a;
    }
    match &span.to {
        Some(b) => format!("{a} .. {}", text(b)),
        None => format!("{a} .."),
    }
}

/// The unit type of a place given by `e` of type `t`: the unit of the place it names, of a
/// keyed place's sections when they hold one, or of the place kind its type names; `Dyn`
/// otherwise (refcore's `m_place_type_unit`). L §5.10.
pub(super) fn place_type_unit(m: &Reality, scope: &Scope, t: &Ty, e: &Expr) -> Typed {
    if let Some(thing) = literal_place(m, scope, e)?.and_then(|name| m.things.get(&name)) {
        return unit_type(m, m.place_unit(thing).as_deref());
    }
    if let Some(info) = place_section(m, scope, e).and_then(|id| m.places.get(&id)) {
        let mut units: BTreeSet<String> = info
            .things
            .iter()
            .filter_map(|(_, name)| m.things.get(name))
            .filter_map(|thing| m.place_unit(thing))
            .collect();
        if let Some(unit) = info.unit.as_ref().filter(|u| *u != "any") {
            units.insert(unit.clone());
        }
        if let [one] = units.iter().collect::<Vec<_>>().as_slice()
            && *one != "any"
        {
            return unit_type(m, Some(one));
        }
    }
    if let Ty::Key(ix) = t
        && m.kinds.get(ix).is_some_and(|k| k.spec == KindSpec::Place)
    {
        return unit_type(m, m.kind_unit(ix));
    }
    Ok(Ty::Dyn)
}

/// A leg's record type: its places, its amount of the leg's unit, its day (refcore's
/// `m_leg_type`). L §5.14.
pub(super) fn leg_type(m: &Reality, unit: &str) -> Typed {
    let places = || Ty::Key("places".to_owned());
    Ok(Ty::Record(vec![
        ("from".to_owned(), places()),
        ("to".to_owned(), places()),
        ("amount".to_owned(), unit_type(m, Some(unit))?),
        ("day".to_owned(), Ty::Date),
    ]))
}

impl Checker {
    /// A place argument: `LOAD-UNIT` unless it is a place (refcore's `m_place_arg`). L §5.10.
    pub(super) fn place_arg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        what: &str,
        depth: u32,
    ) -> Typed {
        let t = self.expr(m, scope, e, Some(&Ty::Key("places".to_owned())), depth)?;
        if !is_place_type(m, &t) {
            return Err(uerr(format!(
                "{what} of {}, which is not a place",
                t.show()
            )));
        }
        Ok(t)
    }

    /// A balance read: a place, a unit's key and a day; marked as a running read, a
    /// projection or `bal(x)` with no day; its type the place's unit (refcore's `m_t_bal`).
    /// L §5.10, §5.14.1; E §3.1 step 7.
    pub(super) fn bal(&mut self, m: &Reality, scope: &Scope, bal: &Bal, depth: u32) -> Typed {
        let t = self.place_arg(m, scope, &bal.place, "bal", depth)?;
        let mut ut = place_type_unit(m, scope, &t, &bal.place)?;
        let target = place_target(m, scope, &bal.place)?;
        let day = match bal.args.as_slice() {
            [unit, day] => {
                self.expr(
                    m,
                    scope,
                    unit,
                    Some(&Ty::Key("currencies".to_owned())),
                    depth,
                )?;
                ut = match unit_named(m, scope, unit) {
                    Some(name) => unit_type(m, Some(&name))?,
                    None => Ty::Dyn,
                };
                Some(day)
            }
            [day] => Some(day),
            _ => None,
        };
        if let Some(day) = day {
            let dt = self.expr(m, scope, day, None, depth)?;
            if !is_date(m, &dt) {
                return Err(uerr(format!("the day of bal is {}, not a date", dt.show())));
            }
        } else if bal.start || bal.modelled || stated(&bal.proj) {
            return Err(refused(
                Load::Parse,
                "cannot parse: bal(x) with no day takes no clause".to_owned(),
            ));
        }
        if !bal.modelled {
            if bal.proj.from.is_some() {
                if matches!(scope.item.kind, NodeKind::Move | NodeKind::MoveOn)
                    && scope.read_in != ReadIn::Query
                {
                    self.projs.push(ProjRead {
                        bal: bal.clone(),
                        holder: scope.item.id.clone(),
                        target: target.clone(),
                        at: scope.file_line(),
                    });
                }
                self.proj(m, scope, &bal.proj, target, depth)?;
            } else if day.is_none() {
                self.graph.balnow.insert(scope.item.clone());
                self.mark(&scope.item, Mark::BalNow, target);
            } else {
                self.mark(&scope.item, Mark::Bal, target);
            }
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        if let Some(of) = &bal.of {
            self.check_of(m, scope, of, Some((&bal.place, &t)), depth)?;
        }
        Ok(ut)
    }

    /// `of q`: a party, a batch, or the pair (a party, then a batch); `LOAD-PARTS` where load
    /// sees the place's kind, a literal place or a place type of a kind that keeps parts, and
    /// it keeps no labels of q's sort (refcore's `m_check_of`). L §3.6, §5.10, §5.14.
    pub(super) fn check_of(
        &mut self,
        m: &Reality,
        scope: &Scope,
        of: &OfQ,
        place: Option<(&Expr, &Ty)>,
        depth: u32,
    ) -> Result<(), Stop> {
        // The pair needs a party's and a batch's labels; one `q` needs its sort's, where load
        // sees it (refcore's `need`).
        let need = match of {
            OfQ::Pair(party, batch) => {
                self.sort_of(m, scope, party, Want::Owner, depth)?;
                self.sort_of(m, scope, batch, Want::Batch, depth)?;
                Labels {
                    owner: true,
                    batch: true,
                }
            }
            OfQ::Ref(q) => {
                let sort = self.sort_of(m, scope, q, Want::Any, depth)?;
                Labels {
                    owner: sort == Some(Sort::Owner),
                    batch: sort == Some(Sort::Batch),
                }
            }
        };
        let Some((p, t)) = place else {
            return Ok(());
        };
        let Some((at, keeps)) = seen_labels(m, scope, p, t)? else {
            return Ok(());
        };
        if keeps.holds(need) {
            return Ok(());
        }
        let q = match of {
            OfQ::Ref(q) => text(q),
            OfQ::Pair(..) => "ofpair".to_owned(),
        };
        Err(refused(
            Load::Parts,
            format!("of {q} at {at}, which keeps no labels of its sort"),
        ))
    }

    /// The sort an `of q` position names: a party's or a batch's label; none where load
    /// doesn't see its type (refcore's `sort_of`).
    fn sort_of(
        &mut self,
        m: &Reality,
        scope: &Scope,
        node: &Expr,
        want: Want,
        depth: u32,
    ) -> Result<Option<Sort>, Stop> {
        let parties = Ty::Key("parties".to_owned());
        let expect = (want != Want::Batch).then_some(&parties);
        let t = self.expr(m, scope, node, expect, depth)?;
        if matches!(t, Ty::Dyn | Ty::Hole | Ty::Unseen) {
            return Ok(None);
        }
        if let Ty::Key(ix) = &t
            && let Some(kind) = m.kinds.get(ix)
        {
            if m.subtree("parties").contains(ix) {
                if want == Want::Batch {
                    return Err(uerr(format!(
                        "of {}: {} is a party, not a batch",
                        text(node),
                        t.show()
                    )));
                }
                return Ok(Some(Sort::Owner));
            }
            if kind.spec == KindSpec::Event {
                if want == Want::Owner {
                    return Err(uerr(format!(
                        "of {}: {} is a batch, not a party",
                        text(node),
                        t.show()
                    )));
                }
                return Ok(Some(Sort::Batch));
            }
        }
        Err(uerr(format!(
            "of {} is {}, not a party, a batch or the pair",
            text(node),
            t.show()
        )))
    }

    /// `m.leg.amount of q`: booking state, read as a balance is (refcore's `m_t_legof`,
    /// `m_legof_read`). L §5.14 The leg read, §5.14.1 (R-10).
    pub(super) fn leg_of(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        of: &OfQ,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let (t, leg_amount) = self.leg_amount(m, scope, inner, expect, depth)?;
        if !leg_amount {
            return Err(refused(
                Load::Field,
                format!(
                    "of after {}: a read of parts follows a leg's amount (m.leg.amount of q)",
                    text(inner)
                ),
            ));
        }
        self.check_of(m, scope, of, None, depth)?;
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.mark(&scope.item, Mark::LegOf, Target::Unknown);
        Ok(t)
    }

    /// `e`'s type, and whether `e` is a movement leg's amount, `m.leg.amount`: its last field
    /// `amount`, the one before it a leg of the movement's kind (refcore's `m_t_legof`, its
    /// test of `f._r`).
    fn leg_amount(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Result<(Ty, bool), Stop> {
        let Expr::Post(base, posts) = e else {
            return Ok((self.expr(m, scope, e, expect, depth)?, false));
        };
        let at = posts.len().checked_sub(2);
        let head = at.and_then(|at| posts.get(..at));
        let tail = at.and_then(|at| posts.get(at..));
        let (Some(head), Some([Postfix::Field(leg), Postfix::Field(amount)])) = (head, tail) else {
            return Ok((self.expr(m, scope, e, expect, depth)?, false));
        };
        if amount != "amount" {
            return Ok((self.expr(m, scope, e, expect, depth)?, false));
        }
        let (ot, first) = if head.is_empty() {
            (self.expr(m, scope, base, None, depth)?, Some(base.as_ref()))
        } else {
            let owner = Expr::Post(base.clone(), head.to_vec());
            (self.expr(m, scope, &owner, None, depth)?, None)
        };
        let is_leg = movement_leg(m, &ot, leg);
        let record = Self::field(m, scope, first, &ot, leg).map_err(|stop| scope.place(stop))?;
        let t = Self::field(m, scope, None, &record, amount).map_err(|stop| scope.place(stop))?;
        Ok((t, is_leg))
    }

    /// A projection's clauses: `from t` a date; a projection mark (refcore's
    /// `m_check_proj`). L §5.14.1; E §3.1 step 7.
    fn proj(
        &mut self,
        m: &Reality,
        scope: &Scope,
        proj: &Proj,
        target: Target,
        depth: u32,
    ) -> Result<(), Stop> {
        if let Some(from) = &proj.from {
            let ft = self.expr(m, scope, from, None, depth)?;
            if !is_date(m, &ft) {
                return Err(uerr(format!(
                    "from of a projection is {}, not a date",
                    ft.show()
                )));
            }
        }
        if proj.random.is_some() {
            return Err(Stop::NotBuilt("a projection's `random at`"));
        }
        self.graph.holders.insert(scope.item.clone());
        self.mark(&scope.item, Mark::Proj, target);
        Ok(())
    }

    /// `low(x, span)`: a place over a closed span, as a running read or a projection
    /// (refcore's `m_t_low`). L §5.14.1.
    pub(super) fn low(&mut self, m: &Reality, scope: &Scope, low: &Low, depth: u32) -> Typed {
        let t = self.place_arg(m, scope, &low.place, "low", depth)?;
        let ut = place_type_unit(m, scope, &t, &low.place)?;
        let target = place_target(m, scope, &low.place)?;
        self.span(m, scope, &low.span, depth)?;
        if low.span.from.is_none() || low.span.to.is_none() {
            return Err(refused(
                Load::Domain,
                "low over a span with an open end".to_owned(),
            ));
        }
        if low.proj.from.is_some() {
            self.proj(m, scope, &low.proj, target, depth)?;
        } else {
            self.mark(&scope.item, Mark::Bal, target);
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        Ok(ut)
    }

    /// `flows(x, span …)`: of a move (its leg named, or its one leg) or of a place; its unit
    /// the leg's or the place's, or the currency it converts to (refcore's `m_t_flows`).
    /// L §5.14.3, §5.14.1.
    pub(super) fn flows(&mut self, m: &Reality, scope: &Scope, flows: &Flows, depth: u32) -> Typed {
        let hit = flow_move(&flows.target)
            .filter(|(id, _, _)| !scope.is_var(id) && m.names.kind(id) == Some(NameKind::Move));
        let mut ut = match hit {
            Some((id, keys, leg)) => self.move_flows(m, scope, id, keys, leg, depth)?,
            None => self.place_flows(m, scope, flows, depth)?,
        };
        self.span(m, scope, &flows.span, depth)?;
        if let Some(key) = &flows.for_key {
            self.expr(m, scope, key, None, depth)?;
        }
        if let Some((cur, fx, mode)) = &flows.convert {
            if !m.currencies.contains_key(cur) {
                return Err(uerr(format!("money in an undeclared currency {cur}")));
            }
            let ft = self.expr(m, scope, fx, None, depth)?;
            let rate =
                matches!(ft, Ty::Unseen) || matches!(&ft, Ty::Num(unit, _) if !unit.is_one());
            if !rate {
                return Err(uerr(format!(
                    "flows in {cur} at {}: the rate is {}",
                    text(fx),
                    ft.show()
                )));
            }
            self.mode_arg(m, scope, "flows", mode, depth)?;
            ut = Ty::Money(cur.clone());
        }
        Ok(ut)
    }

    /// `flows(m[k].leg, …)` of move `id`: its keys, its leg, the leg's unit (refcore's
    /// `m_t_flows`, its move arm).
    fn move_flows(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        leg: Option<&str>,
        depth: u32,
    ) -> Typed {
        let mv = m.moves.at(m.names.move_id(id)?)?;
        if keys.len() > mv.binds.len() {
            return Err(refused(Load::Key, format!("wrong number of keys for {id}")));
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
        let mut legs = m.kind_legs(&mv.kind);
        if let Some(name) = leg {
            legs.retain(|(leg_name, ..)| leg_name == name);
            if legs.is_empty() {
                return Err(refused(
                    Load::Leg,
                    format!("flows({id}.{name}, span): {} names no leg {name}", mv.kind),
                ));
            }
        } else if legs.len() != 1 {
            return Err(refused(
                Load::Leg,
                format!(
                    "flows({id}, span): the kind {} has {} legs; name one, flows({id}.<leg>, ...)",
                    mv.kind,
                    legs.len()
                ),
            ));
        }
        let [(name, unit, _)] = legs.as_slice() else {
            return Err(invariant(format!(
                "unreachable: flows of {id} names one leg here, its kind's or the one written"
            )));
        };
        let mut ut = unit_type(m, Some(unit))?;
        if matches!(ut, Ty::Dyn)
            && let Some(lt) = self.leg_types.get(&(id.to_owned(), name.clone()))
            && !matches!(lt, Ty::Dyn)
        {
            ut = lt.clone();
        }
        self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Flows);
        if mv.decision {
            self.mark(&scope.item, Mark::Flows, Target::Move(id.to_owned()));
        }
        Ok(ut)
    }

    /// `flows(x, …)` of a place: `LOAD-UNIT` for a place of unit `any` that doesn't convert
    /// (refcore's `m_t_flows`, its place arm). L §5.14.3.
    fn place_flows(&mut self, m: &Reality, scope: &Scope, flows: &Flows, depth: u32) -> Typed {
        let target = &flows.target;
        let t = self.place_arg(m, scope, target, "flows", depth)?;
        let ut = place_type_unit(m, scope, &t, target)?;
        if flows.convert.is_none() {
            let held = if let Some(thing) =
                literal_place(m, scope, target)?.and_then(|n| m.things.get(&n))
            {
                m.place_unit(thing)
            } else if let Ty::Key(ix) = &t {
                m.kind_unit(ix).map(str::to_owned)
            } else {
                None
            };
            if held.as_deref() == Some("any") {
                return Err(uerr(format!(
                    "flows of {}, a place of unit any: it sums no one unit; convert it (in CUR at fx mode)",
                    text(target)
                )));
            }
        }
        let place = place_target(m, scope, target)?;
        self.mark(&scope.item, Mark::Flows, place);
        self.edge(&scope.item, Node::balances(), Rel::Flows);
        Ok(ut)
    }

    /// `covered(x, span)`: a place and a span (refcore's `m_c_covered`). L §5.19.
    pub(super) fn covered(
        &mut self,
        m: &Reality,
        scope: &Scope,
        place: &Expr,
        span: &Span,
        depth: u32,
    ) -> Typed {
        self.place_arg(m, scope, place, "covered", depth)?;
        self.span(m, scope, span, depth)?;
        Ok(Ty::Bool)
    }

    /// `covered` as a call, `household.covered(x, d)` in a scenario's expression row, its base
    /// prefix dropped: two arguments, the first a place; a call's argument is an expression,
    /// never the span `covered` reads, so it is refused there (refcore's `m_c_covered` on a
    /// call). L §2.3 Calls, `coveredx`, §5.19.
    pub(super) fn covered_call(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        depth: u32,
    ) -> Typed {
        let [place, _] = args else {
            return Err(refused(
                Load::Parse,
                "cannot parse: covered(x, span) takes two arguments".to_owned(),
            ));
        };
        self.place_arg(m, scope, place, "covered", depth)?;
        Err(refused(
            Load::Parse,
            "cannot parse: covered(x, span)".to_owned(),
        ))
    }

    /// `gap(x, d)`: a place and a date, read as a balance; the place's unit (refcore's
    /// `m_c_gap`). L §5.19.
    pub(super) fn gap(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        let [place, day] = args else {
            return Err(refused(
                Load::Parse,
                "cannot parse: gap(x, d) takes two arguments".to_owned(),
            ));
        };
        let t = self.place_arg(m, scope, place, "gap", depth)?;
        let dt = self.expr(m, scope, day, None, depth)?;
        if !is_date(m, &dt) {
            return Err(uerr(format!("gap(x, d): d is {}, not a date", dt.show())));
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        place_type_unit(m, scope, &t, place)
    }

    /// `observed(m[k])`: a planned instance; a record of its legs and its day; every observed
    /// input read (refcore's `m_c_observed`). L §5.19.
    pub(super) fn observed(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        depth: u32,
    ) -> Typed {
        let [instance] = args else {
            return Err(refused(
                Load::Parse,
                "cannot parse: observed(m[k]) takes one argument".to_owned(),
            ));
        };
        let empty: &[Key] = &[];
        let planned = named_parts(instance).and_then(|(id, posts)| match posts {
            [] => Some((id, empty)),
            [Postfix::Keys(keys)] => Some((id, keys.as_slice())),
            [Postfix::Field(_), ..] | [Postfix::Keys(_), _, ..] => None,
        });
        let Some((id, keys)) =
            planned.filter(|(id, _)| !scope.is_var(id) && m.names.kind(id) == Some(NameKind::Move))
        else {
            return Err(refused(
                Load::Parse,
                "cannot parse: observed takes a planned instance m[k]".to_owned(),
            ));
        };
        let kind = self.move_instance(m, scope, id, keys, depth)?;
        let mut fields = Vec::new();
        for (leg, unit, _) in m.kind_legs(&kind) {
            fields.push((leg, leg_type(m, &unit)?));
        }
        fields.push(("day".to_owned(), Ty::Date));
        for (input, info) in m.inputs.entries() {
            if info.observed {
                self.edge(
                    &scope.item,
                    Node::new(NodeKind::Input, input),
                    Rel::Observed,
                );
            }
        }
        Ok(Ty::Record(fields))
    }

    /// `next(K, d)`: an event kind and a date; every move of K or a subkind read (refcore's
    /// `m_c_next`). L §5.14.1.
    pub(super) fn next(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        let [kind, day] = args else {
            return Err(refused(
                Load::Parse,
                "cannot parse: next(K, d) takes two arguments".to_owned(),
            ));
        };
        let event = if let Expr::Name(id) = kind
            && m.names.kind(id) == Some(NameKind::Kind)
        {
            m.kinds.get(id).filter(|k| k.spec == KindSpec::Event)
        } else {
            None
        };
        let Some(kid) = event.map(|k| k.id.clone()) else {
            return Err(refused(
                Load::Name,
                format!("next takes an event kind: {}", text(kind)),
            ));
        };
        let dt = self.expr(m, scope, day, None, depth)?;
        if !is_date(m, &dt) {
            return Err(uerr(format!("next(K, d): d is {}, not a date", dt.show())));
        }
        let sub = m.subtree(&kid);
        for (move_id, mv) in m.moves.entries() {
            if sub.contains(&mv.kind) {
                self.edge(&scope.item, Node::new(NodeKind::Move, move_id), Rel::Next);
            }
        }
        Ok(Ty::Key(kid))
    }

    /// `unit(x)`: of a place, its unit as a key of the unit kind holding it; of a quantity,
    /// its unit's (refcore's `m_c_unit`). L §5.10 Kind and unit.
    pub(super) fn unit(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Typed {
        let [arg] = args else {
            return Err(refused(
                Load::Parse,
                "cannot parse: unit(x) takes one argument".to_owned(),
            ));
        };
        let t = self.expr(m, scope, arg, Some(&Ty::Key("places".to_owned())), depth)?;
        if !is_place_type(m, &t) {
            return unit_of_quantity(m, &t);
        }
        let mut units: BTreeSet<Option<String>> = BTreeSet::new();
        if let Some(thing) = literal_place(m, scope, arg)?.and_then(|n| m.things.get(&n)) {
            units = BTreeSet::from([m.place_unit(thing)]);
        } else if let Ty::Key(ix) = &t
            && m.kinds.get(ix).is_some()
        {
            units = match m.kind_unit(ix) {
                Some(unit) if unit != "one" => BTreeSet::from([Some(unit.to_owned())]),
                Some(_) | None => kind_place_units(m, ix),
            };
        }
        let any = Some("any".to_owned());
        if units.len() == 1 && units.contains(&any) {
            return Err(uerr(format!(
                "unit({}) of a place of unit any: it has no one unit",
                text(arg)
            )));
        }
        if !units.is_empty() && units.iter().all(|u| quantity_unit(m, u.as_deref())) {
            let named: Vec<&str> = units.iter().filter_map(Option::as_deref).collect();
            return Err(uerr(format!(
                "unit({}): its unit {} is a quantity unit, a key of no unit kind",
                text(arg),
                named.join(", ")
            )));
        }
        let mut kinds: BTreeSet<Option<String>> = BTreeSet::new();
        for unit in &units {
            let kind = match unit.as_deref() {
                Some(u) if m.currencies.contains_key(u) => Some("currencies".to_owned()),
                Some(u) => m.unit_kind_keys.get(u).cloned(),
                None => None,
            };
            kinds.insert(kind);
        }
        if let [Some(kind)] = kinds.iter().collect::<Vec<_>>().as_slice() {
            return Ok(Ty::Key((*kind).clone()));
        }
        Ok(Ty::UnitKey)
    }
}

/// `unit(q)` of a quantity: the unit as a key of the unit kind holding it (refcore's
/// `m_unit_of_quantity`). L §5.10 Kind and unit.
fn unit_of_quantity(m: &Reality, t: &Ty) -> Typed {
    if matches!(t, Ty::Dyn | Ty::Hole | Ty::Flex(_) | Ty::Unseen) {
        return Ok(Ty::UnitKey);
    }
    let class = numcls(m, t);
    let terms: Vec<(String, i64)> = if let (Some("money"), Ty::Money(cur)) = (class, t) {
        vec![(cur.clone(), 1)]
    } else if let (Some("num"), Ty::Num(unit, _)) = (class, t) {
        unit.terms().to_vec()
    } else {
        return Err(uerr(format!(
            "unit of {}, which is neither a place nor a quantity",
            t.show()
        )));
    };
    let [(u, 1)] = terms.as_slice() else {
        return Err(uerr(format!(
            "unit of {}: a plain number or a compound unit is a key of no unit kind",
            t.show()
        )));
    };
    if m.currencies.contains_key(u) {
        return Ok(Ty::Key("currencies".to_owned()));
    }
    if m.kinds.get(u).is_some_and(|k| k.spec == KindSpec::Unit) {
        return Ok(Ty::Key(u.clone()));
    }
    if let Some(kind) = m.unit_kind_keys.get(u) {
        return Ok(Ty::Key(kind.clone()));
    }
    Err(uerr(format!(
        "unit of {}: {u} is a quantity unit, a key of no unit kind",
        t.show()
    )))
}

/// Where load sees the kind of the place an `of q` reads, and the labels it keeps: a literal
/// place's base kind's (none if it keeps no parts), or a place type's kind where that kind
/// keeps parts (refcore's `m_check_of`, its `seen`). L §3.6, §5.10.
fn seen_labels(
    m: &Reality,
    scope: &Scope,
    place: &Expr,
    t: &Ty,
) -> Result<Option<(String, Labels)>, Stop> {
    let none = Labels {
        owner: false,
        batch: false,
    };
    if let Some(thing) = literal_place(m, scope, place)? {
        let keeps = m
            .things
            .get(&thing)
            .and_then(|th| m.thing_parts(th))
            .map_or(none, KindInfo::labels);
        return Ok(Some((thing, keeps)));
    }
    let Ty::Key(kind) = t else {
        return Ok(None);
    };
    Ok(m.parts_kind(kind)
        .map(|keeps| (kind.clone(), keeps.labels())))
}

/// Whether a projection states any clause.
fn stated(proj: &Proj) -> bool {
    proj.from.is_some() || proj.windows_each || proj.random.is_some() || proj.uncertain
}

/// The units the places of kind `kind` and its subkinds hold, a place's none where it holds
/// no one unit (refcore's `m_c_unit`, its kind arm).
fn kind_place_units(m: &Reality, kind: &str) -> BTreeSet<Option<String>> {
    let sub = m.subtree(kind);
    let mut units = BTreeSet::new();
    for (_, thing) in m.things.entries() {
        let of_kind = thing.base.as_ref().is_some_and(|base| sub.contains(base));
        if of_kind && m.is_place(thing) {
            units.insert(m.place_unit(thing));
        }
    }
    units
}

/// Whether `unit` is a quantity unit: a unit that is no currency, no key of a unit kind,
/// neither `any` nor `one` (refcore's `m_c_unit`, its `quantity` test).
fn quantity_unit(m: &Reality, unit: Option<&str>) -> bool {
    let Some(u) = unit else {
        return false;
    };
    !m.currencies.contains_key(u)
        && !m.unit_kind_keys.contains_key(u)
        && !matches!(u, "any" | "one")
}

/// A balance's unit argument named as a unit: a currency, or a key written as a literal
/// (refcore's `m_t_bal`, its `args[0]._r`).
fn unit_named(m: &Reality, scope: &Scope, unit: &Expr) -> Option<String> {
    if let Expr::Lit(Lit::Cur(cur)) = unit {
        return Some(cur.clone());
    }
    let (Expr::Name(name) | Expr::Lit(Lit::Name(name))) = unit else {
        return None;
    };
    (!scope.is_var(name) && !m.names.contains_key(name)).then(|| name.clone())
}

/// A `flows` target read as a move: its id, its keys, its leg (refcore's `m_t_flows`, its
/// `hit`); the caller checks that the id is a move.
fn flow_move(target: &Expr) -> Option<(&str, &[Key], Option<&str>)> {
    let (id, keys) = move_keys(target)?;
    let leg = named_parts(target).and_then(|(_, posts)| match posts.last() {
        Some(Postfix::Field(leg)) => Some(leg.as_str()),
        Some(Postfix::Keys(_)) | None => None,
    });
    Some((id, keys, leg))
}
