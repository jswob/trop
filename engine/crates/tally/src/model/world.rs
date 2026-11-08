//! A reality's scenarios: each one its files declare, the world it is over, and the sections its
//! file holds, so an ask's world can be told from the base's.
//! Spec: L §6.4 (scenarios: `scenario s over b`, a stack of scenarios over the model), §6.1 (a
//! model is its files).
//! Twin: engine/tools/refcore/loader.py (`Loader.build`'s pass 1: `scenario_over`,
//! `tag_scenario`).
//! Invariants: a reality loaded alone, or a base, holds no scenario; a scenario's reality holds
//! its stack, each scenario over the model or over another of the stack.

use std::collections::{BTreeMap, BTreeSet};

/// The scenarios a reality loads (L §6.4), empty for a base.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scenarios {
    /// Each scenario's id, and the scenario it is over, or `None` for the model.
    pub over: BTreeMap<String, Option<String>>,
    /// The id of each section a scenario's file declares, and that scenario.
    pub section: BTreeMap<String, String>,
    /// Each scenario's `from`: its rows of the base's items govern that day or later (L §6.4).
    pub from: BTreeMap<String, crate::time::Day>,
}

impl Scenarios {
    /// The scenario whose file declares the section `id`, or `None` for a base section. L §6.4.
    #[must_use]
    pub fn of(&self, id: &str) -> Option<&str> {
        self.section.get(id).map(String::as_str)
    }

    /// The layer of the section `id` (a version's, a row's): 0 for the base's, or for what no
    /// version states (a thing's own entry), and one more for each scenario of the stack from
    /// the base up to the one whose file declares it, so a scenario's rows stand above its
    /// base's (refcore's `version_layer`, `row_layer`). L §6.4, §4.6 steps 2 and 3.1.
    #[must_use]
    pub fn layer(&self, id: Option<&str>) -> usize {
        let mut at = id.and_then(|id| self.of(id));
        for depth in 0..=self.over.len() {
            let Some(scenario) = at else {
                return depth;
            };
            at = self.over.get(scenario).and_then(Option::as_deref);
        }
        self.over.len()
    }

    /// The scenario whose world this reality is: the top of its stack, the one no other of the
    /// stack is over; `None` for a base. L §6.4.
    #[must_use]
    pub fn own(&self) -> Option<&str> {
        let under: BTreeSet<&str> = self.over.values().filter_map(Option::as_deref).collect();
        self.over
            .keys()
            .map(String::as_str)
            .find(|id| !under.contains(id))
    }
}
