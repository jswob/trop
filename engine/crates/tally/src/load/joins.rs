//! A joint model's joins: the header's `same` lines of things, places and kinds, grouped and planned as one item each.
//! Spec: L §6.1 (Identities are written: two kinds declared alike, two things of one base
//! kind, two places of one kind, unit and price input, never a place whose kind keeps parts),
//! §3.6 (a place that keeps parts), §8.3 (`LOAD-SAME`, `LOAD-PARTS`).
//! Twin: engine/tools/refcore/joint.py (`same_joins`, `sref_tokens`, `sort_name`, `thing_kind`,
//! `find`, `plan_joins` with its union-find and `groups`, `kind_sig`, `full_kind`).
//! Invariants: groups are in the order their names are first written, kinds planned first,
//! then places, then things; a group's root is found within as many links as there are, and
//! each refusal names the line of the joint model's file that joins it.

use super::check::invariant;
use super::joint::{SEP, owner_of, shown};
use super::merge::{Cat, Declared, Decls, Drop, How, Join, Plan, bare, is_builtin_id};
use crate::base::{Code, Load, Place as At, Refusal};
use crate::syntax::{Kind, KindDecl, KindSpec, Line, Price, Stop, Tok, Type};
use std::collections::{BTreeMap, BTreeSet};

/// What a kind's declaration states, for two kinds stated `same` (refcore's `kind_sig`).
type KindSig = (KindSpec, Vec<(String, String)>, Vec<(String, Type)>);

/// The names joined, in groups in the order first named, each group with its root.
type Groups = Vec<(String, Vec<String>)>;

/// One side of a group of joined places or things (refcore's `plan_joins`, its `members`).
struct Side {
    /// The name, under its owner's prefix.
    name: String,
    /// Its owner.
    owner: String,
    /// The name its owner declares it by.
    bare: String,
    /// Its kind, under its owner's prefix where the owner declares it.
    kind: String,
    /// Its unit.
    unit: Option<String>,
    /// Whether its kind is outside.
    outside: bool,
    /// Its price input, with its owner.
    priced: Option<(String, Price)>,
    /// How it is declared.
    how: How,
    /// Whether its kind keeps parts.
    parts: bool,
}

/// The header `same` lines of the joint model that name things, places or kinds, one id on
/// each side (refcore's `same_joins`), and the places of those lines among the file's lines;
/// the others stay for load. L §6.1.
///
/// # Errors
///
/// `LOAD-SAME` for two sides of different sorts.
pub(super) fn same_joins(
    jfile: &str,
    lines: &[Line],
    declared: &Declared,
) -> Result<(Vec<Join>, BTreeSet<usize>), Stop> {
    let mut joins = Vec::new();
    let mut kept = BTreeSet::new();
    for (at, line) in lines.iter().enumerate().skip(1) {
        if line.level == 0 {
            break;
        }
        let Some((a, b)) = one_id_sides(line) else {
            continue;
        };
        let (ca, da) = declared.category(&a.text);
        let (cb, db) = declared.category(&b.text);
        if !(ca.is_some_and(Cat::joined) || cb.is_some_and(Cat::joined)) {
            continue;
        }
        if ca != cb {
            let (sa, sb) = (shown(&a.text), shown(&b.text));
            let thingish = |c: Option<Cat>| matches!(c, Some(Cat::Place | Cat::Thing));
            let message = if thingish(ca) && thingish(cb) {
                format!(
                    "same {sa} = {sb}: things of kinds {} and {}",
                    shown(&thing_kind(&a.text, ca, da)),
                    shown(&thing_kind(&b.text, cb, db))
                )
            } else {
                format!("same {sa} = {sb}: {} and {}", sort_name(ca), sort_name(cb))
            };
            return Err(same_at(message, jfile, line.number));
        }
        joins.push(Join {
            left: a.text.clone(),
            right: b.text.clone(),
            line: line.number,
        });
        kept.insert(at);
    }
    Ok((joins, kept))
}

/// A `same` line's two sides when each is one id (refcore's `sref_tokens` and the test on its
/// sides). L §6.1.
fn one_id_sides(line: &Line) -> Option<(&Tok, &Tok)> {
    let (same, body) = line.toks.split_first()?;
    if same.text != "same" {
        return None;
    }
    let mut eqs = body.iter().enumerate().filter(|(_, t)| t.text == "=");
    let (Some((eq, _)), None) = (eqs.next(), eqs.next()) else {
        return None;
    };
    let (left, right) = body.split_at_checked(eq)?;
    let ([a], [_, b]) = (left, right) else {
        return None;
    };
    (a.kind == Kind::Id && b.kind == Kind::Id).then_some((a, b))
}

/// `LOAD-SAME` at a line of the joint model's file.
fn same_at(message: String, jfile: &str, line: u32) -> Stop {
    Stop::Refused(Refusal::at(
        Code::Load(Load::Same),
        message,
        At::Line(jfile.to_owned(), line),
    ))
}

/// What a message calls a category (refcore's `sort_name`).
fn sort_name(cat: Option<Cat>) -> String {
    match cat {
        Some(Cat::Kind) => "a kind".to_owned(),
        Some(Cat::Place) => "a place".to_owned(),
        Some(Cat::Thing) => "a thing".to_owned(),
        Some(Cat::Input) => "a row".to_owned(),
        Some(Cat::Obs | Cat::Move) => "a movement".to_owned(),
        Some(Cat::Def) => "a def".to_owned(),
        Some(Cat::Index) => "an index".to_owned(),
        Some(other @ (Cat::Source | Cat::Version | Cat::Axis | Cat::Band | Cat::Fill)) => {
            format!("another item ({})", other.word())
        }
        None => "another item".to_owned(),
    }
}

/// A thing's or place's kind, under its owner's prefix where the owner declares it (refcore's
/// `thing_kind`).
fn thing_kind(name: &str, cat: Option<Cat>, d: Option<&Decls>) -> String {
    let Some(d) = d else {
        return "?".to_owned();
    };
    let name_bare = bare(name);
    let place = d.places.get(name_bare).filter(|_| cat == Some(Cat::Place));
    let entry = d.entries(name_bare).first();
    let kind = match (place, entry) {
        (Some(place), _) => place.kind.clone(),
        (None, Some(entry)) => entry.kind.clone(),
        (None, None) => return "?".to_owned(),
    };
    let owner = owner_of(name);
    if is_builtin_id(&kind) || !d.kinds.contains_key(&kind) || owner.is_empty() {
        return kind;
    }
    format!("{owner}{SEP}{kind}")
}

/// A joined group's root (refcore's `find`), walked at most as far as there are links.
fn find(parent: &BTreeMap<String, String>, name: &str) -> String {
    let mut at = name;
    for _ in 0..=parent.len() {
        match parent.get(at) {
            Some(up) if up != at => at = up,
            Some(_) | None => break,
        }
    }
    at.to_owned()
}

/// The names joined, in groups in the order first named, each with its root (refcore's
/// `plan_joins`, its union-find and `groups`), and each name's link to its root.
fn groups_of(joins: &[Join]) -> (Groups, BTreeMap<String, String>) {
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for join in joins {
        for x in [&join.left, &join.right] {
            if !order.contains(x) {
                order.push(x.clone());
            }
        }
        let (ra, rb) = (find(&parent, &join.left), find(&parent, &join.right));
        if ra != rb {
            parent.insert(rb, ra);
        }
    }
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for x in &order {
        let root = find(&parent, x);
        match groups.iter_mut().find(|(r, _)| *r == root) {
            Some((_, group)) => group.push(x.clone()),
            None => groups.push((root, vec![x.clone()])),
        }
    }
    (groups, parent)
}

/// The joins planned (refcore's `plan_joins`): the names joined into groups, in the order
/// first named; kinds first, each a group's first name, declared alike; then places, of one
/// kind, unit and price input, none keyed or keeping parts, an inside one representing them;
/// then things, of one base kind. L §6.1, §3.6.
///
/// # Errors
///
/// `LOAD-SAME` or `LOAD-PARTS` as refcore refuses the group.
pub(super) fn plan_joins(jfile: &str, joins: &[Join], declared: &Declared) -> Result<Plan, Stop> {
    let (groups, parent) = groups_of(joins);
    let mut plan = Plan::default();
    let kinds: Vec<&[String]> = groups
        .iter()
        .map(|(_, g)| g.as_slice())
        .filter(|g| declared.heads_kind(g))
        .collect();
    for group in &kinds {
        if let Some((rep, rest)) = group.split_first() {
            for x in rest {
                plan.jmap.insert(x.clone(), rep.clone());
            }
        }
    }
    for group in &kinds {
        if let Some((rep, rest)) = group.split_first() {
            plan_kinds(&mut plan, rep, rest, declared)?;
        }
    }
    for (root, group) in &groups {
        let Some(head) = group.first() else {
            continue;
        };
        let cat = declared.category(head).0;
        if cat == Some(Cat::Kind) {
            continue;
        }
        let first = joins.iter().find(|j| find(&parent, &j.left) == *root);
        let first = first.ok_or_else(|| invariant("a joined group without its line".to_owned()))?;
        let at = (jfile, first.line);
        let mut sides = Vec::with_capacity(group.len());
        for x in group {
            sides.push(side(x, cat, declared, &plan.jmap, at)?);
        }
        if cat == Some(Cat::Place) {
            plan_places(&mut plan, &sides, (&first.left, &first.right), at)?;
        } else {
            plan_things(&mut plan, &sides, at)?;
        }
    }
    Ok(plan)
}

/// A group of joined kinds (refcore's `plan_joins`, its kind arm): each declared as its first
/// name is, its own declaration dropped. L §6.1.
///
/// # Errors
///
/// `LOAD-SAME same <a> = <b>: kinds declared differently`.
fn plan_kinds(
    plan: &mut Plan,
    rep: &str,
    rest: &[String],
    declared: &Declared,
) -> Result<(), Stop> {
    let nr = declared.kind_decl(rep)?;
    for x in rest {
        let nx = declared.kind_decl(x)?;
        if kind_sig(nr, owner_of(rep), &plan.jmap) != kind_sig(nx, owner_of(x), &plan.jmap) {
            let message = format!(
                "same {} = {}: kinds declared differently",
                shown(rep),
                shown(x)
            );
            return Err(Stop::Refused(Refusal::new(Code::Load(Load::Same), message)));
        }
        plan.drops
            .entry(owner_of(x).to_owned())
            .or_insert_with(Vec::new)
            .push(Drop::Kind(bare(x).to_owned()));
    }
    Ok(())
}

/// What a kind's declaration states, its parent under its owner's prefix and joined (refcore's
/// `kind_sig`): its spec, legs and attributes.
fn kind_sig(kind: &KindDecl, owner: &str, jmap: &BTreeMap<String, String>) -> KindSig {
    let spec = match &kind.spec {
        KindSpec::Sub { parent, certainty } => {
            let full = if is_builtin_id(parent) || owner.is_empty() {
                parent.clone()
            } else {
                format!("{owner}{SEP}{parent}")
            };
            KindSpec::Sub {
                parent: jmap.get(&full).cloned().unwrap_or(full),
                certainty: *certainty,
            }
        }
        other
        @ (KindSpec::Plain | KindSpec::Event(_) | KindSpec::Unit | KindSpec::Place { .. }) => {
            other.clone()
        }
    };
    (spec, kind.legs.clone(), kind.attrs.clone())
}

/// A kind under its owner's prefix where the owner declares it, joined (refcore's
/// `full_kind`).
fn full_kind(owner: &str, kind: &str, d: &Decls, jmap: &BTreeMap<String, String>) -> String {
    if is_builtin_id(kind) || !d.kinds.contains_key(kind) {
        return kind.to_owned();
    }
    let full = if owner.is_empty() {
        kind.to_owned()
    } else {
        format!("{owner}{SEP}{kind}")
    };
    jmap.get(&full).cloned().unwrap_or(full)
}

/// One side of a joined group of places or things (refcore's `plan_joins`, its `members`).
///
/// # Errors
///
/// `LOAD-SAME same names <x>, a keyed place, without its keys`.
fn side(
    name: &str,
    cat: Option<Cat>,
    declared: &Declared,
    jmap: &BTreeMap<String, String>,
    (jfile, line): (&str, u32),
) -> Result<Side, Stop> {
    let owner = owner_of(name);
    let name_bare = bare(name);
    let d = declared
        .category(name)
        .1
        .ok_or_else(|| invariant(format!("the joined name {name} has no owner")))?;
    if cat == Some(Cat::Place) {
        let place = d
            .places
            .get(name_bare)
            .ok_or_else(|| invariant(format!("the joined place {name} isn't declared")))?;
        if place.keyed {
            let message = format!(
                "same names {}, a keyed place, without its keys",
                shown(name)
            );
            return Err(same_at(message, jfile, line));
        }
        return Ok(Side {
            name: name.to_owned(),
            owner: owner.to_owned(),
            bare: name_bare.to_owned(),
            kind: full_kind(owner, &place.kind, d, jmap),
            unit: place.unit.clone(),
            outside: d.is_outside_kind(&place.kind),
            priced: place.priced.clone().map(|p| (owner.to_owned(), p)),
            how: place.how.clone(),
            parts: d.keeps_parts(name_bare),
        });
    }
    let entries = d.entries(name_bare);
    let first = entries
        .iter()
        .find(|e| !e.replace)
        .or_else(|| entries.first())
        .ok_or_else(|| invariant(format!("the joined thing {name} has no entry")))?;
    Ok(Side {
        name: name.to_owned(),
        owner: owner.to_owned(),
        bare: name_bare.to_owned(),
        kind: full_kind(owner, &first.kind, d, jmap),
        unit: None,
        outside: false,
        priced: None,
        how: How::Entry(first.line.clone()),
        parts: false,
    })
}

/// A group of joined places (refcore's `plan_joins`, its place arm): none keeps parts; those
/// not outside are of one kind, unit and price input; the first of them, else the first,
/// represents the group, and every other side's declaration is dropped. L §6.1, §3.6.
///
/// # Errors
///
/// `LOAD-PARTS`, `LOAD-SAME`.
fn plan_places(
    plan: &mut Plan,
    sides: &[Side],
    (a0, b0): (&str, &str),
    (jfile, line): (&str, u32),
) -> Result<(), Stop> {
    if let Some(keeps) = sides.iter().find(|s| s.parts) {
        let message = format!(
            "same {} = {}: {} keeps parts",
            shown(a0),
            shown(b0),
            shown(&keeps.name)
        );
        return Err(Stop::Refused(Refusal::at(
            Code::Load(Load::Parts),
            message,
            At::Line(jfile.to_owned(), line),
        )));
    }
    let inside: Vec<&Side> = sides.iter().filter(|s| !s.outside).collect();
    let kinds: BTreeSet<&str> = inside.iter().map(|s| s.kind.as_str()).collect();
    let mut sorted = kinds.iter();
    if let (Some(k0), Some(k1)) = (sorted.next(), sorted.next())
        && let (Some(p), Some(q)) = (
            inside.iter().find(|s| s.kind == *k0),
            inside.iter().find(|s| s.kind == *k1),
        )
    {
        let message = format!(
            "same {} = {}: places of kinds {} and {}",
            shown(&p.name),
            shown(&q.name),
            shown(&p.kind),
            shown(&q.kind)
        );
        return Err(same_at(message, jfile, line));
    }
    let two = |differ: bool, what: &str| -> Result<(), Stop> {
        if let (true, Some(a), Some(b)) = (differ, inside.first(), inside.get(1)) {
            let message = format!(
                "same {} = {}: places of {what}",
                shown(&a.name),
                shown(&b.name)
            );
            return Err(same_at(message, jfile, line));
        }
        Ok(())
    };
    if let Some(f) = inside.first() {
        two(inside.iter().any(|s| s.unit != f.unit), "two units")?;
        two(
            inside.iter().any(|s| s.priced != f.priced),
            "two price inputs",
        )?;
    }
    // the first side inside represents the group; with every side outside, the first
    let inside_first = sides.iter().enumerate().find(|(_, s)| !s.outside);
    let (rep, rep_side) = inside_first
        .or_else(|| sides.iter().enumerate().next())
        .ok_or_else(|| invariant("a joined group without sides".to_owned()))?;
    let rep_name = rep_side.name.clone();
    let others = sides.iter().enumerate().filter(|(at, _)| *at != rep);
    for s in others.map(|(_, s)| s) {
        plan.jmap.insert(s.name.clone(), rep_name.clone());
        match &s.how {
            How::Section(start) => plan
                .drops
                .entry(s.owner.clone())
                .or_insert_with(Vec::new)
                .push(Drop::Block(start.clone())),
            How::Entry(Some(entry)) => {
                let owned = plan
                    .keys
                    .entry(s.owner.clone())
                    .or_insert_with(BTreeMap::new);
                let at = owned.entry(entry.clone()).or_insert_with(BTreeSet::new);
                at.insert(s.bare.clone());
            }
            How::Entry(None) => {}
        }
    }
    Ok(())
}

/// A group of joined things (refcore's `plan_joins`, its thing arm): of one base kind, the
/// first representing them, every entry kept. L §6.1.
///
/// # Errors
///
/// `LOAD-SAME same <a> = <b>: things of kinds <k> and <l>`.
fn plan_things(plan: &mut Plan, sides: &[Side], (jfile, line): (&str, u32)) -> Result<(), Stop> {
    let Some((first, rest)) = sides.split_first() else {
        return Ok(());
    };
    if let (Some(second), true) = (rest.first(), rest.iter().any(|s| s.kind != first.kind)) {
        let message = format!(
            "same {} = {}: things of kinds {} and {}",
            shown(&first.name),
            shown(&second.name),
            shown(&first.kind),
            shown(&second.kind)
        );
        return Err(same_at(message, jfile, line));
    }
    for s in rest {
        plan.jmap.insert(s.name.clone(), first.name.clone());
    }
    Ok(())
}
