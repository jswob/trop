//! Load's observations: an observed input's rows as real movements, with their day, kind, legs, attributes and what they fulfil.
//! Spec: L §5.19 (observed inputs: `on`, `kind`, legs, attributes, `fulfils`; LOAD-FIELD,
//! LOAD-LEG, LOAD-PRECISION), §5.14.3 (adjustments are the engine's; `adjustments[m[k].leg, V]`),
//! §4.5 (a row's own `known`).
//! Twin: engine/tools/refcore/loader.py (`obs_item`, `obs_fields`, `obs_amount`, `obs_fulfils`,
//! `adj_ref`, `move_keys`).
//! Invariants: an observation's legs are its kind's, in the kind's order, each between places
//! that hold its unit, its amount exact and, in a currency, whole minor units; its attributes are
//! its kind's, kept as written for the evaluator to type.

use super::keys::{KeyLit, lit_text, resolve_key};
use super::places::Written;
use super::steps::{Loader, num_stop, refuse};
use crate::base::Load;
use crate::model::{Fulfils, KeyVal, KindSpec, NameKind, ObsLeg, Observation, Unit};
use crate::syntax::{
    Certainty, Expr, Field, FieldValue, Key, Known, Leg, Lit, Postfix, Row, Stop, Value,
};
use crate::time::Day;

/// What a message names an observation by, and where it was written.
struct At {
    /// `bank[r20260901a]`.
    sid: String,
    /// The file.
    file: String,
    /// The line.
    line: u32,
}

impl At {
    /// A refusal at the observation's line.
    fn refuse(&self, code: Load, message: String) -> Stop {
        refuse(code, message, &self.file, self.line)
    }
}

impl Loader {
    /// A row of an observed input (refcore's `obs_item`). L §5.19.
    pub(super) fn obs_item(
        &mut self,
        vid: &str,
        id: &str,
        row: &Row,
        file: &str,
        line: u32,
    ) -> Result<bool, Stop> {
        if row.given {
            return Err(refuse(Load::Name, format!("{id} is not a def"), file, line));
        }
        let key = match row.keys.as_deref() {
            Some(
                [crate::syntax::RowKey::Lit(lit @ (Lit::Name(_) | Lit::Key(_) | Lit::Num(_)))],
            ) => lit_text(lit),
            Some(_) | None => {
                return Err(refuse(
                    Load::Key,
                    format!("an observation row states its id: {id}[<id>]"),
                    file,
                    line,
                ));
            }
        };
        if row.governs.is_some() || row.instant.is_some() {
            return Err(refuse(
                Load::Parse,
                "cannot parse: an observation row states no governs or instant".to_owned(),
                file,
                line,
            ));
        }
        let at = At {
            sid: format!("{id}[{key}]"),
            file: file.to_owned(),
            line,
        };
        let own_known = self.obs_known(vid, id, &key, row.known.as_ref(), &at)?;
        let fields = match &row.value {
            Value::Record(fields) => Some(fields.as_slice()),
            Value::None => None,
            Value::Hole(_)
            | Value::Never
            | Value::Str(_)
            | Value::List(_)
            | Value::Calendar(_)
            | Value::Days(_)
            | Value::Lit(_)
            | Value::Expr(_) => {
                return Err(at.refuse(
                    Load::Field,
                    format!("an observation is a record with named fields: {}", at.sid),
                ));
            }
        };
        let mut observation = Observation {
            input: id.to_owned(),
            key,
            version: vid.to_owned(),
            own_known,
            locator: row.locator.clone(),
            replace: row.replace,
            removed: fields.is_none(),
            assumed: row.assumed.clone(),
            day: None,
            kind: None,
            legs: Vec::new(),
            attrs: Vec::new(),
            fulfils: None,
            line,
        };
        if let Some(written) = fields {
            self.obs_fields(&mut observation, written, &at)?;
        }
        if let Some(input) = self.m.inputs.get_mut(id) {
            input.observations.push(observation);
        }
        Ok(true)
    }

    /// An observation's own `known`. L §4.5.
    fn obs_known(
        &self,
        vid: &str,
        id: &str,
        key: &str,
        known: Option<&Known>,
        at: &At,
    ) -> Result<Option<Day>, Stop> {
        let Some(written) = known else {
            return Ok(None);
        };
        let Known::Day(day) = written else {
            return Err(Stop::NotBuilt("a known window"));
        };
        if self.m.versions.get(vid).is_some_and(|v| *day < v.known) {
            return Err(at.refuse(
                Load::Known,
                format!("row {id}[{key}] is known before its version {vid}"),
            ));
        }
        if *day > self.m.on {
            return Err(at.refuse(
                Load::Known,
                format!("row {id}[{key}] is known after the model's on date"),
            ));
        }
        Ok(Some(*day))
    }

    /// An observation's fields: its day, kind, `fulfils`, legs and attributes (refcore's
    /// `obs_fields`). L §5.19.
    fn obs_fields(
        &self,
        observation: &mut Observation,
        written: &[Field],
        at: &At,
    ) -> Result<(), Stop> {
        let sid = &at.sid;
        let mut legs: Vec<&Leg> = Vec::new();
        let mut fields: Vec<(String, FieldValue)> = Vec::new();
        for field in written {
            match field {
                Field::Leg(leg) => {
                    if legs.iter().any(|l| l.name == leg.name) {
                        return Err(
                            at.refuse(Load::Leg, format!("leg {} twice in {sid}", leg.name))
                        );
                    }
                    legs.push(leg.as_ref());
                }
                Field::Named(name, value) => {
                    if fields.iter().any(|(n, _)| n == name) {
                        return Err(at.refuse(Load::Field, format!("field {name} twice in {sid}")));
                    }
                    fields.push((name.clone(), value.clone()));
                }
            }
        }
        let on = take_field(&mut fields, "on").ok_or_else(|| {
            at.refuse(
                Load::Field,
                format!("on is a field of an observation; {sid} has none"),
            )
        })?;
        let kind_written = take_field(&mut fields, "kind").ok_or_else(|| {
            at.refuse(
                Load::Field,
                format!("kind is a field of an observation; {sid} has none"),
            )
        })?;
        observation.day = Some(field_day(&on, at, &format!("the on of {sid}"))?);
        let kind = self.obs_kind(&kind_written, at)?;
        if let Some(fulfils) = take_field(&mut fields, "fulfils") {
            observation.fulfils = Some(self.obs_fulfils(&fulfils, at)?);
        }
        let declared = self.m.kind_legs(&kind);
        for leg in &legs {
            if !declared.iter().any(|(name, _, _)| *name == leg.name) {
                return Err(at.refuse(
                    Load::Leg,
                    format!("leg {} is not a leg of {kind} ({sid})", leg.name),
                ));
            }
        }
        for (name, unit, _) in &declared {
            let Some(leg) = legs.iter().find(|l| l.name == *name) else {
                return Err(at.refuse(Load::Leg, format!("{sid} lacks leg {name}")));
            };
            let from = self.place_ref(&leg.from, Written::Place, &at.file, at.line)?;
            let to = self.place_ref(&leg.to, Written::Place, &at.file, at.line)?;
            let (amount, amount_unit) = self.obs_amount(&leg.amount, name, unit, &from, &to, at)?;
            let day = leg
                .on
                .as_ref()
                .map(|d| expr_day(d, at, &format!("the on of leg {name} of {sid}")))
                .transpose()?;
            observation.legs.push(ObsLeg {
                name: name.clone(),
                from,
                to,
                amount,
                unit: amount_unit,
                day,
            });
        }
        let attrs = self.m.kind_attrs(&kind);
        for (name, _) in &fields {
            if !attrs.iter().any(|(a, _)| a == name) {
                return Err(at.refuse(
                    Load::Field,
                    format!("{name} is not a field of {kind} ({sid})"),
                ));
            }
        }
        for (name, _) in &attrs {
            let Some((_, value)) = fields.iter().find(|(n, _)| n == name) else {
                return Err(at.refuse(
                    Load::Leg,
                    format!("attribute {name} is missing or unknown ({sid})"),
                ));
            };
            observation.attrs.push((name.clone(), value.clone()));
        }
        observation.kind = Some(kind);
        Ok(())
    }

    /// An observation's event kind: declared, certain, never `adjustments`. L §5.19, §5.14.3.
    fn obs_kind(&self, written: &FieldValue, at: &At) -> Result<String, Stop> {
        let sid = &at.sid;
        let name = match written {
            FieldValue::Expr(Expr::Name(name)) | FieldValue::Lit(Lit::Name(name)) => {
                Some(name.clone())
            }
            FieldValue::Expr(_) | FieldValue::Lit(_) | FieldValue::Hole(_) | FieldValue::None => {
                None
            }
        };
        let Some(id) = name.filter(|n| self.m.names.get(n) == Some(&NameKind::Kind)) else {
            return Err(at.refuse(Load::Name, format!("unknown event kind in {sid}")));
        };
        let Some(kind) = self.m.kinds.get(&id) else {
            return Err(Stop::NotBuilt("a kind that isn't built"));
        };
        if kind.spec != KindSpec::Event {
            return Err(at.refuse(Load::Kind, format!("{id} is not an event kind ({sid})")));
        }
        if self.m.chain(&id).iter().any(|k| k.id == "adjustments") {
            return Err(at.refuse(
                Load::Kind,
                format!("{sid} of {id}: only the engine makes adjustments"),
            ));
        }
        if kind.certainty == Some(Certainty::Uncertain) {
            return Err(at.refuse(
                Load::Leg,
                format!("observation of uncertain kind {id} ({sid})"),
            ));
        }
        Ok(id)
    }

    /// An observed leg's amount, in the leg's unit and its places' (refcore's `obs_amount`).
    /// L §5.19.
    fn obs_amount(
        &self,
        written: &Expr,
        leg: &str,
        leg_unit: &str,
        from: &str,
        to: &str,
        at: &At,
    ) -> Result<(crate::num::Num, String), Stop> {
        let sid = &at.sid;
        let Some(number) = written_amount(written) else {
            return Err(at.refuse(
                Load::Leg,
                format!("the amount of leg {leg} of {sid} states its unit"),
            ));
        };
        let terms: Vec<(String, i64)> = number
            .unit
            .iter()
            .flat_map(|u| u.terms.iter())
            .map(|t| (t.base.clone(), t.power))
            .collect();
        let (unit, factor) = Unit::from_syntax(terms).map_err(num_stop)?;
        let [(base, 1)] = unit.terms() else {
            return Err(at.refuse(
                Load::Leg,
                format!("leg {leg} of {sid}: {} is not a unit of a leg", number.text),
            ));
        };
        let mut fits = if leg_unit == "any" {
            self.m.currencies.contains_key(base)
                || self.m.units.contains(base)
                || self.m.unit_kind_keys.contains_key(base)
        } else if self.m.currencies.contains_key(leg_unit) || self.m.units.contains(leg_unit) {
            base == leg_unit
        } else {
            self.m
                .unit_kind_keys
                .get(base)
                .is_some_and(|k| self.m.subtree(leg_unit).contains(k))
        };
        for place in [from, to] {
            let held = self.m.things.get(place).and_then(|t| self.m.place_unit(t));
            let ok = held.is_some_and(|p| p == "any" || p == *base);
            fits = fits && ok;
        }
        if !fits {
            return Err(at.refuse(
                Load::Leg,
                format!("leg {leg} of {sid} is of the wrong unit ({})", number.text),
            ));
        }
        let amount = number.value.checked_mul(&factor).map_err(num_stop)?;
        if let Some(minor) = self.m.currencies.get(base) {
            let whole = amount.checked_div(minor).map_err(num_stop)?.is_integer();
            if !whole {
                return Err(at.refuse(
                    Load::Precision,
                    format!("{} is finer than the minor unit", number.text),
                ));
            }
        }
        Ok((amount, base.clone()))
    }

    /// What an observation fulfils: a planned instance, or an adjustment (refcore's
    /// `obs_fulfils`). L §5.19, §5.14.3.
    fn obs_fulfils(&self, written: &FieldValue, at: &At) -> Result<Fulfils, Stop> {
        let sid = &at.sid;
        let FieldValue::Expr(expr) = written else {
            return Err(at.refuse(
                Load::Kind,
                format!("fulfils on a value ({sid}), not a movement"),
            ));
        };
        let Some((name, keys)) = named_keys(expr) else {
            return Err(at.refuse(
                Load::Kind,
                format!("fulfils on a field ({sid}), not a movement"),
            ));
        };
        let Some(kind) = self.m.names.get(&name) else {
            return Err(at.refuse(Load::Name, format!("unknown id {name}")));
        };
        if *kind == NameKind::Kind && name == "adjustments" && !keys.is_empty() {
            return self.adj_ref(&keys, at);
        }
        if *kind != NameKind::Move {
            return Err(at.refuse(
                Load::Kind,
                format!(
                    "fulfils on a {} {name} ({sid}), not a movement",
                    name_word(*kind)
                ),
            ));
        }
        let keys = self.move_keys(&name, &keys, at)?;
        Ok(Fulfils::Instance(name, keys))
    }

    /// `adjustments[m[keys].leg, V]` (refcore's `adj_ref`). L §5.14.3.
    fn adj_ref(&self, keys: &[Expr], at: &At) -> Result<Fulfils, Stop> {
        let sid = &at.sid;
        let shape = || {
            at.refuse(
                Load::Key,
                format!(
                    "an adjustment is named adjustments[<move>[keys].<leg>, <version>] ({sid})"
                ),
            )
        };
        let [leg_ref, Expr::Name(version)] = keys else {
            return Err(shape());
        };
        let Expr::Post(base, posts) = leg_ref else {
            return Err(shape());
        };
        let Expr::Name(name) = base.as_ref() else {
            return Err(at.refuse(
                Load::Kind,
                format!("fulfils on an expression ({sid}), not a movement"),
            ));
        };
        let (move_keys, leg): (Vec<Expr>, &String) = match posts.as_slice() {
            [Postfix::Field(leg)] => (Vec::new(), leg),
            [Postfix::Keys(written), Postfix::Field(leg)] => {
                (key_exprs(written).ok_or_else(shape)?, leg)
            }
            _ => return Err(shape()),
        };
        let Some(kind) = self.m.names.get(name) else {
            return Err(at.refuse(Load::Name, format!("unknown id {name}")));
        };
        if *kind != NameKind::Move {
            return Err(at.refuse(
                Load::Kind,
                format!(
                    "fulfils on a {} {name} ({sid}), not a movement",
                    name_word(*kind)
                ),
            ));
        }
        let move_kind = self.m.moves.get(name).map(|m| m.kind.clone());
        if let Some(of) = move_kind {
            let legs = self.m.kind_legs(&of);
            if !legs.is_empty() && !legs.iter().any(|(l, _, _)| l == leg) {
                return Err(at.refuse(Load::Leg, format!("leg {leg} is not a leg of {of}")));
            }
        }
        if self.m.names.get(version) != Some(&NameKind::Version) {
            return Err(at.refuse(Load::Name, format!("unknown version {version}")));
        }
        let values = self.move_keys(name, &move_keys, at)?;
        Ok(Fulfils::Adjustment(
            name.clone(),
            values,
            leg.clone(),
            version.clone(),
        ))
    }

    /// A move instance's written keys (refcore's `move_keys`). L §2.5.
    fn move_keys(&self, id: &str, keys: &[Expr], at: &At) -> Result<Vec<KeyVal>, Stop> {
        let Some(info) = self.m.moves.get(id) else {
            return Err(Stop::NotBuilt("a move that isn't built"));
        };
        if keys.len() != info.binds.len() {
            return Err(at.refuse(Load::Key, format!("wrong number of keys for {id}")));
        }
        let mut out = Vec::new();
        for (key, bind) in keys.iter().zip(&info.binds) {
            let Some(written) = KeyLit::of_expr(key) else {
                return Err(Stop::NotBuilt("a computed key of a move instance"));
            };
            out.push(resolve_key(
                &self.m,
                &written,
                &bind.domain,
                &at.file,
                at.line,
            )?);
        }
        Ok(out)
    }
}

/// Takes a named field out of the list.
fn take_field(fields: &mut Vec<(String, FieldValue)>, name: &str) -> Option<FieldValue> {
    let value = fields
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.clone())?;
    fields.retain(|(n, _)| n != name);
    Some(value)
}

/// A leg's amount as written: a number with its unit.
fn written_amount(written: &Expr) -> Option<&crate::syntax::Number> {
    let Expr::Lit(Lit::Num(number)) = written else {
        return None;
    };
    number.unit.is_some().then_some(number)
}

/// A field's day: a written date, never a window. L §5.19.
fn field_day(value: &FieldValue, at: &At, what: &str) -> Result<Day, Stop> {
    match value {
        FieldValue::Lit(Lit::Day(day)) => Ok(*day),
        FieldValue::Expr(expr) => expr_day(expr, at, what),
        FieldValue::Lit(_) | FieldValue::Hole(_) | FieldValue::None => {
            Err(at.refuse(Load::Unit, format!("{what} is a day")))
        }
    }
}

/// An expression's day: a written date; a window belongs in an attribute (LOAD-LEG).
fn expr_day(expr: &Expr, at: &At, what: &str) -> Result<Day, Stop> {
    if let Expr::Lit(Lit::Day(day)) = expr {
        return Ok(*day);
    }
    if let Expr::Call(call) = expr
        && (call.name == "window" || call.name == "one_of")
    {
        return Err(at.refuse(
            Load::Leg,
            format!("{what} is a window; a window belongs in an attribute"),
        ));
    }
    Err(at.refuse(Load::Unit, format!("{what} is a day")))
}

/// `m[keys]` or `m`: the name and its key expressions.
fn named_keys(expr: &Expr) -> Option<(String, Vec<Expr>)> {
    if let Expr::Name(name) = expr {
        return Some((name.clone(), Vec::new()));
    }
    let Expr::Post(base, posts) = expr else {
        return None;
    };
    let Expr::Name(name) = base.as_ref() else {
        return None;
    };
    let [Postfix::Keys(keys)] = posts.as_slice() else {
        return None;
    };
    Some((name.clone(), key_exprs(keys)?))
}

/// Key positions that are all expressions, none `*`.
fn key_exprs(keys: &[Key]) -> Option<Vec<Expr>> {
    let mut out = Vec::new();
    for key in keys {
        let Key::Expr(expr) = key else {
            return None;
        };
        out.push(expr.clone());
    }
    Some(out)
}

/// The word a message names a namespace entry by (refcore's `names` kinds).
fn name_word(kind: NameKind) -> &'static str {
    match kind {
        NameKind::Unit => "unit",
        NameKind::Kind => "kind",
        NameKind::Index => "index",
        NameKind::Axis => "axis",
        NameKind::Input => "input",
        NameKind::Def => "def",
        NameKind::Band => "band",
        NameKind::Place => "place",
        NameKind::Fill => "fill",
        NameKind::Move => "move",
        NameKind::Version => "version",
        NameKind::Ask => "ask",
        NameKind::Source => "source",
        NameKind::Cut => "cut",
    }
}
