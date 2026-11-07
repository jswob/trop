//! The canonical format: each file reprinted with what the reality declares, and the reality's
//! hash over the reprints.
//! Spec: L §2.4 (the canonical format: header first, sections in kind order, kinds in tree
//! order, versions last by `known`; clause lines and notes first, then rules or version items;
//! the short form; a sorted `beats`; a text with a duplicated id or a key domain under a prefix
//! no `use` line names has no canonical form, tally 2.1.6); E §3.1 step 1 (the hash: the
//! sha256 of the files' canonical texts in the order of their paths).
//! Twin: engine/tools/refcore/canon.py (`reprint`, `unorderable`, `printed_hash`,
//! `FilePrinter`, `blocks`, `beats_sorted`, `kahn`, `BODY_RANK`, `SECTION_ORDER`,
//! `HEADER_ORDER`).
//! Invariants: every file is split and parsed, then checked for an id declared twice or an
//! unknown prefix, before any is ordered or reprinted (L §8.3, §2.4); a section's
//! lines change only by its parentheses (`parens.rs`) and the orders L §2.4 states; what the
//! spec leaves open keeps the file's order and spelling.

use super::canon_order::{
    Decls, KeyRef, Val, ascii_key, key_comp, known_day, row_key, sorted_by, span_key,
};
use super::item::{first, parse_file, prescan_units};
use super::lex::{Kind, Line, Mark, Role, Tok, split_lines};
use super::parens::canon_block;
use super::parse::Stop;
use super::spacing::line_text;
use super::tree::{
    Def, Domain, Expr, File, Governs, HeaderKind, Item, ItemForm, Key, KindDecl, KindSpec, Lit,
    Postfix, Rule, Section, Version,
};
use crate::api::Source;
use crate::base::{Code, Load, Place, Refusal};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// The header's lines in their order (L §2.4 "Header first: model, then tally, on, start,
/// end, use lines (by name), same lines (by left side), notes").
const HEADER_ORDER: [&str; 9] = [
    "tally", "on", "start", "end", "use", "same", "law", "from", "note",
];

/// A version's clause lines: they keep their order, before its notes and items.
const VERSION_CLAUSES: [&str; 9] = [
    "cite", "governs", "known", "enters", "assumed", "moves", "covers", "pad", "anchor",
];

/// The kind section's place in the section order (refcore's `SECTION_ORDER`).
const KIND_RANK: u8 = 2;

/// Each file's canonical text, in the order given, each reprinted with the declarations of
/// all of them (refcore's `reprint`), the files taken as one reality's.
///
/// # Errors
///
/// As [`reprint_among`].
pub fn reprint(sources: &[Source]) -> Result<Vec<(String, String)>, Stop> {
    reprint_among(sources, &[])
}

/// [`reprint`], where `members` holds, for each member of a joint model among the files, the
/// paths of the files given that are that member's, each member a reality of its own
/// (refcore's `reprint`, its `member_paths`).
///
/// # Errors
///
/// The first refusal met splitting, parsing or ordering (`LOAD-PARSE`; `LOAD-DUP` and
/// `LOAD-NAME` for a text with no canonical form; `LOAD-KEY`), or a form whose order the
/// engine doesn't build yet.
pub fn reprint_among(
    sources: &[Source],
    members: &[BTreeSet<String>],
) -> Result<Vec<(String, String)>, Stop> {
    let mut split = Vec::with_capacity(sources.len());
    for source in sources {
        split.push(split_lines(&source.text, &source.path)?);
    }
    let every: Vec<&[Line]> = split.iter().map(Vec::as_slice).collect();
    let units = prescan_units(&every);
    let mut files = Vec::with_capacity(sources.len());
    for (source, lines) in sources.iter().zip(&split) {
        files.push(parse_file(lines, &source.path, &units)?);
    }
    unorderable(&files, members)?;
    let printer = Printer {
        decls: Decls::of(&files),
        units,
    };
    let mut printed = Vec::with_capacity(sources.len());
    for ((source, lines), file) in sources.iter().zip(&split).zip(&files) {
        let text = printer.text(&source.path, lines, &file.sections)?;
        printed.push((source.path.clone(), text));
    }
    Ok(printed)
}

/// A text with no canonical form, refused as load refuses it, before anything is ordered
/// (refcore's `unorderable`; L §2.4, tally 2.1.6, #91): in each reality, the files given that
/// are no member's and each member's apart, a top-level id declared twice, then a key domain
/// under a prefix no model file's `use` line names (`LOAD-NAME unknown id <a>.<x>`, at its
/// section's line).
fn unorderable(files: &[File], members: &[BTreeSet<String>]) -> Result<(), Stop> {
    let taken: BTreeSet<&String> = members.iter().flatten().collect();
    let own: Vec<&File> = files.iter().filter(|f| !taken.contains(&f.path)).collect();
    declared_once(&own)?;
    for paths in members {
        let group: Vec<&File> = files.iter().filter(|f| paths.contains(&f.path)).collect();
        declared_once(&group)?;
    }
    let mut aliases: BTreeSet<&str> = BTreeSet::new();
    for file in files.iter().filter(|f| f.header.kind == HeaderKind::Model) {
        aliases.extend(file.header.uses.iter().map(|(alias, ..)| alias.as_str()));
    }
    for file in files {
        for section in &file.sections {
            for bind in section.binds() {
                if let Domain::Name {
                    id,
                    qual: Some(qual),
                } = &bind.domain
                    && !aliases.contains(qual.as_str())
                {
                    let message = format!("unknown id {qual}.{id}");
                    return Err(refuse(Load::Name, message, &file.path, section.id_line().1));
                }
            }
        }
    }
    Ok(())
}

/// One reality's files, each top-level id declared once: `LOAD-DUP <x> is declared twice` at
/// the second in the order of the files' paths, a scenario's file checked with the base's
/// files and never with another scenario's (L §2.5, §6.4); files of several models are
/// several realities, each file checked alone (refcore's `unorderable`, its groups).
fn declared_once(group: &[&File]) -> Result<(), Stop> {
    let models = group
        .iter()
        .filter(|f| f.header.kind == HeaderKind::Model)
        .count();
    if models > 1 {
        for file in group {
            declared_once_in(core::slice::from_ref(file))?;
        }
        return Ok(());
    }
    declared_once_in(group)
}

/// [`declared_once`] for one reality's files.
fn declared_once_in(group: &[&File]) -> Result<(), Stop> {
    let mut base: BTreeSet<String> = BTreeSet::new();
    for file in group
        .iter()
        .filter(|f| f.header.kind != HeaderKind::Scenario)
    {
        declare(&mut base, file)?;
    }
    for file in group
        .iter()
        .filter(|f| f.header.kind == HeaderKind::Scenario)
    {
        declare(&mut base.clone(), file)?;
    }
    Ok(())
}

/// Each section of `file` declared into `seen`: `LOAD-DUP` at one already there.
fn declare(seen: &mut BTreeSet<String>, file: &File) -> Result<(), Stop> {
    for section in &file.sections {
        let (id, line) = section.id_line();
        if !seen.insert(id.to_owned()) {
            let message = format!("{id} is declared twice");
            return Err(refuse(Load::Dup, message, &file.path, line));
        }
    }
    Ok(())
}

/// A refusal at a line of a file.
fn refuse(code: Load, message: String, file: &str, line: u32) -> Stop {
    Stop::Refused(Refusal::at(
        Code::Load(code),
        message,
        Place::Line(file.to_owned(), line),
    ))
}

/// The reality's hash: the sha256 of the canonical texts, each ending with its newline, joined
/// in the order given, as 64 lower-case hex digits (E §3.1 step 1).
#[must_use]
pub fn printed_hash(printed: &[(String, String)]) -> String {
    let mut hasher = Sha256::new();
    for (_, text) in printed {
        hasher.update(text.as_bytes());
    }
    let mut hex = String::new();
    for byte in hasher.finalize() {
        for digit in [byte.checked_div(16), byte.checked_rem(16)] {
            if let Some(c) = digit.and_then(|d| char::from_digit(u32::from(d), 16)) {
                hex.push(c);
            }
        }
    }
    hex
}

/// A section's tree and its lines, made canonical.
type Block = (Section, Vec<Line>);

/// Reprints the files of one reality (refcore's `FilePrinter`), each named as the run names it.
struct Printer {
    /// The unit names a number may carry.
    units: BTreeSet<String>,
    /// What the reality declares.
    decls: Decls,
}

impl Printer {
    /// The file's canonical text: its header, then its sections, one blank line between them.
    fn text(&self, file: &str, lines: &[Line], sections: &[Section]) -> Result<String, Stop> {
        let blocks = blocks(lines);
        let Some((head, rest)) = blocks.split_first() else {
            return Err(invariant("a file with no header reprinted"));
        };
        let mut parts = vec![Self::header(head)?.join("\n")];
        let mut secs: Vec<Block> = Vec::with_capacity(rest.len());
        for (_, block) in sections.iter().zip(rest) {
            let mut block = block.to_vec();
            let node = canon_block(&mut block, file, &self.units)?;
            secs.push((node, block));
        }
        for (node, block) in order_sections(secs) {
            parts.push(self.section(file, &node, &block)?.join("\n"));
        }
        let mut text = parts.join("\n\n");
        text.push('\n');
        Ok(text)
    }

    /// The header: its first line, then its lines in L §2.4's order, `use` lines by name and
    /// `same` lines by left side, the others in the file's order.
    fn header(block: &[Line]) -> Result<Vec<String>, Stop> {
        let Some((head, rest)) = block.split_first() else {
            return Err(invariant("a header with no line reprinted"));
        };
        let mut out = vec![line_text(&head.toks)?];
        for word in HEADER_ORDER {
            let lines: Vec<&Line> = rest.iter().filter(|line| first(line) == word).collect();
            let keys = match word {
                "use" => {
                    let mut keys = Vec::with_capacity(lines.len());
                    for line in &lines {
                        let alias = line.toks.get(1).map_or("", |tok| tok.text.as_str());
                        keys.push(key_of(alias));
                    }
                    keys
                }
                "same" => {
                    let mut keys = Vec::with_capacity(lines.len());
                    for line in &lines {
                        let eq = line
                            .toks
                            .iter()
                            .position(|tok| tok.text == "=")
                            .unwrap_or(line.toks.len());
                        keys.push(key_of(&line_text(slice(&line.toks, 1, eq)?)?));
                    }
                    keys
                }
                _ => lines.iter().map(|_| Vec::new()).collect(),
            };
            for at in sorted_by(keys)? {
                if let Some(line) = lines.get(at) {
                    out.push(format!("  {}", line_text(&line.toks)?));
                }
            }
        }
        Ok(out)
    }

    /// One section's lines.
    fn section(&self, file: &str, node: &Section, block: &[Line]) -> Result<Vec<String>, Stop> {
        let rank: Option<fn(&str) -> u8> = match node {
            Section::Def(def) => return self.def(file, def, block),
            Section::Version(version) => return self.version(file, version, block),
            Section::Kind(_) => Some(kind_rank),
            Section::Index(_) => Some(index_rank),
            Section::Input(_) | Section::Band(_) => Some(note_rank),
            Section::Fill(_) => Some(fill_rank),
            Section::Move(_) => Some(move_rank),
            Section::Source(_)
            | Section::Unit(_)
            | Section::Place(_)
            | Section::Ask(_)
            | Section::Axis(_) => None,
        };
        let Some((head, body)) = block.split_first() else {
            return Err(invariant("a section with no line reprinted"));
        };
        let mut out = vec![line_text(&head.toks)?];
        let keys: Vec<Vec<Val>> = body
            .iter()
            .map(|line| match rank {
                Some(rank) => vec![Val::Int(i64::from(rank(first(line))))],
                None => Vec::new(),
            })
            .collect();
        for at in sorted_by(keys)? {
            if let Some(line) = body.get(at) {
                out.push(format!("  {}", line_text(&line.toks)?));
            }
        }
        Ok(out)
    }

    /// A def: the short form where L §2.4 prints it, else its head, notes and sorted rules.
    fn def(&self, file: &str, def: &Def, block: &[Line]) -> Result<Vec<String>, Stop> {
        let Some((head, body)) = block.split_first() else {
            return Err(invariant("a def with no line reprinted"));
        };
        let ht = &head.toks;
        let at = ht.iter().position(|tok| tok.mark.get().role == Role::DefAt);
        if def.short && !def.notes.is_empty() {
            // L §2.4 (tally 2.1, R-11): "A one-line def followed by note lines means the long
            // form, the same def with its one rule labelled base and those notes, and prints
            // as it"
            let eq = ht
                .iter()
                .position(|tok| tok.mark.get().role == Role::DefEq)
                .ok_or_else(|| invariant("a one-line def with no ="))?;
            let a = at.unwrap_or(ht.len());
            let mut line = slice(ht, 0, eq)?.to_vec();
            line.extend_from_slice(slice(ht, a, ht.len())?);
            let mut out = vec![line_text(&line)?];
            for note in body {
                out.push(format!("  {}", line_text(&note.toks)?));
            }
            let value = slice(ht, after(eq)?, a)?;
            out.push(format!("  rule base = {}", line_text(value)?));
            return Ok(out);
        }
        if def.short {
            return Ok(vec![line_text(ht)?]);
        }
        let notes: Vec<&Line> = body.iter().filter(|l| first(l) == "note").collect();
        let rules: Vec<&Line> = body.iter().filter(|l| first(l) == "rule").collect();
        if let ([rule], [line], true) = (def.rules.as_slice(), rules.as_slice(), notes.is_empty())
            && is_plain_base(rule)
        {
            // L §2.4 Short form: "a def with one rule labelled base, no v, no when, no own
            // governs, no notes prints as def x: T = expr at E"
            let a = at.unwrap_or(ht.len());
            let rt = &line.toks;
            let eq = rt
                .iter()
                .position(|tok| tok.text == "=")
                .ok_or_else(|| invariant("a rule with no ="))?;
            let mut short = slice(ht, 0, a)?.to_vec();
            short.push(op("="));
            short.extend_from_slice(slice(rt, after(eq)?, rt.len())?);
            short.extend_from_slice(slice(ht, a, ht.len())?);
            return Ok(vec![line_text(&short)?]);
        }
        let mut out = vec![line_text(ht)?];
        for note in &notes {
            out.push(format!("  {}", line_text(&note.toks)?));
        }
        let mut keys = Vec::with_capacity(rules.len());
        for (rule, line) in def.rules.iter().zip(&rules) {
            keys.push(self.rule_key(file, rule, &def.id, line)?);
        }
        for index in sorted_by(keys)? {
            if let Some(line) = rules.get(index) {
                out.push(format!("  {}", rule_text(&line.toks)?));
            }
        }
        Ok(out)
    }

    /// A rule's sort key (L §2.4, round 19, GD-24): its label, the known date of its `v` (a rule
    /// without `v` first), its own `governs`, then its own canonical line (GD-28); a `v` that
    /// names no version is `LOAD-NAME`.
    fn rule_key(&self, file: &str, rule: &Rule, def: &str, line: &Line) -> Result<Vec<Val>, Stop> {
        let v = match &rule.v {
            None => Val::Tup(vec![Val::Int(0)]),
            Some(v) => match self.decls.known(v) {
                Some(known) => Val::Tup(vec![Val::Int(1), Val::Day(known_day(known))]),
                None => {
                    return Err(Stop::Refused(Refusal::at(
                        Code::Load(Load::Name),
                        format!("rule {def}.{}: v {v} names no version", rule.label),
                        Place::Line(file.to_owned(), rule.line),
                    )));
                }
            },
        };
        Ok(vec![
            Val::Text(rule.label.clone()),
            v,
            rule_span(file, rule),
            ascii_key(rule_text(&line.toks)?),
        ])
    }

    /// A version: its head, its clause lines, its notes, then its items sorted (L §2.4).
    fn version(&self, file: &str, version: &Version, block: &[Line]) -> Result<Vec<String>, Stop> {
        let Some((head, body)) = block.split_first() else {
            return Err(invariant("a version with no line reprinted"));
        };
        let mut out = vec![line_text(&head.toks)?];
        let clause = |line: &&Line| VERSION_CLAUSES.contains(&first(line));
        let note = |line: &&Line| first(line) == "note";
        for line in body.iter().filter(clause).chain(body.iter().filter(note)) {
            out.push(format!("  {}", line_text(&line.toks)?));
        }
        let items: Vec<&Line> = body.iter().filter(|l| !clause(l) && !note(l)).collect();
        let mut keys = Vec::with_capacity(items.len());
        for (item, line) in version.items.iter().zip(&items) {
            keys.push(self.item_key(file, item, line)?);
        }
        for at in sorted_by(keys)? {
            if let (Some(item), Some(line)) = (version.items.get(at), items.get(at)) {
                out.push(format!("  {}", item_text(item, line)?));
            }
        }
        Ok(out)
    }

    /// A version item's sort key (L §2.4): entries first, then by target id, keys in index
    /// order (a keyless row first), then its own span or instant, then its own canonical line.
    fn item_key(&self, file: &str, item: &Item, line: &Line) -> Result<Vec<Val>, Stop> {
        let place = Place::Line(file.to_owned(), item.line);
        match &item.form {
            ItemForm::Enter(enter) => {
                let keys = enter.keys.iter().map(|k| Val::Text(k.text())).collect();
                Ok(vec![
                    Val::Int(0),
                    Val::Text(target(enter.qual.as_deref(), &enter.target)),
                    Val::Tup(vec![Val::Int(1), Val::Tup(keys)]),
                ])
            }
            ItemForm::Rule { qual, def, rule } => Ok(vec![
                Val::Int(1),
                Val::Text(target(qual.as_deref(), def)),
                Val::Tup(vec![Val::Int(0)]),
                rule_span(file, rule),
                ascii_key(item_text(item, line)?),
            ]),
            ItemForm::Same(pair) => self.same_key(&pair.0, item, line, &place),
            ItemForm::Band(row) => {
                let keys: Vec<KeyRef> = row.key.iter().cloned().map(KeyRef::One).collect();
                self.keyed(row.qual.as_deref(), &row.target, &keys, &place)
            }
            ItemForm::Row(row) => {
                let keys: Vec<KeyRef> = row.keys.iter().flatten().map(row_key).collect();
                let mut comps = self.keyed(row.qual.as_deref(), &row.target, &keys, &place)?;
                comps.push(span_key(row.governs.as_ref(), row.instant.as_ref(), &place));
                comps.push(ascii_key(item_text(item, line)?));
                Ok(comps)
            }
        }
    }

    /// A row's first components: its target and its keys, each in its index's order.
    fn keyed(
        &self,
        qual: Option<&str>,
        name: &str,
        keys: &[KeyRef],
        place: &Place,
    ) -> Result<Vec<Val>, Stop> {
        let mut comps = vec![
            Val::Int(1),
            Val::Text(target(qual, name)),
            Val::Tup(vec![Val::Int(i64::from(!keys.is_empty()))]),
        ];
        if !keys.is_empty() {
            let mut each = Vec::with_capacity(keys.len());
            for (position, key) in keys.iter().enumerate() {
                let order = self
                    .decls
                    .key_order(self.decls.domain(qual, name, position));
                each.push(key_comp(key, &order, place)?);
            }
            comps.push(Val::Keys(each));
        }
        Ok(comps)
    }

    /// A `same` item's key (L §2.4, round 19, GD-24): "by its left side (as an item whose
    /// target and keys are its left side's)"; it has no span of its own.
    fn same_key(
        &self,
        left: &Expr,
        item: &Item,
        line: &Line,
        place: &Place,
    ) -> Result<Vec<Val>, Stop> {
        let (name, keys) = same_side(left)?;
        let mut refs = Vec::with_capacity(keys.len());
        for key in keys {
            refs.push(same_key(key)?);
        }
        let mut comps = self.keyed(None, &name, &refs, place)?;
        comps.push(Val::Tie(0, Vec::new()));
        comps.push(ascii_key(item_text(item, line)?));
        Ok(comps)
    }
}

/// A `same` item's left side (L §2.3 `sref`) as a target and its keys (refcore's `item_key`
/// for `same`): a side that ends in keys is keyed by them, its target what comes before them
/// (`x[k]`, `a.x[k]`, `a.b.x[k]`); any other side is its own target (`x`, `a.x`, `a.b.c`,
/// `x[k].f`), written as the side is.
fn same_side(left: &Expr) -> Result<(String, &[Key]), Stop> {
    let no_id = || {
        invariant(
            "unreachable: since tally 2.1.6 (#88, L §2.3 `sref`) the parser refuses with \
             LOAD-PARSE a `same` side that isn't an id with its fields and keys, so `reprint` \
             never orders one",
        )
    };
    if let Expr::Name(id) = left {
        return Ok((id.clone(), &[]));
    }
    let Expr::Post(base, fixes) = left else {
        return Err(no_id());
    };
    let Expr::Name(head) = &**base else {
        return Err(no_id());
    };
    let (keys, before): (&[Key], &[Postfix]) = match fixes.split_last() {
        Some((Postfix::Keys(keys), before)) => (keys, before),
        _ => (&[], fixes),
    };
    let mut target = head.clone();
    for fix in before {
        match fix {
            Postfix::Field(field) => {
                target.push('.');
                target.push_str(field);
            }
            Postfix::Keys(group) => {
                let texts = group
                    .iter()
                    .map(same_key_text)
                    .collect::<Result<Vec<_>, _>>()?;
                target.push('[');
                target.push_str(&texts.join(", "));
                target.push(']');
            }
        }
    }
    Ok((target, keys))
}

/// A `same` side's key as written (`sref`'s keys are literals or places keyed by literals,
/// L §2.3).
fn same_key_text(key: &Key) -> Result<String, Stop> {
    match key {
        Key::Expr(Expr::Post(base, fixes)) => place_key_text(base, fixes),
        Key::Star | Key::Expr(_) => lit_key_text(key),
    }
}

/// A literal key as written.
fn lit_key_text(key: &Key) -> Result<String, Stop> {
    match key {
        Key::Expr(Expr::Lit(lit)) => Ok(lit.text()),
        Key::Expr(Expr::Name(name)) => Ok(name.clone()),
        Key::Star | Key::Expr(_) => Err(no_literal()),
    }
}

/// A place keyed by things as a `same` side's key, `konto[ana]`, as written.
fn place_key_text(base: &Expr, fixes: &[Postfix]) -> Result<String, Stop> {
    let (Expr::Name(name), [Postfix::Keys(keys)]) = (base, fixes) else {
        return Err(no_literal());
    };
    let texts = keys
        .iter()
        .map(lit_key_text)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("{name}[{}]", texts.join(", ")))
}

/// A `same` item's key position as a key: a literal, or a place keyed by things, one key
/// sorted as things are (L §2.4), as a row's key.
fn same_key(key: &Key) -> Result<KeyRef, Stop> {
    match key {
        Key::Expr(Expr::Lit(lit)) => Ok(KeyRef::One(lit.clone())),
        Key::Expr(Expr::Name(name)) => Ok(KeyRef::One(Lit::Name(name.clone()))),
        Key::Expr(Expr::Post(base, fixes)) => Ok(KeyRef::Place(place_key_text(base, fixes)?)),
        Key::Star | Key::Expr(_) => Err(no_literal()),
    }
}

/// A `same` side keyed by what the parser's `sref` never takes.
fn no_literal() -> Stop {
    invariant(
        "unreachable: since tally 2.1.6 (#88, L §2.3 `sref`) the parser refuses with LOAD-PARSE \
         a `same` side keyed by anything but literals or places keyed by literals, so `reprint` \
         never orders one (T's `format/same_key_signed`)",
    )
}

/// A target's id, with its member's prefix.
fn target(qual: Option<&str>, name: &str) -> String {
    match qual {
        Some(q) => format!("{q}.{name}"),
        None => name.to_owned(),
    }
}

/// A rule's span component, by its own `governs`, at its line of `file`.
fn rule_span(file: &str, rule: &Rule) -> Val {
    let governs = rule.governs.clone().map(Governs::Span);
    span_key(
        governs.as_ref(),
        None,
        &Place::Line(file.to_owned(), rule.line),
    )
}

/// Whether a long def's one rule prints in the short form: labelled `base`, with no `v`, no
/// `when`, no own `governs` and no `beats`.
fn is_plain_base(rule: &Rule) -> bool {
    rule.label == "base"
        && rule.v.is_none()
        && rule.when.is_none()
        && rule.governs.is_none()
        && rule.beats.is_empty()
}

/// A rule's line, its `beats` list sorted (L §2.4 "A beats list is sorted by label, with
/// ', '").
fn rule_text(toks: &[Tok]) -> Result<String, Stop> {
    if toks.get(2).is_some_and(|tok| tok.text == "beats") {
        return Ok(format!(
            "{} {}",
            line_text(slice(toks, 0, 3)?)?,
            beats_sorted(toks, 3)?
        ));
    }
    line_text(toks)
}

/// A version item's line; a rule's replacement has its `beats` list sorted.
fn item_text(item: &Item, line: &Line) -> Result<String, Stop> {
    let toks = &line.toks;
    let mut beats = None;
    for (k, tok) in toks.iter().enumerate().skip(5) {
        if tok.text == "beats" {
            beats = Some(k);
            break;
        }
    }
    if let ItemForm::Rule { .. } = item.form
        && let Some(k) = beats
    {
        let rest = after(k)?;
        return Ok(format!(
            "{} {}",
            line_text(slice(toks, 0, rest)?)?,
            beats_sorted(toks, rest)?
        ));
    }
    line_text(toks)
}

/// The labels of a `beats` list from `toks[from]`, sorted, and the rest of the line.
fn beats_sorted(toks: &[Tok], from: usize) -> Result<String, Stop> {
    let mut labels: Vec<String> = Vec::new();
    let mut rest = slice(toks, from, toks.len())?;
    while let Some((label, tail)) = rest.split_first() {
        labels.push(label.text.clone());
        rest = tail;
        match rest.split_first() {
            Some((comma, next)) if comma.text == "," => rest = next,
            Some(_) | None => break,
        }
    }
    labels.sort();
    let mut out = labels.join(", ");
    if !rest.is_empty() {
        out.push(' ');
        out.push_str(&line_text(rest)?);
    }
    Ok(out)
}

/// Kinds' body lines keep their order (a kind's leg, attr, note and parts lines are one
/// repeated choice); other words after them.
fn kind_rank(word: &str) -> u8 {
    match word {
        "note" | "leg" | "attr" | "owned" | "batched" | "step" | "takes" => 0,
        _ => 99,
    }
}

/// An index's lines: `v`, `assumed`, then notes.
fn index_rank(word: &str) -> u8 {
    match word {
        "v" => 0,
        "assumed" => 1,
        "note" => 2,
        _ => 99,
    }
}

/// An input's or a band's lines: notes first.
fn note_rank(word: &str) -> u8 {
    if word == "note" { 0 } else { 99 }
}

/// A fill's lines in the grammar's order.
fn fill_rank(word: &str) -> u8 {
    match word {
        "v" => 0,
        "note" => 1,
        "supply" => 2,
        "demand" => 3,
        "arc" => 4,
        "order" => 5,
        _ => 99,
    }
}

/// A move's lines in the grammar's order.
fn move_rank(word: &str) -> u8 {
    match word {
        "on" => 1,
        "at" => 2,
        "for" => 3,
        "when" => 4,
        "reads" => 5,
        "before" | "after" => 6,
        "ties" => 7,
        "attr" => 8,
        "leg" => 9,
        "fulfils" => 10,
        "note" => 11,
        _ => 99,
    }
}

/// The sections in L §2.4's order: by kind of section, then by id; versions last by `known`;
/// kinds in tree order (refcore's `order_sections`).
fn order_sections(secs: Vec<Block>) -> Vec<Block> {
    let (kinds, mut others): (Vec<Block>, Vec<Block>) = secs
        .into_iter()
        .partition(|(node, _)| matches!(node, Section::Kind(_)));
    others.sort_by(|(a, _), (b, _)| section_key(a).cmp(&section_key(b)));
    let (before, after): (Vec<Block>, Vec<Block>) = others
        .into_iter()
        .partition(|(node, _)| section_key(node).0 < KIND_RANK);
    let mut out = before;
    out.extend(kahn(kinds));
    out.extend(after);
    out
}

/// A section's place: its kind's rank, a version's `known` (others none), its id.
fn section_key(node: &Section) -> (u8, Option<u32>, &str) {
    match node {
        Section::Source(x) => (0, None, &x.id),
        Section::Unit(x) => (1, None, &x.id),
        Section::Kind(x) => (KIND_RANK, None, &x.id),
        Section::Axis(x) => (3, None, &x.id),
        Section::Index(x) => (4, None, &x.id),
        Section::Place(x) => (5, None, &x.id),
        Section::Input(x) => (6, None, &x.id),
        Section::Band(x) => (7, None, &x.id),
        Section::Def(x) => (8, None, &x.id),
        Section::Fill(x) => (9, None, &x.id),
        Section::Move(x) => (10, None, &x.id),
        Section::Ask(x) => (11, None, &x.id),
        Section::Version(x) => (12, Some(known_day(x.known).number()), &x.id),
    }
}

/// A kind section's declaration.
fn kind_decl(node: &Section) -> Option<&KindDecl> {
    match node {
        Section::Kind(kind) => Some(kind),
        Section::Source(_)
        | Section::Version(_)
        | Section::Unit(_)
        | Section::Index(_)
        | Section::Place(_)
        | Section::Input(_)
        | Section::Band(_)
        | Section::Def(_)
        | Section::Move(_)
        | Section::Ask(_)
        | Section::Axis(_)
        | Section::Fill(_) => None,
    }
}

/// The kinds in tree order (L §2.4, round 19, GD-9): each kind followed by its subkinds, depth
/// first, kinds of one parent (or none) by id; a kind whose parent this file doesn't print heads
/// a tree; a cycle of parents prints by id (refcore's `kahn`). One section per id: the last.
fn kahn(kinds: Vec<Block>) -> Vec<Block> {
    let decls: Vec<&KindDecl> = kinds.iter().filter_map(|(n, _)| kind_decl(n)).collect();
    let ids: BTreeSet<&str> = decls.iter().map(|k| k.id.as_str()).collect();
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    let mut children: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut roots: BTreeSet<&str> = BTreeSet::new();
    for (at, kind) in decls.iter().enumerate() {
        by.insert(&kind.id, at);
        match &kind.spec {
            KindSpec::Sub { parent, .. } if ids.contains(parent.as_str()) && *parent != kind.id => {
                match children.get_mut(parent.as_str()) {
                    Some(under) => {
                        under.insert(&kind.id);
                    }
                    None => {
                        children.insert(parent, BTreeSet::from([kind.id.as_str()]));
                    }
                }
            }
            KindSpec::Sub { .. }
            | KindSpec::Plain
            | KindSpec::Event(_)
            | KindSpec::Unit
            | KindSpec::Place { .. } => {
                roots.insert(&kind.id);
            }
        }
    }
    let mut order: Vec<usize> = Vec::with_capacity(decls.len());
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = roots.into_iter().rev().collect();
    while let Some(id) = stack.pop() {
        if !done.insert(id) {
            continue;
        }
        if let Some(at) = by.get(id) {
            order.push(*at);
        }
        if let Some(under) = children.get(id) {
            stack.extend(under.iter().rev().copied());
        }
    }
    for id in &ids {
        if !done.contains(id)
            && let Some(at) = by.get(id)
        {
            order.push(*at);
        }
    }
    let mut slots: Vec<Option<Block>> = kinds.into_iter().map(Some).collect();
    order
        .into_iter()
        .filter_map(|at| slots.get_mut(at).and_then(Option::take))
        .collect()
}

/// The file's blocks: each line at the start of one with the lines indented under it.
fn blocks(lines: &[Line]) -> Vec<&[Line]> {
    let mut out = Vec::new();
    let mut rest = lines;
    while let Some((head, tail)) = rest.split_first() {
        let under = tail.iter().take_while(|l| l.level > head.level).count();
        let Some((block, next)) = under.checked_add(1).and_then(|n| rest.split_at_checked(n))
        else {
            break;
        };
        out.push(block);
        rest = next;
    }
    out
}

/// The position after `at`.
fn after(at: usize) -> Result<usize, Stop> {
    at.checked_add(1)
        .ok_or_else(|| invariant("a line's tokens out of range"))
}

/// Tokens `from..to` of a line.
fn slice(toks: &[Tok], from: usize, to: usize) -> Result<&[Tok], Stop> {
    toks.get(from..to)
        .ok_or_else(|| invariant("a line's tokens out of range"))
}

/// An operator token the reprint adds.
fn op(text: &str) -> Tok {
    Tok {
        kind: Kind::Op,
        text: text.to_owned(),
        ws: true,
        seq: u32::MAX,
        mark: Cell::new(Mark::default()),
    }
}

/// A text as a sort key.
fn key_of(text: &str) -> Vec<Val> {
    vec![Val::Text(text.to_owned())]
}

/// An `ENGINE-INVARIANT` refusal: the reprint met a line its parse doesn't explain.
fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}
