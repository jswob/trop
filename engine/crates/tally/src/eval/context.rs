//! The context's layers over its law date and cut, interned as one small id: its pins, its wording pins and its axis options.
//! Spec: E §3.2 (the context; trimmed to what the item can depend on: an item that no pin
//! reaches shares the unpinned cell, one that reaches no axis has no axis options; a pin set is keyed by the pins' values; a pin given by an
//! expression is evaluated first, without that `as if`), §3.3 (a timeline per pin set the
//! moves reach); P §3.1 (the context interned once as a small id, after it is trimmed), §3.2
//! (a pin set's layer, freed when its `as if` or `solve` step ends), §3.5 (one memo per
//! context, freed when its scope ends); L §5.11 (`as if` and its pins: a pin's value and keys,
//! a hole pin, nested pins don't leak to their siblings), §4.3 (the `law` and `wording`
//! pins), §5.18 (an ask's `with` pins), §8.2 (`pinned:`, `world:`); §6.4 (`as if {scenario
//! s}` is x in s); §5.5 (an axis read at its option; `with a = opt` fixes it, listed
//! `pinned:`); E §10 (an `as if` explained: the pins with their values and the derivation
//! inside them beside the one outside).
//! Twin: engine/tools/refcore/evaluate.py (`PinVal`, `Ctx.pins`, `Ctx.wording`, `Ctx.axes`,
//! `apply_pins`, `e_asif`, `asif_traced`, `pin_lines`, `pin_value`, `wording_ok`,
//! `coerce_to`, `world_chain`, `axis_option`, `def_axes`).
//! Invariants: layer 0 is the unpinned context; two pin sets with the same targets and values
//! are one layer, whatever their texts; a def's cell is kept under its context trimmed to the
//! pins its def reaches, and to every pin the walk reaches where it reads the timeline; a
//! layer no open scope holds is freed with its cells and timelines when the scope that made it
//! ends, never while a read that unwound past the counted depth still needs it; a scope's
//! reads of the walk read the timeline of its world and walk pins (`timelines.rs`). The world
//! is the context's own field, never trimmed; an axis option is the layer's, trimmed as pins
//! are, and set for one combination of an ask's fan-out or by an axis pin.

use std::collections::{BTreeMap, BTreeSet};

use super::state::{Ctx, Engine, Env, invariant, num_stop};
use super::trace::Kind as TraceKind;
use crate::base::Code;
use crate::hole::{Hole, spread};
use crate::model::{
    Class, KeyVal, NameKind, Node, NodeKind, Number, Reality, Ty, Unit, Value, World, currency_id,
    money_num, num_money,
};
use crate::syntax::{Expr, Key, Lit, PRef, Pin, PinForm, Stop};
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
    solve: bool,
    /// The input or def pinned.
    item: String,
    /// Its keys by position, none at a `*`; empty for an item with no keys.
    keys: Vec<Option<KeyVal>>,
    /// The record fields pinned, outermost first.
    fields: Vec<String>,
    /// The value pinned.
    value: Value,
    /// The pin as the register prints it.
    text: String,
}

impl Pinned {
    /// Whether `other` pins the same cells: the same item, keys and fields (refcore's
    /// `key()[:4]`).
    fn same_target(&self, other: &Pinned) -> bool {
        self.solve == other.solve
            && self.item == other.item
            && self.keys == other.keys
            && self.fields == other.fields
    }

    /// Whether `other` is the same pin: the same target and value, whatever its text
    /// (E §3.2: "keyed by the pins' values").
    fn same(&self, other: &Pinned) -> bool {
        self.same_target(other) && self.value == other.value
    }

    /// Whether the pin is wider than `other` at a cell both hold at: at the first key position
    /// where one names a key and the other `*`, `other` names it. The narrower pin wins, as a
    /// row does (L §4.6 step 2; pick P38, #119), whatever the order the pins were written
    /// in; refcore's pins, sorted by their keys' text, put a named key before `*` alike.
    fn wider_than(&self, other: &Pinned) -> bool {
        for (mine, theirs) in self.keys.iter().zip(&other.keys) {
            if mine.is_some() != theirs.is_some() {
                return theirs.is_some();
            }
        }
        false
    }

    /// Whether the pin names exactly the cell at `keys`, every key named, none `*`.
    fn names_cell(&self, keys: &[KeyVal]) -> bool {
        self.keys.len() == keys.len()
            && self
                .keys
                .iter()
                .zip(keys)
                .all(|(pinned, key)| pinned.as_ref() == Some(key))
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
    pins: Vec<Pinned>,
    /// Each item a `wording D for […]` pin names, with D; the first pin naming an item holds
    /// (refcore's `wording_ok`).
    wording: Vec<(String, Day)>,
    /// The option each axis takes, by the axis's id: one combination of an ask's fan-out, or
    /// an axis pin (L §5.5).
    axes: BTreeMap<String, String>,
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
    fn holds(&self, pin: &Pinned) -> bool {
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

    /// Adds pin `pin`, replacing one of the same target.
    fn pin(&mut self, pin: Pinned) {
        self.pins.retain(|p| !p.same_target(&pin));
        self.pins.push(pin);
    }

    /// Adds a `wording D for [ids]` pin: each item it names that no pin named before takes D
    /// (refcore's `wording_ok`: the first pin naming an item holds).
    fn word(&mut self, ids: &[String], day: Day) {
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
    fn intern(&mut self, data: LayerData) -> Result<Layer, Stop> {
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
        let mut narrowest: Option<&Pinned> = None;
        for pin in data.pins.iter().filter(|p| p.holds_at(item, keys)) {
            if !pin.fields.is_empty() {
                return Err(Stop::NotBuilt("a pin on a record field at run"));
            }
            if narrowest.is_none_or(|held| held.wider_than(pin)) {
                narrowest = Some(pin);
            }
        }
        let Some(pin) = narrowest else {
            return Ok(None);
        };
        let (value, text, solve) = (pin.value.clone(), pin.text.clone(), pin.solve);
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

    /// `e as if {pins}`: e evaluated in the context the pins give, their values evaluated
    /// first in the context outside (refcore's `e_asif`); a pin whose key or date is a hole
    /// makes the value that hole, nothing inside evaluated; `{scenario s}` evaluates e in s
    /// (L §6.4), the moves recomputed in a timeline of their own where a pin or the world
    /// reaches them. L §5.11.
    ///
    /// # Errors
    ///
    /// The refusals the pins and e meet; `NotBuilt` for a sample pin, a world of
    /// another stack of scenarios, and a decision's running read that would change timeline.
    pub(super) fn as_if(
        &mut self,
        m: &Reality,
        (inner, pins): (&Expr, &[Pin]),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        if self.tracing() {
            return self.as_if_traced(m, (inner, pins), env, ctx);
        }
        let inside = match self.pinned(m, pins, env, ctx, Pinning::AsIf)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(inside) => inside,
        };
        self.open_layer(m, inside)?;
        let v = self.ev(m, inner, env, inside);
        self.end_layer(inside.layer, v.as_ref().err())?;
        v
    }

    /// `e as if {pins}` explained, under its node `as if {...}`: the pins with their values,
    /// each pin given by an expression with its derivation, read outside the `as if`; then the
    /// derivation inside the pins with its value; then the one outside them with its value,
    /// which brings nothing to the result's register and whose refusal stands in its place
    /// (refcore's `asif_traced`). E §10.
    fn as_if_traced(
        &mut self,
        m: &Reality,
        (inner, pins): (&Expr, &[Pin]),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let node = self.trace_open(TraceKind::Group, "as if {...}");
        let group = self.trace_open(TraceKind::Group, "the pins, evaluated outside the as if");
        let pinned = self.pinned(m, pins, env, ctx, Pinning::AsIf);
        let inside = match pinned {
            Ok(Ok(inside)) => inside,
            Ok(Err(hole)) => {
                self.trace_close(group);
                self.trace_close(node);
                return Ok(Value::Hole(hole));
            }
            Err(stop) => {
                self.trace_close(group);
                self.trace_close(node);
                return Err(stop);
            }
        };
        match self.pin_lines(m, ctx, inside) {
            Ok(lines) => {
                for line in lines {
                    self.trace_leaf(format!("pin {line}"));
                }
            }
            Err(stop) => self.trace_kept(stop),
        }
        self.trace_close(group);
        let within = self.trace_open(TraceKind::Side, "inside the as if");
        let read = self.open_layer(m, inside).and_then(|()| {
            let v = self.ev(m, inner, env, inside);
            self.end_layer(inside.layer, v.as_ref().err())?;
            v
        });
        self.trace_close(within);
        // a read that stops leaves the side as it is and nothing outside (refcore's raise)
        let v = match read {
            Ok(v) => v,
            Err(stop) => {
                self.trace_close(node);
                return Err(stop);
            }
        };
        self.trace_value(within, &Ok(v.clone()));
        let without = self.trace_open(TraceKind::Side, "outside the as if");
        self.push();
        let outside = self.ev(m, inner, env, ctx);
        self.pop_alone();
        self.trace_close(without);
        self.trace_close(node);
        match outside {
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(Stop::Refused(r)),
            Err(Stop::Refused(r)) => {
                self.trace_value(without, &Err(Stop::Refused(r)));
                Ok(v)
            }
            Err(stop) => Err(stop),
            Ok(o) => {
                self.trace_value(without, &Ok(o));
                Ok(v)
            }
        }
    }

    /// The context of an ask's line of context `line` outside the ask's `with` pins:
    /// `unpinned`, the line's context before them, with the options of the axes the line fans
    /// out over, those in `fixed`, which the pins fix, left out (refcore's `explain_pins`, its
    /// `octx`). E §10, L §5.5.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn outside_pins(
        &mut self,
        (unpinned, line): (Ctx, Ctx),
        fixed: &BTreeSet<String>,
    ) -> Result<Ctx, Stop> {
        let axes = &self.layer_data(line.layer)?.axes;
        let options: Vec<(String, String)> = axes
            .iter()
            .filter(|(axis, _)| !fixed.contains(*axis))
            .map(|(axis, option)| (axis.clone(), option.clone()))
            .collect();
        self.with_options(unpinned, &options)
    }

    /// What the pins of `inside` change from `outside`, each as text: the value pins with
    /// their values, the law date, the cut, the wording pins, the axis options and the world
    /// (refcore's `pin_lines`). E §10.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn pin_lines(
        &self,
        m: &Reality,
        outside: Ctx,
        inside: Ctx,
    ) -> Result<Vec<String>, Stop> {
        let (before, after) = (
            self.layer_data(outside.layer)?,
            self.layer_data(inside.layer)?,
        );
        // refcore's pins are sorted by their keys' text, the item's id first
        let mut new: Vec<&Pinned> = after
            .pins
            .iter()
            .filter(|p| !p.solve && !before.holds(p))
            .collect();
        new.sort_by(|a, b| (&a.item, &a.text).cmp(&(&b.item, &b.text)));
        let mut out: Vec<String> = new.iter().map(|p| p.text.clone()).collect();
        if let (true, Some(day)) = (inside.law != outside.law, inside.law) {
            out.push(format!("law {}", day.text()));
        }
        if inside.cut != outside.cut {
            out.push(format!("known {}", inside.cut.text()));
        }
        for (item, day) in &after.wording {
            if !before.wording.contains(&(item.clone(), *day)) {
                out.push(format!("wording {} for {item}", day.text()));
            }
        }
        for (axis, option) in &after.axes {
            if before.axes.get(axis) != Some(option) {
                out.push(format!("{axis} = {option}"));
            }
        }
        if inside.world != outside.world {
            let world = m
                .scenarios
                .scenario_of(inside.world)
                .unwrap_or(&m.header.id);
            out.push(format!("scenario {world}"));
        }
        Ok(out)
    }

    /// The context inside `pins` (refcore's `apply_pins`): each pin's date, keys and value
    /// evaluated in `ctx`; `law D` the law date, `known D` the cut, `wording D for […]` the
    /// named items' wording, a value pin its target's cells. A pin given by an expression
    /// brings what it read to the reader's register, with its `pinned:` entry; an ask's value
    /// pins are listed whether read or not. A pin whose key, or a date pin whose date, is a
    /// hole makes the result that hole, with every such pin's questions. L §5.11, §5.18, §8.2.
    ///
    /// `scenario s` sets the world, listed `world: s` (L §6.4, §8.2).
    ///
    /// # Errors
    ///
    /// The refusals the pins meet; `NotBuilt` for a sample pin and for a world of
    /// another stack of scenarios.
    pub(crate) fn pinned(
        &mut self,
        m: &Reality,
        pins: &[Pin],
        env: &Env,
        ctx: Ctx,
        how: Pinning,
    ) -> Result<Result<Ctx, Hole>, Stop> {
        let mut data = self.layer_data(ctx.layer)?.clone();
        let mut inside = ctx;
        let mut holes: Vec<Hole> = Vec::new();
        for pin in pins {
            match &pin.form {
                PinForm::Scenario(id) => {
                    // `x as if {scenario s}` is x in s (L §6.4 Asking; refcore's `apply_pins_`)
                    inside.world = world_named(m, id)?;
                    self.note("world", id.clone(), String::new());
                }
                PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => {
                    let Some(day) = self.date_pin(m, pin, e, env, ctx, &mut holes)? else {
                        continue;
                    };
                    match &pin.form {
                        PinForm::Law(_) => inside.law = Some(day),
                        PinForm::Known(_) => inside.cut = day,
                        PinForm::Wording(_, ids) => data.word(ids, day),
                        PinForm::Scenario(_) | PinForm::Set { .. } => {}
                    }
                }
                PinForm::Set {
                    target,
                    value,
                    sample,
                } => {
                    if *sample {
                        return Err(Stop::NotBuilt("a sample pin at run"));
                    }
                    if m.names.kind(&target.id) == Some(NameKind::Axis) {
                        // `ax = option` fixes the axis, listed under `pinned:` (L §5.5)
                        let option = axis_pinned(value)?;
                        let text = format!("{} = {option}", target.id);
                        data.axes.insert(target.id.clone(), option);
                        self.note("pinned", text, String::new());
                        continue;
                    }
                    let set = (target, value);
                    if let Some(pinned) = self.value_pin(m, set, env, ctx, (how, &mut holes))? {
                        data.pin(pinned);
                    }
                }
            }
        }
        let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
        if let Some(hole) = spread(&operands) {
            return Ok(Err(hole));
        }
        inside.layer = self.intern(data)?;
        Ok(Ok(inside))
    }

    /// The context of a `solve` step: `ctx` with `item` at `keys` pinned to `value`, listed
    /// nowhere; a pin of that one cell is set aside, and the item's other pins stay, so a
    /// `with` pin of the item still holds at its other cells (L §5.18: "only that cell is
    /// pinned"; pick P40, #120, where refcore's `m_solve_value` drops every pin of the item).
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
        data.pins
            .retain(|p| !(p.item == item && p.fields.is_empty() && p.names_cell(keys)));
        data.pins.push(Pinned {
            solve: true,
            item: item.to_owned(),
            keys: keys.iter().cloned().map(Some).collect(),
            fields: Vec::new(),
            value,
            text: String::new(),
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

    /// A `law`, `known` or `wording` pin's day, its `pinned:` entry noted, with what an
    /// expression read; none, its hole kept in `holes`, for a date that is a hole (refcore's
    /// `apply_pins_`, its date arm). L §5.11, §8.2.
    fn date_pin(
        &mut self,
        m: &Reality,
        pin: &Pin,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
        holes: &mut Vec<Hole>,
    ) -> Result<Option<Day>, Stop> {
        self.push();
        let v = self.ev(m, e, env, ctx);
        let met = self.pop_alone();
        let v = v?;
        if let Value::Hole(hole) = v {
            holes.push(hole);
            return Ok(None);
        }
        let word = match &pin.form {
            PinForm::Law(_) => "law",
            PinForm::Known(_) => "known",
            PinForm::Wording(..) | PinForm::Scenario(_) | PinForm::Set { .. } => "wording",
        };
        let day = crate::ask::date_of(&v, &format!("the {word} pin"))?;
        let expr = !matches!(
            e,
            Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) | Lit::Num(_))
        );
        if expr {
            self.absorb(&met);
        }
        let written = if expr {
            crate::load::expr_text(e)
        } else {
            day.text()
        };
        let head = match &pin.form {
            PinForm::Wording(_, ids) => format!("wording {written} for [{}]", ids.join(", ")),
            PinForm::Law(_) | PinForm::Known(_) | PinForm::Scenario(_) | PinForm::Set { .. } => {
                format!("{word} {written}")
            }
        };
        let text = if expr {
            format!("{head} = {}", day.text())
        } else {
            head
        };
        self.note("pinned", text, String::new());
        Ok(Some(day))
    }

    /// A value pin `x[keys].f = v`: its keys and value evaluated in `ctx`, the value in its
    /// target's type; an ask's noted `pinned:`; none, its hole kept in `holes`, for a key
    /// that is a hole (refcore's `apply_pins_`, its target arm, and `coerce_to`). L §5.11,
    /// §8.2.
    fn value_pin(
        &mut self,
        m: &Reality,
        (target, written): (&PRef, &Expr),
        env: &Env,
        ctx: Ctx,
        (how, holes): (Pinning, &mut Vec<Hole>),
    ) -> Result<Option<Pinned>, Stop> {
        let id = target.id.as_str();
        let (binds, declared) = match m.names.kind(id) {
            Some(NameKind::Input) => {
                let input = m
                    .inputs
                    .get(id)
                    .ok_or_else(|| invariant("a pinned input"))?;
                (&input.binds, input.ty.clone())
            }
            Some(NameKind::Def) => {
                let def = m.defs.get(id).ok_or_else(|| invariant("a pinned def"))?;
                (&def.binds, Some(def.ty.clone()))
            }
            _ => return Err(invariant("a pin load didn't check")),
        };
        let keyed = |key: &Key| matches!(key, Key::Expr(e) if env.read_by(e));
        if env.read_by(written) || target.keys.iter().flatten().any(keyed) {
            // L §8.2: a pin formed per binding tuple prints once, as written, with its
            // binding (refcore's `_pertuple`)
            return Err(Stop::NotBuilt("a pin formed per binding tuple at run"));
        }
        let mut keys = Vec::with_capacity(binds.len());
        let mut texts = Vec::with_capacity(binds.len());
        for (key, bind) in target.keys.iter().flatten().zip(binds) {
            let Key::Expr(e) = key else {
                keys.push(None);
                texts.push("*".to_owned());
                continue;
            };
            let given = self.given_key(m, e, env, ctx)?;
            match self.make_keys(m, id, core::slice::from_ref(bind), vec![given], ctx)? {
                Err(hole) => {
                    holes.push(hole);
                    return Ok(None);
                }
                Ok(made) => {
                    let [one] = made.as_slice() else {
                        return Err(invariant("a pin's key made more keys than one"));
                    };
                    texts.push(one.text());
                    keys.push(Some(one.clone()));
                }
            }
        }
        let mut ty = declared;
        for field in &target.fields {
            ty = ty.and_then(|t| t.field(field).cloned());
        }
        self.push();
        let v = self.ev(m, written, env, ctx);
        let met = self.pop_alone();
        let value = pin_typed(m, v?, ty.as_ref())?;
        if !plain_literal(m, written) {
            self.absorb(&met);
        }
        let mut head = id.to_owned();
        if !texts.is_empty() {
            head = format!("{head}[{}]", texts.join(", "));
        }
        for field in &target.fields {
            head = format!("{head}.{field}");
        }
        let text = format!("{head} = {}", value.typed_text(m).map_err(num_stop)?);
        if how == Pinning::Ask {
            // an ask's `with` pins print with every line (L §5.18 The world of an ask)
            self.note("pinned", text.clone(), String::new());
        }
        Ok(Some(Pinned {
            solve: false,
            item: id.to_owned(),
            keys,
            fields: target.fields.clone(),
            value,
            text,
        }))
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

/// The option an axis pin `ax = option` names, a key literal load checked (refcore's
/// `p.option`). L §5.5.
fn axis_pinned(value: &Expr) -> Result<String, Stop> {
    let (Expr::Name(option) | Expr::Lit(Lit::Key(option))) = value else {
        return Err(invariant(
            "an axis pin that names no option, which load refuses",
        ));
    };
    Ok(option.clone())
}

/// What applies a set of pins: an ask's `with`, whose value pins are listed whether read or
/// not, or an `as if` (L §5.18, §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pinning {
    /// An ask's `with` pins.
    Ask,
    /// An `as if`'s pins.
    AsIf,
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

/// Whether a pin's value is written plainly, so what it read stays off the register: a
/// literal, a negation, a text, or a name that reads no input or def (refcore's
/// `apply_pins_`, its kinds). L §8.2.
fn plain_literal(m: &Reality, e: &Expr) -> bool {
    if let Expr::Name(name) = e {
        return !matches!(m.names.kind(name), Some(NameKind::Input | NameKind::Def));
    }
    matches!(e, Expr::Lit(_) | Expr::Neg(_) | Expr::Str(_))
}

/// A pin's value in its target's type: a number pinned on money is money where it is whole
/// minor units, an int or money pinned on a `num` takes its unit (refcore's `coerce_to`).
/// L §3.2 Inline figures, §5.11.
fn pin_typed(m: &Reality, v: Value, ty: Option<&Ty>) -> Result<Value, Stop> {
    match (ty, v) {
        (Some(Ty::Money(cur)), Value::Number(n)) => {
            let minor = m
                .currencies
                .get(cur)
                .ok_or_else(|| invariant("a currency load didn't declare"))?;
            if !n.q.checked_div(minor).map_err(num_stop)?.is_integer() {
                return Ok(Value::Number(n));
            }
            let id = currency_id(m, cur).map_err(num_stop)?;
            Ok(Value::Money(num_money(m, &n.q, id).map_err(num_stop)?))
        }
        (Some(Ty::Num(unit, shown)), Value::Number(n)) if n.class == Class::Int => {
            Ok(Value::Number(Number {
                q: n.q,
                class: Class::Num(unit.clone(), *shown),
            }))
        }
        (Some(Ty::Num(..)), Value::Money(money)) => {
            let q = money_num(m, &money).map_err(num_stop)?;
            let unit: Unit = crate::model::currency_unit(m, money.currency()).map_err(num_stop)?;
            Ok(Value::Number(Number::plain(q, unit)))
        }
        (_, v) => Ok(v),
    }
}
