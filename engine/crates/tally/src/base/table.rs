//! The id-keyed store: items in the order they were added, found by their typed id or by their name, never by indexing.
//! Spec: BUILDER §3 (`Table<I, T>`: an id-keyed store with a checked lookup, no `Index` impl;
//! D2-4), the interner (D2-3) giving each item's name its symbol; E §9 (a miss of an id the
//! namespace gives refuses with `ENGINE-INVARIANT`).
//! Twin: engine/tools/refcore/model.py (`Reality`'s dicts, which keep insertion order).
//! Invariants: each name is in the table once, under its interned symbol; iteration is in
//! insertion order, which is the reality's section order, as refcore's dicts iterate; an id
//! finds its item through a dense slot vector, never a map keyed by an id; there is no `Index`
//! impl. A name a user wrote, which may name nothing of the sort, is looked up by its text
//! (`get`); an id a namespace check proved is looked up by `at` (the coordinator's P10).

use super::symbol::{Internal, Interner, Symbol};
use core::marker::PhantomData;
use std::collections::BTreeMap;

/// Items found by their typed id `I` or by name, kept in the order they were added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table<I, T> {
    /// The items, in the order added.
    items: Vec<(String, T)>,
    /// Each name's position in `items`.
    by_name: BTreeMap<String, usize>,
    /// Each item's position in `items`, at its name's symbol's slot.
    by_symbol: Vec<Option<usize>>,
    /// The id that finds an item.
    id: PhantomData<I>,
}

impl<I, T> Default for Table<I, T> {
    fn default() -> Table<I, T> {
        Table {
            items: Vec::new(),
            by_name: BTreeMap::new(),
            by_symbol: Vec::new(),
            id: PhantomData,
        }
    }
}

impl<I: Into<Symbol>, T> Table<I, T> {
    /// An empty table.
    #[must_use]
    pub fn new() -> Table<I, T> {
        Table::default()
    }

    /// Adds `item` under `name`, interned in `symbols`; `false`, and nothing added, when the
    /// name is taken.
    ///
    /// # Errors
    ///
    /// `Internal` where `symbols` can't give the name a symbol, or a slot past memory.
    pub fn push(&mut self, symbols: &mut Interner, name: &str, item: T) -> Result<bool, Internal> {
        if self.by_name.contains_key(name) {
            return Ok(false);
        }
        let slot = symbols.intern(name)?.slot()?;
        let len = slot
            .checked_add(1)
            .ok_or(Internal::new("a table's slots fit memory"))?;
        if self.by_symbol.len() < len {
            self.by_symbol.resize(len, None);
        }
        let at = self.items.len();
        let cell = self
            .by_symbol
            .get_mut(slot)
            .ok_or(Internal::new("a table holds a slot for each of its names"))?;
        *cell = Some(at);
        self.by_name.insert(name.to_owned(), at);
        self.items.push((name.to_owned(), item));
        Ok(true)
    }

    /// The item of `id`, which a namespace check proved this table holds.
    ///
    /// # Errors
    ///
    /// `Internal` where it holds none: the namespace and its table disagree.
    pub fn at(&self, id: I) -> Result<&T, Internal> {
        let at = self.position(id)?;
        self.items
            .get(at)
            .map(|(_, item)| item)
            .ok_or(Internal::new(MISSING))
    }

    /// The item of `id`, to change; as [`Table::at`].
    ///
    /// # Errors
    ///
    /// As [`Table::at`].
    pub fn at_mut(&mut self, id: I) -> Result<&mut T, Internal> {
        let at = self.position(id)?;
        self.items
            .get_mut(at)
            .map(|(_, item)| item)
            .ok_or(Internal::new(MISSING))
    }

    /// The position in `items` of the item of `id`.
    fn position(&self, id: I) -> Result<usize, Internal> {
        let slot = id.into().slot()?;
        self.by_symbol
            .get(slot)
            .copied()
            .flatten()
            .ok_or(Internal::new(MISSING))
    }

    /// The item named `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&T> {
        let at = *self.by_name.get(name)?;
        self.items.get(at).map(|(_, item)| item)
    }

    /// The item named `name`, to change.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        let at = *self.by_name.get(name)?;
        self.items.get_mut(at).map(|(_, item)| item)
    }

    /// Whether an item is named `name`.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// The items with their names, in the order added.
    pub fn entries(&self) -> core::slice::Iter<'_, (String, T)> {
        self.items.iter()
    }

    /// The items with their names, in the order added, to change.
    pub fn entries_mut(&mut self) -> core::slice::IterMut<'_, (String, T)> {
        self.items.iter_mut()
    }

    /// The names, in the order added.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.items.iter().map(|(name, _)| name.clone()).collect()
    }

    /// How many items it holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether it holds none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// What a miss of `at` breaks.
const MISSING: &str = "an id the namespace gives names an item of its table";

#[cfg(test)]
mod tests {
    use super::{Interner, Symbol, Table};

    #[test]
    fn insertion_order_and_names() {
        let mut symbols = Interner::new();
        let mut table: Table<Symbol, i32> = Table::new();
        assert_eq!(table.push(&mut symbols, "b", 1), Ok(true), "b");
        assert_eq!(table.push(&mut symbols, "a", 2), Ok(true), "a");
        assert_eq!(table.push(&mut symbols, "b", 3), Ok(false), "b is taken");
        assert_eq!(table.get("b"), Some(&1), "the first b stays");
        assert_eq!(table.names(), ["b", "a"], "in the order added");
        if let Some(a) = table.get_mut("a") {
            *a = 5;
        }
        assert_eq!(table.get("a"), Some(&5), "changed");
        assert_eq!(table.get("c"), None, "absent");
    }

    #[test]
    fn an_id_finds_its_item_and_a_miss_is_internal() {
        let mut symbols = Interner::new();
        let other = symbols.intern("x").expect("x");
        let mut table: Table<Symbol, i32> = Table::new();
        assert_eq!(table.push(&mut symbols, "b", 1), Ok(true), "b");
        assert_eq!(table.push(&mut symbols, "a", 2), Ok(true), "a");
        let a = symbols.find("a").expect("a is interned");
        assert_eq!(table.at(a), Ok(&2), "a by its id");
        if let Ok(item) = table.at_mut(a) {
            *item = 7;
        }
        assert_eq!(table.get("a"), Some(&7), "changed by its id");
        assert!(table.at(other).is_err(), "x is interned, not in the table");
        let late = symbols.intern("late").expect("late");
        assert!(table.at(late).is_err(), "a symbol past the table's slots");
    }
}
