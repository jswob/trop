//! Load's `same` lines of rows and movements: each side named, both sides of one sort, two rows of one type.
//! Spec: L §6.1 (Identities are written: a side is a kind, a thing, a row or a movement, both
//! of one sort, `LOAD-SAME` otherwise; a `same` in a version that names a kind or a thing is
//! refused), §5.19 (`same` between observed rows), §8.3 (`LOAD-SAME`'s forms).
//! Twin: engine/tools/refcore/loader.py (`joint_sames`, `sref`, `move_keys`, `_txt_node`).
//! Invariants: the header's lines are taken first, then the versions' in the order load built
//! their items; each side is resolved before the pair is judged, the left first; things,
//! places and kinds a joint model's header names by id were joined before parsing (`joint.rs`),
//! so only rows and movements stay; a pair is recorded for the run, which reads it at M2.

use super::check::invariant;
use super::keys::{KeyLit, lit_of_expr, lit_text, resolve_key};
use super::steps::{Loader, refuse};
use super::types::text;
use crate::base::Load;
use crate::model::{KeyVal, NameKind, SamePair, Ty};
use crate::syntax::{Expr, Key, Lit, Postfix, Stop};

/// One `same` line as written: its two sides, where it stands, and the version that holds it
/// (`None` for the header's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SameLine {
    /// The left side.
    pub(super) left: Expr,
    /// The right side.
    pub(super) right: Expr,
    /// The file.
    pub(super) file: String,
    /// The line.
    pub(super) line: u32,
    /// The version that states it, or `None` in a joint model's header.
    pub(super) version: Option<String>,
}

/// What a side names (refcore's `sref`): a movement, a row, a thing, or another item.
enum Side {
    /// A move instance or an observed row, by its id.
    Move(String),
    /// An input's row at its keys: the input, its keys, its text and its type.
    Row(String, Vec<KeyVal>, String, Option<Ty>),
    /// A kind, a place or a thing.
    Thing,
    /// Any other item, by what a message calls it (`a def`, `a field path`).
    Other(String),
}

impl Side {
    /// What a message calls the side (refcore's `what`).
    fn what(&self) -> String {
        match self {
            Side::Move(_) => "a movement".to_owned(),
            Side::Row(..) => "a row".to_owned(),
            Side::Thing => "a thing".to_owned(),
            Side::Other(what) => what.clone(),
        }
    }

    /// The side's sort, for the pair's test.
    fn sort(&self) -> u8 {
        match self {
            Side::Move(_) => 0,
            Side::Row(..) => 1,
            Side::Thing => 2,
            Side::Other(_) => 3,
        }
    }
}

/// A side's text as a message names it, a member's prefix printed with its dot (refcore's
/// `shown(_txt_node(n))`).
fn shown(side: &Expr) -> String {
    text(side)
}

impl Loader {
    /// Every `same` line of rows and movements, the header's first, then the versions' (refcore's
    /// `joint_sames`). L §6.1, §5.19.
    pub(super) fn check_sames(&mut self) -> Result<(), Stop> {
        let lines = core::mem::take(&mut self.same_lines);
        for same in &lines {
            let left = self.side(&same.left, same)?;
            let right = self.side(&same.right, same)?;
            let (l, r) = (shown(&same.left), shown(&same.right));
            let at = |message: String| refuse(Load::Same, message, &same.file, same.line);
            if same.version.is_some()
                && (matches!(left, Side::Thing) || matches!(right, Side::Thing))
            {
                return Err(at(format!(
                    "same {l} = {r} in a version names a kind or a thing"
                )));
            }
            if matches!(left, Side::Other(_)) || left.sort() != right.sort() {
                return Err(at(format!(
                    "same {l} = {r}: {} and {}",
                    left.what(),
                    right.what()
                )));
            }
            match (left, right) {
                (Side::Row(a, ka, ta, tya), Side::Row(b, kb, tb, tyb)) => {
                    if tya != tyb {
                        return Err(at(format!(
                            "same {ta} = {tb}: rows of types {} and {}",
                            shown_type(tya.as_ref()),
                            shown_type(tyb.as_ref())
                        )));
                    }
                    self.m.sames.push(SamePair::Rows(a, ka, b, kb));
                }
                (Side::Move(a), Side::Move(b)) => self.m.sames.push(SamePair::Moves(a, b)),
                (Side::Thing, Side::Thing) => {
                    return Err(at(format!(
                        "same {l} = {r}: a thing written with keys that is not a keyed place"
                    )));
                }
                (Side::Move(_) | Side::Row(..) | Side::Thing | Side::Other(_), _) => {
                    return Err(invariant(
                        "a same pair of two sorts past the sort test".to_owned(),
                    ));
                }
            }
        }
        self.same_lines = lines;
        Ok(())
    }

    /// A side of a `same` line (refcore's `sref`). L §6.1.
    fn side(&self, side: &Expr, same: &SameLine) -> Result<Side, Stop> {
        let at = |code: Load, message: String| refuse(code, message, &same.file, same.line);
        let Some((name, keys)) = named(side) else {
            return Ok(Side::Other("a field path".to_owned()));
        };
        let Some(kind) = self.m.names.get(name.as_str()) else {
            if self.m.things.get(name).is_some() {
                return Ok(Side::Thing);
            }
            return Err(at(Load::Name, format!("unknown id {name}")));
        };
        match kind {
            NameKind::Move => {
                let values = self.move_instance_keys(name, keys, &same.file, same.line)?;
                Ok(Side::Move(instance_text(name, &values)))
            }
            NameKind::Input => self.input_side(name, keys, same),
            NameKind::Place | NameKind::Kind => Ok(Side::Thing),
            NameKind::Unit
            | NameKind::Index
            | NameKind::Axis
            | NameKind::Def
            | NameKind::Band
            | NameKind::Fill
            | NameKind::Version
            | NameKind::Ask
            | NameKind::Source
            | NameKind::Cut => Ok(Side::Other(format!("a {}", super::types::name_word(*kind)))),
        }
    }

    /// A side naming an input: an observed row by its id, or a row at its keys. L §6.1, §5.19.
    fn input_side(&self, name: &str, keys: &[Key], same: &SameLine) -> Result<Side, Stop> {
        let at = |message: String| refuse(Load::Key, message, &same.file, same.line);
        let Some(info) = self.m.inputs.get(name) else {
            return Err(Stop::NotBuilt("an input that isn't built"));
        };
        if info.observed {
            let Some(id) = row_id(keys) else {
                return Err(at(format!(
                    "an observation row is named by its id: {name}[<id>]"
                )));
            };
            return Ok(Side::Move(format!("{name}[{id}]")));
        }
        if keys.len() != info.binds.len() {
            return Err(at(format!("wrong number of keys for {name}")));
        }
        let mut values = Vec::with_capacity(keys.len());
        for (key, bind) in keys.iter().zip(&info.binds) {
            let written = match key {
                Key::Expr(key) => KeyLit::of_expr(key),
                Key::Star => None,
            };
            let Some(written) = written else {
                return Err(Stop::NotBuilt("a reference keyed by * or a computed key"));
            };
            values.push(resolve_key(
                &self.m,
                &written,
                &bind.domain,
                &same.file,
                same.line,
            )?);
        }
        let text = instance_text(name, &values);
        Ok(Side::Row(name.to_owned(), values, text, info.ty.clone()))
    }

    /// A move instance's written keys, each a literal (refcore's `move_keys`). L §2.5.
    pub(super) fn move_instance_keys(
        &self,
        name: &str,
        keys: &[Key],
        file: &str,
        line: u32,
    ) -> Result<Vec<KeyVal>, Stop> {
        let Some(info) = self.m.moves.get(name) else {
            return Err(Stop::NotBuilt("a move that isn't built"));
        };
        if keys.len() != info.binds.len() {
            return Err(refuse(
                Load::Key,
                format!("wrong number of keys for {name}"),
                file,
                line,
            ));
        }
        let mut values = Vec::with_capacity(keys.len());
        for (key, bind) in keys.iter().zip(&info.binds) {
            let written = match key {
                Key::Expr(key) => KeyLit::of_expr(key),
                Key::Star => None,
            };
            let Some(written) = written else {
                return Err(Stop::NotBuilt("a reference keyed by * or a computed key"));
            };
            values.push(resolve_key(&self.m, &written, &bind.domain, file, line)?);
        }
        Ok(values)
    }
}

/// A reference's name and written keys: `x` or `x[k, …]`; `None` for any other shape (a
/// field path).
pub(super) fn named(side: &Expr) -> Option<(&String, &[Key])> {
    if let Expr::Name(name) = side {
        return Some((name, &[]));
    }
    if let Expr::Post(base, posts) = side
        && let (Expr::Name(name), [Postfix::Keys(keys)]) = (base.as_ref(), posts.as_slice())
    {
        return Some((name, keys.as_slice()));
    }
    None
}

/// An observed row's id as a reference writes it, one name, key or number (refcore's
/// `resolve_text`, after its `keys[0].k in ('name', 'key', 'num')` test).
pub(super) fn row_id(keys: &[Key]) -> Option<String> {
    let [Key::Expr(key)] = keys else {
        return None;
    };
    let lit = lit_of_expr(key)?;
    matches!(lit, Lit::Name(_) | Lit::Key(_) | Lit::Num(_)).then(|| lit_text(&lit))
}

/// `x[k1, k2]`, or `x` with no keys.
pub(super) fn instance_text(name: &str, keys: &[KeyVal]) -> String {
    if keys.is_empty() {
        return name.to_owned();
    }
    let texts: Vec<String> = keys.iter().map(KeyVal::text).collect();
    format!("{name}[{}]", texts.join(", "))
}

/// A row's type as a message names it.
fn shown_type(ty: Option<&Ty>) -> String {
    ty.map_or_else(String::new, Ty::show)
}
