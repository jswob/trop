//! Kinds and things: the prelude's kinds and the declared ones, each thing with its dated entries.
//! Spec: L §5.4 (currencies), §5.7 (kinds, subkinds, things, lifespans, the prelude), §5.10
//! (places and their units, signs, prices).
//! Twin: engine/tools/refcore/model.py (`KindInfo`, `Entry`, `ThingInfo`, `PlaceThing`,
//! `PRELUDE_PLACE`, `PRELUDE_KINDS`); loader.py (`build_kinds`, the prelude's event kinds).
//! Invariants: every kind but a root has a parent that exists; a subkind's spec is its root's;
//! a thing's base is the kind it entered that isn't plain, or its earliest plain one.

use super::key::{KeyVal, Span};
use crate::syntax::{Certainty, Price, Sign, Type};

/// What a kind's things are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindSpec {
    /// Plain things: people, clients.
    Plain,
    /// Places money or units sit in.
    Place,
    /// Units (`currencies` and kinds of unit).
    Unit,
    /// Events: the kinds of movements.
    Event,
}

/// A kind (refcore's `KindInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindInfo {
    /// Its id.
    pub id: String,
    /// What its things are; a subkind's is its root's.
    pub spec: KindSpec,
    /// The kind it is a subkind of.
    pub parent: Option<String>,
    /// Its subkinds, sorted by id.
    pub children: Vec<String>,
    /// An event kind's certainty, its own or its parent's.
    pub certainty: Option<Certainty>,
    /// A place kind's unit: a currency, a unit, a kind of unit, `any` or `one`.
    pub unit: Option<String>,
    /// A place kind's sign rule.
    pub sign: Option<Sign>,
    /// A place kind's price input.
    pub priced: Option<Price>,
    /// Whether its places lie outside the household.
    pub outside: bool,
    /// An event kind's own legs: (name, unit).
    pub legs: Vec<(String, String)>,
    /// An event kind's own attributes: (name, written type; none for the prelude's).
    pub attrs: Vec<(String, Option<Type>)>,
    /// Whether the prelude declares it.
    pub prelude: bool,
    /// The line of its `kind` section.
    pub line: Option<u32>,
}

impl KindInfo {
    /// A kind with nothing stated but its id, spec and parent.
    #[must_use]
    pub fn new(id: &str, spec: KindSpec, parent: Option<&str>) -> KindInfo {
        KindInfo {
            id: id.to_owned(),
            spec,
            parent: parent.map(str::to_owned),
            children: Vec::new(),
            certainty: None,
            unit: None,
            sign: None,
            priced: None,
            outside: false,
            legs: Vec::new(),
            attrs: Vec::new(),
            prelude: true,
            line: None,
        }
    }
}

/// The prelude's kinds, in refcore's order (`PRELUDE_KINDS`). L §5.7.
pub const PRELUDE_KINDS: [&str; 14] = [
    "places",
    "accounts",
    "cash",
    "holdings",
    "assets",
    "debts",
    "claims",
    "outside",
    "currencies",
    "events",
    "transfers",
    "adjustments",
    "pads",
    "parties",
];

/// The prelude's place kinds: (id, unit, sign, outside) (refcore's `PRELUDE_PLACE`). L §5.7.
const PRELUDE_PLACES: [(&str, &str, Sign, bool); 7] = [
    ("accounts", "one", Sign::Signed, false),
    ("cash", "one", Sign::Signed, false),
    ("holdings", "one", Sign::Min0, false),
    ("assets", "one", Sign::Min0, false),
    ("debts", "one", Sign::Max0, false),
    ("claims", "one", Sign::Min0, false),
    ("outside", "any", Sign::Signed, true),
];

/// The prelude's kinds, built (refcore's `build_kinds`, its prelude part). L §5.7.
#[must_use]
pub fn prelude() -> Vec<KindInfo> {
    let mut kinds = vec![
        KindInfo::new("places", KindSpec::Place, None),
        KindInfo::new("events", KindSpec::Event, None),
        KindInfo::new("currencies", KindSpec::Unit, None),
        KindInfo::new("parties", KindSpec::Plain, None),
    ];
    for (id, unit, sign, outside) in PRELUDE_PLACES {
        let mut kind = KindInfo::new(id, KindSpec::Place, Some("places"));
        kind.unit = Some(unit.to_owned());
        kind.sign = Some(sign);
        kind.outside = outside;
        kinds.push(kind);
    }
    let mut transfers = event_kind("transfers");
    transfers.legs = vec![("cash".to_owned(), "any".to_owned())];
    let mut adjustments = event_kind("adjustments");
    adjustments.attrs = vec![("adjusts".to_owned(), None), ("change".to_owned(), None)];
    let mut pads = event_kind("pads");
    pads.legs = vec![("cash".to_owned(), "any".to_owned())];
    kinds.extend([transfers, adjustments, pads]);
    kinds
}

/// A certain event kind of the prelude, with no leg or attribute yet.
fn event_kind(id: &str) -> KindInfo {
    let mut kind = KindInfo::new(id, KindSpec::Event, Some("events"));
    kind.certainty = Some(Certainty::Certain);
    kind
}

/// A thing's entry into a kind: its lifespan or membership, from a version or a `place`
/// section (refcore's `Entry`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The kind it enters.
    pub kind: String,
    /// Its lifespan in that kind.
    pub span: Span,
    /// The version that states it; none for a `place` section or a currency.
    pub version: Option<String>,
    /// Whether it is a `replace` line.
    pub replace: bool,
    /// The unit it states (a kind of unit `one`).
    pub unit: Option<String>,
    /// The price input it states.
    pub priced: Option<Price>,
    /// Its line.
    pub line: Option<u32>,
}

/// A place a keyed `place` section declares at one binding tuple (refcore's `PlaceThing`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceKeys {
    /// The `place` section.
    pub place: String,
    /// Its key values.
    pub keys: Vec<KeyVal>,
    /// The things among its keys, whose lifespans bound its own.
    pub key_things: Vec<String>,
    /// The keys of open or list indexes a version added: (index, key).
    pub key_indexes: Vec<(String, KeyVal)>,
}

/// A thing: a key of one or more kinds (refcore's `ThingInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thing {
    /// Its name, `konto` or `wcash[ike]`.
    pub name: String,
    /// Its base kind.
    pub base: Option<String>,
    /// Its entries, any kind.
    pub entries: Vec<Entry>,
    /// For a keyed place, its section and keys.
    pub place: Option<PlaceKeys>,
}

impl Thing {
    /// A thing with no entry yet.
    #[must_use]
    pub fn new(name: &str) -> Thing {
        Thing {
            name: name.to_owned(),
            base: None,
            entries: Vec::new(),
            place: None,
        }
    }
}
