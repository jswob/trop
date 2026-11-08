//! `prorata`: a total shared by one binding's weights to a step and read at one key, each rest made by the number kernel's split.
//! Spec: L §5.13 (`prorata(total, w[j] for j in IDX [where c], k, u, mode, rest none | last)`
//! and `… rest largest [ties by e]`: the total is money or a `num CUR`; a key the binding
//! leaves out has share 0, the split still made; the shares' type follows `round`, money when
//! u is a whole number of minor units, under `rest last` only when the total is money too;
//! `RUN-WEIGHTS`, `RUN-PRORATA`, `RUN-TIE`), §5.11 (`ties by (e1, e2)`: the least e first, e2
//! compared among those e1 left tied; a hole member), §7.2 (the step and the mode as `round`
//! reads them: a bare figure takes the total's unit), §8.1 (holes spread), §8.4 (`RUN-NONE`,
//! `RUN-DOMAIN`).
//! Twin: engine/tools/refcore/eval2.py (`m_e_prorata`).
//! Invariants: every share comes from `num::Weights`, the one split (R-114); a tie among the
//! remainders is broken only by the written `ties by`, never by key order, and a term of it is
//! read only for tied keys whose order still decides which of them take a step (D-5); a hole
//! among the total, the weights, the members, the key, the step or the mode, or in a `ties by`
//! term that decides, makes the share that hole (refcore returns the first hole of a `ties by`
//! where the engine spreads them, L §8.1).

use super::aggregate::{Brings, Member, Way};
use super::arith::{currency_of, same_value};
use super::bindings::bind_domain;
use super::builtin::rounding_mode;
use super::expr::{hole_of, literal};
use super::extremes::{Scored, pick_extreme};
use super::read::literal_key;
use super::state::{Ctx, Engine, Env, invariant, not_typed, num_stop, refuse, refuse_load};
use super::years::year_literal;
use crate::base::{Load, Run};
use crate::hole::Hole;
use crate::model::{Number, Reality, Unit, Value, currency_unit, money_num, num_money};
use crate::num::{Money, Num, SplitError, Tie, Weights};
use crate::syntax::{Lit, Prorata, Rest, Step, Stop};

/// A number or money as `prorata` reads it: its value in its unit, the unit, and whether it is
/// money (L §5.13: "The total is money or a `num CUR`").
#[derive(Debug, Clone, PartialEq, Eq)]
struct Amount {
    /// The value in its unit.
    q: Num,
    /// Its unit.
    unit: Unit,
    /// Whether it is money.
    money: bool,
}

impl Amount {
    /// The amount of a number or money (refcore's `Num.q` and `Num.u`). L §5.13, §7.1.
    fn of(m: &Reality, v: &Value) -> Result<Amount, Stop> {
        if let Value::Money(money) = v {
            return Ok(Amount {
                q: money_num(m, money).map_err(num_stop)?,
                unit: currency_unit(m, money.currency()).map_err(num_stop)?,
                money: true,
            });
        }
        let Value::Number(n) = v else {
            return Err(not_typed());
        };
        Ok(Amount {
            q: n.q.clone(),
            unit: n.unit(),
            money: false,
        })
    }

    /// The value in unit `to`, or none where the units don't convert. L §3.2.
    fn in_unit(&self, to: &Unit) -> Result<Option<Num>, Stop> {
        if self.unit == *to {
            return Ok(Some(self.q.clone()));
        }
        let Some(factor) = self.unit.conv(to).map_err(num_stop)? else {
            return Ok(None);
        };
        Ok(Some(self.q.checked_mul(&factor).map_err(num_stop)?))
    }
}

/// `RUN-NONE` for a `none` that `prorata` reads (refcore's `no_none`). L §8.4.
fn no_none(v: &Value) -> Result<(), Stop> {
    if *v == Value::None {
        return Err(refuse(
            Run::None,
            "a value is none here: prorata".to_owned(),
        ));
    }
    Ok(())
}

/// The binding's members with their weights, in binding order: each member, its key and its
/// weight (L §5.11, §5.13 step 1).
struct Shared {
    /// Each member, its scope with its variable bound.
    members: Vec<Member>,
    /// Each member's key.
    keys: Vec<Value>,
    /// Each member's weight.
    weights: Vec<Value>,
}

/// What a rest splits: the checked weights, the total, the step, and the position of key k
/// among the binding's keys, none for a key the binding leaves out (L §5.13).
struct ToShare {
    /// The weights, checked.
    weights: Weights,
    /// The total.
    total: Amount,
    /// The step.
    step: StepIn,
    /// Key k's position among the binding's keys.
    at: Option<usize>,
}

/// The keys at positions `at` as `RUN-TIE` lists them, `q1, q2`. L §8.4.
fn keys_text(m: &Reality, keys: &[Value], at: &[usize]) -> Result<String, Stop> {
    let mut out = Vec::with_capacity(at.len());
    for j in at {
        let key = keys
            .get(*j)
            .ok_or_else(|| invariant("a tied key past the list"))?;
        out.push(key.text(m).map_err(num_stop)?);
    }
    Ok(out.join(", "))
}

/// A group of tied keys, by position among the tied, still to place: the `ties by` term that
/// orders it next and that term's values once read (L §5.11).
struct Pending {
    /// The keys, by position among the tied.
    members: Vec<usize>,
    /// The term that orders them next.
    term: usize,
    /// That term's values, once read.
    scored: Option<Scored>,
}

impl Engine {
    /// `prorata(…)`: key k's share of the total split by the binding's weights to the step,
    /// its rest none, last or largest (refcore's `m_e_prorata`). L §5.13.
    ///
    /// # Errors
    ///
    /// `RUN-WEIGHTS`, `RUN-PRORATA`, `RUN-TIE`, `RUN-NONE`, `RUN-DOMAIN`, the `LOAD-UNIT`
    /// refcore raises at run, and the refusals the expressions it reads meet.
    pub(super) fn prorata(
        &mut self,
        m: &Reality,
        p: &Prorata,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let total = self.ev(m, &p.total, env, ctx)?;
        if let Value::Hole(_) = total {
            return Ok(total);
        }
        no_none(&total)?;
        let shared = match self.shared(m, p, env, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(shared) => shared,
        };
        let k = self.ev(m, &p.key, env, ctx)?;
        if let Value::Hole(_) = k {
            return Ok(k);
        }
        let k = written_key(m, p, k)?;
        let mut at = None;
        for (j, key) in shared.keys.iter().enumerate() {
            if same_value(m, key, &k)? {
                at = Some(j);
                break;
            }
        }
        let weights = weights_of(m, &shared.weights)?;
        let total = Amount::of(m, &total)?;
        let step = match self.step(m, p, &total, env, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(step) => step,
        };
        let split = ToShare {
            weights,
            total,
            step,
            at,
        };
        let made = match p.rest {
            Rest::None | Rest::Last => self.rest_rounded(m, p, &split, env, ctx)?,
            Rest::Largest => self.rest_largest(m, p, &split, &shared, ctx)?,
        };
        match made {
            Err(hole) => Ok(Value::Hole(hole)),
            Ok(q) => share_value(m, p, &split.total, &split.step, q),
        }
    }

    /// The binding's members and their weights, or the hole of the domain, of the members
    /// whose `where` is one and of the weights that are (refcore's `m_e_prorata`, its
    /// `holes`). L §5.11, §5.13 step 1, §8.1.
    fn shared(
        &mut self,
        m: &Reality,
        p: &Prorata,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Shared, Hole>, Stop> {
        let members = match self.members(m, &p.binds, env, ctx, Brings::ToReader)? {
            Err(hole) => return Ok(Err(hole)),
            Ok(members) => members,
        };
        let mut keys = Vec::with_capacity(members.len());
        let mut weights = Vec::with_capacity(members.len());
        let mut holes: Vec<Hole> = Vec::new();
        for member in &members {
            if let Some(h) = &member.unknown {
                holes.push(h.clone());
            }
            let w = self.ev(m, &p.weight, &member.env, ctx)?;
            if let Value::Hole(h) = w {
                holes.push(h);
                continue;
            }
            no_none(&w)?;
            keys.push(bound_key(p, &member.env)?);
            weights.push(w);
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            let Value::Hole(hole) = hole_of(&operands)? else {
                return Err(invariant("holes that spread to no hole"));
            };
            return Ok(Err(hole));
        }
        Ok(Ok(Shared {
            members,
            keys,
            weights,
        }))
    }

    /// `rest none` and `rest last`: key k's share rounded by the mode, the last key's the
    /// remainder under `rest last`; 0 for a key the binding leaves out; or the hole of a mode
    /// that is one. L §5.13 step 2.
    fn rest_rounded(
        &mut self,
        m: &Reality,
        p: &Prorata,
        split: &ToShare,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let mode_expr = p
            .mode
            .as_ref()
            .ok_or_else(|| invariant("prorata with no mode"))?;
        let mode = self.ev(m, mode_expr, env, ctx)?;
        if let Value::Hole(hole) = mode {
            return Ok(Err(hole));
        }
        let mode = rounding_mode(m, "prorata", &mode)?;
        let Some(j) = split.at else {
            return Ok(Ok(Num::zero()));
        };
        let (total, step) = (&split.total.q, &split.step.q);
        let made = if p.rest == Rest::None {
            split.weights.rounded(total, j, step, mode)
        } else {
            split.weights.last_rest(total, j, step, mode)
        };
        Ok(Ok(made.map_err(split_stop)?))
    }

    /// `rest largest`: the split, its tie broken by the `ties by`, and key k's share; 0 for a
    /// key the binding leaves out, the split still made and its refusals standing; or the
    /// hole of a `ties by` term that decides. L §5.13 step 1, step 2.
    ///
    /// # Errors
    ///
    /// `RUN-PRORATA` for a total that isn't a whole number of the step; `RUN-TIE`.
    fn rest_largest(
        &mut self,
        m: &Reality,
        p: &Prorata,
        split: &ToShare,
        shared: &Shared,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let made = match split.weights.largest(&split.total.q, &split.step.q) {
            Err(SplitError::NotWhole) => {
                let message = format!(
                    "{} isn't a whole number of {}",
                    total_text(m, &split.total)?,
                    split.step.shown.text(m).map_err(num_stop)?
                );
                return Err(refuse(Run::Prorata, message));
            }
            made => made.map_err(split_stop)?,
        };
        let chosen = match made.tie() {
            None => Vec::new(),
            Some(tie) => match self.break_tie(m, p, shared, tie, ctx)? {
                Err(hole) => return Ok(Err(hole)),
                Ok(chosen) => chosen,
            },
        };
        let shares = made.shares(&chosen).map_err(split_stop)?;
        let Some(j) = split.at else {
            return Ok(Ok(Num::zero()));
        };
        let share = shares
            .into_iter()
            .nth(j)
            .ok_or_else(|| invariant("a share past the keys"))?;
        Ok(Ok(share))
    }

    /// The step as `round` reads it (refcore's `_stepflex`): `minor(CUR)`, a figure, or a
    /// read; a bare figure takes the total's unit; in the total's unit, above 0; or the hole
    /// of a read that is one. L §5.13, §7.2.
    fn step(
        &mut self,
        m: &Reality,
        p: &Prorata,
        total: &Amount,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<StepIn, Hole>, Stop> {
        let (written, bare) = match &p.unit {
            Step::Number(n) => (
                literal(m, &Lit::Num(n.clone()), env.figures)?,
                n.unit.is_none() && !n.fraction,
            ),
            Step::Minor(cur) => (
                Value::Money(Money::from_minor(1, currency_of(m, cur)?)),
                false,
            ),
            Step::Ref(e) => (self.ev(m, e, env, ctx)?, false),
        };
        if let Value::Hole(h) = written {
            return Ok(Err(h));
        }
        no_none(&written)?;
        let mut u = Amount::of(m, &written)?;
        let shown = if bare {
            u.unit = total.unit.clone();
            Value::Number(Number::plain(u.q.clone(), total.unit.clone()))
        } else {
            written
        };
        let Some(q) = u.in_unit(&total.unit)? else {
            return Err(refuse_load(
                Load::Unit,
                format!(
                    "prorata: the step {} and the total {} have different units",
                    shown.text(m).map_err(num_stop)?,
                    total_text(m, total)?
                ),
            ));
        };
        if q.is_zero() || q.is_negative() {
            return Err(refuse(
                Run::Domain,
                format!(
                    "prorata with a step {} that is not positive",
                    shown.text(m).map_err(num_stop)?
                ),
            ));
        }
        Ok(Ok(StepIn {
            q,
            whole: u.money || (bare && whole_minor(m, &u.q, &total.unit)?),
            shown,
        }))
    }

    /// The tied keys, by position among the binding's keys, that take the split's last steps:
    /// the `take` of them with the least `ties by`, e1 read for every tied key and each later
    /// term only among those the earlier left tied where that tie still decides which take a
    /// step (refcore's `m_e_prorata`, its `cmp` over `n.tie`). L §5.13 step 2, §5.11, D-5.
    ///
    /// # Errors
    ///
    /// `RUN-TIE` with no `ties by`, or one that leaves the tie that decides unbroken.
    fn break_tie(
        &mut self,
        m: &Reality,
        p: &Prorata,
        shared: &Shared,
        tie: &Tie,
        ctx: Ctx,
    ) -> Result<Result<Vec<usize>, Hole>, Stop> {
        let Some(ties) = &p.ties else {
            return Err(refuse(
                Run::Tie,
                format!(
                    "prorata has a tie at {} that changes the result, and no ties by",
                    keys_text(m, &shared.keys, &tie.tied)?
                ),
            ));
        };
        let mut envs: Vec<&Env> = Vec::with_capacity(tie.tied.len());
        for j in &tie.tied {
            let member = shared
                .members
                .get(*j)
                .ok_or_else(|| invariant("a tied key past the members"))?;
            envs.push(&member.env);
        }
        let mut pending = vec![Pending {
            members: (0..envs.len()).collect(),
            term: 0,
            scored: None,
        }];
        let mut chosen: Vec<usize> = Vec::with_capacity(tie.take);
        let mut need = tie.take;
        while let Some(group) = pending.pop() {
            if need == 0 {
                break;
            }
            if let Some(left) = need.checked_sub(group.members.len()) {
                need = left;
                chosen.extend(group.members);
                continue;
            }
            let Some(expr) = ties.get(group.term) else {
                return Err(refuse(
                    Run::Tie,
                    format!(
                        "prorata has a tie at {} that changes the result, and its ties by doesn't break it",
                        keys_text(m, &shared.keys, &tie.tied)?
                    ),
                ));
            };
            let scored = match group.scored {
                Some(scored) => scored,
                None => match self.tie_terms(m, (expr, "prorata"), &envs, &group.members, ctx)? {
                    Err(hole) => return Ok(Err(hole)),
                    Ok(scored) => scored,
                },
            };
            let (least, rest) = match pick_extreme(m, scored, Way::Forward, "prorata")? {
                Err(hole) => return Ok(Err(hole)),
                Ok(picked) => picked,
            };
            if !rest.is_empty() {
                pending.push(Pending {
                    members: rest.iter().map(|(at, _)| *at).collect(),
                    term: group.term,
                    scored: Some(rest),
                });
            }
            pending.push(Pending {
                members: least,
                term: group
                    .term
                    .checked_add(1)
                    .ok_or_else(|| invariant("a ties by's next term"))?,
                scored: None,
            });
        }
        let mut out = Vec::with_capacity(chosen.len());
        for at in chosen {
            out.push(
                *tie.tied
                    .get(at)
                    .ok_or_else(|| invariant("a chosen key past the tie"))?,
            );
        }
        Ok(Ok(out))
    }
}

/// The step in the total's unit, whether it is a whole number of minor units, and the step as
/// refusals print it (L §5.13, §7.2).
struct StepIn {
    /// Its value in the total's unit.
    q: Num,
    /// Whether it is money, or a bare figure that is a whole number of the total's minor units.
    whole: bool,
    /// As it prints.
    shown: Value,
}

/// The key a member binds (refcore's `e2[b.var]`). L §5.13.
fn bound_key(p: &Prorata, env: &Env) -> Result<Value, Stop> {
    let [bind] = p.binds.as_slice() else {
        return Err(Stop::NotBuilt("prorata over a tuple of bindings"));
    };
    env.get(&bind.var)
        .map(|v| v.value.clone())
        .ok_or_else(|| invariant("a bound variable the scope lacks"))
}

/// The key k as the binding's keys are: a key literal (`q1`), or a key in a year form
/// (`2027` over `years`), resolved in the binding's domain as load resolves it (refcore's
/// load, `assign` of "the key of prorata"). L §2.1, §2.5, §5.13.
fn written_key(m: &Reality, p: &Prorata, k: Value) -> Result<Value, Stop> {
    let lit = if let Value::Word(text) = &k {
        Lit::Name(text.clone())
    } else if let Some(lit) = year_literal(&p.key, &k)? {
        lit
    } else {
        return Ok(k);
    };
    let [bind] = p.binds.as_slice() else {
        return Err(Stop::NotBuilt("prorata over a tuple of bindings"));
    };
    let Some(domain) = bind_domain(m, bind) else {
        return Ok(k);
    };
    let found = literal_key(m, &lit, &domain)?;
    Ok(found.as_ref().map_or(k, Value::of_key))
}

/// The weights in the first's unit, checked: `LOAD-UNIT` for weights of units that don't
/// convert, `RUN-WEIGHTS` for one below 0 or a sum of 0 (refcore's `m_e_prorata`). L §5.13
/// step 1.
fn weights_of(m: &Reality, weights: &[Value]) -> Result<Weights, Stop> {
    let mut amounts = Vec::with_capacity(weights.len());
    for w in weights {
        amounts.push(Amount::of(m, w)?);
    }
    let unit = amounts.first().map_or_else(Unit::one, |a| a.unit.clone());
    let mut each = Vec::with_capacity(amounts.len());
    for a in &amounts {
        let Some(q) = a.in_unit(&unit)? else {
            return Err(refuse_load(
                Load::Unit,
                "prorata: weights of different units".to_owned(),
            ));
        };
        each.push(q);
    }
    // refcore's `fmt_q`: each weight a plain number
    let mut texts = Vec::with_capacity(each.len());
    for q in &each {
        let shown = Value::Number(Number::plain(q.clone(), Unit::one()));
        texts.push(shown.text(m).map_err(num_stop)?);
    }
    match Weights::new(each) {
        Err(SplitError::Weights) => Err(refuse(
            Run::Weights,
            format!(
                "prorata weights [{}] are negative or sum to 0",
                texts.join(", ")
            ),
        )),
        made => made.map_err(split_stop),
    }
}

/// The total as refusals print it (refcore's `fmt` of the total). L §2.4.
fn total_text(m: &Reality, total: &Amount) -> Result<String, Stop> {
    let v = if total.money {
        Value::Money(num_money(m, &total.q, currency_id_of(m, &total.unit)?).map_err(num_stop)?)
    } else {
        Value::Number(Number::plain(total.q.clone(), total.unit.clone()))
    };
    v.text(m).map_err(num_stop)
}

/// The currency of a currency unit. L §3.2.
fn currency_id_of(m: &Reality, unit: &Unit) -> Result<crate::num::Currency, Stop> {
    let [(cur, 1)] = unit.terms() else {
        return Err(invariant("a total whose unit is no currency"));
    };
    currency_of(m, cur)
}

/// Whether `q`, in currency unit `unit`, is a whole number of its minor units (L §7.2 Its
/// type).
fn whole_minor(m: &Reality, q: &Num, unit: &Unit) -> Result<bool, Stop> {
    let [(cur, 1)] = unit.terms() else {
        return Ok(false);
    };
    let minor = m
        .currencies
        .get(cur)
        .ok_or_else(|| invariant("a currency load didn't declare"))?;
    Ok(q.checked_div(minor).map_err(num_stop)?.is_integer())
}

/// A split's error as its refusal. L §5.13, §8.4.
fn split_stop(error: SplitError) -> Stop {
    match error {
        SplitError::Weights => invariant("weights checked twice"),
        SplitError::NotWhole => invariant("a total off the step outside rest largest"),
        SplitError::Num(e) => num_stop(e),
    }
}

/// The share `q` in its type: money when the step is a whole number of minor units, under
/// `rest last` only when the total is money too; else a `num` of the total's unit (refcore's
/// `n._money`, check2.py's `t_prorata`). L §5.13.
fn share_value(
    m: &Reality,
    p: &Prorata,
    total: &Amount,
    step: &StepIn,
    q: Num,
) -> Result<Value, Stop> {
    if step.whole && (p.rest != Rest::Last || total.money) {
        let cur = currency_id_of(m, &total.unit)?;
        return match num_money(m, &q, cur) {
            Ok(money) => Ok(Value::Money(money)),
            Err(_) => Err(refuse_load(
                Load::Unit,
                format!(
                    "prorata to money: the step {} is not a whole number of minor units",
                    step.shown.text(m).map_err(num_stop)?
                ),
            )),
        };
    }
    Ok(Value::Number(Number::plain(q, total.unit.clone())))
}
