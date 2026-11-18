//! Load's checker, its defs: a def's scope, its rules' `when` and value, their labels and `beats`, and the key offsets of its arithmetic.
//! Spec: L §5.11 (a def's rules: `when` a bool, the value of the def's type, labels, `beats`
//! naming a rule of the def and never in a cycle, a rule's guards), §5.9 (a band row's
//! value), §4.3 (what an `at` reads), §7.1 (a key variable's integer offsets), §8.3
//! (`LOAD-UNIT`, `-LABEL`, `-CYCLE`).
//! Twin: engine/tools/refcore/check.py (`def_scope`, `check_def` with its labels, `at_reads`,
//! `when_vars`, `guard` and `item_offsets`, `_beats_cycle`, `tested_vars`); check3.py
//! (`m_check_move`'s `when`).
//! Invariants: a def's parts are checked in refcore's order, each refusal at the line its
//! check names; a `beats` cycle is named from the first label in text order, its walk a
//! worklist, never a recursion.

use super::check::{Checker, Offsets, Scope, Stands, law_date};
use super::random::dist_whole;
use super::steps::refuse;
use super::texts::{children, covered_spans};
use super::types::{is_bool, refused, uerr};
use crate::base::{Code, Load, Place as At, Refusal};
use crate::model::{DefInfo, Node, NodeKind, Reality, Rel, Ty};
use crate::syntax::{Expr, LawDate, Lit, RuleValue, Stop};
use std::collections::{BTreeMap, BTreeSet};

impl Checker {
    /// A def's scope at `line` (refcore's `def_scope`).
    pub(super) fn def_scope(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        stands: Stands,
        at: (&str, u32),
    ) -> Result<Scope, Stop> {
        let scope = Scope::new(Node::new(NodeKind::Def, &def.id), stands, at.0, at.1);
        self.keyed(m, scope, &def.binds)
    }

    /// A def: its key binds' `where`, its `at`, its rules' labels and `beats`, then each
    /// rule's `when` (a bool) and value (of the def's type) (refcore's `check_def`).
    /// L §5.11, §4.3, §2.3.
    pub(super) fn check_def(&mut self, m: &Reality, def: &DefInfo, file: &str) -> Result<(), Stop> {
        let line = def.decl.line;
        let at_scope = self.def_scope(m, def, Stands::At, (file, line))?;
        for bind in &def.binds {
            if let Some(filter) = &bind.node.filter {
                let scope = self.def_scope(m, def, Stands::Rule, (file, line))?;
                let t = self.expr(m, &scope, filter, None, 0)?;
                if !is_bool(&t) {
                    return Err(scope.place(uerr(format!(
                        "where of a key bind of {} is {}, not bool",
                        def.id,
                        t.show()
                    ))));
                }
            }
        }
        let node = Node::new(NodeKind::Def, &def.id);
        let before = self
            .graph
            .edges
            .get(&node)
            .map_or_else(BTreeMap::new, Clone::clone);
        if let Some(LawDate::Expr(at)) = &def.at {
            let t = self.expr(m, &at_scope, at, None, 0)?;
            law_date(m, &t, at, &format!("at of {}", def.id)).map_err(|s| at_scope.place(s))?;
        }
        if def.at.is_some() {
            let made = new_reads(&before, self.graph.edges.get(&node));
            self.at_reads.insert(def.id.clone(), made);
        }
        let beats = labels(def, file)?;
        beats_cycle(&def.id, &beats, (file, line))?;
        self.offsets.insert(def.id.clone(), key_offsets(def));
        let guards = rule_guards(def, &beats);
        for (label, versions) in &def.rules {
            for version in versions {
                let rule = &version.rule;
                let mut scope = self.def_scope(m, def, Stands::Rule, (file, rule.line))?;
                scope.cited = version.v.is_some();
                scope.guards = guards.get(label).map_or_else(BTreeSet::new, Clone::clone);
                covered_spans(rule.when.as_ref(), &mut scope.covered);
                // a kind test of the `when` narrows throughout its value and its when (L §5.7)
                let scope = scope.narrowed(m, rule.when.as_ref());
                if let Some(when) = &rule.when {
                    self.when(m, &scope, &scope, when, &format!("when of rule {label}"))?;
                }
                let what = format!("rule {label} of {}", def.id);
                self.rule_value(m, &scope, &rule.value, &def.ty, &what)?;
            }
        }
        Ok(())
    }

    /// A `when`: a bool, typed in `scope` as a `when` is (`missing()` allowed in a cited
    /// rule's), refused at `owner`'s line (refcore's `check_def`, `m_check_move`, their
    /// `when`). L §5.11, §5.14.
    pub(super) fn when(
        &mut self,
        m: &Reality,
        scope: &Scope,
        owner: &Scope,
        when: &Expr,
        what: &str,
    ) -> Result<(), Stop> {
        let mut in_when = scope.clone();
        in_when.in_when = true;
        let t = self.expr(m, &in_when, when, None, 0)?;
        if !is_bool(&t) {
            return Err(owner.place(uerr(format!("{what} is {}, not bool", t.show()))));
        }
        Ok(())
    }

    /// A rule's or a band row's value, of the type `ty` (refcore's `check_def`, its value
    /// arm; `check_band`). L §5.11, §5.9.
    pub(super) fn rule_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        value: &RuleValue,
        ty: &Ty,
        what: &str,
    ) -> Result<(), Stop> {
        match value {
            RuleValue::Hole(_) => Ok(()),
            RuleValue::Solve(solve) => {
                if !scope.cited {
                    let message = "solve outside an ask (allowed in a rule with v)".to_owned();
                    return Err(scope.place(refused(Load::AskOnly, message)));
                }
                self.check_solve(m, scope, solve, 0)
                    .map(|_| ())
                    .map_err(|stop| scope.place(stop))
            }
            RuleValue::Expr(e) => {
                if matches!(ty, Ty::Dist(_)) {
                    let t = self.expr(m, &scope.dist_scope(), e, Some(ty), 0)?;
                    self.assign(m, scope, ty, e, &t, what)
                        .map_err(|stop| scope.place(stop))?;
                    return dist_whole(e, what).map_err(|stop| scope.place(stop));
                }
                let t = self.expr(m, scope, e, Some(ty), 0)?;
                self.assign(m, scope, ty, e, &t, what)
                    .map_err(|stop| scope.place(stop))
            }
        }
    }
}

/// A def's rules' `beats`: each names a rule of the def, at its line; the labels each beats
/// (refcore's `check_def`, its labels). L §5.11.
fn labels(def: &DefInfo, file: &str) -> Result<BTreeMap<String, BTreeSet<String>>, Stop> {
    let names: BTreeSet<&str> = def.rules.iter().map(|(label, _)| label.as_str()).collect();
    let mut beats: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (label, versions) in &def.rules {
        let mut beaten = BTreeSet::new();
        for version in versions {
            for beat in &version.rule.beats {
                if !names.contains(beat.as_str()) {
                    return Err(Stop::Refused(Refusal::at(
                        Code::Load(Load::Label),
                        format!("beats names no rule {beat}"),
                        At::Line(file.to_owned(), version.rule.line),
                    )));
                }
                beaten.insert(beat.clone());
            }
        }
        if !beaten.is_empty() {
            beats.insert(label.clone(), beaten);
        }
    }
    Ok(beats)
}

/// `LOAD-CYCLE` when a def's `beats` run in a cycle, the path from the first label in text
/// order, at the def's line (refcore's `_beats_cycle`, its depth-first walk as a worklist).
/// L §5.11, §8.3.
fn beats_cycle(
    def: &str,
    beats: &BTreeMap<String, BTreeSet<String>>,
    at: (&str, u32),
) -> Result<(), Stop> {
    let mut done: BTreeSet<String> = BTreeSet::new();
    for start in beats.keys() {
        if done.contains(start) {
            continue;
        }
        let mut path: Vec<(String, Vec<String>)> = vec![(start.clone(), beaten(beats, start))];
        while let Some((_, pending)) = path.last_mut() {
            let Some(next) = pending.pop() else {
                if let Some((finished, _)) = path.pop() {
                    done.insert(finished);
                }
                continue;
            };
            if path.iter().any(|(label, _)| *label == next) {
                let mut names: Vec<String> = path.into_iter().map(|(label, _)| label).collect();
                names.push(next);
                return Err(refuse(
                    Load::Cycle,
                    format!("a beats cycle in {def}: {}", names.join(" -> ")),
                    at.0,
                    at.1,
                ));
            }
            if !done.contains(&next) {
                let below = beaten(beats, &next);
                path.push((next, below));
            }
        }
    }
    Ok(())
}

/// The reads `after` holds that `before` didn't, each with its new relations: what an `at`
/// read (refcore's `check_def`, its `at_reads`). L §4.3.
fn new_reads(
    before: &BTreeMap<Node, Vec<Rel>>,
    after: Option<&BTreeMap<Node, Vec<Rel>>>,
) -> BTreeMap<Node, Vec<Rel>> {
    let mut out = BTreeMap::new();
    for (target, rels) in after.into_iter().flatten() {
        let old: &[Rel] = before.get(target).map_or(&[], Vec::as_slice);
        let added: Vec<Rel> = rels
            .iter()
            .filter(|rel| !old.contains(rel))
            .cloned()
            .collect();
        if !added.is_empty() {
            out.insert(target.clone(), added);
        }
    }
    out
}

/// Each rule's guards: the key variables its `when` tests, and those of the `when` of each
/// rule that beats it (refcore's `check_def`, its `when_vars` and `guard`). L §5.11.
fn rule_guards(
    def: &DefInfo,
    beats: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let keyvars: Vec<String> = def.binds.iter().map(|bind| bind.var.clone()).collect();
    let mut when_vars: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (label, versions) in &def.rules {
        let mut vars = BTreeSet::new();
        for version in versions {
            if let Some(when) = &version.rule.when {
                vars.extend(tested_vars(when, &keyvars));
            }
        }
        when_vars.insert(label, vars);
    }
    let mut out = BTreeMap::new();
    for (label, _) in &def.rules {
        let mut guard = when_vars
            .get(label.as_str())
            .map_or_else(BTreeSet::new, Clone::clone);
        for (beater, beaten) in beats {
            if beaten.contains(label)
                && let Some(vars) = when_vars.get(beater.as_str())
            {
                guard.extend(vars.iter().cloned());
            }
        }
        out.insert(label.clone(), guard);
    }
    out
}

/// The key variables a condition tests: each named in a side of a comparison or an `in`
/// (refcore's `tested_vars`). L §5.11.
pub(super) fn tested_vars(e: &Expr, keyvars: &[String]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![e];
    while let Some(next) = stack.pop() {
        let mut sides: Vec<&Expr> = Vec::new();
        if let Expr::Cmp(cmp) = next {
            sides.extend([&cmp.0, &cmp.2]);
        }
        if let Expr::In(test) = next {
            sides.extend([&test.member, &test.set]);
        }
        for side in sides {
            let mut below = vec![side];
            while let Some(x) = below.pop() {
                if let Expr::Name(name) = x
                    && keyvars.contains(name)
                {
                    out.insert(name.clone());
                }
                below.extend(children(x));
            }
        }
        stack.extend(children(next));
    }
    out
}

/// The labels `label` beats, last first, so that popping takes them in text order.
fn beaten(beats: &BTreeMap<String, BTreeSet<String>>, label: &str) -> Vec<String> {
    let Some(set) = beats.get(label) else {
        return Vec::new();
    };
    set.iter().rev().cloned().collect()
}

/// The integer offsets of a def's key arithmetic, `k + 1` or `k - 2` with `k` a key
/// variable (refcore's `check_def`, `item_offsets`). L §7.1.
fn key_offsets(def: &DefInfo) -> Offsets {
    let vars: BTreeSet<&str> = def.binds.iter().map(|bind| bind.var.as_str()).collect();
    let mut out = Offsets::new();
    let mut todo: Vec<&Expr> = Vec::new();
    for (_, versions) in &def.rules {
        for version in versions {
            todo.extend(version.rule.when.as_ref());
            if let RuleValue::Expr(e) = &version.rule.value {
                todo.push(e);
            }
        }
    }
    while let Some(e) = todo.pop() {
        if let Expr::Sum(first, rest) = e
            && let Expr::Name(var) = first.as_ref()
            && vars.contains(var.as_str())
            && let Some((_, Expr::Lit(Lit::Num(n)))) = rest.first()
            && n.whole
            && n.unit.is_none()
            && let Some(offset) = n.value.to_i64()
        {
            if let Some(set) = out.get_mut(var) {
                set.insert(offset);
            } else {
                out.insert(var.clone(), BTreeSet::from([offset]));
            }
        }
        todo.extend(children(e));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::beats_cycle;
    use crate::syntax::Stop;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn a_beats_cycle_is_named_from_the_first_label() {
        let set = |labels: &[&str]| -> BTreeSet<String> {
            labels.iter().map(|l| (*l).to_owned()).collect()
        };
        let mut beats = BTreeMap::new();
        beats.insert("base".to_owned(), set(&["waived"]));
        beats.insert("waived".to_owned(), set(&["base"]));
        let refused = match beats_cycle("fee", &beats, ("a.tally", 3)) {
            Err(Stop::Refused(refusal)) => refusal.text(),
            other => format!("{other:?}"),
        };
        assert_eq!(
            refused, "LOAD-CYCLE a beats cycle in fee: base -> waived -> base (a.tally:3)",
            "the cycle"
        );
        let mut chain = BTreeMap::new();
        chain.insert("a".to_owned(), set(&["b", "c"]));
        chain.insert("b".to_owned(), set(&["c"]));
        assert!(
            beats_cycle("fee", &chain, ("a.tally", 3)).is_ok(),
            "a chain is no cycle"
        );
    }
}
