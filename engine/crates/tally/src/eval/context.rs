//! The context's layers over its law date and cut, interned as one small id: the pins and wording pins of an ask's `with`, an `as if` or a `solve` step.
//! Spec: E §3.2 (the context; trimmed to what the item can depend on: an item that no pin
//! reaches shares the unpinned cell; a pin set is keyed by the pins' values; a pin given by an
//! expression is evaluated first, without that `as if`), §3.3 (a timeline per pin set the
//! moves reach); P §3.1 (the context interned once as a small id, after it is trimmed), §3.2
//! (a pin set's layer, freed when its `as if` or `solve` step ends), §3.5 (one memo per
//! context, freed when its scope ends); L §5.11 (`as if` and its pins: a pin's value and keys,
//! a hole pin, nested pins don't leak to their siblings), §4.3 (the `law` and `wording`
//! pins), §5.18 (an ask's `with` pins), §8.2 (`pinned:`).
//! Twin: engine/tools/refcore/evaluate.py (`PinVal`, `Ctx.pins`, `Ctx.wording`, `apply_pins`,
//! `e_asif`, `pin_value`, `wording_ok`, `coerce_to`).
//! Invariants: layer 0 is the unpinned context; two pin sets with the same targets and values
//! are one layer, whatever their texts; a def's cell is kept under its context trimmed to the
//! pins its def reaches; a layer no open scope holds is freed with its cells when the scope
//! that made it ends, never while a read that unwound past the counted depth still needs it;
//! a pin the moves reach, where the walk is read, stops `NotBuilt` (one timeline per pin set
//! is the world layer's, with it). The world (G6) and the axis options (G7) join the layer.

use std::collections::{BTreeMap, BTreeSet};

use super::read::make_keys;
use super::state::{Ctx, Engine, Env, invariant, num_stop};
use crate::hole::{Hole, spread};
use crate::model::{
    Class, KeyVal, NameKind, Node, NodeKind, Number, Reality, Ty, Unit, Value, currency_id,
    money_num, num_money,
};
use crate::syntax::{Expr, Key, Lit, Pin, PinForm, Stop};
use crate::time::Day;

/// A context's layer above its law date and cut: an interned set of pins (E §3.2, P §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    /// Whether the pin holds at `keys` of its item: every key it names is the key read.
    fn holds_at(&self, item: &str, keys: &[KeyVal]) -> bool {
        self.item == item
            && (self.keys.is_empty() && keys.is_empty() || self.keys.len() == keys.len())
            && self
                .keys
                .iter()
                .zip(keys)
                .all(|(pinned, key)| pinned.as_ref().is_none_or(|p| p == key))
    }
}

/// What a layer holds: its pins and its wording pins (E §3.2; refcore's `Ctx.pins` and
/// `Ctx.wording`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LayerData {
    /// The pins, in the order applied: a later pin of the same target replaces an earlier.
    pins: Vec<Pinned>,
    /// Each item a `wording D for […]` pin names, with D; the first pin naming an item holds
    /// (refcore's `wording_ok`).
    wording: Vec<(String, Day)>,
}

impl LayerData {
    /// Whether it pins nothing: the unpinned context.
    fn is_empty(&self) -> bool {
        self.pins.is_empty() && self.wording.is_empty()
    }

    /// Whether every pin and wording pin of this layer is one of `other`'s.
    fn within(&self, other: &LayerData) -> bool {
        self.pins
            .iter()
            .all(|p| other.pins.iter().any(|o| p.same(o)))
            && self.wording.iter().all(|w| other.wording.contains(w))
    }

    /// Whether the two layers hold the same pins, as sets.
    fn same(&self, other: &LayerData) -> bool {
        self.pins.len() == other.pins.len()
            && self.wording.len() == other.wording.len()
            && self.within(other)
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
    /// Each def's trimmed layer of a layer (E §3.2).
    trimmed: BTreeMap<(String, Layer), Layer>,
    /// The layers of the scopes open now: an ask's `with`, an `as if`, a `solve` step.
    open: Vec<Layer>,
    /// The items the moves, places, prices, parts and fills reach, once computed.
    timeline: Option<BTreeSet<Node>>,
    /// Whether each item that held a pin reads the timeline, once computed.
    walkers: BTreeMap<Node, bool>,
}

impl Default for Layers {
    fn default() -> Layers {
        Layers {
            slots: vec![Some(LayerData::default())],
            trimmed: BTreeMap::new(),
            open: Vec::new(),
            timeline: None,
            walkers: BTreeMap::new(),
        }
    }
}

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
        let found = self
            .layers
            .slots
            .iter()
            .position(|slot| slot.as_ref().is_some_and(|held| held.same(&data)));
        if let Some(at) = found {
            return layer_at(at);
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

    /// The value a pin of the context gives `item` at `keys`, the latest pin that holds there,
    /// its `pinned:` entry noted unless it is a `solve` step's (refcore's `pin_value`).
    /// L §5.11, §5.18, §8.2.
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
        let Some(pin) = data.pins.iter().rev().find(|p| p.holds_at(item, keys)) else {
            return Ok(None);
        };
        if !pin.fields.is_empty() {
            return Err(Stop::NotBuilt("a pin on a record field at run"));
        }
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
    /// item the def reaches, so a def no pin reaches shares the unpinned cell (E §3.2,
    /// P §3.1).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn trimmed(&mut self, m: &Reality, def: &str, ctx: Ctx) -> Result<Ctx, Stop> {
        if ctx.layer.is_base() {
            return Ok(ctx);
        }
        let key = (def.to_owned(), ctx.layer);
        if let Some(layer) = self.layers.trimmed.get(&key) {
            return Ok(Ctx {
                layer: *layer,
                ..ctx
            });
        }
        let data = self.layer_data(ctx.layer)?;
        let from = Node::new(NodeKind::Def, def);
        let reached = |item: &str| item_node(m, item).is_none_or(|to| m.graph.reaches(&from, &to));
        let kept = LayerData {
            pins: data
                .pins
                .iter()
                .filter(|p| reached(&p.item))
                .cloned()
                .collect(),
            wording: data
                .wording
                .iter()
                .filter(|(item, _)| reached(item))
                .cloned()
                .collect(),
        };
        let layer = self.intern(kept)?;
        self.layers.trimmed.insert(key, layer);
        Ok(Ctx { layer, ..ctx })
    }

    /// `e as if {pins}`: e evaluated in the context the pins give, their values evaluated
    /// first in the context outside (refcore's `e_asif`); a pin whose key or date is a hole
    /// makes the value that hole, nothing inside evaluated. L §5.11.
    ///
    /// # Errors
    ///
    /// The refusals the pins and e meet; `NotBuilt` for a scenario, an axis or a sample pin,
    /// and for a pin the moves reach where the walk is read.
    pub(super) fn as_if(
        &mut self,
        m: &Reality,
        (inner, pins): (&Expr, &[Pin]),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let inside = match self.pinned(m, pins, env, ctx, Pinning::AsIf)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(inside) => inside,
        };
        self.open_layer(inside.layer);
        let v = self.ev(m, inner, env, inside);
        if matches!(v, Err(Stop::Deep)) {
            self.leave_layer(inside.layer)?;
        } else {
            self.close_layer(inside.layer)?;
        }
        v
    }

    /// The context inside `pins` (refcore's `apply_pins`): each pin's date, keys and value
    /// evaluated in `ctx`; `law D` the law date, `known D` the cut, `wording D for […]` the
    /// named items' wording, a value pin its target's cells. A pin given by an expression
    /// brings what it read to the reader's register, with its `pinned:` entry; an ask's value
    /// pins are listed whether read or not. A pin whose key, or a date pin whose date, is a
    /// hole makes the result that hole, with every such pin's questions. L §5.11, §5.18, §8.2.
    ///
    /// # Errors
    ///
    /// The refusals the pins meet; `NotBuilt` for a scenario, an axis or a sample pin, and for
    /// a pin the moves reach where the walk is read.
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
                PinForm::Scenario(_) => return Err(Stop::NotBuilt("a scenario pin at run")),
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
                    let set = SetPin { target, value };
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
        self.no_pinned_walk(m, inside.layer)?;
        Ok(Ok(inside))
    }

    /// The context of a `solve` step: `ctx` with `item` at `keys` pinned to `value`, any
    /// other pin of the item set aside, listed nowhere (refcore's `m_solve_value`, `F`).
    /// L §5.18.
    ///
    /// # Errors
    ///
    /// `NotBuilt` for a pin the moves reach where the walk is read; `ENGINE-INVARIANT` for a
    /// freed layer.
    pub(super) fn solve_step(
        &mut self,
        m: &Reality,
        (item, keys): (&str, &[KeyVal]),
        value: Value,
        ctx: Ctx,
    ) -> Result<Ctx, Stop> {
        let mut data = self.layer_data(ctx.layer)?.clone();
        data.pins.retain(|p| p.item != item);
        data.pins.push(Pinned {
            solve: true,
            item: item.to_owned(),
            keys: keys.iter().cloned().map(Some).collect(),
            fields: Vec::new(),
            value,
            text: String::new(),
        });
        let layer = self.intern(data)?;
        self.no_pinned_walk(m, layer)?;
        Ok(Ctx { layer, ..ctx })
    }

    /// Opens a scope holding `layer`: its cells stay while it is open (P §3.5).
    pub(crate) fn open_layer(&mut self, layer: Layer) {
        self.layers.open.push(layer);
    }

    /// Leaves the scope that holds `layer` without freeing anything: a read unwound past the
    /// counted depth through it, and its replay needs the layer's cells (R-16).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a scope closed out of order.
    pub(crate) fn leave_layer(&mut self, layer: Layer) -> Result<(), Stop> {
        if self.layers.open.pop() != Some(layer) {
            return Err(invariant("a layer's scope closed out of order"));
        }
        Ok(())
    }

    /// Closes the scope that holds `layer`: every layer no open scope holds is freed with its
    /// cells (P §3.2, §3.5: freed when its `as if` or `solve` step ends).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a scope closed out of order.
    pub(crate) fn close_layer(&mut self, layer: Layer) -> Result<(), Stop> {
        self.leave_layer(layer)?;
        let mut open = Vec::with_capacity(self.layers.open.len());
        for held in &self.layers.open {
            open.push(self.layer_data(*held)?.clone());
        }
        let mut freed = BTreeSet::new();
        for (at, slot) in self.layers.slots.iter_mut().enumerate().skip(1) {
            let Some(data) = slot else {
                continue;
            };
            if !open.iter().any(|o| data.within(o)) {
                freed.insert(layer_at(at)?);
                *slot = None;
            }
        }
        if freed.is_empty() {
            return Ok(());
        }
        self.layers
            .trimmed
            .retain(|(_, from), to| !freed.contains(from) && !freed.contains(to));
        self.cells.free(&freed);
        Ok(())
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
        set: SetPin<'_>,
        env: &Env,
        ctx: Ctx,
        (how, holes): (Pinning, &mut Vec<Hole>),
    ) -> Result<Option<Pinned>, Stop> {
        let id = set.target.id.as_str();
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
            Some(NameKind::Axis) => return Err(Stop::NotBuilt("an axis pin at run")),
            _ => return Err(invariant("a pin load didn't check")),
        };
        let mut keys = Vec::with_capacity(binds.len());
        let mut texts = Vec::with_capacity(binds.len());
        for (key, bind) in set.target.keys.iter().flatten().zip(binds) {
            let Key::Expr(e) = key else {
                keys.push(None);
                texts.push("*".to_owned());
                continue;
            };
            let given = self.given_key(m, e, env, ctx)?;
            match make_keys(m, id, core::slice::from_ref(bind), vec![given], ctx.cut)? {
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
        for field in &set.target.fields {
            ty = ty.and_then(|t| t.field(field).cloned());
        }
        self.push();
        let v = self.ev(m, set.value, env, ctx);
        let met = self.pop_alone();
        let value = pin_typed(m, v?, ty.as_ref())?;
        if !plain_literal(m, set.value) {
            self.absorb(&met);
        }
        let mut head = id.to_owned();
        if !texts.is_empty() {
            head = format!("{head}[{}]", texts.join(", "));
        }
        for field in &set.target.fields {
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
            fields: set.target.fields.clone(),
            value,
            text,
        }))
    }

    /// Stops `NotBuilt` where a pin of `layer` is on an item the walk reads (a move's, a
    /// place's, a price's, parts' or a fill's cone) and the item holding the pin reads the
    /// timeline: the walk is one per cut, not yet one per pin set (E §3.3; L §5.11
    /// "`flows` inside sees the recomputed movements").
    fn no_pinned_walk(&mut self, m: &Reality, layer: Layer) -> Result<(), Stop> {
        if layer.is_base() {
            return Ok(());
        }
        let cone = match self.layers.timeline.take() {
            Some(cone) => cone,
            None => timeline_cone(m),
        };
        let data = self.layer_data(layer)?;
        let in_cone = |item: &str| item_node(m, item).is_none_or(|node| cone.contains(&node));
        let touches = data.pins.iter().any(|p| in_cone(&p.item))
            || data.wording.iter().any(|(item, _)| in_cone(item));
        self.layers.timeline = Some(cone);
        if !touches {
            return Ok(());
        }
        let reads = match self.holder.clone() {
            None => true,
            Some(holder) => {
                if let Some(known) = self.layers.walkers.get(&holder) {
                    *known
                } else {
                    let reads = reads_timeline(m, &holder);
                    self.layers.walkers.insert(holder, reads);
                    reads
                }
            }
        };
        if reads {
            return Err(Stop::NotBuilt(
                "a pin the moves reach, where the walk is read (a timeline per pin set)",
            ));
        }
        Ok(())
    }
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

/// A value pin as written: its target and its value.
#[derive(Debug, Clone, Copy)]
struct SetPin<'a> {
    /// What is pinned.
    target: &'a crate::syntax::PRef,
    /// Its value.
    value: &'a Expr,
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
