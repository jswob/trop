//! The one id-keyed store: items in the order they were added, found by their name, never by indexing.
//! Spec: BUILDER §3 (`Table<I, T>`: `push`, `get`, `iter`, no `Index` impl; D2-4).
//! Twin: engine/tools/refcore/model.py (`Reality`'s dicts, which keep insertion order).
//! Invariants: each name is in the table once; iteration is in insertion order, which is
//! the reality's section order, as refcore's dicts iterate; there is no `Index` impl.

use std::collections::BTreeMap;

/// Items found by name, kept in the order they were added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table<T> {
    /// The items, in the order added.
    items: Vec<(String, T)>,
    /// Each name's position in `items`.
    by_name: BTreeMap<String, usize>,
}

impl<T> Default for Table<T> {
    fn default() -> Table<T> {
        Table {
            items: Vec::new(),
            by_name: BTreeMap::new(),
        }
    }
}

impl<T> Table<T> {
    /// An empty table.
    #[must_use]
    pub fn new() -> Table<T> {
        Table::default()
    }

    /// Adds `item` under `name`; `false`, and nothing added, when the name is taken.
    pub fn push(&mut self, name: &str, item: T) -> bool {
        if self.by_name.contains_key(name) {
            return false;
        }
        self.by_name.insert(name.to_owned(), self.items.len());
        self.items.push((name.to_owned(), item));
        true
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

#[cfg(test)]
mod tests {
    use super::Table;

    #[test]
    fn insertion_order_and_names() {
        let mut table = Table::new();
        assert!(table.push("b", 1), "b");
        assert!(table.push("a", 2), "a");
        assert!(!table.push("b", 3), "b is taken");
        assert_eq!(table.get("b"), Some(&1), "the first b stays");
        assert_eq!(table.names(), ["b", "a"], "in the order added");
        if let Some(a) = table.get_mut("a") {
            *a = 5;
        }
        assert_eq!(table.get("a"), Some(&5), "changed");
        assert_eq!(table.get("c"), None, "absent");
    }
}
