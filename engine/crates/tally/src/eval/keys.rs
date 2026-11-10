//! A read's keys: each key written or computed made a key of its position's index, kind, axis, list or range, and checked against what the cut and the world see.
//! Spec: L §2.5 (keys by position, key literals, a stepped range's keys), §4.2 (keys as
//! values), §4.6 (a day or a period read at a period key), §5.5 (an axis's option), §5.7 and
//! §5.11 (a movement as the key of an event kind), §6.4 (keys a world sees), §7.6 item 3 (a
//! key offset's 18 digits), §8.1 (a read whose key is a hole carries the key's questions
//! only), §8.4 (`RUN-KEY`, `RUN-NONE`).
//! Twin: engine/tools/refcore/evaluate.py (`key_values`, `to_key`, `check_key`) and loader.py
//! (`resolve_key_literal`).
//! Invariants: a key a read takes is a key its domain holds and the cut and the read's world
//! see (L §6.4); a hole key makes the read that hole, built in the hole module.

use super::arith::int_of;
use super::ranges::value_key;
use super::state::{Ctx, Engine, Env, a_day, invariant, num_stop, refuse};
use crate::base::Run;
use crate::hole::Hole;
use crate::load::index_key_of;
use crate::model::{BindInfo, Domain, IndexInfo, KeyVal, Reality, Value};
use crate::syntax::{Lit, Stop};
use crate::time::{Day, PeriodKind};

/// A key a read is given: a value, or a key literal its position resolves (L §2.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Given {
    /// A value.
    Value(Value),
    /// A key literal, resolved in the position's domain.
    Literal(Lit),
}

impl Engine {
    /// The keys of a read that are movements, each checked to be a movement of its position's
    /// event kind or a subkind (refcore's `check_key`, a movement). L §5.7, §5.11.
    ///
    /// # Errors
    ///
    /// `RUN-KEY` for a movement of another kind.
    pub(super) fn movement_keys(
        &mut self,
        m: &Reality,
        binds: &[BindInfo],
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<(), Stop> {
        for (bind, key) in binds.iter().zip(keys) {
            let (Domain::Kind(kind), KeyVal::Mv(mv)) = (&bind.domain, key) else {
                continue;
            };
            if !self.mv_kind_in(m, mv, kind, ctx)? {
                return Err(refuse(
                    Run::Key,
                    format!("{} is not a movement of {kind}", mv.id),
                ));
            }
        }
        Ok(())
    }

    /// The keys of a read: each given key made a key of its position's domain, checked
    /// against the domain, the cut and the world of `ctx`, a stepped range's against its
    /// values with the read's earlier keys bound; a hole key makes the read that hole,
    /// carrying the keys' questions only (refcore's `key_values`). L §2.5, §8.1, §6.4.
    ///
    /// # Errors
    ///
    /// `RUN-KEY`, `RUN-NONE` and the conversions' refusals.
    pub(crate) fn make_keys(
        &mut self,
        m: &Reality,
        item: &str,
        binds: &[BindInfo],
        given: Vec<Given>,
        ctx: Ctx,
    ) -> Result<Result<Vec<KeyVal>, Hole>, Stop> {
        if given.len() != binds.len() {
            return Err(Stop::NotBuilt("a read with fewer or more keys than binds"));
        }
        // a range's ends may read the keys before it (refcore's `benv`)
        let ranged = binds.iter().any(|b| b.domain == Domain::Range);
        let mut earlier = Env::empty();
        let mut keys = Vec::with_capacity(binds.len());
        let mut holes: Vec<Hole> = Vec::new();
        for (bind, written) in binds.iter().zip(given) {
            let made = match written {
                Given::Value(Value::Hole(h)) => Err(h),
                at_range if bind.domain == Domain::Range => {
                    self.range_key(m, bind, at_range, (&earlier, ctx))?
                }
                Given::Value(v) => Ok(to_key(m, item, &v, &bind.domain)?),
                Given::Literal(lit) => {
                    Ok(literal_key(m, &lit, &bind.domain)?.ok_or_else(lacks_at_run)?)
                }
            };
            let key = match made {
                Ok(key) => key,
                Err(h) => {
                    // a later range end that reads this key reads its hole (L §8.1)
                    if ranged {
                        earlier.bind(&bind.var, Value::Hole(h.clone()), Some(bind.domain.clone()));
                    }
                    holes.push(h);
                    continue;
                }
            };
            check_key(m, &key, &bind.domain, (ctx.cut, ctx.world))?;
            if ranged {
                earlier.bind(&bind.var, Value::of_key(&key), Some(bind.domain.clone()));
            }
            keys.push(key);
        }
        let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
        if let Some(hole) = crate::hole::spread(&operands) {
            return Ok(Err(hole));
        }
        Ok(Ok(keys))
    }
}

/// A value made a key of `domain`, an axis's option included, a stepped range's value as its
/// key, unchecked against the range (refcore's `to_key`). L §2.5, §4.2, §5.5.
///
/// # Errors
///
/// `RUN-NONE` for `none`, `RUN-KEY` for a value the domain doesn't hold; `NotBuilt` for a key
/// list.
pub(crate) fn to_key(m: &Reality, item: &str, v: &Value, domain: &Domain) -> Result<KeyVal, Stop> {
    if *v == Value::None {
        return Err(refuse(Run::None, "a key is none here: a key".to_owned()));
    }
    if let Value::OutKey { index, text } = v {
        return Err(refuse(Run::Key, format!("{text} is outside {index}")));
    }
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            index_key(m, item, ix, v)
        }
        Domain::Kind(kind) => {
            if let Value::Mv(mv) = v {
                // a key of an event kind is a movement; its kind is checked where the walk
                // can find it (`movement_keys`, L §5.7, §5.11)
                return Ok(KeyVal::Mv(mv.clone()));
            }
            let Value::Key(KeyVal::Thing(name)) = v else {
                return Err(refuse(
                    Run::Key,
                    format!("{} is not a key of {kind}", v.text(m).map_err(num_stop)?),
                ));
            };
            if !m.thing_in(name, kind) {
                return Err(refuse(Run::Key, format!("{name} is not a key of {kind}")));
            }
            Ok(KeyVal::Thing(name.clone()))
        }
        Domain::Axis(axis) => {
            // the axis id in a key position reads at the current option (L §5.5)
            let Value::Key(key @ KeyVal::Key { index, name }) = v else {
                let shown = v.text(m).map_err(num_stop)?;
                return Err(refuse(Run::Key, format!("{shown} is not a key of {axis}")));
            };
            if index != axis {
                return Err(refuse(Run::Key, format!("{name} is not a key of {axis}")));
            }
            if !axis_has(m, axis, name) {
                return Err(invariant("an axis's option it lacks"));
            }
            Ok(key.clone())
        }
        Domain::Range => match value_key(m, v)? {
            Some(key) => Ok(key),
            None => Err(refuse(
                Run::Key,
                format!(
                    "{} is not a value of a stepped range",
                    v.text(m).map_err(num_stop)?
                ),
            )),
        },
        Domain::List(_) => Err(Stop::NotBuilt("a key of a key list")),
    }
}

/// A value made a key of index `ix`: an int of an integer index, the key holding a day or a
/// period of a period index, a key of a list or open index as it is (refcore's `to_key`);
/// an integer past 64 bits is no key of an integer index, whose keys have at most 18 digits
/// (L §7.6 item 3), `RUN-KEY`.
fn index_key(m: &Reality, item: &str, ix: &IndexInfo, v: &Value) -> Result<KeyVal, Stop> {
    if ix.is_int {
        if let Value::Number(n) = v
            && n.q.to_i64().is_none()
            && let Some(text) = n.q.integer_text()
        {
            return Err(refuse(Run::Key, format!("{text} is outside {}", ix.id)));
        }
        return Ok(KeyVal::Int(int_of(m, v)?));
    }
    if ix.period.is_some() {
        return period_key(m, item, ix, v);
    }
    if let Value::Key(key @ KeyVal::Key { .. }) = v {
        return Ok(key.clone());
    }
    Err(refuse(
        Run::Key,
        format!("{} is outside {}", v.text(m).map_err(num_stop)?, ix.id),
    ))
}

/// The key of a period index a day or a period stands for: the key holding the day, the
/// key equal to the period or holding it (refcore's `to_key`, `holding` and `period`); the
/// end of time is no day a key holds, `RUN-LIMIT`. L §4.6, §7.6 item 1.
fn period_key(m: &Reality, item: &str, ix: &IndexInfo, v: &Value) -> Result<KeyVal, Stop> {
    let outside = |text: String| refuse(Run::Key, format!("{text} is outside {}", ix.id));
    if let Value::Day(day) = v {
        let day = a_day(*day)?;
        return ix.holding(day).cloned().ok_or_else(|| {
            let def_days = m.defs.get(item).is_some() && ix.period == Some(PeriodKind::Day);
            if def_days {
                refuse(
                    Run::Key,
                    format!("{} is outside the days of {item}", day.text()),
                )
            } else {
                outside(day.text())
            }
        });
    }
    if let Value::Period(p) = v {
        for key in ix.keys() {
            match key {
                KeyVal::Day(d) if p.kind() == PeriodKind::Day && *d == p.start() => {
                    return Ok(key.clone());
                }
                KeyVal::Period(k) if k == p || (k.start() <= p.start() && p.end() <= k.end()) => {
                    return Ok(key.clone());
                }
                KeyVal::Day(_)
                | KeyVal::Period(_)
                | KeyVal::Key { .. }
                | KeyVal::Thing(_)
                | KeyVal::Int(_)
                | KeyVal::Mv(_)
                | KeyVal::Value(_) => {}
            }
        }
        return Err(outside(p.label()));
    }
    Err(outside(v.text(m).map_err(num_stop)?))
}

/// A key literal resolved in `domain`: a key of an index, an integer or a period by its
/// label or a key by its text, a thing of a kind by its name, an option of an axis (refcore's
/// `resolve_key_literal`); `None` for a literal the domain doesn't hold, which its caller
/// refuses. L §2.5, §5.5.
///
/// # Errors
///
/// An invariant for an index load didn't build; `NotBuilt` for a key list or a range.
pub(super) fn literal_key(m: &Reality, lit: &Lit, domain: &Domain) -> Result<Option<KeyVal>, Stop> {
    let text = lit.text();
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            Ok(index_key_of(ix, lit))
        }
        Domain::Kind(kind) => Ok(m.thing_in(&text, kind).then_some(KeyVal::Thing(text))),
        Domain::Axis(axis) => Ok(axis_has(m, axis, &text).then_some(KeyVal::Key {
            index: axis.clone(),
            name: text,
        })),
        Domain::List(_) | Domain::Range => {
            Err(Stop::NotBuilt("a key literal of a key list or a range"))
        }
    }
}

/// Whether axis `axis` has an option named `name` (refcore's `k in m.axes[ax].options`).
/// L §5.5.
fn axis_has(m: &Reality, axis: &str, name: &str) -> bool {
    option_at(m, axis, name).is_some()
}

/// The place of option `name` among axis `axis`'s, in the order written (refcore's
/// `m.axes[ax].options.index`); none for an axis or an option the reality lacks. L §5.5.
pub(super) fn option_at(m: &Reality, axis: &str, name: &str) -> Option<usize> {
    let options = &m.axes.get(axis)?.options;
    options.iter().position(|(option, _)| option == name)
}

/// Whether the cut and the world of `seen` see key `key` of `domain`: a key its index holds
/// and no version known after the cut or unseen in the world added, a thing an entry they
/// see brings (refcore's `check_key`, `IndexInfo.visible`). L §4.5, §6.4.
///
/// # Errors
///
/// `RUN-KEY`.
pub(super) fn check_key(
    m: &Reality,
    key: &KeyVal,
    domain: &Domain,
    (cut, world): (Day, crate::model::World),
) -> Result<(), Stop> {
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            if ix.pos(key).is_none() {
                return Err(refuse(Run::Key, format!("{} is outside {id}", key.text())));
            }
            if let Some(version) = ix.added_by(key) {
                let known = m
                    .versions
                    .get(version)
                    .map(|v| v.known)
                    .ok_or_else(|| invariant("a key added by a version load didn't build"))?;
                if known > cut || !m.scenarios.sees(world, Some(version)) {
                    return Err(refuse(
                        Run::Key,
                        format!("{} was added after the cut", key.text()),
                    ));
                }
            }
            Ok(())
        }
        Domain::Kind(kind) => {
            if let KeyVal::Mv(_) = key {
                return Ok(());
            }
            let KeyVal::Thing(name) = key else {
                return Err(refuse(
                    Run::Key,
                    format!("{} is not a key of {kind}", key.text()),
                ));
            };
            let thing = m
                .things
                .get(name)
                .ok_or_else(|| refuse(Run::Key, format!("{name} is not a key of {kind}")))?;
            if !thing_seen(m, thing, (cut, world)) {
                return Err(refuse(Run::Key, format!("{name} was added after the cut")));
            }
            Ok(())
        }
        Domain::List(_) | Domain::Range | Domain::Axis(_) => Ok(()),
    }
}

/// Whether the cut and the world of `seen` see an entry of `thing`: one with no version, or
/// whose version is known by the cut, in a file the world sees (refcore's `thing_seen`,
/// `in_world`). L §4.5, §5.7, §6.4.
#[must_use]
pub(crate) fn thing_seen(
    m: &Reality,
    thing: &crate::model::Thing,
    (cut, world): (Day, crate::model::World),
) -> bool {
    if let Some(keys) = &thing.place {
        let own = thing
            .entries
            .first()
            .is_none_or(|e| m.entry_seen(thing, e, world));
        return own
            && keys
                .key_things
                .iter()
                .filter_map(|name| m.things.get(name))
                .all(|t| thing_seen(m, t, (cut, world)));
    }
    for entry in &thing.entries {
        if !m.entry_seen(thing, entry, world) {
            continue;
        }
        let Some(version) = &entry.version else {
            return true;
        };
        if m.versions.get(version).is_some_and(|v| v.known <= cut) {
            return true;
        }
    }
    false
}

/// A literal key its index or kind lacks: load refuses it with its place (`LOAD-KEY`), and a
/// key a version adds after the cut is `RUN-KEY` before it comes here (L §2.5, §5.6).
pub(super) fn lacks_at_run() -> Stop {
    invariant("a literal key its index or kind lacks: load refuses it first (LOAD-KEY)")
}
