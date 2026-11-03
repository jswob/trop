//! Load's places: each `place` section's thing, or one per binding tuple, and a place as rows write it.
//! Spec: L §5.7 (lifespans have a start), §5.10 (places: their kind, unit, price, keyed places
//! and their lifespans), §5.3 (`covers`, `pad`, `anchor` name places).
//! Twin: engine/tools/refcore/loader.py (`build_places`, `check_unit_word`, `place_ref`,
//! `keyed_place`).
//! Invariants: a keyed place has one thing per tuple of its keys' values, named `p[k1, k2]` in
//! bind order; things of a kind are taken in name order; a place written in a row names a thing
//! whose base is a place kind.

use super::keys::{KeyLit, keyed_place_thing, lit_of_expr};
use super::spans::lspan_to_span;
use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{Domain, Entry, KeyVal, KindSpec, NameKind, PlaceKeys, Thing, is_currency_code};
use crate::syntax::{Expr, Key, Postfix, Section, Stop};

/// Which refusal a place written where none is names: `LOAD-COVERS` in a `covers` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Written {
    /// In a row, a leg or a gap role.
    Place,
    /// In a `covers` line.
    Covers,
}

/// A place's thing at one tuple of keys, before it is named.
struct Tuple {
    /// The key values, in bind order.
    keys: Vec<KeyVal>,
    /// The things among them.
    key_things: Vec<String>,
    /// The keys of open or list indexes a version added.
    key_indexes: Vec<(String, KeyVal)>,
}

impl Loader {
    /// Each `place` section's thing or things (refcore's `build_places`). L §5.10.
    pub(super) fn build_places(&mut self) -> Result<(), Stop> {
        for id in self.m.places.names() {
            let Some(place) = self.m.places.get(&id) else {
                continue;
            };
            let decl = place.decl.clone();
            let binds = place.binds.clone();
            let file = self.place_file(&id);
            let line = decl.line;
            let Some(kind) = self
                .m
                .kinds
                .get(&decl.kind)
                .filter(|_| self.m.names.get(&decl.kind) == Some(&NameKind::Kind))
            else {
                return Err(refuse(
                    Load::Name,
                    format!("unknown id {}", decl.kind),
                    &file,
                    line,
                ));
            };
            let kind_id = kind.id.clone();
            if kind.spec != KindSpec::Place {
                return Err(refuse(
                    Load::Kind,
                    format!("place {id} of {kind_id}, not a place kind"),
                    &file,
                    line,
                ));
            }
            if !kind.children.is_empty() {
                return Err(refuse(
                    Load::Leaf,
                    format!("{id} enters {kind_id}, which has subkinds"),
                    &file,
                    line,
                ));
            }
            let unit = self.m.kind_unit(&kind_id).unwrap_or("None").to_owned();
            if unit == "one" && decl.unit.is_none() {
                return Err(refuse(
                    Load::PlaceKind,
                    format!("{id} enters a kind of unit one without its unit"),
                    &file,
                    line,
                ));
            }
            if let Some(stated) = &decl.unit {
                self.check_unit_word(stated, &file, line)?;
                if unit != "one" {
                    return Err(refuse(
                        Load::Unit,
                        format!(
                            "place {id} states {stated}; its kind {kind_id} is of unit {unit}, not one"
                        ),
                        &file,
                        line,
                    ));
                }
            }
            let lives = lspan_to_span(&decl.lives, &file, line)?;
            if lives.start().is_none() && !lives.is_always() {
                return Err(refuse(
                    Load::Lifespan,
                    format!("{id}'s lifespan has no start"),
                    &file,
                    line,
                ));
            }
            let entry = Entry {
                kind: kind_id.clone(),
                span: lives.clone(),
                version: None,
                replace: false,
                unit: decl.unit.clone(),
                priced: decl.priced.clone(),
                line: Some(line),
            };
            let mut named = Vec::new();
            if binds.is_empty() {
                if self.m.things.contains(&id) {
                    return Err(refuse(
                        Load::Dup,
                        format!("{id} is declared twice"),
                        &file,
                        line,
                    ));
                }
                let mut thing = Thing::new(&id);
                thing.base = Some(kind_id.clone());
                thing.entries.push(entry);
                self.m.things.push(&id, thing);
                named.push((Vec::new(), id.clone()));
            } else {
                for tuple in self.place_tuples(&id, &binds, &file, line)? {
                    let texts: Vec<String> = tuple.keys.iter().map(KeyVal::text).collect();
                    let name = format!("{id}[{}]", texts.join(", "));
                    let mut thing = Thing::new(&name);
                    thing.base = Some(kind_id.clone());
                    thing.entries.push(entry.clone());
                    thing.place = Some(PlaceKeys {
                        place: id.clone(),
                        keys: tuple.keys.clone(),
                        key_things: tuple.key_things,
                        key_indexes: tuple.key_indexes,
                    });
                    self.m.things.push(&name, thing);
                    named.push((tuple.keys, name));
                }
            }
            if let Some(held) = self.m.places.get_mut(&id) {
                held.kind = kind_id;
                held.lives = lives;
                held.things = named;
            }
        }
        Ok(())
    }

    /// Every tuple of a keyed place's keys: things of its kinds by name, keys of its indexes in
    /// their order.
    fn place_tuples(
        &self,
        id: &str,
        binds: &[crate::model::BindInfo],
        file: &str,
        line: u32,
    ) -> Result<Vec<Tuple>, Stop> {
        let mut tuples = vec![Tuple {
            keys: Vec::new(),
            key_things: Vec::new(),
            key_indexes: Vec::new(),
        }];
        for bind in binds {
            let options: Vec<(KeyVal, Option<String>)> = match &bind.domain {
                Domain::Kind(kind) => self.kind_options(kind),
                Domain::Index(index) => self.index_options(index)?,
                Domain::Axis(_) | Domain::List(_) | Domain::Range => {
                    return Err(refuse(
                        Load::Domain,
                        format!(
                            "place {id} is keyed by {}; a place is keyed by kinds or indexes",
                            bind.var
                        ),
                        file,
                        line,
                    ));
                }
            };
            let mut next = Vec::new();
            for tuple in &tuples {
                for (key, added) in &options {
                    next.push(tuple.grown(key, added.as_deref()));
                }
            }
            tuples = next;
        }
        Ok(tuples)
    }

    /// The things a keyed place's key over a kind takes, by name: every thing that ever
    /// enters the kind or a subkind, keyed places aside. L §5.10.
    fn kind_options(&self, kind: &str) -> Vec<(KeyVal, Option<String>)> {
        let sub = self.m.subtree(kind);
        let mut names: Vec<&String> = Vec::new();
        for (name, thing) in self.m.things.entries() {
            let of_kind = thing.entries.iter().any(|e| sub.contains(&e.kind));
            if thing.place.is_none() && of_kind {
                names.push(name);
            }
        }
        names.sort();
        names
            .into_iter()
            .map(|n| (KeyVal::Thing(n.clone()), None))
            .collect()
    }

    /// The keys a keyed place's key over an index takes, in its order, each with the index
    /// when a version added it. L §5.10.
    fn index_options(&self, index: &str) -> Result<Vec<(KeyVal, Option<String>)>, Stop> {
        let Some(info) = self.m.indexes.get(index) else {
            return Err(Stop::NotBuilt("an index that isn't built"));
        };
        let mut out = Vec::new();
        for key in &info.keys {
            let added = info.added_by(key).is_some().then(|| index.to_owned());
            out.push((key.clone(), added));
        }
        Ok(out)
    }

    /// The file a `place` section is in.
    fn place_file(&self, id: &str) -> String {
        let found = self.sections.iter().find(|(_, section)| {
            let Section::Place(place) = section else {
                return false;
            };
            place.id == id
        });
        found.map_or_else(String::new, |(file, _)| file.clone())
    }

    /// A place's stated unit is a currency, a unit or a key of a kind of unit (refcore's
    /// `check_unit_word`). L §5.10.
    fn check_unit_word(&self, unit: &str, file: &str, line: u32) -> Result<(), Stop> {
        if self.m.currencies.contains_key(unit)
            || self.m.units.contains(unit)
            || self.m.unit_kind_keys.contains_key(unit)
        {
            return Ok(());
        }
        if is_currency_code(unit) {
            return Err(refuse(
                Load::Unit,
                format!("money in an undeclared currency {unit}"),
                file,
                line,
            ));
        }
        Err(refuse(
            Load::Name,
            format!("unknown id {unit} (a unit)"),
            file,
            line,
        ))
    }

    /// The place a row, a leg, a `covers` line or a gap role writes: a place's id, a keyed
    /// place at its keys, or a thing of a place kind (refcore's `place_ref`). L §5.10.
    pub(super) fn place_ref(
        &self,
        written: &Expr,
        role: Written,
        file: &str,
        line: u32,
    ) -> Result<String, Stop> {
        let not_place =
            |name: &str| refuse(Load::Covers, format!("{name} is not a place"), file, line);
        if let Expr::Name(id) = written {
            if self.m.names.get(id) == Some(&NameKind::Place) {
                let keyed = self.m.places.get(id).is_some_and(|p| !p.binds.is_empty());
                if keyed {
                    return Err(refuse(
                        Load::Key,
                        format!("wrong number of keys for place {id}"),
                        file,
                        line,
                    ));
                }
                return Ok(id.clone());
            }
            if self.m.things.get(id).is_some_and(|t| self.m.is_place(t)) {
                return Ok(id.clone());
            }
            if role == Written::Covers {
                return Err(not_place(id));
            }
            if !self.m.names.contains_key(id) && !self.m.things.contains(id) {
                return Err(refuse(Load::Name, format!("unknown id {id}"), file, line));
            }
            return Err(refuse(
                Load::Kind,
                format!("{id} is not a place"),
                file,
                line,
            ));
        }
        let Some((name, keys)) = keyed_ref(written) else {
            if role == Written::Covers {
                return Err(not_place("an expression"));
            }
            return Err(refuse(
                Load::Kind,
                "a place is written by its id: an expression".to_owned(),
                file,
                line,
            ));
        };
        if self.m.names.get(&name) != Some(&NameKind::Place) {
            if role == Written::Covers {
                return Err(not_place(&name));
            }
            return Err(refuse(Load::Name, format!("unknown id {name}"), file, line));
        }
        let Some(place) = self.m.places.get(&name) else {
            return Err(Stop::NotBuilt("a place that isn't built"));
        };
        if keys.len() != place.binds.len() {
            return Err(refuse(
                Load::Key,
                format!("wrong number of keys for place {name}"),
                file,
                line,
            ));
        }
        let mut lits = Vec::new();
        for key in &keys {
            let Some(KeyLit::Lit(lit)) = KeyLit::of_expr(key) else {
                return Err(Stop::NotBuilt("a keyed place's key that isn't a literal"));
            };
            lits.push(lit);
        }
        keyed_place_thing(&self.m, &name, &lits, file, line)?.ok_or_else(|| {
            refuse(
                Load::Key,
                format!("no place {name} at these keys"),
                file,
                line,
            )
        })
    }
}

/// A keyed place as written, `konto[ana]`: its name and its key expressions.
fn keyed_ref(written: &Expr) -> Option<(String, Vec<Expr>)> {
    let Expr::Post(base, posts) = written else {
        return None;
    };
    let Expr::Name(name) = base.as_ref() else {
        return None;
    };
    let [Postfix::Keys(keys)] = posts.as_slice() else {
        return None;
    };
    let mut exprs = Vec::new();
    for key in keys {
        let Key::Expr(expr) = key else {
            return None;
        };
        lit_of_expr(expr)?;
        exprs.push(expr.clone());
    }
    Some((name.clone(), exprs))
}

impl Tuple {
    /// This tuple with one more key: a thing of a kind, or a key of an index (with the index,
    /// when a version added it).
    fn grown(&self, key: &KeyVal, added: Option<&str>) -> Tuple {
        let mut grown = Tuple {
            keys: self.keys.clone(),
            key_things: self.key_things.clone(),
            key_indexes: self.key_indexes.clone(),
        };
        grown.keys.push(key.clone());
        if let KeyVal::Thing(name) = key {
            grown.key_things.push(name.clone());
        }
        if let Some(index) = added {
            grown.key_indexes.push((index.to_owned(), key.clone()));
        }
        grown
    }
}
