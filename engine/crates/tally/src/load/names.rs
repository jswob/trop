//! The one namespace: the ids the language declares, then every section's id, each once.
//! Spec: L §2.5 (one namespace across a reality's files; a key is never an id), §2.2
//! (built-in ids), §5.9 (a band named as a built-in), §5.19 (the built-in inputs), §5.7 (the
//! prelude's kinds, `parties` among them), §8.3 (`LOAD-DUP`, `LOAD-KEY`).
//! Twin: engine/tools/refcore/loader.py (`register_builtins`, `register_names`,
//! `check_key_ids`).
//! Invariants: a built-in id is registered before any section's; the first section whose id is
//! already named is refused at its line, in section order (files by full path, then lines); no
//! key of a list or open index, and no thing but a currency or a declared place, is named like
//! an id.

use super::steps::{Loader, refuse};
use crate::base::{Code, Load, Refusal};
use crate::model::{
    BUILTIN_INDEXES, BUILTIN_INPUTS, BindInfo, Domain, InputInfo, NameKind, PRELUDE_KINDS, Ty,
};
use crate::syntax::{Bind, Domain as BindDomain, Section, Stop, is_builtin};

impl Loader {
    /// The built-in inputs, the prelude's kinds, `cut` and the built-in indexes, named first
    /// (refcore's `register_builtins`). L §5.19, §5.7, §4.1.
    pub(super) fn register_builtins(&mut self) {
        for id in BUILTIN_INPUTS {
            let mut input = InputInfo::new(id, None);
            if id == "account_place" {
                input.binds = vec![builtin_bind("a", Domain::Index("bank_account".to_owned()))];
                input.ty = Some(Ty::Key("places".to_owned()));
            } else {
                input.binds = vec![builtin_bind("x", Domain::Kind("places".to_owned()))];
                input.reading = true;
            }
            self.m.names.insert(id.to_owned(), NameKind::Input);
            self.m.inputs.push(id, input);
        }
        for id in PRELUDE_KINDS {
            self.m.names.insert(id.to_owned(), NameKind::Kind);
        }
        self.m.names.insert("cut".to_owned(), NameKind::Cut);
        for id in BUILTIN_INDEXES {
            self.m.names.insert(id.to_owned(), NameKind::Index);
        }
    }

    /// Every section's id into the one namespace, in section order (refcore's
    /// `register_names`): an id named twice is `LOAD-DUP`, and so is a band with a built-in
    /// function's name. L §2.5, §5.9.
    pub(super) fn register_names(&mut self) -> Result<(), Stop> {
        for (file, section) in &self.sections {
            let (id, line) = section.id_line();
            if self.m.names.contains_key(id) {
                let message = if is_builtin_id(id) {
                    format!("{id} is declared twice (a built-in id)")
                } else {
                    format!("{id} is declared twice")
                };
                return Err(refuse(Load::Dup, message, file, line));
            }
            if matches!(section, Section::Band(_)) && is_builtin(id) {
                let message = format!("band {id} has a built-in's name");
                return Err(refuse(Load::Dup, message, file, line));
            }
            self.m.names.insert(id.to_owned(), section_kind(section));
            if let Section::Source(source) = section {
                self.m.sources.push(id, source.as_ref().clone());
            }
        }
        Ok(())
    }
}

impl Loader {
    /// No key literal is also a top-level id: a key of a list or open index, then a thing
    /// that isn't a currency or a declared place, then an axis's option (refcore's
    /// `check_key_ids`). L §2.5, §5.5, §8.3.
    pub(super) fn check_key_ids(&self) -> Result<(), Stop> {
        for (id, index) in self.m.indexes.entries() {
            for (text, _) in &index.texts {
                if !self.m.names.contains_key(text) {
                    continue;
                }
                let message = format!("a key is also an id: {text} (a key of {id})");
                return Err(match self.index_file(id).zip(index.line) {
                    Some((file, line)) => refuse(Load::Key, message, file, line),
                    None => Stop::Refused(Refusal::new(Code::Load(Load::Key), message)),
                });
            }
        }
        for (name, _) in self.m.things.entries() {
            let other = self.m.names.get(name).copied();
            let shared = other.is_some_and(|k| k != NameKind::Unit && k != NameKind::Place);
            if shared && !self.m.currencies.contains_key(name) {
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Key),
                    format!("a key is also an id: {name} (a thing)"),
                )));
            }
        }
        for (id, axis) in self.m.axes.entries() {
            for (option, _) in &axis.options {
                if self.m.names.contains_key(option) {
                    return Err(Stop::Refused(Refusal::new(
                        Code::Load(Load::Key),
                        format!("a key is also an id: {option} (an option of {id})"),
                    )));
                }
            }
        }
        Ok(())
    }

    /// The file an `index` section stands in, by its id; none for a built-in index.
    fn index_file(&self, id: &str) -> Option<&str> {
        self.sections.iter().find_map(|(file, section)| {
            let Section::Index(decl) = section else {
                return None;
            };
            (decl.id == id).then_some(file.as_str())
        })
    }
}

/// A built-in input's key bind, written as `x in places` would be.
fn builtin_bind(var: &str, domain: Domain) -> BindInfo {
    let id = match &domain {
        Domain::Index(id) | Domain::Kind(id) | Domain::Axis(id) => id.clone(),
        Domain::List(_) | Domain::Range => String::new(),
    };
    BindInfo {
        var: var.to_owned(),
        node: Bind {
            var: var.to_owned(),
            domain: BindDomain::Name {
                id: id.clone(),
                qual: None,
            },
            on: None,
            over: None,
            filter: None,
            text: format!("{var} in {id}"),
        },
        domain,
    }
}

/// Whether the language declares `id` (refcore's `BUILTIN_IDS`): a built-in input or index,
/// a prelude kind (`parties` among them), or `cut`. L §2.5.
fn is_builtin_id(id: &str) -> bool {
    BUILTIN_INPUTS.contains(&id)
        || PRELUDE_KINDS.contains(&id)
        || BUILTIN_INDEXES.contains(&id)
        || id == "cut"
}

/// What a section's id names.
fn section_kind(section: &Section) -> NameKind {
    match section {
        Section::Source(_) => NameKind::Source,
        Section::Version(_) => NameKind::Version,
        Section::Unit(_) => NameKind::Unit,
        Section::Kind(_) => NameKind::Kind,
        Section::Index(_) => NameKind::Index,
        Section::Place(_) => NameKind::Place,
        Section::Input(_) => NameKind::Input,
        Section::Band(_) => NameKind::Band,
        Section::Def(_) => NameKind::Def,
        Section::Move(_) => NameKind::Move,
        Section::Ask(_) => NameKind::Ask,
        Section::Axis(_) => NameKind::Axis,
        Section::Fill(_) => NameKind::Fill,
    }
}

#[cfg(test)]
mod tests {
    use crate::api::Source;
    use crate::load::load;
    use crate::syntax::{Stop, parse_reality};

    const MODEL: &str =
        "model m\n  tally 2\n  on 2026-01-01\n  start 2026-01-01\n  end 2026-12-31\n";

    fn refusal(text: &str) -> String {
        let files = parse_reality(&[Source {
            path: "m.tally".to_owned(),
            text: text.to_owned(),
        }])
        .expect("it parses");
        match load(files) {
            Err(Stop::Refused(r)) => r.text(),
            Ok(_) | Err(Stop::NotBuilt(_) | Stop::Deep) => String::new(),
        }
    }

    #[test]
    fn an_id_is_declared_once_built_ins_and_parties_included() {
        assert_eq!(
            refusal(&format!("{MODEL}\ninput x: num\n\ninput x: num\n")),
            "LOAD-DUP x is declared twice (m.tally:9)",
            "a second x"
        );
        assert_eq!(
            refusal(&format!("{MODEL}\nkind parties: plain\n")),
            "LOAD-DUP parties is declared twice (a built-in id) (m.tally:7)",
            "the prelude's parties"
        );
    }

    #[test]
    fn a_band_is_not_named_as_a_built_in() {
        assert_eq!(
            refusal(&format!(
                "{MODEL}\nunit PLN: currency minor 0.01\n\nband bal(x: money PLN): num\n"
            )),
            "LOAD-DUP band bal has a built-in's name (m.tally:9)",
            "bal is a built-in function"
        );
    }
}
