//! Load's kinds and things: the prelude, the declared kinds and subkinds, and the things versions enter.
//! Spec: L §5.4 (currencies are things of `currencies`), §5.7 (kinds, subkinds, entries,
//! lifespans, base kinds, kinds of unit); E §3.1 step 2.
//! Twin: engine/tools/refcore/loader.py (`build_kinds`, `build_entries`, `entry`).
//! Invariants: a subkind's spec is its root's and an event subkind's certainty its nearest
//! stated one; children are sorted; entries are read in versions' known order, ties in section
//! order; every thing has a base kind once this step ends.

use super::keys::key_source_text;
use super::spans::lspan_to_span;
use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{Entry, IndexForm, KeyVal, KindInfo, KindSpec, NameKind, Span, Thing, prelude};
use crate::syntax::{Enter, ItemForm, KindSpec as Written, Section, Stop};
use crate::time::Day;
use std::collections::BTreeSet;

impl Loader {
    /// The prelude's kinds, then the declared ones; subkinds, their specs and certainties
    /// (refcore's `build_kinds`). L §5.7.
    pub(super) fn build_kinds(&mut self) -> Result<(), Stop> {
        for kind in prelude() {
            let id = kind.id.clone();
            self.m.kinds.push(&id, kind);
        }
        let mut subs: BTreeSet<String> = BTreeSet::new();
        for (file, section) in &self.sections {
            let Section::Kind(decl) = section else {
                continue;
            };
            let mut info = match &decl.spec {
                Written::Plain => KindInfo::new(&decl.id, KindSpec::Plain, None),
                Written::Unit => KindInfo::new(&decl.id, KindSpec::Unit, None),
                Written::Event(certainty) => {
                    let mut event = KindInfo::new(&decl.id, KindSpec::Event, Some("events"));
                    event.certainty = Some(*certainty);
                    event
                }
                Written::Place {
                    unit,
                    sign,
                    priced,
                    outside,
                } => {
                    let mut place = KindInfo::new(&decl.id, KindSpec::Place, Some("places"));
                    place.unit = Some(unit.clone());
                    place.sign = Some(*sign);
                    place.priced.clone_from(priced);
                    place.outside = *outside;
                    place
                }
                Written::Sub { parent, certainty } => {
                    if self.m.names.get(parent) != Some(&NameKind::Kind) {
                        return Err(refuse(
                            Load::Name,
                            format!("unknown id {parent}"),
                            file,
                            decl.line,
                        ));
                    }
                    subs.insert(decl.id.clone());
                    let mut sub = KindInfo::new(&decl.id, KindSpec::Plain, Some(parent));
                    sub.certainty = *certainty;
                    sub
                }
            };
            info.prelude = false;
            info.line = Some(decl.line);
            info.legs.clone_from(&decl.legs);
            info.attrs = decl
                .attrs
                .iter()
                .map(|(name, ty)| (name.clone(), Some(ty.clone())))
                .collect();
            self.m.kinds.push(&decl.id, info);
        }
        self.link_kinds(&subs)
    }

    /// Children, the check that no kind is its own subkind, and what subkinds inherit.
    fn link_kinds(&mut self, subs: &BTreeSet<String>) -> Result<(), Stop> {
        let ids = self.m.kinds.names();
        for id in &ids {
            let Some(parent) = self.m.kinds.get(id).and_then(|k| k.parent.clone()) else {
                continue;
            };
            let Some(up) = self.m.kinds.get_mut(&parent) else {
                return Err(Stop::NotBuilt("a kind whose parent is missing"));
            };
            up.children.push(id.clone());
        }
        for id in &ids {
            self.no_kind_cycle(id)?;
        }
        for id in &ids {
            let chain = self.m.chain(id);
            let spec = chain
                .iter()
                .find(|k| !subs.contains(&k.id))
                .map_or(KindSpec::Plain, |k| k.spec);
            let certainty = chain.iter().find_map(|k| k.certainty);
            if let Some(kind) = self.m.kinds.get_mut(id) {
                kind.spec = spec;
                if spec == KindSpec::Event {
                    kind.certainty = certainty;
                }
                kind.children.sort();
            }
        }
        Ok(())
    }

    /// A kind's chain of parents ends; a kind that is its own subkind is refused. L §5.7.
    fn no_kind_cycle(&self, id: &str) -> Result<(), Stop> {
        let mut seen = BTreeSet::new();
        let mut at = id.to_owned();
        while let Some(parent) = self.m.kinds.get(&at).and_then(|k| k.parent.clone()) {
            if !seen.insert(at.clone()) {
                let Some((file, line)) = self.kind_place(id) else {
                    return Err(Stop::NotBuilt("a cycle of the prelude's kinds"));
                };
                return Err(refuse(
                    Load::Cycle,
                    format!("kind {id} is a subkind of itself"),
                    &file,
                    line,
                ));
            }
            at = parent;
        }
        Ok(())
    }

    /// The file and line of a declared kind.
    fn kind_place(&self, id: &str) -> Option<(String, u32)> {
        for (file, section) in &self.sections {
            if let Section::Kind(kind) = section
                && kind.id == id
            {
                return Some((file.clone(), kind.line));
            }
        }
        None
    }

    /// The currencies' things, the versions' entries in known order, base kinds and the keys
    /// of kinds of unit (refcore's `build_entries`). L §5.4, §5.7.
    pub(super) fn build_entries(&mut self) -> Result<(), Stop> {
        for code in self.currency_order.clone() {
            let mut thing = Thing::new(&code);
            thing.base = Some("currencies".to_owned());
            thing.entries.push(Entry {
                kind: "currencies".to_owned(),
                span: Span::always(),
                version: None,
                replace: false,
                unit: None,
                priced: None,
                line: None,
            });
            self.m.things.push(&code, thing);
        }
        let mut order: Vec<(Day, String)> = self
            .m
            .versions
            .entries()
            .map(|(id, v)| (v.known, id.clone()))
            .collect();
        order.sort_by_key(|(known, _)| *known);
        let sections = core::mem::take(&mut self.sections);
        let entered = self.enter_all(&sections, &order);
        self.sections = sections;
        entered?;
        self.base_kinds()?;
        for (name, thing) in self.m.things.entries() {
            for entry in &thing.entries {
                if self.m.kinds.get(&entry.kind).map(|k| k.spec) == Some(KindSpec::Unit) {
                    self.m
                        .unit_kind_keys
                        .insert(name.clone(), entry.kind.clone());
                }
            }
        }
        Ok(())
    }

    /// Every `+` line of every version, in versions' known order.
    fn enter_all(
        &mut self,
        sections: &[(String, Section)],
        order: &[(Day, String)],
    ) -> Result<(), Stop> {
        for (_, id) in order {
            let Some((file, Section::Version(version))) = sections.iter().find(|(_, s)| {
                let Section::Version(v) = s else {
                    return false;
                };
                v.id == *id
            }) else {
                continue;
            };
            for item in &version.items {
                if let ItemForm::Enter(enter) = &item.form {
                    self.entry(id, file, enter, item.line)?;
                }
            }
        }
        Ok(())
    }

    /// One `+` line: keys added to a list index, or things entering a kind (refcore's `entry`).
    /// L §4.1, §5.7.
    fn entry(&mut self, version: &str, file: &str, enter: &Enter, line: u32) -> Result<(), Stop> {
        if enter.qual.is_some() {
            return Err(Stop::NotBuilt("a member's prefix"));
        }
        let target = &enter.target;
        let texts = enter
            .keys
            .iter()
            .map(|k| key_source_text(k, file, line))
            .collect::<Result<Vec<String>, Stop>>()?;
        match self.m.names.get(target) {
            Some(NameKind::Index) => self.enter_index(version, file, enter, &texts, line),
            Some(NameKind::Kind) => self.enter_kind(version, file, enter, &texts, line),
            Some(_) => Err(refuse(
                Load::Name,
                format!("{target} is not a kind or an index"),
                file,
                line,
            )),
            None => Err(refuse(
                Load::Name,
                format!("unknown id {target}"),
                file,
                line,
            )),
        }
    }

    /// Keys a version adds to a list index. L §4.1.
    fn enter_index(
        &mut self,
        version: &str,
        file: &str,
        enter: &Enter,
        texts: &[String],
        line: u32,
    ) -> Result<(), Stop> {
        let target = &enter.target;
        let Some(index) = self.m.indexes.get_mut(target) else {
            return Err(Stop::NotBuilt("an index that isn't built"));
        };
        if index.form != IndexForm::List {
            let form = if index.form == IndexForm::Open {
                "open"
            } else {
                "range"
            };
            return Err(refuse(
                Load::Kind,
                format!("an addition to a {form} index {target}"),
                file,
                line,
            ));
        }
        if enter.replace || enter.lives.is_some() || enter.unit.is_some() || enter.priced.is_some()
        {
            return Err(refuse(
                Load::Parse,
                "cannot parse: an index entry holds keys only".to_owned(),
                file,
                line,
            ));
        }
        for text in texts {
            if index.by_text(text).is_some() {
                return Err(refuse(
                    Load::Dup,
                    format!("a key entered twice: {text}"),
                    file,
                    line,
                ));
            }
            let key = KeyVal::Key {
                index: target.clone(),
                name: text.clone(),
            };
            index.texts.push((text.clone(), key.clone()));
            index.keys.push(key.clone());
            index.added.push((key, version.to_owned()));
        }
        index.finish();
        Ok(())
    }

    /// Things a version enters into a kind, with their lifespan. L §5.7.
    fn enter_kind(
        &mut self,
        version: &str,
        file: &str,
        enter: &Enter,
        texts: &[String],
        line: u32,
    ) -> Result<(), Stop> {
        let target = &enter.target;
        let Some(kind) = self.m.kinds.get(target) else {
            return Err(Stop::NotBuilt("a kind that isn't built"));
        };
        let named = texts.join(", ");
        if !kind.children.is_empty() {
            return Err(refuse(
                Load::Leaf,
                format!("{named} enters {target}, which has subkinds"),
                file,
                line,
            ));
        }
        let Some(lives) = &enter.lives else {
            return Err(refuse(
                Load::Lifespan,
                format!("{named} enters {target} without lives"),
                file,
                line,
            ));
        };
        self.check_entry_unit(kind.spec, file, enter, texts, line)?;
        let span = lspan_to_span(lives, file, line)?;
        if span.start().is_none() && !span.is_always() {
            return Err(refuse(
                Load::Lifespan,
                format!("{named}'s lifespan has no start"),
                file,
                line,
            ));
        }
        for name in texts {
            self.enter_thing(version, file, enter, name, &span, line)?;
        }
        Ok(())
    }

    /// A unit on an entry is stated only into a place kind of unit `one`, and there it must be.
    fn check_entry_unit(
        &self,
        spec: KindSpec,
        file: &str,
        enter: &Enter,
        texts: &[String],
        line: u32,
    ) -> Result<(), Stop> {
        let target = &enter.target;
        let first = texts.first().map_or("", String::as_str);
        match spec {
            KindSpec::Event => Err(refuse(
                Load::Kind,
                format!("an entry into the event kind {target}"),
                file,
                line,
            )),
            KindSpec::Place => {
                let unit = self.m.kind_unit(target).unwrap_or("None");
                if unit == "one" && enter.unit.is_none() {
                    return Err(refuse(
                        Load::PlaceKind,
                        format!("{first} enters a kind of unit one without its unit"),
                        file,
                        line,
                    ));
                }
                if let Some(stated) = &enter.unit
                    && unit != "one"
                {
                    return Err(refuse(
                        Load::Unit,
                        format!(
                            "{first} states {stated}; its kind {target} is of unit {unit}, not one"
                        ),
                        file,
                        line,
                    ));
                }
                Ok(())
            }
            KindSpec::Plain | KindSpec::Unit => {
                if enter.unit.is_some() || enter.priced.is_some() {
                    return Err(refuse(
                        Load::Parse,
                        format!("cannot parse: a unit on an entry into {target}, not a place kind"),
                        file,
                        line,
                    ));
                }
                Ok(())
            }
        }
    }

    /// One thing's entry: a first one, or a `replace` of an earlier one. L §5.7, §4.6 step 2.
    fn enter_thing(
        &mut self,
        version: &str,
        file: &str,
        enter: &Enter,
        name: &str,
        span: &Span,
        line: u32,
    ) -> Result<(), Stop> {
        let target = &enter.target;
        if self.m.names.contains_key(name) {
            return Err(refuse(
                Load::Key,
                format!("a key is also an id: {name}"),
                file,
                line,
            ));
        }
        let known = self.m.versions.get(version).map(|v| v.known);
        if !self.m.things.contains(name) {
            self.m.things.push(name, Thing::new(name));
        }
        let mut olds: Vec<(Option<Day>, bool)> = Vec::new();
        if let Some(thing) = self.m.things.get(name) {
            for entry in thing.entries.iter().filter(|e| e.kind == *target) {
                olds.push((self.entry_known(entry), entry.replace));
            }
        }
        let Some(thing) = self.m.things.get_mut(name) else {
            return Err(Stop::NotBuilt("a thing that isn't built"));
        };
        if enter.replace {
            let earlier = olds.iter().any(|(old, _)| old.is_some() && *old < known);
            if !earlier {
                return Err(refuse(
                    Load::Replace,
                    format!("replace {target} + [{name}]: no earlier row to replace"),
                    file,
                    line,
                ));
            }
        } else if olds.iter().any(|(_, replace)| !replace) {
            return Err(refuse(
                Load::Dup,
                format!("a key entered twice: {name} in {target}"),
                file,
                line,
            ));
        }
        thing.entries.push(Entry {
            kind: target.clone(),
            span: span.clone(),
            version: Some(version.to_owned()),
            replace: enter.replace,
            unit: enter.unit.clone(),
            priced: enter.priced.clone(),
            line: Some(line),
        });
        Ok(())
    }

    /// Each thing's base kind: the one kind it enters that isn't plain, or its earliest plain
    /// one by known, then by its lifespan's start. L §5.7.
    fn base_kinds(&mut self) -> Result<(), Stop> {
        let mut bases: Vec<(String, String)> = Vec::new();
        for (name, thing) in self.m.things.entries() {
            if thing.base.is_some() {
                continue;
            }
            let firsts: Vec<&Entry> = thing.entries.iter().filter(|e| !e.replace).collect();
            let mut nonplain: BTreeSet<&str> = BTreeSet::new();
            for entry in &firsts {
                let spec = self.m.kinds.get(&entry.kind).map(|k| k.spec);
                if spec.is_some_and(|s| s != KindSpec::Plain) {
                    nonplain.insert(entry.kind.as_str());
                }
            }
            let listed: Vec<&str> = nonplain.iter().copied().collect();
            if let [_, second, ..] = listed.as_slice() {
                let entry = firsts.iter().find(|e| e.kind == *second);
                let (file, line) = self.entry_place(entry.copied())?;
                return Err(refuse(
                    Load::Dup,
                    format!("a key entered twice: {name} ({})", listed.join(", ")),
                    &file,
                    line,
                ));
            }
            if let Some(kind) = nonplain.first() {
                bases.push((name.clone(), (*kind).to_owned()));
                continue;
            }
            let mut ordered: Vec<(Option<Day>, Option<Day>, &Entry)> = firsts
                .iter()
                .map(|e| (self.entry_known(e), e.span.start(), *e))
                .collect();
            ordered.sort_by_key(|(known, start, _)| (*known, *start));
            if let [(k1, s1, a), (k2, s2, b), ..] = ordered.as_slice()
                && (k1, s1) == (k2, s2)
                && a.kind != b.kind
            {
                let (file, line) = self.entry_place(Some(*b))?;
                return Err(refuse(
                    Load::Dup,
                    format!(
                        "a key entered twice: {name} ({}, {}, known and starting the same day)",
                        a.kind, b.kind
                    ),
                    &file,
                    line,
                ));
            }
            if let Some((_, _, entry)) = ordered.first() {
                bases.push((name.clone(), entry.kind.clone()));
            }
        }
        for (name, kind) in bases {
            if let Some(thing) = self.m.things.get_mut(&name) {
                thing.base = Some(kind);
            }
        }
        Ok(())
    }

    /// The known day of an entry's version.
    fn entry_known(&self, entry: &Entry) -> Option<Day> {
        entry
            .version
            .as_deref()
            .and_then(|v| self.m.versions.get(v))
            .map(|v| v.known)
    }

    /// The file and line of a version's entry, for a refusal.
    fn entry_place(&self, entry: Option<&Entry>) -> Result<(String, u32), Stop> {
        let version = entry
            .and_then(|e| e.version.as_deref())
            .and_then(|v| self.m.versions.get(v));
        match (version, entry.and_then(|e| e.line)) {
            (Some(v), Some(line)) => Ok((v.file.clone(), line)),
            (Some(_) | None, None) | (None, Some(_)) => {
                Err(Stop::NotBuilt("an entry no version states"))
            }
        }
    }
}
