//! The interner: each distinct name's text once, as a small symbol that a typed id wraps.
//! Spec: BUILDER §3 (names and text keys go through one hand-written interner, D2-3; the
//! table of a typed id, D2-4); E §9 (a broken invariant refuses with `ENGINE-INVARIANT`).
//! Twin: none: refcore keys its dicts by the text itself (engine/tools/refcore/model.py).
//! Invariants: a text has one symbol and a symbol one text; a symbol's number is the order its
//! text was first interned, which nothing may order by: `Symbol` has no `Ord` (R-47), so an
//! order of names is their texts' own; past 2^32 texts is `Internal`, never a wrap.

use std::collections::BTreeMap;

/// A name's text, interned: the same text always gives the same symbol (D2-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol(u32);

impl Symbol {
    /// Its slot: the position of its text in the interner, and of its item in a table's slots.
    ///
    /// # Errors
    ///
    /// `Internal` where the number doesn't fit a position, which no platform the engine
    /// builds for has.
    pub fn slot(self) -> Result<usize, Internal> {
        usize::try_from(self.0).map_err(|_| Internal::new("a symbol's number is a position"))
    }
}

/// A broken invariant of the engine's own tables (an id that names nothing, more than 2^32
/// names): the run refuses with `ENGINE-INVARIANT`, never a crash (E §9; R-115).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Internal {
    /// What should have held, for the refusal's message.
    held: &'static str,
}

impl Internal {
    /// The invariant `held` is broken.
    #[must_use]
    pub fn new(held: &'static str) -> Internal {
        Internal { held }
    }

    /// The refusal's message: `unreachable: <what should have held>`.
    #[must_use]
    pub fn message(self) -> String {
        format!("unreachable: {}", self.held)
    }
}

/// Every name's text a reality holds, each once, with its symbol (D2-3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Interner {
    /// The texts, by their symbol's number.
    texts: Vec<String>,
    /// Each text's symbol.
    by_text: BTreeMap<String, Symbol>,
}

impl Interner {
    /// An interner holding no text.
    #[must_use]
    pub fn new() -> Interner {
        Interner::default()
    }

    /// The symbol of `text`, a new one when the text is new.
    ///
    /// # Errors
    ///
    /// `Internal` past 2^32 texts.
    pub fn intern(&mut self, text: &str) -> Result<Symbol, Internal> {
        if let Some(symbol) = self.by_text.get(text) {
            return Ok(*symbol);
        }
        let number = u32::try_from(self.texts.len())
            .map_err(|_| Internal::new("a reality holds at most 2^32 names"))?;
        let symbol = Symbol(number);
        self.texts.push(text.to_owned());
        self.by_text.insert(text.to_owned(), symbol);
        Ok(symbol)
    }

    /// The symbol of `text`, if it was interned.
    #[must_use]
    pub fn find(&self, text: &str) -> Option<Symbol> {
        self.by_text.get(text).copied()
    }

    /// The text of `symbol`.
    ///
    /// # Errors
    ///
    /// `Internal` for a symbol this interner didn't make.
    pub fn text(&self, symbol: Symbol) -> Result<&str, Internal> {
        self.texts
            .get(symbol.slot()?)
            .map(String::as_str)
            .ok_or(Internal::new("a symbol's text is in its interner"))
    }
}

#[cfg(test)]
mod tests {
    use super::Interner;

    #[test]
    fn one_symbol_per_text() {
        let mut symbols = Interner::new();
        let b = symbols.intern("b").expect("b");
        let a = symbols.intern("a").expect("a");
        assert_eq!(symbols.intern("b"), Ok(b), "b again is the same symbol");
        assert_ne!(a, b, "two texts, two symbols");
        assert_eq!(symbols.find("a"), Some(a), "found");
        assert_eq!(symbols.find("c"), None, "never interned");
        assert_eq!(symbols.text(b), Ok("b"), "its text");
        assert_eq!(symbols.text(a), Ok("a"), "its text");
        let other = Interner::new();
        assert!(other.text(a).is_err(), "a symbol another interner made");
    }
}
