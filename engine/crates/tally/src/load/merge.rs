//! A joint model's merge: each member's lines under its prefix, the header's `same` lines of things, places and kinds joined, as one set of lines.
//! Spec: L §6.1 (a member's items read as `a.item`; the prelude and the built-in ids one in
//! every member; any other unit a member's own; Shared law; the declarations Identities join),
//! L §2.2 and §4.1 (a member's own calendar where its run differs), §8.3 (`LOAD-VERSION`,
//! `LOAD-SAME`).
//! Twin: engine/tools/refcore/joint.py (`joint_texts` from its `jlines` on, `collect_decls`,
//! `Decls`, `category`, `decl_keeps_parts`, `price_of_decl`, `shared_law`, `header_tally`,
//! `header_clause`, `blocks`). The joins planned (`same_joins`, `plan_joins`) are `joins.rs`'s,
//! a member's lines under its prefix (`member_lines`, `rename_tokens`) `prefixed.rs`'s.
//! Invariants: only a token a member's own parse read as an id is renamed; a line is named by
//! its file and place, never by address; the joint model's file comes first, then each
//! member's files by alias, then the joint model's other files.

use super::check::invariant;
use super::joins::{plan_joins, same_joins};
use super::joint::{SEP, owner_of};
use super::prefixed::{
    dedupe_units, drop_blocks, member_lines, own_calendar, rename_line, retok, rewrite_member_refs,
};
use crate::api::Source;
use crate::base::{Code, Load, Refusal};
use crate::model::{BUILTIN_INDEXES, BUILTIN_INPUTS, PRELUDE_KINDS};
use crate::syntax::{
    Enter, InputForm, ItemForm, Kind, KindDecl, KindSpec, Line, Lit, Price, Section, Stop, Tok,
    UnitForm, parse_lines, prescan_units, split_lines,
};
use crate::time::Day;
use std::collections::{BTreeMap, BTreeSet};

/// The prelude's place kinds, and `places` (refcore's `PRELUDE_PLACE`).
const PLACE_KINDS: [&str; 8] = [
    "places", "accounts", "cash", "holdings", "assets", "debts", "claims", "outside",
];

/// The calendar's built-in indexes, a member's own where its run differs (refcore's
/// `CALENDAR_IDS`).
pub(super) const CALENDAR: [&str; 3] = ["days", "months", "years"];

/// One member, loaded alone, as the merge takes it.
pub(super) struct Member {
    /// Its alias.
    pub(super) alias: String,
    /// Its files' lines: as split, or, for a member that is a joint model, as merged.
    pub(super) files: Vec<(String, Vec<Line>)>,
    /// Its model file's path.
    pub(super) mfile: String,
    /// For a member that is a joint model, its own names its `same` lines joined to one of its
    /// members' (refcore's `own_alias`): `ania` to `a·ania`.
    pub(super) own_alias: BTreeMap<String, String>,
    /// Whether it is a joint model.
    pub(super) nested: bool,
}

/// The merged lines, and what a joint model that has this one as its member reads of it.
pub(super) struct Merged {
    /// The lines of every file, the joint model's first.
    pub(super) files: Vec<(String, Vec<Line>)>,
    /// The joint model's own names its `same` lines joined to a member's.
    pub(super) own_alias: BTreeMap<String, String>,
}

/// Where a line stands: its file and its place among the file's lines.
pub(super) type LineAt = (String, usize);

/// A file's lines, with its path.
pub(super) type FileLines = (String, Vec<Line>);

/// What a section or a name is (refcore's `cat`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Cat {
    /// A `source`.
    Source,
    /// A `version`.
    Version,
    /// A `kind`.
    Kind,
    /// An `axis`.
    Axis,
    /// An `index`.
    Index,
    /// A `place`, or a thing of a place kind.
    Place,
    /// An `input`.
    Input,
    /// An observed `input`.
    Obs,
    /// A `band`.
    Band,
    /// A `def`.
    Def,
    /// A `fill`.
    Fill,
    /// A `move`.
    Move,
    /// A thing a version enters into a kind.
    Thing,
}

impl Cat {
    /// The word refcore keeps the category by.
    pub(super) fn word(self) -> &'static str {
        match self {
            Cat::Source => "source",
            Cat::Version => "version",
            Cat::Kind => "kind",
            Cat::Axis => "axis",
            Cat::Index => "index",
            Cat::Place => "place",
            Cat::Input => "input",
            Cat::Obs => "obs",
            Cat::Band => "band",
            Cat::Def => "def",
            Cat::Fill => "fill",
            Cat::Move => "move",
            Cat::Thing => "thing",
        }
    }

    /// Whether a header's `same` line of this category is joined before parsing.
    pub(super) fn joined(self) -> bool {
        matches!(self, Cat::Kind | Cat::Place | Cat::Thing)
    }
}

/// How a place is declared: by its section, at its first line, or by an entry into a place
/// kind, at the entry's line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum How {
    /// A `place` section.
    Section(LineAt),
    /// An entry of a version.
    Entry(Option<LineAt>),
}

/// A place a file set declares (refcore's `Decls.places`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlaceDecl {
    /// Its kind, as written.
    pub(super) kind: String,
    /// Its unit, where stated.
    pub(super) unit: Option<String>,
    /// Its price input, where stated.
    pub(super) priced: Option<Price>,
    /// How it is declared.
    pub(super) how: How,
    /// Whether it is keyed.
    pub(super) keyed: bool,
}

/// One entry of a thing into a kind (refcore's `Decls.things`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ThingEntry {
    /// The kind, as written.
    pub(super) kind: String,
    /// Whether it is a `replace` line.
    pub(super) replace: bool,
    /// Its unit, where stated.
    unit: Option<String>,
    /// Its price input, where stated.
    priced: Option<Price>,
    /// Its line.
    pub(super) line: Option<LineAt>,
}

/// The names a file set declares, before renaming, with what each is (refcore's `Decls`).
#[derive(Debug, Clone, Default)]
pub(super) struct Decls {
    /// Each name's category.
    cat: BTreeMap<String, Cat>,
    /// The kinds.
    pub(super) kinds: BTreeMap<String, KindDecl>,
    /// The places, by section or by entry.
    pub(super) places: BTreeMap<String, PlaceDecl>,
    /// The things, each with its entries in the order written.
    things: BTreeMap<String, Vec<ThingEntry>>,
    /// The units it declares, currencies aside.
    units: BTreeSet<String>,
    /// The indexes.
    indexes: BTreeSet<String>,
}

impl Decls {
    /// A kind and its parents among the kinds declared (refcore's `kind_chain`).
    fn chain(&self, kind: &str) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        let mut at = Some(kind.to_owned());
        while let Some(k) = at {
            if seen.contains(&k) {
                break;
            }
            at = match self.kinds.get(&k).map(|d| &d.spec) {
                Some(KindSpec::Sub { parent, .. }) => Some(parent.clone()),
                Some(_) | None => None,
            };
            seen.push(k);
        }
        seen
    }

    /// Whether a kind is a place kind (refcore's `is_place_kind`).
    fn is_place_kind(&self, kind: &str) -> bool {
        self.chain(kind).iter().any(|k| {
            PLACE_KINDS.contains(&k.as_str())
                || matches!(
                    self.kinds.get(k).map(|d| &d.spec),
                    Some(KindSpec::Place { .. })
                )
        })
    }

    /// Whether a kind is outside (refcore's `is_outside_kind`).
    pub(super) fn is_outside_kind(&self, kind: &str) -> bool {
        self.chain(kind).iter().any(|k| {
            k == "outside"
                || matches!(
                    self.kinds.get(k).map(|d| &d.spec),
                    Some(KindSpec::Place { outside: true, .. })
                )
        })
    }

    /// Whether a kind is a kind of unit (refcore's `is_unit_kind`).
    fn is_unit_kind(&self, kind: &str) -> bool {
        self.chain(kind).iter().any(|k| {
            k == "currencies" || matches!(self.kinds.get(k).map(|d| &d.spec), Some(KindSpec::Unit))
        })
    }

    /// Whether a declared place's kind, or a parent of it, keeps parts (refcore's
    /// `decl_keeps_parts`). L §3.6.
    pub(super) fn keeps_parts(&self, place: &str) -> bool {
        let Some(decl) = self.places.get(place) else {
            return false;
        };
        let mut seen = BTreeSet::new();
        let mut at = Some(decl.kind.clone());
        while let Some(k) = at {
            let Some(kind) = self.kinds.get(&k) else {
                break;
            };
            if !seen.insert(k) {
                break;
            }
            if !kind.parts.is_empty() {
                return true;
            }
            at = match &kind.spec {
                KindSpec::Sub { parent, .. } => Some(parent.clone()),
                KindSpec::Plain | KindSpec::Event(_) | KindSpec::Unit | KindSpec::Place { .. } => {
                    None
                }
            };
        }
        false
    }

    /// A thing's entries, in the order written.
    pub(super) fn entries(&self, thing: &str) -> &[ThingEntry] {
        let none: &[ThingEntry] = &[];
        self.things.get(thing).map_or(none, Vec::as_slice)
    }
}

/// What every owner declares: each member's by its alias, the joint model's own under `""`.
pub(super) struct Declared {
    /// Each owner's declarations.
    by: BTreeMap<String, Decls>,
}

impl Declared {
    /// A name's category and the declarations of its owner (refcore's `category`).
    pub(super) fn category(&self, name: &str) -> (Option<Cat>, Option<&Decls>) {
        let d = self.by.get(owner_of(name));
        (d.and_then(|d| d.cat.get(bare(name)).copied()), d)
    }

    /// Whether a joined group's first name is a kind: a group of kinds.
    pub(super) fn heads_kind(&self, group: &[String]) -> bool {
        group
            .first()
            .is_some_and(|x| self.category(x).0 == Some(Cat::Kind))
    }

    /// The kind a joined kind's name declares.
    pub(super) fn kind_decl(&self, name: &str) -> Result<&KindDecl, Stop> {
        self.category(name)
            .1
            .and_then(|d| d.kinds.get(bare(name)))
            .ok_or_else(|| invariant(format!("the joined kind {name} isn't declared")))
    }
}

/// The names the merge renames, by member (refcore's `rename`, `rename_j`, `alias`,
/// `keynames` and `own_cal`); not the reality's namespace (`model::Names`).
struct Renames {
    /// What each member declares, its own units and, where its run differs, its calendar.
    rename: BTreeMap<String, BTreeSet<String>>,
    /// The same, with the names a member that is a joint model joined to its members'.
    rename_j: BTreeMap<String, BTreeSet<String>>,
    /// Those joined names, `f·ania` to `f·a·ania`.
    alias: BTreeMap<String, String>,
    /// The things and places each member declares, for keys written as literals.
    keynames: BTreeMap<String, BTreeSet<String>>,
    /// Each member whose run differs from the joint model's, with its run.
    own_cal: BTreeMap<String, (Day, Day)>,
}

impl Renames {
    /// The names of every member (refcore's `joint_texts`, its renaming sets). L §6.1, §2.2.
    fn of(
        jlines: &[FileLines],
        members: &[Member],
        decls: &BTreeMap<String, Decls>,
    ) -> Result<Renames, Stop> {
        let span = |ls: &[Line]| (clause(ls, Clause::Start), clause(ls, Clause::End));
        let jmodel = jlines.iter().find(|(_, ls)| first_word(ls) == "model");
        let jspan = jmodel.map_or((None, None), |(_, ls)| span(ls));
        let mut names = Renames {
            rename: BTreeMap::new(),
            rename_j: BTreeMap::new(),
            alias: BTreeMap::new(),
            keynames: BTreeMap::new(),
            own_cal: BTreeMap::new(),
        };
        for m in members {
            let d = decls
                .get(&m.alias)
                .ok_or_else(|| invariant("a member without its declarations".to_owned()))?;
            let mut own: BTreeSet<String> = d
                .cat
                .keys()
                .filter(|n| !is_builtin_id(n))
                .cloned()
                .collect();
            own.extend(d.units.iter().cloned());
            let mspan = m
                .files
                .iter()
                .find(|(p, _)| *p == m.mfile)
                .map(|(_, ls)| span(ls));
            if let Some((Some(s), Some(e))) = mspan
                && (Some(s), Some(e)) != jspan
            {
                own.extend(CALENDAR.iter().map(|c| (*c).to_owned()));
                names.own_cal.insert(m.alias.clone(), (s, e));
            }
            let things = d
                .cat
                .iter()
                .filter(|(_, c)| matches!(c, Cat::Thing | Cat::Place));
            let keys = things.map(|(n, _)| n.clone()).collect();
            names.keynames.insert(m.alias.clone(), keys);
            let mut joint_own = own.clone();
            joint_own.extend(m.own_alias.keys().cloned());
            names.rename_j.insert(m.alias.clone(), joint_own);
            names.rename.insert(m.alias.clone(), own);
            for (x, r) in &m.own_alias {
                let (from, to) = (
                    format!("{}{SEP}{x}", m.alias),
                    format!("{}{SEP}{r}", m.alias),
                );
                names.alias.insert(from, to);
            }
        }
        Ok(names)
    }

    /// The names a member renames.
    fn of_member(&self, alias: &str) -> Result<&BTreeSet<String>, Stop> {
        self.rename
            .get(alias)
            .ok_or_else(|| invariant(format!("member {alias} without its names")))
    }

    /// A line of the joint model's own with its reads of members as one name each, and the
    /// names a member that is a joint model joined to its members' (refcore's `joint_texts`,
    /// its rewrite of `jlines`).
    fn joint_reads(&self, toks: &[Tok]) -> Result<Vec<Tok>, Stop> {
        let read = rewrite_member_refs(toks, &self.rename_j, &self.keynames)?;
        Ok(read
            .iter()
            .map(|t| match self.alias.get(&t.text) {
                Some(to) if t.kind == Kind::Id => retok(t, to.clone()),
                Some(_) | None => t.clone(),
            })
            .collect())
    }
}

/// One `same` line of the joint model's header that joins two things, places or kinds.
pub(super) struct Join {
    /// The left side's name.
    pub(super) left: String,
    /// The right side's name.
    pub(super) right: String,
    /// The line's number in the joint model's file.
    pub(super) line: u32,
}

/// What a joined name drops from its owner's lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Drop {
    /// A place section, by its first line.
    Block(LineAt),
    /// A kind section, by the kind's id.
    Kind(String),
}

/// The joins planned (refcore's `plan_joins`' results).
#[derive(Default)]
pub(super) struct Plan {
    /// Each joined name to its group's representative.
    pub(super) jmap: BTreeMap<String, String>,
    /// The sections each owner drops.
    pub(super) drops: BTreeMap<String, Vec<Drop>>,
    /// The entry keys each owner drops, by the entry's line.
    pub(super) keys: BTreeMap<String, BTreeMap<LineAt, BTreeSet<String>>>,
}

impl Plan {
    /// The sections an owner drops.
    fn drops_of(&self, owner: &str) -> &[Drop] {
        let none: &[Drop] = &[];
        self.drops.get(owner).map_or(none, Vec::as_slice)
    }

    /// A token with the joins applied: a name joined to another's as its group's
    /// representative.
    fn joined(&self, t: &Tok) -> Tok {
        match self.jmap.get(&t.text) {
            Some(to) if t.kind == Kind::Id => retok(t, to.clone()),
            Some(_) | None => t.clone(),
        }
    }

    /// A line with the joins applied to each of its tokens.
    fn joined_line(&self, line: &Line) -> Line {
        Line {
            level: line.level,
            toks: line.toks.iter().map(|t| self.joined(t)).collect(),
            number: line.number,
        }
    }
}

/// The joint model's own files and its members' lines merged into one set of lines (refcore's
/// `joint_texts`, from its `jlines` on). L §6.1.
///
/// # Errors
///
/// `LOAD-VERSION`, `LOAD-SAME`, `LOAD-PARTS`, a parse refusal, as refcore meets them.
pub(super) fn merge(jfile: &str, own: &[Source], members: Vec<Member>) -> Result<Merged, Stop> {
    let mut jlines: Vec<FileLines> = Vec::with_capacity(own.len());
    for file in own {
        jlines.push((file.path.clone(), split_lines(&file.text, &file.path)?));
    }
    let mut every: Vec<&[Line]> = jlines.iter().map(|(_, l)| l.as_slice()).collect();
    for m in &members {
        every.extend(m.files.iter().map(|(_, l)| l.as_slice()));
    }
    let units = prescan_units(&every);
    let mut decls: BTreeMap<String, Decls> = BTreeMap::new();
    for m in &members {
        decls.insert(m.alias.clone(), collect_decls(&m.files, &units)?);
    }
    let mut members = members;
    members.sort_by(|a, b| a.alias.cmp(&b.alias));
    check_tally(&jlines, &members)?;
    shared_law(&members)?;
    let names = Renames::of(&jlines, &members, &decls)?;
    for (_, ls) in &mut jlines {
        for line in ls.iter_mut() {
            line.toks = names.joint_reads(&line.toks)?;
        }
    }
    decls.insert(String::new(), collect_decls(&jlines, &units)?);
    let declared = Declared { by: decls };
    let none: &[Line] = &[];
    let jf_lines = jlines
        .iter()
        .find(|(p, _)| p == jfile)
        .map_or(none, |(_, ls)| ls.as_slice());
    let (joins, kept) = same_joins(jfile, jf_lines, &declared)?;
    let plan = plan_joins(jfile, &joins, &declared)?;
    let own_alias: BTreeMap<String, String> = plan
        .jmap
        .iter()
        .filter(|(x, _)| owner_of(x).is_empty())
        .map(|(x, r)| (x.clone(), r.clone()))
        .collect();
    let hoisted = hoisted_sames(&members, &names, &plan)?;
    let (out, owners) = members_merged(&members, &names, &plan)?;
    let (first, rest) = joint_merged(jfile, jlines, &kept, &plan, &hoisted)?;
    let mut files = Vec::new();
    files.extend(first);
    files.extend(out);
    files.extend(rest);
    Ok(Merged {
        files: dedupe_units(files, &owners)?,
        own_alias,
    })
}

/// The header `same` lines of each member that is a joint model, under its prefix, that its
/// own merge left for load (refcore's `hoisted`). L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a member without its names or its model's file.
fn hoisted_sames(members: &[Member], names: &Renames, plan: &Plan) -> Result<Vec<Line>, Stop> {
    let mut hoisted = Vec::new();
    for m in members.iter().filter(|m| m.nested) {
        let own = names.of_member(&m.alias)?;
        let Some((_, lines)) = m.files.iter().find(|(p, _)| *p == m.mfile) else {
            return Err(invariant(format!(
                "member {} without its model's file",
                m.alias
            )));
        };
        // The header's lines after the `model` line (refcore's `mls[1:]`).
        let header = lines.iter().skip(1).take_while(|l| l.level > 0);
        for line in header {
            if line_word(line) == "same" {
                hoisted.push(Line {
                    level: line.level,
                    toks: rename_line(&m.alias, &line.toks, own, &plan.jmap),
                    number: line.number,
                });
            }
        }
    }
    Ok(hoisted)
}

/// Each member's files under its prefix, in alias order, its own calendar after its model
/// file's header where its run differs; and each file's owner. L §6.1, §2.2.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a member without its names.
fn members_merged(
    members: &[Member],
    names: &Renames,
    plan: &Plan,
) -> Result<(Vec<FileLines>, BTreeMap<String, String>), Stop> {
    let mut out = Vec::new();
    let mut owners: BTreeMap<String, String> = BTreeMap::new();
    for m in members {
        let own = names.of_member(&m.alias)?;
        let drops = plan.drops_of(&m.alias);
        let keys = plan.keys.get(&m.alias);
        for (path, ls) in &m.files {
            owners.insert(path.clone(), m.alias.clone());
            let lines = member_lines(&m.alias, path, ls, own, plan, drops, keys);
            let lines = match names.own_cal.get(&m.alias) {
                Some((s, e)) if *path == m.mfile => {
                    let after = header_end(&lines, Header::Whole)?;
                    spliced(lines, after, own_calendar(&m.alias, path, *s, *e)?)
                }
                Some(_) | None => lines,
            };
            out.push((path.clone(), lines));
        }
    }
    Ok((out, owners))
}

/// The joint model's own files: its header's joined `same` lines and the declarations joined
/// to another's dropped, the joins applied, its members' header `same` lines after its own
/// header lines; its model file apart, which comes first. L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a header longer than its file.
fn joint_merged(
    jfile: &str,
    jlines: Vec<FileLines>,
    kept: &BTreeSet<usize>,
    plan: &Plan,
    hoisted: &[Line],
) -> Result<(Option<FileLines>, Vec<FileLines>), Stop> {
    let (mut first, mut rest) = (None, Vec::new());
    for (path, ls) in jlines {
        let is_jfile = path == jfile;
        let pairs: Vec<(usize, Line)> = ls
            .into_iter()
            .enumerate()
            .filter(|(at, _)| !(is_jfile && kept.contains(at)))
            .collect();
        let kept_lines = drop_blocks(&path, pairs, plan.drops_of(""), plan.keys.get(""));
        let lines: Vec<Line> = kept_lines
            .iter()
            .map(|line| plan.joined_line(line))
            .collect();
        if !is_jfile {
            rest.push((path, lines));
            continue;
        }
        let lines = if hoisted.is_empty() {
            lines
        } else {
            let after = header_end(&lines, Header::BeforeNotes)?;
            spliced(lines, after, hoisted.to_vec())
        };
        first = Some((path, lines));
    }
    Ok((first, rest))
}

/// How much of a file's header a line goes after.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Header {
    /// All of it.
    Whole,
    /// Its lines before its first `note`.
    BeforeNotes,
}

/// The place after a file's header lines, or its lines before the first `note` (refcore's
/// `k`).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a header longer than its file.
fn header_end(lines: &[Line], header: Header) -> Result<usize, Stop> {
    let taken = |l: &&Line| l.level > 0 && (header == Header::Whole || line_word(l) != "note");
    let under = lines.iter().skip(1).take_while(taken).count();
    under
        .checked_add(1)
        .ok_or_else(|| invariant("a header longer than a file".to_owned()))
}

/// `lines` with `inserted` put before the line at `at`.
fn spliced(lines: Vec<Line>, at: usize, inserted: Vec<Line>) -> Vec<Line> {
    let mut rest = lines.into_iter();
    let mut out: Vec<Line> = rest.by_ref().take(at).collect();
    out.extend(inserted);
    out.extend(rest);
    out
}

/// What a file set declares (refcore's `collect_decls`): each file parsed with the merge's
/// unit names, its sections' categories, kinds, places, units and indexes, then the things its
/// versions enter; a key of a unit kind is a unit, one of a place kind a place. L §6.1.
///
/// # Errors
///
/// A parse refusal.
fn collect_decls(files: &[FileLines], units: &BTreeSet<String>) -> Result<Decls, Stop> {
    let mut d = Decls::default();
    for (path, lines) in files {
        let file = parse_lines(lines, path, units)?;
        let starts: Vec<usize> = blocks(lines).iter().skip(1).map(|(b0, _)| *b0).collect();
        for (section, start) in file.sections.iter().zip(starts) {
            declare(&mut d, section, (path.clone(), start));
        }
        for section in &file.sections {
            let Section::Version(version) = section else {
                continue;
            };
            for item in &version.items {
                if let ItemForm::Enter(enter) = &item.form {
                    let at = lines.iter().rposition(|l| l.number == item.line);
                    entered(&mut d, enter, at.map(|at| (path.clone(), at)).as_ref());
                }
            }
        }
    }
    let names: Vec<String> = d.things.keys().cloned().collect();
    for name in names {
        let Some(first) = d.entries(&name).first().cloned() else {
            continue;
        };
        if d.is_unit_kind(&first.kind) {
            d.things.remove(&name);
            d.units.insert(name);
        } else if d.is_place_kind(&first.kind) {
            d.places.entry(name.clone()).or_insert(PlaceDecl {
                kind: first.kind,
                unit: first.unit,
                priced: first.priced,
                how: How::Entry(first.line),
                keyed: false,
            });
            d.cat.entry(name).or_insert(Cat::Place);
        } else {
            d.cat.entry(name).or_insert(Cat::Thing);
        }
    }
    Ok(d)
}

/// The things one `+` line enters into a kind, by name (refcore's `collect_decls`, its loop
/// over a version's entries); an index's keys, the calendar's and a member's prefixed entry
/// are no things. L §6.1.
fn entered(d: &mut Decls, enter: &Enter, line: Option<&LineAt>) {
    let target = &enter.target;
    let calendar = ["days", "months", "years", "bank_account"].contains(&target.as_str());
    if enter.qual.is_some() || d.indexes.contains(target) || calendar {
        return;
    }
    for key in &enter.keys {
        let Lit::Name(name) = key else {
            continue;
        };
        let entry = ThingEntry {
            kind: target.clone(),
            replace: enter.replace,
            unit: enter.unit.clone(),
            priced: enter.priced.clone(),
            line: line.cloned(),
        };
        d.things
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }
}

/// One section's declaration (refcore's `collect_decls`, its first loop).
fn declare(d: &mut Decls, section: &Section, at: LineAt) {
    let (id, cat) = match section {
        Section::Ask(_) => return,
        Section::Unit(unit) => {
            if let UnitForm::Named(_) = unit.form {
                d.units.insert(unit.id.clone());
            }
            return;
        }
        Section::Source(s) => (&s.id, Cat::Source),
        Section::Version(v) => (&v.id, Cat::Version),
        Section::Kind(k) => {
            d.kinds.insert(k.id.clone(), (**k).clone());
            (&k.id, Cat::Kind)
        }
        Section::Axis(x) => (&x.id, Cat::Axis),
        Section::Index(x) => {
            d.indexes.insert(x.id.clone());
            (&x.id, Cat::Index)
        }
        Section::Place(p) => {
            let decl = PlaceDecl {
                kind: p.kind.clone(),
                unit: p.unit.clone(),
                priced: p.priced.clone(),
                how: How::Section(at),
                keyed: !p.binds.is_empty(),
            };
            d.places.insert(p.id.clone(), decl);
            (&p.id, Cat::Place)
        }
        Section::Input(i) => {
            let observed = matches!(i.form, InputForm::Observed);
            (&i.id, if observed { Cat::Obs } else { Cat::Input })
        }
        Section::Band(b) => (&b.id, Cat::Band),
        Section::Def(x) => (&x.id, Cat::Def),
        Section::Fill(f) => (&f.id, Cat::Fill),
        Section::Move(x) => (&x.id, Cat::Move),
    };
    d.cat.insert(id.clone(), cat);
}

/// Each member file's `tally` against the joint model's (refcore's `header_tally` check).
/// L §6.1.
///
/// # Errors
///
/// `LOAD-VERSION member <a> declares tally <x>, the joint model <y>`.
fn check_tally(jlines: &[FileLines], members: &[Member]) -> Result<(), Stop> {
    let tally = jlines
        .iter()
        .rev()
        .find(|(_, ls)| first_word(ls) == "model")
        .and_then(|(_, ls)| clause_tok(ls, Clause::Tally));
    for m in members {
        for (_, ls) in &m.files {
            let theirs = clause_tok(ls, Clause::Tally);
            if theirs.map(|t| &t.kind) != tally.map(|t| &t.kind) {
                let message = format!(
                    "member {} declares tally {}, the joint model {}",
                    m.alias,
                    tally_word(theirs),
                    tally_word(tally)
                );
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Version),
                    message,
                )));
            }
        }
    }
    Ok(())
}

/// A `tally` clause's version as refcore prints it, `None` where there is none.
fn tally_word(version: Option<&Tok>) -> String {
    version.map_or_else(|| "None".to_owned(), |t| t.text.clone())
}

/// Two members that copy different versions of one law part are refused (refcore's
/// `shared_law`). L §6.1 Shared law.
///
/// # Errors
///
/// `LOAD-SAME members <a> and <b> copy different versions of law part <p>`.
fn shared_law(members: &[Member]) -> Result<(), Stop> {
    let mut laws: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    for m in members {
        for (pid, versions) in m.files.iter().filter_map(|(_, ls)| law_part(ls)) {
            let by = laws.entry(pid).or_insert_with(BTreeMap::new);
            by.entry(m.alias.clone())
                .or_insert_with(BTreeSet::new)
                .extend(versions);
        }
    }
    for (pid, by) in &laws {
        let mut sets = by.values();
        let first = sets.next();
        if by.len() < 2 || sets.all(|s| Some(s) == first) {
            continue;
        }
        let mut aliases = by.keys();
        let (Some(a), Some(b)) = (aliases.next(), aliases.next()) else {
            continue;
        };
        let message = format!("members {a} and {b} copy different versions of law part {pid}");
        return Err(Stop::Refused(Refusal::new(Code::Load(Load::Same), message)));
    }
    Ok(())
}

/// A member's file that is a law part of its own, `part <p>` with `law` in its header: its id
/// and the versions it holds (refcore's `shared_law`, its loop). L §6.1 Shared law.
fn law_part(lines: &[Line]) -> Option<(String, BTreeSet<String>)> {
    let (head, header) = lines.split_first()?;
    let [word, pid, ..] = head.toks.as_slice() else {
        return None;
    };
    if word.text != "part" || pid.text.contains(SEP) {
        return None;
    }
    let law = header
        .iter()
        .take_while(|l| l.level > 0)
        .any(|l| l.level == 1 && line_word(l) == "law");
    if !law {
        return None;
    }
    let mut versions = BTreeSet::new();
    for line in lines
        .iter()
        .filter(|l| l.level == 0 && line_word(l) == "version")
    {
        if let Some(id) = line.toks.get(1) {
            versions.insert(id.text.clone());
        }
    }
    Some((pid.text.clone(), versions))
}

/// A name without its outer member's prefix.
pub(super) fn bare(name: &str) -> &str {
    name.split_once(SEP).map_or(name, |(_, rest)| rest)
}

/// The level-0 blocks of a file's lines, `[start, end)`, its header first (refcore's `blocks`).
pub(super) fn blocks(lines: &[Line]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if at == 0 || line.level == 0 {
            if let Some(last) = out.last_mut() {
                last.1 = at;
            }
            out.push((at, lines.len()));
        }
    }
    out
}

/// A line's first word.
pub(super) fn line_word(line: &Line) -> &str {
    line.toks.first().map_or("", |t| t.text.as_str())
}

/// A file's first word.
fn first_word(lines: &[Line]) -> &str {
    lines.first().map_or("", line_word)
}

/// A header clause the merge reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Clause {
    /// `tally`.
    Tally,
    /// `start`.
    Start,
    /// `end`.
    End,
}

impl Clause {
    /// Its word.
    fn text(self) -> &'static str {
        match self {
            Clause::Tally => "tally",
            Clause::Start => "start",
            Clause::End => "end",
        }
    }
}

/// The token after a header clause's word (refcore's `header_tally`, `header_clause`).
fn clause_tok(lines: &[Line], word: Clause) -> Option<&Tok> {
    let (_, header) = lines.split_first()?;
    header
        .iter()
        .take_while(|l| l.level > 0)
        .find(|l| line_word(l) == word.text() && l.toks.len() > 1)
        .and_then(|l| l.toks.get(1))
}

/// A header clause's day (`start`, `end`).
fn clause(lines: &[Line], word: Clause) -> Option<Day> {
    match clause_tok(lines, word).map(|t| &t.kind) {
        Some(Kind::Day(day)) => Some(*day),
        Some(_) | None => None,
    }
}

/// Whether an id is one in every member: a built-in input or index, a prelude kind, or `cut`
/// (refcore's `BUILTIN_IDS`). L §2.2, §6.1.
pub(super) fn is_builtin_id(id: &str) -> bool {
    BUILTIN_INPUTS.contains(&id)
        || BUILTIN_INDEXES.contains(&id)
        || PRELUDE_KINDS.contains(&id)
        || id == "cut"
}
