//! Aggregates over bindings: tuples in binding order, the unknown members a hole `where` makes, and each aggregate's rule for holes.
//! Spec: L §5.11 (aggregates; a key whose `where` is a hole is an unknown member; `sum` over
//! no keys is 0 in the term's type; `count`, `any`, `all`), §8.1 (holes spread). The values a
//! binding ranges over are `bindings.rs`'s; the extremes, `min` to `last`, `extremes.rs`'s.
//! E §4.3: a `min` of `bal(x, d) from t` over the days a .. b is offered first, its range's
//! values read once, to the range minimum (`walk/projection.rs`'s `lowest_shape` and
//! `range_minimum`, fence F5 or its plain twin's one walk); this aggregate reads it term by
//! term from the same values only where the range minimum leaves it, on a `partial` line and
//! in a traced read. The two agree: T's law `low-min` (the same `min` with a `where` every
//! day meets) and every vector of a spelled `min` against refcore, which reads it term by term.
//! Twin: engine/tools/refcore/evaluate.py (`members`, `zero_of`, `e_agg`, `agg_anyall`).
//! Invariants: tuples come in binding order, depth first as refcore's generator, one at a
//! time, so `first` and `last` (from the last) read no `where` past their key; an unknown
//! member's term counts only as its questions, except a 0 in a sum; a domain that is a hole
//! makes the aggregate that hole; a binding over movements or an observed input takes its
//! members from the book (`book/members.rs`), and a member's register reaches the reader only
//! when the member passes the `where`: an aggregate's reader, or an ask's line for that
//! tuple alone.

use super::arith::{Op, arith};
use super::bindings::{Values, bind_domain, none_here};
use super::expr::hole_of;
use super::extremes::{bare_key, min_max};
use super::partial;
use super::register::Collected;
use super::state::{Ctx, Engine, Env, invariant, not_typed, num_stop, refuse};
use super::trace::Snap;
use super::windows::End;
use crate::base::{Code, Refusal, Run};
use crate::book::Reading;
use crate::hole::{Hole, spread};
use crate::model::{Class, Number, Reality, Ty, Value, Window, currency_id};
use crate::num::{Money, Num};
use crate::syntax::{Agg, AggFn, Bind, Expr, Stop};
use crate::walk::lowest_shape;

/// A binding tuple: its scope, the questions that make it an unknown member, and what its
/// movements brought when it keeps them for its own line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The scope with the tuple's variables bound.
    pub env: Env,
    /// None for a member; the hole of a `where` (or a membership) that is one.
    pub unknown: Option<Hole>,
    /// The registers of the movements it binds, kept for an ask's line (`Brings::ToTuple`);
    /// empty when they went to the reader (`Brings::ToReader`).
    pub brought: Collected,
}

/// Where the register a bound movement brings goes (refcore's `members`, its `regs`).
/// L §5.19 Bindings, §8.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Brings {
    /// To the reader's collector: an aggregate's terms all reach one value.
    ToReader,
    /// To the tuple's own line: an ask's `for` prints a line per tuple, each with the
    /// register of what it binds.
    ToTuple,
}

/// One binding's members, each with what deciding it read and the refusal its `where` met;
/// or the hole of a domain that is one, with what reading it read (refcore's `move_tuples`,
/// its `cd` and `cw`). E §4.1 step 1, L §5.14 Keys.
pub(crate) type Decided = Result<Vec<(Member, Collected, Option<Refusal>)>, (Hole, Collected)>;

/// What a binding's `where` says of a member: left out, or kept with the hole the `where` is
/// where it is one. L §5.11.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kept {
    /// The `where` is false.
    Out,
    /// The `where` is true, or a hole.
    In(Option<Hole>),
}

/// The frame of a binding over movements: each member's value and hole, and what it
/// brings, kept by position (refcore's `members`, `_colls`). L §5.19 Bindings.
fn moves_frame(
    level: usize,
    (env, unknown, brought): (Env, Option<Hole>, Collected),
    moves: Vec<crate::book::MvMember>,
) -> Frame {
    let mut values = Vec::with_capacity(moves.len());
    let mut colls = Vec::with_capacity(moves.len());
    let mut refused = Vec::with_capacity(moves.len());
    for mv in moves {
        values.push((mv.value, mv.unknown));
        colls.push(mv.coll);
        refused.push(mv.refused);
    }
    Frame {
        level,
        env,
        unknown,
        brought,
        values,
        next: 0,
        colls,
        refused,
    }
}

/// A frame of the walk over the bindings: the binding's values and the next one to take.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Frame {
    /// The binding's position.
    level: usize,
    /// The scope before this binding.
    env: Env,
    /// The unknown-member hole so far.
    unknown: Option<Hole>,
    /// What the movements bound so far brought, for the tuple's own line.
    brought: Collected,
    /// The values.
    values: Vec<(Value, Option<Hole>)>,
    /// The next value's position.
    next: usize,
    /// What each value brings to a reader that keeps it: a movement's register.
    colls: Vec<Collected>,
    /// What each value's existence refused at run: a movement's, read as a refused member
    /// (`Reading::Each`).
    refused: Vec<Option<Refusal>>,
}

/// The end a reader takes a binding's values from: the first, or the last, as `last` reads
/// them (L §5.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Way {
    /// From the first value.
    Forward,
    /// From the last value.
    Back,
}

impl Frame {
    /// The frame with its values in the order `way` takes them: `last` reads from the last
    /// binding (L §5.11).
    fn taken(mut self, way: Way) -> Frame {
        if way == Way::Back {
            self.values.reverse();
            self.colls.reverse();
            self.refused.reverse();
        }
        self
    }
}

/// A walk over binding tuples, one at a time, so a reader can stop at the tuple it wants
/// (refcore's `members` as a generator). L §5.11.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Walk {
    /// The frames, the innermost last.
    frames: Vec<Frame>,
    /// The end each binding's values are taken from.
    way: Way,
    /// How its reader takes a tuple refused at run.
    reading: Reading,
}

/// What a walk's next step meets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Step {
    /// A tuple whose `where`s keep it.
    Member(Member),
    /// An inner binding's domain that is a hole.
    Hole(Hole),
    /// A tuple refused at run, by its `where`, or under `Reading::Each` by its domain or a
    /// movement's existence, with that refusal, and its scope where the last binding's `where`
    /// refused it: `any`, `all` and `sum` read it as a refused member, every other reader
    /// stops on it (L §5.11; tally 2.3.1, rulings 17 statement A).
    Refused(Stop, Option<Env>),
    /// No tuple is left.
    Done,
}

/// One binding tuple of `any`, `all` or `sum`, read as its domain, then its `where`, then its
/// term, by the first of those that isn't plainly known (tally 2.3.1, rulings 17 statement A).
/// L §5.11, §8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Tuple {
    /// Its domain and `where` keep it (its `where` may be a hole): its term decides.
    Member(Member),
    /// An inner binding's domain that is a hole: a hole member.
    Hole(Hole),
    /// Refused at run, with its scope where its last `where` refused it, so that a `sum` can
    /// read its term.
    Refused(Stop, Option<Env>),
}

/// A binding's value bound, its `where` not yet read: its scope, its own unknown-member hole
/// (a movement's day that is one) and its movement's register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Candidate {
    /// The scope with the value bound.
    pub(super) env: Env,
    /// The value's own hole.
    pub(super) own: Option<Hole>,
    /// What it brings to a reader that keeps it.
    pub(super) brings: Option<Collected>,
}

impl Engine {
    /// An aggregate (refcore's `e_agg`): a `min` of `walk::lowest_shape`'s shape, but on a
    /// `partial` line or in a traced read, its range's values read once, offered to the range
    /// minimum (`walk/projection.rs`'s `range_minimum`, fence F5 or its plain twin) and, where
    /// that leaves it, read term by term from the same values; any other, term by term. L
    /// §5.11, E §4.3.
    pub(super) fn aggregate(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let walk = match lowest_shape(agg) {
            Some((range, read)) if self.reads_lowest(ctx) => {
                let days = self.range_values(m, range, env, ctx)?;
                if let Values::Known(known) = &days
                    && let Some(v) = self.range_minimum(m, read, known, (env, ctx))?
                {
                    return Ok(v);
                }
                walk_of(frame_of(days, env), (Way::Forward, Reading::Stops))
            }
            _ if matches!(agg.func, AggFn::First | AggFn::Last) => {
                return self.first_last(m, agg, env, ctx);
            }
            _ => {
                let reading = match agg.func {
                    AggFn::Any | AggFn::All | AggFn::Sum => Reading::Each,
                    AggFn::Count
                    | AggFn::Min
                    | AggFn::Max
                    | AggFn::Argmin
                    | AggFn::Argmax
                    | AggFn::First
                    | AggFn::Last => Reading::Stops,
                };
                self.tuple_walk(m, &agg.binds, env, ctx, (Way::Forward, reading))?
            }
        };
        if let AggFn::Any | AggFn::All | AggFn::Sum = agg.func {
            let tuples = match self.walk_terms(m, &agg.binds, walk, ctx)? {
                Err(hole) => return Ok(Value::Hole(hole)),
                Ok(tuples) => tuples,
            };
            if agg.func == AggFn::Sum {
                return self.terms_aggregate(m, agg, &tuples, ctx);
            }
            let Some(term) = &agg.term else {
                return Err(invariant("any or all with no term"));
            };
            return self.any_all(m, agg.func, term, &tuples, ctx);
        }
        let members = match self.walk_members(m, &agg.binds, walk, ctx, Brings::ToReader)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(members) => members,
        };
        if let (AggFn::Count, None) = (agg.func, &agg.term) {
            return count_members(&members);
        }
        let tuples: Vec<Tuple> = members.into_iter().map(Tuple::Member).collect();
        self.terms_aggregate(m, agg, &tuples, ctx)
    }

    /// `count`, `sum`, `min`, `max`, `argmin`, `argmax` over their terms (refcore's
    /// `e_agg`). A `sum`'s tuples by statement A: one whose term is known 0 adds 0 whatever
    /// its `where`; else the first refused in binding order refuses it, else a hole one makes
    /// it a hole (tally 2.3.1, rulings 17); every other aggregate's tuples are its members.
    /// L §5.11, §8.
    fn terms_aggregate(
        &mut self,
        m: &Reality,
        agg: &Agg,
        tuples: &[Tuple],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg.func.word();
        let snap = match (agg.func, &agg.term) {
            (AggFn::Min | AggFn::Max | AggFn::Argmin | AggFn::Argmax, Some(_)) => self.trace_snap(),
            _ => None,
        };
        let mut terms: Vec<(&Env, Value)> = Vec::with_capacity(tuples.len());
        let mut holes: Vec<Hole> = Vec::new();
        let mut unknown_member = false;
        for tuple in tuples {
            let member = match tuple {
                Tuple::Member(member) => member,
                Tuple::Hole(h) => {
                    unknown_member = true;
                    holes.push(h.clone());
                    continue;
                }
                Tuple::Refused(stop, env) => {
                    if self.adds_zero(m, agg, env.as_ref(), ctx)? {
                        continue;
                    }
                    return Err(stop.clone());
                }
            };
            let Some(t) = self.member_term(m, agg, member, ctx)? else {
                // its `where` is a hole before its term is refused: a hole member
                unknown_member = true;
                holes.extend(member.unknown.clone());
                continue;
            };
            if let Some(unknown) = &member.unknown {
                if agg.func == AggFn::Sum && is_zero(&t) {
                    continue;
                }
                unknown_member = true;
                holes.push(unknown.clone());
                if let Value::Hole(h) = t {
                    holes.push(h);
                }
                continue;
            }
            if let Value::Hole(h) = t {
                holes.push(h);
                continue;
            }
            if t == Value::None {
                return Err(none_term(m, agg.term.as_ref(), &member.env, word));
            }
            if let Value::Window(w) = &t
                && !matches!(
                    agg.func,
                    AggFn::Min | AggFn::Max | AggFn::Argmin | AggFn::Argmax
                )
            {
                // only a comparison decides a window: a term of any other aggregate needs
                // its one value (L §4.4)
                holes.push(window_term_hole(w, word));
                continue;
            }
            terms.push((&member.env, t));
        }
        if !holes.is_empty() {
            if ctx.partial && agg.func == AggFn::Sum && !unknown_member {
                // a `partial` line's sum: its known terms' sum and each hole term; a member
                // whose `where` is a hole isn't a term it is linear in, so that sum is plain
                // (L §5.18)
                let known = terms.into_iter().map(|(_, t)| t).collect();
                let known = sum_terms(m, agg, known)?;
                return partial::lin_sum(m, known, &holes);
            }
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        let pick = match agg.func {
            AggFn::Count => {
                let n = terms
                    .iter()
                    .filter(|(_, t)| *t == Value::Bool(true))
                    .count();
                return Ok(Value::int(
                    i64::try_from(n).map_err(|_| invariant("a count past the integers"))?,
                ));
            }
            AggFn::Sum => {
                let terms = terms.into_iter().map(|(_, t)| t).collect();
                return sum_terms(m, agg, terms);
            }
            AggFn::Argmin | AggFn::Argmax => {
                let key = self.arg_extreme(m, agg, &terms, ctx)?;
                self.winning_term(m, (snap, &key), agg, &terms, ctx)?;
                return Ok(key);
            }
            AggFn::Min => End::Low,
            AggFn::Max => End::High,
            AggFn::First | AggFn::Last | AggFn::Any | AggFn::All => {
                return Err(invariant("an aggregate with no terms to fold"));
            }
        };
        if terms.is_empty() {
            return Err(refuse(Run::Empty, format!("{word} over no keys")));
        }
        if snap.is_none() {
            return min_max(m, terms.into_iter().map(|(_, t)| t).collect(), pick);
        }
        let extreme = min_max(m, terms.iter().map(|(_, t)| t.clone()).collect(), pick)?;
        self.winning_term(m, (snap, &extreme), agg, &terms, ctx)?;
        Ok(extreme)
    }

    /// Whether a `sum` tuple refused at run adds 0: its scope `env` is known, its term there is
    /// known and 0 (tally 2.3.1, rulings 17 statement A: "a `sum` member whose term is known 0
    /// adds 0 whatever its `where`"). L §5.11, §8.
    fn adds_zero(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: Option<&Env>,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        let (AggFn::Sum, Some(term), Some(env)) = (agg.func, &agg.term, env) else {
            return Ok(false);
        };
        match self.ev(m, term, env, ctx) {
            Ok(t) => Ok(is_zero(&t)),
            Err(Stop::Refused(r)) if a_terms_refusal(&r) => Ok(false),
            Err(stop) => Err(stop),
        }
    }

    /// Member `member`'s term of aggregate `agg` (its bare key where it has none); none for a
    /// `sum` member whose `where` is a hole and whose term is refused: a hole member, its
    /// `where` read first (tally 2.3.1, rulings 17 statement A). L §5.11, §8.
    fn member_term(
        &mut self,
        m: &Reality,
        agg: &Agg,
        member: &Member,
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        let Some(term) = &agg.term else {
            return bare_key(agg, &member.env).map(Some);
        };
        match self.ev(m, term, &member.env, ctx) {
            Err(Stop::Refused(r))
                if agg.func == AggFn::Sum && member.unknown.is_some() && a_terms_refusal(&r) =>
            {
                Ok(None)
            }
            read => read.map(Some),
        }
    }

    /// Explains the `min`, `max`, `argmin` or `argmax` of `terms` whose result is `result` by
    /// its first winning term in binding order, where `snap` holds the trace before its terms
    /// were read (refcore's `e_agg`, its `trace_winner` calls). L §5.18, E §10.
    fn winning_term(
        &mut self,
        m: &Reality,
        (snap, result): (Option<Snap>, &Value),
        agg: &Agg,
        terms: &[(&Env, Value)],
        ctx: Ctx,
    ) -> Result<(), Stop> {
        let (Some(snap), Some(term)) = (snap, &agg.term) else {
            return Ok(());
        };
        let keyed = matches!(agg.func, AggFn::Argmin | AggFn::Argmax);
        let mut gives = Vec::with_capacity(terms.len());
        for (env, t) in terms {
            let given = if keyed {
                bare_key(agg, env)?
            } else {
                t.clone()
            };
            gives.push((term, *env, given));
        }
        self.trace_winner(m, (snap, result), &gives, ctx)
    }

    /// `any(c for …)`, `all(c for …)` under Kleene logic (refcore's `agg_anyall`) over its
    /// tuples by statement A: a member that decides (`true` for `any`, `false` for `all`)
    /// decides whatever the others are, refused and hole ones included, so the answer is the
    /// same in every binding order; where none decides, the first refused tuple in binding
    /// order refuses it, else a hole one makes it a hole (tally 2.3.1, #253's option B;
    /// rulings 17). L §5.11, §8, §2.5.
    fn any_all(
        &mut self,
        m: &Reality,
        func: AggFn,
        term: &Expr,
        tuples: &[Tuple],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = func.word();
        let decides = func == AggFn::Any;
        let mut holes: Vec<Hole> = Vec::new();
        let mut refused: Option<Stop> = None;
        for tuple in tuples {
            let member = match tuple {
                Tuple::Member(member) => member,
                Tuple::Hole(h) => {
                    holes.push(h.clone());
                    continue;
                }
                Tuple::Refused(stop, _) => {
                    refused.get_or_insert_with(|| stop.clone());
                    continue;
                }
            };
            let read = match self.ev(m, term, &member.env, ctx) {
                Err(Stop::Refused(r)) if a_terms_refusal(&r) => Err(Stop::Refused(r)),
                Err(stop) => return Err(stop),
                Ok(t) => Ok(t),
            };
            if let Some(unknown) = &member.unknown {
                // its `where` is a hole, read before its term: a hole member
                holes.push(unknown.clone());
                if let Ok(Value::Hole(h)) = read {
                    holes.push(h);
                }
                continue;
            }
            let t = match read {
                Ok(t) => t,
                Err(stop) => {
                    refused.get_or_insert(stop);
                    continue;
                }
            };
            if t == Value::None {
                refused.get_or_insert_with(|| none_term(m, Some(term), &member.env, word));
                continue;
            }
            if let Value::Hole(h) = t {
                holes.push(h);
                continue;
            }
            let Value::Bool(holds) = t else {
                return Err(not_typed());
            };
            if holds == decides {
                return Ok(Value::Bool(decides));
            }
        }
        if let Some(stop) = refused {
            return Err(stop);
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        Ok(Value::Bool(!decides))
    }

    /// The binding tuples in binding order, depth first; the first domain that is a hole
    /// makes the aggregate that hole (refcore's `members`). A bound movement's register goes
    /// where `brings` says: to the reader, or kept for the tuple (refcore's `regs`, an ask's
    /// `for`). L §5.11, §5.19 Bindings, §8.2.
    pub(crate) fn members(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        env: &Env,
        ctx: Ctx,
        brings: Brings,
    ) -> Result<Result<Vec<Member>, Hole>, Stop> {
        let walk = self.tuple_walk(m, binds, env, ctx, (Way::Forward, Reading::Stops))?;
        self.walk_members(m, binds, walk, ctx, brings)
    }

    /// [`Engine::members`] taken from the walk `walk` over the tuples of `binds`, or the hole
    /// of their first domain (refcore's `members`). L §5.11, §5.19 Bindings, §8.2.
    fn walk_members(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        walk: Result<Walk, Hole>,
        ctx: Ctx,
        brings: Brings,
    ) -> Result<Result<Vec<Member>, Hole>, Stop> {
        let mut walk = match walk {
            Err(hole) => return Ok(Err(hole)),
            Ok(walk) => walk,
        };
        let mut out: Vec<Member> = Vec::new();
        let mut domain_hole: Option<Hole> = None;
        loop {
            match self.next_tuple(m, binds, &mut walk, ctx, brings)? {
                Step::Member(member) => out.push(member),
                Step::Hole(h) => {
                    domain_hole.get_or_insert(h);
                }
                Step::Refused(stop, _) => return Err(stop),
                Step::Done => break,
            }
        }
        Ok(domain_hole.map_or(Ok(out), Err))
    }

    /// The tuples of `any`, `all` or `sum` from the walk `walk` over the tuples of `binds`, by
    /// statement A: each in its place, a member, an inner domain's hole or a refusal at run;
    /// or the hole of their first domain, where no tuple is known (refcore's `members`). A
    /// load refusal or a broken invariant stops them where it is met. L §5.11, §8.
    fn walk_terms(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        walk: Result<Walk, Hole>,
        ctx: Ctx,
    ) -> Result<Result<Vec<Tuple>, Hole>, Stop> {
        let mut walk = match walk {
            Err(hole) => return Ok(Err(hole)),
            Ok(walk) => walk,
        };
        let mut out: Vec<Tuple> = Vec::new();
        loop {
            match self.next_tuple(m, binds, &mut walk, ctx, Brings::ToReader)? {
                Step::Member(member) => out.push(Tuple::Member(member)),
                Step::Refused(stop, env) => out.push(Tuple::Refused(stop, env)),
                Step::Hole(h) => out.push(Tuple::Hole(h)),
                Step::Done => break,
            }
        }
        Ok(Ok(out))
    }

    /// The walk over the binding tuples, depth first, each binding's values from the end
    /// `way` says (the last, `last`'s), a tuple refused at run taken as `reading` says; the
    /// hole of a first domain that is one (refcore's `members`). L §5.11.
    pub(super) fn tuple_walk(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        env: &Env,
        ctx: Ctx,
        (way, reading): (Way, Reading),
    ) -> Result<Result<Walk, Hole>, Stop> {
        let Some(first) = binds.first() else {
            return Err(invariant("an aggregate with no binding"));
        };
        let frame = frame_of(self.bind_values(m, first, env, (ctx, reading))?, env);
        Ok(walk_of(frame, (way, reading)))
    }

    /// The frame of the first binding's values in scope `env`, or the hole of a domain that
    /// is one (refcore's `members`, its first binding). L §5.11.
    fn first_frame(
        &mut self,
        m: &Reality,
        bind: &Bind,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Frame, Hole>, Stop> {
        Ok(frame_of(
            self.bind_values(m, bind, env, (ctx, Reading::Stops))?,
            env,
        ))
    }

    /// The walk's next tuple whose `where`s keep it, each `where` read when its binding is
    /// reached; or the hole of an inner domain that is one; or a tuple refused at run; or the
    /// end (refcore's `members`, one step). L §5.11, §5.19 Bindings, §8.2.
    pub(super) fn next_tuple(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        walk: &mut Walk,
        ctx: Ctx,
        brings: Brings,
    ) -> Result<Step, Stop> {
        let reading = walk.reading;
        while let Some(frame) = walk.frames.last_mut() {
            let Some((value, own)) = frame.values.get(frame.next).cloned() else {
                walk.frames.pop();
                continue;
            };
            let brought = frame.colls.get(frame.next).cloned();
            let exists = frame.refused.get(frame.next).cloned().flatten();
            frame.next = frame
                .next
                .checked_add(1)
                .ok_or_else(|| invariant("a binding's position"))?;
            if let Some(r) = exists {
                // a movement whose existence is refused: its domain's read (statement A)
                return Ok(Step::Refused(Stop::Refused(r), None));
            }
            let (level, scope, so_far) = (frame.level, frame.env.clone(), frame.unknown.clone());
            let mut path = frame.brought.clone();
            let bind = binds
                .get(level)
                .ok_or_else(|| invariant("a binding past the list"))?;
            let next = level
                .checked_add(1)
                .ok_or_else(|| invariant("a binding's position"))?;
            let written = self.bind_text(&bind.text);
            let inner = scope.with_bind((&bind.var, &written), value, bind_domain(m, bind));
            let kept = match self.where_keeps(m, bind, &inner, ctx) {
                Err(Stop::Refused(r)) if a_terms_refusal(&r) => {
                    let last = binds.get(next).is_none().then_some(inner);
                    return Ok(Step::Refused(Stop::Refused(r), last));
                }
                kept => kept?,
            };
            let Kept::In(hole) = kept else {
                continue;
            };
            let unknown = joined(joined(so_far, own), hole);
            if let Some(c) = &brought {
                self.bring(brings, &mut path, c);
            }
            let Some(bind_next) = binds.get(next) else {
                return Ok(Step::Member(Member {
                    env: inner,
                    unknown,
                    brought: path,
                }));
            };
            let values = match self.bind_values(m, bind_next, &inner, (ctx, reading)) {
                Err(Stop::Refused(r)) if reading == Reading::Each && a_terms_refusal(&r) => {
                    return Ok(Step::Refused(Stop::Refused(r), None));
                }
                values => values?,
            };
            let deeper = match values {
                Values::Unknown(h) => return Ok(Step::Hole(h)),
                Values::Known(values) => Frame {
                    level: next,
                    env: inner,
                    unknown,
                    brought: path,
                    values,
                    next: 0,
                    colls: Vec::new(),
                    refused: Vec::new(),
                },
                Values::Moves(moves) => moves_frame(next, (inner, unknown, path), moves),
            };
            walk.frames.push(deeper.taken(walk.way));
        }
        Ok(Step::Done)
    }

    /// A binding's values bound one by one in scope `env`, their `where` not yet read: the
    /// candidates `first` and `last` over things or movements order by their `ties by` before
    /// reading it; or the hole of a domain that is one (refcore's `members`, one binding).
    /// L §5.11.
    pub(super) fn bind_candidates(
        &mut self,
        m: &Reality,
        bind: &Bind,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Vec<Candidate>, Hole>, Stop> {
        let frame = match self.first_frame(m, bind, env, ctx)? {
            Err(hole) => return Ok(Err(hole)),
            Ok(frame) => frame,
        };
        let mut colls = frame.colls.into_iter();
        let mut out = Vec::with_capacity(frame.values.len());
        let written = self.bind_text(&bind.text);
        for (value, own) in frame.values {
            out.push(Candidate {
                env: env.with_bind((&bind.var, &written), value, bind_domain(m, bind)),
                own,
                brings: colls.next(),
            });
        }
        Ok(Ok(out))
    }

    /// Whether binding `bind`'s `where` keeps a candidate, with its unknown-member hole; its
    /// movement's register is its reader's to bring, when it reaches it (refcore's `members`,
    /// one value). L §5.11, §8.2.
    pub(super) fn candidate_kept(
        &mut self,
        m: &Reality,
        bind: &Bind,
        cand: &Candidate,
        ctx: Ctx,
    ) -> Result<Kept, Stop> {
        let Kept::In(hole) = self.where_keeps(m, bind, &cand.env, ctx)? else {
            return Ok(Kept::Out);
        };
        Ok(Kept::In(joined(cand.own.clone(), hole)))
    }

    /// A bound movement's register `met`, to the reader's collector or to the tuple's `path`
    /// (refcore's `self.cur.merge(mc)` or `regs + [mc]`). L §5.19 Bindings, §8.2.
    fn bring(&mut self, brings: Brings, path: &mut Collected, met: &Collected) {
        match brings {
            Brings::ToReader => self.absorb(met),
            Brings::ToTuple => path.merge(met),
        }
    }

    /// One binding's members, each with what deciding it read, kept apart from the reader:
    /// the domain's reads, the member's own register and its `where`'s reads; or the hole of
    /// a domain that is one, with what reading it read (refcore's `move_tuples`, its `cd` and
    /// `cw`). A member whose `where` is refused at run is kept with that refusal: its tuple is
    /// a refused instance (L §5.14 Keys, §8.4; tally 2.1.6). L §5.11, E §4.1 step 1.
    ///
    /// # Errors
    ///
    /// The refusals the domain meets; a load refusal a `where` meets at run.
    pub(crate) fn binding_members(
        &mut self,
        m: &Reality,
        bind: &Bind,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Decided, Stop> {
        self.push();
        let values = self.bind_values(m, bind, env, (ctx, Reading::Stops));
        let domain = self.pop_alone();
        let (values, brings) = match values? {
            Values::Unknown(h) => return Ok(Err((h, domain))),
            Values::Known(values) => (values, Vec::new()),
            Values::Moves(moves) => {
                let frame = moves_frame(0, (env.clone(), None, Collected::default()), moves);
                (frame.values, frame.colls)
            }
        };
        let mut out = Vec::with_capacity(values.len());
        let written = self.bind_text(&bind.text);
        for (i, (value, own)) in values.into_iter().enumerate() {
            let mut read = domain.clone();
            if let Some(c) = brings.get(i) {
                read.merge(c);
            }
            let inner = env.with_bind((&bind.var, &written), value, bind_domain(m, bind));
            self.push();
            let kept = self.where_keeps(m, bind, &inner, ctx);
            read.merge(&self.pop_alone());
            let (unknown, refused) = match kept {
                Ok(Kept::Out) => continue,
                Ok(Kept::In(h)) => (joined(own, h), None),
                Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => (None, Some(r)),
                Err(stop) => return Err(stop),
            };
            let member = Member {
                env: inner,
                unknown,
                brought: Collected::default(),
            };
            out.push((member, read, refused));
        }
        Ok(Ok(out))
    }

    /// Whether binding `bind`'s `where` keeps the member bound in `inner`, and the hole it is
    /// where it is one (refcore's `members`, its `where`). L §5.11.
    ///
    /// # Errors
    ///
    /// The refusals the `where` meets; `RUN-NONE` for a `where` that is `none`.
    pub(crate) fn where_keeps(
        &mut self,
        m: &Reality,
        bind: &Bind,
        inner: &Env,
        ctx: Ctx,
    ) -> Result<Kept, Stop> {
        let Some(filter) = &bind.filter else {
            return Ok(Kept::In(None));
        };
        let tested = self.ev(m, filter, inner, ctx)?;
        if let Value::Hole(h) = tested {
            return Ok(Kept::In(Some(h)));
        }
        if tested == Value::None {
            return Err(refuse(Run::None, "a where is none here".to_owned()));
        }
        let Value::Bool(holds) = tested else {
            return Err(not_typed());
        };
        Ok(if holds { Kept::In(None) } else { Kept::Out })
    }
}

/// The frame of the first binding's values `values` in scope `env`, or the hole of a domain
/// that is one (refcore's `members`, its first binding). L §5.11.
fn frame_of(values: Values, env: &Env) -> Result<Frame, Hole> {
    Ok(match values {
        Values::Unknown(h) => return Err(h),
        Values::Known(values) => Frame {
            level: 0,
            env: env.clone(),
            unknown: None,
            brought: Collected::default(),
            values,
            next: 0,
            colls: Vec::new(),
            refused: Vec::new(),
        },
        Values::Moves(moves) => moves_frame(0, (env.clone(), None, Collected::default()), moves),
    })
}

/// The walk over the binding tuples from the first binding's frame `first`, its values taken
/// from the end `way` says, a tuple refused at run taken as `reading` says; or the hole of a
/// first domain that is one (refcore's `members`). L §5.11.
fn walk_of(first: Result<Frame, Hole>, (way, reading): (Way, Reading)) -> Result<Walk, Hole> {
    first.map(|frame| Walk {
        frames: vec![frame.taken(way)],
        way,
        reading,
    })
}

/// Whether refusal `r` of a tuple's domain, `where` or term is one a member is read as: every
/// run refusal; a load refusal and a broken invariant stop the read (L §8, §5.11).
fn a_terms_refusal(r: &Refusal) -> bool {
    !matches!(r.code(), Code::Load(_) | Code::EngineInvariant)
}

/// `count(k in ks)`: the tuples, unless one is an unknown member (L §5.11).
fn count_members(members: &[Member]) -> Result<Value, Stop> {
    let holes: Vec<Option<&Hole>> = members.iter().map(|mm| mm.unknown.as_ref()).collect();
    if let Some(h) = spread(&holes) {
        return Ok(Value::Hole(h));
    }
    let n = i64::try_from(members.len()).map_err(|_| invariant("a count past the integers"))?;
    Ok(Value::int(n))
}

/// The hole of a term of aggregate `word` that is a ranged date, window `w`: the term needs
/// its one value, a read off the readers' list, so `HOLE-IMPRECISE` named by the window's
/// input and carrying its unknown days (refcore's `carry_hole`; §8.2 `unknown day:`). L §4.4
/// ("Every other read is the hole").
pub(super) fn window_term_hole(w: &Window, word: &str) -> Hole {
    let window = w.question_text();
    let text = format!("date within {window} not precise enough for a term of {word}");
    Hole::asked(&text, w.figure()).carrying(&w.carry)
}

/// The refusal of a term of aggregate `word` that is `none`: its term as written, each
/// variable by its value, `rate[k2] is none here: an aggregate`; a bare key, never `none`,
/// keeps refcore's line. L §8.4 `RUN-NONE`'s row, §5.11 (pick P3).
fn none_term(m: &Reality, term: Option<&Expr>, env: &Env, word: &str) -> Stop {
    match term {
        Some(e) => none_here(m, e, env, "aggregate"),
        None => refuse(Run::None, format!("a term of {word} is none here")),
    }
}

/// Whether a term is zero: an unknown member's 0 adds nothing to a sum (L §5.11).
fn is_zero(v: &Value) -> bool {
    if let Value::Number(n) = v {
        return n.q.is_zero();
    }
    if let Value::Money(money) = v {
        return money.is_zero();
    }
    false
}

/// The sum of known terms, left to right; 0 in the type load gave the sum over none
/// (refcore's `zero_of`). L §5.11.
fn sum_terms(m: &Reality, agg: &Agg, terms: Vec<Value>) -> Result<Value, Stop> {
    let mut it = terms.into_iter();
    let Some(first) = it.next() else {
        let ty = agg
            .ty
            .get()
            .ok_or_else(|| invariant("a sum load didn't type"))?;
        return zero_in(m, &ty);
    };
    let mut acc = first;
    for t in it {
        acc = arith(m, Op::Add, &acc, &t)?;
    }
    Ok(acc)
}

/// 0 in type `ty`, the type load's checker gave a sum (refcore's `zero_of`): money 0, a
/// `num u` 0, an int 0 for a bare figure, a key, a unit only the run knows or a unit over a
/// kind's base (what the 0 meets gives it that). L §5.11, §3.2, §7.1.
///
/// # Errors
///
/// `RUN-DOMAIN` for a sum of rates (a rate carries its period, and there is none);
/// `NotBuilt` for a type load's checker doesn't work out.
fn zero_in(m: &Reality, ty: &Ty) -> Result<Value, Stop> {
    match ty {
        Ty::Money(cur) => {
            let id = currency_id(m, cur).map_err(num_stop)?;
            Ok(Value::Money(Money::from_minor(0, id)))
        }
        Ty::Num(unit, _)
            if unit
                .terms()
                .iter()
                .any(|(base, _)| m.kinds.get(base).is_some()) =>
        {
            Ok(Value::int(0))
        }
        Ty::Num(unit, shown) => Ok(Value::Number(Number {
            q: Num::zero(),
            class: Class::Num(unit.clone(), *shown),
        })),
        Ty::Int | Ty::Flex(_) | Ty::Year4 | Ty::Key(_) | Ty::KeyLit(_) | Ty::Dyn => {
            Ok(Value::int(0))
        }
        Ty::Rate => Err(refuse(
            Run::Domain,
            "the sum of rates over no keys (a rate carries its period)".to_owned(),
        )),
        Ty::Bool
        | Ty::Text
        | Ty::Date
        | Ty::Dates
        | Ty::Period(_)
        | Ty::Dist(_)
        | Ty::DistKeys(_)
        | Ty::Calendar
        | Ty::Code(_)
        | Ty::MonthDay
        | Ty::Record(_)
        | Ty::List(_)
        | Ty::Tuple(_)
        | Ty::KeyList
        | Ty::UnitKey
        | Ty::Unseen
        | Ty::None
        | Ty::Hole
        | Ty::Never
        | Ty::KindName => Err(Stop::NotBuilt(
            "a sum over no keys of a type load's checker doesn't work out",
        )),
    }
}

/// Two unknown-member holes as one.
fn joined(a: Option<Hole>, b: Option<Hole>) -> Option<Hole> {
    match (a, b) {
        (Some(x), Some(y)) => spread(&[Some(&x), Some(&y)]),
        (x, None) => x,
        (None, y) => y,
    }
}

#[cfg(test)]
mod tests {
    use super::joined;
    use crate::api::{Asks, Files, Given, Request, run};
    use crate::hole::Hole;

    /// `any` and `all` over three keys whose first term is refused (a division by zero),
    /// whose second decides, and the same with no deciding term.
    const ANY_ALL: &str = r#"model any_all
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source s "invented values"

index ks = [c, a, b]

input t[k in ks]: int

input ten: int

def r[k in ks]: bool
  rule one = ten / t[k] > t[b]

ask q_any: any(r[k] for k in ks)
ask q_all: all(r[k] for k in ks)
ask q_none: any(r[k] for k in ks where k != a)
ask q_where_any: any(t[k] > 0 for k in ks where ten / t[k] > 0)
ask q_where_all: all(t[k] > 1 for k in ks where ten / t[k] > 0)
ask q_where_none: any(t[k] > t[b] for k in ks where ten / t[k] > 0)
ask q_where_sum: sum(t[k] for k in ks where ten / t[k] > 0)

version v
  cite s
  governs always
  known 2026-01-01
  ten = 10 "s"
  t[a] = 1 "s"
  t[b] = 5 "s"
  t[c] = 0 "s"
"#;

    #[test]
    fn a_deciding_term_decides_any_and_all_past_a_refused_one() {
        // L §5.11, §8 (tally 2.3.1, #253's option B): r[c] is refused, r[a] true, r[b]
        // false; `any` is true and `all` false whatever the binding order, and with no
        // deciding term the refusal stands; a refused `where` is such a term, and a `sum`
        // member whose term is known 0 adds 0 past it (rulings 17 statement A)
        let given = vec![Given {
            path: "any_all.tally".to_owned(),
            bytes: Some(ANY_ALL.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Every);
        let answer = run(Files::new(given), &request).expect("an answer");
        let values: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("q_"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            values,
            [
                "q_all: false",
                "q_any: true",
                "q_none: refused: RUN-DIV division by zero",
                "q_where_all: false",
                "q_where_any: true",
                "q_where_none: refused: RUN-DIV division by zero",
                "q_where_sum: 6"
            ],
            "the deciding term decides; a refusal alone refuses"
        );
    }

    /// Tuples refused by a `where`, an inner domain that is a hole, and a movement whose
    /// `when` is refused, each beside a member that decides.
    const STATEMENT_A: &str = r#"model statement_a
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source s "invented values"

unit PLN: currency minor 0.01

index ns = 1..3

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input x[n in ns]: int

input dh[n in ns]: date

input z: int

input amt: money PLN

input small: money PLN

input d2: date

move paid: transfers
  on d2
  leg cash from world to konto amount amt

move maybe: transfers
  on d2
  when x[3] / z > 0
  leg cash from world to konto amount amt

ask q_dom: any(x[n] > 0 for n in ns where x[3] / x[n] > 0, p in accounts on dh[n])
ask q_dom_all: all(x[n] < 0 for n in ns where x[3] / x[n] > 0, p in accounts on dh[n])
ask q_moves: any(t.cash.amount > small for t in transfers over 2026-03)
ask q_count: count(t in transfers over 2026-03)

version v
  cite s
  governs always
  known 2026-01-01
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"
  x[1] = 0 "s"
  x[2] = 5 "s"
  x[3] = 5 "s"
  dh[1] = 2026-02-01 "s"
  dh[2] = 2026-02-01 "s"
  dh[3] = ?"dh3?" "s"
  z = 0 "s"
  amt = 30.00 PLN "s"
  small = 10.00 PLN "s"
  d2 = 2026-03-02 "s"
"#;

    #[test]
    fn every_tuple_is_one_member_of_any_and_all() {
        // tally 2.3.1, rulings 17 statement A: n=1's `where` is refused, n=3's inner domain
        // is a hole, `maybe`'s existence is refused; the member that decides decides past
        // each, and `count` past a refused key still refuses (L §5.11, §8)
        let given = vec![Given {
            path: "statement_a.tally".to_owned(),
            bytes: Some(STATEMENT_A.as_bytes().to_vec()),
        }];
        let answer = run(Files::new(given), &Request::new(Asks::Every)).expect("an answer");
        let lines: Vec<&str> = answer
            .lines()
            .iter()
            .map(String::as_str)
            .filter(|line| !line.starts_with("  cut"))
            .collect();
        assert_eq!(
            lines,
            [
                "q_count: refused: RUN-DIV division by zero",
                "q_dom: true",
                "q_dom_all: false",
                "q_moves: true",
                "  unfulfilled: paid"
            ],
            "the deciding member decides"
        );
    }

    #[test]
    fn unknown_members_join_their_questions() {
        let a = Hole::asked("a?", "x");
        let b = Hole::asked("b?", "y");
        let both = joined(Some(a.clone()), Some(b.clone())).expect("a hole");
        let other_way = joined(Some(b), Some(a.clone())).expect("a hole");
        assert_eq!(both, other_way, "joined in either order");
        assert_eq!(
            both.question_texts(),
            vec!["a?".to_owned(), "b?".to_owned()],
            "both questions"
        );
        assert_eq!(joined(Some(a.clone()), None), Some(a), "one side");
        assert_eq!(joined(None, None), None, "a member");
    }
}
