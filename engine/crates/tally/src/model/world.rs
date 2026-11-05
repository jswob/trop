//! A reality's scenarios: each one its files declare, the world it is over, and the sections its
//! file holds, so an ask's world can be told from the base's.
//! Spec: L §6.4 (scenarios: `scenario s over b`, a stack of scenarios over the model), §6.1 (a
//! model is its files).
//! Twin: engine/tools/refcore/loader.py (`Loader.build`'s pass 1: `scenario_over`,
//! `tag_scenario`).
//! Invariants: a reality loaded alone, or a base, holds no scenario; a scenario's reality holds
//! its stack, each scenario over the model or over another of the stack.

use std::collections::BTreeMap;

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
}
