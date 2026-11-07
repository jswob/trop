//! Load's parts: a place kind's parts clauses, its arrival and leaving rules, `owns` at a place that keeps parts, a reading's stated parts, and what an arrival rule may read.
//! Spec: L §3.6 (parts, the arrival rule, the leaving rule, `owns` at such a place), §5.7
//! (the parts clauses: where they stand, in which order, `min 0`, no asset kind), §5.19
//! (stated parts: beside their total, at least 0, no two overlapping, at most the total),
//! §5.14.1 (the arrival rule is a reader too), §8.3 (`LOAD-PARTS`, `LOAD-CYCLE`).
//! Twin: engine/tools/refcore/parts.py (`chk_check_parts`, `chk_check_arrival_rule`,
//! `chk_check_takes`, `_strip_scope`, `chk_parts_walk_bound`, `_day_offset`,
//! `chk_check_arrival_reads`); loader.py (`row_item`'s `of`, `check_part_rows`, `of_row_q`,
//! `_parts_overlap`).
//! Invariants: kinds are checked in id order, each clause's refusal at its own line and a
//! kind's at its first clause's; the parts clauses' reads are kept apart from the graph's
//! edges (`parts_reads`), so no reader's cone holds them; a stated part's label is resolved
//! only where load sees it, as refcore resolves it.

use super::check::{Checker, Files, Scope, Stands, VarDom, invariant};
use super::keys::{KeyLit, resolve_key};
use super::steps::{Loader, num_stop, refuse};
use super::strata::reaches_mark;
use super::types::{children, text};
use super::values::{at_row, row_file};
use super::worlds::no_version;
use crate::base::{Code, Load};
use crate::model::{
    DefInfo, Domain, Graph, InputInfo, KeyVal, KindInfo, KindSpec, Labels, NameKind, Node,
    NodeKind, Number, Pat, Reality, Row, RowInstant, Ty, Value,
};
use crate::num::{Money, Num};
use crate::syntax::{
    AddOp, Call, Edge, Expr, Key, LawDate, Lit, OfRow, PartsClause, Postfix, RuleValue, Sign, Step,
    Stop, Takes, Value as Written,
};
use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// A parts clause's word, in the order the clauses go (refcore's `CLAUSE_ORDER`). L §5.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Clause {
    /// `owned by D`.
    Owned,
    /// `batched`.
    Batched,
    /// `step u`.
    Step,
    /// `takes k for w …`.
    Takes,
}

impl Clause {
    /// The clause a written one is.
    fn of(clause: &PartsClause) -> Clause {
        match clause {
            PartsClause::Owned { .. } => Clause::Owned,
            PartsClause::Batched => Clause::Batched,
            PartsClause::Step(_) => Clause::Step,
            PartsClause::Takes(_) => Clause::Takes,
        }
    }

    /// Its place in the order the clauses go.
    fn rank(self) -> u8 {
        match self {
            Clause::Owned => 0,
            Clause::Batched => 1,
            Clause::Step => 2,
            Clause::Takes => 3,
        }
    }

    /// Its words, as a message names it (refcore's `CLAUSE_TEXT`).
    fn text(self) -> &'static str {
        match self {
            Clause::Owned => "owned by",
            Clause::Batched => "batched",
            Clause::Step => "step",
            Clause::Takes => "takes",
        }
    }
}

impl Checker {
    /// The parts clauses of every kind that states them, in id order, then `owns` rows at
    /// the places that keep parts (refcore's `chk_check_parts`). L §5.7, §3.6.
    ///
    /// # Errors
    ///
    /// `LOAD-PARTS` as L §8.3 words it; the arrival rule's `LOAD-NAME`, the leaving rule's
    /// `LOAD-DUP`, `LOAD-NAME` and `LOAD-SHADOW`; a refusal met typing a clause's terms.
    pub(super) fn check_parts(&mut self, m: &Reality, files: &Files) -> Result<(), Stop> {
        let mut kinds: Vec<&KindInfo> = m
            .kinds
            .entries()
            .map(|(_, kind)| kind)
            .filter(|kind| !kind.parts.is_empty())
            .collect();
        kinds.sort_by(|a, b| a.id.cmp(&b.id));
        let mut kept = Vec::new();
        for kind in kinds {
            let file = files.get("kind", &kind.id)?;
            clauses(m, kind, file)?;
            kept.push(kind.id.clone());
            for (clause, line) in &kind.parts {
                if let PartsClause::Owned { rule, ties } = clause {
                    self.arrival_rule(m, kind, rule, ties.as_deref(), (file, *line))?;
                }
            }
            for (clause, line) in &kind.parts {
                if let PartsClause::Takes(takes) = clause {
                    self.takes(m, kind, takes, (file, *line))?;
                }
            }
        }
        owns_rows(m, &kept, files)?;
        self.graph.parts_kinds = kept;
        Ok(())
    }

    /// The `owned` clause's arrival rule: a def of type `num` keyed by an event kind and by
    /// `parties`, at the law date `m.day` of its movement key; its `ties by` typed over the
    /// def's keys (refcore's `chk_check_arrival_rule`). L §3.6, §5.7.
    fn arrival_rule(
        &mut self,
        m: &Reality,
        kind: &KindInfo,
        rule: &str,
        ties: Option<&[Expr]>,
        (file, line): (&str, u32),
    ) -> Result<(), Stop> {
        let not_one = || {
            refuse(
                Load::Parts,
                format!(
                    "arrival rule {rule} is not a num keyed by a movement and a party at its movement's day"
                ),
                file,
                line,
            )
        };
        match m.names.kind(rule) {
            None => {
                return Err(
                    refuse(Load::Name, format!("unknown id {rule}"), file, line).naming(rule)
                );
            }
            Some(NameKind::Def) => {}
            Some(_) => return Err(not_one()),
        }
        let def = m
            .defs
            .get(rule)
            .ok_or_else(|| invariant(format!("def {rule} is named but not built")))?;
        if !is_arrival_rule(m, def) {
            return Err(not_one());
        }
        self.graph
            .parts_reads
            .insert(Node::new(NodeKind::Def, rule));
        if let Some(ties) = ties {
            let item = Node::new(NodeKind::Parts, &kind.id);
            let scope = self
                .def_scope(m, def, Stands::Rule, (file, line))?
                .for_item(item.clone());
            for term in ties {
                self.expr(m, &scope, term, None, 0)?;
            }
            strip(&mut self.graph, &item);
        }
        Ok(())
    }

    /// The `takes` clause: its part and outflow bound, `v V` a version, each term typed over
    /// both, the `ties by` over the part; then the kind's `step`, any literal allowed
    /// (refcore's `chk_check_takes`). L §3.6.
    fn takes(
        &mut self,
        m: &Reality,
        kind: &KindInfo,
        takes: &Takes,
        (file, line): (&str, u32),
    ) -> Result<(), Stop> {
        let (part, outflow) = (&takes.part, &takes.outflow);
        if part == outflow {
            return Err(refuse(
                Load::Dup,
                format!("{outflow} is declared twice (takes {part} for {outflow})"),
                file,
                line,
            ));
        }
        if let Some(v) = &takes.v
            && !m.versions.contains(v)
        {
            let message = format!("takes of {}: v {v} names no version", kind.id);
            let (code, message) = no_version(m, &kind.id, v, message);
            return Err(refuse(code, message, file, line));
        }
        for var in [part, outflow] {
            if m.names.contains_key(var) {
                return Err(refuse(
                    Load::Shadow,
                    format!("{var} shadows a top-level id"),
                    file,
                    line,
                ));
            }
        }
        let item = Node::new(NodeKind::Parts, &kind.id);
        let mut scope = Scope::new(item.clone(), Stands::Rule, file, line);
        let k = self.new_var(part, part_type(), VarDom::Arg, Vec::new())?;
        let w = self.new_var(
            outflow,
            Ty::Key("events".to_owned()),
            VarDom::Arg,
            Vec::new(),
        )?;
        scope.vars.insert(part.clone(), k);
        scope.vars.insert(outflow.clone(), w);
        scope.keyvars = vec![part.clone(), outflow.clone()];
        for term in &takes.order {
            self.expr(m, &scope, term, None, 0)?;
        }
        if let Some(ties) = &takes.ties {
            let mut tie_scope = Scope::new(item.clone(), Stands::Rule, file, line);
            let tie_part = self.new_var(part, part_type(), VarDom::Arg, Vec::new())?;
            tie_scope.vars.insert(part.clone(), tie_part);
            for term in ties {
                self.expr(m, &tie_scope, term, None, 0)?;
            }
        }
        strip(&mut self.graph, &item);
        for (clause, step_line) in &kind.parts {
            if let PartsClause::Step(step) = clause {
                let mut step_scope = Scope::new(item.clone(), Stands::Decl, file, *step_line);
                step_scope.allow_lit = true;
                self.expr(m, &step_scope, &step_expr(step), None, 0)?;
            }
        }
        strip(&mut self.graph, &item);
        Ok(())
    }
}

/// A kind's clauses as L §5.7 allows them: a place kind's, under no kind that keeps parts,
/// each once and in order, `step` and `takes` beside `owned by` or `batched`, `min 0`, no
/// asset kind (refcore's `chk_check_parts`, its kind checks). L §5.7.
fn clauses(m: &Reality, kind: &KindInfo, file: &str) -> Result<(), Stop> {
    let id = &kind.id;
    let Some((_, first)) = kind.parts.first() else {
        return Ok(());
    };
    let at_first = |message: String| refuse(Load::Parts, message, file, *first);
    if kind.spec != KindSpec::Place {
        return Err(at_first(format!(
            "kind {id} keeps parts and is not a place kind"
        )));
    }
    if let Some(up) = m
        .chain(id)
        .into_iter()
        .skip(1)
        .find(|k| !k.parts.is_empty())
    {
        return Err(at_first(format!(
            "{id} states parts clauses under {}, which keeps parts",
            up.id
        )));
    }
    let mut seen: Vec<Clause> = Vec::new();
    for (written, line) in &kind.parts {
        let clause = Clause::of(written);
        if seen.contains(&clause) {
            return Err(refuse(
                Load::Parts,
                format!("kind {id}: {} stated twice", clause.text()),
                file,
                *line,
            ));
        }
        if let Some(before) = seen.last()
            && clause.rank() < before.rank()
        {
            return Err(refuse(
                Load::Parts,
                format!(
                    "kind {id}: {} after {} (the parts clauses go owned by, batched, step, takes)",
                    clause.text(),
                    before.text()
                ),
                file,
                *line,
            ));
        }
        seen.push(clause);
    }
    if !(seen.contains(&Clause::Owned) || seen.contains(&Clause::Batched)) {
        return Err(at_first(format!("kind {id}: owned by or batched missing")));
    }
    for needed in [Clause::Step, Clause::Takes] {
        if !seen.contains(&needed) {
            return Err(at_first(format!("kind {id}: {} missing", needed.text())));
        }
    }
    let sign = match m.kind_sign(id) {
        Some(Sign::Min0) => None,
        Some(Sign::Signed) => Some("signed"),
        Some(Sign::Max0) => Some("max 0"),
        // refcore prints the missing sign as Python's `None`.
        None => Some("None"),
    };
    if let Some(sign) = sign {
        return Err(at_first(format!("kind {id} keeps parts and is {sign}")));
    }
    if m.subtree("assets").contains(id) {
        return Err(at_first(format!(
            "kind {id} keeps parts and is an asset kind"
        )));
    }
    Ok(())
}

/// Whether a def is an arrival rule: keyed by an event kind and then by `parties` (either a
/// subkind), of type `num`, at `m.day` of its movement key (refcore's
/// `chk_check_arrival_rule`, its `ok`). L §3.6.
fn is_arrival_rule(m: &Reality, def: &DefInfo) -> bool {
    let [movement, party] = def.binds.as_slice() else {
        return false;
    };
    let event = matches!(&movement.domain, Domain::Kind(k)
        if m.kinds.get(k).is_some_and(|kind| kind.spec == KindSpec::Event));
    let parties = matches!(&party.domain, Domain::Kind(k) if m.subtree("parties").contains(k));
    let num = matches!(&def.ty, Ty::Num(unit, _) if unit.is_one());
    let at_day = match &def.at {
        Some(LawDate::Expr(at)) => is_day_of(at, &movement.var),
        Some(LawDate::Reader) | None => false,
    };
    event && parties && num && at_day
}

/// Whether `e` is `var.day`, in any brackets.
fn is_day_of(e: &Expr, var: &str) -> bool {
    let mut at = e;
    while let Expr::Paren(inner) = at {
        at = inner;
    }
    let Expr::Post(base, posts) = at else {
        return false;
    };
    matches!(base.as_ref(), Expr::Name(name) if name == var)
        && matches!(posts.as_slice(), [Postfix::Field(field)] if field == "day")
}

/// A part's type, read through its built-in fields (refcore's `PART_TY`). L §3.6.
fn part_type() -> Ty {
    Ty::Record(vec![
        ("owner".to_owned(), Ty::Key("parties".to_owned())),
        ("batch".to_owned(), Ty::Key("events".to_owned())),
    ])
}

/// A `step` clause's unit as the expression refcore's `step_unit` parses it to.
fn step_expr(step: &Step) -> Expr {
    match step {
        Step::Number(number) => Expr::Lit(Lit::Num(number.clone())),
        Step::Minor(cur) => Expr::Call(Box::new(Call {
            name: "minor".to_owned(),
            prefix: Vec::new(),
            band_key: None,
            args: vec![Expr::Lit(Lit::Cur(cur.clone()))],
        })),
        Step::Ref(e) => e.clone(),
    }
}

/// Takes back the edges and marks a parts clause's terms recorded on their pseudo-item,
/// keeping what they read (refcore's `_strip_scope`). L §3.6, §8.5.
fn strip(graph: &mut Graph, item: &Node) {
    if let Some(reads) = graph.edges.remove(item) {
        graph.parts_reads.extend(reads.into_keys());
    }
    graph.marks.remove(item);
}

/// `owns[p, x]` where x keeps parts: one place has one source of ownership (refcore's
/// `chk_check_parts`, its end). L §3.6.
fn owns_rows(m: &Reality, kept: &[String], files: &Files) -> Result<(), Stop> {
    if kept.is_empty() {
        return Ok(());
    }
    let Some(owns) = m.inputs.get("owns") else {
        return Ok(());
    };
    for row in &owns.rows {
        let names: Vec<String> = row
            .pattern
            .iter()
            .map(|pat| match pat {
                Pat::Key(KeyVal::Thing(name) | KeyVal::Key { name, .. }) => name.clone(),
                // refcore's key value with no name prints as Python's `None`.
                Pat::Key(KeyVal::Day(_) | KeyVal::Period(_) | KeyVal::Int(_) | KeyVal::Mv(_)) => {
                    "None".to_owned()
                }
                Pat::Star | Pat::Range(..) => "*".to_owned(),
            })
            .collect();
        for name in &names {
            let keeps = m
                .things
                .get(name)
                .is_some_and(|thing| m.thing_parts(thing).is_some());
            if keeps {
                return Err(refuse(
                    Load::Parts,
                    format!("owns[{}]: {name} keeps parts", names.join(", ")),
                    files.of_version(&row.version)?,
                    row.line,
                ));
            }
        }
    }
    Ok(())
}

/// Each kind's arrival rule, in kind order: the def its `owned` clause names.
fn arrival_rules(m: &Reality, graph: &Graph) -> Vec<String> {
    let mut out = Vec::new();
    for id in &graph.parts_kinds {
        let Some(kind) = m.kinds.get(id) else {
            continue;
        };
        for (clause, _) in &kind.parts {
            if let PartsClause::Owned { rule, .. } = clause {
                out.push(rule.clone());
            }
        }
    }
    out
}

/// The defs an arrival rule reaches through defs that reach a running read: the walk runs
/// them when it books an arrival, so they are walk-bound (refcore's `chk_parts_walk_bound`).
/// L §5.14.1; E §3.2.
pub(super) fn parts_walk_bound(m: &Reality, graph: &Graph) -> BTreeSet<String> {
    let mut stack: Vec<Node> = arrival_rules(m, graph)
        .iter()
        .map(|rule| Node::new(NodeKind::Def, rule))
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(next) = stack.pop() {
        if seen.contains(&next) {
            continue;
        }
        if let Some(reads) = graph.reads(&next) {
            stack.extend(reads.keys().filter(|t| t.kind == NodeKind::Def).cloned());
        }
        seen.insert(next);
    }
    seen.into_iter()
        .filter(|def| reaches_mark(graph, def))
        .map(|def| def.id)
        .collect()
}

/// What an arrival rule reads of its own arrival's day or later, which load sees in the rule
/// itself, written from its movement key's day: `bal(y, m.day)` (not `start`) or a later
/// day, `value(y, m.day)` or later (refcore's `chk_check_arrival_reads`). L §5.14.1, §3.6.
///
/// # Errors
///
/// `LOAD-CYCLE`, at the rule's line.
pub(super) fn arrival_reads(m: &Reality, graph: &Graph, file_of: &Files) -> Result<(), Stop> {
    for rule_id in arrival_rules(m, graph) {
        let Some(def) = m.defs.get(&rule_id) else {
            continue;
        };
        if let Some((message, line)) = own_day_read(def)? {
            return Err(refuse(
                Load::Cycle,
                format!("arrival rule {} reads {message}", def.id),
                file_of.get("def", &def.id)?,
                line,
            ));
        }
    }
    Ok(())
}

/// An arrival rule's first read of its arrival's day's end or later, in its rules' order,
/// each rule's value then its `when`, with the rule's line.
fn own_day_read(def: &DefInfo) -> Result<Option<(String, u32)>, Stop> {
    let Some(var) = def.binds.first().map(|b| b.var.as_str()) else {
        return Ok(None);
    };
    for version in def.rules.iter().flat_map(|(_, versions)| versions) {
        let rule = &version.rule;
        let value = match &rule.value {
            RuleValue::Expr(e) => Some(e),
            RuleValue::Solve(_) | RuleValue::Hole(_) => None,
        };
        for part in [value, rule.when.as_ref()].into_iter().flatten() {
            if let Some(message) = reads_own_day(part, var)? {
                return Ok(Some((message, rule.line)));
            }
        }
    }
    Ok(None)
}

/// The first read in `e` of the arrival's day's end or later, as the message names it.
fn reads_own_day(e: &Expr, var: &str) -> Result<Option<String>, Stop> {
    let mut stack = vec![e];
    while let Some(next) = stack.pop() {
        // A balance read at a day, not a projection and not `modelled` (refcore's `_form`
        // 'day').
        if let Expr::Bal(bal) = next
            && bal.proj.from.is_none()
            && !bal.modelled
            && let Some(day) = bal.args.last()
            && let Some(off) = day_offset(day, var)?
            && (off > 0 || (off == 0 && !bal.start))
        {
            let start = if bal.start { " start" } else { "" };
            return Ok(Some(format!(
                "bal({}, {}{start}): {}",
                text(&bal.place),
                text(day),
                which_day(off)
            )));
        }
        if let Expr::Value(value) = next
            && let Some(off) = day_offset(&value.day, var)?
            && off >= 0
        {
            return Ok(Some(format!(
                "value({}, {}): {}",
                text(&value.place),
                text(&value.day),
                which_day(off)
            )));
        }
        let mut under = children(next);
        under.reverse();
        stack.extend(under);
    }
    Ok(None)
}

/// The day an arrival rule reads, against its arrival's.
fn which_day(off: i64) -> &'static str {
    if off == 0 {
        "the end of its arrival's day"
    } else {
        "a day after its arrival's"
    }
}

/// 0 for `var.day`, n for `var.day + n` as written (an int or a duration in days), none for
/// any other day (refcore's `_day_offset`). L §5.14.1, §8.3 item 9.
fn day_offset(e: &Expr, var: &str) -> Result<Option<i64>, Stop> {
    let overflow = || invariant("a day offset past the integers".to_owned());
    let mut total: i64 = 0;
    let mut at = e;
    loop {
        while let Expr::Paren(inner) = at {
            at = inner;
        }
        if is_day_of(at, var) {
            return Ok(Some(total));
        }
        let Expr::Sum(first, rest) = at else {
            return Ok(None);
        };
        for (op, term) in rest {
            let Expr::Lit(Lit::Num(number)) = term else {
                return Ok(None);
            };
            if !number.whole {
                return Ok(None);
            }
            let Some(n) = number.value.to_i64() else {
                return Ok(None);
            };
            total = match op {
                AddOp::Add => total.checked_add(n),
                AddOp::Sub => total.checked_sub(n),
            }
            .ok_or_else(overflow)?;
        }
        at = first;
    }
}

/// A stated part's label where load sees it: a party, a batch (a movement), or the pair
/// (refcore's `of_row_q`'s tuples). L §5.19.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PartQ {
    /// A party.
    Owner(String),
    /// A batch: a move instance or an observed row.
    Batch(String),
    /// A party and a batch.
    Pair(String, String),
}

impl PartQ {
    /// Its one label's name.
    fn name(&self) -> &str {
        match self {
            PartQ::Owner(name) | PartQ::Batch(name) | PartQ::Pair(name, _) => name,
        }
    }

    /// As a message names it: the label, or `(p, b)`.
    fn text(&self) -> String {
        match self {
            PartQ::Owner(name) | PartQ::Batch(name) => name.clone(),
            PartQ::Pair(party, batch) => format!("({party}, {batch})"),
        }
    }

    /// The sorts of label a place must keep for it.
    fn needs(&self) -> Labels {
        match self {
            PartQ::Owner(_) => Labels {
                owner: true,
                batch: false,
            },
            PartQ::Batch(_) => Labels {
                owner: false,
                batch: true,
            },
            PartQ::Pair(..) => Labels {
                owner: true,
                batch: true,
            },
        }
    }

    /// Its party and its batch, each where it names one.
    fn pattern(&self) -> (Option<&str>, Option<&str>) {
        match self {
            PartQ::Owner(party) => (Some(party), None),
            PartQ::Batch(batch) => (None, Some(batch)),
            PartQ::Pair(party, batch) => (Some(party), Some(batch)),
        }
    }

    /// Whether two labels may name one part: one label twice, a party and a batch, or a
    /// party or a batch and a pair that holds it (refcore's `_parts_overlap`). L §5.19.
    fn overlaps(&self, other: &PartQ) -> bool {
        let meet = |a: Option<&str>, b: Option<&str>| a.is_none() || b.is_none() || a == b;
        let ((party, batch), (party2, batch2)) = (self.pattern(), other.pattern());
        meet(party, party2) && meet(batch, batch2)
    }
}

/// A stated part checked so far: the total row it stands beside (its place in the input's
/// rows), its label, its value.
struct StatedPart {
    /// The total's place in the input's rows.
    total: usize,
    /// Its label, where load sees it.
    q: Option<PartQ>,
    /// Its value, where load converts it.
    value: Option<Value>,
    /// Its row's place in the input's part rows.
    row: usize,
}

impl Loader {
    /// A balance reading's stated parts: each beside a total of its place in its version at its
    /// instant, at a place that keeps its label's sort, at least 0, no two of a total's
    /// overlapping, and together at most the total (refcore's `check_part_rows`). L §5.19.
    ///
    /// # Errors
    ///
    /// `LOAD-PARTS`, at the part's row; a refusal of its value, as a reading's.
    pub(super) fn check_part_rows(
        &self,
        id: &str,
        input: &InputInfo,
        files: &BTreeMap<String, String>,
    ) -> Result<(), Stop> {
        let mut parts: Vec<StatedPart> = Vec::new();
        for (at_part, row) in input.part_rows.iter().enumerate() {
            let file = row_file(files, row)?;
            let at = |message: String| refuse(Load::Parts, message, &file, row.line);
            let (_, value) = self
                .place_reading(id, row)
                .map_err(|stop| at_row(stop, &file, row.line))?;
            let Some(Pat::Key(KeyVal::Thing(place))) = row.pattern.first() else {
                return Err(invariant(format!("a stated part of {id} names no place")));
            };
            let labels = self
                .m
                .things
                .get(place)
                .and_then(|thing| self.m.thing_parts(thing))
                .map_or(
                    Labels {
                        owner: false,
                        batch: false,
                    },
                    KindInfo::labels,
                );
            let q = match &row.of {
                Some(of) => self.of_row_q(of, &file, row.line)?,
                None => None,
            };
            if let Some(q) = &q
                && !labels.holds(q.needs())
            {
                return Err(at(format!(
                    "of {} at {place}, which keeps no labels of its sort",
                    q.text()
                )));
            }
            let instant = instant_text(row)?;
            let mut total = None;
            for (at_row, t) in input.rows.iter().enumerate() {
                if t.version == row.version
                    && matches!(t.pattern.first(), Some(Pat::Key(KeyVal::Thing(p))) if p == place)
                    && t.instant.is_some()
                    && instant_text(t)? == instant
                    && t.value != Written::None
                {
                    total = Some(at_row);
                }
            }
            let Some(total) = total else {
                return Err(at(format!(
                    "the parts of {place} at {instant} have no total in {}",
                    row.version
                )));
            };
            let qt = q
                .as_ref()
                .map_or_else(|| of_text(row.of.as_ref()), PartQ::text);
            if value.as_ref().is_some_and(is_negative) {
                return Err(at(format!("the stated part of {qt} at {place} is below 0")));
            }
            if let Some(q) = &q
                && let Some(q2) = overlapping(&parts, total, q)
            {
                return Err(at(format!(
                    "the stated parts {q2} and {} of {place} overlap",
                    q.text()
                )));
            }
            parts.push(StatedPart {
                total,
                q,
                value,
                row: at_part,
            });
        }
        self.parts_over_totals(id, input, &parts, files)
    }

    /// Each total's stated parts sum to at most it, where load sees every amount (refcore's
    /// `check_part_rows`, its end). L §5.19.
    fn parts_over_totals(
        &self,
        id: &str,
        input: &InputInfo,
        parts: &[StatedPart],
        files: &BTreeMap<String, String>,
    ) -> Result<(), Stop> {
        for (at_total, total) in input.rows.iter().enumerate() {
            let mine: Vec<&StatedPart> = parts.iter().filter(|p| p.total == at_total).collect();
            let Some(last) = mine.last() else {
                continue;
            };
            let (_, Some(total_value)) = self.place_reading(id, total)? else {
                continue;
            };
            let values: Option<Vec<&Value>> = mine.iter().map(|p| p.value.as_ref()).collect();
            let Some(values) = values else {
                continue;
            };
            let Some(sum) = amounts_over(&total_value, &values)? else {
                continue;
            };
            let Some(Pat::Key(KeyVal::Thing(place))) = total.pattern.first() else {
                return Err(invariant(format!("a total of {id} names no place")));
            };
            let last = input
                .part_rows
                .get(last.row)
                .ok_or_else(|| invariant(format!("a stated part of {id} is gone")))?;
            return Err(refuse(
                Load::Parts,
                format!(
                    "the stated parts of {place} sum to {}, over its total {}",
                    sum.text(&self.m).map_err(num_stop)?,
                    total_value.text(&self.m).map_err(num_stop)?
                ),
                &row_file(files, last)?,
                last.line,
            ));
        }
        Ok(())
    }

    /// The label a stated part's `of` names, where load sees it: a party's thing, a move
    /// instance or an observed row, or their pair; none where it names anything else or load
    /// can't resolve it (refcore's `of_row_q`, `sref`). L §5.19.
    fn of_row_q(&self, of: &OfRow, file: &str, line: u32) -> Result<Option<PartQ>, Stop> {
        match of {
            OfRow::One(e) => self.of_one(e, file, line),
            OfRow::Pair(a, b) => {
                let (Some(a), Some(b)) = (self.of_one(a, file, line)?, self.of_one(b, file, line)?)
                else {
                    return Ok(None);
                };
                Ok(Some(PartQ::Pair(a.name().to_owned(), b.name().to_owned())))
            }
        }
    }

    /// One side of a stated part's label (refcore's `of_row_q`, its `one`).
    fn of_one(&self, e: &Expr, file: &str, line: u32) -> Result<Option<PartQ>, Stop> {
        let (name, keys): (&str, &[Key]) = if let Expr::Name(name) = e {
            (name, &[])
        } else if let Expr::Post(base, posts) = e
            && let Expr::Name(name) = base.as_ref()
            && let [Postfix::Keys(keys)] = posts.as_slice()
        {
            (name, keys)
        } else {
            return Ok(None);
        };
        match self.m.names.kind(name) {
            None | Some(NameKind::Place | NameKind::Kind) => Ok(self.party(name)),
            Some(NameKind::Move) => self.batch_of_move(name, keys, file, line),
            Some(NameKind::Input) => Ok(self.batch_of_row(name, keys)),
            Some(
                NameKind::Unit
                | NameKind::Index
                | NameKind::Axis
                | NameKind::Def
                | NameKind::Band
                | NameKind::Fill
                | NameKind::Version
                | NameKind::Ask
                | NameKind::Source
                | NameKind::Cut,
            ) => Ok(None),
        }
    }

    /// A thing of `parties` named as a label (refcore's `of_row_q`, a thing).
    fn party(&self, name: &str) -> Option<PartQ> {
        let thing = self.m.things.get(name)?;
        let parties = self.m.subtree("parties");
        let party = thing.base.as_ref().is_some_and(|b| parties.contains(b))
            || thing.entries.iter().any(|e| parties.contains(&e.kind));
        party.then(|| PartQ::Owner(name.to_owned()))
    }

    /// A move instance named as a batch, its keys resolved as literals; none where they
    /// aren't (refcore's `sref`, its move, a refusal taken as no label).
    fn batch_of_move(
        &self,
        name: &str,
        keys: &[Key],
        file: &str,
        line: u32,
    ) -> Result<Option<PartQ>, Stop> {
        let Some(mv) = self.m.moves.get(name) else {
            return Ok(None);
        };
        if keys.len() != mv.binds.len() {
            return Ok(None);
        }
        let mut texts = Vec::new();
        for (key, bind) in keys.iter().zip(&mv.binds) {
            let Some(written) = (match key {
                Key::Expr(e) => KeyLit::of_expr(e),
                Key::Star => None,
            }) else {
                return Ok(None);
            };
            match resolve_key(&self.m, &written, &bind.domain, file, line) {
                Ok(value) => texts.push(value.text()),
                // A key its index doesn't hold (`LOAD-KEY`) names no batch here; the key is
                // refused where it is typed. Any other stop is passed on.
                Err(Stop::Refused(r)) if r.code() == Code::Load(Load::Key) => return Ok(None),
                Err(stop) => return Err(stop),
            }
        }
        let id = if texts.is_empty() {
            name.to_owned()
        } else {
            format!("{name}[{}]", texts.join(", "))
        };
        Ok(Some(PartQ::Batch(id)))
    }

    /// An observed row named as a batch, by its one key (refcore's `sref`, an observation).
    fn batch_of_row(&self, name: &str, keys: &[Key]) -> Option<PartQ> {
        let input = self.m.inputs.get(name)?;
        if !input.observed {
            return None;
        }
        let [Key::Expr(key)] = keys else {
            return None;
        };
        let key = if let Expr::Name(text) | Expr::Lit(Lit::Key(text)) = key {
            text.clone()
        } else if let Expr::Lit(Lit::Num(number)) = key {
            number.text.clone()
        } else {
            return None;
        };
        Some(PartQ::Batch(format!("{name}[{key}]")))
    }
}

/// A row's instant as a message names it: `2026-01-01 start`, `after x` (refcore's
/// `instant_text`).
fn instant_text(row: &Row) -> Result<String, Stop> {
    match &row.instant {
        Some(RowInstant::At(day, edge)) => {
            let edge = match edge {
                Some(Edge::Start) => "start",
                Some(Edge::End) => "end",
                // refcore prints the missing edge as Python's `None`.
                None => "None",
            };
            Ok(format!("{} {edge}", day.text()))
        }
        Some(RowInstant::After(e)) => Ok(format!("after {}", text(e))),
        None => Err(invariant("a reading with no instant".to_owned())),
    }
}

/// The label, as a message names it, of a part stated earlier beside the same total that may
/// name one part with `q` (refcore's `check_part_rows`, its overlap loop). L §5.19.
fn overlapping(parts: &[StatedPart], total: usize, q: &PartQ) -> Option<String> {
    for part in parts {
        if part.total != total {
            continue;
        }
        if let Some(earlier) = &part.q
            && earlier.overlaps(q)
        {
            return Some(earlier.text());
        }
    }
    None
}

/// A stated part's `of` as written, where load sees no label (refcore's `_txt_node`).
fn of_text(of: Option<&OfRow>) -> String {
    match of {
        Some(OfRow::One(e)) => text(e),
        Some(OfRow::Pair(..)) => "ofpair".to_owned(),
        None => String::new(),
    }
}

/// Whether an amount is below 0.
fn is_negative(value: &Value) -> bool {
    matches!(value, Value::Money(money) if money.is_negative())
        || matches!(value, Value::Number(number) if number.q.is_negative())
}

/// The parts' sum where it is over the total, every amount of the total's unit; none where
/// it isn't over, or an amount is of another unit (refcore sums their quantities as they
/// are, a corner only a place of unit `any` meets).
fn amounts_over(total: &Value, parts: &[&Value]) -> Result<Option<Value>, Stop> {
    if let Value::Money(t) = total {
        let mut sum = Money::from_minor(0, t.currency());
        for part in parts {
            let Value::Money(p) = part else {
                return Ok(None);
            };
            if p.currency() != t.currency() {
                return Ok(None);
            }
            sum = sum.checked_add(p).map_err(num_stop)?;
        }
        let over = sum.compare(t).map_err(num_stop)? == Ordering::Greater;
        return Ok(over.then_some(Value::Money(sum)));
    }
    let Value::Number(t) = total else {
        return Ok(None);
    };
    let mut sum = Num::zero();
    for part in parts {
        let Value::Number(p) = part else {
            return Ok(None);
        };
        if p.class != t.class {
            return Ok(None);
        }
        sum = sum.checked_add(&p.q).map_err(num_stop)?;
    }
    let over = sum.compare(&t.q).map_err(num_stop)? == Ordering::Greater;
    Ok(over.then(|| {
        Value::Number(Number {
            q: sum,
            class: t.class.clone(),
        })
    }))
}
