//! A joint model's merge: each member's lines under its prefix, the header's `same` lines of things, places and kinds joined, as one set of lines.
//! Spec: L §6.1 (a member's items read as `a.item`, its keys written as literals `a.x[k]`; the
//! prelude and the built-in ids one in every member; currencies one by their code, two minor
//! units `LOAD-SAME`; any other unit a member's own; Shared law; Identities are written: two
//! kinds declared alike, two things of one base kind, two places of one kind, unit and price
//! input, never a place whose kind keeps parts, `LOAD-PARTS`), L §2.2 and §4.1 (a member's own
//! calendar where its run differs), §8.3 (`LOAD-VERSION`, `LOAD-SAME`, `LOAD-PARTS`).
//! Twin: engine/tools/refcore/joint.py (`joint_texts` from its `jlines` on, `collect_decls`,
//! `Decls`, `shared_law`, `rewrite_member_refs`, `rewrite_member_names`, `_rename_ok`,
//! `retok`, `same_joins`, `category`, `thing_kind`, `sort_name`, `plan_joins`, `kind_sig`,
//! `decl_keeps_parts`, `price_of_decl`, `member_lines`, `rename_tokens`, `drop_entry_keys`,
//! `drop_blocks`, `dedupe_units`, `own_calendar_lines`, `header_tally`, `header_clause`,
//! `blocks`).
//! Invariants: only a token a member's own parse read as an id is renamed; a line is named by
//! its file and place, never by address; the joint model's file comes first, then each
//! member's files by alias, then the joint model's other files.

use super::check::invariant;
use super::joint::{SEP, owner_of, shown};
use crate::api::Source;
use crate::base::{Code, Load, Place as At, Refusal};
use crate::model::{BUILTIN_INDEXES, BUILTIN_INPUTS, PRELUDE_KINDS};
use crate::syntax::{
    Enter, InputForm, ItemForm, Kind, KindDecl, KindSpec, Line, Lit, Price, Section, Stop, Tok,
    Type, UnitForm, parse_lines, prescan_units, split_lines,
};
use crate::time::Day;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// The prelude's place kinds, and `places` (refcore's `PRELUDE_PLACE`).
const PLACE_KINDS: [&str; 8] = [
    "places", "accounts", "cash", "holdings", "assets", "debts", "claims", "outside",
];

/// The calendar's built-in indexes, a member's own where its run differs (refcore's
/// `CALENDAR_IDS`).
const CALENDAR: [&str; 3] = ["days", "months", "years"];

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
type LineAt = (String, usize);

/// A file's lines, with its path.
type FileLines = (String, Vec<Line>);

/// What a kind's declaration states, for two kinds stated `same` (refcore's `kind_sig`).
type KindSig = (KindSpec, Vec<(String, String)>, Vec<(String, Type)>);

/// The names joined, in groups in the order first named, each group with its root.
type Groups = Vec<(String, Vec<String>)>;

/// What a section or a name is (refcore's `cat`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cat {
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
    fn word(self) -> &'static str {
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
    fn joined(self) -> bool {
        matches!(self, Cat::Kind | Cat::Place | Cat::Thing)
    }
}

/// How a place is declared: by its section, at its first line, or by an entry into a place
/// kind, at the entry's line.
#[derive(Debug, Clone, PartialEq, Eq)]
enum How {
    /// A `place` section.
    Section(LineAt),
    /// An entry of a version.
    Entry(Option<LineAt>),
}

/// A place a file set declares (refcore's `Decls.places`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PlaceDecl {
    /// Its kind, as written.
    kind: String,
    /// Its unit, where stated.
    unit: Option<String>,
    /// Its price input, where stated.
    priced: Option<Price>,
    /// How it is declared.
    how: How,
    /// Whether it is keyed.
    keyed: bool,
}

/// One entry of a thing into a kind (refcore's `Decls.things`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ThingEntry {
    /// The kind, as written.
    kind: String,
    /// Whether it is a `replace` line.
    replace: bool,
    /// Its unit, where stated.
    unit: Option<String>,
    /// Its price input, where stated.
    priced: Option<Price>,
    /// Its line.
    line: Option<LineAt>,
}

/// The names a file set declares, before renaming, with what each is (refcore's `Decls`).
#[derive(Debug, Clone, Default)]
struct Decls {
    /// Each name's category.
    cat: BTreeMap<String, Cat>,
    /// The kinds.
    kinds: BTreeMap<String, KindDecl>,
    /// The places, by section or by entry.
    places: BTreeMap<String, PlaceDecl>,
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
    fn is_outside_kind(&self, kind: &str) -> bool {
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
    fn keeps_parts(&self, place: &str) -> bool {
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
    fn entries(&self, thing: &str) -> &[ThingEntry] {
        let none: &[ThingEntry] = &[];
        self.things.get(thing).map_or(none, Vec::as_slice)
    }
}

/// What every owner declares: each member's by its alias, the joint model's own under `""`.
struct Declared {
    /// Each owner's declarations.
    by: BTreeMap<String, Decls>,
}

impl Declared {
    /// A name's category and the declarations of its owner (refcore's `category`).
    fn category(&self, name: &str) -> (Option<Cat>, Option<&Decls>) {
        let d = self.by.get(owner_of(name));
        (d.and_then(|d| d.cat.get(bare(name)).copied()), d)
    }

    /// Whether a joined group's first name is a kind: a group of kinds.
    fn heads_kind(&self, group: &[String]) -> bool {
        group
            .first()
            .is_some_and(|x| self.category(x).0 == Some(Cat::Kind))
    }

    /// The kind a joined kind's name declares.
    fn kind_decl(&self, name: &str) -> Result<&KindDecl, Stop> {
        self.category(name)
            .1
            .and_then(|d| d.kinds.get(bare(name)))
            .ok_or_else(|| invariant(format!("the joined kind {name} isn't declared")))
    }
}

/// The names the merge renames, by member (refcore's `rename`, `rename_j`, `alias`,
/// `keynames` and `own_cal`).
struct Names {
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

impl Names {
    /// The names of every member (refcore's `joint_texts`, its renaming sets). L §6.1, §2.2.
    fn of(
        jlines: &[FileLines],
        members: &[Member],
        decls: &BTreeMap<String, Decls>,
    ) -> Result<Names, Stop> {
        let span = |ls: &[Line]| (clause(ls, Clause::Start), clause(ls, Clause::End));
        let jmodel = jlines.iter().find(|(_, ls)| first_word(ls) == "model");
        let jspan = jmodel.map_or((None, None), |(_, ls)| span(ls));
        let mut names = Names {
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
struct Join {
    /// The left side's name.
    left: String,
    /// The right side's name.
    right: String,
    /// The line's number in the joint model's file.
    line: u32,
}

/// What a joined name drops from its owner's lines.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Drop {
    /// A place section, by its first line.
    Block(LineAt),
    /// A kind section, by the kind's id.
    Kind(String),
}

/// The joins planned (refcore's `plan_joins`' results).
#[derive(Default)]
struct Plan {
    /// Each joined name to its group's representative.
    jmap: BTreeMap<String, String>,
    /// The sections each owner drops.
    drops: BTreeMap<String, Vec<Drop>>,
    /// The entry keys each owner drops, by the entry's line.
    keys: BTreeMap<String, BTreeMap<LineAt, BTreeSet<String>>>,
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
    let names = Names::of(&jlines, &members, &decls)?;
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
fn hoisted_sames(members: &[Member], names: &Names, plan: &Plan) -> Result<Vec<Line>, Stop> {
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
    names: &Names,
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

/// The header `same` lines of the joint model that name things, places or kinds, one id on
/// each side (refcore's `same_joins`), and the places of those lines among the file's lines;
/// the others stay for load. L §6.1.
///
/// # Errors
///
/// `LOAD-SAME` for two sides of different sorts.
fn same_joins(
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

/// A name without its outer member's prefix.
fn bare(name: &str) -> &str {
    name.split_once(SEP).map_or(name, |(_, rest)| rest)
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
fn plan_joins(jfile: &str, joins: &[Join], declared: &Declared) -> Result<Plan, Stop> {
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

/// A member's lines under its prefix (refcore's `member_lines`): its header a part of the joint
/// model, its asks and the declarations joined to another's dropped, its names renamed and
/// the joins applied. L §6.1.
fn member_lines(
    alias: &str,
    path: &str,
    lines: &[Line],
    names: &BTreeSet<String>,
    plan: &Plan,
    drops: &[Drop],
    keys: Option<&BTreeMap<LineAt, BTreeSet<String>>>,
) -> Vec<Line> {
    let mut keep: Vec<(Option<usize>, Line)> = Vec::new();
    for (n, (b0, b1)) in blocks(lines).into_iter().enumerate() {
        let Some((first, under)) = lines.get(b0..b1).and_then(<[Line]>::split_first) else {
            continue;
        };
        if n == 0 {
            keep.push((None, part_header(alias, first)));
            let kept = under
                .iter()
                .filter(|l| matches!(line_word(l), "tally" | "law"));
            keep.extend(kept.map(|l| (None, l.clone())));
            continue;
        }
        let word = line_word(first);
        if word == "ask" || dropped(word, first, path, b0, drops) {
            continue;
        }
        let block = lines.iter().enumerate().take(b1).skip(b0);
        keep.extend(block.map(|(at, line)| (Some(at), line.clone())));
    }
    let mut out = Vec::with_capacity(keep.len());
    for (at, line) in keep {
        let gone = keys.zip(at).and_then(|(k, i)| k.get(&(path.to_owned(), i)));
        let toks = match gone {
            Some(gone) => match drop_entry_keys(&line.toks, gone) {
                Some(toks) => toks,
                None => continue,
            },
            None => line.toks,
        };
        out.push(Line {
            toks: rename_line(alias, &toks, names, &plan.jmap),
            ..line
        });
    }
    out
}

/// A member file's header as a part of the joint model: `part a·member` for its model file,
/// `part a·p` for its part `p` (refcore's `member_lines`, its header). L §6.1.
fn part_header(alias: &str, first: &Line) -> Line {
    let pid = match first.toks.get(1) {
        Some(id) if line_word(first) == "part" => id.text.as_str(),
        Some(_) | None => "member",
    };
    let word = Tok {
        kind: Kind::Id,
        text: "part".to_owned(),
        ws: false,
        seq: 0,
        mark: Cell::default(),
    };
    let id = Tok {
        kind: Kind::Id,
        text: format!("{alias}{SEP}{pid}"),
        ws: true,
        seq: 1,
        mark: Cell::default(),
    };
    Line {
        level: 0,
        toks: vec![word, id],
        number: first.number,
    }
}

/// Whether a block is a declaration joined to another's, and dropped.
fn dropped(word: &str, first: &Line, path: &str, start: usize, drops: &[Drop]) -> bool {
    if drops.contains(&Drop::Block((path.to_owned(), start))) {
        return true;
    }
    word == "kind"
        && first
            .toks
            .get(1)
            .is_some_and(|id| drops.contains(&Drop::Kind(id.text.clone())))
}

/// The joint model's own lines with the declarations joined to another's dropped (refcore's
/// `drop_blocks`), each line with its place among the file's lines as split.
fn drop_blocks(
    path: &str,
    pairs: Vec<(usize, Line)>,
    drops: &[Drop],
    keys: Option<&BTreeMap<LineAt, BTreeSet<String>>>,
) -> Vec<Line> {
    if drops.is_empty() && keys.is_none() {
        return pairs.into_iter().map(|(_, line)| line).collect();
    }
    let levels: Vec<Line> = pairs.iter().map(|(_, l)| l.clone()).collect();
    let mut out = Vec::with_capacity(pairs.len());
    for (b0, b1) in blocks(&levels) {
        let Some(block) = pairs.get(b0..b1) else {
            continue;
        };
        let Some((start, first)) = block.first() else {
            continue;
        };
        if dropped(line_word(first), first, path, *start, drops) {
            continue;
        }
        for (at, line) in block {
            match keys.and_then(|k| k.get(&(path.to_owned(), *at))) {
                Some(gone) => out.extend(drop_entry_keys(&line.toks, gone).map(|toks| Line {
                    toks,
                    ..line.clone()
                })),
                None => out.push(line.clone()),
            }
        }
    }
    out
}

/// An entry line `K + [k1, k2] …` without the keys named; `None` when no key is left
/// (refcore's `drop_entry_keys`).
fn drop_entry_keys(toks: &[Tok], names: &BTreeSet<String>) -> Option<Vec<Tok>> {
    let open = toks.iter().position(|t| t.text == "[");
    let close = toks.iter().position(|t| t.text == "]");
    let (Some(i), Some(j)) = (open, close) else {
        return Some(toks.to_vec());
    };
    let none: &[Tok] = &[];
    let inside = toks
        .get(i..j)
        .and_then(<[Tok]>::split_first)
        .map_or(none, |(_, rest)| rest);
    let left: Vec<&Tok> = inside
        .iter()
        .filter(|t| t.text != "," && !names.contains(&t.text))
        .collect();
    if left.is_empty() {
        return None;
    }
    let mut out: Vec<Tok> = toks.iter().take(i).cloned().collect();
    out.extend(toks.get(i).cloned());
    for (k, t) in left.iter().enumerate() {
        if k > 0 {
            out.push(Tok {
                kind: Kind::Op,
                text: ",".to_owned(),
                ws: false,
                seq: t.seq,
                mark: Cell::default(),
            });
        }
        out.push((*t).clone());
    }
    out.extend(toks.iter().skip(j).cloned());
    Some(out)
}

/// A currency declared by several files is one when the declarations agree (refcore's
/// `dedupe_units`): its later declarations dropped. L §6.1 (Currencies).
///
/// # Errors
///
/// `LOAD-SAME currency <C> is declared with different minor units by <a> and <b>`.
fn dedupe_units(
    files: Vec<FileLines>,
    owners: &BTreeMap<String, String>,
) -> Result<Vec<FileLines>, Stop> {
    let mut seen: BTreeMap<String, (Vec<String>, String)> = BTreeMap::new();
    let mut out = Vec::with_capacity(files.len());
    for (path, lines) in files {
        let who = owners
            .get(&path)
            .cloned()
            .unwrap_or_else(|| "the joint model".to_owned());
        let mut keep = Vec::with_capacity(lines.len());
        for (n, (b0, b1)) in blocks(&lines).into_iter().enumerate() {
            let block = lines.get(b0..b1).unwrap_or(&[]);
            let Some((code, sig)) = currency_block(n, block) else {
                keep.extend(block.iter().cloned());
                continue;
            };
            match seen.get(&code) {
                Some((before, by)) if *before != sig => {
                    let message = format!(
                        "currency {code} is declared with different minor units by {by} and {who}"
                    );
                    return Err(Stop::Refused(Refusal::new(Code::Load(Load::Same), message)));
                }
                Some(_) => {}
                None => {
                    seen.insert(code, (sig, who.clone()));
                    keep.extend(block.iter().cloned());
                }
            }
        }
        out.push((path, keep));
    }
    Ok(out)
}

/// A block that declares a currency, `unit XYZ: …`, past a file's header: its code and its
/// first line's words (refcore's `dedupe_units`, its test). L §6.1 (Currencies).
fn currency_block(n: usize, block: &[Line]) -> Option<(String, Vec<String>)> {
    let first = block.first()?;
    let [unit, code, ..] = first.toks.as_slice() else {
        return None;
    };
    if n == 0 || unit.text != "unit" || code.kind != Kind::Cur {
        return None;
    }
    let sig = first.toks.iter().map(|t| t.text.clone()).collect();
    Some((code.text.clone(), sig))
}

/// A member's own calendar indexes, `a·days`, `a·months` and `a·years` over its run (refcore's
/// `own_calendar_lines`). L §2.2, §4.1.
///
/// # Errors
///
/// A split refusal, which the text written can't meet.
fn own_calendar(alias: &str, path: &str, start: Day, end: Day) -> Result<Vec<Line>, Stop> {
    let text = format!(
        "index zzcaldays = day {}..{}\nindex zzcalmonths = month {:04}-{:02}..{:04}-{:02}\nindex zzcalyears = year {:04}..{:04}\n",
        start.text(),
        end.text(),
        start.year(),
        start.month(),
        end.year(),
        end.month(),
        start.year(),
        end.year()
    );
    let mut lines = split_lines(&text, path)?;
    for (line, name) in lines.iter_mut().zip(CALENDAR) {
        line.toks = line
            .toks
            .iter()
            .map(|t| {
                if t.kind == Kind::Id && t.text.starts_with("zzcal") {
                    retok(t, format!("{alias}{SEP}{name}"))
                } else {
                    t.clone()
                }
            })
            .collect();
    }
    Ok(lines)
}

/// A line's tokens with a member's names under its prefix and the joins applied (refcore's
/// `rename_tokens`): only a token its own parse read as an id that names an item.
fn rename_line(
    alias: &str,
    toks: &[Tok],
    names: &BTreeSet<String>,
    jmap: &BTreeMap<String, String>,
) -> Vec<Tok> {
    let mut out = Vec::with_capacity(toks.len());
    for (j, t) in toks.iter().enumerate() {
        if t.kind == Kind::Id && names.contains(&t.text) && t.mark.get().name && rename_ok(toks, j)
        {
            let full = format!("{alias}{SEP}{}", t.text);
            out.push(retok(t, jmap.get(&full).cloned().unwrap_or(full)));
        } else {
            out.push(t.clone());
        }
    }
    out
}

/// Whether the id token at `j` names an item, not a field, a rule label, a leg's or an
/// attribute's name, a record field or a band's argument (refcore's `_rename_ok`).
fn rename_ok(toks: &[Tok], j: usize) -> bool {
    let Some(t) = toks.get(j) else {
        return false;
    };
    if t.kind != Kind::Id {
        return false;
    }
    let prev = j.checked_sub(1).and_then(|i| toks.get(i));
    let next = j.checked_add(1).and_then(|i| toks.get(i));
    if prev.is_some_and(|p| p.text == ".") && !t.ws {
        return false;
    }
    if prev.is_some_and(|p| matches!(p.text.as_str(), "rule" | "leg" | "attr")) {
        return false;
    }
    if next.is_some_and(|n| n.text == ":")
        && prev.is_some_and(|p| matches!(p.text.as_str(), "(" | ","))
    {
        return false;
    }
    !in_beats_list(toks, j)
}

/// Whether the token at `j` is a label of a `beats` list (refcore's `_rename_ok`, its walk
/// back over the list).
fn in_beats_list(toks: &[Tok], j: usize) -> bool {
    let mut k = j;
    while let Some(i) = k.checked_sub(1) {
        let Some(c) = toks.get(i) else {
            break;
        };
        let listed = c.text == "," || (c.kind == Kind::Id && c.text != "beats");
        if !listed {
            break;
        }
        let before = i.checked_sub(1).and_then(|b| toks.get(b));
        if c.kind == Kind::Id && before.is_some_and(|b| b.text != "," && b.text != "beats") {
            break;
        }
        k = i;
    }
    k.checked_sub(1)
        .and_then(|i| toks.get(i))
        .is_some_and(|c| c.text == "beats")
}

/// The joint model's reads of a member, `a . x` touching, as one name `a·x` (`a . b . x` for a
/// member's member), and a key written as a literal right after a member's item as that
/// member's thing (refcore's `rewrite_member_refs`). L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a name past its line.
fn rewrite_member_refs(
    toks: &[Tok],
    rename: &BTreeMap<String, BTreeSet<String>>,
    keynames: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<Tok>, Stop> {
    let mut res = rewrite_member_names(toks, rename)?;
    let empty = BTreeSet::new();
    for j in 0..res.len() {
        let Some(t) = res.get(j) else {
            break;
        };
        let opens = j
            .checked_add(1)
            .and_then(|n| res.get(n))
            .is_some_and(|n| n.text == "[" && !n.ws);
        if t.kind != Kind::Id || !t.text.contains(SEP) || !opens {
            continue;
        }
        let member = owner_of(&t.text).to_owned();
        let pre = t.text.rsplit_once(SEP).map_or("", |(p, _)| p);
        let rel = pre
            .split_once(SEP)
            .map_or_else(String::new, |(_, r)| format!("{r}{SEP}"));
        let names = keynames.get(&member).unwrap_or(&empty);
        member_keys(&mut res, j, (&member, &rel), names);
    }
    Ok(res)
}

/// The keys written as literals in the `[…]` after the member's item at `j`, each a thing or a
/// place the member declares, renamed under its prefix (refcore's `rewrite_member_refs`, its
/// walk over the keys). L §6.1.
fn member_keys(res: &mut [Tok], j: usize, (member, rel): (&str, &str), names: &BTreeSet<String>) {
    let mut depth: u32 = 0;
    let Some(from) = j.checked_add(1) else {
        return;
    };
    for k in from..res.len() {
        let Some(at) = res.get(k) else {
            return;
        };
        let s = at.text.as_str();
        if s == "[" || s == "(" {
            let Some(deeper) = depth.checked_add(1) else {
                return;
            };
            depth = deeper;
            continue;
        }
        if s == "]" || s == ")" {
            match depth.checked_sub(1) {
                Some(0) | None => return,
                Some(shallower) => depth = shallower,
            }
            continue;
        }
        if depth != 1 || at.kind != Kind::Id {
            continue;
        }
        let before = k.checked_sub(1).and_then(|b| res.get(b));
        let after = k.checked_add(1).and_then(|a| res.get(a));
        let listed = before.is_some_and(|b| b.text == "[" || b.text == ",")
            && after.is_some_and(|a| a.text == "," || a.text == "]");
        let full = format!("{rel}{}", at.text);
        if listed && names.contains(&full) {
            let renamed = retok_named(at, format!("{member}{SEP}{full}"));
            if let Some(slot) = res.get_mut(k) {
                *slot = renamed;
            }
        }
    }
}

/// `a . x` as one name `a·x`, the longest chain a member declares (refcore's
/// `rewrite_member_names`). L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a name past its line.
fn rewrite_member_names(
    toks: &[Tok],
    rename: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<Tok>, Stop> {
    let past = || invariant("a member's name past its line".to_owned());
    let mut out = Vec::with_capacity(toks.len());
    let mut j: usize = 0;
    while let Some(t) = toks.get(j) {
        let after_dot = j
            .checked_sub(1)
            .and_then(|i| toks.get(i))
            .is_some_and(|p| p.text == ".")
            && !t.ws;
        let names = rename
            .get(&t.text)
            .filter(|_| t.kind == Kind::Id && !after_dot);
        let Some((taken, joined)) = names.and_then(|names| longest_chain(toks, j, names)) else {
            out.push(t.clone());
            j = j.checked_add(1).ok_or_else(past)?;
            continue;
        };
        out.push(retok_named(t, format!("{}{SEP}{joined}", t.text)));
        j = j.checked_add(taken).ok_or_else(past)?;
    }
    Ok(out)
}

/// The longest chain `. x . y` after the member's alias at `j` that it declares, joined by its
/// separator, with the number of tokens the alias and the chain take (refcore's
/// `rewrite_member_names`, its walk). L §6.1.
fn longest_chain(toks: &[Tok], j: usize, names: &BTreeSet<String>) -> Option<(usize, String)> {
    let mut chain: Vec<&str> = Vec::new();
    let mut k = j.checked_add(1)?;
    while let (Some(dot), Some(id)) = (toks.get(k), k.checked_add(1).and_then(|n| toks.get(n))) {
        if dot.text != "." || dot.ws || id.kind != Kind::Id || id.ws {
            break;
        }
        chain.push(&id.text);
        k = k.checked_add(2)?;
    }
    let sep = SEP.to_string();
    for m in (1..=chain.len()).rev() {
        let joined = chain.get(..m)?.join(&sep);
        if names.contains(&joined) {
            return Some((m.checked_mul(2)?.checked_add(1)?, joined));
        }
    }
    None
}

/// A token renamed: an id with the text given, its place and mark kept (refcore's `retok`).
fn retok(t: &Tok, text: String) -> Tok {
    Tok {
        kind: Kind::Id,
        text,
        ws: t.ws,
        seq: t.seq,
        mark: Cell::new(t.mark.get()),
    }
}

/// A token renamed and read as an id (refcore's `retok` and its `x.nm = True`).
fn retok_named(t: &Tok, text: String) -> Tok {
    let mut mark = t.mark.get();
    mark.name = true;
    Tok {
        kind: Kind::Id,
        text,
        ws: t.ws,
        seq: t.seq,
        mark: Cell::new(mark),
    }
}

/// The level-0 blocks of a file's lines, `[start, end)`, its header first (refcore's `blocks`).
fn blocks(lines: &[Line]) -> Vec<(usize, usize)> {
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
fn line_word(line: &Line) -> &str {
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
fn is_builtin_id(id: &str) -> bool {
    BUILTIN_INPUTS.contains(&id)
        || BUILTIN_INDEXES.contains(&id)
        || PRELUDE_KINDS.contains(&id)
        || id == "cut"
}
