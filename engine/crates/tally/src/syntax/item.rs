//! The file grammar: the header, the sections, a version's items and rows, and the units prescan.
//! Spec: L §2.3 (`file`, `header`, `section`, `version`, `known`, `vitem`, `enter`, `row`,
//! `bandrow`, `value`, `rowmeta`, `instant`, `interval`), L §5.1 (the header's checks), L §8.3
//! (which refusal prints: split every file, then parse). The declaring sections are
//! `decls.rs`'s.
//! Twin: engine/tools/refcore/parser.py (`parse_header`, `parse_section`, `_notes`, `_flat`,
//! `sec_version`, `vitem`, `value`, `rowmeta`, `interval`, `parse_file`, `prescan_units`).
//! Invariants: every file is split into lines before any is parsed; each section reads only
//! the lines indented under it; the error texts are refcore's word for word.

use super::ask::ask;
use super::decls::{
    axis, band, def, fill, index, input, kind, lit_list, move_section, place, place_unit,
    rule_body, source, unit,
};
use super::expr::{Ends, Rec};
use super::lex::{Kind, Line, split_lines};
use super::parse::{P, Parens, Stop};
use super::reads::dist_kind;
use super::tree::{
    BandRow, Edge, Enter, Expr, File, Gap, Governs, Header, HeaderKind, Instant, Interval, Item,
    ItemForm, Known, LSpan, Lit, Row, RowKey, RuleValue, Section, Value, Version,
};
use crate::api::Source as Text;
use crate::base::{Code, Load, Place as At, Refusal};
use crate::num::Num;
use crate::time::Day;
use std::cell::Cell;
use std::collections::BTreeSet;

/// The deepest an `as if` may nest (L §2.3, `LOAD-ASIF`).
const MAX_ASIF: u8 = 4;

/// Splits every file, then finds the unit names, then parses every file, in the order given
/// (the read step's, by full path). L §8.3.
///
/// # Errors
///
/// The first refusal met, or the first construct the engine doesn't build yet.
pub fn parse_reality(sources: &[Text]) -> Result<Vec<File>, Stop> {
    let mut split = Vec::with_capacity(sources.len());
    for source in sources {
        split.push(split_lines(&source.text, &source.path)?);
    }
    let files: Vec<(&str, &[Line])> = sources
        .iter()
        .zip(&split)
        .map(|(source, lines)| (source.path.as_str(), lines.as_slice()))
        .collect();
    parse_split(&files)
}

/// Finds the unit names across files already split into lines, then parses each in the order
/// of their full paths, by code point, and gives them back in the order given: a world's
/// files, each with its path (refcore's `load_lines` and its `by_path`). L §8.3.
///
/// # Errors
///
/// The first refusal met, or the first construct the engine doesn't build yet.
pub fn parse_split(split: &[(&str, &[Line])]) -> Result<Vec<File>, Stop> {
    let every: Vec<&[Line]> = split.iter().map(|(_, file)| *file).collect();
    let units = prescan_units(&every);
    let order: BTreeSet<(&str, usize)> = split
        .iter()
        .enumerate()
        .map(|(at, (path, _))| (*path, at))
        .collect();
    let mut parsed: Vec<Option<File>> = split.iter().map(|_| None).collect();
    for (path, at) in order {
        let (Some((_, lines)), Some(slot)) = (split.get(at), parsed.get_mut(at)) else {
            return Err(Stop::Refused(Refusal::new(
                Code::EngineInvariant,
                "a file's place past the files".to_owned(),
            )));
        };
        *slot = Some(parse_file(lines, path, &units)?);
    }
    Ok(parsed.into_iter().flatten().collect())
}

/// One file's lines parsed with the unit names `units` (refcore's `parse_file`): a joint
/// model's merge reads what each member declares this way. L §2.3, §6.1.
///
/// # Errors
///
/// As [`parse_split`].
pub(crate) fn parse_lines(
    lines: &[Line],
    file: &str,
    units: &BTreeSet<String>,
) -> Result<File, Stop> {
    parse_file(lines, file, units)
}

/// The first token's text of a line, or nothing.
pub(super) fn first(line: &Line) -> &str {
    line.toks.first().map_or("", |tok| tok.text.as_str())
}

/// The unit names a number may carry: declared `unit` ids, and the keys entered into kinds of
/// unit (refcore's `prescan_units`, L §3.2).
pub(crate) fn prescan_units(files: &[&[Line]]) -> BTreeSet<String> {
    let mut units = BTreeSet::new();
    let mut unit_kinds = BTreeSet::from(["currencies".to_owned()]);
    for line in files.iter().copied().flatten() {
        let top = line.level == 0 || (line.level == 1 && first(line) == "kind");
        if !top {
            continue;
        }
        if let [head, id, ..] = line.toks.as_slice()
            && head.text == "unit"
            && id.kind == Kind::Id
        {
            units.insert(id.text.clone());
        }
        if let [head, id, colon, word] = line.toks.as_slice()
            && head.text == "kind"
            && colon.text == ":"
            && word.text == "unit"
        {
            unit_kinds.insert(id.text.clone());
        }
    }
    for line in files.iter().copied().flatten() {
        let toks = match line.toks.split_first() {
            Some((head, rest)) if head.text == "replace" => rest,
            Some(_) | None => line.toks.as_slice(),
        };
        if let [kind, plus, open, rest @ ..] = toks
            && kind.kind == Kind::Id
            && unit_kinds.contains(&kind.text)
            && plus.text == "+"
            && open.text == "["
        {
            for tok in rest.iter().take_while(|tok| tok.text != "]") {
                if tok.kind == Kind::Id || tok.kind == Kind::Key {
                    units.insert(tok.text.clone());
                }
            }
        }
    }
    units
}

/// The lines at the front of `after` indented deeper than `level`, and the lines after them.
fn children(after: &[Line], level: u32) -> (&[Line], &[Line]) {
    let count = after.iter().take_while(|line| line.level > level).count();
    after.split_at_checked(count).unwrap_or((after, &[]))
}

/// A file: its header, then its sections, each at level 0; once every section parses, the
/// first whose `as if` nests deeper than 4 is `LOAD-ASIF` at its line (refcore's
/// `parse_file`, L §2.3).
pub(super) fn parse_file(
    lines: &[Line],
    file: &str,
    units: &BTreeSet<String>,
) -> Result<File, Stop> {
    let Some((head, under_head)) = lines.split_first() else {
        return Err(Stop::Refused(Refusal::at(
            Code::Load(Load::Header),
            "file has no header".to_owned(),
            At::File(file.to_owned()),
        )));
    };
    let (header_body, mut rest) = children(under_head, 0);
    let asif = Cell::new(0);
    let header = parse_header(
        &P::new(&head.toks, file, head.number, units, &asif),
        head,
        header_body,
    )?;
    let mut sections = Vec::new();
    let mut too_deep = None;
    while let Some((line, under)) = rest.split_first() {
        let mut p = P::new(&line.toks, file, line.number, units, &asif);
        if line.level != 0 {
            return Err(p.cannot("indentation"));
        }
        let (body, next) = children(under, 0);
        p.i = 1;
        asif.set(0);
        sections.push(parse_section(&mut p, first(line), body)?);
        if too_deep.is_none() && asif.get() > MAX_ASIF {
            too_deep = Some(p.refuse(Load::AsIf, "as if deeper than 4".to_owned()));
        }
        rest = next;
    }
    if let Some(refusal) = too_deep {
        return Err(refusal);
    }
    Ok(File {
        path: file.to_owned(),
        header,
        sections,
    })
}

/// One section from its lines, its first line at level 0 and the lines under it, parsed again
/// for the reprint (refcore's `parse_section`, `canon.reparse`): each token's mark is cleared
/// first; `Parens::Blind` parses for the reprint's comparison (no parentheses, no source
/// texts).
pub(super) fn parse_block(
    block: &[Line],
    file: &str,
    units: &BTreeSet<String>,
    parens: Parens,
) -> Result<Section, Stop> {
    for tok in block.iter().flat_map(|line| &line.toks) {
        tok.mark.set(tok.mark.get().cleared());
    }
    let Some((head, body)) = block.split_first() else {
        return Err(Stop::Refused(Refusal::at(
            Code::Load(Load::Parse),
            "cannot parse: a section reprinted".to_owned(),
            At::File(file.to_owned()),
        )));
    };
    let asif = Cell::new(0);
    let mut p = P::new(&head.toks, file, head.number, units, &asif);
    p.blind = parens == Parens::Blind;
    if head.level != 0 {
        return Err(p.cannot("indentation"));
    }
    p.i = 1;
    parse_section(&mut p, first(head), body)
}

/// The header lines a header kind takes, in order.
fn header_order(kind: HeaderKind) -> &'static [&'static str] {
    match kind {
        HeaderKind::Model => &["tally", "on", "start", "end", "use", "same", "note"],
        HeaderKind::Part => &["tally", "law", "note"],
        HeaderKind::Scenario => &["tally", "from", "note"],
    }
}

/// The header: checked before the grammar (L §5.1), then parsed; `p` is over its first line.
fn parse_header(p: &P<'_>, head: &Line, body: &[Line]) -> Result<Header, Stop> {
    let refuse = |message: &str| p.refuse(Load::Header, message.to_owned());
    let kind = match (head.level, first(head)) {
        (0, "model") => HeaderKind::Model,
        (0, "part") => HeaderKind::Part,
        (0, "scenario") => HeaderKind::Scenario,
        _ => return Err(refuse("file has no model, part or scenario header")),
    };
    let words: Vec<&str> = body.iter().filter(|l| l.level == 1).map(first).collect();
    if !words.contains(&"tally") {
        return Err(refuse("missing tally version"));
    }
    if kind == HeaderKind::Model {
        if !words.contains(&"on") {
            return Err(refuse("missing on date"));
        }
        if !words.contains(&"start") || !words.contains(&"end") {
            return Err(refuse("missing start or end"));
        }
    }
    if kind == HeaderKind::Scenario && !words.contains(&"from") {
        return Err(refuse("missing from date"));
    }
    let mut q = p.under(head);
    let id = q.ident("an id")?;
    let over = if kind == HeaderKind::Scenario {
        q.expect("over")?;
        Some(q.ident("an id")?)
    } else {
        None
    };
    q.end()?;
    let mut header = Header {
        kind,
        id,
        over,
        tally: Num::zero(),
        on: None,
        start: None,
        end: None,
        uses: Vec::new(),
        sames: Vec::new(),
        notes: Vec::new(),
        law: false,
        from: None,
        line: head.number,
    };
    header_lines(p, &mut header, body)?;
    if let (HeaderKind::Model, Some(start), Some(stop)) = (kind, header.start, header.end)
        && start > stop
    {
        return Err(refuse("start after end"));
    }
    Ok(header)
}

/// The header's own lines, in their order, each single one once.
fn header_lines(p: &P<'_>, header: &mut Header, body: &[Line]) -> Result<(), Stop> {
    let order = header_order(header.kind);
    let mut pos = 0_usize;
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for line in body {
        if line.level != 1 {
            return Err(p.cannot_at("indentation in the header", line.number));
        }
        let word = first(line);
        let Some(k) = order.iter().position(|w| *w == word) else {
            return Err(p.cannot_at(&format!("{word} in a header"), line.number));
        };
        if k < pos {
            let what = format!("header line {word} out of order");
            return Err(p.cannot_at(&what, line.number));
        }
        let single = ["tally", "on", "start", "end", "law", "from"].contains(&word);
        if single && k == pos && seen.contains(word) {
            return Err(p.cannot_at(&format!("header line {word} twice"), line.number));
        }
        seen.insert(word);
        pos = k;
        let mut q = p.under(line);
        header_line(header, word, &mut q)?;
        q.end()?;
    }
    Ok(())
}

/// One header line's value.
fn header_line(header: &mut Header, word: &str, q: &mut P<'_>) -> Result<(), Stop> {
    match word {
        "tally" => header.tally = q.integer()?,
        "on" => header.on = Some(q.day()?),
        "start" => header.start = Some(q.day()?),
        "end" => header.end = Some(q.day()?),
        "law" => header.law = true,
        "from" => header.from = Some(q.day()?),
        "use" => {
            let alias = q.ident("an id")?;
            q.expect("=")?;
            let path = q.string()?;
            q.expect("sha256")?;
            let hash = q.string()?;
            header.uses.push((alias, path, hash));
        }
        "same" => {
            let left = q.same_ref()?;
            q.expect("=")?;
            let right = q.same_ref()?;
            header.sames.push((left, right));
        }
        _ => header.notes.push(q.string()?),
    }
    Ok(())
}

/// One section, by its first word; `p` is over its line, after the word.
fn parse_section(p: &mut P<'_>, word: &str, body: &[Line]) -> Result<Section, Stop> {
    match word {
        "source" => Ok(Section::Source(Box::new(source(p, body)?))),
        "version" => Ok(Section::Version(Box::new(version(p, body)?))),
        "unit" => Ok(Section::Unit(Box::new(unit(p, body)?))),
        "kind" => Ok(Section::Kind(Box::new(kind(p, body)?))),
        "index" => Ok(Section::Index(Box::new(index(p, body)?))),
        "place" => Ok(Section::Place(Box::new(place(p, body)?))),
        "input" => Ok(Section::Input(Box::new(input(p, body)?))),
        "band" => Ok(Section::Band(Box::new(band(p, body)?))),
        "def" => Ok(Section::Def(Box::new(def(p, body)?))),
        "move" => Ok(Section::Move(Box::new(move_section(p, body)?))),
        "ask" => Ok(Section::Ask(Box::new(ask(p, body)?))),
        "axis" => Ok(Section::Axis(Box::new(axis(p, body)?))),
        "fill" => Ok(Section::Fill(Box::new(fill(p, body)?))),
        _ => Err(p.cannot(&format!("expected a section, got {word}"))),
    }
}

/// Refuses a body line indented other than one level under its section (refcore's `_flat`).
pub(super) fn flat(p: &P<'_>, body: &[Line]) -> Result<(), Stop> {
    match body.iter().find(|line| line.level != 1) {
        Some(line) => Err(p.cannot_at("indentation", line.number)),
        None => Ok(()),
    }
}

/// What a section that takes notes only says of another line.
#[derive(Debug, Clone, Copy)]
pub(super) enum Other {
    /// The same words whatever the line: `a place has no body but notes`.
    Says(&'static str),
    /// The line's word in the section: `x in an input`.
    In(&'static str),
}

/// The body's `note` lines, every one parsed first, then a refusal at the first line that is
/// no note (refcore's `_notes`, then its caller's check).
pub(super) fn only_notes(p: &P<'_>, body: &[Line], other: Other) -> Result<Vec<String>, Stop> {
    let mut notes = Vec::new();
    for line in body.iter().filter(|line| first(line) == "note") {
        let mut q = p.under(line);
        notes.push(q.string()?);
        q.end()?;
    }
    let Some(line) = body.iter().find(|line| first(line) != "note") else {
        return Ok(notes);
    };
    let what = match other {
        Other::Says(text) => text.to_owned(),
        Other::In(section) => format!("{} in {section}", first(line)),
    };
    Err(p.cannot_at(&what, line.number))
}

/// Refuses any body.
pub(super) fn no_body(p: &P<'_>, body: &[Line], what: &str) -> Result<(), Stop> {
    match body.first() {
        Some(line) => Err(p.cannot_at(what, line.number)),
        None => Ok(()),
    }
}

/// The order of a version's lines; a `pad` or `anchor` is a `gap`, anything else an `item`.
const VERSION_ORDER: [&str; 10] = [
    "cite", "governs", "known", "enters", "assumed", "moves", "covers", "gap", "note", "item",
];

/// The version lines that stand at most once.
const VERSION_ONCE: [&str; 6] = ["cite", "governs", "known", "enters", "assumed", "covers"];

/// A version line's tag in `VERSION_ORDER`.
fn version_tag(word: &str) -> &str {
    match word {
        "cite" | "governs" | "known" | "enters" | "assumed" | "moves" | "covers" | "note" => word,
        "pad" | "anchor" => "gap",
        _ => "item",
    }
}

/// What a version's lines say, gathered in order.
struct VersionLines {
    /// `cite`.
    cite: Option<String>,
    /// `governs`.
    governs: Option<Governs>,
    /// `known`.
    known: Option<Known>,
    /// `enters`.
    enters: Option<Day>,
    /// `assumed`.
    assumed: Option<String>,
    /// `moves`.
    moves: Vec<(Expr, LSpan)>,
    /// `covers`.
    covers: Vec<(Expr, Option<LSpan>)>,
    /// `pad` and `anchor`.
    gaps: Vec<Gap>,
    /// `note`.
    notes: Vec<String>,
    /// The items.
    items: Vec<Item>,
}

/// `version id` and its lines, in order: cite, governs, known, enters, assumed, moves, covers,
/// gaps, notes, items (L §2.3 `version`).
fn version(p: &mut P<'_>, body: &[Line]) -> Result<Version, Stop> {
    let id = p.ident("an id")?;
    p.end()?;
    flat(p, body)?;
    let mut lines = VersionLines {
        cite: None,
        governs: None,
        known: None,
        enters: None,
        assumed: None,
        moves: Vec::new(),
        covers: Vec::new(),
        gaps: Vec::new(),
        notes: Vec::new(),
        items: Vec::new(),
    };
    let mut pos: Option<usize> = None;
    for line in body {
        let word = first(line);
        let tag = version_tag(word);
        let k = VERSION_ORDER
            .iter()
            .position(|w| *w == tag)
            .unwrap_or(VERSION_ORDER.len());
        let once = VERSION_ONCE.contains(&tag);
        if pos.is_some_and(|at| k < at || (k == at && once)) {
            let what = format!("version line {word} out of order or repeated");
            return Err(p.cannot_at(&what, line.number));
        }
        pos = Some(k);
        version_line(&mut p.under(line), tag, &mut lines)?;
    }
    let Some(cite) = lines.cite else {
        return Err(p.cannot(&format!("version {id} has no cite")));
    };
    let Some(known) = lines.known else {
        return Err(p.refuse(Load::Known, format!("version {id} has no known date")));
    };
    Ok(Version {
        id,
        cite,
        governs: lines.governs,
        known,
        enters: lines.enters,
        assumed: lines.assumed,
        moves: lines.moves,
        covers: lines.covers,
        gaps: lines.gaps,
        notes: lines.notes,
        items: lines.items,
        line: p.line,
    })
}

/// One version line, by its tag; `q` is over it, after its first word.
fn version_line(q: &mut P<'_>, tag: &str, lines: &mut VersionLines) -> Result<(), Stop> {
    match tag {
        "cite" => lines.cite = Some(q.ident("an id")?),
        "governs" => lines.governs = Some(governs(q)?),
        "known" => lines.known = Some(q.known()?),
        "enters" => lines.enters = Some(q.day()?),
        "assumed" => lines.assumed = Some(q.string()?),
        "moves" => {
            let moved = q.reference()?;
            q.expect("to")?;
            lines.moves.push((moved, q.lspan()?));
        }
        "covers" => lines.covers = covers(q)?,
        "gap" => lines.gaps.push(gap(q)?),
        "note" => lines.notes.push(q.string()?),
        _ => {
            q.i = 0;
            let form = item(q)?;
            lines.items.push(Item { form, line: q.line });
            return Ok(());
        }
    }
    q.end()
}

/// `pad x from y` or `anchor x`, after its word.
fn gap(q: &mut P<'_>) -> Result<Gap, Stop> {
    if q.toks.first().is_some_and(|tok| tok.text == "pad") {
        let place = q.reference()?;
        q.expect("from")?;
        return Ok(Gap::Pad(place, q.reference()?));
    }
    Ok(Gap::Anchor(q.reference()?))
}

/// `governs always` or `governs span`, after the word.
fn governs(q: &mut P<'_>) -> Result<Governs, Stop> {
    if q.accept("always")? {
        return Ok(Governs::Always);
    }
    Ok(Governs::Span(q.lspan()?))
}

/// `covers x [span] {, x [span]}`.
fn covers(q: &mut P<'_>) -> Result<Vec<(Expr, Option<LSpan>)>, Stop> {
    let mut items = Vec::new();
    loop {
        let place = q.reference()?;
        let span = if !q.done() && !q.at(",", 0) {
            Some(q.lspan()?)
        } else {
            None
        };
        items.push((place, span));
        if !q.accept(",")? {
            return Ok(items);
        }
    }
}

/// Whether an item replaces one before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Replace {
    /// A new item.
    No,
    /// `replace …`.
    Yes,
}

/// A version's item (L §2.3 `vitem`).
fn item(q: &mut P<'_>) -> Result<ItemForm, Stop> {
    let replace = if q.accept("replace")? {
        Replace::Yes
    } else {
        Replace::No
    };
    if replace == Replace::Yes && q.accept("rule")? {
        return replace_rule(q);
    }
    let given = q.accept("given")?;
    if given && replace == Replace::Yes {
        return Err(q.err("a row"));
    }
    if q.accept("same")? {
        if replace == Replace::Yes || given {
            return Err(q.err("a row"));
        }
        let left = q.same_ref()?;
        q.expect("=")?;
        let right = q.same_ref()?;
        q.end()?;
        return Ok(ItemForm::Same(Box::new((left, right))));
    }
    let (qual, target) = q.qid()?;
    if q.at("+", 0) {
        if given {
            return Err(q.err("a row"));
        }
        return enter(q, qual, target, replace);
    }
    let mut first_key = None;
    let keys = if q.accept("[")? {
        first_key = Some(q.i);
        let mut keys = vec![q.row_key()?];
        while q.accept(",")? {
            keys.push(q.row_key()?);
        }
        q.expect("]")?;
        Some(keys)
    } else {
        None
    };
    let of = if q.at("of", 0) && !q.at(":", 1) {
        q.skip(1)?;
        let of = q.of_row()?;
        if !q.at("=", 0) {
            return Err(q.err("\"=\""));
        }
        Some(of)
    } else {
        None
    };
    if q.accept(":")? {
        if given {
            return Err(q.err("a row"));
        }
        let keys = keys.zip(first_key);
        return band_row(q, (qual, target), keys, replace);
    }
    q.expect("=")?;
    let (value, assumed) = value(q)?;
    let meta = row_meta(q)?;
    q.end()?;
    Ok(ItemForm::Row(Box::new(Row {
        qual,
        target,
        keys,
        of,
        value,
        assumed,
        governs: meta.governs,
        known: meta.known,
        instant: meta.instant,
        locator: meta.locator,
        given,
        replace: replace == Replace::Yes,
    })))
}

/// `replace rule [qual.]def.label rulebody`, after `replace rule`.
fn replace_rule(q: &mut P<'_>) -> Result<ItemForm, Stop> {
    let first_name = q.ident("a def")?;
    q.expect(".")?;
    let label = q.label()?;
    let (qual, def, label) = if q.at(".", 0) && !q.spaced(0) {
        q.skip(1)?;
        (Some(first_name), label, q.label()?)
    } else {
        (None, first_name, label)
    };
    Ok(ItemForm::Rule {
        qual,
        def,
        rule: Box::new(rule_body(q, label)?),
    })
}

/// `qid + [keys] [unit] [priced p] [lives span]`.
fn enter(
    q: &mut P<'_>,
    qual: Option<String>,
    target: String,
    replace: Replace,
) -> Result<ItemForm, Stop> {
    q.skip(1)?;
    q.expect("[")?;
    let keys = lit_list(q)?;
    let unit = place_unit(q)?;
    let priced = if q.accept("priced")? {
        Some(q.price()?)
    } else {
        None
    };
    let lives = if q.accept("lives")? {
        Some(q.life_span()?)
    } else {
        None
    };
    q.end()?;
    Ok(ItemForm::Enter(Box::new(Enter {
        qual,
        target,
        keys,
        unit,
        priced,
        lives,
        replace: replace == Replace::Yes,
    })))
}

/// `qid ["[" key "]"] ":" interval "=" (expr | hole) [locator]`; `keys` are the row keys read
/// before the `:`, with the position of the first.
fn band_row(
    q: &mut P<'_>,
    (qual, target): (Option<String>, String),
    keys: Option<(Vec<RowKey>, usize)>,
    replace: Replace,
) -> Result<ItemForm, Stop> {
    let key = match keys {
        None => None,
        Some((keys, first)) => {
            if keys.len() != 1 {
                return Err(q.err("one key of a keyed band"));
            }
            Some(band_key(q, first)?)
        }
    };
    let interval = interval(q)?;
    q.expect("=")?;
    let value = if q.at("?", 0) && q.at_str(1) {
        q.skip(1)?;
        RuleValue::Hole(q.string()?)
    } else {
        RuleValue::Expr(q.expr(Ends::Free)?)
    };
    let locator = if q.at_str(0) { Some(q.string()?) } else { None };
    q.end()?;
    Ok(ItemForm::Band(Box::new(BandRow {
        qual,
        target,
        key,
        interval,
        value,
        locator,
        replace: replace == Replace::Yes,
    })))
}

/// A band row's key read again from `first` as the grammar reads it, one literal with no
/// sign and then `]`, so `*`, a range, a place or a sign is refused where the grammar stops
/// (refcore's `vitem`, issue #88). L §2.3 `bandrow`.
fn band_key(q: &mut P<'_>, first: usize) -> Result<Lit, Stop> {
    let after = q.i;
    q.i = first;
    let key = q.lit_unsigned()?;
    if !q.at("]", 0) {
        return Err(q.err("\"]\""));
    }
    q.i = after;
    Ok(key)
}

/// A row's value and its `assumed` reason (L §2.3 `value`).
fn value(q: &mut P<'_>) -> Result<(Value, Option<String>), Stop> {
    if q.at("?", 0) && q.at_str(1) {
        q.skip(1)?;
        return Ok((Value::Hole(q.string()?), None));
    }
    if q.accept("never")? {
        return Ok((Value::Never, None));
    }
    let value = if q.accept("none")? {
        Value::None
    } else if q.at_str(0) {
        Value::Str(q.string()?)
    } else if q.accept("(")? {
        let fields = q.fields(Rec::Row)?;
        q.expect(")")?;
        Value::Record(fields)
    } else if q.accept("[")? {
        if q.accept("]")? {
            Value::List(Vec::new())
        } else {
            Value::List(lit_list(q)?)
        }
    } else if q.at("calendar", 0) && q.at("(", 1) {
        Value::Calendar(q.calendar()?)
    } else if (q.at("window", 0) || q.at("one_of", 0)) && q.at("(", 1) {
        Value::Days(q.day_set()?)
    } else {
        plain_value(q)?
    };
    let assumed = if q.accept("assumed")? {
        Some(q.string()?)
    } else {
        None
    };
    Ok((value, assumed))
}

/// The operators that make a row's value an expression (a scenario's row).
const VALUE_OPERATORS: [&str; 7] = [".", "*", "/", "+", "-", "(", "["];

/// A distribution with its `to u`, a literal row value, or an expression where an operator
/// follows (refcore's `value`).
fn plain_value(q: &mut P<'_>) -> Result<Value, Stop> {
    let distribution = q
        .peek(0)
        .filter(|tok| tok.kind == Kind::Id)
        .and_then(|tok| dist_kind(&tok.text));
    if let Some(func) = distribution
        && q.at("(", 1)
    {
        let dist = q.dist(func)?;
        if !q.accept("to")? {
            return Ok(Value::Dist(dist));
        }
        let step = q.step_unit()?;
        return Ok(Value::Dist(Expr::To(Box::new(dist), Box::new(step))));
    }
    let save = q.i;
    if !(q.at_id(0) && q.at(".", 1)) {
        let lit = q.lit()?;
        let operator_next = q
            .peek(0)
            .is_some_and(|tok| VALUE_OPERATORS.contains(&tok.text.as_str()));
        if !operator_next {
            return Ok(Value::Lit(lit));
        }
    }
    q.i = save;
    Ok(Value::Expr(q.expr(Ends::Free)?))
}

/// A row's meta (L §2.3 `rowmeta`).
struct Meta {
    /// `governs`.
    governs: Option<Governs>,
    /// `known`.
    known: Option<Known>,
    /// `at d [start|end]` or `after x`.
    instant: Option<Instant>,
    /// The locator string.
    locator: Option<String>,
}

/// A row's meta: `governs`, `known`, its instant, its locator.
fn row_meta(q: &mut P<'_>) -> Result<Meta, Stop> {
    let governs_clause = if q.accept("governs")? {
        Some(governs(q)?)
    } else {
        None
    };
    let known = if q.accept("known")? {
        Some(q.known()?)
    } else {
        None
    };
    let instant = if q.accept("at")? {
        let day = q.day()?;
        let edge = if q.accept("start")? {
            Some(Edge::Start)
        } else if q.accept("end")? {
            Some(Edge::End)
        } else {
            None
        };
        Some(Instant::At(day, edge))
    } else if q.accept("after")? {
        Some(Instant::After(q.reference()?))
    } else {
        None
    };
    let locator = if q.at_str(0) { Some(q.string()?) } else { None };
    Ok(Meta {
        governs: governs_clause,
        known,
        instant,
        locator,
    })
}

/// Whether an interval's end is closed: written `shut`, it is; written `open`, it isn't.
fn closed(q: &mut P<'_>, shut: &str, open: &str, what: &str) -> Result<bool, Stop> {
    if q.accept(shut)? {
        return Ok(true);
    }
    if q.accept(open)? {
        return Ok(false);
    }
    Err(q.err(what))
}

/// A band's interval: `[` or `(`, a low end or `-inf`, a high end or `inf`, `]` or `)`.
fn interval(q: &mut P<'_>) -> Result<Interval, Stop> {
    let lo_closed = closed(q, "[", "(", "an interval")?;
    let lo = if q.at("-", 0) && q.at("inf", 1) {
        q.skip(2)?;
        None
    } else {
        Some(q.lit()?)
    };
    q.expect(",")?;
    let hi = if q.accept("inf")? {
        None
    } else {
        Some(q.lit()?)
    };
    let hi_closed = closed(q, "]", ")", "\"]\" or \")\"")?;
    Ok(Interval {
        lo,
        hi,
        lo_closed,
        hi_closed,
    })
}
