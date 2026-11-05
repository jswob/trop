//! Load's law dates: which defs and bands are law-dated, and that whatever reads one states its law date, or takes its reader's and is read by one that does.
//! Spec: L §4.3 (a fact is law-dated when its value can change with the law date; an input
//! keyed by one period index, or a calendar read at each day asked, isn't; a def that
//! states `at e` dates its own reads; `at reader` takes its reader's; the `at` ends every
//! chain), §5.14 (a move's `at`), §5.18 (an ask's `law`), §8.3 (`LOAD-LAWDATE`).
//! Twin: engine/tools/refcore/check.py (`check_lawdates`: `ver_timeless`, `own_dated`,
//! `at_dated`, `needs_reader_law`, `reads_dated`, the fixed point, the defs' and the asks'
//! refusals); check3.py (`m_check_move_lawdates`); model.py (`row_timeless`,
//! `InputInfo.lawless`, `undated_read`).
//! Invariants: bands and defs only ever turn law-dated, to their least fixed point, each
//! pass over them in declaration order as refcore's; a read made only at a day the reader
//! states (a price, `last`, `since`) or of a calendar at each day asked dates nothing; an
//! ask's reads are taken in the order it first made them; every search is a loop over an
//! explicit stack (R-16).

use super::check::{Files, Walked, invariant};
use super::steps::refuse;
use crate::base::Load;
use crate::model::{
    DefInfo, Graph, InputInfo, MoveInfo, Node, NodeKind, Reality, Rel, Row, Span, TimePos,
};
use crate::syntax::{LawDate, PinForm, Stop};
use std::collections::{BTreeMap, BTreeSet};

/// Each def's reads inside its `at`, with their relations.
type AtReads = BTreeMap<String, BTreeMap<Node, Vec<Rel>>>;

/// Which bands and defs are law-dated (refcore's `banddated` and `lawdated`). L §4.3.
pub(super) struct LawDated {
    /// Each band's.
    bands: BTreeMap<String, bool>,
    /// Each def's.
    defs: BTreeMap<String, bool>,
}

impl LawDated {
    /// Whether band `id` is law-dated; a band load doesn't know is (refcore's
    /// `banddated.get(b, True)`).
    fn band(&self, id: &str) -> bool {
        !matches!(self.bands.get(id), Some(false))
    }

    /// Whether def `id` is law-dated.
    fn def(&self, id: &str) -> bool {
        matches!(self.defs.get(id), Some(true))
    }
}

/// Whether a read with `rels` reads at the law date at all: not only at a day the reader
/// states, nor a calendar at each day asked (refcore's `rels - {'pin', 'calendar',
/// 'explicit', 'lawpin'}`). L §4.3.
fn dating(rels: &[Rel]) -> bool {
    rels.iter()
        .any(|rel| !matches!(rel, Rel::Calendar | Rel::Explicit | Rel::Pin | Rel::LawPin))
}

/// Whether a move's read with `rels` dates it: neither a binding, `flows`, `next`,
/// `observed` nor a balance, which read the timeline (refcore's `m_check_move_lawdates`,
/// its `skip`). L §4.3, §5.14.
fn dating_move(rels: &[Rel]) -> bool {
    rels.iter().any(|rel| {
        !matches!(
            rel,
            Rel::Calendar
                | Rel::Explicit
                | Rel::Pin
                | Rel::LawPin
                | Rel::Bind
                | Rel::Flows
                | Rel::Next
                | Rel::Observed
                | Rel::Bal
        )
    })
}

/// Whether version `id` governs `always` under every cut: its span `always`, no `moves`
/// line moving it (refcore's `ver_timeless`). L §4.3, §4.6.
fn version_timeless(m: &Reality, id: &str) -> bool {
    let Some(version) = m.versions.get(id) else {
        return false;
    };
    let always = version.governs.as_ref().is_some_and(Span::is_always);
    always && version.moved_by.is_empty()
}

/// Whether a row's span is `always` under every cut: no instant, and its own span, else
/// its version's (refcore's `row_timeless`). L §4.3.
fn row_timeless(m: &Reality, row: &Row) -> bool {
    if row.instant.is_some() {
        return false;
    }
    match &row.own_span {
        Some(span) => span.is_always(),
        None => version_timeless(m, &row.version),
    }
}

/// Whether an input isn't law-dated: every row governs always, or it has none; a reading
/// or an observation is (refcore's `InputInfo.lawless`). L §4.3 rule 1.
fn lawless(m: &Reality, input: &InputInfo) -> bool {
    !input.reading && !input.observed && input.rows.iter().all(|row| row_timeless(m, row))
}

/// Whether an input is read without a law date: lawless, or keyed by one period index and
/// read at its key's period (refcore's `undated_read`). L §4.3 rule 1, §4.6 step 1.
fn undated_read(m: &Reality, input: &InputInfo) -> bool {
    lawless(m, input)
        || (!input.reading && !input.observed && matches!(input.time_pos, TimePos::At(_)))
}

/// Whether a def is law-dated by its own rows and rules: a given row, a rule's own span, or
/// its `v` or `replace rule` version, whose span isn't always (refcore's `own_dated`).
/// L §4.3 rule 1.
fn own_dated(m: &Reality, def: &DefInfo) -> bool {
    if def.givens.iter().any(|row| !row_timeless(m, row)) {
        return true;
    }
    for (_, versions) in &def.rules {
        for rv in versions {
            if rv.span.is_some()
                || dated_version(m, rv.v.as_ref())
                || dated_version(m, rv.version.as_ref())
            {
                return true;
            }
        }
    }
    false
}

/// Whether a rule's version is named and governs a span that isn't always.
fn dated_version(m: &Reality, version: Option<&String>) -> bool {
    version.is_some_and(|id| !version_timeless(m, id))
}

/// Whether a read of `target`, a band or a def, dates its reader by what is known so far;
/// `None` for an input or anything else, which each caller decides.
fn dated_item(m: &Reality, at: &AtReads, dated: &LawDated, target: &Node) -> Option<bool> {
    match target.kind {
        NodeKind::Band => Some(dated.band(&target.id)),
        NodeKind::Def => Some(dated_def(m, at, dated, &target.id)),
        NodeKind::Input
        | NodeKind::Move
        | NodeKind::MoveOn
        | NodeKind::Place
        | NodeKind::Ask
        | NodeKind::Price
        | NodeKind::Balances
        | NodeKind::Parts
        | NodeKind::Axis
        | NodeKind::Fill => None,
    }
}

/// Whether def `id`'s `at e` reads, directly or through defs that state their own `at`, a
/// law-dated fact: its readers then need a law date for it (refcore's `at_dated`). L §4.3.
fn at_dated(m: &Reality, at: &AtReads, dated: &LawDated, id: &str) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![id.to_owned()];
    while let Some(next) = stack.pop() {
        if !seen.insert(next.clone()) {
            continue;
        }
        for (target, rels) in at.get(&next).into_iter().flatten() {
            if !dating(rels) {
                continue;
            }
            if target.kind == NodeKind::Band && dated.band(&target.id) {
                return true;
            }
            if target.kind == NodeKind::Input
                && let Some(input) = m.inputs.get(&target.id)
                && !undated_read(m, input)
            {
                return true;
            }
            let Some(def) = m
                .defs
                .get(&target.id)
                .filter(|_| target.kind == NodeKind::Def)
            else {
                continue;
            };
            match &def.at {
                Some(LawDate::Reader) => return true,
                Some(LawDate::Expr(_)) => stack.push(target.id.clone()),
                None if own_dated(m, def) => return true,
                None => {}
            }
        }
    }
    false
}

/// Whether def `id`'s readers need a law date for it: it states `at reader`, or its `at e`
/// reads a law-dated fact (refcore's `needs_reader_law`). L §4.3.
fn needs_reader(m: &Reality, at: &AtReads, dated: &LawDated, id: &str) -> bool {
    match m.defs.get(id).and_then(|def| def.at.as_ref()) {
        Some(LawDate::Reader) => true,
        Some(LawDate::Expr(_)) => at_dated(m, at, dated, id),
        None => false,
    }
}

/// Whether a read of def `id` dates its reader: the def needs its reader's law date, or
/// states no `at` and is law-dated.
fn dated_def(m: &Reality, at: &AtReads, dated: &LawDated, id: &str) -> bool {
    let no_at = m.defs.get(id).is_some_and(|def| def.at.is_none());
    needs_reader(m, at, dated, id) || (no_at && dated.def(id))
}

/// Whether `item`'s reads make it law-dated, a band's rows (`band`) or a def's rules
/// (refcore's `reads_dated`, and its fixed point's def arm, which passes over an input
/// whose rows all govern always). L §4.3, §5.9.
fn reads_dated(m: &Reality, graph: &Graph, at: &AtReads, dated: &LawDated, item: &Node) -> bool {
    for (target, rels) in graph.reads(item).into_iter().flatten() {
        if !dating(rels) {
            continue;
        }
        if let Some(found) = dated_item(m, at, dated, target) {
            if found {
                return true;
            }
            continue;
        }
        if target.kind != NodeKind::Input {
            continue;
        }
        let Some(input) = m.inputs.get(&target.id) else {
            return true;
        };
        if !input.observed && !undated_read(m, input) {
            return true;
        }
    }
    false
}

/// The law-dated bands and defs, to their least fixed point; then `LOAD-LAWDATE` for a
/// law-dated def that states no `at`, and for an ask with no `law` that reaches an `at
/// reader` def, a def whose `at` reads law-dated facts, or a law-dated input or band
/// directly (refcore's `check_lawdates`). L §4.3, §5.18.
///
/// # Errors
///
/// The first such refusal, at the def's or the ask's line.
pub(super) fn check_lawdates(
    m: &Reality,
    walked: &Walked,
    files: &Files,
) -> Result<LawDated, Stop> {
    let graph = &walked.graph;
    let at = &walked.at_reads;
    let mut dated = LawDated {
        bands: BTreeMap::new(),
        defs: BTreeMap::new(),
    };
    for (id, band) in m.bands.entries() {
        let tables = band.tables.iter();
        let in_time = tables
            .filter(|table| !version_timeless(m, &table.version))
            .count();
        dated.bands.insert(id.clone(), in_time > 0);
    }
    for (id, def) in m.defs.entries() {
        dated.defs.insert(id.clone(), own_dated(m, def));
    }
    let mut changed = true;
    while changed {
        changed = false;
        for (id, _) in m.bands.entries() {
            let band = Node::new(NodeKind::Band, id);
            if !dated.band(id) && reads_dated(m, graph, at, &dated, &band) {
                dated.bands.insert(id.clone(), true);
                changed = true;
            }
        }
        for (id, _) in m.defs.entries() {
            let def = Node::new(NodeKind::Def, id);
            if !dated.def(id) && reads_dated(m, graph, at, &dated, &def) {
                dated.defs.insert(id.clone(), true);
                changed = true;
            }
        }
    }
    for (id, def) in m.defs.entries() {
        if dated.def(id) && def.at.is_none() {
            let message = format!("{id} reads law-dated facts and states no at");
            return Err(refuse(
                Load::LawDate,
                message,
                files.get("def", id)?,
                def.decl.line,
            ));
        }
    }
    for ask in m.asks.iter().filter(|a| !m.asks_alone.contains_key(&a.id)) {
        let law_pin = ask.pins.iter().any(|p| matches!(p.form, PinForm::Law(_)));
        if ask.law.is_some() || law_pin {
            continue;
        }
        let Some(reads) = graph.reads(&Node::new(NodeKind::Ask, &ask.id)) else {
            continue;
        };
        for target in walked.ask_reads.get(&ask.id).into_iter().flatten() {
            let rels = reads
                .get(target)
                .ok_or_else(|| invariant(format!("ask {} lost a read", ask.id)))?;
            if !dating(rels) {
                continue;
            }
            if let Some(message) = ask_lawdate(m, at, &dated, &ask.id, target) {
                return Err(refuse(
                    Load::LawDate,
                    message,
                    files.get("ask", &ask.id)?,
                    ask.line,
                ));
            }
        }
    }
    Ok(dated)
}

/// The `LOAD-LAWDATE` message for an ask with no `law` that reads `target`, where it needs
/// one (refcore's `check_lawdates`, its ask loop). L §4.3, §5.18.
fn ask_lawdate(
    m: &Reality,
    at: &AtReads,
    dated: &LawDated,
    ask: &str,
    target: &Node,
) -> Option<String> {
    let id = &target.id;
    if target.kind == NodeKind::Def && needs_reader(m, at, dated, id) {
        let reader = m
            .defs
            .get(id)
            .is_some_and(|def| def.at == Some(LawDate::Reader));
        if reader {
            return Some(format!("ask {ask} reaches an at reader def without law"));
        }
        return Some(format!(
            "ask {ask} reaches {id}, whose at reads law-dated facts at its reader's law date, without law (L §4.3)"
        ));
    }
    let band = target.kind == NodeKind::Band && dated.band(id);
    let input = target.kind == NodeKind::Input
        && m.inputs
            .get(id)
            .is_some_and(|input| read_dated_by_ask(m, input));
    (band || input).then(|| format!("ask {ask} reads law-dated {id} directly, without law"))
}

/// Whether an ask's line is the reader of a law-dated input it reads directly: not
/// observed, not lawless, and not read at its own time key. L §4.3, §4.6 step 1.
fn read_dated_by_ask(m: &Reality, input: &InputInfo) -> bool {
    !input.observed
        && !lawless(m, input)
        && matches!(input.time_pos, TimePos::None | TimePos::Multi)
}

/// `LOAD-LAWDATE` for a move whose `at` reads a def that takes its reader's law date, or a
/// law-dated input or band; and for a move with no `at` whose legs, day or attributes read
/// a law-dated fact (refcore's `m_check_move_lawdates`). L §4.3, §5.14.
///
/// # Errors
///
/// The first such refusal, at the move's line.
pub(super) fn check_move_lawdates(
    m: &Reality,
    walked: &Walked,
    dated: &LawDated,
    files: &Files,
) -> Result<(), Stop> {
    for (id, mv) in m.moves.entries() {
        if let Some(message) = move_lawdate(m, walked, dated, mv) {
            return Err(refuse(
                Load::LawDate,
                message,
                files.get("move", id)?,
                mv.decl.line,
            ));
        }
    }
    Ok(())
}

/// The `LOAD-LAWDATE` message for a move, where it needs one: its `at` reads a def that takes
/// its reader's law date, or a law-dated input or band itself; or, with no `at`, its legs,
/// day or attributes read a law-dated fact (refcore's `m_check_move_lawdates`, one move).
/// L §4.3, §5.14.
fn move_lawdate(m: &Reality, walked: &Walked, dated: &LawDated, mv: &MoveInfo) -> Option<String> {
    let id = mv.id.as_str();
    let at = &walked.at_reads;
    let direct: &[Node] = walked.move_at.get(id).map_or(&[], Vec::as_slice);
    let at_defs: BTreeSet<&str> = direct
        .iter()
        .filter(|node| node.kind == NodeKind::Def)
        .map(|node| node.id.as_str())
        .collect();
    if let Some(def) = at_defs
        .into_iter()
        .find(|def| needs_reader(m, at, dated, def))
    {
        return Some(format!(
            "{id}: its at reads {def}, which takes its law date from its reader"
        ));
    }
    if mv.decl.at.is_some() {
        let read = direct.iter().find(|node| at_reads_dated(m, dated, node))?;
        return Some(format!("move {id}'s at reads law-dated {}", read.id));
    }
    for item in [
        Node::new(NodeKind::Move, id),
        Node::new(NodeKind::MoveOn, id),
    ] {
        for (target, rels) in m.graph.reads(&item).into_iter().flatten() {
            if !dating_move(rels) {
                continue;
            }
            let found = match dated_item(m, at, dated, target) {
                Some(found) => found,
                None => input_dated(m, target),
            };
            if found {
                return Some(format!("{id} reads law-dated facts and states no at"));
            }
        }
    }
    None
}

/// Whether a read in a move's `at` is of a law-dated input or band: the `at` ends every
/// chain, so it reads nothing law-dated itself. L §4.3.
fn at_reads_dated(m: &Reality, dated: &LawDated, node: &Node) -> bool {
    match node.kind {
        NodeKind::Band => dated.band(&node.id),
        NodeKind::Input => input_dated(m, node),
        NodeKind::Def
        | NodeKind::Move
        | NodeKind::MoveOn
        | NodeKind::Place
        | NodeKind::Ask
        | NodeKind::Price
        | NodeKind::Balances
        | NodeKind::Parts
        | NodeKind::Axis
        | NodeKind::Fill => false,
    }
}

/// Whether `node` is a law-dated input as a move reads it: not observed, not built in, not
/// read without a law date.
fn input_dated(m: &Reality, node: &Node) -> bool {
    if node.kind != NodeKind::Input {
        return false;
    }
    m.inputs
        .get(&node.id)
        .is_some_and(|input| !input.observed && !input.builtin && !undated_read(m, input))
}
