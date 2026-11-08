//! Aggregates over bindings: tuples in binding order, the unknown members a hole `where` makes, and each aggregate's rule for holes.
//! Spec: L §5.11 (aggregates; a key whose `where` is a hole is an unknown member; `sum` over
//! no keys is 0 in the term's type; `count`, `any`, `all`), §8.1 (holes spread). The values a
//! binding ranges over are `bindings.rs`'s; the extremes, `min` to `last`, `extremes.rs`'s.
//! Twin: engine/tools/refcore/evaluate.py (`members`, `zero_of`, `e_agg`, `agg_anyall`).
//! Invariants: tuples come in binding order, depth first as refcore's generator, one at a
//! time, so `first` and `last` (from the last) read no `where` past their key; an unknown
//! member's term counts only as its questions, except a 0 in a sum; a domain that is a hole
//! makes the aggregate that hole; a binding over movements or an observed input takes its
//! members from the book (`book/members.rs`), and a member's register reaches the reader only
//! when the member passes the `where`: an aggregate's reader, or an ask's line for that
//! tuple alone.

use super::arith::{Op, arith};
use super::bindings::{Values, bind_domain};
use super::expr::hole_of;
use super::extremes::{bare_key, min_max};
use super::state::{Collected, Ctx, Engine, Env, invariant, not_typed, refuse};
use super::ty::zero_of;
use super::windows::End;
use crate::base::{Code, Refusal, Run};
use crate::hole::{Hole, spread};
use crate::model::{Reality, Value};
use crate::syntax::{Agg, AggFn, Bind, Expr, Stop};

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
pub(super) enum Kept {
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
    for mv in moves {
        values.push((mv.value, mv.unknown));
        colls.push(mv.coll);
    }
    Frame {
        level,
        env,
        unknown,
        brought,
        values,
        next: 0,
        colls,
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
}

/// What a walk's next step meets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Step {
    /// A tuple whose `where`s keep it.
    Member(Member),
    /// An inner binding's domain that is a hole.
    Hole(Hole),
    /// No tuple is left.
    Done,
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
    /// An aggregate (refcore's `e_agg`). L §5.11.
    pub(super) fn aggregate(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        if let AggFn::First | AggFn::Last = agg.func {
            return self.first_last(m, agg, env, ctx);
        }
        let members = match self.members(m, &agg.binds, env, ctx, Brings::ToReader)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(members) => members,
        };
        if let (AggFn::Count, None) = (agg.func, &agg.term) {
            return count_members(&members);
        }
        if let AggFn::Any | AggFn::All = agg.func {
            let term = agg
                .term
                .as_ref()
                .ok_or_else(|| invariant("any or all with no term"))?;
            return self.any_all(m, agg.func, term, &members, ctx);
        }
        self.terms_aggregate(m, agg, &members, env, ctx)
    }

    /// `count`, `sum`, `min`, `max`, `argmin`, `argmax` over their terms (refcore's
    /// `e_agg`). L §5.11.
    fn terms_aggregate(
        &mut self,
        m: &Reality,
        agg: &Agg,
        members: &[Member],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg.func.word();
        let mut terms: Vec<(&Env, Value)> = Vec::with_capacity(members.len());
        let mut holes: Vec<Hole> = Vec::new();
        for member in members {
            let t = match &agg.term {
                None => bare_key(agg, &member.env)?,
                Some(term) => self.ev(m, term, &member.env, ctx)?,
            };
            if let Some(unknown) = &member.unknown {
                if agg.func == AggFn::Sum && is_zero(&t) {
                    continue;
                }
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
                return Err(refuse(Run::None, format!("a term of {word} is none here")));
            }
            terms.push((&member.env, t));
        }
        if !holes.is_empty() {
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
                return sum_terms(m, agg, terms, env, ctx);
            }
            AggFn::Argmin | AggFn::Argmax => return self.arg_extreme(m, agg, &terms, ctx),
            AggFn::Min => End::Low,
            AggFn::Max => End::High,
            AggFn::First | AggFn::Last | AggFn::Any | AggFn::All => {
                return Err(invariant("an aggregate with no terms to fold"));
            }
        };
        if terms.is_empty() {
            return Err(refuse(Run::Empty, format!("{word} over no keys")));
        }
        min_max(m, terms.into_iter().map(|(_, t)| t).collect(), pick)
    }

    /// `any(c for …)`, `all(c for …)` under Kleene logic (refcore's `agg_anyall`). L §5.11.
    fn any_all(
        &mut self,
        m: &Reality,
        func: AggFn,
        term: &Expr,
        members: &[Member],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = func.word();
        let decides = func == AggFn::Any;
        let mut holes: Vec<Hole> = Vec::new();
        for member in members {
            let t = self.ev(m, term, &member.env, ctx)?;
            if t == Value::None {
                return Err(refuse(Run::None, format!("a term of {word} is none here")));
            }
            if let Some(unknown) = &member.unknown {
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
            let Value::Bool(holds) = t else {
                return Err(not_typed());
            };
            if holds == decides {
                return Ok(Value::Bool(decides));
            }
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
        let mut walk = match self.tuple_walk(m, binds, env, ctx, Way::Forward)? {
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
                Step::Done => break,
            }
        }
        Ok(domain_hole.map_or(Ok(out), Err))
    }

    /// The walk over the binding tuples, depth first, each binding's values from the end
    /// `way` says (the last, `last`'s); the hole of a first domain that is one (refcore's
    /// `members`). L §5.11.
    pub(super) fn tuple_walk(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        env: &Env,
        ctx: Ctx,
        way: Way,
    ) -> Result<Result<Walk, Hole>, Stop> {
        let Some(first) = binds.first() else {
            return Err(invariant("an aggregate with no binding"));
        };
        Ok(self.first_frame(m, first, env, ctx)?.map(|frame| Walk {
            frames: vec![frame.taken(way)],
            way,
        }))
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
        Ok(Ok(match self.bind_values(m, bind, env, ctx)? {
            Values::Unknown(h) => return Ok(Err(h)),
            Values::Known(values) => Frame {
                level: 0,
                env: env.clone(),
                unknown: None,
                brought: Collected::default(),
                values,
                next: 0,
                colls: Vec::new(),
            },
            Values::Moves(moves) => {
                moves_frame(0, (env.clone(), None, Collected::default()), moves)
            }
        }))
    }

    /// The walk's next tuple whose `where`s keep it, each `where` read when its binding is
    /// reached; or the hole of an inner domain that is one; or the end (refcore's `members`,
    /// one step). L §5.11, §5.19 Bindings, §8.2.
    pub(super) fn next_tuple(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        walk: &mut Walk,
        ctx: Ctx,
        brings: Brings,
    ) -> Result<Step, Stop> {
        while let Some(frame) = walk.frames.last_mut() {
            let Some((value, own)) = frame.values.get(frame.next).cloned() else {
                walk.frames.pop();
                continue;
            };
            let brought = frame.colls.get(frame.next).cloned();
            frame.next = frame
                .next
                .checked_add(1)
                .ok_or_else(|| invariant("a binding's position"))?;
            let (level, scope, unknown) = (frame.level, frame.env.clone(), frame.unknown.clone());
            let mut path = frame.brought.clone();
            let bind = binds
                .get(level)
                .ok_or_else(|| invariant("a binding past the list"))?;
            let Some((inner, unknown)) =
                self.take_value(m, bind, &scope, value, joined(unknown, own), ctx)?
            else {
                continue;
            };
            if let Some(c) = &brought {
                self.bring(brings, &mut path, c);
            }
            let next = level
                .checked_add(1)
                .ok_or_else(|| invariant("a binding's position"))?;
            let Some(bind_next) = binds.get(next) else {
                return Ok(Step::Member(Member {
                    env: inner,
                    unknown,
                    brought: path,
                }));
            };
            let deeper = match self.bind_values(m, bind_next, &inner, ctx)? {
                Values::Unknown(h) => return Ok(Step::Hole(h)),
                Values::Known(values) => Frame {
                    level: next,
                    env: inner,
                    unknown,
                    brought: path,
                    values,
                    next: 0,
                    colls: Vec::new(),
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
        for (value, own) in frame.values {
            out.push(Candidate {
                env: env.with(&bind.var, value, bind_domain(m, bind)),
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
        let values = self.bind_values(m, bind, env, ctx);
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
        for (i, (value, own)) in values.into_iter().enumerate() {
            let mut read = domain.clone();
            if let Some(c) = brings.get(i) {
                read.merge(c);
            }
            let inner = env.with(&bind.var, value, bind_domain(m, bind));
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

    /// A value of binding `bind` bound, its `where` tested: none when the `where` is false
    /// (refcore's `members`, one value). L §5.11.
    fn take_value(
        &mut self,
        m: &Reality,
        bind: &Bind,
        scope: &Env,
        value: Value,
        unknown: Option<Hole>,
        ctx: Ctx,
    ) -> Result<Option<(Env, Option<Hole>)>, Stop> {
        let inner = scope.with(&bind.var, value, bind_domain(m, bind));
        Ok(match self.where_keeps(m, bind, &inner, ctx)? {
            Kept::Out => None,
            Kept::In(h) => Some((inner, joined(unknown, h))),
        })
    }

    /// Whether binding `bind`'s `where` keeps the member bound in `inner`, and the hole it is
    /// where it is one (refcore's `members`, its `where`). L §5.11.
    fn where_keeps(
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

/// `count(k in ks)`: the tuples, unless one is an unknown member (L §5.11).
fn count_members(members: &[Member]) -> Result<Value, Stop> {
    let holes: Vec<Option<&Hole>> = members.iter().map(|mm| mm.unknown.as_ref()).collect();
    if let Some(h) = spread(&holes) {
        return Ok(Value::Hole(h));
    }
    let n = i64::try_from(members.len()).map_err(|_| invariant("a count past the integers"))?;
    Ok(Value::int(n))
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

/// The sum of known terms, left to right; 0 in the term's type over none (refcore's
/// `zero_of`). L §5.11.
fn sum_terms(
    m: &Reality,
    agg: &Agg,
    terms: Vec<Value>,
    env: &Env,
    ctx: Ctx,
) -> Result<Value, Stop> {
    let mut it = terms.into_iter();
    let Some(first) = it.next() else {
        let term = agg
            .term
            .as_ref()
            .ok_or_else(|| invariant("a sum with no term"))?;
        return zero_of(m, term, env, &agg.binds, ctx);
    };
    let mut acc = first;
    for t in it {
        acc = arith(m, Op::Add, &acc, &t)?;
    }
    Ok(acc)
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
    use crate::hole::Hole;

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
