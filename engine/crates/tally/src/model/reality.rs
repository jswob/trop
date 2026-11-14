//! The reality as load leaves it: the namespace, the kinds and things, the indexes, the versions and the declarations, with the questions asked of them.
//! Spec: E §3.1 (steps 2, 4, 9); L §2.5 (one namespace), §5.7 (kinds: units, signs, legs,
//! attributes by inheritance; lifespans under a cut), §5.10 (a place's unit and price).
//! Twin: engine/tools/refcore/model.py (`Reality`, `BUILTIN_INPUTS`, `BUILTIN_INDEXES`,
//! `ThingInfo.span_in`, `kind_unit` to `lifespan`).
//! Invariants: every name is in the namespace once; a kind's chain of parents ends; a thing's
//! lifespan under a cut is its latest entry the cut and the read's world see, a `replace` line
//! winning (L §6.4: a scenario's entry is seen in its world and those over it).

use super::decl::{BandInfo, DefInfo, Domain, InputInfo, MoveInfo, PlaceInfo};
use super::graph::Graph;
use super::ids::{
    AxisId, BandId, DefId, FillId, IndexId, InputId, KindId, MoveId, Names, PlaceId, SourceId,
    ThingId, VersionId,
};
use super::index::{IndexInfo, VersionInfo};
use super::key::{KeyVal, Span};
use super::kind::{Entry, KindInfo, KindSpec, Thing};
use super::ty::Ty;
use super::world::{Scenarios, World};
use crate::base::{Internal, Interner, Place, Refusal, Table, Warn};
use crate::num::Num;
use crate::syntax::{Ask, Axis, Expr, Fill, Header, Price, Sign, Source, Type};
use crate::time::Day;
use std::collections::{BTreeMap, BTreeSet};

/// What a name in the one namespace names, its sort (refcore's `Reality.names` kinds); a
/// `Name` holds it with its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameKind {
    /// A unit or currency.
    Unit,
    /// A kind.
    Kind,
    /// An index.
    Index,
    /// An axis.
    Axis,
    /// An input.
    Input,
    /// A def.
    Def,
    /// A band.
    Band,
    /// A place.
    Place,
    /// A fill.
    Fill,
    /// A move.
    Move,
    /// A version.
    Version,
    /// An ask.
    Ask,
    /// A source.
    Source,
    /// `cut`.
    Cut,
}

/// How a price input values a place (refcore's `pn._mode`, `pn._cur`, `pn._per`). L §5.10.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priced {
    /// `num CUR` or `money CUR`: the value of the place's one position, in CUR.
    Whole(String),
    /// `num CUR/u`: a price in CUR per unit u of its positions, (CUR, u).
    PerUnit(String, String),
}

/// The id a price written as a reference names, `px` or `px[k]` (L §2.3 `price`).
#[must_use]
pub fn price_id(input: &Expr) -> Option<&str> {
    if let Expr::Name(id) = input {
        return Some(id);
    }
    if let Expr::Post(base, _) = input
        && let Expr::Name(id) = base.as_ref()
    {
        return Some(id);
    }
    None
}

/// The inputs the language declares (refcore's `BUILTIN_INPUTS`). L §5.19.
pub const BUILTIN_INPUTS: [&str; 3] = ["opening", "stated", "account_place"];

/// The indexes the language declares (refcore's `BUILTIN_INDEXES`). L §4.1.
pub const BUILTIN_INDEXES: [&str; 4] = ["days", "months", "years", "bank_account"];

/// The resolved reality (refcore's `Reality`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reality {
    /// The model's header.
    pub header: Header,
    /// S, the run's start.
    pub start: Day,
    /// E, the run's end.
    pub end: Day,
    /// The model's `on` date.
    pub on: Day,
    /// Every name's text, interned: the namespace's ids and the tables' slots (D2-3).
    pub symbols: Interner,
    /// The one namespace.
    pub names: Names,
    /// The sources.
    pub sources: Table<SourceId, Source>,
    /// The versions, in section order.
    pub versions: Table<VersionId, VersionInfo>,
    /// The declared units that aren't currencies.
    pub units: BTreeSet<String>,
    /// The currencies, with their minor units.
    pub currencies: BTreeMap<String, Num>,
    /// The keys of kinds of unit, with their kind.
    pub unit_kind_keys: BTreeMap<String, String>,
    /// The kinds.
    pub kinds: Table<KindId, KindInfo>,
    /// The things.
    pub things: Table<ThingId, Thing>,
    /// The indexes.
    pub indexes: Table<IndexId, IndexInfo>,
    /// The inputs.
    pub inputs: Table<InputId, InputInfo>,
    /// The defs.
    pub defs: Table<DefId, DefInfo>,
    /// The bands.
    pub bands: Table<BandId, BandInfo>,
    /// The places.
    pub places: Table<PlaceId, PlaceInfo>,
    /// The moves.
    pub moves: Table<MoveId, MoveInfo>,
    /// The asks, in section order.
    pub asks: Vec<Ask>,
    /// The warnings load decided.
    pub warnings: Vec<(Warn, String)>,
    /// The item graph and the classes and strata load decides from it.
    pub graph: Graph,
    /// The scenarios this reality loads with the base: none for a base (L §6.4).
    pub scenarios: Scenarios,
    /// The axes, each with its options in the order written (L §5.5).
    pub axes: Table<AxisId, Axis>,
    /// The fills, as written (L §5.12).
    pub fills: Table<FillId, Fill>,
    /// The worlds an ask may name: the model, this reality's scenarios and the run's
    /// (refcore's `all_worlds`). L §6.4.
    pub all_worlds: BTreeSet<String>,
    /// For the base of a run with scenarios, each section id a scenario's file declares, with
    /// the first such scenario by id (refcore's `declared_ids`): the base names none of them
    /// (L §6.4, issue #75). Empty for any other load.
    pub elsewhere: BTreeMap<String, String>,
    /// The asks refused alone, each with its refusal: one that names a world the run doesn't
    /// load (refcore's `asks_alone`). L §6.4, §8.
    pub asks_alone: BTreeMap<String, Refusal>,
    /// Each ask whose line can hold a key literal no position resolves (bare, chosen by an
    /// `if`, a list's item), with the ask's line, where a value holding one meets
    /// `LOAD-KEY ambiguous key <k>` as it is evaluated (refcore's `e_name`, its `keylit` arm;
    /// pick P39). L §2.5, §8.3.
    pub asks_unresolved: BTreeMap<String, Place>,
    /// Each key list's binding site (file, line, token) with the index its key positions
    /// fixed: its keys are that index's (refcore's `b._var.fixed`). L §2.5.
    pub list_keys: BTreeMap<(String, u32, usize), String>,
    /// The sites (file, line, token) of the kind tests whose narrowing a field read uses: the
    /// run evaluates them first in their conjunction (refcore's `narrow_mark`). L §5.7.
    pub narrow_tests: BTreeSet<crate::syntax::SiteKey>,
    /// The rows and movements stated `same`, in the order written (refcore's `same_rows` and
    /// `same_moves`): read together at run. L §6.1, §5.19.
    pub sames: Vec<SamePair>,
    /// A joint model's members, as the run may ask them (refcore's `joint_members`), or `None`
    /// for a model that is no joint model. L §6.1; E §11.
    pub joint: Option<Joint>,
}

/// What a run knows of a joint model's members: each by its alias, the asks it may be asked
/// as `a.q` (refcore's `joint_members`, read by `knows_ask`). L §6.1; E §11.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Joint {
    /// Each member by its alias.
    pub members: BTreeMap<String, MemberAsks>,
}

/// A member's asks: its own (those its base answers, and those its scenario files declare),
/// and, for a member that is itself a joint model, its members' (`a.b.q`). L §6.1; E §11.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemberAsks {
    /// The asks it may be asked by their own ids.
    pub asks: BTreeSet<String>,
    /// The asks its own namespace holds, refused alone or not (refcore's `lookup` on the
    /// member finds them); its scenario files' asks are no names of it.
    pub named: BTreeSet<String>,
    /// Its own members, when it is a joint model.
    pub members: BTreeMap<String, MemberAsks>,
}

impl Joint {
    /// Whether `a.q` names an ask member `a`'s namespace holds, so that reading or calling it
    /// is no value (refcore's `member_read` and `prefixed_call`, their ask arm). L §6.1, §5.18.
    #[must_use]
    pub fn names_ask(&self, alias: &str, id: &str) -> bool {
        self.members
            .get(alias)
            .is_some_and(|member| member.named.contains(id))
    }

    /// Whether `id` names a member's ask, `a.q` or `a.b.q` (refcore's `knows_ask`, its member
    /// arm). E §11.
    #[must_use]
    pub fn knows(&self, id: &str) -> bool {
        let mut members = &self.members;
        let mut rest = id;
        while let Some((alias, ask)) = rest.split_once('.') {
            let Some(member) = members.get(alias) else {
                return false;
            };
            if member.asks.contains(ask) {
                return true;
            }
            members = &member.members;
            rest = ask;
        }
        false
    }
}

/// Two rows or two movements stated `same` (refcore's `same_rows`, `same_moves`). L §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamePair {
    /// Two rows: each input's id and keys.
    Rows(String, Vec<KeyVal>, String, Vec<KeyVal>),
    /// Two movements: each instance's id, `m[k]` or an observed row's `x[id]`.
    Moves(String, String),
}

impl Reality {
    /// The id of the thing named `name`, which its table holds (the caller pushed or found
    /// it).
    ///
    /// # Errors
    ///
    /// `Internal` where the name was never interned.
    pub fn thing_id(&self, name: &str) -> Result<ThingId, Internal> {
        super::ids::thing_id(&self.symbols, name)
    }

    /// The kind `id`.
    #[must_use]
    pub fn kind(&self, id: &str) -> Option<&KindInfo> {
        self.kinds.get(id)
    }

    /// The kind and its parents, nearest first.
    #[must_use]
    pub fn chain(&self, id: &str) -> Vec<&KindInfo> {
        let mut out = Vec::new();
        let mut next = self.kinds.get(id);
        while let Some(kind) = next {
            if out.iter().any(|k: &&KindInfo| k.id == kind.id) {
                break;
            }
            out.push(kind);
            next = kind.parent.as_deref().and_then(|p| self.kinds.get(p));
        }
        out
    }

    /// The kind and its subkinds, all the way down (refcore's `KindInfo.subtree`).
    #[must_use]
    pub fn subtree(&self, id: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut stack = vec![id.to_owned()];
        while let Some(next) = stack.pop() {
            if out.insert(next.clone())
                && let Some(kind) = self.kinds.get(&next)
            {
                stack.extend(kind.children.iter().cloned());
            }
        }
        out
    }

    /// Whether the thing `name` is ever of kind `kind` or a subkind (refcore's `thing_in`).
    #[must_use]
    pub fn thing_in(&self, name: &str, kind: &str) -> bool {
        let Some(thing) = self.things.get(name) else {
            return false;
        };
        let sub = self.subtree(kind);
        thing.entries.iter().any(|e| sub.contains(&e.kind))
    }

    /// The kind whose parts clauses a kind keeps: itself or the nearest parent that states
    /// them (refcore's `parts_kind`). L §3.6, §5.7.
    #[must_use]
    pub fn parts_kind(&self, id: &str) -> Option<&KindInfo> {
        self.chain(id).into_iter().find(|k| !k.parts.is_empty())
    }

    /// The parts kind of a place thing's base kind; none for a place that keeps no parts
    /// (refcore's `thing_parts`). L §3.6.
    #[must_use]
    pub fn thing_parts(&self, thing: &Thing) -> Option<&KindInfo> {
        self.parts_kind(thing.base.as_deref()?)
    }

    /// A place kind's unit, its own or its nearest parent's (refcore's `kind_unit`).
    #[must_use]
    pub fn kind_unit(&self, id: &str) -> Option<&str> {
        self.chain(id).into_iter().find_map(|k| k.unit.as_deref())
    }

    /// A place kind's sign rule (refcore's `kind_sign`).
    #[must_use]
    pub fn kind_sign(&self, id: &str) -> Option<Sign> {
        self.chain(id).into_iter().find_map(|k| k.sign)
    }

    /// Whether a place kind lies outside (refcore's `kind_outside`).
    #[must_use]
    pub fn kind_outside(&self, id: &str) -> bool {
        self.chain(id).into_iter().any(|k| k.outside)
    }

    /// A place kind's price input (refcore's `kind_priced`).
    #[must_use]
    pub fn kind_priced(&self, id: &str) -> Option<&Price> {
        self.chain(id).into_iter().find_map(|k| k.priced.as_ref())
    }

    /// An event kind's legs, its parents' first: (name, unit, the kind that declares it)
    /// (refcore's `kind_legs`). L §5.7.
    #[must_use]
    pub fn kind_legs(&self, id: &str) -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        for kind in self.chain(id).into_iter().rev() {
            for (name, unit) in &kind.legs {
                out.push((name.clone(), unit.clone(), kind.id.clone()));
            }
        }
        out
    }

    /// An event kind's attributes, its parents' first: (name, written type; none for the
    /// prelude's) (refcore's `kind_attrs`). L §5.7.
    #[must_use]
    pub fn kind_attrs(&self, id: &str) -> Vec<(String, Option<Type>)> {
        let mut out = Vec::new();
        for kind in self.chain(id).into_iter().rev() {
            out.extend(kind.attrs.iter().cloned());
        }
        out
    }

    /// Whether a thing is a place (refcore's `is_place_thing`).
    #[must_use]
    pub fn is_place(&self, thing: &Thing) -> bool {
        thing
            .base
            .as_deref()
            .and_then(|b| self.kinds.get(b))
            .is_some_and(|k| k.spec == KindSpec::Place)
    }

    /// A place's unit: its kind's, or the one it states for a kind of unit `one`; `any`
    /// (refcore's `place_unit`). L §5.7.
    #[must_use]
    pub fn place_unit(&self, thing: &Thing) -> Option<String> {
        let base = thing.base.as_deref()?;
        let stated = thing
            .entries
            .iter()
            .find(|e| e.kind == base && e.unit.is_some())
            .and_then(|e| e.unit.clone());
        if stated.is_some() {
            return stated;
        }
        match self.kind_unit(base) {
            Some("one") | None => None,
            Some(unit) => Some(unit.to_owned()),
        }
    }

    /// How a price input of type `ty` values a place: `num CUR` (or `money CUR`) whole, `num
    /// CUR/u` per unit u, else not a price (refcore's `m_resolve_price`, its type test).
    /// L §5.10.
    #[must_use]
    pub fn price_mode(&self, ty: Option<&Ty>) -> Option<Priced> {
        let ty = ty?;
        let terms: Vec<(String, i64)> = if let Ty::Money(cur) = ty {
            vec![(cur.clone(), 1)]
        } else if let Ty::Num(unit, _) = ty {
            unit.terms().to_vec()
        } else {
            return None;
        };
        let cur = terms
            .iter()
            .find(|(base, exp)| *exp == 1 && self.currencies.contains_key(base));
        let per = terms.iter().find(|(_, exp)| *exp == -1);
        match (terms.as_slice(), cur, per) {
            ([_], Some((cur, _)), _) => Some(Priced::Whole(cur.clone())),
            ([_, _], Some((cur, _)), Some((per, _))) => {
                Some(Priced::PerUnit(cur.clone(), per.clone()))
            }
            _ => None,
        }
    }

    /// The input or def a price clause names, and how it values its place, from its declared
    /// type (refcore's `pn._r`, `pn._mode`). L §5.10.
    #[must_use]
    pub fn clause_mode(&self, price: &Price) -> Option<(String, Priced)> {
        let id = price_id(&price.input)?;
        let kind = self.names.kind(id)?;
        let ty = if kind == NameKind::Input {
            self.inputs.get(id)?.ty.clone()
        } else if kind == NameKind::Def {
            Some(self.defs.get(id)?.ty.clone())
        } else {
            return None;
        };
        Some((id.to_owned(), self.price_mode(ty.as_ref())?))
    }

    /// The domain of an input's or a def's first key, a price's key. L §5.8, §5.10.
    #[must_use]
    pub fn first_domain(&self, id: &str) -> Option<Domain> {
        if let Some(input) = self.inputs.get(id) {
            return input.binds.first().map(|b| b.domain.clone());
        }
        self.defs
            .get(id)
            .and_then(|def| def.binds.first())
            .map(|b| b.domain.clone())
    }

    /// Whether a price keyed by `id` is keyed by units: the currencies or a kind of unit, a
    /// subkind's root included (refcore's `_unit_keyed`). L §5.10 (`by p` keyed by unit).
    #[must_use]
    pub fn unit_keyed(&self, id: &str) -> bool {
        let Some(Domain::Kind(kind)) = self.first_domain(id) else {
            return false;
        };
        self.kinds.get(&kind).is_none()
            || self
                .chain(&kind)
                .iter()
                .any(|k| k.id == "currencies" || k.spec == KindSpec::Unit)
    }

    /// The price a place is valued by: on its entry or `place` section, else on its kind
    /// (refcore's `price_of`). L §5.10.
    #[must_use]
    pub fn price_of(&self, thing: &Thing) -> Option<Price> {
        let base = thing.base.as_deref()?;
        let own = thing
            .entries
            .iter()
            .rev()
            .find(|e| e.kind == base && e.priced.is_some())
            .and_then(|e| e.priced.clone());
        own.or_else(|| self.kind_priced(base).cloned())
    }

    /// Whether a read in `world` sees `entry` of `thing`: a version's entry where its scenario
    /// is seen, a thing's own entry where the file declaring its section is (refcore's
    /// `in_world` of the entry's version, `item_in_world`). L §6.4.
    #[must_use]
    pub fn entry_seen(&self, thing: &Thing, entry: &Entry, world: World) -> bool {
        let section = entry.version.as_ref().unwrap_or_else(|| own_section(thing));
        self.scenarios.sees(world, Some(section))
    }

    /// The span over which `thing` is of `kind` under cut `cut` in world `world`: its latest
    /// entry the cut and the world see, a `replace` line winning (refcore's
    /// `ThingInfo.span_in`; a keyed place's is the overlap of its section's span and its key
    /// things', `PlaceThing.span_in`). L §5.7, §6.4.
    #[must_use]
    pub fn span_in(&self, thing: &Thing, kind: &str, (cut, world): (Day, World)) -> Option<Span> {
        if let Some(keys) = &thing.place {
            return self.place_span(thing, keys.key_things.as_slice(), kind, (cut, world));
        }
        let mut best: Option<&Entry> = None;
        let mut best_known: Option<Day> = None;
        let seen = thing
            .entries
            .iter()
            .filter(|e| e.kind == kind && self.entry_seen(thing, e, world));
        for entry in seen {
            let known = match &entry.version {
                None => None,
                Some(v) => match self.versions.get(v) {
                    Some(version) if version.known <= cut => Some(version.known),
                    Some(_) | None => continue,
                },
            };
            let wins = best.is_none_or(|held| {
                entry.replace && (held.version.is_none() || !held.replace || known >= best_known)
            });
            if wins {
                best = Some(entry);
                best_known = known;
            }
        }
        best.map(|entry| entry.span.clone())
    }

    /// A keyed place's span in world `world`: its section's, cut to its key things'
    /// lifespans.
    fn place_span(
        &self,
        thing: &Thing,
        key_things: &[String],
        kind: &str,
        (cut, world): (Day, World),
    ) -> Option<Span> {
        let own = thing
            .entries
            .first()
            .filter(|e| e.kind == kind && self.entry_seen(thing, e, world))?;
        let mut start = own.span.start();
        let mut end = own.span.end();
        for name in key_things {
            let key = self.things.get(name)?;
            let span = self.span_in(key, key.base.as_deref()?, (cut, world))?;
            if span.is_always() {
                continue;
            }
            if let Some(s) = span.start() {
                start = Some(start.map_or(s, |held| held.max(s)));
            }
            if let Some(e) = span.end() {
                end = Some(end.map_or(e, |held| held.min(e)));
            }
        }
        if let (Some(s), Some(e)) = (start, end)
            && e <= s
        {
            return Some(Span::between(Some(s), Some(s)));
        }
        Some(Span::between(start, end))
    }

    /// The moves world `world` holds, by id: a move a scenario's file declares exists only in
    /// that scenario and those over it (refcore's `m_plan_moves`, `item_in_world`). L §6.4.
    #[must_use]
    pub fn moves_in(&self, world: World) -> Vec<&MoveInfo> {
        let mut out: Vec<&MoveInfo> = self
            .moves
            .entries()
            .map(|(_, mv)| mv)
            .filter(|mv| self.scenarios.sees(world, Some(&mv.id)))
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    /// A thing's lifespan in its base kind under cut `cut` in world `world` (refcore's
    /// `lifespan`). L §5.7, §6.4.
    #[must_use]
    pub fn lifespan(&self, thing: &Thing, seen: (Day, World)) -> Option<Span> {
        self.span_in(thing, thing.base.as_deref()?, seen)
    }

    /// The sections of a scenario that state thing `thing` in world `world`: its own section
    /// or the versions of its entries, and those of a keyed place's key things, sorted (L §5.7,
    /// a lifespan is a fact; §6.4). E §9 item 3.
    #[must_use]
    pub fn thing_scenario_sections(&self, thing: &Thing, world: World) -> BTreeSet<String> {
        let mut holders = vec![thing];
        if let Some(keys) = &thing.place {
            holders.extend(keys.key_things.iter().filter_map(|k| self.things.get(k)));
        }
        let mut out = BTreeSet::new();
        for held in holders {
            let own = own_section(held);
            if self.scenarios.of(own).is_some() && self.scenarios.sees(world, Some(own)) {
                out.insert(own.clone());
            }
            let versions = held.entries.iter().filter_map(|e| e.version.as_ref());
            out.extend(
                versions
                    .filter(|v| {
                        self.scenarios.of(v).is_some() && self.scenarios.sees(world, Some(v))
                    })
                    .cloned(),
            );
        }
        out
    }
}

/// The section that declares a thing's own entries: its place's, for a place's thing, else
/// its own name's (refcore's `item_in_world`). L §6.4.
fn own_section(thing: &Thing) -> &String {
    match &thing.place {
        Some(p) => &p.place,
        None => &thing.name,
    }
}
