//! A reality's scenarios: each one its files declare, the world it is over, and the sections its
//! file holds, so an ask's world can be told from the base's.
//! Spec: L §6.4 (scenarios: `scenario s over b`, a stack of scenarios over the model), §6.1 (a
//! model is its files).
//! Twin: engine/tools/refcore/loader.py (`Loader.build`'s pass 1: `scenario_over`,
//! `tag_scenario`).
//! Invariants: a reality loaded alone, or a base, holds no scenario; a scenario's reality holds
//! its stack, each scenario over the model or over another of the stack; a world is the layers
//! of that stack a read sees, the lowest ones or the whole, so a world below the top hides the
//! sections of the scenarios over it (refcore's `world_chain`, `in_world`).

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

    /// The world `id` names in this reality: the model's (`model`, its own id), a scenario of
    /// its stack, or none for a world the stack doesn't hold (another stack's scenario). The
    /// world of the top of the stack is the reality's whole. L §6.4 (`in s`, `as if {scenario
    /// s}`, `over s0`).
    #[must_use]
    pub fn world(&self, id: &str, model: &str) -> Option<World> {
        let depth = if id == model {
            0
        } else {
            let mut depth = 0;
            let mut at = Some(id);
            while let Some(scenario) = at {
                if depth > self.over.len() {
                    return None;
                }
                at = self.over.get(scenario)?.as_deref();
                depth = depth.checked_add(1)?;
            }
            depth
        };
        if depth >= self.over.len() {
            return Some(World::Whole);
        }
        Some(World::Lowest(depth))
    }

    /// The model's world in this reality: the base alone, its whole for a reality with no
    /// scenario. L §6.4 (`in household`, the model's id).
    #[must_use]
    pub fn model_world(&self) -> World {
        if self.over.is_empty() {
            World::Whole
        } else {
            World::Lowest(0)
        }
    }

    /// Whether a read in world `world` sees the section `id` (a version's, an item's, a row's
    /// version): the base's always, a scenario's in that scenario and those over it (refcore's
    /// `in_world`, `item_in_world`). L §6.4.
    #[must_use]
    pub fn sees(&self, world: World, id: Option<&str>) -> bool {
        match world {
            World::Whole => true,
            World::Lowest(depth) => self.layer(id) <= depth,
        }
    }

    /// The scenario world `world` is, its id: none for the model's (L §6.4).
    #[must_use]
    pub fn scenario_of(&self, world: World) -> Option<&str> {
        let depth = match world {
            World::Whole => self.over.len(),
            World::Lowest(depth) => depth,
        };
        let mut at = self.own();
        let mut level = self.over.len();
        while let Some(scenario) = at {
            if level == depth {
                return Some(scenario);
            }
            at = self.over.get(scenario).and_then(Option::as_deref);
            level = level.checked_sub(1)?;
        }
        None
    }
}

/// A world of a reality: the layers of its stack of scenarios a read sees (L §6.4: "worlds
/// below it are evaluated in the same reality with the higher layers hidden", refcore's
/// `world_chain`). A reality with no scenario has one world, its whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum World {
    /// Every layer: the reality's own world, the top of its stack.
    Whole,
    /// The model and the lowest scenarios of the stack, this many; 0 is the model's world.
    Lowest(usize),
}
