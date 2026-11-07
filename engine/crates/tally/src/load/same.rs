//! Load's `same` lines of rows and movements: each side named, both sides of one sort, two rows of one type.
//! Spec: L §6.1 (Identities are written: a side is a kind, a thing, a row or a movement, both
//! of one sort, `LOAD-SAME` otherwise; a `same` in a version that names a kind or a thing is
//! refused), §5.19 (`same` between observed rows), §8.3 (`LOAD-SAME`'s forms).
//! Twin: engine/tools/refcore/loader.py (`joint_sames`, `sref`, `move_keys`, `_txt_node`,
//! `joint_place_sames`, `place_side`, `_has_keys`).
//! Invariants: the header's lines are taken first, then the versions' in the order load built
//! their items; each side is resolved before the pair is judged, the left first; things,
//! places and kinds a joint model's header names by id were joined before parsing (`joint.rs`),
//! so only rows and movements stay; a pair is recorded for the run, which reads it at M2; a
//! joint model's own row of a member's item stands only at a pair stated `same`.

use super::check::invariant;
use super::joint::owner_of;
use super::keys::{KeyLit, lit_of_expr, lit_text, resolve_key};
use super::steps::{Loader, refuse};
use super::types::text;
use crate::base::Load;
use crate::model::{KeyVal, NameKind, Pat, SamePair, Ty, pattern_text};
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
    super::joint::shown(&text(side))
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
                            "same {} = {}: rows of types {} and {}",
                            super::joint::shown(&ta),
                            super::joint::shown(&tb),
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
        if self.m.joint.is_some() {
            self.joint_rows()?;
        }
        Ok(())
    }

    /// A joint model's own rows of a member's item: each stands only at a pair of rows stated
    /// `same`, at all its keys (refcore's `joint_sames`, its last loop); a scenario's row may
    /// replace a member's. L §6.1 (The joint model's own rows; Writes into members).
    fn joint_rows(&self) -> Result<(), Stop> {
        for (id, input) in self.m.inputs.entries() {
            let member = owner_of(id);
            if member.is_empty() || input.observed {
                continue;
            }
            for row in &input.rows {
                let scenario = self.m.scenarios.of(&row.version).is_some();
                if !owner_of(&row.version).is_empty() || scenario {
                    continue;
                }
                let keys: Vec<&KeyVal> = row
                    .pattern
                    .iter()
                    .filter_map(|p| match p {
                        Pat::Key(key) => Some(key),
                        Pat::Star | Pat::Range(..) => None,
                    })
                    .collect();
                let at = |x: &str, xs: &[KeyVal]| x == id && xs.iter().eq(keys.iter().copied());
                let stated = self.m.sames.iter().any(|pair| match pair {
                    SamePair::Rows(a, ka, b, kb) => at(a, ka) || at(b, kb),
                    SamePair::Moves(..) => false,
                });
                if !stated || keys.len() != input.binds.len() {
                    let file = self
                        .m
                        .versions
                        .get(&row.version)
                        .map(|v| v.file.clone())
                        .ok_or_else(|| invariant(format!("version {} isn't built", row.version)))?;
                    let message = format!(
                        "the joint model states {}{}, an item of member {member} that no same line joins",
                        super::joint::shown(id),
                        pattern_text(&row.pattern)
                    );
                    return Err(refuse(Load::Same, message, &file, row.line));
                }
            }
        }
        Ok(())
    }

    /// A joint model's header `same` lines whose side has keys and names a place (refcore's
    /// `joint_place_sames`): a place written beside another kind of side is refused; two
    /// places at their keys are joined before the rows are read, which isn't built yet. L §6.1.
    pub(super) fn joint_place_sames(&self) -> Result<(), Stop> {
        if self.m.joint.is_none() {
            return Ok(());
        }
        for same in self.same_lines.iter().filter(|s| s.version.is_none()) {
            if !(has_keys(&same.left) || has_keys(&same.right)) {
                continue;
            }
            let a = self.place_side(&same.left, same)?;
            let b = self.place_side(&same.right, same)?;
            match (a, b) {
                (false, false) => {}
                (true, true) => return Err(Stop::NotBuilt("two places stated same at their keys")),
                (true, false) | (false, true) => {
                    let message = format!(
                        "same {} = {}: a place and a thing",
                        shown(&same.left),
                        shown(&same.right)
                    );
                    return Err(refuse(Load::Same, message, &same.file, same.line));
                }
            }
        }
        Ok(())
    }

    /// Whether a header `same` line's side names a place (refcore's `place_side`, not `None`):
    /// a place by its id, at its keys when it is keyed, or a thing of a place kind. L §6.1.
    fn place_side(&self, side: &Expr, same: &SameLine) -> Result<bool, Stop> {
        let Some((name, keys)) = named(side) else {
            return Ok(false);
        };
        let wrong = || {
            let message = format!("wrong number of keys for {name}");
            refuse(Load::Key, message, &same.file, same.line)
        };
        match self.m.names.get(name.as_str()) {
            None => {
                Ok(keys.is_empty() && self.m.things.get(name).is_some_and(|t| self.m.is_place(t)))
            }
            Some(NameKind::Place) => {
                let Some(place) = self.m.places.get(name) else {
                    return Err(Stop::NotBuilt("a place that isn't built"));
                };
                if place.binds.is_empty() {
                    return if keys.is_empty() {
                        Ok(true)
                    } else {
                        Err(wrong())
                    };
                }
                if keys.is_empty() {
                    let message = format!(
                        "same names {}, a keyed place, without its keys",
                        super::joint::shown(name)
                    );
                    return Err(refuse(Load::Same, message, &same.file, same.line));
                }
                if keys.len() != place.binds.len() {
                    return Err(wrong());
                }
                Ok(true)
            }
            Some(
                NameKind::Unit
                | NameKind::Kind
                | NameKind::Index
                | NameKind::Axis
                | NameKind::Input
                | NameKind::Def
                | NameKind::Band
                | NameKind::Fill
                | NameKind::Move
                | NameKind::Version
                | NameKind::Ask
                | NameKind::Source
                | NameKind::Cut,
            ) => Ok(false),
        }
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
            return Err(at(Load::Name, format!("unknown id {name}")).naming(name));
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
            // The parser's `sref` keys a side by literals and places keyed by literals only.
            let Some(written) = written else {
                return Err(invariant(
                    "a same side keyed by * or a computed key".to_owned(),
                ));
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
            // An observation's attribute naming a move by a computed key (`item: wage[2026-02 +
            // 1]`) reaches this (a `same` side's keys and `*` are refused at parse, #88);
            // refcore prints `LOAD-KEY None is not a key of <ix>` there (issue #100, P1).
            let Some(written) = written else {
                return Err(Stop::NotBuilt("a reference keyed by a computed key"));
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

/// Whether a side is written with keys (refcore's `_has_keys`): `x[k]`, or a field of one.
fn has_keys(side: &Expr) -> bool {
    matches!(side, Expr::Post(_, posts) if posts.iter().any(|p| matches!(p, Postfix::Keys(_))))
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
