//! Load's version items: `covers` and gap roles, rows of inputs and `given`s of defs, band rows, and replaced rules.
//! Spec: L §4.5 (a row's own `known`), §4.6 (rows: pattern, `*`, key ranges, own `governs`,
//! readings' instants), §5.3 (`covers`, `pad`, `anchor`, LOAD-GOVERNS), §5.9 (band tables),
//! §5.11 step 2 (`replace rule`), §2.5 (stable ids).
//! Twin: engine/tools/refcore/loader.py (`build_items`, `target`, `rule_item`, `band_item`,
//! `band_end`, `row_item`, `check_rows`' stable ids).
//! Invariants: a row's pattern has one position per key bind of its target; a band's table is
//! one per (version, key); a row's stable id is `version:target[keys]`, its own span or instant
//! added where two rows of a version share a pattern.

use super::keys::{KeyLit, RangeKey, key_order, not_a_range_value, range_key_of};
use super::places::Written;
use super::same::SameLine;
use super::spans::{governs_span, lspan_to_span};
use super::steps::{Loader, refuse};
use super::worlds::no_version;
use crate::base::Load;
use crate::model::{
    BandRow, BandTable, Domain, GapRole, IndexForm, KeyVal, NameKind, Pat, Reality, Row,
    RowInstant, RuleVer, Ty, pattern_text,
};
use crate::syntax::{
    BandRow as WrittenBand, Domain as WrittenDomain, Expr, Gap, Instant, Item, ItemForm, Known,
    Lit, Row as WrittenRow, RowKey, Rule, Section, Stop,
};
use crate::time::Day;
use std::collections::BTreeMap;

impl Loader {
    /// Each version's covers, gap roles and items (refcore's `build_items`). L §5.3.
    pub(super) fn build_items(&mut self) -> Result<(), Stop> {
        let sections = core::mem::take(&mut self.sections);
        let built = self.items_all(&sections);
        self.sections = sections;
        built
    }

    /// The items of every version, in section order.
    fn items_all(&mut self, sections: &[(String, Section)]) -> Result<(), Stop> {
        for (file, section) in sections {
            let Section::Version(version) = section else {
                continue;
            };
            let vid = &version.id;
            let line = version.line;
            let has_governs = self
                .m
                .versions
                .get(vid)
                .is_some_and(|v| v.governs.is_some());
            let mut covers = Vec::new();
            for (place, span) in &version.covers {
                let thing = self.place_ref(place, Written::Covers, file, line)?;
                let own = span
                    .as_ref()
                    .map(|s| lspan_to_span(s, file, line))
                    .transpose()?;
                covers.push((thing, own));
            }
            if !has_governs && covers.iter().any(|(_, s)| s.is_none()) {
                return Err(refuse(
                    Load::Governs,
                    format!("version {vid} covers a place for its governs span and has no governs"),
                    file,
                    line,
                ));
            }
            let mut roles: Vec<(String, GapRole)> = Vec::new();
            for gap in &version.gaps {
                let (place, role) = match gap {
                    Gap::Pad(x, y) => (
                        self.place_ref(x, Written::Place, file, line)?,
                        GapRole::Pad(self.place_ref(y, Written::Place, file, line)?),
                    ),
                    Gap::Anchor(x) => (
                        self.place_ref(x, Written::Place, file, line)?,
                        GapRole::Anchor,
                    ),
                };
                if roles.iter().any(|(p, _)| *p == place) {
                    return Err(refuse(
                        Load::GapRole,
                        format!("{vid} states two gap roles for {place}"),
                        file,
                        line,
                    ));
                }
                roles.push((place, role));
            }
            let mut needs_governs = false;
            for item in &version.items {
                needs_governs |= self.item(vid, item, file)?;
            }
            if let Some(info) = self.m.versions.get_mut(vid) {
                info.covers = covers;
                info.gap_roles.clone_from(&roles);
            }
            self.check_gap_roles(vid, &roles, file)?;
            if needs_governs && !has_governs {
                return Err(refuse(
                    Load::Governs,
                    format!("version {vid} has rows with no time of their own and no governs"),
                    file,
                    line,
                ));
            }
        }
        Ok(())
    }

    /// A version's balance readings name a place it states a pad or an anchor for. L §5.3.
    fn check_gap_roles(
        &self,
        vid: &str,
        roles: &[(String, GapRole)],
        file: &str,
    ) -> Result<(), Stop> {
        let Some(stated) = self.m.inputs.get("stated") else {
            return Ok(());
        };
        for row in stated.rows.iter().filter(|r| r.version == vid) {
            let Some(Pat::Key(KeyVal::Thing(name))) = row.pattern.first() else {
                continue;
            };
            if !roles.iter().any(|(p, _)| p == name) {
                return Err(refuse(
                    Load::GapRole,
                    format!("{vid} states balance readings of {name} with neither pad nor anchor"),
                    file,
                    row.line,
                ));
            }
        }
        Ok(())
    }

    /// One item of a version; whether it needs the version's `governs` (it states no time of
    /// its own). L §5.3.
    fn item(&mut self, vid: &str, item: &Item, file: &str) -> Result<bool, Stop> {
        match &item.form {
            ItemForm::Enter(_) => Ok(false),
            ItemForm::Same(sides) => {
                let (left, right) = sides.as_ref();
                self.same_lines.push(SameLine {
                    left: left.clone(),
                    right: right.clone(),
                    file: file.to_owned(),
                    line: item.line,
                    version: Some(vid.to_owned()),
                });
                Ok(false)
            }
            ItemForm::Rule { qual, def, rule } => {
                unprefixed(qual.as_deref(), def, file, item.line)?;
                self.rule_item(vid, def, rule.as_ref(), file, item.line)?;
                Ok(rule.governs.is_none())
            }
            ItemForm::Band(row) => {
                self.band_item(vid, row.as_ref(), file, item.line)?;
                Ok(true)
            }
            ItemForm::Row(row) => Ok(!self.row_item(vid, row.as_ref(), file, item.line)?),
        }
    }

    /// The input or def a row names (refcore's `target`; its prefix, if any, in `unprefixed`).
    fn target_of(&self, name: &str, want: NameKind, file: &str, line: u32) -> Result<(), Stop> {
        let word = if want == NameKind::Def {
            "def"
        } else {
            "input"
        };
        match self.m.names.kind(name) {
            Some(kind) if kind == want => Ok(()),
            Some(_) => Err(refuse(
                Load::Name,
                format!("{name} is not {word}"),
                file,
                line,
            )),
            None => Err(refuse(Load::Name, format!("unknown id {name}"), file, line).naming(name)),
        }
    }

    /// `replace rule d.l …`: a rule's later version (refcore's `rule_item`). L §5.11 step 2.
    fn rule_item(
        &mut self,
        vid: &str,
        def: &str,
        rule: &Rule,
        file: &str,
        line: u32,
    ) -> Result<(), Stop> {
        self.target_of(def, NameKind::Def, file, line)?;
        let label = &rule.label;
        if let Some(v) = &rule.v
            && v != vid
        {
            let message = if self.m.versions.contains(v) {
                (
                    Load::Version,
                    format!("replace rule {def}.{label} in {vid} names version {v}"),
                )
            } else {
                let message = format!("rule {def}.{label}: v {v} names no version");
                no_version(&self.m, def, v, message)
            };
            return Err(refuse(message.0, message.1, file, line));
        }
        let span = rule
            .governs
            .as_ref()
            .map(|g| lspan_to_span(g, file, line))
            .transpose()?;
        let ver = RuleVer {
            label: label.clone(),
            rule: rule.clone(),
            version: Some(vid.to_owned()),
            v: None,
            span,
            replace: true,
        };
        let info = self.m.defs.at_mut(self.m.names.def_id(def)?)?;
        if let Some((_, vers)) = info.rules.iter_mut().find(|(l, _)| l == label) {
            vers.push(ver);
        } else {
            info.rules.push((label.clone(), vec![ver]));
        }
        Ok(())
    }

    /// A band row, into its version's table for its key (refcore's `band_item`). L §5.9.
    fn band_item(
        &mut self,
        vid: &str,
        row: &WrittenBand,
        file: &str,
        line: u32,
    ) -> Result<(), Stop> {
        unprefixed(row.qual.as_deref(), &row.target, file, line)?;
        let id = &row.target;
        match self.m.names.kind(id) {
            Some(NameKind::Band) => {}
            Some(_) => {
                return Err(refuse(Load::Name, format!("{id} is not band"), file, line));
            }
            None => {
                return Err(refuse(Load::Name, format!("unknown id {id}"), file, line).naming(id));
            }
        }
        let band_id = self.m.names.band_id(id)?;
        let band = self.m.bands.at(band_id)?;
        let first_bind = band.binds.first().cloned();
        let arg_type = band.arg_type.clone();
        let key = match (&row.key, first_bind) {
            (Some(written), Some(bind)) => {
                let lit = KeyLit::Lit(written.clone());
                Some(self.pattern_key(&lit, &bind, None, file, line)?)
            }
            (Some(_), None) => {
                return Err(refuse(
                    Load::Key,
                    format!("band {id} has no key"),
                    file,
                    line,
                ));
            }
            (None, Some(_)) => {
                return Err(refuse(
                    Load::Key,
                    format!("wrong number of keys: band {id} is keyed"),
                    file,
                    line,
                ));
            }
            (None, None) => None,
        };
        let interval = &row.interval;
        let lo = interval
            .lo
            .as_ref()
            .map(|l| self.band_end(l, id, &arg_type, file, line))
            .transpose()?;
        let hi = interval
            .hi
            .as_ref()
            .map(|l| self.band_end(l, id, &arg_type, file, line))
            .transpose()?;
        let written = BandRow {
            lo,
            lo_closed: interval.lo_closed,
            hi,
            hi_closed: interval.hi_closed,
            value: row.value.clone(),
            locator: row.locator.clone(),
            replace: row.replace,
            line,
        };
        let held = self.m.bands.at_mut(band_id)?;
        if let Some(table) = held
            .tables
            .iter_mut()
            .find(|t| t.version == vid && t.key == key)
        {
            table.rows.push(written);
        } else {
            held.tables.push(BandTable {
                version: vid.to_owned(),
                key,
                rows: vec![written],
            });
        }
        Ok(())
    }

    /// A row of an input, or a `given` of a def (refcore's `row_item`); whether it states a
    /// time of its own. L §4.6.
    fn row_item(
        &mut self,
        vid: &str,
        row: &WrittenRow,
        file: &str,
        line: u32,
    ) -> Result<bool, Stop> {
        unprefixed(row.qual.as_deref(), &row.target, file, line)?;
        let id = &row.target;
        let want = if row.given {
            NameKind::Def
        } else {
            NameKind::Input
        };
        self.target_of(id, want, file, line)?;
        let balance_input = !row.given
            && (id == "stated" || id == "opening")
            && self.m.inputs.get(id).is_some_and(|input| input.builtin);
        if row.of.is_some() && !balance_input {
            // L §2.3 row (tally 2.1): `of` only on a row of `opening` or `stated` (§5.19).
            return Err(refuse(
                Load::Parts,
                format!("of on a row of {id}"),
                file,
                line,
            ));
        }
        let (binds, reading, balance) = if row.given {
            let def = self.m.defs.at(self.m.names.def_id(id)?)?;
            (def.binds.clone(), false, false)
        } else {
            let input = self.m.inputs.at(self.m.names.input_id(id)?)?;
            if input.observed {
                return self.obs_item(vid, id, row, file, line);
            }
            let balance = input.builtin && (id == "stated" || id == "opening");
            (input.binds.clone(), input.reading, balance)
        };
        let (pattern, stated) = self.row_pattern(vid, id, row, &binds, file, line)?;
        let own_span = row
            .governs
            .as_ref()
            .map(|g| governs_span(g, file, line))
            .transpose()?;
        let (own_known, own_window) = self.row_known(vid, id, &pattern, row, file, line)?;
        let read_as = if balance {
            ReadAs::Balance
        } else if reading {
            ReadAs::Reading
        } else {
            ReadAs::Plain
        };
        let instant = row_instant(id, &pattern, row.instant.as_ref(), read_as, (file, line))?;
        if instant.is_some() && own_span.is_some() {
            return Err(refuse(
                Load::Parse,
                "cannot parse: a reading states an instant, not governs".to_owned(),
                file,
                line,
            ));
        }
        let own_time = own_span.is_some() || instant.is_some();
        let built = Row {
            target: id.clone(),
            pattern,
            stated,
            value: row.value.clone(),
            assumed: row.assumed.clone(),
            version: vid.to_owned(),
            own_span,
            own_known,
            own_window,
            instant,
            locator: row.locator.clone(),
            given: row.given,
            replace: row.replace,
            line,
            sid: String::new(),
            of: row.of.clone(),
        };
        if row.given {
            if let Some(def) = self.m.defs.get_mut(id) {
                def.givens.push(built);
            }
        } else if let Some(input) = self.m.inputs.get_mut(id) {
            // A reading's stated parts are kept beside its readings, never a total (L §5.19).
            if built.of.is_some() {
                input.part_rows.push(built);
            } else {
                input.rows.push(built);
            }
        }
        Ok(own_time)
    }

    /// A row's pattern, one position per key bind, and how many it states. L §4.6 step 1.
    fn row_pattern(
        &mut self,
        vid: &str,
        id: &str,
        row: &WrittenRow,
        binds: &[crate::model::BindInfo],
        file: &str,
        line: u32,
    ) -> Result<(Vec<Pat>, usize), Stop> {
        let mut pattern = Vec::new();
        let mut stated: usize = 0;
        let keys: &[RowKey] = row.keys.as_deref().unwrap_or(&[]);
        if keys.len() > binds.len() {
            return Err(refuse(
                Load::Key,
                format!("wrong number of keys for {id}"),
                file,
                line,
            ));
        }
        for (key, bind) in keys.iter().zip(binds) {
            let pat = match key {
                RowKey::Star => Pat::Star,
                RowKey::Range(a, b) => {
                    let unordered = match &bind.domain {
                        Domain::Kind(kind) => Some(format!("the kind {kind}")),
                        Domain::Index(ix) => self
                            .m
                            .indexes
                            .get(ix)
                            .filter(|i| i.form == IndexForm::Open)
                            .map(|_| format!("the open index {ix}")),
                        Domain::Axis(_) | Domain::List(_) | Domain::Range => None,
                    };
                    if let Some(what) = unordered {
                        return Err(refuse(
                            Load::Unit,
                            format!("a key range over {what}: it has no order"),
                            file,
                            line,
                        ));
                    }
                    let lo = self.pattern_key(&KeyLit::Lit(a.clone()), bind, None, file, line)?;
                    let hi = self.pattern_key(&KeyLit::Lit(b.clone()), bind, None, file, line)?;
                    if matches!((&lo, &hi), (KeyVal::Value(_), _) | (_, KeyVal::Value(_)))
                        || !whole_steps(&self.m, &bind.node.domain)
                    {
                        // refcore stops on it (2.1.6: `Num` has no order)
                        return Err(Stop::NotBuilt(
                            "a row's key range over a stepped range's values not integers",
                        ));
                    }
                    if key_order(&self.m, &lo, &bind.domain) > key_order(&self.m, &hi, &bind.domain)
                    {
                        return Err(refuse(
                            Load::Key,
                            format!("range {}..{} is out of order", lo.text(), hi.text()),
                            file,
                            line,
                        ));
                    }
                    Pat::Range(lo, hi)
                }
                RowKey::Lit(lit) => {
                    let written = KeyLit::Lit(lit.clone());
                    if bind.domain == Domain::Range {
                        let value = range_key_of(&self.m, &written, file, line)?;
                        self.range_keys.extend(RangeKey::of(bind, lit, file, line));
                        Pat::Key(value)
                    } else {
                        Pat::Key(self.pattern_key(&written, bind, Some(vid), file, line)?)
                    }
                }
                RowKey::Place(name, place_keys) => {
                    let written = KeyLit::Place(name.clone(), place_keys.clone());
                    if bind.domain == Domain::Range {
                        return Err(not_a_range_value(&written, file, line));
                    }
                    Pat::Key(self.pattern_key(&written, bind, Some(vid), file, line)?)
                }
            };
            if pat != Pat::Star {
                stated = stated
                    .checked_add(1)
                    .ok_or(Stop::NotBuilt("too many keys"))?;
            }
            pattern.push(pat);
        }
        while pattern.len() < binds.len() {
            pattern.push(Pat::Star);
        }
        Ok((pattern, stated))
    }

    /// A row's own `known`: its day, or its window's first and last days; not before its
    /// version's, not after `on` (refcore's `row_item`). L §4.5.
    fn row_known(
        &self,
        vid: &str,
        id: &str,
        pattern: &[Pat],
        row: &WrittenRow,
        file: &str,
        line: u32,
    ) -> Result<(Option<Day>, Option<Day>), Stop> {
        let Some(known) = &row.known else {
            return Ok((None, None));
        };
        let (day, window) = match known {
            Known::Day(day) => (*day, None),
            Known::Window(a, b) => (*a, Some(*b)),
        };
        let shown = format!("{id}{}", load_pattern_text(pattern));
        let version_known = self.m.versions.get(vid).map(|v| v.known);
        if version_known.is_some_and(|k| day < k) {
            return Err(refuse(
                Load::Known,
                format!("row {shown} is known before its version {vid}"),
                file,
                line,
            ));
        }
        if window.unwrap_or(day) > self.m.on {
            return Err(refuse(
                Load::Known,
                format!("row {shown} is known after the model's on date"),
                file,
                line,
            ));
        }
        Ok((Some(day), window))
    }

    /// Each row's stable id, `version:target[keys]`, with its own span or instant where two
    /// rows of a version share a pattern (refcore's `check_rows`, its end). L §2.5.
    pub(super) fn finish_rows(&mut self) {
        for (_, input) in self.m.inputs.entries_mut() {
            if !input.observed {
                set_sids(&input.id, &mut input.rows);
            }
        }
        for (_, def) in self.m.defs.entries_mut() {
            set_sids(&def.id, &mut def.givens);
        }
    }
}

/// How an input's rows are read: at an instant, or over spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadAs {
    /// Over spans: no instant.
    Plain,
    /// A reading, at an instant.
    Reading,
    /// A balance reading (`opening`, `stated`), at the start or end of a day, or after a move.
    Balance,
}

/// A row's instant: only a reading's rows have one, and they must; a reading's row with
/// none is named with its keys (L §4.6 Readings, §5.19, §8.3).
fn row_instant(
    id: &str,
    pattern: &[Pat],
    written: Option<&Instant>,
    read_as: ReadAs,
    (file, line): (&str, u32),
) -> Result<Option<RowInstant>, Stop> {
    let Some(instant) = written else {
        if read_as == ReadAs::Plain {
            return Ok(None);
        }
        return Err(refuse(
            Load::Instant,
            format!(
                "row {id}{} of a reading has no instant",
                load_pattern_text(pattern)
            ),
            file,
            line,
        ));
    };
    if read_as == ReadAs::Plain {
        return Err(refuse(
            Load::Instant,
            format!("{id} is not a reading; its rows take no instant"),
            file,
            line,
        ));
    }
    match instant {
        Instant::After(expr) => Ok(Some(RowInstant::After(expr.clone()))),
        Instant::At(day, edge) => {
            if read_as == ReadAs::Balance && edge.is_none() {
                return Err(refuse(
                    Load::Instant,
                    format!(
                        "balance reading {id} at {} needs start, end or after",
                        day.text()
                    ),
                    file,
                    line,
                ));
            }
            Ok(Some(RowInstant::At(*day, *edge)))
        }
    }
}

/// The stable ids of one target's rows: `version:target[keys]`, with the row's own `governs`
/// span or its instant where its version holds more than one row of that target at those
/// keys (refcore's loader, its stable ids). L §2.5, §4.6.
fn set_sids(id: &str, rows: &mut [Row]) {
    let mut count: BTreeMap<(String, String), usize> = BTreeMap::new();
    for row in rows.iter() {
        let slot = count
            .entry((row.version.clone(), pattern_text(&row.pattern)))
            .or_insert(0);
        if let Some(more) = slot.checked_add(1) {
            *slot = more;
        }
    }
    for row in rows.iter_mut() {
        let keys = pattern_text(&row.pattern);
        let mut sid = format!("{}:{id}{keys}", row.version);
        let shared = count
            .get(&(row.version.clone(), keys))
            .is_some_and(|n| *n > 1);
        if shared {
            if let Some(span) = &row.own_span {
                sid.push(' ');
                sid.push_str(&span.text());
            } else if let Some(RowInstant::At(day, edge)) = &row.instant {
                // its instant, a day's start or end (L §4.6): two readings of one day are two
                // rows (refcore's id gives the day alone)
                sid.push(' ');
                sid.push_str(&match edge {
                    Some(edge) => crate::book::instant_text(*day, *edge),
                    None => day.text(),
                });
            }
        }
        row.sid = sid;
    }
}

/// Whether a bind's values are days, periods or integers, so a row's key range orders them
/// as keys (L §4.6 step 1, §5.6): any domain but a range; a range whose first end is a day,
/// a month, a quarter or the read of a `date` or period item, whatever its step; a range
/// whose first end and step (if it has one) are whole: a whole figure of no unit, negated
/// or not, or the read of an `int` item. Any other first end or step (a fraction, a unit,
/// an expression, a `num` read) may give fractions at run, which a key range doesn't order.
fn whole_steps(m: &Reality, domain: &WrittenDomain) -> bool {
    let WrittenDomain::Range(parts) = domain else {
        return true;
    };
    let (first, _, step) = parts.as_ref();
    if dated(m, first) {
        return true;
    }
    whole(m, first) && step.as_ref().is_none_or(|s| whole(m, s))
}

/// Whether a range's first end is a day or a period as written: a literal, or the read of
/// an item of that type. L §5.6.
fn dated(m: &Reality, e: &Expr) -> bool {
    if matches!(
        e,
        Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..))
    ) {
        return true;
    }
    let Expr::Name(name) = e else {
        return false;
    };
    matches!(declared(m, name), Some(Ty::Date | Ty::Period(_)))
}

/// Whether a range's end or step is whole as written: a whole figure of no unit, negated or
/// bracketed, or the read of an `int` item. L §3.2, §5.6.
fn whole(m: &Reality, e: &Expr) -> bool {
    let mut node = e;
    while let Expr::Neg(inner) | Expr::Paren(inner) = node {
        node = inner;
    }
    if matches!(node, Expr::Lit(Lit::Num(n)) if n.whole && n.unit.is_none()) {
        return true;
    }
    let Expr::Name(name) = node else {
        return false;
    };
    matches!(declared(m, name), Some(Ty::Int))
}

/// The declared type of input or def `name`.
fn declared(m: &Reality, name: &str) -> Option<Ty> {
    if let Some(input) = m.inputs.get(name) {
        return input.ty.clone();
    }
    m.defs.get(name).map(|def| def.ty.clone())
}

/// A row's pattern as load's messages print it: every position, trailing stars too, and
/// nothing when every position is a star (refcore's loader `pattern_text`).
fn load_pattern_text(pattern: &[Pat]) -> String {
    if pattern.iter().all(|p| *p == Pat::Star) {
        return String::new();
    }
    let parts: Vec<String> = pattern
        .iter()
        .map(|p| match p {
            Pat::Key(key) => key.text(),
            Pat::Star => "*".to_owned(),
            Pat::Range(a, b) => format!("{}..{}", a.text(), b.text()),
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

/// A row's, band row's or rule's target written with a prefix, `a.x`, is an unknown id: a
/// joint model's merge has made each member's item one name, so no prefix is left that names
/// one (refcore's `target`, whose `qualified` names nothing since GD-1). L §6.1, §2.5.
fn unprefixed(qual: Option<&str>, name: &str, file: &str, line: u32) -> Result<(), Stop> {
    match qual {
        Some(prefix) => {
            let id = format!("{prefix}.{name}");
            Err(refuse(Load::Name, format!("unknown id {id}"), file, line).naming(&id))
        }
        None => Ok(()),
    }
}
