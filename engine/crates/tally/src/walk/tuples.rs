//! The plan pass's binding tuples: a move's keys bound depth first at its law date, and the tuple and places a link names.
//! Spec: E §4.1 steps 1 and 2 (enumerate a move's binding tuples at the cut, each binding read
//! at the move's law date with the keys bound before it); L §5.14 Keys (a member whose `where`
//! is a hole is a hole tuple; one whose `where` is refused ends its tuple there, a refused one;
//! tally 2.1.6), §4.3 (a move's law date by its `at`; a key's `where` that reads a law-dated
//! fact, while the `at` reads a key not yet bound, waits for the tuple's `at` and is read
//! there, in binding order: #235's P30, P31); L §5.19 Links (the tuple of a move at its keys under a
//! cut, its `when` read at the cut, and the places its legs give).
//! Twin: engine/tools/refcore/moves.py (`move_tuples` with its `go`, `move_law`,
//! `link_tuple`, `link_places`); refcore guards a waiting `where` (#235). `linked_instance`
//! (`o.fulfils`, read in `book/members.rs` `fulfils_of`) departs from refcore's `find_inst` on
//! purpose: the instance by its tuple and its day, with no list and no guard on it (P22, P30).
//! Invariants: tuples come in binding order, depth first; what deciding a tuple read goes with
//! its instance, never to the reader of the move's instances; a link whose `when` is a hole or
//! refused is taken as existing and judged where its instance is; a key's `where` waits only
//! where load marked it (`MoveInfo::dated_wheres`: the move has an `at` that reads that key or
//! a later one), and is read at its tuple's law date once the `at`'s keys are bound, giving
//! there what its law-free twin gives; where no law date can be had (the `at` refused or a
//! hole, a later key's domain a hole or its `where` refused first) it is left unread and the
//! tuple stands with that refusal or hole (#235's P31, its Q-3; the gate review's items 1,
//! 15); a tuple
//! with every key bound never leaves with it unread.

use super::instance::{Exists, join};
use super::narrow::KeptTuple;
use super::plan::{invariant, refuse};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, Kept, Member, reads_var};
use crate::hole::Hole;
use crate::model::{Domain, KeyVal, MoveInfo, Node, NodeKind, Reality, Value};
use crate::syntax::Stop;
use crate::time::Day;

/// A binding tuple of a move: its scope, keys, and the hole of a `where` that is one, or of a
/// domain or law date that is one, and what deciding it read (refcore's `move_tuples`
/// statuses and colls).
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// tuple's `at`, which reads a key not bound when they were (L §4.3, E §4.1 step 1;
    /// #235's P30).
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
                // every key bound, so the `at` has its keys: a `where` still waiting was
                // never read, which no answer may take as true (R-27)
                if !tuple.waiting.is_empty() {
                    return Err(invariant(
                        "a waiting where left unread with every key bound",
                    ));
                }
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
                law @ (Law::Day(_) | Law::None) => law,
            };
            // a `where` that reads a law-dated fact, while the `at` reads a key not yet bound,
            // waits for its tuple's `at` (L §4.3, E §4.1 step 1; #235's P30)
            let pos = tuple.keys.len();
            let waits = mv.dated_wheres.contains(&pos);
            if waits && law != Law::None {
                // load marks a key only where the `at` reads it or a key bound after it
                return Err(invariant("a waiting where with its law date known"));
            }
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
            let members = match self.binding_members(m, node, &tuple.env, ctx.at_law(law.day()))? {
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
    /// tuple's law date in binding order, before a later key's refused `where`: a false one
    /// drops the tuple, a hole joins its existence, a refused one makes it a refused tuple. The
    /// tuple as it was while the `at` still reads a key not yet bound, or where the `at` is
    /// refused: its instance then reads the `at` again and is refused alone, as on the path of
    /// a `where` that reads nothing law-dated. L §4.3 (a key binding's `where` is read for each
    /// tuple at the move's `at`), E §4.1 steps 1 and 2 (#235's P30, P31).
    fn waiting_wheres(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        mut tuple: Tuple,
        ctx: Ctx,
    ) -> Result<Option<Tuple>, Stop> {
        if tuple.waiting.is_empty() {
            return Ok(Some(tuple));
        }
        self.push();
        let law = self.move_law(m, mv, &tuple.env, ctx);
        tuple.coll.merge(&self.pop_alone());
        let law = match law {
            Ok(law) => law,
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                // no law date to read the `where` at: the instance reads the `at` again and
                // is refused alone, as a refused `at` is on the law-free path (#235's P31, Q-3)
                tuple.waiting.clear();
                return Ok(Some(tuple));
            }
            Err(stop) => return Err(stop),
        };
        if law == Law::None {
            return Ok(Some(tuple));
        }
        for pos in core::mem::take(&mut tuple.waiting) {
            let bind = mv
                .binds
                .get(pos)
                .ok_or_else(|| invariant("a waiting key's binding"))?;
            self.push();
            let kept = self.where_at_law(m, (&bind.node, &tuple.env), &law, ctx);
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

    /// A `where` read at a tuple's law date `law`: a hole law date makes it that hole.
    /// L §4.3, §5.14 Keys.
    fn where_at_law(
        &mut self,
        m: &Reality,
        (bind, env): (&crate::syntax::Bind, &Env),
        law: &Law,
        ctx: Ctx,
    ) -> Result<Kept, Stop> {
        match law {
            Law::Hole(h) => Ok(Kept::In(Some(h.clone()))),
            Law::None => Err(invariant("a waiting where with no law date")),
            Law::Day(d) => self.where_keeps(m, bind, env, ctx.at_law(Some(*d))),
        }
    }

    /// Under a traced read, what decides that move `mv`'s instance at keys `keys` exists at
    /// cut `cut`, read again in binding order, traced and kept nowhere: its tuple's `where`s,
    /// each at the tuple, up to one that drops or refuses it, then, for a plain tuple, the
    /// move's `when` at its law date, a plan's or a decision's. An existence read and a
    /// binding that judges a link take these and no other tuple's reads (refcore's
    /// `move_tuples` and `link_tuple`, for the named tuple; L §5.19 Links, §5.14 Keys, §8.2
    /// One read set; E §10; P30, P42, P43).
    ///
    /// # Errors
    ///
    /// A load refusal met at run; as [`Engine::move_tuples`].
    pub(crate) fn trace_existence(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<(), Stop> {
        if !self.tracing() {
            return Ok(());
        }
        let before = self.trace_walked();
        let tuples = self.plan_tuples(m, mv, cut);
        self.trace_walked_back(before);
        let Some(tuple) = tuple_at(tuples?, keys) else {
            return Ok(());
        };
        let tctx = self.timeline_ctx(None, cut);
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.push();
        self.trace_plan_open();
        let read = self.existence_reads(m, mv, &tuple, tctx);
        let closed = self.trace_plan_close();
        self.pop_alone();
        self.holder = outer;
        self.trace_walked_back(before);
        closed?;
        match read {
            // a decision's `when` that reads running state is read at its walk point (E §4.2
            // step 3; tally 2.3.1 R6)
            Err(Stop::Refused(r))
                if mv.decision && r.code() == Code::Load(Load::Reads) && self.plans_making == 0 =>
            {
                Ok(())
            }
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(Stop::Refused(r)),
            Ok(()) | Err(Stop::Refused(_)) => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// The reads [`Engine::trace_existence`] traces for tuple `tuple` of move `mv`, in the
    /// existence cell's order: each bound key's `where` at the tuple's law date, in binding
    /// order, up to one that drops it or is refused, then its day, then the move's `when`; a
    /// false or `none` read ends them, a refused one doesn't (refcore's `move_tuples`, for one
    /// tuple, and `link_tuple`). L §5.14 Keys, An instance's existence is a cell, §4.3, §5.19
    /// Links (tally 2.3.1 R6).
    ///
    /// # Errors
    ///
    /// The first refusal met; as [`Engine::move_law`].
    fn existence_reads(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        tuple: &Tuple,
        tctx: Ctx,
    ) -> Result<(), Stop> {
        let law = match self.move_law(m, mv, &tuple.env, tctx)? {
            Law::Hole(_) => return Ok(()),
            law @ (Law::Day(_) | Law::None) => law.day(),
        };
        let ctx = tctx.at_law(law);
        let mut bad = None;
        for bind in mv.binds.iter().take(tuple.keys.len()) {
            match self.where_keeps(m, &bind.node, &tuple.env, ctx) {
                Ok(Kept::Out) => return Ok(()),
                Ok(_) => {}
                Err(Stop::Refused(r))
                    if !matches!(r.code(), Code::Load(_) | Code::Run(Run::Cycle)) =>
                {
                    bad = Some(r);
                    break;
                }
                Err(stop) => return Err(stop),
            }
        }
        if tuple.domain_hole.is_some() {
            return bad.map_or(Ok(()), |r| Err(Stop::Refused(r)));
        }
        let read = [Some(&mv.decl.on), mv.decl.when.as_ref()];
        for x in read.into_iter().flatten() {
            match self.existence_read(m, x, &tuple.env, ctx)? {
                Existence::Ends => return Ok(()),
                Existence::Read | Existence::Hole => {}
                Existence::Refused(r) => {
                    bad.get_or_insert(r);
                }
            }
        }
        bad.map_or(Ok(()), |r| Err(Stop::Refused(r)))
    }

    /// A move's law date by its `at`, with the keys bound so far: none while it reads a key
    /// not yet bound, by the same walk load marks the keys by (`reads_var`; refcore's
    /// `move_law` counts any variable, an aggregate's own too). L §4.3, §5.14.
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
            .any(|b| env.get(&b.var).is_none() && reads_var(at, &b.var))
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
        // a window `at` is a hole law date, as any movement's whose day is a hole (L §4.4)
        Ok(
            match crate::eval::one_day(m, &v, &format!("the law date of {}", mv.id))? {
                Ok(d) => Law::Day(d),
                Err(h) => Law::Hole(h),
            },
        )
    }

    /// One read of a link's existence cell, its day or its `when`, at the law date `ctx`
    /// holds: `none`, `never` or false ends it (no instance); a run-time refusal is the
    /// instance's; a load refusal or a cycle is the reader's (refcore's `link_tuple`, its
    /// `on` and `when`). L §5.14 An instance's existence is a cell, §5.19 Links (tally 2.3.1
    /// R6, rulings 16 Q1).
    ///
    /// # Errors
    ///
    /// A load refusal or `RUN-CYCLE` met reading it.
    fn existence_read(
        &mut self,
        m: &Reality,
        x: &crate::syntax::Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Existence, Stop> {
        match self.ev(m, x, env, ctx) {
            Ok(Value::None | Value::Never | Value::Bool(false)) => Ok(Existence::Ends),
            Ok(Value::Hole(_) | Value::Window(_)) => Ok(Existence::Hole),
            Ok(_) => Ok(Existence::Read),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_) | Code::Run(Run::Cycle)) => {
                Ok(Existence::Refused(r))
            }
            Err(stop) => Err(stop),
        }
    }

    /// The `for` of move `mv`'s binding tuple at keys `keys` under cut `cut`, its existence
    /// unread: the period a row that names it is for; none where the move has no `for` or no
    /// tuple has those keys (refcore's `named_for`). L §5.14 `for k`, §5.19 Links (tally 2.3.1
    /// rulings 22 (1), the critic's P1-C6).
    ///
    /// # Errors
    ///
    /// The refusals the binding tuples and the `for` meet.
    pub(crate) fn named_for(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<Option<Value>, Stop> {
        let Some(e) = &mv.decl.for_period else {
            return Ok(None);
        };
        let found = match self.kept_tuple(m, mv, keys, cut)? {
            KeptTuple::Kept(found) => found.map(|t| *t),
            KeptTuple::Own => tuple_at(self.plan_tuples(m, mv, cut)?, keys),
        };
        let Some(tuple) = found else {
            return Ok(None);
        };
        let tctx = self.timeline_ctx(None, cut);
        Ok(Some(self.ev(m, e, &tuple.env, tctx)?))
    }

    /// Move `mv`'s binding tuples at cut `cut`, what making them reads the plan's, as its list
    /// makes them, with no guard on the list: an existence read's (P30). E §4.1 step 1, §3.2.
    ///
    /// # Errors
    ///
    /// The refusals the binding tuples meet.
    fn plan_tuples(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<Vec<Tuple>, Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let tctx = self.timeline_ctx(None, cut);
        let made = self.move_tuples(m, mv, tctx);
        self.reads_back(mark);
        self.holder = outer;
        made
    }
}

/// What one read of a link's existence cell gives (refcore's `link_tuple`). L §5.14.
enum Existence {
    /// `none`, `never` or false: no instance, whatever came before.
    Ends,
    /// Any other value: the read goes on.
    Read,
    /// A hole or a window: a day that is a hole makes the booking cut a hole with it, so no
    /// later read ends the cell (L §5.14 l.1361).
    Hole,
    /// A run-time refusal: the instance's, unless a later read ends it.
    Refused(Refusal),
}

/// A move's law date: none, a day, or a hole (L §4.3, §5.14 `at e`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Law {
    /// No `at`, or one that reads a key not yet bound.
    None,
    /// A day.
    Day(Day),
    /// A hole.
    Hole(Hole),
}

impl Law {
    /// The law date an instance is read at: the day; none for no law date or a hole. L §4.3.
    #[must_use]
    pub fn day(&self) -> Option<Day> {
        match self {
            Law::Day(d) => Some(*d),
            Law::None | Law::Hole(_) => None,
        }
    }
}

/// The tuple a link names at keys `keys`: the one with those keys, or the refused one a
/// `where` ended before them, which stands for its keys' instances alone (L §5.14 Keys, An
/// instance's existence is a cell; tally 2.3.1 R6).
pub(super) fn tuple_at(tuples: Vec<Tuple>, keys: &[KeyVal]) -> Option<Tuple> {
    tuples
        .into_iter()
        .find(|t| t.keys == keys || (t.refused.is_some() && keys.starts_with(&t.keys)))
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
