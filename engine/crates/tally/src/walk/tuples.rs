//! The plan pass's binding tuples: a move's keys bound depth first at its law date, and the tuple and places a link names.
//! Spec: E §4.1 steps 1 and 2 (enumerate a move's binding tuples at the cut, each binding read
//! at the move's law date with the keys bound before it); L §5.14 Keys (a member whose `where`
//! is a hole is a hole tuple; one whose `where` is refused ends its tuple there, a refused one;
//! tally 2.1.6), §4.3 (a move's law date by its `at`); L §5.19 Links (the tuple of a move at
//! its keys under a cut, its `when` read at the cut, and the places its legs give).
//! Twin: engine/tools/refcore/moves.py (`move_tuples` with its `go`, `move_law`,
//! `link_tuple`, `link_places`).
//! Invariants: tuples come in binding order, depth first; what deciding a tuple read goes with
//! its instance, never to the reader of the move's instances; a link whose `when` is a hole or
//! refused is taken as existing and judged where its instance is.

use super::plan::{Exists, date_of, invariant, join, refuse};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, Member, reads_name};
use crate::hole::Hole;
use crate::model::{Domain, KeyVal, MoveInfo, Reality, Value};
use crate::syntax::Stop;
use crate::time::Day;

/// A binding tuple of a move: its scope, keys, and the hole of a `where` that is one, or of a
/// domain or law date that is one, and what deciding it read (refcore's `move_tuples`
/// statuses and colls).
pub(super) struct Tuple {
    /// The scope.
    pub(super) env: Env,
    /// The keys.
    pub(super) keys: Vec<KeyVal>,
    /// Whether it exists.
    pub(super) exists: Exists,
    /// The hole of a domain or of a law date: the instance's day and existence are that hole.
    pub(super) domain_hole: Option<Hole>,
    /// The refusal its last key's `where` met: a refused instance, its later keys unbound
    /// where it isn't the move's last binding (L §5.14 Keys, §8.4; tally 2.1.6).
    pub(super) refused: Option<Refusal>,
    /// What deciding it read: its law dates, domains, members and `where`s, which go with its
    /// instance, not to the reader of the move's instances.
    pub(super) coll: Collected,
}

impl Engine {
    /// A move's binding tuples at the cut, depth first in binding order, each binding read at
    /// the move's law date with the keys bound before it, each tuple with what deciding it
    /// read, kept from the reader; a member whose `where` is refused ends its tuple there, a
    /// refused one (refcore's `move_tuples`). E §4.1 step 1, L §5.14 Keys (tally 2.1.6).
    pub(super) fn move_tuples(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        ctx: Ctx,
    ) -> Result<Vec<Tuple>, Stop> {
        let mut out = Vec::new();
        let mut work = vec![Tuple {
            env: Env::empty(),
            keys: Vec::new(),
            exists: Exists::Yes,
            domain_hole: None,
            refused: None,
            coll: Collected::default(),
        }];
        while let Some(tuple) = work.pop() {
            if tuple.refused.is_some() {
                out.push(tuple);
                continue;
            }
            let Some(bind) = mv.binds.get(tuple.keys.len()) else {
                out.push(tuple);
                continue;
            };
            self.push();
            let law = self.move_law(m, mv, &tuple.env, ctx);
            let mut decided = tuple.coll.clone();
            decided.merge(&self.pop_alone());
            let law = match law? {
                Law::Hole(h) => {
                    out.push(Tuple {
                        domain_hole: Some(h),
                        coll: decided,
                        ..tuple
                    });
                    continue;
                }
                Law::Day(d) => Some(d),
                Law::None => None,
            };
            let members = match self.binding_members(m, &bind.node, &tuple.env, ctx.at_law(law))? {
                Err((h, read)) => {
                    decided.merge(&read);
                    out.push(Tuple {
                        domain_hole: Some(h),
                        coll: decided,
                        ..tuple
                    });
                    continue;
                }
                Ok(members) => members,
            };
            let mut next = Vec::with_capacity(members.len());
            for (member, read, refused) in members {
                let mut extended = extend(m, mv, &tuple, &bind.var, &bind.domain, member)?;
                extended.coll = decided.clone();
                extended.coll.merge(&read);
                extended.refused = refused;
                next.push(extended);
            }
            work.extend(next.into_iter().rev());
        }
        Ok(out)
    }

    /// A move's law date by its `at`, with the keys bound so far: none while it reads a key
    /// not yet bound (refcore's `move_law`). L §4.3, §5.14.
    pub(super) fn move_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Law, Stop> {
        let Some(at) = &mv.decl.at else {
            return Ok(Law::None);
        };
        if mv
            .binds
            .iter()
            .any(|b| env.get(&b.var).is_none() && reads_name(at, &b.var))
        {
            return Ok(Law::None);
        }
        let v = self.ev(m, at, env, ctx.at_law(None))?;
        if let Value::Hole(h) = v {
            return Ok(Law::Hole(h));
        }
        if v == Value::None {
            return Err(refuse(
                Run::None,
                format!("the law date of {} is none here", mv.id),
            ));
        }
        Ok(Law::Day(date_of(&v)?))
    }

    /// The binding tuple of `mv` at `keys` under cut `cut`, its `when` read at the cut: none
    /// when no tuple has those keys or its `when` is false; a `when` that is a hole or refused,
    /// or a refused `where`, is taken as existing, its link judged where its instance is
    /// (refcore's `link_tuple`).
    /// L §5.19 Links, E §4.1 step 2.
    ///
    /// # Errors
    ///
    /// The refusals the binding tuples meet; a load refusal met at run.
    pub(crate) fn link_tuple(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<Option<Env>, Stop> {
        let tctx = self.timeline_ctx(None, cut);
        for tuple in self.move_tuples(m, mv, tctx)? {
            if tuple.keys != keys {
                continue;
            }
            if tuple.exists == Exists::No {
                return Ok(None);
            }
            if let (Some(when), Exists::Yes, None, None) = (
                &mv.decl.when,
                &tuple.exists,
                &tuple.domain_hole,
                &tuple.refused,
            ) {
                match self.when_at_cut(m, mv, when, &tuple.env, tctx) {
                    Ok(false) => return Ok(None),
                    Ok(true) => {}
                    Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {}
                    Err(stop) => return Err(stop),
                }
            }
            return Ok(Some(tuple.env));
        }
        Ok(None)
    }

    /// Whether a link's instance may exist by its `when` at the ask's cut: false only for a
    /// `when` that is false (refcore's `link_tuple`, its `when`). L §5.19.
    fn when_at_cut(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        when: &crate::syntax::Expr,
        env: &Env,
        tctx: Ctx,
    ) -> Result<bool, Stop> {
        let law = match self.move_law(m, mv, env, tctx)? {
            Law::Hole(_) => return Ok(true),
            Law::Day(d) => Some(d),
            Law::None => None,
        };
        let w = self.ev(m, when, env, tctx.at_law(law))?;
        Ok(w != Value::Bool(false))
    }

    /// The places the legs of `mv`'s instance at `env` give at the cut, no amount read; none
    /// where a place can't be read here, so the link is judged by its keys alone (refcore's
    /// `link_places`). L §5.19 Links.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    pub(crate) fn link_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        env: &Env,
        cut: Day,
    ) -> Result<std::collections::BTreeSet<String>, Stop> {
        let tctx = self.timeline_ctx(None, cut);
        let none = std::collections::BTreeSet::new();
        let law = match self.move_law(m, mv, env, tctx) {
            Ok(Law::Hole(_)) => return Ok(none),
            Ok(Law::Day(d)) => Some(d),
            Ok(Law::None) => None,
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => return Ok(none),
            Err(stop) => return Err(stop),
        };
        let mut places = std::collections::BTreeSet::new();
        for leg in &mv.decl.legs {
            for e in [&leg.from, &leg.to] {
                match self.ev(m, e, env, tctx.at_law(law)) {
                    Ok(Value::Key(KeyVal::Thing(name))) => {
                        places.insert(name);
                    }
                    Ok(_) => return Ok(none),
                    Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                        return Ok(none);
                    }
                    Err(stop) => return Err(stop),
                }
            }
        }
        Ok(places)
    }
}

/// A move's law date: none, a day, or a hole.
pub(super) enum Law {
    /// No `at`, or one that reads a key not yet bound.
    None,
    /// A day.
    Day(Day),
    /// A hole.
    Hole(Hole),
}

/// A tuple extended by one binding's member: its variable bound, its key made a key of the
/// binding's domain, its hole joined (refcore's `move_tuples`, `go`). L §5.14.
fn extend(
    m: &Reality,
    mv: &MoveInfo,
    tuple: &Tuple,
    var: &str,
    domain: &Domain,
    member: Member,
) -> Result<Tuple, Stop> {
    let value = member
        .env
        .get(var)
        .map(|v| v.value.clone())
        .ok_or_else(|| invariant("a member without its variable"))?;
    let listed = matches!(domain, Domain::List(_) | Domain::Range);
    let key = if let Value::Key(k) = &value {
        k.clone()
    } else if let Value::Mv(r) = &value {
        KeyVal::Mv(r.clone())
    } else if let (Value::Number(n), true) = (&value, listed) {
        KeyVal::Int(
            n.q.to_i64()
                .ok_or(Stop::NotBuilt("a key that isn't an integer"))?,
        )
    } else if let (Value::Day(d), true) = (&value, listed) {
        KeyVal::Day(*d)
    } else {
        crate::eval::to_key(m, &mv.id, &value, domain)?
    };
    let mut keys = tuple.keys.clone();
    keys.push(key);
    let exists = match member.unknown {
        None => tuple.exists.clone(),
        Some(h) => join(tuple.exists.clone(), &h),
    };
    Ok(Tuple {
        env: member.env,
        keys,
        exists,
        domain_hole: None,
        refused: None,
        coll: tuple.coll.clone(),
    })
}
