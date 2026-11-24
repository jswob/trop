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

use super::instance::{Dated, Exists, Inst, join};
use super::narrow::KeptTuple;
use super::plan::{day_dated, invariant, refuse};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, Kept, Member, reads_var};
use crate::hole::Hole;
use crate::model::{Domain, KeyVal, MoveInfo, MvRef, Node, NodeKind, Reality, Value};
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

    /// Under a traced read, what a link's verdict that the instance it names exists rests on,
    /// read where a binding judges the link and in the order judging it reads them: move
    /// `mv`'s binding tuples at cut `cut`, then its `when` at the tuple at `keys`, a plan's or
    /// a decision's, at its law date; traced and kept nowhere (refcore's `link_tuple`; L §5.19
    /// Links, §8.2 One read set; E §10; P31, P42).
    ///
    /// # Errors
    ///
    /// A load refusal met at run; as [`Engine::move_tuples`].
    pub(crate) fn trace_link_when(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<(), Stop> {
        let Some(when) = &mv.decl.when else {
            return Ok(());
        };
        if !self.tracing() {
            return Ok(());
        }
        let before = self.trace_walked();
        let tctx = self.timeline_ctx(None, cut);
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.push();
        self.trace_plan_open();
        let read = match self.move_tuples(m, mv, tctx) {
            Ok(tuples) => Ok(self.tuple_when(m, mv, (when, keys), &tuples, tctx)),
            Err(stop) => Err(stop),
        };
        let closed = self.trace_plan_close();
        self.pop_alone();
        self.holder = outer;
        self.trace_walked_back(before);
        closed?;
        match read? {
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(Stop::Refused(r)),
            Ok(_) | Err(Stop::Refused(_)) => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// Move `mv`'s `when` at its plain tuple at `keys` among `tuples`, at its law date; none
    /// where no plain tuple is there or its law date is a hole (refcore's `link_tuple`, its
    /// `when`). L §5.19 Links, §4.3.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_law`] and the `when`'s evaluation.
    fn tuple_when(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (when, keys): (&crate::syntax::Expr, &[KeyVal]),
        tuples: &[Tuple],
        tctx: Ctx,
    ) -> Result<Value, Stop> {
        let Some(tuple) = tuples
            .iter()
            .find(|t| t.keys == keys && t.refused.is_none() && t.domain_hole.is_none())
        else {
            return Ok(Value::None);
        };
        match self.move_law(m, mv, &tuple.env, tctx)? {
            Law::Hole(_) => Ok(Value::None),
            law @ (Law::Day(_) | Law::None) => self.ev(m, when, &tuple.env, tctx.at_law(law.day())),
        }
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
        // the move's tuples at the cut, kept once for every link and read there (fence F7)
        let found = match self.kept_tuple(m, mv, keys, cut)? {
            KeptTuple::Kept(found, _) => found.map(|t| *t),
            KeptTuple::Own => self
                .move_tuples(m, mv, tctx)?
                .into_iter()
                .find(|t| t.keys == keys),
        };
        let Some(tuple) = found else {
            return Ok(None);
        };
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
        Ok(Some(tuple.env))
    }

    /// What `o.fulfils` names at keys `keys` of move `mv` under cut `cut`: the instance the
    /// move's list holds there, by its tuple and its day at the cut alone, never its body,
    /// since a link reads only the instance's existence, never its amounts; none where the
    /// list holds none (no tuple at the keys, or a day `never` or `none`). An existence read
    /// takes no guard on the move's list: what it evaluates are ordinary reads, a cycle only
    /// where one returns to a cell still being evaluated (P30). A decision's is taken from its
    /// list; a refused tuple's, whose `when` decides whether the list holds it, from the list
    /// or, with the list in progress, as the list makes it; and where the move's tuples stop
    /// its list, the list's stop (refcore's `find_inst`). A traced read reads as an untraced
    /// one: its explanation lists the rows the answer rests on, none of the list's bodies (E
    /// §10). What reading them meets is the plan's, no reader's, and a list it makes leaves
    /// what judging the list's links meets to the list's next reader of its instances
    /// ([`Engine::existence_list`]). L §5.19 Links, §5.11, E §4.1 steps 1–2, §3.2 (P22, P30,
    /// P40).
    ///
    /// # Errors
    ///
    /// A load refusal met at run; the stop of the move's tuples; as
    /// [`Engine::move_instances`] for those taken from the list.
    pub(crate) fn linked_instance(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<Option<MvRef>, Stop> {
        let open = !self.plan_active.contains(&(mv.id.clone(), cut));
        // a decision's instances are its runs' (E §4.1)
        if !mv.decision {
            // the move's tuples at the cut, kept once for every link and read there (fence F7)
            let (found, stops) = match self.kept_tuple(m, mv, keys, cut)? {
                KeptTuple::Kept(found, stops) => (found.map(|t| *t), stops),
                KeptTuple::Own => {
                    let tuples = self.plan_tuples(m, mv, cut)?;
                    let stops = tuples
                        .iter()
                        .any(|t| t.refused.is_some() && t.keys.len() < mv.binds.len());
                    (tuples.into_iter().find(|t| t.keys == keys), stops)
                }
            };
            match (stops, found) {
                (false, None) => return Ok(None),
                (false, Some(tuple)) if tuple.refused.is_none() => {
                    return self.linked_day(m, mv, tuple, cut);
                }
                (false, Some(tuple)) if !open => return self.linked_refused(m, mv, tuple, cut),
                (true, _) if !open => return Err(self.tuples_stop(m, mv, cut)?),
                _ => {}
            }
        }
        let insts = self.existence_list(m, mv, cut)?;
        Ok(insts
            .iter()
            .find(|i| i.keys == keys)
            .map(crate::book::inst_ref))
    }

    /// The instance of refused tuple `tuple` of move `mv` at cut `cut`, read while the move's
    /// list there is made: as the list makes it, its day, `when` and keys deciding whether it
    /// exists, with no guard on the list (refcore's `insts_of_tuples`, its refused tuples).
    /// L §5.19 Links, §5.14 Keys, E §4.1 step 2 (P30).
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn linked_refused(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        tuple: Tuple,
        cut: Day,
    ) -> Result<Option<MvRef>, Stop> {
        let Some(err) = tuple.refused.clone() else {
            return Err(invariant("a refused link tuple with no refusal"));
        };
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let tctx = self.timeline_ctx(None, cut);
        let made = self.refused_tuple(m, mv, (tuple, None), err, tctx);
        self.reads_back(mark);
        self.holder = outer;
        Ok(made?.as_ref().map(crate::book::inst_ref))
    }

    /// The stop of move `mv`'s list at cut `cut` read while it is made: the refusal of its
    /// first tuple refused with its later keys unbound, as the list meets it (refcore's
    /// `insts_of_tuples` raises). L §5.14 Keys, §8.4, E §4.1 step 1 (P30).
    ///
    /// # Errors
    ///
    /// The refusals the binding tuples meet; `ENGINE-INVARIANT` where no tuple stops the list.
    fn tuples_stop(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<Stop, Stop> {
        for tuple in self.plan_tuples(m, mv, cut)? {
            if let (Some(err), true) = (tuple.refused, tuple.keys.len() < mv.binds.len()) {
                return Ok(Stop::Refused(err));
            }
        }
        Err(invariant("a stopped list with no refused tuple"))
    }

    /// The instance of a link's plain or hole tuple as the list holds it, its day read at cut
    /// `cut` as the list reads it, with no guard on the move's list (P30), and nothing of its
    /// body; none where the day is `never` or `none` (refcore's `eval_inst`, its day). A run-time
    /// refusal reading the day is the instance's own: the list holds it refused, with no day;
    /// but a cycle, which is refused at the cell it returns to. E §4.1 step 2, L §5.19 Links,
    /// §5.11 (P32).
    ///
    /// # Errors
    ///
    /// A load refusal met at run; `RUN-CYCLE` where reading the day returns to a cell still
    /// being evaluated.
    fn linked_day(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        tuple: Tuple,
        cut: Day,
    ) -> Result<Option<MvRef>, Stop> {
        let mut inst = Inst::new(mv, tuple.keys, tuple.env);
        if let Some(hole) = tuple.domain_hole {
            inst.day = Some(Dated::Hole(hole));
            return Ok(Some(crate::book::inst_ref(&inst)));
        }
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        // the day an existence read takes is an ordinary read: a traced read prints its rows,
        // in a read set of its own, as the plan pass's (L §5.19, §8.2; E §10; P30, P31)
        let traced = self.tracing();
        let mark = if traced {
            self.trace_plan_open();
            None
        } else {
            Some(self.reads_apart())
        };
        let tctx = self.timeline_ctx(None, cut);
        self.push();
        let read = self.law_and_day(m, mv, &mut inst, tctx);
        self.pop_alone();
        let closed = match mark {
            Some(mark) => {
                self.reads_back(mark);
                Ok(())
            }
            None => self.trace_plan_close().map(|_| ()),
        };
        self.holder = outer;
        closed?;
        let dated = match read {
            Ok(Ok(Value::Never | Value::None)) => return Ok(None),
            Ok(Ok(v)) => day_dated(m, v, &inst.id),
            Ok(Err(h)) => Ok(Dated::Hole(h)),
            Err(stop) => Err(stop),
        };
        match dated {
            Ok(day) => inst.day = Some(day),
            // a cycle is never the instance's own: the chain is refused at the cell it returns
            // to (L §5.11; P32)
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_) | Code::Run(Run::Cycle)) => {
            }
            Err(stop) => return Err(stop),
        }
        Ok(Some(crate::book::inst_ref(&inst)))
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
            law @ (Law::Day(_) | Law::None) => law.day(),
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
            Ok(law @ (Law::Day(_) | Law::None)) => law.day(),
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
