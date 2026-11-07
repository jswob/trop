//! The typed ids and the one namespace that gives them: each name with its sort and its id.
//! Spec: L §2.5 (one namespace across a reality's files: an id names one thing); BUILDER §3
//! (typed ids into `Table<I, T>`, D2-4; interned names, D2-3); E §9 (`ENGINE-INVARIANT`).
//! Twin: engine/tools/refcore/model.py (`Reality.names`, which maps an id to its kind).
//! Invariants: an id is made only here, from the symbol of a name the namespace declares (or,
//! for a thing, a name its table holds), so an id of a sort is a name of that sort; ids have
//! no `Ord` (R-47): the namespace iterates in its names' text order.

use super::reality::NameKind;
use crate::base::{Internal, Interner, Symbol};
use std::collections::BTreeMap;
use std::collections::btree_map::Iter;

/// A source's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(Symbol);

/// A version's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionId(Symbol);

/// A kind's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindId(Symbol);

/// A thing's id, its name: a key of a kind (L §5.7), or a place's thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThingId(Symbol);

/// An index's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexId(Symbol);

/// An input's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputId(Symbol);

/// A def's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefId(Symbol);

/// A band's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandId(Symbol);

/// A place's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceId(Symbol);

/// A move's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveId(Symbol);

/// An axis's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisId(Symbol);

/// A fill's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillId(Symbol);

impl From<SourceId> for Symbol {
    fn from(id: SourceId) -> Symbol {
        id.0
    }
}

impl From<VersionId> for Symbol {
    fn from(id: VersionId) -> Symbol {
        id.0
    }
}

impl From<KindId> for Symbol {
    fn from(id: KindId) -> Symbol {
        id.0
    }
}

impl From<ThingId> for Symbol {
    fn from(id: ThingId) -> Symbol {
        id.0
    }
}

impl From<IndexId> for Symbol {
    fn from(id: IndexId) -> Symbol {
        id.0
    }
}

impl From<InputId> for Symbol {
    fn from(id: InputId) -> Symbol {
        id.0
    }
}

impl From<DefId> for Symbol {
    fn from(id: DefId) -> Symbol {
        id.0
    }
}

impl From<BandId> for Symbol {
    fn from(id: BandId) -> Symbol {
        id.0
    }
}

impl From<PlaceId> for Symbol {
    fn from(id: PlaceId) -> Symbol {
        id.0
    }
}

impl From<MoveId> for Symbol {
    fn from(id: MoveId) -> Symbol {
        id.0
    }
}

impl From<AxisId> for Symbol {
    fn from(id: AxisId) -> Symbol {
        id.0
    }
}

impl From<FillId> for Symbol {
    fn from(id: FillId) -> Symbol {
        id.0
    }
}

/// What a name in the one namespace names, with its id where a table holds it (refcore's
/// `Reality.names` kinds). L §2.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Name {
    /// A unit or currency.
    Unit,
    /// A kind.
    Kind(KindId),
    /// An index.
    Index(IndexId),
    /// An axis.
    Axis(AxisId),
    /// An input.
    Input(InputId),
    /// A def.
    Def(DefId),
    /// A band.
    Band(BandId),
    /// A place.
    Place(PlaceId),
    /// A fill.
    Fill(FillId),
    /// A move.
    Move(MoveId),
    /// A version.
    Version(VersionId),
    /// An ask.
    Ask,
    /// A source.
    Source(SourceId),
    /// `cut`.
    Cut,
}

impl Name {
    /// The name of sort `kind` whose text is interned as `symbol`.
    fn of(kind: NameKind, symbol: Symbol) -> Name {
        match kind {
            NameKind::Unit => Name::Unit,
            NameKind::Kind => Name::Kind(KindId(symbol)),
            NameKind::Index => Name::Index(IndexId(symbol)),
            NameKind::Axis => Name::Axis(AxisId(symbol)),
            NameKind::Input => Name::Input(InputId(symbol)),
            NameKind::Def => Name::Def(DefId(symbol)),
            NameKind::Band => Name::Band(BandId(symbol)),
            NameKind::Place => Name::Place(PlaceId(symbol)),
            NameKind::Fill => Name::Fill(FillId(symbol)),
            NameKind::Move => Name::Move(MoveId(symbol)),
            NameKind::Version => Name::Version(VersionId(symbol)),
            NameKind::Ask => Name::Ask,
            NameKind::Source => Name::Source(SourceId(symbol)),
            NameKind::Cut => Name::Cut,
        }
    }

    /// Its sort.
    #[must_use]
    pub fn kind(self) -> NameKind {
        match self {
            Name::Unit => NameKind::Unit,
            Name::Kind(_) => NameKind::Kind,
            Name::Index(_) => NameKind::Index,
            Name::Axis(_) => NameKind::Axis,
            Name::Input(_) => NameKind::Input,
            Name::Def(_) => NameKind::Def,
            Name::Band(_) => NameKind::Band,
            Name::Place(_) => NameKind::Place,
            Name::Fill(_) => NameKind::Fill,
            Name::Move(_) => NameKind::Move,
            Name::Version(_) => NameKind::Version,
            Name::Ask => NameKind::Ask,
            Name::Source(_) => NameKind::Source,
            Name::Cut => NameKind::Cut,
        }
    }
}

/// The one namespace: every name a reality declares, with what it names (refcore's
/// `Reality.names`), in its names' text order. L §2.5.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Names {
    /// Each name's sort and id, by its text.
    by_text: BTreeMap<String, Name>,
}

impl Names {
    /// An empty namespace.
    #[must_use]
    pub fn new() -> Names {
        Names::default()
    }

    /// Names `text` as a name of sort `kind`, its text interned in `symbols`; a name declared
    /// again takes the later sort, as a map's insert does (load refuses a second declaration
    /// first, `LOAD-DUP`).
    ///
    /// # Errors
    ///
    /// `Internal` where `symbols` can't give the text a symbol.
    pub fn declare(
        &mut self,
        symbols: &mut Interner,
        text: &str,
        kind: NameKind,
    ) -> Result<(), Internal> {
        let symbol = symbols.intern(text)?;
        self.by_text.insert(text.to_owned(), Name::of(kind, symbol));
        Ok(())
    }

    /// What `text` names, with its id.
    #[must_use]
    pub fn get(&self, text: &str) -> Option<Name> {
        self.by_text.get(text).copied()
    }

    /// The sort of what `text` names.
    #[must_use]
    pub fn kind(&self, text: &str) -> Option<NameKind> {
        self.get(text).map(Name::kind)
    }

    /// Whether `text` is a name.
    #[must_use]
    pub fn contains_key(&self, text: &str) -> bool {
        self.by_text.contains_key(text)
    }

    /// Every name with what it names, in text order.
    pub fn entries(&self) -> Iter<'_, String, Name> {
        self.by_text.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{Name, Names};
    use crate::base::{Interner, Table};
    use crate::model::NameKind;

    #[test]
    fn a_declared_name_finds_its_item() {
        let mut symbols = Interner::new();
        let mut names = Names::new();
        names
            .declare(&mut symbols, "x", NameKind::Input)
            .expect("x");
        names
            .declare(&mut symbols, "cut", NameKind::Cut)
            .expect("cut");
        let mut inputs = Table::new();
        assert_eq!(inputs.push(&mut symbols, "x", 3), Ok(true), "x");
        assert_eq!(names.kind("x"), Some(NameKind::Input), "x is an input");
        assert_eq!(names.get("cut"), Some(Name::Cut), "cut");
        assert_eq!(names.kind("y"), None, "y is no name");
        let Some(Name::Input(x)) = names.get("x") else {
            panic!("x is an input");
        };
        assert_eq!(inputs.at(x), Ok(&3), "x by its id");
    }
}
