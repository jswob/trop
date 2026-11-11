//! Def cells: a def read at its keys, its law date set by `at`, its value from a given or from the rule that wins, kept in the memo.
//! Spec: L §5.11 (def and rule: `at`, givens, the versions of a rule in force, `when`,
//! `beats`, the candidates, a hole `when`, `RUN-CONFLICT`, `RUN-NO-RULE`), §4.3 (the law date),
//! §8.2 (`assumed`, `given`, `missing` entries); E §3.2 (the memo: one cell per def, keys and
//! context, its result kept, refusal included; a cell read while it is computed is `RUN-CYCLE`);
//! L §4.4 (a window a def computes is the read's value in its cell's world; one it passes on
//! keeps the read that made it).
//! Twin: engine/tools/refcore/evaluate.py (`read_def`, `_read_def`, `law_value`, `def_cell`,
//! `coerce_def`, `rule_in_force`, `beats_order`, `eval_rules`).
//! Invariants: every result a cell computes is kept, with the register entries it met; the
//! entries reach every reader of the cell; recursion through cells is counted (R-16).

use super::state::{
    Cell, Ctx, DEPTH_LIMIT, Engine, Env, Task, a_day, invariant, num_stop, refuse, refuse_load,
};
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{
    Class, DefInfo, KeyVal, Node, NodeKind, Number, Reality, RuleVer, Shown, Ty, Unit, Value,
    pattern_text,
};
use crate::read::{Clocks, Test, candidates, choose, origin, rule_in_force, worded};
use crate::syntax::{LawDate, RuleValue, Stop};
use crate::time::Day;

use super::convert::{ConvertFor, Converted, convert};
use super::memo::{CellId, CellKey, LawClass, Read, rows_class, rules_class};
use super::read::{at_keys, pinned_at, shown};
use super::trace::Seen;
use super::windows::as_read;

impl Engine {
    /// A read of def `def` at keys already made keys (refcore's `read_def`): its law date by
    /// its `at`, then its cell. Its key binds' `where` and its `at` are the def's own, so the
    /// def holds the projections they write (L §5.14.1: "the item holding the read: in a def";
    /// "the def that holds it, never … who reads the def"). L §5.11, §4.3.
    ///
    /// # Errors
    ///
    /// The refusals of the def's evaluation.
    pub(super) fn read_def_at(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let outer = self.holding(Node::new(NodeKind::Def, &def.id));
        let read = self.read_def_held(m, def, keys, ctx);
        self.holder = outer;
        read
    }

    /// [`Engine::read_def_at`] with the def holding what its `where` and `at` read. L §5.11,
    /// §5.14.1.
    fn read_def_held(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        // the cell's context is made in a frame of its own, gone while the cell reads on
        // (R-16)
        let env = key_env(def, keys);
        let cell = match self.cell_ctx(m, def, (keys, &env), ctx)? {
            Err(read) => return Ok(read),
            Ok(cell) => cell,
        };
        let value = self.def_cell(m, def, keys, &env, cell)?;
        // a window the def computed is this read's value, in the world its cell is computed
        // in; one it passes on keeps the read that made it (L §4.4)
        Ok(as_read(value, (&def.id, keys), cell, None))
    }

    /// The context def `def`'s cell at its keys is computed in, `env` binding them: its law
    /// date by its `at`, after its key binds' `where`; else the read's value already: a pin
    /// of the cell, or a hole law date (refcore's `read_def`). L §5.11, §4.3, §5.14.1.
    fn cell_ctx(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        (keys, env): (&[KeyVal], &Env),
        ctx: Ctx,
    ) -> Result<Result<Ctx, Value>, Stop> {
        let ctx = if m.graph.walk_bound.contains(&def.id) {
            ctx
        } else {
            Ctx { at: None, ..ctx }
        };
        self.index_assumed(m, &def.binds);
        // a pin wins over the def's rules and givens (L §5.11); the def's cell keeps only the
        // pins its def reaches (E §3.2)
        if let Some(pinned) = self.pin_of(&def.id, keys, ctx)? {
            return pinned_at(pinned, &def.binds, keys).map(Err);
        }
        let ctx = self.trimmed(m, &def.id, ctx)?;
        for bind in &def.binds {
            let Some(filter) = &bind.node.filter else {
                continue;
            };
            if self.ev(m, filter, env, ctx.at_law(None))? != Value::Bool(true) {
                return Err(refuse(
                    Run::Domain,
                    format!("{} read at a key its key bind's where leaves out", def.id),
                ));
            }
        }
        let cell = match &def.at {
            None => ctx.at_law(None),
            Some(LawDate::Reader) => {
                if ctx.law.is_none() {
                    return Err(refuse_load(
                        Load::LawDate,
                        format!("ask reaches an at reader def ({}) without law", def.id),
                    ));
                }
                ctx
            }
            Some(LawDate::Expr(e)) => {
                let law = self.ev(m, e, env, ctx)?;
                if let Value::Hole(_) = law {
                    return Ok(Err(law));
                }
                if let Value::Window(_) = law {
                    // a window `at` reads the def once per law-date class the window overlaps
                    // (L §4.3, §5.14 `at e`): not a load refusal, and not built (M2, with
                    // `windows each`)
                    return Err(Stop::NotBuilt("a def's law date set by a window"));
                }
                let Value::Day(day) = law else {
                    return Err(bad_law_date(m, def, &law)?);
                };
                ctx.at_law(Some(a_day(day)?))
            }
        };
        Ok(Ok(cell))
    }

    /// A def's cell: the memo's result, or the rules evaluated now and kept (refcore's
    /// `def_cell`); past the counted depth the cell waits on the work stack and the read
    /// unwinds (R-16). E §3.2.
    pub(super) fn def_cell(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        keys: &[KeyVal],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        // the memo's lookup and the keeping run in frames of their own, gone while the rules
        // read on (R-16)
        let id = match self.open_cell(def, keys, ctx)? {
            Err(read) => return read,
            Ok(id) => id,
        };
        let outer = self.holding(Node::new(NodeKind::Def, &def.id));
        let node = self.trace_def(&id, (&def.id, keys), ctx.law);
        let result = self
            .rules(m, def, (keys, env, ctx), node)
            .and_then(|v| coerce_def(m, def, v));
        self.close_cell(id, (node, outer), ctx, result)
    }

    /// The start of def `def`'s cell at its keys: the memo's result, a cell read while it is
    /// computed, or a cell past the counted depth, waiting on the work stack, as the read's
    /// result; else the cell opened and counted (refcore's `def_cell`). E §3.2.
    fn open_cell(
        &mut self,
        def: &DefInfo,
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<Result<CellId, Result<Value, Stop>>, Stop> {
        let key = CellKey::new(&def.id, keys, ctx);
        // under a trace the cone is evaluated again: the memo gives only a cell the trace
        // has met, which is referenced by its node (E §10)
        let seen = self.trace_seen(&key, ctx.law);
        if let Some((cell, class)) = self
            .cells
            .get(&key, ctx.law)
            .filter(|_| seen != Seen::Unmet)
        {
            let result = cell.result.clone();
            let met = cell.met.clone();
            self.absorb(&met);
            self.narrow(ctx.law, class);
            self.read(Read::Cell(CellId::new(key, ctx.law)));
            if let Seen::Met(at) = seen {
                self.trace_again(at);
            }
            #[cfg(feature = "counters")]
            self.counts.add(crate::Unit::DefHit);
            return Ok(Err(result));
        }
        let id = CellId::new(key, ctx.law);
        if self.active.contains(&id) {
            self.narrow_open_to_days();
            self.read(Read::Cell(id.clone()));
            if self.guarded.as_ref().is_some_and(|g| g.contains(&id)) {
                // the walk a leg's read started inside this cell needs it: that walk's
                // attempt is given up (refcore's `outer_hit`)
                return Ok(Err(Err(Stop::NotBuilt(super::GUARDED))));
            }
            return Ok(Err(Err(refuse(
                Run::Cycle,
                format!(
                    "{} reads itself through a computed key",
                    origin(&def.id, keys)
                ),
            ))));
        }
        if self.depth >= DEPTH_LIMIT {
            self.work.push(Task {
                def: def.id.clone(),
                keys: keys.to_vec(),
                ctx,
                unwound: Vec::new(),
            });
            return Ok(Err(Err(Stop::Deep)));
        }
        self.active.insert(id.clone());
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or_else(|| invariant("the depth count"))?;
        self.push();
        self.open_class(ctx.law);
        self.open_reads();
        Ok(Ok(id))
    }

    /// The end of cell `id`, its rules' `result` computed under trace node `node`, the
    /// holder `outer` restored: uncounted, and kept in the memo with the register entries
    /// and reads it met, unless it waits on the work stack or a walk's attempt was given up
    /// (refcore's `def_cell`). E §3.2.
    fn close_cell(
        &mut self,
        id: CellId,
        (node, outer): (Option<usize>, Option<Node>),
        ctx: Ctx,
        result: Result<Value, Stop>,
    ) -> Result<Value, Stop> {
        self.trace_close(node);
        self.trace_cell(&id, node, &result);
        self.holder = outer;
        let reads = self.close_reads();
        let class = self.close_class();
        let met = self.pop_alone();
        self.depth = self
            .depth
            .checked_sub(1)
            .ok_or_else(|| invariant("the depth count"))?;
        let class = class?;
        let reads = reads?;
        if result == Err(Stop::Deep) {
            self.suspend(id)?;
            #[cfg(feature = "counters")]
            self.counts.add(crate::Unit::DefUnwound);
            return result;
        }
        self.active.remove(&id);
        if result == Err(Stop::NotBuilt(super::GUARDED)) {
            // a walk's attempt given up: nothing it reached is kept (refcore's journal)
            return result;
        }
        self.absorb(&met);
        #[cfg(feature = "counters")]
        self.counts.miss(id.key.def());
        // a walk-bound cell is kept at its own point and law date (E §3.2)
        let class = match (ctx.at, ctx.law) {
            (Some(_), Some(day)) => LawClass::day(day),
            _ => class,
        };
        let cell = Cell {
            result: result.clone(),
            met,
            reads,
        };
        let read = Read::Cell(id.clone());
        let kept = self.cells.keep(id.key, ctx.law, class, cell)?;
        self.narrow(ctx.law, kept);
        self.read(read);
        result
    }

    /// A def's value at its keys: a given that governs, else the rule that wins (refcore's
    /// `eval_rules`); under a trace, each rule's state on the def cell's node `node` (E §10).
    /// L §5.11 steps 1–5.
    fn rules(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        (keys, env, ctx): (&[KeyVal], &Env, Ctx),
        node: Option<usize>,
    ) -> Result<Value, Stop> {
        // the rule is chosen in its own frame, gone before its value reads on (R-16)
        let rule = match self.winning(m, def, (keys, env, ctx), node)? {
            Err(decided) => return Ok(decided),
            Ok(win) => version_of(def, win)?,
        };
        match &rule.rule.value {
            RuleValue::Hole(text) => Ok(Value::Hole(Hole::asked(text, &origin(&def.id, keys)))),
            RuleValue::Expr(e) => {
                let group = self.trace_group();
                let value = self.ev(m, e, env, ctx);
                self.trace_close(group);
                self.trace_groups(node, &rule.label, (None, group));
                at_cell(m, (def, keys), value)
            }
            RuleValue::Solve(_) => Err(Stop::NotBuilt("`solve`")),
        }
    }

    /// The rule of def `def` that wins at its keys (L §5.11 steps 1–4), with its register
    /// entries; else the value already decided: a given that governs, or a hole `when` that
    /// keeps the def a hole (refcore's `eval_rules`). L §5.11.
    fn winning(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        (keys, env, ctx): (&[KeyVal], &Env, Ctx),
        node: Option<usize>,
    ) -> Result<Result<InForce, Value>, Stop> {
        let figure = origin(&def.id, keys);
        if let Some(given) = self.given(m, def, keys, ctx, node)? {
            return Ok(Err(given));
        }
        if let Some(day) = ctx.law {
            let class = rules_class(m, def, ctx.cut, day)?;
            self.narrow(ctx.law, class);
        }
        let live = in_force(m, def, self.clocks(ctx, &def.id)?)?;
        let order = beats_order(def, &live)?;
        let mut applicable: Vec<InForce> = Vec::new();
        let mut unknown: Vec<(InForce, Hole)> = Vec::new();
        let mut missing_true: Vec<InForce> = Vec::new();
        for at in order {
            let rule = version_of(def, at)?;
            if beaten_by_any(def, &applicable, at) {
                self.trace_beaten(def, node, at, &applicable);
                continue;
            }
            let Some(when) = &rule.rule.when else {
                applicable.push(at);
                continue;
            };
            self.push();
            let group = self.trace_group();
            let w = self.ev(m, when, env, ctx);
            self.trace_close(group);
            let met = self.pop_alone();
            self.absorb(&met);
            if met.missing {
                missing_true.push(at);
            }
            let tested = w?;
            self.trace_tested(node, (&rule.label, group), &tested);
            if let Value::Hole(h) = tested {
                unknown.push((at, h));
                continue;
            }
            if tested == Value::None {
                return Err(refuse(
                    Run::None,
                    format!("the when of {}.{} is none here", def.id, rule.label),
                ));
            }
            let Value::Bool(holds) = tested else {
                return Err(refuse_load(
                    Load::Unit,
                    format!("the when of {}.{} is not bool", def.id, rule.label),
                ));
            };
            if holds {
                applicable.push(at);
            }
        }
        let cands: Vec<InForce> = applicable
            .iter()
            .copied()
            .filter(|l| !beaten_by_any(def, &applicable, *l))
            .collect();
        let law_text = ctx.law.map_or_else(|| "no law date".to_owned(), Day::text);
        if let [first, second, ..] = cands.as_slice() {
            self.narrow_to_day(ctx.law);
            return Err(refuse(
                Run::Conflict,
                format!(
                    "rules {} and {} both win for {figure} on {law_text}",
                    version_of(def, *first)?.label,
                    version_of(def, *second)?.label
                ),
            ));
        }
        self.trace_candidates(def, node, &applicable, &cands);
        let mut relevant: Vec<Option<&Hole>> = Vec::new();
        for (at, hole) in &unknown {
            let beats_cand = cands.iter().any(|c| beats(def, *at, *c));
            let beaten_by_cand = cands.iter().any(|c| beats(def, *c, *at));
            if beats_cand || !beaten_by_cand || cands.is_empty() {
                relevant.push(Some(hole));
            }
        }
        if let Some(hole) = spread(&relevant) {
            if let Some(first) = cands.first() {
                self.trace_state(node, label_of(def, *first), "candidate".to_owned());
            }
            return Ok(Err(Value::Hole(hole)));
        }
        let Some(win) = cands.first().copied() else {
            self.narrow_to_day(ctx.law);
            return Err(refuse(
                Run::NoRule,
                format!("no rule applies to {figure} on {law_text}"),
            ));
        };
        let rule = version_of(def, win)?;
        self.trace_won(m, node, rule);
        self.rule_entries(m, def, rule, &missing_true, win)?;
        Ok(Ok(win))
    }

    /// Under a trace, rule `at`'s state on the def cell's node `node`: beaten, by the rules
    /// among `applicable` that beat it (refcore's `eval_rules`, its states). L §5.18.
    fn trace_beaten(
        &mut self,
        def: &DefInfo,
        node: Option<usize>,
        at: InForce,
        applicable: &[InForce],
    ) {
        if node.is_some() {
            self.trace_state(node, label_of(def, at), beaten_text(def, applicable, at));
        }
    }

    /// Under a trace, the state the `when` of the rule labelled `label` gives it on the def
    /// cell's node `node`, `tested` its value: `unknown` for a hole, `not applicable` for
    /// false, none yet for true; and the group the `when` read (refcore's `eval_rules`).
    /// L §5.18.
    fn trace_tested(
        &mut self,
        node: Option<usize>,
        (label, group): (&str, Option<usize>),
        tested: &Value,
    ) {
        if node.is_none() {
            return;
        }
        if let Value::Hole(_) = tested {
            self.trace_state(node, label, "unknown".to_owned());
        } else if *tested == Value::Bool(false) {
            self.trace_state(node, label, "not applicable".to_owned());
        }
        self.trace_groups(node, label, (group, None));
    }

    /// Under a trace, the candidates `cands` of the def cell's node `node` are known: each
    /// rule of `applicable` that isn't one is beaten, and the rules show (refcore's
    /// `eval_rules`, where it sets `rule_states`). L §5.18.
    fn trace_candidates(
        &mut self,
        def: &DefInfo,
        node: Option<usize>,
        applicable: &[InForce],
        cands: &[InForce],
    ) {
        if node.is_none() {
            return;
        }
        for at in applicable.iter().filter(|at| !cands.contains(at)) {
            self.trace_beaten(def, node, *at, applicable);
        }
        self.trace_shown(node);
    }

    /// Under a trace, rule `rule` won on the def cell's node `node`, with the version that
    /// states it and its source (refcore's `explain_node`). L §5.18, E §10.
    fn trace_won(&mut self, m: &Reality, node: Option<usize>, rule: &RuleVer) {
        if node.is_none() {
            return;
        }
        match won_text(m, rule) {
            Ok(state) => self.trace_state(node, &rule.label, state),
            Err(stop) => self.trace_kept(stop),
        }
    }

    /// The register entries of the rule that wins: its versions' `assumed`, and `missing` for
    /// a cited rule whose `when` read a `missing(…)` that held (L §8.2).
    fn rule_entries(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        rule: &RuleVer,
        missing_held: &[InForce],
        win: InForce,
    ) -> Result<(), Stop> {
        for version in [&rule.v, &rule.version].into_iter().flatten() {
            let info = m
                .versions
                .get(version)
                .ok_or_else(|| invariant("a rule names a version load didn't build"))?;
            if let Some(reason) = &info.assumed {
                self.note(
                    "assumed",
                    reason.clone(),
                    format!("{}.{}", def.id, rule.label),
                );
            }
        }
        if missing_held.contains(&win)
            && let Some(v) = &rule.v
        {
            self.note("missing", v.clone(), String::new());
        }
        Ok(())
    }

    /// The `given` row that governs the def at its keys, as a value with its `given` entry
    /// (L §5.11 step 1); under a trace, the row and a `given` line on the def cell's node
    /// `node` (E §10).
    fn given(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        keys: &[KeyVal],
        ctx: Ctx,
        node: Option<usize>,
    ) -> Result<Option<Value>, Stop> {
        if def.givens.is_empty() {
            return Ok(None);
        }
        self.read(Read::Givens {
            def: def.id.clone(),
            keys: keys.to_vec(),
        });
        if let Some(day) = ctx.law {
            let class = rows_class(m, &def.givens, ctx.cut, day)?;
            self.narrow(ctx.law, class);
        }
        let clocks = self.clocks(ctx, &def.id)?;
        let binds = (def.binds.as_slice(), keys);
        let cands = candidates(m, &def.givens, binds, Test::At(ctx.law), clocks)?;
        let cands = worded(m, &def.givens, cands, clocks)?;
        let Some(at) = choose(m, &def.givens, &def.binds, &cands, ctx.cut, &def.id)? else {
            return Ok(None);
        };
        let row = def
            .givens
            .get(at)
            .ok_or_else(|| invariant("the read rule took a given the def lacks"))?;
        let figure = origin(&def.id, keys);
        self.note_version(&row.version);
        self.note_row(&row.sid, &row.version, row.locator.as_ref());
        if m.scenarios.of(&row.version).is_some() {
            self.note_scenario(&row.version, &def.id);
        }
        self.read(Read::Row {
            version: row.version.clone(),
            sid: row.sid.clone(),
        });
        let traced = self.trace_row(m, row);
        let of = ConvertFor::row(&def.unit_factor, &def.ty, &def.binds, &row.pattern);
        let value = if let crate::syntax::Value::Expr(e) = &row.value {
            // a scenario's expression row, at the def's law date (L §6.4)
            self.row_expr(m, (row, e), &figure, ctx)?
        } else {
            match convert(m, &row.value, &def.ty, &def.id, &of)? {
                Converted::Hole(text) => Value::Hole(Hole::asked(&text, &figure)),
                Converted::Value(v) => v,
            }
        };
        let shown = shown(m, &value)?;
        let keys_text = pattern_text(&row.pattern);
        self.note(
            "given",
            format!("{}{keys_text} = {shown} ({})", def.id, row.version),
            String::new(),
        );
        if traced.is_some() {
            self.trace_value(traced, &Ok(value.clone()));
            let line = format!("given {}{keys_text} (version {})", def.id, row.version);
            self.trace_line(node, line);
        }
        Ok(Some(value))
    }
}

/// A rule in force: its label's position among the def's rules and its version's position
/// among the label's versions (L §5.11 step 2).
type InForce = (usize, usize);

/// The rules in force at the law date and cut of `clocks`, under its `wording` pin of the def
/// if one names it, in its world, in the def's order of labels (L §5.11 step 2, §4.3, §6.4).
fn in_force(m: &Reality, def: &DefInfo, clocks: Clocks) -> Result<Vec<InForce>, Stop> {
    let mut live = Vec::new();
    for (label_at, (label, versions)) in def.rules.iter().enumerate() {
        if let Some(at) = rule_in_force(m, &def.id, label, versions, clocks)? {
            live.push((label_at, at));
        }
    }
    Ok(live)
}

/// `beaten by a, b`: the rules among `applicable` that beat rule `at`, by label (refcore's
/// `eval_rules`, its states). L §5.18.
fn beaten_text(def: &DefInfo, applicable: &[InForce], at: InForce) -> String {
    // the labels in their text's order, each once
    let by: std::collections::BTreeSet<&str> = applicable
        .iter()
        .filter(|o| **o != at && beats(def, **o, at))
        .map(|o| label_of(def, *o))
        .collect();
    format!(
        "beaten by {}",
        by.into_iter().collect::<Vec<_>>().join(", ")
    )
}

/// `won`, with the version that states the rule and its source where it cites one: its `v`,
/// else the version holding it (refcore's `explain_node`, `rv.v or rv.version`). E §10.
fn won_text(m: &Reality, rule: &RuleVer) -> Result<String, Stop> {
    let Some(version) = rule.v.as_ref().or(rule.version.as_ref()) else {
        return Ok("won".to_owned());
    };
    let cite = &m
        .versions
        .get(version)
        .ok_or_else(|| invariant("a rule names a version load didn't build"))?
        .cite;
    Ok(format!("won (version {version}, source {cite})"))
}

/// The version of a rule in force.
fn version_of(def: &DefInfo, entry: InForce) -> Result<&RuleVer, Stop> {
    def.rules
        .get(entry.0)
        .and_then(|(_, versions)| versions.get(entry.1))
        .ok_or_else(|| invariant("a rule version the def lacks"))
}

/// The label of a rule in force.
fn label_of(def: &DefInfo, entry: InForce) -> &str {
    def.rules
        .get(entry.0)
        .map_or("", |(label, _)| label.as_str())
}

/// Whether rule `a` beats rule `b` (L §5.11 `beats`).
fn beats(def: &DefInfo, a: InForce, b: InForce) -> bool {
    let target = label_of(def, b);
    let Ok(version) = version_of(def, a) else {
        return false;
    };
    version.rule.beats.iter().any(|x| x == target)
}

/// Whether a rule other than `x` among `rules` beats `x`.
fn beaten_by_any(def: &DefInfo, rules: &[InForce], x: InForce) -> bool {
    rules.iter().any(|o| *o != x && beats(def, *o, x))
}

/// The rules in force in the order their `when`s are tried: each once no rule still left
/// beats it, ties in the text order of their labels (refcore's `beats_order`). L §5.11 step 3.
fn beats_order(def: &DefInfo, live: &[InForce]) -> Result<Vec<InForce>, Stop> {
    let mut left: Vec<InForce> = live.to_vec();
    let mut done = Vec::with_capacity(left.len());
    while !left.is_empty() {
        let mut ready: Vec<InForce> = Vec::new();
        for x in &left {
            if !beaten_by_any(def, &left, *x) {
                ready.push(*x);
            }
        }
        if ready.is_empty() {
            return Err(refuse_load(
                Load::Cycle,
                format!("a beats cycle in {}", def.id),
            ));
        }
        ready.sort_by(|a, b| label_of(def, *a).cmp(label_of(def, *b)));
        left.retain(|x| !ready.contains(x));
        done.extend(ready);
    }
    Ok(done)
}

/// A def cell's value at its `keys`: a number whose unit names a key's variable, a figure in
/// a per-unit price's declared type `num CUR/u` or one typed like a branch read at the key,
/// is in CUR per the key's unit (L §3.2 Per-unit prices of a unit kind, Inline figures).
/// Refcore keeps `$u` (its `_flexto` and `typed`).
/// The rule's value `value` is first a whole value in the def's type ([`whole_value`]). Its own
/// function, so the def's recursive evaluation keeps its frame (R-16).
fn at_cell(
    m: &Reality,
    (def, keys): (&DefInfo, &[KeyVal]),
    value: Result<Value, Stop>,
) -> Result<Value, Stop> {
    let v = whole_value(m, &def.ty, value?)?;
    if let Value::Number(n) = v {
        return Ok(Value::Number(at_keys(n, &def.binds, keys)?));
    }
    Ok(v)
}

/// A whole value takes its target's type: a bare figure becomes the def's number type, `1`
/// in a `num %` def being 1 % (L §3.2 Inline figures; refcore's `_flexto` and `typed`).
fn whole_value(m: &Reality, ty: &Ty, v: Value) -> Result<Value, Stop> {
    let Value::Number(Number {
        q,
        class: Class::Flex,
    }) = v
    else {
        return Ok(v);
    };
    if *ty == Ty::Rate {
        return Err(refuse_load(
            Load::Unit,
            "a bare figure as a rate: a rate states its period (L §3.3)".to_owned(),
        ));
    }
    if let Some(typed_value) = super::expr::typed(m, &q, ty)? {
        return Ok(typed_value);
    }
    let class = if q.is_integer() {
        Class::Int
    } else {
        Class::Num(Unit::one(), Shown::Plain)
    };
    Ok(Value::Number(Number { q, class }))
}

/// A def's value in its declared type (refcore's `coerce_def`): an int that is one, money
/// from a bare 0, a key of its index. L §5.11, §3.2.
fn coerce_def(m: &Reality, def: &DefInfo, v: Value) -> Result<Value, Stop> {
    if matches!(v, Value::Hole(_) | Value::None | Value::Never) {
        return Ok(v);
    }
    match (&def.ty, v) {
        (Ty::Int, Value::Number(n)) => {
            if !n.q.is_integer() {
                return Err(refuse(
                    Run::NotInt,
                    format!(
                        "{} is not an integer",
                        Value::Number(n).text(m).map_err(num_stop)?
                    ),
                ));
            }
            Ok(Value::Number(Number {
                q: n.q,
                class: Class::Int,
            }))
        }
        (Ty::Num(_, shown), Value::Number(n)) if matches!(n.class, Class::Int) => {
            Ok(Value::Number(Number {
                q: n.q,
                class: Class::Num(Unit::one(), *shown),
            }))
        }
        (Ty::Num(..), Value::Money(money)) => {
            let q = crate::model::money_num(m, &money).map_err(num_stop)?;
            let unit = crate::model::currency_unit(m, money.currency()).map_err(num_stop)?;
            Ok(Value::Number(Number::plain(q, unit)))
        }
        (Ty::Money(cur), Value::Number(n)) => {
            if n.q.is_zero() && matches!(n.class, Class::Int | Class::Flex) {
                let id = super::arith::currency_of(m, cur)?;
                return Ok(Value::Money(crate::num::Money::from_minor(0, id)));
            }
            Err(refuse_load(
                Load::Unit,
                format!("a num stored as money in {}", def.id),
            ))
        }
        (Ty::Key(ix), Value::Word(word)) => Ok(index_word(m, ix, word)),
        (_, v) => Ok(v),
    }
}

/// Key literal `word` as a key of index `ix` where `ix` holds it; else the literal as it
/// stands, which a kind's or an axis's position resolves (L §2.5).
fn index_word(m: &Reality, ix: &str, word: String) -> Value {
    let key = KeyVal::Key {
        index: ix.to_owned(),
        name: word.clone(),
    };
    if m.indexes
        .get(ix)
        .is_some_and(|index| index.pos(&key).is_some())
    {
        return Value::of_key(&key);
    }
    Value::Word(word)
}

/// A def's scope at its keys: each key variable bound to its key, with its domain (refcore's
/// `read_def`, the env it builds). L §5.11.
pub(super) fn key_env(def: &DefInfo, keys: &[KeyVal]) -> Env {
    let mut env = Env::empty();
    for (bind, key) in def.binds.iter().zip(keys) {
        let written = std::rc::Rc::from(bind.node.text.as_str());
        env.bind_written(
            (&bind.var, &written),
            Value::of_key(key),
            Some(bind.domain.clone()),
        );
    }
    env
}

/// The refusal of an `at` whose value is no day (refcore's `law_value`). L §4.3.
fn bad_law_date(m: &Reality, def: &DefInfo, v: &Value) -> Result<Stop, Stop> {
    if *v == Value::None {
        return Ok(refuse(
            Run::None,
            format!("the law date of {} is none here", def.id),
        ));
    }
    if *v == Value::Never {
        return Ok(refuse(
            Run::Domain,
            format!("the law date of {} is never", def.id),
        ));
    }
    if let Value::Period(p) = v {
        return Ok(refuse_load(
            Load::LawPeriod,
            format!("at {}: a period is not a law date", p.label()),
        ));
    }
    Ok(refuse_load(
        Load::Unit,
        format!(
            "the law date of {} is {}",
            def.id,
            v.text(m).map_err(num_stop)?
        ),
    ))
}
