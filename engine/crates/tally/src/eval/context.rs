//! The context's layers over its law date and cut, interned as one small id: its pins, its wording pins and its axis options.
//! Spec: E §3.2 (the context; trimmed to what the item can depend on: an item that no pin
//! reaches shares the unpinned cell, one that reaches no axis has no axis options; a pin set is
//! keyed by the pins' values), §3.3 (a timeline per pin set the moves reach); P §3.1 (the
//! context interned once as a small id, after it is trimmed), §3.2 (a pin set's layer, freed
//! when its `as if` or `solve` step ends), §3.5 (one memo per context, freed when its scope
//! ends); L §5.11 (nested pins don't leak to their siblings), §4.3 (the `wording` pins), §5.18
//! (a `solve` step pins its one cell), §6.4 (the world a scenario names), §5.5 (an axis read
//! at its option). The pins themselves are applied in `pins.rs`.
//! Twin: engine/tools/refcore/evaluate.py (`PinVal`, `Ctx.pins`, `Ctx.wording`, `Ctx.axes`,
//! `wording_ok`, `world_chain`, `axis_option`, `def_axes`).
//! Invariants: layer 0 is the unpinned context; two pin sets with the same targets and values
//! are one layer, whatever their texts; a def's cell is kept under its context trimmed to the
//! pins its def reaches, and to every pin the walk reaches where it reads the timeline; a
//! layer no open scope holds is freed with its cells and timelines when the scope that made it
//! ends, never while a read that unwound past the counted depth still needs it; a scope's
//! reads of the walk read the timeline of its world and walk pins (`timelines.rs`). The world
//! is the context's own field, never trimmed; an axis option is the layer's, trimmed as pins
//! are, and set for one combination of an ask's fan-out or by an axis pin.

use std::collections::{BTreeMap, BTreeSet};

use super::state::{Ctx, Engine, invariant, refuse};
use crate::base::Run;
use crate::model::{KeyVal, NameKind, Node, NodeKind, Reality, Value, World};
use crate::syntax::Stop;
use crate::time::Day;

/// A context's layer above its law date and cut: an interned set of pins (E §3.2, P §3.1).
/// It has no order of its own (R-47): a map keyed by layers is keyed by their numbers, the
/// order they were interned in, and no output is sorted by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer(u32);

impl Layer {
    /// The unpinned context.
    pub const BASE: Layer = Layer(0);

    /// Whether this is the unpinned context.
    #[must_use]
    pub fn is_base(self) -> bool {
        self == Layer::BASE
    }

    /// The layer's number, which names its world in a read's text (L §4.4).
    #[must_use]
    pub fn number(self) -> u32 {
        self.0
    }
}

/// One pin of a layer: a value an input's or a def's reads at matching keys take
/// (refcore's `PinVal`). L §5.11.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pinned {
    /// A `solve` step's pin of its variable, which the register doesn't list (L §5.18).
    pub(super) solve: bool,
    /// The input or def pinned.
    pub(super) item: String,
    /// Its keys by position, none at a `*`; empty for an item with no keys.
    pub(super) keys: Vec<Option<KeyVal>>,
    /// The record fields pinned, outermost first.
    pub(super) fields: Vec<String>,
    /// The value pinned.
    pub(super) value: Value,
    /// The pin as the register prints it.
    pub(super) text: String,
    /// Its scope: a pin written inside others is deeper, and wins (L §4.6's cascade; tally
    /// 2.1.7 R4).
    pub(super) layer: u32,
    /// The pin of its own set it ties with on the same cells, by its text: the cell is
    /// refused where it is read (`RUN-CONFLICT`, R4).
    pub(super) tied: Option<String>,
}

impl Pinned {
    /// Whether its keys are `other`'s or lie inside them at every position (a key inside
    /// `*`): it is at least as narrow (L §4.6's cascade).
    fn inside(&self, other: &Pinned) -> bool {
        self.keys
            .iter()
            .zip(&other.keys)
            .all(|(mine, theirs)| theirs.is_none() || mine == theirs)
    }

    /// Whether it lies inside each of `others`: the narrowest of them (L §4.6 step 4.1).
    fn inside_all(&self, others: &[&Pinned]) -> bool {
        others.iter().all(|other| self.inside(other))
    }

    /// Whether `other` pins the same cells: the same item, keys and fields (refcore's
    /// `key()[:4]`).
    fn same_target(&self, other: &Pinned) -> bool {
        self.solve == other.solve
            && self.item == other.item
            && self.keys == other.keys
            && self.fields == other.fields
    }

    /// Whether `other` is the same pin: the same target, value and scope, whatever its text
    /// (E §3.2: "keyed by the pins' values"; the scope decides which pin wins, L §4.6 step 2,
    /// refcore's `PinVal.key`).
    fn same(&self, other: &Pinned) -> bool {
        self.same_target(other) && self.value == other.value && self.layer == other.layer
    }

    /// Whether the pin holds at `keys` of its item: every key it names is the key read.
    fn holds_at(&self, item: &str, keys: &[KeyVal]) -> bool {
        self.item == item
            && (self.keys.is_empty() && keys.is_empty() || self.keys.len() == keys.len())
            && self
                .keys
                .iter()
                .zip(keys)
                .all(|(pinned, key)| pinned.is_none() || pinned.as_ref() == Some(key))
    }
}

/// What a layer holds: its pins, its wording pins and its axis options (E §3.2; refcore's
/// `Ctx.pins`, `Ctx.wording` and `Ctx.axes`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LayerData {
    /// The pins, in the order applied: a later pin of the same target replaces an earlier.
    pub(super) pins: Vec<Pinned>,
    /// Each item a `wording D for […]` pin names, with D; the first pin naming an item holds
    /// (refcore's `wording_ok`).
    pub(super) wording: Vec<(String, Day)>,
    /// The option each axis takes, by the axis's id: one combination of an ask's fan-out, or
    /// an axis pin (L §5.5).
    pub(super) axes: BTreeMap<String, String>,
}

impl LayerData {
    /// Whether it pins nothing and sets no axis: the unpinned context.
    fn is_empty(&self) -> bool {
        self.pins.is_empty() && self.wording.is_empty() && self.axes.is_empty()
    }

    /// Whether every pin, wording pin and axis option of this layer is one of `other`'s.
    fn within(&self, other: &LayerData) -> bool {
        self.pins.iter().all(|p| other.holds(p))
            && self.wording.iter().all(|w| other.wording.contains(w))
            && self
                .axes
                .iter()
                .all(|(axis, option)| other.axes.get(axis) == Some(option))
    }

    /// Whether `pin` is one of the layer's pins, by target and value.
    pub(super) fn holds(&self, pin: &Pinned) -> bool {
        self.pins.iter().any(|p| p.same(pin))
    }

    /// Whether the two layers hold the same pins and axis options, as sets.
    fn same(&self, other: &LayerData) -> bool {
        self.pins.len() == other.pins.len()
            && self.wording.len() == other.wording.len()
            && self.axes.len() == other.axes.len()
            && self.within(other)
    }

    /// The items its pins and wording pins name, each once.
    fn items(&self) -> BTreeSet<&str> {
        let mut out: BTreeSet<&str> = self.pins.iter().map(|p| p.item.as_str()).collect();
        out.extend(self.wording.iter().map(|(item, _)| item.as_str()));
        out
    }

    /// The layer with only the pins and wording pins on `items` and the options of `axes`
    /// (E §3.2: the context trimmed to what a reader can depend on).
    fn kept(&self, items: &BTreeSet<&str>, axes: &BTreeSet<String>) -> LayerData {
        LayerData {
            pins: self
                .pins
                .iter()
                .filter(|p| items.contains(p.item.as_str()))
                .cloned()
                .collect(),
            wording: self
                .wording
                .iter()
                .filter(|(item, _)| items.contains(item.as_str()))
                .cloned()
                .collect(),
            axes: self
                .axes
                .iter()
                .filter(|(axis, _)| axes.contains(*axis))
                .map(|(axis, option)| (axis.clone(), option.clone()))
                .collect(),
        }
    }

    /// Adds pin `pin` at its layer; a pin of its own set on the same cells ties with it, so
    /// the pin kept is refused where it is read (refcore's `apply_pins_`; tally 2.1.7 R4).
    pub(super) fn pin(&mut self, mut pin: Pinned) {
        pin.tied = self
            .pins
            .iter()
            .find(|p| p.layer == pin.layer && p.same_target(&pin))
            .map(|p| p.text.clone());
        self.pins
            .retain(|p| !(p.layer == pin.layer && p.same_target(&pin)));
        self.pins.push(pin);
    }

    /// The layer a set of pins applied over these takes: inside every pin in force.
    pub(super) fn next_layer(&self) -> Result<u32, Stop> {
        let Some(deepest) = self.pins.iter().map(|p| p.layer).max() else {
            return Ok(1);
        };
        deepest
            .checked_add(1)
            .ok_or_else(|| invariant("a pin's layer past the integers"))
    }

    /// Adds a `wording D for [ids]` pin: each item it names that no pin named before takes D
    /// (refcore's `wording_ok`: the first pin naming an item holds).
    pub(super) fn word(&mut self, ids: &[String], day: Day) {
        for id in ids {
            if self.wording_of(id).is_none() {
                self.wording.push((id.clone(), day));
            }
        }
    }

    /// The wording day of `item`, if a pin names it.
    fn wording_of(&self, item: &str) -> Option<Day> {
        self.wording
            .iter()
            .find(|(named, _)| named == item)
            .map(|(_, day)| *day)
    }
}

/// The engine's layers: each interned layer by its id, the trimmed layer of each def, the
/// layers the open scopes hold, and what the graph tells of the timeline (P §3.1, §3.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Layers {
    /// Each layer by its id; a freed one is empty. The base is slot 0.
    slots: Vec<Option<LayerData>>,
    /// Each def's trimmed layer of a layer, by the def's id and the layer's number (E §3.2).
    trimmed: BTreeMap<(String, u32), Layer>,
    /// The layers of the scopes open now: an ask's `with`, an `as if`, a `solve` step, a
    /// line's world.
    open: Vec<Layer>,
    /// The items the moves, places, prices, parts and fills reach, once computed.
    timeline: Option<BTreeSet<Node>>,
    /// Whether each def reads the timeline, once computed.
    walkers: BTreeMap<Node, bool>,
    /// The axes each def reaches on the graph, once computed (L §5.5, E §3.2).
    def_axes: BTreeMap<String, BTreeSet<String>>,
    /// The axes the walk's readers reach, once computed (E §3.3).
    walk_axes: Option<BTreeSet<String>>,
    /// Each layer's pins on items the walk reaches and its options of the axes the walk
    /// reaches, by the layer's number (E §3.3).
    walk_pins: BTreeMap<u32, Layer>,
    /// The layers of the scopes a read unwound through past the counted depth: their cells
    /// stay until the outermost read ends, since its replay and its read again need them
    /// (R-16).
    kept: Vec<Layer>,
}

impl Default for Layers {
    fn default() -> Layers {
        Layers {
            slots: vec![Some(LayerData::default())],
            trimmed: BTreeMap::new(),
            open: Vec::new(),
            timeline: None,
            walkers: BTreeMap::new(),
            def_axes: BTreeMap::new(),
            walk_axes: None,
            walk_pins: BTreeMap::new(),
            kept: Vec::new(),
        }
    }
}

/// The stop of an axis read where its context sets no option of it: outside the ask's line,
/// or in a walk whose timeline sets none, which then reads each combination's (L §5.5).
pub(crate) const UNSET_AXIS: &str = "an axis read outside the combinations its ask fans out over";

/// The stop of a walk that a leg's read started inside a cell's evaluation, and that needed a
/// cell still being computed outside it: the walk's attempt is given up there, nothing past
/// that point is kept, and the read takes the walk up to the leg's day instead (E §3.2,
/// evaluation is demand-driven; E §4.2 step 5; refcore's `guarded_walk`). Only the read that
/// started the walk meets it; a run that met it anywhere else would print not implemented.
pub(crate) const GUARDED: &str = "a walk given up for a cell still being computed outside it";

/// The node kinds the walk reads from: whatever they reach, the timeline depends on
/// (E §3.3).
const WALKERS: [NodeKind; 6] = [
    NodeKind::Move,
    NodeKind::MoveOn,
    NodeKind::Place,
    NodeKind::Price,
    NodeKind::Parts,
    NodeKind::Fill,
];

impl Engine {
    /// What layer `layer` holds.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a layer freed or never made.
    pub(super) fn layer_data(&self, layer: Layer) -> Result<&LayerData, Stop> {
        let at = usize::try_from(layer.0).map_err(|_| invariant("a layer's number"))?;
        self.layers
            .slots
            .get(at)
            .and_then(Option::as_ref)
            .ok_or_else(|| invariant("a read in a layer that was freed"))
    }

    /// The layer holding `data`: the one interned with the same pins, or a new one; the base
    /// for no pins (P §3.1).
    pub(super) fn intern(&mut self, data: LayerData) -> Result<Layer, Stop> {
        if data.is_empty() {
            return Ok(Layer::BASE);
        }
        for (at, slot) in self.layers.slots.iter().enumerate() {
            if let Some(held) = slot
                && held.same(&data)
            {
                return layer_at(at);
            }
        }
        self.layers.slots.push(Some(data));
        let at = self
            .layers
            .slots
            .len()
            .checked_sub(1)
            .ok_or_else(|| invariant("a count of layers"))?;
        layer_at(at)
    }

    /// The value a pin of the context gives `item` at `keys`: of the pins that hold there,
    /// the narrowest (P38), its `pinned:` entry noted unless it is a `solve` step's (refcore's
    /// `pin_value`). L §5.11, §5.18, §8.2, §4.6 step 2.
    ///
    /// # Errors
    ///
    /// `NotBuilt` for a pin on a record field; `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn pin_of(
        &mut self,
        item: &str,
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        if ctx.layer.is_base() {
            return Ok(None);
        }
        let data = self.layer_data(ctx.layer)?;
        let hits: Vec<&Pinned> = data
            .pins
            .iter()
            .filter(|p| p.holds_at(item, keys))
            .collect();
        if hits.iter().any(|p| !p.fields.is_empty()) {
            return Err(Stop::NotBuilt("a pin on a record field at run"));
        }
        // L §4.6's cascade: the innermost layer, then the narrower pin; two still tied hold
        // at once, refused at the cell (tally 2.1.7 R4)
        let Some(top) = hits.iter().map(|p| p.layer).max() else {
            return Ok(None);
        };
        let met = if self.pins_read.keeping {
            hits.clone()
        } else {
            Vec::new()
        };
        let hits: Vec<&Pinned> = hits.into_iter().filter(|p| p.layer == top).collect();
        let best: Vec<&Pinned> = hits
            .iter()
            .copied()
            .filter(|p| p.inside_all(&hits))
            .collect();
        let pin = match (best.as_slice(), hits.as_slice()) {
            ([one], _) => *one,
            (_, [first, second, ..]) => {
                return Err(conflict(&first.text, &second.text, item, keys));
            }
            (_, _) => return Err(invariant("pins that hold with no narrowest")),
        };
        if let Some(earlier) = &pin.tied {
            return Err(conflict(earlier, &pin.text, item, keys));
        }
        // L §8.2 One read set: the pins this read met under the claim it took (R4's cascade)
        let passed: Vec<String> = met
            .iter()
            .filter(|q| q.text != pin.text)
            .map(|q| q.text.clone())
            .collect();
        let (value, text, solve) = (pin.value.clone(), pin.text.clone(), pin.solve);
        if self.pins_read.keeping {
            self.pins_read.shadowed.extend(passed);
            if !self.pins_read.taken.contains(&text) {
                self.pins_read.taken.insert(text.clone());
            }
        }
        if !solve {
            self.note("pinned", text, String::new());
        }
        Ok(Some(value))
    }

    /// The day a `wording D for […]` pin of the context names `item` with, if one does
    /// (refcore's `wording_ok`). L §4.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn wording_of(&self, item: &str, ctx: Ctx) -> Result<Option<Day>, Stop> {
        if ctx.layer.is_base() {
            return Ok(None);
        }
        Ok(self.layer_data(ctx.layer)?.wording_of(item))
    }

    /// Context `ctx` trimmed for a cell of def `def`: only the pins and wording pins whose
    /// item the def reaches and the options of the axes it reaches, and where the def reads
    /// the timeline every one the walk reaches too, so a def no pin or axis reaches shares
    /// the unpinned cell (E §3.2: "an item that reaches no axis has no axis options", §3.3;
    /// P §3.1). The world is kept whole: a cell of one world is never another's.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn trimmed(&mut self, m: &Reality, def: &str, ctx: Ctx) -> Result<Ctx, Stop> {
        if ctx.layer.is_base() {
            return Ok(ctx);
        }
        let key = (def.to_owned(), ctx.layer.number());
        if let Some(layer) = self.layers.trimmed.get(&key) {
            return Ok(Ctx {
                layer: *layer,
                ..ctx
            });
        }
        let from = Node::new(NodeKind::Def, def);
        let walks = self.reads_walk(m, &from);
        let cone = self.walk_cone(m);
        let mut axes = self.def_axes(m, def);
        if walks {
            axes.extend(self.walk_axes(m, &cone));
        }
        let data = self.layer_data(ctx.layer)?;
        let mut items = BTreeSet::new();
        for item in data.items() {
            if reaches_item(m, &from, item) || (walks && in_cone(m, &cone, item)) {
                items.insert(item);
            }
        }
        let kept = data.kept(&items, &axes);
        self.layers.timeline = Some(cone);
        let layer = self.intern(kept)?;
        self.layers.trimmed.insert(key, layer);
        Ok(Ctx { layer, ..ctx })
    }

    /// The pins and wording pins of `layer` on items the walk reaches, and its options of the
    /// axes the walk reaches: the layer a timeline of a reader in `layer` is kept under
    /// (E §3.3: "a timeline per pin set the moves reach", "the axis options are those of the
    /// axes the moves reach").
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn walk_pins(&mut self, m: &Reality, layer: Layer) -> Result<Layer, Stop> {
        if layer.is_base() {
            return Ok(layer);
        }
        if let Some(kept) = self.layers.walk_pins.get(&layer.number()) {
            return Ok(*kept);
        }
        let cone = self.walk_cone(m);
        let axes = self.walk_axes(m, &cone);
        let data = self.layer_data(layer)?;
        let mut items = BTreeSet::new();
        for item in data.items() {
            if in_cone(m, &cone, item) {
                items.insert(item);
            }
        }
        let kept = data.kept(&items, &axes);
        self.layers.timeline = Some(cone);
        let pins = self.intern(kept)?;
        self.layers.walk_pins.insert(layer.number(), pins);
        Ok(pins)
    }

    /// The axes def `def` reaches on the graph, once computed (refcore's `def_axes`, as load
    /// counts an ask's, `reach_axes`). L §5.5, E §3.2.
    fn def_axes(&mut self, m: &Reality, def: &str) -> BTreeSet<String> {
        if let Some(axes) = self.layers.def_axes.get(def) {
            return axes.clone();
        }
        let axes = crate::load::reach_axes(m, &m.graph, &Node::new(NodeKind::Def, def));
        self.layers.def_axes.insert(def.to_owned(), axes.clone());
        axes
    }

    /// The axes the walk's readers reach, `cone` being the items they reach, once computed:
    /// what each of them reads by id, as a key or as a mode (E §3.3).
    fn walk_axes(&mut self, m: &Reality, cone: &BTreeSet<Node>) -> BTreeSet<String> {
        if let Some(axes) = &self.layers.walk_axes {
            return axes.clone();
        }
        let mut axes = BTreeSet::new();
        let walkers = m.graph.axes_of.keys().filter(|n| WALKERS.contains(&n.kind));
        for node in cone.iter().chain(walkers) {
            if let Some(read) = m.graph.axes_of.get(node) {
                axes.extend(read.iter().cloned());
            }
            if node.kind == NodeKind::Axis {
                axes.insert(node.id.clone());
            }
        }
        self.layers.walk_axes = Some(axes.clone());
        axes
    }

    /// The axes the walk's readers reach that layer `layer` sets no option of, in id order
    /// (E §3.3).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn unset_walk_axes(
        &mut self,
        m: &Reality,
        layer: Layer,
    ) -> Result<Vec<String>, Stop> {
        let cone = self.walk_cone(m);
        let axes = self.walk_axes(m, &cone);
        self.layers.timeline = Some(cone);
        let set = &self.layer_data(layer)?.axes;
        Ok(axes.into_iter().filter(|a| !set.contains_key(a)).collect())
    }

    /// The option axis `axis` takes in context `ctx`: a combination of the ask's fan-out or
    /// an axis pin (refcore's `axis_option`). L §5.5.
    ///
    /// # Errors
    ///
    /// `NotBuilt` for an axis the context sets no option of: read outside the ask's line
    /// (its `law`, `as known` or `with` pins, which refcore reads without the axes, its
    /// interpreter guard); `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn axis_option(&self, axis: &str, ctx: Ctx) -> Result<Value, Stop> {
        let option = if ctx.layer.is_base() {
            None
        } else {
            self.layer_data(ctx.layer)?.axes.get(axis).cloned()
        };
        let Some(name) = option else {
            return Err(Stop::NotBuilt(UNSET_AXIS));
        };
        Ok(Value::Key(KeyVal::Key {
            index: axis.to_owned(),
            name,
        }))
    }

    /// Context `ctx` with each axis of `combination` at its option: one combination of an
    /// ask's fan-out (L §5.5).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn with_options(
        &mut self,
        ctx: Ctx,
        combination: &[(String, String)],
    ) -> Result<Ctx, Stop> {
        let mut data = self.layer_data(ctx.layer)?.clone();
        for (axis, option) in combination {
            data.axes.insert(axis.clone(), option.clone());
        }
        let layer = self.intern(data)?;
        Ok(Ctx { layer, ..ctx })
    }

    /// The items the walk's readers reach, once computed; the caller puts it back in
    /// `layers.timeline` when done (E §3.1 step 5, §3.3).
    fn walk_cone(&mut self, m: &Reality) -> BTreeSet<Node> {
        match self.layers.timeline.take() {
            Some(cone) => cone,
            None => timeline_cone(m),
        }
    }

    /// Whether `item` reads the timeline, once judged (E §3.3).
    fn reads_walk(&mut self, m: &Reality, item: &Node) -> bool {
        if let Some(known) = self.layers.walkers.get(item) {
            return *known;
        }
        let reads = reads_timeline(m, item);
        self.layers.walkers.insert(item.clone(), reads);
        reads
    }

    /// The context of a `solve` step: `ctx` with `item` at `keys` pinned to `value`, listed
    /// nowhere, in a layer inside the pins in force and outside every pin written in F, so
    /// the item's other pins still hold at its other cells (L §5.18: "only that cell is
    /// pinned"; L §4.6's cascade, tally 2.1.7 R4, #120; refcore's `m_solve_value`).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn solve_step(
        &mut self,
        (item, keys): (&str, &[KeyVal]),
        value: Value,
        ctx: Ctx,
    ) -> Result<Ctx, Stop> {
        let mut data = self.layer_data(ctx.layer)?.clone();
        let depth = data.next_layer()?;
        data.pins.push(Pinned {
            solve: true,
            item: item.to_owned(),
            keys: keys.iter().cloned().map(Some).collect(),
            fields: Vec::new(),
            value,
            text: String::new(),
            layer: depth,
            tied: None,
        });
        let layer = self.intern(data)?;
        Ok(Ctx { layer, ..ctx })
    }

    /// Opens a scope of context `inside`: its layer's cells stay while it is open (P §3.5),
    /// and its reads of the walk read the timeline of its world and of the pins its walk
    /// reaches (E §3.3).
    ///
    /// # Errors
    ///
    /// `NotBuilt` for a decision's running read that would change timeline;
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn open_layer(&mut self, m: &Reality, inside: Ctx) -> Result<(), Stop> {
        self.enter_timeline(m, inside)?;
        self.layers.open.push(inside.layer);
        Ok(())
    }

    /// Leaves the scope that holds `layer` without freeing anything: a read unwound past the
    /// counted depth through it, and its replay needs the layer's cells (R-16); the timeline
    /// before it is back.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a scope closed out of order.
    pub(crate) fn leave_layer(&mut self, layer: Layer) -> Result<(), Stop> {
        if self.layers.open.pop() != Some(layer) {
            return Err(invariant("a layer's scope closed out of order"));
        }
        self.leave_timeline()
    }

    /// Ends the scope that holds `layer`, whose read stopped with `stop` if it stopped:
    /// closed, or, where the read unwound past the counted depth, left with its cells kept
    /// until the outermost read ends (R-16, P §3.5).
    ///
    /// # Errors
    ///
    /// As [`Engine::close_layer`].
    pub(crate) fn end_layer(&mut self, layer: Layer, stop: Option<&Stop>) -> Result<(), Stop> {
        if !matches!(stop, Some(Stop::Deep)) {
            return self.close_layer(layer);
        }
        self.leave_layer(layer)?;
        if !self.layers.kept.contains(&layer) {
            self.layers.kept.push(layer);
        }
        Ok(())
    }

    /// Closes the scope that holds `layer`: every layer no open scope holds is freed with its
    /// cells and the timelines kept under it (P §3.2, §3.5: freed when its `as if` or
    /// `solve` step ends).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a scope closed out of order; `ENGINE-INVARIANT asks move no
    /// money: …` for a freed timeline's walk whose entries changed.
    pub(crate) fn close_layer(&mut self, layer: Layer) -> Result<(), Stop> {
        self.leave_layer(layer)?;
        self.free_layers()
    }

    /// Ends an outermost read: the layers its unwinding kept are let go, to be freed at the
    /// next scope's end that no scope holds them past (R-16, P §3.5). Nothing is freed here: a
    /// read may hand back a layer it has just made, such as an ask's `with` pins', which its
    /// caller opens next.
    pub(super) fn release_kept(&mut self) {
        self.layers.kept.clear();
    }

    /// Frees every layer that no open scope, no scope a read unwound through and no cell on
    /// the work stack holds, with its cells and the timelines kept under it (P §3.5).
    fn free_layers(&mut self) -> Result<(), Stop> {
        let mut holders: Vec<Layer> = self.layers.open.clone();
        holders.extend(&self.layers.kept);
        for task in &self.work {
            holders.push(task.ctx.layer);
        }
        let mut held = Vec::with_capacity(holders.len());
        for layer in holders {
            held.push(self.layer_data(layer)?.clone());
        }
        let mut freed = BTreeSet::new();
        for (at, slot) in self.layers.slots.iter_mut().enumerate().skip(1) {
            let Some(data) = slot else {
                continue;
            };
            if !held.iter().any(|o| data.within(o)) {
                freed.insert(layer_at(at)?.number());
                *slot = None;
            }
        }
        if freed.is_empty() {
            return Ok(());
        }
        self.layers
            .trimmed
            .retain(|(_, from), to| !freed.contains(from) && !freed.contains(&to.number()));
        self.layers
            .walk_pins
            .retain(|from, to| !freed.contains(from) && !freed.contains(&to.number()));
        self.cells.free(&freed);
        self.free_timelines(&freed)
    }
}

/// The world scenario `id` names in reality `m`: the model's, one of its stack's (L §6.4).
///
/// # Errors
///
/// `NotBuilt` for a scenario of another stack, which the run answers in the reality holding
/// it (refcore's `make_cross`): not built.
pub(crate) fn world_named(m: &Reality, id: &str) -> Result<World, Stop> {
    m.scenarios.world(id, &m.header.id).ok_or(Stop::NotBuilt(
        "a world on another stack of scenarios than the ask's (refcore's cross evaluation)",
    ))
}

/// Whether the def at `from` reaches the item `item`, as the graph's edges tell; an item the
/// graph has no node for is kept, as reached (E §3.2: a cell is keyed by the pins it reads).
fn reaches_item(m: &Reality, from: &Node, item: &str) -> bool {
    match item_node(m, item) {
        Some(to) => m.graph.reaches(from, &to),
        None => true,
    }
}

/// Whether item `item` is in the timeline's cone `cone`; an item the graph has no node for is
/// taken as in it.
fn in_cone(m: &Reality, cone: &BTreeSet<Node>, item: &str) -> bool {
    match item_node(m, item) {
        Some(node) => cone.contains(&node),
        None => true,
    }
}

/// The graph's node of an input, a def or a band named `item`; none for another name.
fn item_node(m: &Reality, item: &str) -> Option<Node> {
    let kind = match m.names.kind(item)? {
        NameKind::Input => NodeKind::Input,
        NameKind::Def => NodeKind::Def,
        NameKind::Band => NodeKind::Band,
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Place
        | NameKind::Fill
        | NameKind::Move
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source
        | NameKind::Cut => return None,
    };
    Some(Node::new(kind, item))
}

/// `RUN-CONFLICT pins <a> and <b> both hold for <x>[k]`: two pins of one layer, neither
/// narrower, hold for one cell, whose keys only the run gives (L §4.6's cascade, §8.4; tally
/// 2.1.7 R4).
fn conflict(a: &str, b: &str, item: &str, keys: &[KeyVal]) -> Stop {
    refuse(
        Run::Conflict,
        format!(
            "pins {a} and {b} both hold for {}",
            crate::read::origin(item, keys)
        ),
    )
}

/// The layer at slot `at`.
fn layer_at(at: usize) -> Result<Layer, Stop> {
    Ok(Layer(
        u32::try_from(at).map_err(|_| invariant("a count of layers"))?,
    ))
}

/// Every item the walk's readers reach: the moves, their days, the places, the prices, the
/// parts and the fills, with all they read (E §3.1 step 5, §3.3).
fn timeline_cone(m: &Reality) -> BTreeSet<Node> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<&Node> = m
        .graph
        .edges
        .keys()
        .filter(|n| WALKERS.contains(&n.kind))
        .collect();
    while let Some(next) = stack.pop() {
        if let Some(reads) = m.graph.edges.get(next) {
            for target in reads.keys() {
                if seen.insert(target.clone()) {
                    stack.push(target);
                }
            }
        }
    }
    seen
}

/// Whether `holder` reads the timeline: it reaches the balances, a move, a move's day, a
/// place, a price, parts or a fill (E §3.3).
fn reads_timeline(m: &Reality, holder: &Node) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![holder];
    while let Some(next) = stack.pop() {
        if next.kind == NodeKind::Balances || WALKERS.contains(&next.kind) {
            return true;
        }
        if let Some(reads) = m.graph.edges.get(next) {
            for target in reads.keys() {
                if seen.insert(target) {
                    stack.push(target);
                }
            }
        }
    }
    false
}
