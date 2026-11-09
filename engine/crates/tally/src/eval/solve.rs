//! `solve` in an ask: the grid point nearest the root of f − g by exact bisection, each step's pin a layer freed after it.
//! Spec: L §5.18 (`solve f == g for v in [lo, hi] to u`: v an input or a def, pinned at each
//! step, the pin replacing v's own row and listed nowhere; the grid lo + i·u holding lo and
//! hi; the bracket's literals and u in v's type, a rate solved over rates; a hole in F at any
//! point evaluated makes the result that hole; steps 1–3; `RUN-SOLVE` "no sign change on
//! [lo, hi]"; its cost ⌈log₂((hi − lo)/u)⌉ + 3 evaluations of F's cone), §8.4; P §3.2, §3.5
//! (a `solve` step's pin layer, freed when the step ends).
//! Twin: engine/tools/refcore/eval2.py (`m_solve_value`, its `F`); check2.py
//! (`m_check_solve`, whose refusals load makes first).
//! Invariants: every grid point is exact; the interval [low, high] holds a sign change and
//! halves on the grid at each step, so the search ends; each step's layer is freed before the
//! next; a solve in a rule stops `NotBuilt`.

use core::cmp::Ordering;

use super::arith::{Op, arith, currency_of};
use super::expr::literal;
use super::read::make_keys;
use super::stack::Item;
use super::state::{Collected, Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use crate::base::{Code, Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{
    Class, KeyVal, NameKind, Number, Reality, Shown, Ty, Unit, Value, currency_unit, money_num,
    num_money,
};
use crate::num::{Mode, Money, Num};
use crate::syntax::{Expr, Key, Lit, Postfix, Solve, Step, Stop};

/// The type of a solve's grid points, with the unit its numbers are in (L §5.18: "The
/// bracket's literals and u take v's type").
#[derive(Debug, Clone, PartialEq, Eq)]
enum Grid {
    /// Money of this currency, its numbers in its units.
    Money(String),
    /// A `num` of this unit, shown as its type shows it.
    Num(Unit, Shown),
    /// An `int`.
    Int,
    /// A rate, its numbers per the step's period, this unit (`1/year`).
    Rate(Unit),
}

impl Grid {
    /// The grid of v's type `ty`, a rate's period the step's; none for a type load refuses.
    fn of(ty: &Ty, step: &Value) -> Option<Grid> {
        if let Ty::Money(cur) = ty {
            return Some(Grid::Money(cur.clone()));
        }
        if let Ty::Num(unit, shown) = ty {
            return Some(Grid::Num(unit.clone(), *shown));
        }
        if *ty == Ty::Int {
            return Some(Grid::Int);
        }
        if *ty == Ty::Rate
            && let Value::Number(n) = step
        {
            return Some(Grid::Rate(n.unit()));
        }
        None
    }

    /// The value at grid number `q` (refcore's `mk`).
    fn value(&self, m: &Reality, q: &Num) -> Result<Value, Stop> {
        match self {
            Grid::Money(cur) => {
                let id = currency_of(m, cur)?;
                Ok(Value::Money(num_money(m, q, id).map_err(num_stop)?))
            }
            Grid::Num(unit, shown) => Ok(Value::Number(Number {
                q: q.clone(),
                class: Class::Num(unit.clone(), *shown),
            })),
            Grid::Int => Ok(Value::Number(Number {
                q: q.clone(),
                class: Class::Int,
            })),
            Grid::Rate(per) => Ok(Value::Number(Number {
                q: q.clone(),
                class: Class::Rate(per.clone()),
            })),
        }
    }

    /// The unit the grid's numbers are in.
    fn unit(&self, m: &Reality) -> Result<Unit, Stop> {
        match self {
            Grid::Money(cur) => currency_unit(m, currency_of(m, cur)?).map_err(num_stop),
            Grid::Num(unit, _) | Grid::Rate(unit) => Ok(unit.clone()),
            Grid::Int => Ok(Unit::one()),
        }
    }
}

/// What is solved for: its item, its keys, and v's type (L §5.18).
struct Aim {
    /// The input or def.
    item: String,
    /// Its keys.
    keys: Vec<KeyVal>,
    /// Its type.
    ty: Ty,
}

/// What is solved for, with its grid.
struct Target {
    /// The item and keys.
    aim: Aim,
    /// Its grid points' type.
    grid: Grid,
}

/// The sign of F at a grid point, or the hole F is there.
enum Sign {
    /// F's sign, as F compares with 0.
    Of(Ordering),
    /// F is a hole there.
    Hole(Hole),
}

impl Engine {
    /// An ask's `solve` line in scope `env`: its value, or the run-time refusal it met, with
    /// the register entries every step's reads met (refcore's `evaluate`, `one_solve`).
    /// L §5.18, §8.2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run, which refuses the run; `NotBuilt` for a form not built yet.
    pub fn solve_line(
        &mut self,
        m: &Reality,
        form: &Solve,
        env: &Env,
        ctx: Ctx,
    ) -> Result<(Item, Collected), Stop> {
        self.push();
        let item = match self.solve(m, form, env, ctx) {
            Ok(v) => Ok(Item::Value(v)),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => Ok(Item::Refused(r)),
            Err(stop) => Err(stop),
        };
        let met = self.pop_alone();
        Ok((item?, met))
    }

    /// `solve f == g for v in [lo, hi] to u` (refcore's `m_solve_value`): F(lo) or F(hi)
    /// that is 0 is the answer; else F(lo) and F(hi) of one sign is `RUN-SOLVE`; else
    /// bisection on the grid lo + i·u, the result the grid point nearest the root, a tie going
    /// up. A key of v that is a hole, or a hole F at a point evaluated, is the result.
    /// L §5.18 steps 1–3.
    ///
    /// # Errors
    ///
    /// `RUN-SOLVE`; `LOAD-DOMAIN` for a bracket off its grid; `LOAD-UNIT` for a rate's
    /// bracket and step of different periods; `RUN-NONE` for a `none` side; what f and g meet.
    pub fn solve(&mut self, m: &Reality, form: &Solve, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let aim = match self.solve_target(m, &form.target, env, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(aim) => aim,
        };
        let (step, grid) = match self.solve_grid(m, form, &aim.ty, env, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(made) => made,
        };
        let target = Target { aim, grid };
        let lo = literal(m, &form.lo, env.figures)?;
        let hi = literal(m, &form.hi, env.figures)?;
        let lq = grid_number(m, (&lo, bare(&form.lo)), &target.grid)?;
        let hq = grid_number(m, (&hi, bare(&form.hi)), &target.grid)?;
        let span = hq.checked_sub(&lq).map_err(num_stop)?;
        if span.is_negative() || !span.checked_div(&step).map_err(num_stop)?.is_integer() {
            return Err(refuse_load(
                Load::Domain,
                "solve: [lo, hi] is not on the grid lo + i u".to_owned(),
            ));
        }
        // step 1: an end at the root is the answer; else the ends' signs must differ
        let at_lo = match self.sign_at(m, form, &target, &lq, (env, ctx))? {
            Sign::Hole(hole) => return Ok(Value::Hole(hole)),
            Sign::Of(Ordering::Equal) => return target.grid.value(m, &lq),
            Sign::Of(sign) => sign,
        };
        let at_hi = match self.sign_at(m, form, &target, &hq, (env, ctx))? {
            Sign::Hole(hole) => return Ok(Value::Hole(hole)),
            Sign::Of(Ordering::Equal) => return target.grid.value(m, &hq),
            Sign::Of(sign) => sign,
        };
        if at_lo == at_hi {
            return Err(refuse(
                Run::Solve,
                format!(
                    "no sign change on [{}, {}]",
                    lo.text(m).map_err(num_stop)?,
                    hi.text(m).map_err(num_stop)?
                ),
            ));
        }
        // step 2: halve [low, high] on the grid while it holds more than one step; F(low)
        // keeps F(lo)'s sign
        let (mut low, mut high) = (lq, hq);
        while high
            .checked_sub(&low)
            .and_then(|width| width.compare(&step))
            .map_err(num_stop)?
            == Ordering::Greater
        {
            let half = high
                .checked_sub(&low)
                .and_then(|width| width.checked_div(&Num::int(2)))
                .and_then(|width| width.round(&step, Mode::Floor))
                .map_err(num_stop)?;
            let mid = low.checked_add(&half).map_err(num_stop)?;
            match self.sign_at(m, form, &target, &mid, (env, ctx))? {
                Sign::Hole(hole) => return Ok(Value::Hole(hole)),
                Sign::Of(Ordering::Equal) => return target.grid.value(m, &mid),
                Sign::Of(sign) if sign == at_lo => low = mid,
                Sign::Of(_) => high = mid,
            }
        }
        // step 3: the grid point nearer the root, a tie going up
        let centre = step
            .checked_div(&Num::int(2))
            .and_then(|width| low.checked_add(&width))
            .map_err(num_stop)?;
        match self.sign_at(m, form, &target, &centre, (env, ctx))? {
            Sign::Hole(hole) => Ok(Value::Hole(hole)),
            Sign::Of(sign) if sign == Ordering::Equal || sign == at_lo => {
                target.grid.value(m, &high)
            }
            Sign::Of(_) => target.grid.value(m, &low),
        }
    }

    /// The item and keys solved for, its keys evaluated as a read's, and its type; or the hole
    /// of a key that is one (refcore's `m_solve_value`, its keys). L §5.18.
    fn solve_target(
        &mut self,
        m: &Reality,
        target: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Aim, Hole>, Stop> {
        let (item, written): (&str, &[Key]) = if let Expr::Name(id) = target {
            (id, &[])
        } else if let Expr::Post(base, posts) = target
            && let (Expr::Name(id), [Postfix::Keys(keys)]) = (base.as_ref(), posts.as_slice())
        {
            (id, keys)
        } else {
            return Err(invariant("a solve's target load didn't check"));
        };
        let (binds, ty) = if let Some(input) = m.inputs.get(item)
            && m.names.kind(item) == Some(NameKind::Input)
        {
            let ty = input
                .ty
                .clone()
                .ok_or_else(|| invariant("a solved input's type"))?;
            (&input.binds, ty)
        } else if let Some(def) = m.defs.get(item) {
            (&def.binds, def.ty.clone())
        } else {
            return Err(invariant("a solve's target load didn't check"));
        };
        let mut given = Vec::with_capacity(written.len());
        for key in written {
            let Key::Expr(e) = key else {
                return Err(invariant("a `*` in a solve's target, which load refuses"));
            };
            given.push(self.key_given(m, e, env, ctx)?);
        }
        Ok(match make_keys(m, item, binds, given, ctx)? {
            Err(hole) => Err(hole),
            Ok(keys) => Ok(Aim {
                item: item.to_owned(),
                keys,
                ty,
            }),
        })
    }

    /// The step u as a number of the grid's unit, and the grid: v's type, a rate's period the
    /// step's (refcore's `m_solve_value`, its step); the hole of a read that is one. A bare
    /// step takes v's unit. L §5.18, §7.2.
    fn solve_grid(
        &mut self,
        m: &Reality,
        form: &Solve,
        ty: &Ty,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<(Num, Grid), Hole>, Stop> {
        let (written, is_bare) = match &form.unit {
            Step::Number(n) => (
                literal(m, &Lit::Num(n.clone()), env.figures)?,
                n.unit.is_none(),
            ),
            Step::Minor(cur) => (
                Value::Money(Money::from_minor(1, currency_of(m, cur)?)),
                false,
            ),
            Step::Ref(e) => {
                let read = self.value(m, e, env, ctx)?;
                if let Value::Hole(hole) = read {
                    return Ok(Err(hole));
                }
                let (q, unit) = step_number(m, &read)?;
                let grid = Grid::of(ty, &read)
                    .ok_or_else(|| invariant("a solve's type or step load didn't check"))?;
                return Ok(Ok((step_in_grid(m, (&read, q, unit), &grid)?, grid)));
            }
        };
        let grid = Grid::of(ty, &written)
            .ok_or_else(|| invariant("a solve's type or step load didn't check"))?;
        let step = grid_number(m, (&written, is_bare), &grid)?;
        Ok(Ok((step, grid)))
    }

    /// F's sign at grid number `point`: f and g read in a layer pinning v there, freed after
    /// (refcore's `F`); a hole side is the hole of both sides' questions. L §5.18, P §3.5.
    fn sign_at(
        &mut self,
        m: &Reality,
        form: &Solve,
        target: &Target,
        point: &Num,
        (env, ctx): (&Env, Ctx),
    ) -> Result<Sign, Stop> {
        let value = target.grid.value(m, point)?;
        let step = self.solve_step((&target.aim.item, &target.aim.keys), value, ctx)?;
        self.open_layer(m, step)?;
        let sides = self.sides(m, form, env, step);
        self.end_layer(step.layer, sides.as_ref().err())?;
        let (left, right) = sides?;
        if left.as_hole().is_some() || right.as_hole().is_some() {
            let hole = spread(&[left.as_hole(), right.as_hole()])
                .ok_or_else(|| invariant("a hole side with no hole"))?;
            return Ok(Sign::Hole(hole));
        }
        if left == Value::None || right == Value::None {
            return Err(refuse(Run::None, "a value is none here: solve".to_owned()));
        }
        let gap = arith(m, Op::Sub, &left, &right)?;
        let q = if let Value::Money(money) = &gap {
            money_num(m, money).map_err(num_stop)?
        } else if let Value::Number(n) = gap {
            n.q
        } else {
            return Err(invariant(
                "F of a solve that isn't a number, which load refuses",
            ));
        };
        Ok(Sign::Of(q.compare(&Num::zero()).map_err(num_stop)?))
    }

    /// f and g read in the step's context, each as an ask's line reads it.
    fn sides(
        &mut self,
        m: &Reality,
        form: &Solve,
        env: &Env,
        ctx: Ctx,
    ) -> Result<(Value, Value), Stop> {
        let left = self.value(m, &form.left, env, ctx)?;
        let right = self.value(m, &form.right, env, ctx)?;
        Ok((left, right))
    }
}

/// A step `to u` read at run, its number and unit (refcore's `m_step_value`): a value that
/// isn't a number or money is `LOAD-UNIT the step of to is <v>`; one that isn't positive is
/// `RUN-DOMAIN to <v>: a step that is not positive` (L §7.1 precision steps, §8.4).
///
/// # Errors
///
/// `LOAD-UNIT`, `RUN-DOMAIN`, as above.
fn step_number(m: &Reality, v: &Value) -> Result<(Num, Unit), Stop> {
    let (q, unit) = if let Value::Money(money) = v {
        (
            money_num(m, money).map_err(num_stop)?,
            currency_unit(m, money.currency()).map_err(num_stop)?,
        )
    } else if let Value::Number(n) = v {
        (n.q.clone(), n.unit())
    } else {
        let text = v.text(m).map_err(num_stop)?;
        return Err(refuse_load(Load::Unit, format!("the step of to is {text}")));
    };
    if q.compare(&Num::zero()).map_err(num_stop)? != Ordering::Greater {
        let text = v.text(m).map_err(num_stop)?;
        return Err(refuse(
            Run::Domain,
            format!("to {text}: a step that is not positive"),
        ));
    }
    Ok((q, unit))
}

/// A step read at run as a number of the grid's unit (refcore's `m_step_value`): a number
/// with no unit as it is, a rate as its grid takes it, else converted to v's unit, or
/// `LOAD-UNIT to <v>: the step and the value (<unit>) have different units`. L §7.1.
///
/// # Errors
///
/// `LOAD-UNIT`, as above.
fn step_in_grid(m: &Reality, (v, q, unit): (&Value, Num, Unit), grid: &Grid) -> Result<Num, Stop> {
    if let Grid::Rate(_) = grid {
        return grid_number(m, (v, false), grid);
    }
    let to = grid.unit(m)?;
    if unit == Unit::one() || unit == to {
        return Ok(q);
    }
    let Some(factor) = unit.conv(&to).map_err(num_stop)? else {
        let text = v.text(m).map_err(num_stop)?;
        return Err(refuse_load(
            Load::Unit,
            format!(
                "to {text}: the step and the value ({}) have different units",
                to.text()
            ),
        ));
    };
    q.checked_mul(&factor).map_err(num_stop)
}

/// Whether a bracket's literal is a bare figure, which takes v's type (L §5.18, §7.1).
fn bare(lit: &Lit) -> bool {
    matches!(lit, Lit::Num(n) if n.unit.is_none())
}

/// A bracket end or a step as a number of the grid's unit: a bare figure as it is, money in
/// its currency's units, a number converted to the grid's unit; a rate's period must be the
/// grid's (L §5.18, §3.2).
///
/// # Errors
///
/// `LOAD-UNIT` for a rate of another period, or a unit that doesn't convert.
fn grid_number(m: &Reality, (v, is_bare): (&Value, bool), grid: &Grid) -> Result<Num, Stop> {
    let (q, unit) = if let Value::Money(money) = v {
        (
            money_num(m, money).map_err(num_stop)?,
            currency_unit(m, money.currency()).map_err(num_stop)?,
        )
    } else if let Value::Number(n) = v {
        (n.q.clone(), n.unit())
    } else {
        return Err(invariant("a solve's bracket load didn't type"));
    };
    if is_bare {
        return Ok(q);
    }
    let to = grid.unit(m)?;
    if unit == to {
        return Ok(q);
    }
    if let Grid::Rate(_) = grid {
        return Err(refuse_load(
            Load::Unit,
            "solve: the bracket and the step are rates of different periods".to_owned(),
        ));
    }
    let factor = unit.conv(&to).map_err(num_stop)?.ok_or_else(|| {
        refuse_load(
            Load::Unit,
            "solve: the bracket and the step are of different units".to_owned(),
        )
    })?;
    q.checked_mul(&factor).map_err(num_stop)
}
