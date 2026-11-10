//! Load's replacement rule: rows at the same keys overlap only through `replace`, and each observation id is one movement on a covered day.
//! Spec: L §4.6 steps 2–4 (replacement, precedence by layer, narrower row and span; ties),
//! §5.3 (LOAD-OVERLAP, LOAD-REPLACE; rows of one version ordered by precedence), §5.19
//! (an observation's id, restated with `replace`; `covers` and LOAD-COVERS), §6.4 (a
//! scenario's row above its base's whatever their `known`), §8.3.
//! Twin: engine/tools/refcore/loader.py (`check_overlap`, `_pos_rel`, `narrower`,
//! `patterns_meet`, `key_order`, `row_layer`, `version_layer`, `check_obs`).
//! Invariants: a row's span is its own, else its version's in force at `on`; two readings
//! overlap only at one instant stated alike, and two `after m` instants never do (refcore's
//! instants compare by their node); a target's rows are grouped by pattern in the order the
//! patterns first appear, and the first pair that breaks the rule is refused at refcore's row.

use super::check::invariant;
use super::steps::refuse;
use crate::base::Load;
use crate::model::{
    BindInfo, Domain, KeyVal, Observation, Pat, Reality, Row, RowInstant, Span, pattern_text,
};
use crate::syntax::Stop;
use crate::time::Day;
use core::cmp::Ordering;

/// What a balance reading's value groups by (refcore's `r.val` in `check_overlap`): the unit
/// of its quantity, a hole, or `none`, which states no balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Stated {
    /// A quantity in this unit.
    Unit(String),
    /// A stated hole.
    Hole,
    /// `none`.
    Nothing,
}

/// One group of a target's rows: a pattern, a balance reading's unit, the rows' places.
struct Group {
    /// The rows' pattern.
    pattern: Vec<Pat>,
    /// A balance reading's unit.
    unit: Option<String>,
    /// The rows, by their place in the target's rows.
    rows: Vec<usize>,
}

/// The replacement rule over one target's rows (refcore's `check_overlap`): for an input or
/// a def's givens, with `stated` the value of each balance reading of `opening` or `stated`.
/// L §4.6 steps 2–4, §5.3.
///
/// # Errors
///
/// `LOAD-OVERLAP` or `LOAD-REPLACE` at the row refcore names.
pub(super) fn check_overlap(
    m: &Reality,
    id: &str,
    binds: &[BindInfo],
    rows: &[Row],
    stated: Option<&[Stated]>,
) -> Result<(), Stop> {
    let groups = group_rows(rows, stated);
    for group in &groups {
        same_keys(m, id, rows, &group.rows)?;
    }
    if stated.is_some() {
        return Ok(());
    }
    for (i, a) in groups.iter().enumerate() {
        for b in groups.iter().skip(i).skip(1) {
            ties_across(m, id, binds, rows, a, b)?;
        }
    }
    Ok(())
}

/// A target's rows grouped by pattern, and a balance reading's by its unit too; a reading
/// of `none` joins the one unit's group of its pattern, else the pattern's own (refcore's
/// `check_overlap`, its groups).
fn group_rows(rows: &[Row], stated: Option<&[Stated]>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut nothing = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        let unit = match stated.and_then(|s| s.get(at)) {
            Some(Stated::Unit(u)) => Some(u.clone()),
            Some(Stated::Nothing) => {
                nothing.push(at);
                continue;
            }
            Some(Stated::Hole) | None => None,
        };
        join(&mut groups, &row.pattern, unit, at);
    }
    for at in nothing {
        let Some(row) = rows.get(at) else {
            continue;
        };
        let units: Vec<&Group> = groups
            .iter()
            .filter(|g| g.pattern == row.pattern && g.unit.is_some())
            .collect();
        let unit = match units.as_slice() {
            [one] => one.unit.clone(),
            _ => None,
        };
        join(&mut groups, &row.pattern, unit, at);
    }
    groups
}

/// Puts row `at` into the group of `pattern` and `unit`, a new one at the end if none.
fn join(groups: &mut Vec<Group>, pattern: &[Pat], unit: Option<String>, at: usize) {
    if let Some(group) = groups
        .iter_mut()
        .find(|g| g.pattern == pattern && g.unit == unit)
    {
        group.rows.push(at);
        return;
    }
    groups.push(Group {
        pattern: pattern.to_vec(),
        unit,
        rows: vec![at],
    });
}

/// Rows at the same keys (refcore's `check_overlap`, its pairs and its `replace`s): two
/// that overlap from two layers, the higher says `replace`; from one layer, known the same
/// day, they tie unless precedence orders a span over `always`; else the later says
/// `replace`; and a `replace` has an earlier row to replace. L §4.6 step 2, §6.4.
fn same_keys(m: &Reality, id: &str, rows: &[Row], group: &[usize]) -> Result<(), Stop> {
    let mut members = Vec::new();
    for at in group {
        let row = rows.get(*at);
        members.push(row.ok_or_else(|| invariant("a row group past its rows".to_owned()))?);
    }
    for (i, a) in members.iter().enumerate() {
        for b in members.iter().skip(i).skip(1) {
            pair(m, id, a, b)?;
        }
    }
    for (i, r) in members.iter().enumerate() {
        if !r.replace {
            continue;
        }
        let mut replaces = false;
        for (j, o) in members.iter().enumerate() {
            if i == j || layer(m, &o.version) > layer(m, &r.version) {
                continue;
            }
            let earlier =
                known(m, o)? < known(m, r)? || layer(m, &o.version) < layer(m, &r.version);
            if earlier && overlaps(m, o, r)? && !owners_differ(m, o, r) {
                replaces = true;
                break;
            }
        }
        if !replaces {
            let message = format!(
                "replace {id}{}: no earlier row to replace",
                pattern_text(&r.pattern)
            );
            return Err(at_row(m, r, Load::Replace, message)?);
        }
    }
    Ok(())
}

/// Two rows at the same keys (refcore's `check_overlap`, its loop over pairs). L §4.6
/// steps 2 and 4, §6.4.
fn pair(m: &Reality, id: &str, a: &Row, b: &Row) -> Result<(), Stop> {
    if !overlaps(m, a, b)? || owners_differ(m, a, b) {
        return Ok(());
    }
    let keys = keys_or_none(a);
    let neither = |reality: &Reality| -> Result<String, Stop> {
        Ok(format!(
            "rows {} and {} of {id} overlap at {keys}, neither replacing the other",
            row_loc(reality, a)?,
            row_loc(reality, b)?
        ))
    };
    let (la, lb) = (layer(m, &a.version), layer(m, &b.version));
    if la != lb {
        let higher = if la > lb { a } else { b };
        if higher.replace {
            return Ok(());
        }
        return Err(at_row(m, higher, Load::Overlap, neither(m)?)?);
    }
    let (ka, kb) = (known(m, a)?, known(m, b)?);
    if ka == kb && a.instant.is_none() && b.instant.is_none() && always(m, a)? != always(m, b)? {
        return Ok(());
    }
    if ka == kb {
        if a.version == b.version {
            let message = format!(
                "rows {} and {} of {id} overlap at {keys} in one version",
                row_loc(m, a)?,
                row_loc(m, b)?
            );
            return Err(at_row(m, b, Load::Overlap, message)?);
        }
        return Err(at_row(m, b, Load::Overlap, neither(m)?)?);
    }
    let later = if ka > kb { a } else { b };
    if later.replace {
        return Ok(());
    }
    Err(at_row(m, later, Load::Overlap, neither(m)?)?)
}

/// Rows of two patterns that one read could match, neither narrower, tie where their spans
/// overlap, or their instants are one (refcore's `check_overlap`, its ties between
/// patterns). L §4.6 steps 3 and 4.
fn ties_across(
    m: &Reality,
    id: &str,
    binds: &[BindInfo],
    rows: &[Row],
    a: &Group,
    b: &Group,
) -> Result<(), Stop> {
    if narrower(m, binds, &a.pattern, &b.pattern)
        || narrower(m, binds, &b.pattern, &a.pattern)
        || !patterns_meet(m, binds, &a.pattern, &b.pattern)
    {
        return Ok(());
    }
    for x in a.rows.iter().filter_map(|at| rows.get(*at)) {
        for y in b.rows.iter().filter_map(|at| rows.get(*at)) {
            if owners_differ(m, x, y) || layer(m, &x.version) != layer(m, &y.version) {
                continue;
            }
            if x.instant.is_some() || y.instant.is_some() {
                if same_instant(x, y) {
                    let message = format!(
                        "rows {} and {} of {id} overlap and tie",
                        row_loc(m, x)?,
                        row_loc(m, y)?
                    );
                    return Err(at_row(m, y, Load::Overlap, message)?);
                }
                continue;
            }
            let (Some(sx), Some(sy)) = (span(m, x)?, span(m, y)?) else {
                continue;
            };
            if sx.is_always() != sy.is_always() || !sx.overlaps(&sy) {
                continue;
            }
            let message = format!(
                "rows {} and {} of {id} overlap at {} and {} and tie",
                row_loc(m, x)?,
                row_loc(m, y)?,
                pattern_text(&x.pattern),
                pattern_text(&y.pattern)
            );
            return Err(at_row(m, y, Load::Overlap, message)?);
        }
    }
    Ok(())
}

/// Whether two rows are facts about one item from two members of a joint model, or from a
/// member and the joint model, read together rather than ordered (refcore's
/// `owners_differ`); a scenario's row replaces any. L §6.1, §6.4.
fn owners_differ(m: &Reality, a: &Row, b: &Row) -> bool {
    let scenario = |r: &Row| m.scenarios.of(&r.version).is_some();
    !scenario(a)
        && !scenario(b)
        && super::joint::owner_of(&a.version) != super::joint::owner_of(&b.version)
}

/// Whether two rows overlap: at one instant stated alike, or on spans that share a day
/// (refcore's `ov`). L §4.6 step 2.
fn overlaps(m: &Reality, one: &Row, other: &Row) -> Result<bool, Stop> {
    if one.instant.is_some() || other.instant.is_some() {
        return Ok(same_instant(one, other));
    }
    Ok(match (span(m, one)?, span(m, other)?) {
        (Some(first), Some(second)) => first.overlaps(&second),
        _ => false,
    })
}

/// Whether two readings state one instant: a day and its place in the day alike; an `after m`
/// is its own (refcore compares the instants' nodes by identity).
fn same_instant(a: &Row, b: &Row) -> bool {
    match (&a.instant, &b.instant) {
        (Some(RowInstant::At(x, ex)), Some(RowInstant::At(y, ey))) => x == y && ex == ey,
        _ => false,
    }
}

/// How a key stands to another at one position (refcore's `_pos_rel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rel {
    /// Stated alike.
    Alike,
    /// Strictly inside the other.
    Inside,
}

/// At one position, `a` stated as `b` is, or lying strictly inside it (refcore's `_pos_rel`).
/// L §4.6 steps 2 and 3.2.
fn pos_rel(m: &Reality, domain: &Domain, a: &Pat, b: &Pat) -> Option<Rel> {
    if a == b {
        return Some(Rel::Alike);
    }
    let (lb, hb) = match b {
        Pat::Star => return Some(Rel::Inside),
        Pat::Key(_) => return None,
        Pat::Range(lo, hi) => (lo, hi),
    };
    let (la, ha) = match a {
        Pat::Star => return None,
        Pat::Key(k) => (k, k),
        Pat::Range(lo, hi) => (lo, hi),
    };
    let ranks = [la, ha, lb, hb].map(|k| rank(m, domain, k, Compare::Containment));
    let [Some(ra), Some(rha), Some(rb), Some(rhb)] = ranks else {
        return None;
    };
    let low = rank_cmp(&rb, &ra)?;
    let high = rank_cmp(&rha, &rhb)?;
    (low != Ordering::Greater && high != Ordering::Greater).then_some(Rel::Inside)
}

/// Whether pattern `a` is narrower than `b`: at every position alike or inside, and at one
/// strictly inside (refcore's `narrower`). L §4.6 step 3.2.
fn narrower(m: &Reality, binds: &[BindInfo], a: &[Pat], b: &[Pat]) -> bool {
    let mut strict = false;
    for ((x, y), bind) in a.iter().zip(b).zip(binds) {
        match pos_rel(m, &bind.domain, x, y) {
            None => return false,
            Some(Rel::Inside) => strict = true,
            Some(Rel::Alike) => {}
        }
    }
    strict
}

/// Whether one read could match both patterns (refcore's `patterns_meet`). L §4.6 step 1.
fn patterns_meet(m: &Reality, binds: &[BindInfo], a: &[Pat], b: &[Pat]) -> bool {
    for ((x, y), bind) in a.iter().zip(b).zip(binds) {
        let ends = |p: &Pat| match p {
            Pat::Star => None,
            Pat::Key(k) => Some((k.clone(), k.clone())),
            Pat::Range(lo, hi) => Some((lo.clone(), hi.clone())),
        };
        let (Some((la, ha)), Some((lb, hb))) = (ends(x), ends(y)) else {
            continue;
        };
        if let (Pat::Key(kx), Pat::Key(ky)) = (x, y) {
            if kx != ky {
                return false;
            }
            continue;
        }
        let ranks = [&la, &ha, &lb, &hb].map(|k| rank(m, &bind.domain, k, Compare::Before));
        let [Some(ra), Some(rha), Some(rb), Some(rhb)] = ranks else {
            continue;
        };
        let before = |first: &Rank, second: &Rank| rank_cmp(first, second) == Some(Ordering::Less);
        if before(&rha, &rb) || before(&rhb, &ra) {
            return false;
        }
    }
    true
}

/// Which comparison a key's rank is for: a period of a range compares only by `<`, by its
/// first day (refcore's `Period.__lt__`), so it has no rank for containment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Compare {
    /// `<` only.
    Before,
    /// `<=` both ways.
    Containment,
}

/// A key's place in its domain's order (refcore's `key_order`): an index's position, a key
/// list's, a thing's stable id `kind:key`, a range's value.
fn rank(m: &Reality, domain: &Domain, key: &KeyVal, compare: Compare) -> Option<Rank> {
    match domain {
        Domain::Index(id) => m.indexes.get(id)?.pos(key).map(Rank::Pos),
        Domain::List(keys) => {
            let text = key.text();
            keys.iter().position(|k| k.text() == text).map(Rank::Pos)
        }
        Domain::Kind(_) => {
            let KeyVal::Thing(name) = key else {
                return None;
            };
            let base = m.things.get(name)?.base.as_deref()?;
            Some(Rank::Text(format!("{base}:{name}")))
        }
        Domain::Range => match key {
            KeyVal::Int(n) => Some(Rank::Int(*n)),
            KeyVal::Day(day) => Some(Rank::Day(*day)),
            KeyVal::Period(p) => (compare == Compare::Before).then(|| Rank::Day(p.start())),
            KeyVal::Key { .. } | KeyVal::Thing(_) | KeyVal::Mv(_) | KeyVal::Value(_) => None,
        },
        Domain::Axis(_) => None,
    }
}

/// A key's place in its order.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Rank {
    /// A position.
    Pos(usize),
    /// A stable id, by code point.
    Text(String),
    /// An integer.
    Int(i64),
    /// A day.
    Day(Day),
}

/// How two places in one order compare; none for two of different orders (refcore's
/// comparison of `key_order`s, a `TypeError` it catches).
fn rank_cmp(a: &Rank, b: &Rank) -> Option<Ordering> {
    match (a, b) {
        (Rank::Pos(x), Rank::Pos(y)) => Some(x.cmp(y)),
        (Rank::Text(x), Rank::Text(y)) => Some(x.as_str().cmp(y.as_str())),
        (Rank::Int(x), Rank::Int(y)) => Some(x.cmp(y)),
        (Rank::Day(x), Rank::Day(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

/// Every observed input's rows (refcore's `check_obs`): each id once, restated only by a
/// later row that says `replace` (a scenario's above its base's), and each observation of a
/// covered place dated inside the span its version covers. L §5.19, §4.6 step 2.
///
/// # Errors
///
/// `LOAD-REPLACE`, `LOAD-OVERLAP` or `LOAD-COVERS`, at the observation's row.
pub(super) fn check_obs(m: &Reality) -> Result<(), Stop> {
    for (qid, input) in m.inputs.entries() {
        if !input.observed {
            continue;
        }
        let mut keys: Vec<&str> = Vec::new();
        for o in &input.observations {
            if !keys.contains(&o.key.as_str()) {
                keys.push(&o.key);
            }
        }
        for key in keys {
            let mut rows = Vec::new();
            for o in input.observations.iter().filter(|o| o.key == key) {
                rows.push((layer(m, &o.version), obs_known(m, o)?, o));
            }
            rows.sort_by_key(|(l, k, _)| (*l, *k));
            one_id(m, qid, key, &rows)?;
        }
        for o in input.observations.iter().filter(|o| !o.removed) {
            covered(m, o)?;
        }
    }
    Ok(())
}

/// The rows of one observation id, by layer then `known`: the first replaces nothing, each
/// later one says `replace` and isn't known the day the one before it is (refcore's
/// `check_obs`, its loop over an id's rows). L §5.19.
fn one_id(
    m: &Reality,
    qid: &str,
    key: &str,
    rows: &[(usize, Day, &Observation)],
) -> Result<(), Stop> {
    let mut before: Option<&(usize, Day, &Observation)> = None;
    for row in rows {
        let (l, k, o) = row;
        let Some((pl, pk, prev)) = before else {
            if o.replace {
                let message = format!("replace {}: no earlier row to replace", o.sid());
                return Err(at_obs(m, o, Load::Replace, message)?);
            }
            before = Some(row);
            continue;
        };
        let locs = (obs_loc(m, prev)?, obs_loc(m, o)?);
        if k == pk && l == pl {
            let message = format!("rows {} and {} of {qid} overlap at [{key}]", locs.0, locs.1);
            return Err(at_obs(m, o, Load::Overlap, message)?);
        }
        if !o.replace {
            let message = format!(
                "rows {} and {} of {qid} overlap at [{key}], neither replacing the other",
                locs.0, locs.1
            );
            return Err(at_obs(m, o, Load::Overlap, message)?);
        }
        before = Some(row);
    }
    Ok(())
}

/// An observation's legs at a place its version covers are dated inside the span covered
/// (refcore's `check_obs`, its end). L §5.19, §5.3.
fn covered(m: &Reality, o: &Observation) -> Result<(), Stop> {
    let version = m
        .versions
        .get(&o.version)
        .ok_or_else(|| invariant(format!("version {} isn't built", o.version)))?;
    for (place, own) in &version.covers {
        let span = own.clone().or_else(|| version.span_at(m.on).cloned());
        for leg in &o.legs {
            if leg.from != *place && leg.to != *place {
                continue;
            }
            let Some(day) = leg.day.or(o.day) else {
                continue;
            };
            if span.as_ref().is_none_or(|s| !s.contains(day)) {
                let message = format!(
                    "observation {} of {place} is dated outside the span {} covers",
                    o.sid(),
                    version.id
                );
                return Err(at_obs(m, o, Load::Covers, message)?);
            }
        }
    }
    Ok(())
}

/// A row's pattern as a message prints it, or `no keys`.
fn keys_or_none(row: &Row) -> String {
    let text = pattern_text(&row.pattern);
    if text.is_empty() {
        "no keys".to_owned()
    } else {
        text
    }
}

/// A row's span: its own, else its version's in force at `on` (refcore's `Row.span`).
/// L §4.6 step 1.
fn span(m: &Reality, row: &Row) -> Result<Option<Span>, Stop> {
    if let Some(own) = &row.own_span {
        return Ok(Some(own.clone()));
    }
    let version = m
        .versions
        .get(&row.version)
        .ok_or_else(|| invariant(format!("version {} isn't built", row.version)))?;
    Ok(version.span_at(m.on).cloned())
}

/// Whether a row governs `always` (refcore's `check_overlap`, its precedence of a span over
/// `always`). L §4.6 step 2.
fn always(m: &Reality, row: &Row) -> Result<bool, Stop> {
    Ok(span(m, row)?.is_some_and(|s| s.is_always()))
}

/// A row's `known`: its own, else its version's, a window by its first day. L §4.5.
fn known(m: &Reality, row: &Row) -> Result<Day, Stop> {
    if let Some(own) = row.own_known {
        return Ok(own);
    }
    version_known(m, &row.version)
}

/// An observation's `known`: its own, else its version's. L §4.5.
fn obs_known(m: &Reality, o: &Observation) -> Result<Day, Stop> {
    if let Some(own) = o.own_known {
        return Ok(own);
    }
    version_known(m, &o.version)
}

/// A version's `known`, or its window's first day.
fn version_known(m: &Reality, id: &str) -> Result<Day, Stop> {
    m.versions
        .get(id)
        .map(|v| v.known)
        .ok_or_else(|| invariant(format!("version {id} isn't built")))
}

/// A version's layer in its stack of scenarios (refcore's `version_layer`). L §4.6 step 3.1,
/// §6.4.
fn layer(m: &Reality, version: &str) -> usize {
    m.scenarios.layer(Some(version))
}

/// The file and line a version's item stands at.
fn place(m: &Reality, version: &str, line: u32) -> Result<(String, u32), Stop> {
    m.versions
        .get(version)
        .map(|v| (v.file.clone(), line))
        .ok_or_else(|| invariant(format!("version {version} isn't built")))
}

/// A row as a message names it, `base.tally:43` (refcore's `Row.loc`).
fn row_loc(m: &Reality, row: &Row) -> Result<String, Stop> {
    let (file, line) = place(m, &row.version, row.line)?;
    Ok(format!("{file}:{line}"))
}

/// An observation as a message names it, `base.tally:55`.
fn obs_loc(m: &Reality, o: &Observation) -> Result<String, Stop> {
    let (file, line) = place(m, &o.version, o.line)?;
    Ok(format!("{file}:{line}"))
}

/// A refusal at a row's line.
fn at_row(m: &Reality, row: &Row, code: Load, message: String) -> Result<Stop, Stop> {
    let (file, line) = place(m, &row.version, row.line)?;
    Ok(refuse(code, message, &file, line))
}

/// A refusal at an observation's line.
fn at_obs(m: &Reality, o: &Observation, code: Load, message: String) -> Result<Stop, Stop> {
    let (file, line) = place(m, &o.version, o.line)?;
    Ok(refuse(code, message, &file, line))
}
