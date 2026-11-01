//! The reality as load leaves it: the namespace, the kinds and things, the indexes, the versions and the declarations, with the questions asked of them.
//! Spec: E §3.1 (steps 2, 4, 9); L §2.5 (one namespace), §5.7 (kinds: units, signs, legs,
//! attributes by inheritance; lifespans under a cut), §5.10 (a place's unit and price).
//! Twin: engine/tools/refcore/model.py (`Reality`, `BUILTIN_INPUTS`, `BUILTIN_INDEXES`,
//! `ThingInfo.span_in`, `kind_unit` to `lifespan`).
//! Invariants: every name is in the namespace once; a kind's chain of parents ends; a thing's
//! lifespan under a cut is its latest entry the cut sees, a `replace` line winning.

use super::decl::{BandInfo, DefInfo, InputInfo, MoveInfo, PlaceInfo};
use super::index::{IndexInfo, VersionInfo};
use super::key::Span;
use super::kind::{Entry, KindInfo, KindSpec, Thing};
use crate::base::{Table, Warn};
use crate::num::Num;
use crate::syntax::{Ask, Header, Price, Sign, Source};
use crate::time::Day;
use std::collections::{BTreeMap, BTreeSet};

/// What a name in the one namespace names (refcore's `Reality.names` kinds).
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
    /// The one namespace.
    pub names: BTreeMap<String, NameKind>,
    /// The sources.
    pub sources: Table<Source>,
    /// The versions, in section order.
    pub versions: Table<VersionInfo>,
    /// The declared units that aren't currencies.
    pub units: BTreeSet<String>,
    /// The currencies, with their minor units.
    pub currencies: BTreeMap<String, Num>,
    /// The keys of kinds of unit, with their kind.
    pub unit_kind_keys: BTreeMap<String, String>,
    /// The kinds.
    pub kinds: Table<KindInfo>,
    /// The things.
    pub things: Table<Thing>,
    /// The indexes.
    pub indexes: Table<IndexInfo>,
    /// The inputs.
    pub inputs: Table<InputInfo>,
    /// The defs.
    pub defs: Table<DefInfo>,
    /// The bands.
    pub bands: Table<BandInfo>,
    /// The places.
    pub places: Table<PlaceInfo>,
    /// The moves.
    pub moves: Table<MoveInfo>,
    /// The asks, in section order.
    pub asks: Vec<Ask>,
    /// The warnings load decided.
    pub warnings: Vec<(Warn, String)>,
}

impl Reality {
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

    /// The span over which `thing` is of `kind` under cut `cut`: its latest entry the cut
    /// sees, a `replace` line winning (refcore's `ThingInfo.span_in`; a keyed place's is the
    /// overlap of its section's span and its key things', `PlaceThing.span_in`). L §5.7.
    #[must_use]
    pub fn span_in(&self, thing: &Thing, kind: &str, cut: Day) -> Option<Span> {
        if let Some(keys) = &thing.place {
            return self.place_span(thing, keys.key_things.as_slice(), kind, cut);
        }
        let mut best: Option<&Entry> = None;
        let mut best_known: Option<Day> = None;
        for entry in thing.entries.iter().filter(|e| e.kind == kind) {
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

    /// A keyed place's span: its section's, cut to its key things' lifespans.
    fn place_span(
        &self,
        thing: &Thing,
        key_things: &[String],
        kind: &str,
        cut: Day,
    ) -> Option<Span> {
        let own = thing.entries.first().filter(|e| e.kind == kind)?;
        let mut start = own.span.start();
        let mut end = own.span.end();
        for name in key_things {
            let key = self.things.get(name)?;
            let span = self.span_in(key, key.base.as_deref()?, cut)?;
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

    /// A thing's lifespan in its base kind under cut `cut` (refcore's `lifespan`). L §5.7.
    #[must_use]
    pub fn lifespan(&self, thing: &Thing, cut: Day) -> Option<Span> {
        self.span_in(thing, thing.base.as_deref()?, cut)
    }
}
