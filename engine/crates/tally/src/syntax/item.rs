//! The file grammar: the header, the sections, a version's items and rows, and the units prescan.
//! Spec: L §2.3 (`file` to `ask`, `vitem`, `row`, `value`, `rowmeta`, `interval`, `rulebody`),
//! L §5.1 (the header's checks), L §8.3 (which refusal prints: split every file, then parse).
//! Twin: engine/tools/refcore/parser.py (`parse_header`, `parse_section`, `sec_*`, `vitem`,
//! `value`, `rowmeta`, `interval`, `rulebody`, `parse_file`, `prescan_units`).
//! Invariants: every file is split into lines before any is parsed; each section reads only
//! the lines indented under it; the error texts are refcore's word for word.

use super::expr::{Ends, Rec};
use super::lex::{Kind, Line, split_lines};
use super::parse::{P, Stop};
use super::tree::{
    Ask, AskBody, Band, BandRow, Bind, Certainty, Def, Edge, Enter, Expr, File, Gap, Governs,
    Header, HeaderKind, Index, IndexForm, IndexFrom, Input, InputForm, Instant, Interval, Item,
    ItemForm, KindDecl, KindSpec, Known, LSpan, LawDate, Lit, Move, Order, Place, Residuals, Row,
    RowKey, Rule, RuleValue, Section, Sign, Source, TypedInput, UnitDecl, UnitForm, Value, Version,
};
use crate::api::Source as Text;
use crate::base::{Code, Load, Place as At, Refusal};
use crate::num::Num;
use crate::time::Day;
use std::collections::BTreeSet;

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
    let units = prescan_units(&split);
    let mut files = Vec::with_capacity(sources.len());
    for (source, lines) in sources.iter().zip(&split) {
        files.push(parse_file(lines, &source.path, &units)?);
    }
    Ok(files)
}

/// The first token's text of a line, or nothing.
fn first(line: &Line) -> &str {
    line.toks.first().map_or("", |tok| tok.text.as_str())
}

/// The unit names a number may carry: declared `unit` ids, and the keys entered into kinds of
/// unit (refcore's `prescan_units`, L §3.2).
fn prescan_units(files: &[Vec<Line>]) -> BTreeSet<String> {
    let mut units = BTreeSet::new();
    let mut unit_kinds = BTreeSet::from(["currencies".to_owned()]);
    for line in files.iter().flatten() {
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
    for line in files.iter().flatten() {
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

/// A file: its header, then its sections, each at level 0.
fn parse_file(lines: &[Line], file: &str, units: &BTreeSet<String>) -> Result<File, Stop> {
    let Some((head, under_head)) = lines.split_first() else {
        return Err(Stop::Refused(Refusal::at(
            Code::Load(Load::Header),
            "file has no header".to_owned(),
            At::File(file.to_owned()),
        )));
    };
    let (header_body, mut rest) = children(under_head, 0);
    let header = parse_header(
        &P::new(&head.toks, file, head.number, units),
        head,
        header_body,
    )?;
    let mut sections = Vec::new();
    while let Some((line, under)) = rest.split_first() {
        let mut p = P::new(&line.toks, file, line.number, units);
        if line.level != 0 {
            return Err(p.cannot("indentation"));
        }
        let (body, next) = children(under, 0);
        p.i = 1;
        sections.push(parse_section(&mut p, first(line), body)?);
        rest = next;
    }
    Ok(File {
        path: file.to_owned(),
        header,
        sections,
    })
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
            let left = q.reference()?;
            q.expect("=")?;
            let right = q.reference()?;
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
        "axis" => Err(Stop::NotBuilt("axes")),
        "fill" => Err(Stop::NotBuilt("fills")),
        _ => Err(p.cannot(&format!("expected a section, got {word}"))),
    }
}

/// Refuses a body line indented other than one level under its section (refcore's `_flat`).
fn flat(p: &P<'_>, body: &[Line]) -> Result<(), Stop> {
    match body.iter().find(|line| line.level != 1) {
        Some(line) => Err(p.cannot_at("indentation", line.number)),
        None => Ok(()),
    }
}

/// What a section that takes notes only says of another line.
#[derive(Debug, Clone, Copy)]
enum Other {
    /// The same words whatever the line: `a place has no body but notes`.
    Says(&'static str),
    /// The line's word in the section: `x in an input`.
    In(&'static str),
}

/// The body's `note` lines, every one parsed first, then a refusal at the first line that is
/// no note (refcore's `_notes`, then its caller's check).
fn only_notes(p: &P<'_>, body: &[Line], other: Other) -> Result<Vec<String>, Stop> {
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
fn no_body(p: &P<'_>, body: &[Line], what: &str) -> Result<(), Stop> {
    match body.first() {
        Some(line) => Err(p.cannot_at(what, line.number)),
        None => Ok(()),
    }
}

/// `source id "title" [at "address"]`.
fn source(p: &mut P<'_>, body: &[Line]) -> Result<Source, Stop> {
    let id = p.ident("an id")?;
    let title = p.string()?;
    let address = if p.accept("at")? {
        Some(p.string()?)
    } else {
        None
    };
    p.end()?;
    no_body(p, body, "a source has no body")?;
    Ok(Source {
        id,
        title,
        at: address,
        line: p.line,
    })
}

/// `unit CUR: currency minor m`, or `unit id "label"`.
fn unit(p: &mut P<'_>, body: &[Line]) -> Result<UnitDecl, Stop> {
    let (id, form) = if p.kind(0) == Some(&Kind::Cur) {
        let id = p.cur()?;
        p.expect(":")?;
        p.expect("currency")?;
        p.expect("minor")?;
        (id, UnitForm::Currency(p.lit_unsigned()?))
    } else {
        let id = p.ident("an id")?;
        (id, UnitForm::Named(p.string()?))
    };
    p.end()?;
    no_body(p, body, "a unit has no body")?;
    Ok(UnitDecl {
        id,
        form,
        line: p.line,
    })
}

/// `certain` or `uncertain`, if next.
fn certainty(p: &mut P<'_>) -> Result<Option<Certainty>, Stop> {
    if p.accept("certain")? {
        return Ok(Some(Certainty::Certain));
    }
    if p.accept("uncertain")? {
        return Ok(Some(Certainty::Uncertain));
    }
    Ok(None)
}

/// `kind id [: kspec]` and its lines (L §2.3 `kind`).
fn kind(p: &mut P<'_>, body: &[Line]) -> Result<KindDecl, Stop> {
    let id = p.ident("an id")?;
    let spec = if p.accept(":")? {
        kind_spec(p, &id)?
    } else {
        KindSpec::Plain
    };
    p.end()?;
    flat(p, body)?;
    let mut decl = KindDecl {
        id,
        spec,
        legs: Vec::new(),
        attrs: Vec::new(),
        notes: Vec::new(),
        line: p.line,
    };
    for line in body {
        let mut q = p.under(line);
        match first(line) {
            "leg" => {
                let name = q.ident("an id")?;
                q.expect(":")?;
                let unit = if q.kind(0) == Some(&Kind::Cur) {
                    q.cur()?
                } else {
                    q.ident("an id")?
                };
                decl.legs.push((name, unit));
            }
            "attr" => {
                let name = q.ident("an id")?;
                q.expect(":")?;
                decl.attrs.push((name, q.ty()?));
            }
            "note" => decl.notes.push(q.string()?),
            "owned" | "batched" | "step" | "takes" => return Err(Stop::NotBuilt("parts clauses")),
            word => return Err(p.cannot_at(&format!("{word} in a kind"), line.number)),
        }
        q.end()?;
    }
    Ok(decl)
}

/// What follows `kind id:`.
fn kind_spec(p: &mut P<'_>, id: &str) -> Result<KindSpec, Stop> {
    if p.accept("place")? {
        return place_kind(p, id);
    }
    if p.accept("unit")? {
        return Ok(KindSpec::Unit);
    }
    if p.at("event", 0) && (p.at("certain", 1) || p.at("uncertain", 1) || p.peek(1).is_none()) {
        p.skip(1)?;
        return match certainty(p)? {
            Some(c) => Ok(KindSpec::Event(c)),
            None => Err(p.err("certain or uncertain")),
        };
    }
    let parent = p.ident("a kind")?;
    Ok(KindSpec::Sub {
        parent,
        certainty: certainty(p)?,
    })
}

/// `place unit sign [priced p] [outside]`, after `kind id: place`.
fn place_kind(p: &mut P<'_>, id: &str) -> Result<KindSpec, Stop> {
    let unit = if p.kind(0) == Some(&Kind::Cur) {
        p.cur()?
    } else {
        p.ident("a unit, any or one")?
    };
    let sign = if p.accept("signed")? {
        Sign::Signed
    } else if p.accept("min")? {
        zero(p)?;
        Sign::Min0
    } else if p.accept("max")? {
        zero(p)?;
        Sign::Max0
    } else {
        let message = format!("place kind {id} has no unit or sign rule");
        return Err(p.refuse(Load::PlaceKind, message));
    };
    let priced = if p.accept("priced")? {
        Some(p.price()?)
    } else {
        None
    };
    let outside = p.accept("outside")?;
    Ok(KindSpec::Place {
        unit,
        sign,
        priced,
        outside,
    })
}

/// The `0` of `min 0` or `max 0`.
fn zero(p: &mut P<'_>) -> Result<(), Stop> {
    match p.kind(0) {
        Some(Kind::Int(n)) if n.is_zero() => p.skip(1),
        Some(_) | None => Err(p.err("0")),
    }
}

/// `lit, … ]`, after its `[`.
fn lit_list(p: &mut P<'_>) -> Result<Vec<Lit>, Stop> {
    let mut keys = vec![p.lit()?];
    while p.accept(",")? {
        keys.push(p.lit()?);
    }
    p.expect("]")?;
    Ok(keys)
}

/// `index id (= [keys] | = [period] lo .. hi [by n] [from f] | : open)` and its lines.
fn index(p: &mut P<'_>, body: &[Line]) -> Result<Index, Stop> {
    let id = p.ident("an id")?;
    let form = if p.accept(":")? {
        p.expect("open")?;
        IndexForm::Open
    } else {
        p.expect("=")?;
        if p.accept("[")? {
            IndexForm::List(lit_list(p)?)
        } else {
            index_range(p)?
        }
    };
    p.end()?;
    flat(p, body)?;
    let mut decl = Index {
        id,
        form,
        v: None,
        assumed: None,
        notes: Vec::new(),
        line: p.line,
    };
    for line in body {
        let mut q = p.under(line);
        match first(line) {
            "v" => decl.v = Some(q.ident("an id")?),
            "assumed" => decl.assumed = Some(q.string()?),
            "note" => decl.notes.push(q.string()?),
            word => return Err(p.cannot_at(&format!("{word} in an index"), line.number)),
        }
        q.end()?;
    }
    Ok(decl)
}

/// The periods an index's range may name.
const INDEX_PERIODS: [&str; 4] = ["year", "month", "day", "int"];

/// `[period] lo .. hi [by n] [from f]`.
fn index_range(p: &mut P<'_>) -> Result<IndexForm, Stop> {
    let period = p
        .peek(0)
        .filter(|tok| tok.kind == Kind::Id && INDEX_PERIODS.contains(&tok.text.as_str()));
    if period.is_some() {
        p.skip(1)?;
    }
    let lo = p.lit()?;
    p.expect("..")?;
    let hi = p.lit()?;
    let by = if p.accept("by")? {
        Some(p.integer()?)
    } else {
        None
    };
    let from = if p.accept("from")? {
        Some(index_from(p)?)
    } else {
        None
    };
    Ok(IndexForm::Range {
        period: period.map(|tok| tok.text.clone()),
        lo,
        hi,
        by,
        from,
    })
}

/// An index's `from`: a month-day, or a day of the month.
fn index_from(p: &mut P<'_>) -> Result<IndexFrom, Stop> {
    let from = match p.kind(0) {
        Some(Kind::MonthDay(m, d)) => IndexFrom::MonthDay(*m, *d),
        Some(Kind::Int(n)) => IndexFrom::Day(n.clone()),
        Some(_) | None => return Err(p.err("a month-day or a day of the month")),
    };
    p.skip(1)?;
    Ok(from)
}

/// The keys of a declaration, if `[` is next.
fn decl_binds(p: &mut P<'_>) -> Result<Vec<Bind>, Stop> {
    if p.at("[", 0) {
        return p.bind_list();
    }
    Ok(Vec::new())
}

/// A unit after a place's kind: a currency, or an id other than `priced` and `lives`.
fn place_unit(p: &mut P<'_>) -> Result<Option<String>, Stop> {
    if p.kind(0) == Some(&Kind::Cur) {
        return Ok(Some(p.cur()?));
    }
    let named = p
        .peek(0)
        .is_some_and(|tok| tok.kind == Kind::Id && tok.text != "priced" && tok.text != "lives");
    if named {
        return Ok(Some(p.ident("an id")?));
    }
    Ok(None)
}

/// `place id [binds]: kind [unit] [priced p] lives span`, with notes only.
fn place(p: &mut P<'_>, body: &[Line]) -> Result<Place, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.expect(":")?;
    let kind = p.ident("a place kind")?;
    let unit = place_unit(p)?;
    let priced = if p.accept("priced")? {
        Some(p.price()?)
    } else {
        None
    };
    p.expect("lives")?;
    let lives = p.lspan()?;
    p.end()?;
    flat(p, body)?;
    let notes = only_notes(p, body, Other::Says("a place has no body but notes"))?;
    Ok(Place {
        id,
        binds,
        kind,
        unit,
        priced,
        lives,
        notes,
        line: p.line,
    })
}

/// `input id [binds]: type [reading] [prices y [by ix]] [question "…"]`, or `: observed`.
fn input(p: &mut P<'_>, body: &[Line]) -> Result<Input, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.expect(":")?;
    let form = if p.at("observed", 0) && p.peek(1).is_none() {
        p.skip(1)?;
        if !binds.is_empty() {
            return Err(p.err("no keys on an observed input"));
        }
        InputForm::Observed
    } else {
        typed_input(p)?
    };
    p.end()?;
    let notes = only_notes(p, body, Other::In("an input"))?;
    Ok(Input {
        id,
        binds,
        form,
        notes,
        line: p.line,
    })
}

/// An input's `type [reading] [prices y [by ix]] [question "…"]`.
fn typed_input(p: &mut P<'_>) -> Result<InputForm, Stop> {
    let ty = p.ty()?;
    let reading = p.accept("reading")?;
    let (prices, prices_by) = if p.accept("prices")? {
        let year = p.lit()?;
        let by = if p.accept("by")? {
            Some(p.reference()?)
        } else {
            None
        };
        (Some(year), by)
    } else {
        (None, None)
    };
    let question = if p.accept("question")? {
        Some(p.string()?)
    } else {
        None
    };
    Ok(InputForm::Typed(Box::new(TypedInput {
        ty,
        reading,
        prices,
        prices_by,
        question,
    })))
}

/// `band id [binds](x: type): type`, with notes only.
fn band(p: &mut P<'_>, body: &[Line]) -> Result<Band, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.expect("(")?;
    let arg = p.ident("an id")?;
    p.expect(":")?;
    let arg_type = p.ty()?;
    p.expect(")")?;
    p.expect(":")?;
    let ty = p.ty()?;
    p.end()?;
    let notes = only_notes(p, body, Other::In("a band"))?;
    Ok(Band {
        id,
        binds,
        arg,
        arg_type,
        ty,
        notes,
        line: p.line,
    })
}

/// `at reader` or `at e`, if `at` is next.
fn law_date(p: &mut P<'_>) -> Result<Option<LawDate>, Stop> {
    if !p.accept("at")? {
        return Ok(None);
    }
    if p.accept("reader")? {
        return Ok(Some(LawDate::Reader));
    }
    Ok(Some(LawDate::Expr(p.expr(Ends::Free)?)))
}

/// A rule's value: `?"…"` or an expression; `solve` is built later.
fn rule_value(p: &mut P<'_>) -> Result<RuleValue, Stop> {
    if p.at("?", 0) && p.at_str(1) {
        p.skip(1)?;
        return Ok(RuleValue::Hole(p.string()?));
    }
    if p.at("solve", 0) {
        return Err(Stop::NotBuilt("solve"));
    }
    Ok(RuleValue::Expr(p.expr(Ends::Free)?))
}

/// `def id [binds]: type = value [at d]` (short), or `def id [binds]: type [at d]` and rules.
fn def(p: &mut P<'_>, body: &[Line]) -> Result<Def, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.expect(":")?;
    let ty = p.ty()?;
    if p.accept("=")? {
        let value = rule_value(p)?;
        let law = law_date(p)?;
        p.end()?;
        let mut notes = Vec::new();
        if !body.is_empty() {
            flat(p, body)?;
            notes = only_notes(p, body, Other::Says("a short def has no body but notes"))?;
        }
        let base = Rule {
            label: "base".to_owned(),
            beats: Vec::new(),
            v: None,
            governs: None,
            value,
            when: None,
            line: p.line,
        };
        return Ok(Def {
            id,
            binds,
            ty,
            at: law,
            rules: vec![base],
            notes,
            short: true,
            line: p.line,
        });
    }
    let law = law_date(p)?;
    p.end()?;
    flat(p, body)?;
    let mut decl = Def {
        id,
        binds,
        ty,
        at: law,
        rules: Vec::new(),
        notes: Vec::new(),
        short: false,
        line: p.line,
    };
    def_lines(p, &mut decl, body)?;
    if decl.rules.is_empty() {
        return Err(p.cannot(&format!("def {} has no rule", decl.id)));
    }
    Ok(decl)
}

/// A long def's lines: its notes, then its rules.
fn def_lines(p: &P<'_>, decl: &mut Def, body: &[Line]) -> Result<(), Stop> {
    for line in body {
        let mut q = p.under(line);
        match first(line) {
            "note" if decl.rules.is_empty() => {
                decl.notes.push(q.string()?);
                q.end()?;
            }
            "note" => return Err(p.cannot_at("notes come before rules", line.number)),
            "rule" => {
                let label = q.label()?;
                decl.rules.push(rule_body(&mut q, label)?);
            }
            word => return Err(p.cannot_at(&format!("{word} in a def"), line.number)),
        }
    }
    Ok(())
}

/// `rulebody = [beats l, …] [v V] [governs s] = value [when c]`, to the line's end.
fn rule_body(q: &mut P<'_>, label: String) -> Result<Rule, Stop> {
    let mut beats = Vec::new();
    if q.accept("beats")? {
        beats.push(q.label()?);
        while q.accept(",")? {
            beats.push(q.label()?);
        }
    }
    let v = if q.at("v", 0) && q.at_id(1) {
        q.skip(1)?;
        Some(q.ident("an id")?)
    } else {
        None
    };
    let governs = if q.accept("governs")? {
        Some(q.lspan()?)
    } else {
        None
    };
    q.expect("=")?;
    let value = rule_value(q)?;
    let when = if q.accept("when")? {
        Some(q.expr(Ends::Free)?)
    } else {
        None
    };
    q.end()?;
    Ok(Rule {
        label,
        beats,
        v,
        governs,
        value,
        when,
        line: q.line,
    })
}

/// `move id [binds]: kind` and its lines (L §2.3 `move`).
fn move_section(p: &mut P<'_>, body: &[Line]) -> Result<Move, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.expect(":")?;
    let mut kind = p.ident("an event kind")?;
    if p.at(".", 0) && !p.spaced(0) {
        p.skip(1)?;
        kind = format!("{kind}.{}", p.ident("an id")?);
    }
    p.end()?;
    flat(p, body)?;
    let mut decl = Move {
        id,
        binds,
        kind,
        on: Expr::None,
        at: None,
        for_period: None,
        when: None,
        reads: Vec::new(),
        order: Vec::new(),
        ties: None,
        attrs: Vec::new(),
        legs: Vec::new(),
        fulfils: None,
        notes: Vec::new(),
        line: p.line,
    };
    let mut on = None;
    for line in body {
        let mut q = p.under(line);
        match first(line) {
            "on" => on = Some(q.expr(Ends::Free)?),
            "ties" => {
                q.i = 0;
                decl.ties = Some(q.ties()?);
            }
            "leg" => {
                q.i = 0;
                decl.legs.push(q.leg()?);
            }
            "note" => decl.notes.push(q.string()?),
            word => move_line(&mut q, word, &mut decl)?,
        }
        q.end()?;
    }
    let Some(day) = on else {
        return Err(p.cannot(&format!("move {} has no on", decl.id)));
    };
    decl.on = day;
    Ok(decl)
}

/// One of a move's other lines, after its first word.
fn move_line(q: &mut P<'_>, word: &str, decl: &mut Move) -> Result<(), Stop> {
    match word {
        "at" => decl.at = Some(q.expr(Ends::Free)?),
        "for" => decl.for_period = Some(q.expr(Ends::Free)?),
        "when" => decl.when = Some(q.expr(Ends::Free)?),
        "reads" => {
            decl.reads = vec![q.reference()?];
            while q.accept(",")? {
                decl.reads.push(q.reference()?);
            }
        }
        "before" => decl.order.push((Order::Before, q.ident("an id")?)),
        "after" => decl.order.push((Order::After, q.ident("an id")?)),
        "attr" => {
            let name = q.ident("an id")?;
            q.expect("=")?;
            decl.attrs.push((name, q.expr(Ends::Free)?));
        }
        "fulfils" => decl.fulfils = Some(q.reference()?),
        _ => return Err(q.cannot(&format!("{word} in a move"))),
    }
    Ok(())
}

/// An ask's body expression; `in ID` after it names the world unless a comma or `on`
/// follows (parse rule 4).
fn ask_expr(p: &mut P<'_>) -> Result<Expr, Stop> {
    let save = p.i;
    let expr = p.expr(Ends::AtIn)?;
    if p.at("in", 0) && p.at_id(1) && (p.at(",", 2) || p.at("on", 2)) {
        p.i = save;
        return p.expr(Ends::Free);
    }
    Ok(expr)
}

/// The ask forms the engine builds later, by their first word.
const LATER_ASKS: [&str; 5] = ["explain", "inputs-of", "readers-of", "check", "solve"];

/// The random ask's clauses, built later.
const RANDOM_CLAUSES: [&str; 4] = ["random", "paths", "path", "orient"];

/// `ask id: body [for binds] [in world] [law e] [as known e] [windows each] [with uncertain]
/// [partial]`, one line (L §2.3 `ask`); the other forms and clauses are built later.
fn ask(p: &mut P<'_>, body: &[Line]) -> Result<Ask, Stop> {
    let id = p.ident("an id")?;
    p.expect(":")?;
    if let Some(word) = LATER_ASKS.into_iter().find(|word| p.at(word, 0)) {
        return Err(Stop::NotBuilt(word));
    }
    if p.at("diff", 0) && p.at("(", 1) {
        return Err(Stop::NotBuilt("diff"));
    }
    let ask_body = if p.accept("residuals")? {
        let place = p.reference()?;
        let span = p.span()?;
        let by = if p.accept("by")? {
            Some(p.ident("an id")?)
        } else {
            None
        };
        AskBody::Residuals(Box::new(Residuals { place, span, by }))
    } else {
        let mut exprs = vec![ask_expr(p)?];
        while p.accept(",")? {
            exprs.push(ask_expr(p)?);
        }
        AskBody::Exprs(exprs)
    };
    let binds = if p.accept("for")? {
        p.ask_for = true;
        let binds = p.binds();
        p.ask_for = false;
        binds?
    } else {
        Vec::new()
    };
    let world = if p.at("in", 0) && p.at_id(1) {
        p.skip(1)?;
        Some(p.ident("an id")?)
    } else {
        None
    };
    if p.at("with", 0) && !p.at("uncertain", 1) {
        return Err(Stop::NotBuilt("pins"));
    }
    let mut decl = Ask {
        id,
        body: ask_body,
        binds,
        world,
        law: None,
        known: None,
        windows_each: false,
        uncertain: false,
        partial: false,
        line: p.line,
    };
    ask_clauses(p, &mut decl)?;
    no_body(p, body, "an ask is one line")?;
    Ok(decl)
}

/// An ask's clauses after its world: `law`, `as known`, `windows each`, `with uncertain`,
/// `partial`, to the line's end.
fn ask_clauses(p: &mut P<'_>, decl: &mut Ask) -> Result<(), Stop> {
    if p.accept("law")? {
        decl.law = Some(p.expr(Ends::Free)?);
    }
    if p.at("as", 0) && p.at("known", 1) {
        p.skip(2)?;
        decl.known = Some(p.expr(Ends::Free)?);
    }
    if p.accept("windows")? {
        p.each()?;
        decl.windows_each = true;
    }
    decl.uncertain = p.with_uncertain()?;
    if RANDOM_CLAUSES.iter().any(|word| p.at(word, 0)) {
        return Err(Stop::NotBuilt("random asks"));
    }
    decl.partial = p.accept("partial")?;
    p.end()
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
    /// `covers`.
    covers: Vec<(Expr, Option<LSpan>)>,
    /// `pad` and `anchor`.
    gaps: Vec<Gap>,
    /// `note`.
    notes: Vec<String>,
    /// The items.
    items: Vec<Item>,
}

/// `version id` and its lines, in order: cite, governs, known, enters, assumed, covers, gaps,
/// notes, items (L §2.3 `version`).
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
        "moves" => return Err(Stop::NotBuilt("version moves")),
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
        let left = q.reference()?;
        q.expect("=")?;
        let right = q.reference()?;
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
    let keys = if q.accept("[")? {
        let mut keys = vec![q.row_key()?];
        while q.accept(",")? {
            keys.push(q.row_key()?);
        }
        q.expect("]")?;
        Some(keys)
    } else {
        None
    };
    if q.at("of", 0) && !q.at(":", 1) {
        return Err(Stop::NotBuilt("parts in a reading's row"));
    }
    if q.accept(":")? {
        if given {
            return Err(q.err("a row"));
        }
        return band_row(q, qual, target, keys, replace);
    }
    q.expect("=")?;
    let (value, assumed) = value(q)?;
    let meta = row_meta(q)?;
    q.end()?;
    Ok(ItemForm::Row(Box::new(Row {
        qual,
        target,
        keys,
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

/// `qid ["[" key "]"] ":" interval "=" (expr | hole) [locator]`.
fn band_row(
    q: &mut P<'_>,
    qual: Option<String>,
    target: String,
    keys: Option<Vec<RowKey>>,
    replace: Replace,
) -> Result<ItemForm, Stop> {
    let key = match keys {
        None => None,
        Some(keys) => {
            let [key] =
                <[RowKey; 1]>::try_from(keys).map_err(|_| q.err("one key of a keyed band"))?;
            Some(key)
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

/// The distributions, built later.
const DISTRIBUTIONS: [&str; 9] = [
    "normal",
    "lognormal",
    "exponential",
    "gamma",
    "uniform",
    "poisson",
    "binomial",
    "negbin",
    "discrete",
];

/// The operators that make a row's value an expression (a scenario's row).
const VALUE_OPERATORS: [&str; 7] = [".", "*", "/", "+", "-", "(", "["];

/// A literal row value, or an expression where an operator follows.
fn plain_value(q: &mut P<'_>) -> Result<Value, Stop> {
    let distribution = q
        .peek(0)
        .is_some_and(|tok| tok.kind == Kind::Id && DISTRIBUTIONS.contains(&tok.text.as_str()));
    if distribution && q.at("(", 1) {
        return Err(Stop::NotBuilt("distributions"));
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
