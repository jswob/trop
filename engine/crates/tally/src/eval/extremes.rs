//! The extremes over bindings: `min` and `max`, `argmin` and `argmax`, `first` and `last`, and the `ties by` that breaks a tie among them.
//! Spec: L §5.11 (aggregates: `min`/`max`/`argmin`/`argmax` over no keys are `RUN-EMPTY`,
//! a hole if any term is; `first` reads its `where` from the first binding, `last` from the
//! last, each stopping at the first key whose `where` is true, a hole if a hole comes before
//! it, `HOLE-NOTYET` over a binding empty at the cut whose span reaches past it, else
//! `RUN-EMPTY`; ties are stated: `ties by (e1, e2)` compares e1, then e2 among those still
//! tied, `RUN-TIE` without one, and over things, movements or an open index's keys the `ties
//! by` term is the order `first` and `last` run in), §4.4 (windows: `min`/`max` give the
//! interval of the ends' extremes, `argmin`/`argmax` the key decided below or above every
//! other, two reads of one item one value, else `HOLE-IMPRECISE`, as an undecided comparison
//! in a `ties by`), §5.6, §5.7 (things, movements and an open index's keys have no order a
//! rule can read), §3.1, §8.1 (holes spread), §8.4 (`RUN-TIE`, `RUN-EMPTY`, `RUN-NONE`).
//! Twin: engine/tools/refcore/evaluate.py (`agg_minmax`, `extreme`, `agg_arg`, `tie_break`,
//! `agg_first`, `tie_groups`, `notyet_bind`, `unordered_what`).
//! Invariants: a tie is broken only by the written `ties by`, never by index order or binding
//! order; a `ties by` term is read only among the candidates still tied, and a hole there
//! makes the result that hole (L §8.1; refcore reads every term, #116); every order an
//! extreme takes is decided by its comparisons, never assumed, and a window's hole comes only
//! from a comparison that decides the extreme; a `ties by` is read only for values the
//! `where` keeps; `first` and `last` bring no register past the key they return, and over
//! bindings in binding order read no `where` past it (#117).

use super::aggregate::{Brings, Candidate, Kept, Step, Way};
use super::arith::{Test, compare};
use super::bindings::{day_of_binding, reads_name, span_reads};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, a_day, invariant, num_stop, refuse};
use super::windows::{End, as_window, decided_order, imprecise, one_read, window_end};
use crate::base::Run;
use crate::hole::{Hole, spread};
use crate::model::{IndexForm, KindSpec, Reality, Value, Window};
use crate::syntax::{Agg, AggFn, Bind, Domain as Over, Expr, Stop};
use core::cmp::Ordering;

/// A binding with no order a rule can read, by what it ranges over (refcore's
/// `unordered_what`). L §5.7, §5.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unordered {
    /// A kind's things.
    Things,
    /// An event kind's or an observed input's movements.
    Movements,
    /// An open index's keys.
    OpenKeys,
}

impl Unordered {
    /// The words `RUN-TIE` names it by (L §8.4).
    fn word(self) -> &'static str {
        match self {
            Unordered::Things => "things",
            Unordered::Movements => "movements",
            Unordered::OpenKeys => "open-index keys",
        }
    }
}

/// What a binding ranges over when it has no order a rule can read, else none (refcore's
/// `unordered_what`). L §5.7, §5.6.
fn unordered(m: &Reality, bind: &Bind) -> Option<Unordered> {
    let Over::Name { id, qual: None } = &bind.domain else {
        return None;
    };
    if let Some(ix) = m.indexes.get(id) {
        return (ix.form == IndexForm::Open).then_some(Unordered::OpenKeys);
    }
    if let Some(kind) = m.kinds.get(id) {
        if kind.spec == KindSpec::Event {
            return Some(Unordered::Movements);
        }
        return Some(Unordered::Things);
    }
    m.inputs
        .get(id)
        .is_some_and(|i| i.observed)
        .then_some(Unordered::Movements)
}

/// The `min` or `max` of known values: of windows, the window from the extreme of their
/// lowest ends to the extreme of their highest, one value where those are equal (refcore's
/// `agg_minmax`, `extreme`). L §5.11, §4.4.
///
/// # Errors
///
/// The refusals of comparing the values; an invariant for no value, which every caller
/// refuses first (`RUN-EMPTY`, the call's arity).
pub(super) fn min_max(m: &Reality, vals: Vec<Value>, pick: End) -> Result<Value, Stop> {
    let Some(w) = vals.iter().find_map(as_window) else {
        return extreme_of(m, vals, pick);
    };
    let src = w.src.clone();
    let los: Vec<Value> = vals.iter().map(|v| window_end(v, End::Low)).collect();
    let his: Vec<Value> = vals.iter().map(|v| window_end(v, End::High)).collect();
    let (lo, hi) = (extreme_of(m, los, pick)?, extreme_of(m, his, pick)?);
    if compare(m, &lo, &hi, Test::Order)? == Ordering::Equal {
        return Ok(lo);
    }
    Ok(Value::Window(Box::new(Window {
        lo,
        hi,
        days: None,
        src,
        figure: None,
        read: None,
    })))
}

/// The least (`End::Low`) or greatest of values none of which is a window, the first of
/// equals (refcore's `extreme`). L §5.11.
fn extreme_of(m: &Reality, vals: Vec<Value>, pick: End) -> Result<Value, Stop> {
    let mut best: Option<Value> = None;
    for v in vals {
        best = Some(match best {
            None => v,
            Some(held) => {
                let order = compare(m, &v, &held, Test::Order)?;
                let better = match pick {
                    End::Low => order == Ordering::Less,
                    End::High => order == Ordering::Greater,
                };
                if better { v } else { held }
            }
        });
    }
    best.ok_or_else(|| invariant("an extreme of no values"))
}

/// A bare binding's value in scope `env`: its one variable's (refcore's `bare_key`). L §5.11.
pub(super) fn bare_key(agg: &Agg, env: &Env) -> Result<Value, Stop> {
    let [bind] = agg.binds.as_slice() else {
        return Err(Stop::NotBuilt("a bare aggregate over a tuple of keys"));
    };
    env.get(&bind.var)
        .map(|v| v.value.clone())
        .ok_or_else(|| invariant("a bound variable the scope lacks"))
}

/// The keys of tied candidates as `RUN-TIE` lists them, `b, c` (refcore's `fmt` of each
/// `bare_key`). L §8.4.
fn tied_keys(m: &Reality, agg: &Agg, envs: &[&Env]) -> Result<String, Stop> {
    let mut out = Vec::with_capacity(envs.len());
    for env in envs {
        out.push(bare_key(agg, env)?.text(m).map_err(num_stop)?);
    }
    Ok(out.join(", "))
}

/// The `RUN-TIE` of tied candidates that the `ties by` leaves tied (refcore's `tie_break`,
/// `agg_first`). L §5.11, §8.4.
fn unbroken(m: &Reality, agg: &Agg, envs: &[&Env]) -> Result<Stop, Stop> {
    Ok(refuse(
        Run::Tie,
        format!(
            "{} has a tie at {} that changes the result, and its ties by doesn't break it",
            agg.func.word(),
            tied_keys(m, agg, envs)?
        ),
    ))
}

/// Candidates by their position, each with a value of a `ties by` term.
pub(super) type Scored = Vec<(usize, Value)>;

/// The order of two `ties by` values where their comparison decides it, else the
/// `HOLE-IMPRECISE` of the window between them (L §4.4, §5.11; pick P33).
fn tie_order(
    m: &Reality,
    (a, b): (&Value, &Value),
    word: &str,
) -> Result<Result<Ordering, Hole>, Stop> {
    if let Some(order) = decided_order(m, a, b)? {
        return Ok(Ok(order));
    }
    let w = as_window(a)
        .or_else(|| as_window(b))
        .ok_or_else(|| invariant("an undecided order with no window"))?;
    Ok(Err(imprecise(m, w, &format!("the ties by of {word}"))?))
}

/// The scored candidates whose value is the least (`Way::Forward`) or the greatest, in the
/// order given, and the rest: the extreme found by comparing each value with the extreme so
/// far, a comparison that doesn't decide passed over, then compared with every other value,
/// so only a comparison that decides the extreme can make the result a window's hole (L §5.11
/// "the one with the least e", §4.4; pick P33). The later value is compared with the
/// earlier, as refcore compares them.
pub(super) fn pick_extreme(
    m: &Reality,
    scored: Scored,
    way: Way,
    word: &str,
) -> Result<Result<(Vec<usize>, Scored), Hole>, Stop> {
    let wanted = match way {
        Way::Forward => Ordering::Less,
        Way::Back => Ordering::Greater,
    };
    let mut best: Option<usize> = None;
    for (i, (_, v)) in scored.iter().enumerate() {
        let Some((_, held)) = best.and_then(|b| scored.get(b)) else {
            best = Some(i);
            continue;
        };
        if matches!(tie_order(m, (v, held), word)?, Ok(order) if order == wanted) {
            best = Some(i);
        }
    }
    let Some(b) = best else {
        return Ok(Ok((Vec::new(), Vec::new())));
    };
    let (_, extreme) = scored
        .get(b)
        .cloned()
        .ok_or_else(|| invariant("an extreme past its candidates"))?;
    let (mut group, mut rest): (Vec<usize>, Scored) = (Vec::new(), Vec::new());
    for (i, (at, v)) in scored.into_iter().enumerate() {
        if i == b {
            group.push(at);
            continue;
        }
        let order = match tie_order(m, (&v, &extreme), word)? {
            Err(hole) => return Ok(Err(hole)),
            Ok(order) => order,
        };
        if order == wanted {
            return Err(invariant("a value beyond the extreme"));
        }
        if order == Ordering::Equal {
            group.push(at);
        } else {
            rest.push((at, v));
        }
    }
    Ok(Ok((group, rest)))
}

/// A group of candidates in a `ties by` order, by position: the term that orders it next and,
/// once read, that term's values (L §5.11).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Group {
    /// The candidates, in binding order.
    members: Vec<usize>,
    /// The term that orders them next.
    term: usize,
    /// That term's values, once read.
    scored: Option<Scored>,
}

/// The candidates of a `ties by` by position, in its order: the groups still to visit, the
/// next on top (L §5.11).
#[derive(Debug, Clone, PartialEq, Eq)]
struct TieOrder {
    /// The groups, the next to visit last.
    groups: Vec<Group>,
    /// The end the order is visited from: its end, as `last` reads it, or its start.
    way: Way,
}

impl TieOrder {
    /// All `n` candidates in one group, the first term to order it (L §5.11).
    fn of(n: usize, way: Way) -> TieOrder {
        TieOrder {
            groups: vec![Group {
                members: (0..n).collect(),
                term: 0,
                scored: None,
            }],
            way,
        }
    }
}

/// What `first` or `last` met: its key; the holes met before it, or with no key after them;
/// or nothing.
enum Met {
    /// The key it returns.
    Key(Value),
    /// The holes that come before the key, or with none.
    Holes(Vec<Hole>),
    /// No key and no hole.
    Nothing,
}

/// The holes met, as one (L §8.1: each question once).
fn holes_of(holes: &[Hole]) -> Result<Value, Stop> {
    let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
    hole_of(&operands)
}

impl Engine {
    /// `argmin(t for b)`, `argmax(t for b)` over the known terms, each with its scope: the
    /// key whose term is the extreme, a tie broken by `ties by`; of windows, the key decided
    /// below (above) every other, else `HOLE-IMPRECISE` (refcore's `agg_arg`). L §5.11, §4.4.
    pub(super) fn arg_extreme(
        &mut self,
        m: &Reality,
        agg: &Agg,
        terms: &[(&Env, Value)],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg.func.word();
        if terms.is_empty() {
            return Err(refuse(Run::Empty, format!("{word} over no keys")));
        }
        let wanted = if agg.func == AggFn::Argmin {
            Ordering::Less
        } else {
            Ordering::Greater
        };
        let Some(best) = extreme_keys(m, terms, wanted)? else {
            let w = terms
                .iter()
                .find_map(|(_, t)| as_window(t))
                .ok_or_else(|| invariant("an undecided extreme with no window"))?;
            return Ok(Value::Hole(imprecise(m, w, word)?));
        };
        let mut tied: Vec<&Env> = Vec::with_capacity(best.len());
        for at in best {
            let (env, _) = terms
                .get(at)
                .ok_or_else(|| invariant("an extreme past its terms"))?;
            tied.push(*env);
        }
        match self.tie_break(m, agg, &tied, ctx)? {
            Ok(at) => bare_key(
                agg,
                tied.get(at)
                    .ok_or_else(|| invariant("a tie broken past its candidates"))?,
            ),
            Err(hole) => Ok(Value::Hole(hole)),
        }
    }

    /// The position in `tied` of the one candidate whose `ties by` is least, each term read
    /// only among those the earlier left tied; with one candidate, it (refcore's
    /// `tie_break`). L §5.11, §8.4.
    ///
    /// # Errors
    ///
    /// `RUN-TIE` for a tie with no `ties by`, or one it doesn't break.
    fn tie_break(
        &mut self,
        m: &Reality,
        agg: &Agg,
        tied: &[&Env],
        ctx: Ctx,
    ) -> Result<Result<usize, Hole>, Stop> {
        if tied.len() == 1 {
            return Ok(Ok(0));
        }
        if agg.ties.is_none() {
            return Err(refuse(
                Run::Tie,
                format!(
                    "{} has a tie at {} that changes the result, and no ties by",
                    agg.func.word(),
                    tied_keys(m, agg, tied)?
                ),
            ));
        }
        let mut order = TieOrder::of(tied.len(), Way::Forward);
        let group = match self.next_tied(m, agg, tied, &mut order, ctx)? {
            Err(hole) => return Ok(Err(hole)),
            Ok(group) => group.ok_or_else(|| invariant("a tie with no candidates"))?,
        };
        let [one] = group.as_slice() else {
            let mut envs: Vec<&Env> = Vec::with_capacity(group.len());
            for at in &group {
                let env = tied
                    .get(*at)
                    .ok_or_else(|| invariant("a tied candidate past the list"))?;
                envs.push(*env);
            }
            return Err(unbroken(m, agg, &envs)?);
        };
        Ok(Ok(*one))
    }

    /// The next group of candidates in the `ties by` order, from its end for `last`: of a
    /// group with more than one candidate, the term read once and the least (greatest) of it
    /// taken out, ordered by the next term, the rest left for later; so e2 is read only among
    /// those e1 left tied, and a group is ordered only when it is reached. A term that is a
    /// hole for one of them, or a comparison of a window that doesn't decide the extreme, is
    /// the result's hole (refcore's `tie_groups`, `tie_break`). L §5.11, §4.4, §8.1.
    fn next_tied(
        &mut self,
        m: &Reality,
        agg: &Agg,
        envs: &[&Env],
        order: &mut TieOrder,
        ctx: Ctx,
    ) -> Result<Result<Option<Vec<usize>>, Hole>, Stop> {
        let Some(ties) = &agg.ties else {
            return Err(invariant("a ties by with no term"));
        };
        let word = agg.func.word();
        while let Some(group) = order.groups.pop() {
            let Some(expr) = ties.get(group.term).filter(|_| group.members.len() > 1) else {
                return Ok(Ok(Some(group.members)));
            };
            let read = if let Some(scored) = group.scored {
                Ok(scored)
            } else {
                self.tie_terms(m, (expr, word), envs, &group.members, ctx)?
            };
            let picked = match read {
                Err(hole) => return Ok(Err(hole)),
                Ok(scored) => pick_extreme(m, scored, order.way, word)?,
            };
            let (extreme, rest) = match picked {
                Err(hole) => return Ok(Err(hole)),
                Ok(picked) => picked,
            };
            if !rest.is_empty() {
                order.groups.push(Group {
                    members: rest.iter().map(|(at, _)| *at).collect(),
                    term: group.term,
                    scored: Some(rest),
                });
            }
            order.groups.push(Group {
                members: extreme,
                term: group
                    .term
                    .checked_add(1)
                    .ok_or_else(|| invariant("a ties by's next term"))?,
                scored: None,
            });
        }
        Ok(Ok(None))
    }

    /// A `ties by` term read for each candidate of a group: their values by position, or the
    /// hole of every term that is one; a term that is `none` is refused, a refusal outranking
    /// a hole. L §5.11, §8.1, §8.4.
    pub(super) fn tie_terms(
        &mut self,
        m: &Reality,
        (expr, word): (&Expr, &str),
        envs: &[&Env],
        group: &[usize],
        ctx: Ctx,
    ) -> Result<Result<Scored, Hole>, Stop> {
        let mut scored: Scored = Vec::with_capacity(group.len());
        let mut holes: Vec<Hole> = Vec::new();
        for &at in group {
            let env = envs
                .get(at)
                .ok_or_else(|| invariant("a tied candidate past the list"))?;
            let v = self.ev(m, expr, env, ctx)?;
            if v == Value::None {
                return Err(refuse(
                    Run::None,
                    format!("the ties by of {word} is none here"),
                ));
            }
            if let Value::Hole(h) = v {
                holes.push(h);
            } else {
                scored.push((at, v));
            }
        }
        if holes.is_empty() {
            return Ok(Ok(scored));
        }
        let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
        spread(&operands)
            .map(Err)
            .ok_or_else(|| invariant("holes that spread to none"))
    }

    /// `first(b where c)` and `last(…)`: the first (last) binding tuple whose `where` is
    /// true, in binding order with no `where` read past it, or over things, movements or an
    /// open index's keys in the order of the `ties by` among those the `where` keeps; a hole
    /// met before it makes the result that hole; none, `HOLE-NOTYET` where a binding of things
    /// or movements reaches past the cut, else `RUN-EMPTY` (refcore's `agg_first`). L §5.11,
    /// §3.1, §8.1, §8.4.
    pub(super) fn first_last(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg.func.word();
        let un = agg.binds.iter().find_map(|b| unordered(m, b));
        if let (Some(what), None) = (un, &agg.ties) {
            return Err(refuse(
                Run::Tie,
                format!("{word} over {} has no ties by", what.word()),
            ));
        }
        let way = if agg.func == AggFn::Last {
            Way::Back
        } else {
            Way::Forward
        };
        let met = if un.is_some() {
            self.first_by_ties(m, agg, env, way, ctx)?
        } else {
            self.first_in_order(m, agg, env, way, ctx)?
        };
        match met {
            Met::Key(key) => return Ok(key),
            Met::Holes(holes) => return holes_of(&holes),
            Met::Nothing => {}
        }
        if let Some((var, dom)) = self.notyet_bind(m, &agg.binds, env, ctx)? {
            let text = format!("{var}: no {dom} yet, and its span reaches past the cut");
            return Ok(Value::Hole(Hole::asked(&text, &dom)));
        }
        Err(refuse(Run::Empty, format!("{word} over no keys")))
    }

    /// `first` (`last`, from the end `way` names) over bindings in binding order: the tuples
    /// one at a time, each binding's `where` read as it is reached, up to the first tuple it
    /// keeps that is no unknown member. L §5.11, §8.2.
    fn first_in_order(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        way: Way,
        ctx: Ctx,
    ) -> Result<Met, Stop> {
        let mut walk = match self.tuple_walk(m, &agg.binds, env, ctx, way)? {
            Err(hole) => return Ok(Met::Holes(vec![hole])),
            Ok(walk) => walk,
        };
        let mut holes: Vec<Hole> = Vec::new();
        loop {
            let member = match self.next_tuple(m, &agg.binds, &mut walk, ctx, Brings::ToReader)? {
                Step::Member(member) => member,
                Step::Hole(hole) => {
                    holes.push(hole);
                    continue;
                }
                Step::Done => break,
            };
            let Some(hole) = member.unknown else {
                if holes.is_empty() {
                    return Ok(Met::Key(bare_key(agg, &member.env)?));
                }
                return Ok(Met::Holes(holes));
            };
            holes.push(hole);
        }
        Ok(if holes.is_empty() {
            Met::Nothing
        } else {
            Met::Holes(holes)
        })
    }

    /// `first` (`last`, from the end `way` names) over things, movements or an open index's
    /// keys: every value's `where` read first, since it guards the `ties by` ("a term that is
    /// `none` is refused unless the `where` tests it out"); the values it keeps in the order
    /// of the `ties by`, group by group, each group's register brought as it is reached, up to
    /// the first group with a true one; two true in one group are a tie it doesn't break
    /// (refcore's `agg_first`, `members`, `tie_groups`). L §5.11, §8.2.
    fn first_by_ties(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        way: Way,
        ctx: Ctx,
    ) -> Result<Met, Stop> {
        let [bind] = agg.binds.as_slice() else {
            return Err(Stop::NotBuilt("first or last over a tuple with no order"));
        };
        let cands: Vec<Candidate> = match self.bind_candidates(m, bind, env, ctx)? {
            Err(hole) => return Ok(Met::Holes(vec![hole])),
            Ok(cands) => cands,
        };
        let mut kept: Vec<(&Candidate, Option<Hole>)> = Vec::with_capacity(cands.len());
        for cand in &cands {
            if let Kept::In(hole) = self.candidate_kept(m, bind, cand, ctx)? {
                kept.push((cand, hole));
            }
        }
        let envs: Vec<&Env> = kept.iter().map(|(c, _)| &c.env).collect();
        let mut order = TieOrder::of(kept.len(), way);
        let mut holes: Vec<Hole> = Vec::new();
        loop {
            let group = match self.next_tied(m, agg, &envs, &mut order, ctx)? {
                Err(hole) => {
                    holes.push(hole);
                    return Ok(Met::Holes(holes));
                }
                Ok(None) => break,
                Ok(Some(group)) => group,
            };
            let mut trues: Vec<&Env> = Vec::new();
            for at in group {
                let (cand, hole) = kept
                    .get(at)
                    .ok_or_else(|| invariant("a candidate past the list"))?;
                if let Some(c) = &cand.brings {
                    self.absorb(c);
                }
                match hole {
                    Some(h) => holes.push(h.clone()),
                    None => trues.push(&cand.env),
                }
            }
            let Some(first) = trues.first() else {
                continue;
            };
            if !holes.is_empty() {
                return Ok(Met::Holes(holes));
            }
            if trues.len() > 1 {
                return Err(unbroken(m, agg, &trues)?);
            }
            return Ok(Met::Key(bare_key(agg, first)?));
        }
        Ok(if holes.is_empty() {
            Met::Nothing
        } else {
            Met::Holes(holes)
        })
    }

    /// The first binding of things or movements whose day is after the cut, or whose span
    /// reaches past it, its day or span read in the aggregate's scope; a binding whose day or
    /// span reads an earlier binding's variable, or is a hole, is passed over: its variable
    /// and what it ranges over (refcore's `notyet_bind`). L §3.1, §5.11.
    fn notyet_bind(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Option<(String, String)>, Stop> {
        let mut earlier: Vec<&str> = Vec::with_capacity(binds.len());
        for bind in binds {
            let reads_earlier = reads_any(bind, &earlier);
            earlier.push(&bind.var);
            let what = unordered(m, bind);
            if reads_earlier || !matches!(what, Some(Unordered::Things | Unordered::Movements)) {
                continue;
            }
            let Over::Name { id, .. } = &bind.domain else {
                continue;
            };
            if self.reaches_past(m, bind, env, ctx)? {
                return Ok(Some((bind.var.clone(), id.clone())));
            }
        }
        Ok(None)
    }

    /// Whether a binding's day is after the cut, or its span reaches past the day after it;
    /// a day or span that is a hole or `none` doesn't (refcore's `notyet_bind`, its tests).
    /// L §3.1.
    fn reaches_past(
        &mut self,
        m: &Reality,
        bind: &Bind,
        env: &Env,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        if let Some(on) = &bind.on {
            let day = self.ev(m, on, env, ctx)?;
            if matches!(day, Value::Hole(_) | Value::None) {
                return Ok(false);
            }
            return Ok(day_of_binding(&day)? > ctx.cut);
        }
        let Some(over) = &bind.over else {
            return Ok(false);
        };
        let Ok(span) = self.span_value(m, over, env, ctx)? else {
            return Ok(false);
        };
        let after = a_day(ctx.cut)?
            .next()
            .ok_or_else(|| invariant("the day after the cut"))?;
        Ok(span.end().is_none_or(|end| end > after))
    }
}

/// Whether a binding's day or span may read one of the variables `vars` (refcore's
/// `reads_var` over the earlier bindings). L §5.11.
fn reads_any(bind: &Bind, vars: &[&str]) -> bool {
    for var in vars {
        let on = bind.on.as_ref().is_some_and(|e| reads_name(e, var));
        let over = bind.over.as_ref().is_some_and(|s| span_reads(s, var));
        if on || over {
            return true;
        }
    }
    false
}

/// The positions of the terms that are the least (`wanted` Less) or the greatest, all those
/// equal to it; with a window among them, those decided below (above) every other term but
/// the ones that are one value with them, two reads of one item (B5), none where no term is
/// (refcore's `agg_arg`; equal values of two reads stay undecided there, pick P32). L §5.11,
/// §4.4.
fn extreme_keys(
    m: &Reality,
    terms: &[(&Env, Value)],
    wanted: Ordering,
) -> Result<Option<Vec<usize>>, Stop> {
    if terms.iter().any(|(_, t)| as_window(t).is_some()) {
        return decided_keys(m, terms, wanted);
    }
    let mut best: Vec<usize> = Vec::new();
    for (at, (_, t)) in terms.iter().enumerate() {
        let Some((_, held)) = best.first().and_then(|b| terms.get(*b)) else {
            best.push(at);
            continue;
        };
        let order = compare(m, t, held, Test::Order)?;
        if order == wanted {
            best = vec![at];
        } else if order == Ordering::Equal {
            best.push(at);
        }
    }
    Ok(Some(best))
}

/// The positions of the terms decided below (`wanted` Less) or above every other term but
/// those that are one value with them; none where no term is (refcore's `agg_arg`). L §5.11,
/// §4.4.
fn decided_keys(
    m: &Reality,
    terms: &[(&Env, Value)],
    wanted: Ordering,
) -> Result<Option<Vec<usize>>, Stop> {
    let mut wins: Vec<usize> = Vec::new();
    for (i, (_, t)) in terms.iter().enumerate() {
        let mut beats = true;
        for (j, (_, u)) in terms.iter().enumerate() {
            if i == j || one_read(t, u) {
                continue;
            }
            if decided_order(m, t, u)? != Some(wanted) {
                beats = false;
                break;
            }
        }
        if beats {
            wins.push(i);
        }
    }
    Ok((!wins.is_empty()).then_some(wins))
}
