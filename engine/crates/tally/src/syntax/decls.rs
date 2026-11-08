//! The declaring sections: `source`, `unit`, `kind` with its lines, `axis`, `fill`, `index`, `place`, `input`, `band`, `def` with its rules, and `move`.
//! Spec: L §2.3 (`source`, `unitdecl`, `kind`, `kspec`, `sign`, `kline`, `axis`, `index`,
//! `place`, `input`, `band`, `def`, `lawdate`, `rule`, `rulebody`, `fill`, `move`), L §2.2
//! (`prorata` the clause word of `takes`, not the built-in).
//! Twin: engine/tools/refcore/parser.py (`sec_source`, `sec_unit`, `sec_kind`, `sec_axis`,
//! `sec_fill`, `sec_index`, `sec_place`, `sec_input`, `sec_band`, `_lawdate`, `sec_def`,
//! `rulebody`, `sec_move`).
//! Invariants: each section reads only the lines indented under it; a fill's line written
//! twice keeps the last; the error texts are refcore's word for word.

use super::expr::Ends;
use super::item::{Other, first, flat, no_body, only_notes};
use super::lex::{Kind, Line, Role};
use super::parse::{P, Stop};
use super::tree::{
    Axis, Band, Bind, Certainty, Def, Expr, Fill, FillLines, Index, IndexForm, IndexFrom, Input,
    InputForm, KindDecl, KindSpec, LawDate, Lit, Move, MoveLines, Order, PartsClause, Place, Rule,
    RuleValue, Sign, Source, Takes, TypedInput, UnitDecl, UnitForm,
};
use crate::base::Load;

/// `source id "title" [at "address"]`.
pub(super) fn source(p: &mut P<'_>, body: &[Line]) -> Result<Source, Stop> {
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
pub(super) fn unit(p: &mut P<'_>, body: &[Line]) -> Result<UnitDecl, Stop> {
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
pub(super) fn kind(p: &mut P<'_>, body: &[Line]) -> Result<KindDecl, Stop> {
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
        parts: Vec::new(),
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
            "owned" if q.at("by", 0) => {
                q.skip(1)?;
                let rule = q.ident("an arrival rule")?;
                let ties = if q.at("ties", 0) {
                    Some(q.ties()?)
                } else {
                    None
                };
                decl.parts
                    .push((PartsClause::Owned { rule, ties }, line.number));
            }
            "batched" => decl.parts.push((PartsClause::Batched, line.number)),
            "step" => {
                let step = q.step_unit()?;
                decl.parts.push((PartsClause::Step(step), line.number));
            }
            "takes" => {
                let takes = takes(&mut q)?;
                decl.parts
                    .push((PartsClause::Takes(Box::new(takes)), line.number));
            }
            word => return Err(p.cannot_at(&format!("{word} in a kind"), line.number)),
        }
        q.end()?;
    }
    Ok(decl)
}

/// `takes K for W [v V] (order e, … [prorata] | prorata) [ties by …]`, after `takes`: the
/// `ties by` belongs to `prorata` (L §2.3 `kline`).
fn takes(q: &mut P<'_>) -> Result<Takes, Stop> {
    let part = q.ident("a part")?;
    q.expect("for")?;
    let outflow = q.ident("an outflow")?;
    let v = if q.accept("v")? {
        Some(q.ident("a version")?)
    } else {
        None
    };
    let mut order = Vec::new();
    if q.accept("order")? {
        order.push(q.expr(Ends::Free)?);
        while q.accept(",")? {
            order.push(q.expr(Ends::Free)?);
        }
    } else if !prorata_word(q) {
        return Err(q.err("order or prorata"));
    }
    let prorata = prorata_word(q);
    if prorata {
        q.skip(1)?;
    }
    let ties = if prorata && q.at("ties", 0) {
        Some(q.ties()?)
    } else {
        None
    };
    Ok(Takes {
        part,
        outflow,
        v,
        order,
        prorata,
        ties,
    })
}

/// Whether `prorata` here is the clause word of `takes`, not the built-in (L §2.2).
fn prorata_word(q: &P<'_>) -> bool {
    q.at("prorata", 0) && !q.at("(", 1)
}

/// `axis id` and its options, each `key "label"`; two at least (L §2.3 `axis`).
pub(super) fn axis(p: &mut P<'_>, body: &[Line]) -> Result<Axis, Stop> {
    let id = p.ident("an id")?;
    p.end()?;
    flat(p, body)?;
    let mut options = Vec::new();
    for line in body {
        let mut q = p.under(line);
        q.i = 0;
        let tok = q.next()?;
        let key = match tok.kind {
            Kind::Id | Kind::Key | Kind::Int(_) => tok.text.clone(),
            Kind::Dec(_)
            | Kind::Str(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur
            | Kind::Op => {
                q.i = 0;
                return Err(q.err("an option key"));
            }
        };
        let label = q.string()?;
        q.end()?;
        options.push((key, label));
    }
    if options.len() < 2 {
        return Err(p.cannot(&format!("axis {id} needs two options")));
    }
    Ok(Axis {
        id,
        options,
        line: p.line,
    })
}

/// `fill id [binds]` and its lines: `v`, notes, `supply`, `demand`, `arc`, `order`; the last
/// four required (L §2.3 `fill`).
pub(super) fn fill(p: &mut P<'_>, body: &[Line]) -> Result<Fill, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.end()?;
    flat(p, body)?;
    let mut lines = FillParts::default();
    for line in body {
        let mut q = p.under(line);
        fill_line(&mut q, first(line), &mut lines)?;
        q.end()?;
    }
    let (Some(supply), Some(demand), Some(arc), Some(order)) =
        (lines.supply, lines.demand, lines.arc, lines.order)
    else {
        let what = format!("fill {id} needs supply, demand, arc and order");
        return Err(p.cannot(&what));
    };
    Ok(Fill {
        id,
        binds,
        v: lines.v,
        notes: lines.notes,
        supply: supply.0,
        supply_binds: supply.1,
        demand: demand.0,
        demand_binds: demand.1,
        arc,
        order: order.0,
        ties: order.1,
        line: p.line,
        lines: lines.at,
    })
}

/// What a fill's lines say, gathered in order; a line written twice keeps the last.
#[derive(Default)]
struct FillParts {
    /// `v`.
    v: Option<String>,
    /// Notes.
    notes: Vec<String>,
    /// `supply e [for binds]`.
    supply: Option<(Expr, Vec<Bind>)>,
    /// `demand e [for binds]`.
    demand: Option<(Expr, Vec<Bind>)>,
    /// `arc e`.
    arc: Option<Expr>,
    /// `order e, … [ties by …]`.
    order: Option<(Vec<Expr>, Option<Vec<Expr>>)>,
    /// The line of each.
    at: FillLines,
}

/// One of a fill's lines, after its first word.
fn fill_line(q: &mut P<'_>, word: &str, lines: &mut FillParts) -> Result<(), Stop> {
    match word {
        "v" => lines.v = Some(q.ident("an id")?),
        "note" => lines.notes.push(q.string()?),
        "supply" | "demand" => {
            let side = q.expr(Ends::Free)?;
            let binds = if q.accept("for")? {
                q.binds()?
            } else {
                Vec::new()
            };
            if word == "supply" {
                lines.supply = Some((side, binds));
                lines.at.supply = q.line;
            } else {
                lines.demand = Some((side, binds));
                lines.at.demand = q.line;
            }
        }
        "arc" => {
            lines.arc = Some(q.expr(Ends::Free)?);
            lines.at.arc = q.line;
        }
        "order" => {
            let mut keys = vec![q.expr(Ends::Free)?];
            while q.accept(",")? {
                keys.push(q.expr(Ends::Free)?);
            }
            let ties = if q.at("ties", 0) {
                Some(q.ties()?)
            } else {
                None
            };
            lines.order = Some((keys, ties));
            lines.at.order = q.line;
        }
        _ => return Err(q.cannot(&format!("{word} in a fill"))),
    }
    Ok(())
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
pub(super) fn lit_list(p: &mut P<'_>) -> Result<Vec<Lit>, Stop> {
    let mut keys = vec![p.lit()?];
    while p.accept(",")? {
        keys.push(p.lit()?);
    }
    p.expect("]")?;
    Ok(keys)
}

/// `index id (= [keys] | = [period] lo .. hi [by n] [from f] | : open)` and its lines.
pub(super) fn index(p: &mut P<'_>, body: &[Line]) -> Result<Index, Stop> {
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
pub(super) fn place_unit(p: &mut P<'_>) -> Result<Option<String>, Stop> {
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
pub(super) fn place(p: &mut P<'_>, body: &[Line]) -> Result<Place, Stop> {
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
pub(super) fn input(p: &mut P<'_>, body: &[Line]) -> Result<Input, Stop> {
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
pub(super) fn band(p: &mut P<'_>, body: &[Line]) -> Result<Band, Stop> {
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

/// A rule's value: `?"…"`, `solve …`, or an expression (L §2.3 `rulebody`).
fn rule_value(p: &mut P<'_>) -> Result<RuleValue, Stop> {
    if p.at("solve", 0) {
        return Ok(RuleValue::Solve(Box::new(p.solve()?)));
    }
    short_value(p)
}

/// A short def's value: `?"…"` or an expression; `solve` is a rule's only (L §2.3 `def`).
fn short_value(p: &mut P<'_>) -> Result<RuleValue, Stop> {
    if p.at("?", 0) && p.at_str(1) {
        p.skip(1)?;
        return Ok(RuleValue::Hole(p.string()?));
    }
    Ok(RuleValue::Expr(p.expr(Ends::Free)?))
}

/// `def id [binds]: type = value [at d]` (short), or `def id [binds]: type [at d]` and rules.
pub(super) fn def(p: &mut P<'_>, body: &[Line]) -> Result<Def, Stop> {
    let id = p.ident("an id")?;
    let binds = decl_binds(p)?;
    p.expect(":")?;
    let ty = p.ty()?;
    if p.accept("=")? {
        p.role(p.back(), Role::DefEq);
        let value = short_value(p)?;
        if p.at("at", 0) {
            p.role(Some(p.i), Role::DefAt);
        }
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
    if p.at("at", 0) {
        p.role(Some(p.i), Role::DefAt);
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
pub(super) fn rule_body(q: &mut P<'_>, label: String) -> Result<Rule, Stop> {
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
pub(super) fn move_section(p: &mut P<'_>, body: &[Line]) -> Result<Move, Stop> {
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
        lines: MoveLines::at_move(p.line),
        line: p.line,
    };
    let mut on = None;
    for line in body {
        let mut q = p.under(line);
        match first(line) {
            "on" => {
                on = Some(q.expr(Ends::Free)?);
                decl.lines.on = line.number;
            }
            "ties" => {
                q.i = 0;
                decl.ties = Some(q.ties()?);
                decl.lines.ties = line.number;
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

/// One of a move's other lines, after its first word, with the line it stands on.
fn move_line(q: &mut P<'_>, word: &str, decl: &mut Move) -> Result<(), Stop> {
    let line = q.line;
    match word {
        "at" => {
            decl.at = Some(q.expr(Ends::Free)?);
            decl.lines.at = line;
        }
        "for" => {
            decl.for_period = Some(q.expr(Ends::Free)?);
            decl.lines.for_period = line;
        }
        "when" => {
            decl.when = Some(q.expr(Ends::Free)?);
            decl.lines.when = line;
        }
        "reads" => {
            decl.reads = vec![q.reference()?];
            while q.accept(",")? {
                decl.reads.push(q.reference()?);
            }
            decl.lines.reads = line;
        }
        "before" => decl.order.push((Order::Before, q.ident("an id")?)),
        "after" => decl.order.push((Order::After, q.ident("an id")?)),
        "attr" => {
            let name = q.ident("an id")?;
            q.expect("=")?;
            decl.attrs.push((name, q.expr(Ends::Free)?));
            decl.lines.attrs.push(line);
        }
        "fulfils" => {
            decl.fulfils = Some(q.reference()?);
            decl.lines.fulfils = line;
        }
        _ => return Err(q.cannot(&format!("{word} in a move"))),
    }
    Ok(())
}
