//! Load's checks of a binding written in an expression: its variable's name, what it ranges over, and whether it may state `on` or `over`.
//! Spec: L §5.6 (a binding is over an index, a kind, a key list or a stepped range; `on` and
//! `over` state when a thing of a kind is alive, so no other binding has them), §2.5 (a bound
//! variable never takes the name of a top-level id, of a variable in scope or of a key of its
//! own domain), §5.7 and §5.14.3 (a binding over a kind or over movements needs `on` or
//! `over`), §8.3 (`LOAD-SHADOW`, `LOAD-NAME`, `LOAD-ALIVE`, `LOAD-DOMAIN`, `LOAD-KEY`).
//! Twin: engine/tools/refcore/check.py (`bind`, `bind_finish`, the checks that need no type);
//! check3.py (`m_bind_movements`, its `LOAD-ALIVE`).
//! Invariants: the checks run in refcore's order; a check that needs an expression's type (a
//! range's ends, `on`'s date, `where`'s bool) or a rule's guards (`LOAD-COVERED`) is the
//! checker's (M0's G4b), so none is guessed here; a refusal names no place, the walk puts it
//! at its line.

use crate::base::{Code, Load, Refusal};
use crate::model::{KindSpec, NameKind, Reality};
use crate::syntax::{Bind, Domain as Written, Lit, Stop};

/// What a binding ranges over, as refcore's `bind` resolves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Over {
    /// The keys of an index.
    Index,
    /// The things of a kind that isn't an event kind.
    Things(String),
    /// The movements of an event kind.
    Movements(String),
    /// The rows of an observed input.
    Observed(String),
    /// A key list's keys.
    List,
    /// A stepped range's values.
    Range,
}

impl Over {
    /// Whether things or movements are bound, which are alive on some days: only then may a
    /// binding state `on` or `over`. L §5.6.
    fn is_alive(&self) -> bool {
        match self {
            Over::Things(_) | Over::Movements(_) | Over::Observed(_) => true,
            Over::Index | Over::List | Over::Range => false,
        }
    }
}

/// `LOAD-SHADOW` for a binding whose variable is already bound around it, refcore's first
/// check of a binding; the walk, which holds the variables in scope, makes it. L §2.5.
pub(super) fn bound_twice(var: &str) -> Stop {
    refused(
        Load::Shadow,
        format!("bound variable {var} has the name of a bound variable in scope"),
    )
}

/// What `bind` ranges over, its variable no top-level id and no key of its domain (refcore's
/// `bind`, after its scope check, up to its `bind_finish`). L §5.6, §2.5.
///
/// # Errors
///
/// `LOAD-SHADOW`, `LOAD-NAME`, `LOAD-ALIVE`, `LOAD-DOMAIN` or `LOAD-KEY`, with no place; a
/// member's prefix stops "not built".
pub(super) fn over(m: &Reality, bind: &Bind) -> Result<Over, Stop> {
    let var = &bind.var;
    if m.names.contains_key(var) {
        return Err(refused(
            Load::Shadow,
            format!("bound variable {var} has the name of a top-level id"),
        ));
    }
    match &bind.domain {
        Written::Name { qual: Some(_), .. } => Err(Stop::NotBuilt("a member's prefix")),
        Written::Name { id, qual: None } => over_name(m, bind, id),
        Written::List(keys) => {
            if keys.iter().any(|key| key_text(key) == *var) {
                return Err(refused(
                    Load::Shadow,
                    format!("bound variable {var} has the spelling of a key of its list"),
                ));
            }
            Ok(Over::List)
        }
        Written::Range(_) => Ok(Over::Range),
    }
}

/// A binding over a name: an index, a kind, an event kind's movements or an observed input's
/// rows (refcore's `bind`, its `dname` arm, and `m_bind_movements`). L §5.6, §5.7, §5.14.3.
fn over_name(m: &Reality, bind: &Bind, id: &str) -> Result<Over, Stop> {
    let var = &bind.var;
    let alive = bind.on.is_some() || bind.over.is_some();
    let needs_alive = || refused(Load::Alive, format!("binding over {id} needs on or over"));
    match m.names.get(id) {
        None => Err(refused(Load::Name, format!("unknown id {id}"))),
        Some(NameKind::Index) => {
            if m.indexes
                .get(id)
                .is_some_and(|ix| ix.by_text(var).is_some())
            {
                return Err(key_spelling(var, id));
            }
            Ok(Over::Index)
        }
        Some(NameKind::Kind) => {
            let event = m.kinds.get(id).is_some_and(|k| k.spec == KindSpec::Event);
            if event {
                return if alive {
                    Ok(Over::Movements(id.to_owned()))
                } else {
                    Err(needs_alive())
                };
            }
            if m.thing_in(var, id) {
                return Err(key_spelling(var, id));
            }
            if !alive {
                return Err(needs_alive());
            }
            Ok(Over::Things(id.to_owned()))
        }
        Some(NameKind::Input) if m.inputs.get(id).is_some_and(|i| i.observed) => {
            if alive {
                Ok(Over::Observed(id.to_owned()))
            } else {
                Err(needs_alive())
            }
        }
        Some(NameKind::Axis) => Err(refused(
            Load::Domain,
            format!(
                "a binding over the axis {id}: a bind is over an index, a kind, a key list or a stepped range (L §5.6)"
            ),
        )),
        Some(
            NameKind::Unit
            | NameKind::Input
            | NameKind::Def
            | NameKind::Band
            | NameKind::Place
            | NameKind::Fill
            | NameKind::Move
            | NameKind::Version
            | NameKind::Ask
            | NameKind::Source
            | NameKind::Cut,
        ) => Err(refused(
            Load::Key,
            format!("{id} is not an index, a kind, a key list or a range"),
        )),
    }
}

/// `on` or `over` (`word`) only on a binding over things or movements (refcore's
/// `bind_finish`, its domain checks, each after its clause's own check). L §5.6.
///
/// # Errors
///
/// `LOAD-DOMAIN`, with no place.
pub(super) fn alive_clause(word: &str, over: &Over) -> Result<(), Stop> {
    if over.is_alive() {
        return Ok(());
    }
    Err(refused(
        Load::Domain,
        format!("{word} on a binding that is not over a kind"),
    ))
}

/// `LOAD-SHADOW`: a bound variable spelled as a key of its domain.
fn key_spelling(var: &str, id: &str) -> Stop {
    refused(
        Load::Shadow,
        format!("bound variable {var} has the spelling of a key of {id}"),
    )
}

/// A key list's key as its text compares (refcore's `resolve_text`). L §2.5.
fn key_text(key: &Lit) -> String {
    key.text()
}

/// A load refusal with no place yet.
fn refused(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

#[cfg(test)]
mod tests {
    use super::{Over, alive_clause, bound_twice, over};
    use crate::api::Source;
    use crate::load::load;
    use crate::model::Reality;
    use crate::syntax::{Bind, Domain, Expr, Lit, Stop, parse_reality};

    /// A reality with an index, an input, a place kind and an observed input.
    fn reality() -> Reality {
        let text = "model m\n  tally 2\n  on 2026-03-01\n  start 2026-01-01\n  end 2026-12-31\n\n\
                    unit PLN: currency minor 0.01\n\nindex qs = [q1, q2]\n\ninput amt: money PLN\n\n\
                    input bank_rows: observed\n\nplace konto: accounts PLN lives 2026-01-01..\n";
        let files = parse_reality(&[Source {
            path: "a.tally".to_owned(),
            text: text.to_owned(),
        }])
        .expect("it parses");
        load(files).expect("it loads")
    }

    /// `var in id`.
    fn bind(var: &str, id: &str) -> Bind {
        Bind {
            var: var.to_owned(),
            domain: Domain::Name {
                id: id.to_owned(),
                qual: None,
            },
            on: None,
            over: None,
            filter: None,
            text: String::new(),
        }
    }

    /// `var in id on cut`.
    fn alive(var: &str, id: &str) -> Bind {
        Bind {
            on: Some(Expr::Cut),
            ..bind(var, id)
        }
    }

    /// The text of a refusal, or what else came back.
    fn message(result: Result<Over, Stop>) -> String {
        match result {
            Err(Stop::Refused(refusal)) => refusal.text(),
            other => format!("{other:?}"),
        }
    }

    /// Each binding with the refusal it gets, or what it ranges over.
    fn check_all(m: &Reality, cases: &[(Bind, &str)]) {
        for (bind, want) in cases {
            assert_eq!(
                message(over(m, bind)),
                *want,
                "{} in {:?}",
                bind.var,
                bind.domain
            );
        }
    }

    #[test]
    fn a_bound_variable_takes_no_name_already_in_use() {
        let listed = Bind {
            domain: Domain::List(vec![Lit::Name("a".to_owned()), Lit::Name("b".to_owned())]),
            ..bind("b", "qs")
        };
        check_all(
            &reality(),
            &[
                (
                    bind("amt", "qs"),
                    "LOAD-SHADOW bound variable amt has the name of a top-level id",
                ),
                (
                    bind("q1", "qs"),
                    "LOAD-SHADOW bound variable q1 has the spelling of a key of qs",
                ),
                (
                    listed,
                    "LOAD-SHADOW bound variable b has the spelling of a key of its list",
                ),
            ],
        );
        assert_eq!(
            message(Err(bound_twice("k"))),
            "LOAD-SHADOW bound variable k has the name of a bound variable in scope",
            "a variable bound around it"
        );
    }

    #[test]
    fn a_binding_names_what_it_ranges_over() {
        check_all(
            &reality(),
            &[
                (bind("k", "qs"), "Ok(Index)"),
                (bind("e", "event"), "LOAD-NAME unknown id event"),
                (
                    bind("x", "amt"),
                    "LOAD-KEY amt is not an index, a kind, a key list or a range",
                ),
            ],
        );
    }

    #[test]
    fn a_binding_over_things_or_movements_says_when() {
        check_all(
            &reality(),
            &[
                (
                    bind("p", "accounts"),
                    "LOAD-ALIVE binding over accounts needs on or over",
                ),
                (
                    bind("t", "transfers"),
                    "LOAD-ALIVE binding over transfers needs on or over",
                ),
                (
                    bind("o", "bank_rows"),
                    "LOAD-ALIVE binding over bank_rows needs on or over",
                ),
                (alive("p", "accounts"), "Ok(Things(\"accounts\"))"),
                (alive("o", "bank_rows"), "Ok(Observed(\"bank_rows\"))"),
            ],
        );
        let observed = Over::Observed("bank_rows".to_owned());
        assert!(alive_clause("on", &observed).is_ok(), "on on observed rows");
        let refused = match alive_clause("over", &Over::Index) {
            Err(Stop::Refused(refusal)) => refusal.text(),
            other => format!("{other:?}"),
        };
        assert_eq!(
            refused, "LOAD-DOMAIN over on a binding that is not over a kind",
            "over on an index's keys"
        );
    }
}
