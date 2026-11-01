//! Load's version items: `covers` and gap roles, rows of inputs and `given`s of defs, band rows, and replaced rules.
//! Spec: L §4.5 (a row's own `known`), §4.6 (rows: pattern, `*`, key ranges, own `governs`,
//! readings' instants), §5.3 (`covers`, `pad`, `anchor`, LOAD-GOVERNS), §5.9 (band tables),
//! §5.11 step 2 (`replace rule`), §2.5 (stable ids).
//! Twin: engine/tools/refcore/loader.py (`build_items`, `target`, `rule_item`, `band_item`,
//! `band_end`, `row_item`, `check_rows`' stable ids).
//! Invariants: a row's pattern has one position per key bind of its target; a band's table is
//! one per (version, key); a row's stable id is `version:target[keys]`, its own span or instant
//! added where two rows of a version share a pattern.

use super::keys::{KeyLit, key_order, range_key_of};
use super::places::Written;
use super::spans::{governs_span, lspan_to_span};
use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{
    BandEnd, BandRow, BandTable, Domain, GapRole, IndexForm, KeyVal, NameKind, Pat, Row,
    RowInstant, RuleVer, pattern_text,
};
use crate::syntax::{
    BandRow as WrittenBand, Gap, Instant, Item, ItemForm, Known, Lit, Row as WrittenRow, RowKey,
    Rule, Section, Stop,
};
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
            ItemForm::Same(_) => Err(Stop::NotBuilt("same lines")),
            ItemForm::Rule { qual, def, rule } => {
                if qual.is_some() {
                    return Err(Stop::NotBuilt("a member's prefix"));
                }
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

    /// The input or def a row names (refcore's `target`).
    fn target_of(&self, name: &str, want: NameKind, file: &str, line: u32) -> Result<(), Stop> {
        let word = if want == NameKind::Def {
            "def"
        } else {
            "input"
        };
        match self.m.names.get(name) {
            Some(kind) if *kind == want => Ok(()),
            Some(_) => Err(refuse(
                Load::Name,
                format!("{name} is not {word}"),
                file,
                line,
            )),
            None => Err(refuse(Load::Name, format!("unknown id {name}"), file, line)),
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
                (
                    Load::Name,
                    format!("rule {def}.{label}: v {v} names no version"),
                )
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
        let Some(info) = self.m.defs.get_mut(def) else {
            return Err(Stop::NotBuilt("a def that isn't built"));
        };
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
        if row.qual.is_some() {
            return Err(Stop::NotBuilt("a member's prefix"));
        }
        let id = &row.target;
        match self.m.names.get(id) {
            Some(NameKind::Band) => {}
            Some(_) => {
                return Err(refuse(Load::Name, format!("{id} is not band"), file, line));
            }
            None => {
                return Err(refuse(Load::Name, format!("unknown id {id}"), file, line));
            }
        }
        let Some(band) = self.m.bands.get(id) else {
            return Err(Stop::NotBuilt("a band that isn't built"));
        };
        let first_bind = band.binds.first().cloned();
        let arg_binds = band.binds.clone();
        let key = match (&row.key, first_bind) {
            (Some(written), Some(bind)) => {
                let Some(lit) = KeyLit::of_row_key(written) else {
                    return Err(Stop::NotBuilt("a band key that isn't one key"));
                };
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
            .map(|l| self.band_end(l, &arg_binds, file, line))
            .transpose()?;
        let hi = interval
            .hi
            .as_ref()
            .map(|l| self.band_end(l, &arg_binds, file, line))
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
        let Some(held) = self.m.bands.get_mut(id) else {
            return Err(Stop::NotBuilt("a band that isn't built"));
        };
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

    /// A band row's end: a number in the argument's unit (refcore's `band_end`). L §5.9.
    fn band_end(
        &self,
        lit: &Lit,
        binds: &[crate::model::BindInfo],
        file: &str,
        line: u32,
    ) -> Result<BandEnd, Stop> {
        let Lit::Num(n) = lit else {
            return Err(Stop::NotBuilt("a band over values that aren't numbers"));
        };
        let (_, factor) = self.unit_of(n.unit.as_ref(), binds, file, line)?;
        let value = n
            .value
            .checked_mul(&factor)
            .map_err(super::steps::num_stop)?;
        Ok(BandEnd::Num(value))
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
        if row.qual.is_some() {
            return Err(Stop::NotBuilt("a member's prefix"));
        }
        let id = &row.target;
        let want = if row.given {
            NameKind::Def
        } else {
            NameKind::Input
        };
        self.target_of(id, want, file, line)?;
        let (binds, reading, balance) = if row.given {
            let Some(def) = self.m.defs.get(id) else {
                return Err(Stop::NotBuilt("a def that isn't built"));
            };
            (def.binds.clone(), false, false)
        } else {
            let Some(input) = self.m.inputs.get(id) else {
                return Err(Stop::NotBuilt("an input that isn't built"));
            };
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
        let own_known = self.row_known(vid, id, &pattern, row, file, line)?;
        let read_as = if balance {
            ReadAs::Balance
        } else if reading {
            ReadAs::Reading
        } else {
            ReadAs::Plain
        };
        let instant = row_instant(id, row.instant.as_ref(), read_as, file, line)?;
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
            instant,
            locator: row.locator.clone(),
            given: row.given,
            replace: row.replace,
            line,
            sid: String::new(),
        };
        if row.given {
            if let Some(def) = self.m.defs.get_mut(id) {
                def.givens.push(built);
            }
        } else if let Some(input) = self.m.inputs.get_mut(id) {
            input.rows.push(built);
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
                RowKey::Lit(_) | RowKey::Place(..) => {
                    let Some(written) = KeyLit::of_row_key(key) else {
                        return Err(Stop::NotBuilt("a row key of that form"));
                    };
                    let value = if bind.domain == Domain::Range {
                        range_key_of(&written)?
                    } else {
                        self.pattern_key(&written, bind, Some(vid), file, line)?
                    };
                    Pat::Key(value)
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

    /// A row's own `known`: not before its version's, not after `on`. L §4.5.
    fn row_known(
        &self,
        vid: &str,
        id: &str,
        pattern: &[Pat],
        row: &WrittenRow,
        file: &str,
        line: u32,
    ) -> Result<Option<crate::time::Day>, Stop> {
        let Some(known) = &row.known else {
            return Ok(None);
        };
        let Known::Day(day) = known else {
            return Err(Stop::NotBuilt("a known window"));
        };
        let shown = format!("{id}{}", load_pattern_text(pattern));
        let version_known = self.m.versions.get(vid).map(|v| v.known);
        if version_known.is_some_and(|k| *day < k) {
            return Err(refuse(
                Load::Known,
                format!("row {shown} is known before its version {vid}"),
                file,
                line,
            ));
        }
        if *day > self.m.on {
            return Err(refuse(
                Load::Known,
                format!("row {shown} is known after the model's on date"),
                file,
                line,
            ));
        }
        Ok(Some(*day))
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

/// A row's instant: only a reading's rows have one, and they must (L §4.6 Readings, §5.19).
fn row_instant(
    id: &str,
    written: Option<&Instant>,
    read_as: ReadAs,
    file: &str,
    line: u32,
) -> Result<Option<RowInstant>, Stop> {
    let Some(instant) = written else {
        if read_as == ReadAs::Plain {
            return Ok(None);
        }
        return Err(refuse(
            Load::Instant,
            format!("row {id} of a reading has no instant"),
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

/// The stable ids of one target's rows.
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
            } else if let Some(RowInstant::At(day, _)) = &row.instant {
                sid.push(' ');
                sid.push_str(&day.text());
            }
        }
        row.sid = sid;
    }
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
