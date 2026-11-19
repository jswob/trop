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

use super::instance::{Exists, join};
use super::plan::{date_of, invariant, refuse};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Ctx, Decided, Engine, Env, Kept, LawClass, Member, reads_name};
use crate::hole::{Hole, Unknown};
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
    /// The positions of its keys whose `where` reads a law-dated fact and waits for the
    /// tuple's `at`, which reads a key not bound when they were (L §4.3, E §4.1 step 1; P30).
    pub(super) waiting: Vec<usize>,
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
            waiting: Vec::new(),
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
                law @ (Law::Day(_) | Law::Window(..) | Law::None) => law,
            };
            // a `where` that reads a law-dated fact, while the `at` reads a key not yet bound,
            // waits for its tuple's `at` (L §4.3, E §4.1 step 1; P30)
            let pos = tuple.keys.len();
            let waits = law == Law::None && mv.dated_wheres.contains(&pos);
            let unfiltered;
            let node = if waits {
                unfiltered = crate::syntax::Bind {
                    filter: None,
                    ..bind.node.clone()
                };
                &unfiltered
            } else {
                &bind.node
            };
            let members = match self.members_over_law(m, mv, (node, &tuple.env), &law, ctx)? {
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
                if waits {
                    extended.waiting.push(pos);
                }
                next.extend(self.waiting_wheres(m, mv, extended, ctx)?);
            }
            work.extend(next.into_iter().rev());
        }
        Ok(out)
    }

    /// A tuple's waiting `where`s, read once the keys its `at` reads are bound, each at the
    /// tuple's law date in binding order: a false one drops the tuple, a hole joins its
    /// existence, a refused one makes it a refused tuple; the tuple as it was while the `at`
    /// still reads a key not yet bound. L §4.3 (a key binding's `where` is read for each tuple
    /// at the move's `at`), E §4.1 step 1 (P30).
    fn waiting_wheres(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        mut tuple: Tuple,
        ctx: Ctx,
    ) -> Result<Option<Tuple>, Stop> {
        if tuple.waiting.is_empty() || tuple.refused.is_some() {
            return Ok(Some(tuple));
        }
        self.push();
        let law = self.move_law(m, mv, &tuple.env, ctx);
        tuple.coll.merge(&self.pop_alone());
        let law = law?;
        if law == Law::None {
            return Ok(Some(tuple));
        }
        for pos in core::mem::take(&mut tuple.waiting) {
            let bind = mv
                .binds
                .get(pos)
                .ok_or_else(|| invariant("a waiting key's binding"))?;
            self.push();
            let kept = self.where_at_law(m, mv, (&bind.node, &tuple.env), &law, ctx);
            tuple.coll.merge(&self.pop_alone());
            match kept {
                Ok(Kept::Out) => return Ok(None),
                Ok(Kept::In(None)) => {}
                Ok(Kept::In(Some(h))) => tuple.exists = join(tuple.exists, &h),
                Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                    tuple.refused = Some(r);
                    return Ok(Some(tuple));
                }
                Err(stop) => return Err(stop),
            }
        }
        Ok(Some(tuple))
    }

    /// A `where` read at a tuple's law date `law`: a hole law date makes it that hole; under a
    /// window `at`, read once per law-date class the window holds, `HOLE-IMPRECISE` where two
    /// differ, as a member's `where` is in `members_over_law`. L §4.3, §4.4, §5.14 Keys.
    fn where_at_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (bind, env): (&crate::syntax::Bind, &Env),
        law: &Law,
        ctx: Ctx,
    ) -> Result<Kept, Stop> {
        let day = match law {
            Law::Hole(h) => return Ok(Kept::In(Some(h.clone()))),
            Law::None => return Err(invariant("a waiting where with no law date")),
            Law::Day(d) => return self.where_keeps(m, bind, env, ctx.at_law(Some(*d))),
            Law::Window(..) => law
                .first()
                .ok_or_else(|| invariant("a window with no day"))?,
        };
        let mark = self.open_law_class(day);
        let first = self.where_keeps(m, bind, env, ctx.at_law(Some(day)));
        let mut class = self.close_law_class(mark, day)?;
        let first = first?;
        if let (Kept::Out | Kept::In(None), Some(window)) = (&first, law.window()) {
            for at in law.rest() {
                if class.holds(*at) {
                    continue;
                }
                let opened = self.open_law_class(*at);
                let other = self.where_keeps(m, bind, env, ctx.at_law(Some(*at)));
                class = self.close_law_class(opened, *at)?;
                if other? != first {
                    return Ok(Kept::In(Some(imprecise_law(window, &mv.id))));
                }
            }
        }
        Ok(first)
    }

    /// The members of binding `bind` in scope `env` at law date `law`: under a window `at`,
    /// read at each law date the window can take; where they differ, `HOLE-IMPRECISE` is the
    /// domain's hole where the domain differs, else the hole of each member whose `where`
    /// differs (refcore's `move_tuples`, its `lds`; #169-e). L §4.3, §4.4, §5.14 `at e`.
    fn members_over_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (bind, env): (&crate::syntax::Bind, &Env),
        law: &Law,
        ctx: Ctx,
    ) -> Result<Decided, Stop> {
        let (Law::Window(_, text, carry), Some(day)) = (law, law.first()) else {
            return self.binding_members(m, bind, env, ctx.at_law(law.first()));
        };
        // once per law-date class: a day its class holds gives what it gave (L §4.4's cost)
        let (first, mut class) = self.members_at(m, (bind, env), (ctx, day))?;
        let Ok(members) = &first else {
            return Ok(first);
        };
        let key = members_key(members);
        let mut days = vec![day];
        let mut differ = false;
        for at in law.rest() {
            if class.holds(*at) {
                continue;
            }
            let (other, next) = self.members_at(m, (bind, env), (ctx, *at))?;
            differ |= !other.as_ref().is_ok_and(|o| members_key(o) == key);
            days.push(*at);
            class = next;
        }
        if !differ {
            return Ok(first);
        }
        // the domain is compared whole, each member's `where` on its own: a member every law
        // date keeps alike stays a member (refcore's `move_tuples`, its `lds`; #169-e)
        let hole = imprecise_law((text, carry), &mv.id);
        self.binding_members_over(m, (bind, env), ctx, (days, hole))
    }

    /// A binding's members at law date `day`, with the law dates that give the same
    /// (E §3.2). L §4.3, §5.14 `at e`.
    fn members_at(
        &mut self,
        m: &Reality,
        (bind, env): (&crate::syntax::Bind, &Env),
        (ctx, day): (Ctx, Day),
    ) -> Result<(Decided, LawClass), Stop> {
        let mark = self.open_law_class(day);
        let members = self.binding_members(m, bind, env, ctx.at_law(Some(day)));
        let class = self.close_law_class(mark, day);
        Ok((members?, class?))
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
        // an open end of a lifespan, `at lifespan(p).to` for a living p (L §3.6): only this
        // value is refused, as refcore's `date_of` does, and the other asks are answered
        if v == Value::Never {
            return Err(refuse(
                Run::None,
                format!("never in the law date of {}", mv.id),
            ));
        }
        if let Value::Window(w) = &v
            && matches!(w.lo, Value::Day(_))
        {
            // a window `at` reads the instance at each law date it can take (L §5.14 `at e`,
            // §4.3; refcore's `m_move_law`, `law_days`): the memo shares a class's cells
            let text = w.question_text(m).map_err(crate::eval::num_stop)?;
            return Ok(Law::Window(
                crate::eval::window_days(w)?,
                text,
                w.carry.clone(),
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
                    // a cycle is never taken as existing: the link would stand unjudged
                    Err(Stop::Refused(r))
                        if !matches!(r.code(), Code::Load(_) | Code::Run(Run::Cycle)) => {}
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
            // a window `at`: its first law date (refcore's `link_tuple`, `law_days(law)[0]`)
            law @ (Law::Day(_) | Law::Window(..) | Law::None) => law.first(),
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
            // a window `at`: its first law date (refcore's `link_places`, `law_days(law)[0]`)
            Ok(law @ (Law::Day(_) | Law::Window(..) | Law::None)) => law.first(),
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

/// A move's law date: none, a day, a window, or a hole (L §4.3, §5.14 `at e`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Law {
    /// No `at`, or one that reads a key not yet bound.
    None,
    /// A day.
    Day(Day),
    /// A window: the instance is read at each law date it can take, its first first, and
    /// one value where they all agree (L §4.3, §5.14 `at e`); with the window as a question
    /// names it, and the unknown days it carries (L §4.4; tally 2.1.8 R13).
    Window(Vec<Day>, String, Vec<Unknown>),
    /// A hole.
    Hole(Hole),
}

impl Law {
    /// The law date an instance is read at first: the day, a window's first; none for no
    /// law date or a hole. L §4.3.
    #[must_use]
    pub fn first(&self) -> Option<Day> {
        match self {
            Law::Day(d) => Some(*d),
            Law::Window(days, ..) => days.first().copied(),
            Law::None | Law::Hole(_) => None,
        }
    }

    /// The law dates past the first that a window `at` reads too, in order. L §4.3.
    #[must_use]
    pub fn rest(&self) -> &[Day] {
        match self {
            Law::Window(days, ..) => match days.as_slice() {
                [_, rest @ ..] => rest,
                [] => &[],
            },
            Law::Day(_) | Law::None | Law::Hole(_) => &[],
        }
    }

    /// The window as a question names it, and the unknown days it carries, for a window
    /// `at`. L §4.4.
    #[must_use]
    pub fn window(&self) -> Option<(&str, &[Unknown])> {
        match self {
            Law::Window(_, text, carry) => Some((text, carry)),
            Law::Day(_) | Law::None | Law::Hole(_) => None,
        }
    }
}

/// `HOLE-IMPRECISE` of a window `at` whose law dates give `what` different values:
/// `?"date within <w> not precise enough for the law date of <what>" (<what>)`, carrying the
/// window's unknown days (refcore's `imprecise_law`). L §4.3, §4.4, §8.1.
pub(super) fn imprecise_law((window, carry): (&str, &[Unknown]), what: &str) -> Hole {
    let text = format!("date within {window} not precise enough for the law date of {what}");
    Hole::asked(&text, what).carrying(carry)
}

/// What a binding's members are, to compare them across law dates: each member's scope, its
/// `where`'s hole and whether it was refused (refcore's `vkey`s of `bind_each`, a refusal
/// as `'err'`). L §4.3.
fn members_key(
    members: &[(Member, Collected, Option<Refusal>)],
) -> Vec<(&Env, Option<&Hole>, bool)> {
    members
        .iter()
        .map(|(member, _, refused)| (&member.env, member.unknown.as_ref(), refused.is_some()))
        .collect()
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
    } else if let (Value::Number(_), true) = (&value, listed) {
        // an integer an int key, any other number by its value (L §2.5, §5.6)
        crate::eval::value_key(m, &value)?.ok_or_else(|| invariant("a number no key holds"))?
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
        waiting: tuple.waiting.clone(),
    })
}
