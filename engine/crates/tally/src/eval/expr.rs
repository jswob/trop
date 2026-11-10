//! The evaluator's walk over an expression: one arm per node kind, each calling the step that computes it.
//! Spec: E §3.2 (a tree-walk over the resolved tree); L §2.5 (names: a bound variable, an item,
//! a key literal, a movement instance by its move and keys), §3.1 (the absences), §3.2 (inline figures), §5.11 (logic under Kleene
//! rules, `if` with a hole condition), §5.14 (a movement's fields and kind, `in` over a
//! movement kind, movements equal by identity), §5.19 (`covered`), §4.1 (a keyed place
//! `name[keys]`), §8.1 (holes spread), §8.4 (`RUN-NONE`).
//! Twin: engine/tools/refcore/evaluate.py (`ev`, `e_num`, `e_name`, `e_index`, `e_field`,
//! `read`, `binop`, `e_not`, `e_and`, `e_or`, `e_if`, `e_cmp`, `compare`, `e_in`, `e_record`)
//! and mreads.py (`m_mv_field`, `m_mv_kind_in`, `m_c_covered`, `m_e_moveref`); moves.py
//! (`m_find_inst`).
//! Invariants: an operation that meets a hole is the hole of its operands' questions (the hole
//! module's one rule); `none` only compares; every node kind S doesn't build stops `NotBuilt`.

use core::cmp::Ordering;

use super::arith::{Op, Test, arith, compare, duration, negate, same_value};
use super::keys::{Given, lacks_at_run, literal_key, make_keys};
use super::state::{
    Ctx, Engine, Env, Figures, invariant, not_typed, num_stop, refuse, refuse_load, unit_stop,
};
use super::ty::{figure_like, sample};
use super::windows::{as_window, window_arith, window_compare, window_negated};
use super::years::{as_year, period_like, period_literal, year_literal};
use crate::base::{Load, Run};
use crate::book::inst_ref;
use crate::hole::{Hole, spread};
use crate::model::{
    Class, Code, Domain, KeyVal, MvRef, MvSrc, NameKind, Number, Reality, Shown, Ty, Unit, Value,
    currency_id, num_money,
};
use crate::read::{Asked, member_span};
use crate::syntax::{AddOp, CmpOp, Expr, Field, FieldValue, In, Key, Lit, MulOp, Postfix, Stop};
use crate::time::Period;

impl Engine {
    /// The value of expression `e` in scope `env` and context `ctx` (refcore's `ev`). E §3.2.
    ///
    /// # Errors
    ///
    /// The refusals of L §8.4 the expression meets; `NotBuilt` for a node S doesn't build.
    pub(crate) fn ev(&mut self, m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        #[cfg(feature = "counters")]
        self.counts.add(crate::Unit::Ev);
        let v = self.node(m, e, env, ctx);
        #[cfg(feature = "counters")]
        if let Ok(Value::Number(_) | Value::Money(_)) = &v {
            self.counts.add(crate::Unit::NumResult);
        }
        v
    }

    /// One node's value, by its kind (refcore's `ev`). E §3.2.
    fn node(&mut self, m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        match e {
            Expr::Lit(lit) => literal(m, lit, env.figures),
            Expr::Str(s) => Ok(Value::Text(s.clone())),
            Expr::Name(name) => self.name(m, name, env, ctx),
            Expr::None => Ok(Value::None),
            Expr::Never => Ok(Value::Never),
            Expr::Cut => Ok(Value::Day(ctx.cut)),
            Expr::If(parts) => self.choice(m, parts, env, ctx),
            Expr::Or(terms) => self.logic(m, terms, Logic::Any, env, ctx),
            Expr::And(terms) => self.logic(m, terms, Logic::All, env, ctx),
            Expr::Not(x) => self.not(m, x, env, ctx),
            Expr::Cmp(c) => self.comparison(m, (e, &c.0, c.1, &c.2), env, ctx),
            Expr::In(x) => self.membership(m, x, env, ctx),
            Expr::Sum(first, rest) => self.sum(m, first, rest, env, ctx),
            Expr::Prod(first, rest) => self.product(m, first, rest, env, ctx),
            Expr::Neg(x) => self.minus(m, x, env, ctx),
            Expr::Post(base, posts) => self.post(m, base, posts, env, ctx),
            Expr::Call(call) => self.call(m, call, env, ctx),
            Expr::Paren(x) => self.ev(m, x, env, ctx),
            Expr::Record(fields) => self.record(m, fields, env, ctx),
            Expr::List(items) => self.list(m, items, env, ctx),
            Expr::Agg(agg) => self.aggregate(m, agg, env, ctx),
            Expr::LegOf(..) => Err(Stop::NotBuilt("a leg's amount of a party or a batch")),
            Expr::To(..) => Err(Stop::NotBuilt("`to u`, a transcendental value")),
            Expr::Last(last) => self.last_read(m, (&last.0, &last.1), env, ctx),
            Expr::Bal(b) => self.bal(m, b, env, ctx),
            Expr::Flows(f) => self.flows(m, f, env, ctx),
            Expr::Covered(c) => self.covered_call(m, (&c.0, &c.1), env, ctx),
            Expr::Calendar(..) => Err(Stop::NotBuilt("a calendar written in a rule")),
            Expr::Value(v) => self.valuation(m, v, env, ctx),
            Expr::Query(..) => Err(Stop::NotBuilt("a query (mean, share, quantile)")),
            Expr::Low(low) => self.low(m, low, env, ctx),
            Expr::Crps(..) => Err(Stop::NotBuilt("`crps`")),
            Expr::DistFn(..) => Err(Stop::NotBuilt("`cdf`, `pdf` and `pmf`")),
            Expr::Dist(..) => Err(Stop::NotBuilt("distributions")),
            Expr::Prorata(p) => self.prorata(m, p, env, ctx),
            Expr::AsIf(inner, pins) => self.as_if(m, (inner, pins), env, ctx),
        }
    }

    /// A name: a bound variable, a read of an input or a def with no keys, `cut`, an axis at
    /// its option, a thing, else a key literal its use resolves (refcore's `e_name`). L §2.5,
    /// §5.5.
    pub(super) fn name(
        &mut self,
        m: &Reality,
        name: &str,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        if let Some(var) = env.get(name) {
            return Ok(var.value.clone());
        }
        match m.names.kind(name) {
            Some(NameKind::Input | NameKind::Def) => self.read_item(m, name, &[], env, ctx),
            Some(NameKind::Move) => self.move_ref(m, name, &[], env, ctx),
            Some(NameKind::Cut) => Ok(Value::Day(ctx.cut)),
            Some(NameKind::Axis) => self.axis_option(name, ctx),
            Some(_) | None => {
                if m.things.get(name).is_some() {
                    return Ok(Value::Key(KeyVal::Thing(name.to_owned())));
                }
                Ok(Value::Word(name.to_owned()))
            }
        }
    }

    /// A movement instance named by move `id` and the keys written: the instance the cut sees
    /// at those keys, else a movement known by its id alone; a hole key makes it that hole
    /// (refcore's `e_moveref`, `find_inst`). L §2.5, §5.14.
    fn move_ref(
        &mut self,
        m: &Reality,
        id: &str,
        keys: &[Key],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let mv = m
            .moves
            .get(id)
            .ok_or_else(|| invariant("a movement named by a move load didn't build"))?;
        let given = self.given_keys(m, keys, env, ctx)?;
        let made = match make_keys(m, id, &mv.binds, given, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(made) => made,
        };
        let insts = self.move_instances(m, mv, ctx.cut)?;
        if let Some(inst) = insts.iter().find(|i| i.keys == made) {
            return Ok(Value::Mv(Box::new(inst_ref(inst))));
        }
        let id = if made.is_empty() {
            id.to_owned()
        } else {
            let texts: Vec<String> = made.iter().map(KeyVal::text).collect();
            format!("{id}[{}]", texts.join(", "))
        };
        Ok(Value::Mv(Box::new(MvRef {
            id,
            day: None,
            src: MvSrc::Missing,
        })))
    }

    /// A read of input or def `id` at the keys written (refcore's `read`): each key made a key
    /// of its position; a hole key makes the read that hole. L §4.6, §5.11, §8.1.
    pub(super) fn read_item(
        &mut self,
        m: &Reality,
        id: &str,
        keys: &[Key],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        self.read_item_in(m, (id, keys), env, (ctx, ctx))
    }

    /// [`Engine::read_item`], its keys made in context `ctx` and the item read in `read`: a
    /// scenario row's prefixed name is read in the base, its keys made where the row is read
    /// (refcore's `read`, its `_world` arm). L §4.6, §6.4.
    pub(super) fn read_item_in(
        &mut self,
        m: &Reality,
        (id, keys): (&str, &[Key]),
        env: &Env,
        (ctx, read): (Ctx, Ctx),
    ) -> Result<Value, Stop> {
        let mut given = Vec::with_capacity(keys.len());
        for key in keys {
            let Key::Expr(e) = key else {
                return Err(Stop::NotBuilt("`*` in a read's keys"));
            };
            given.push(self.given_key(m, e, env, ctx)?);
        }
        if let Some(input) = m.inputs.get(id) {
            return match make_keys(m, id, &input.binds, given, ctx)? {
                Err(hole) => Ok(Value::Hole(hole)),
                Ok(made) => {
                    self.movement_keys(m, &input.binds, &made, ctx)?;
                    self.read_input_at(m, input, &made, read, Asked::Plain)
                }
            };
        }
        let def = m
            .defs
            .get(id)
            .ok_or_else(|| invariant("a read of an item that is neither an input nor a def"))?;
        match make_keys(m, id, &def.binds, given, ctx)? {
            Err(hole) => Ok(Value::Hole(hole)),
            Ok(made) => {
                self.movement_keys(m, &def.binds, &made, ctx)?;
                self.read_def_at(m, def, &made, read)
            }
        }
    }

    /// A keyed place at the keys written: its thing; a hole key makes it that hole;
    /// `RUN-KEY` for keys that name none of its places (refcore's `e_place`). L §5.10.
    fn place_at(
        &mut self,
        m: &Reality,
        id: &str,
        keys: &[Key],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let place = m
            .places
            .get(id)
            .ok_or_else(|| invariant("a place load didn't build"))?;
        let mut given = Vec::with_capacity(keys.len());
        for key in keys {
            let Key::Expr(e) = key else {
                return Err(Stop::NotBuilt("`*` in a place's keys"));
            };
            given.push(self.given_key(m, e, env, ctx)?);
        }
        let made = match make_keys(m, id, &place.binds, given, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(made) => made,
        };
        if let Some((_, name)) = place.things.iter().find(|(k, _)| *k == made) {
            return Ok(Value::Key(KeyVal::Thing(name.clone())));
        }
        let texts: Vec<String> = made.iter().map(KeyVal::text).collect();
        Err(refuse(
            Run::Key,
            format!("{id}[{}] is not a place", texts.join(", ")),
        ))
    }

    /// A key as written: a literal, bracketed or not, or a name no variable or item holds is a
    /// key literal its position resolves; so is an int a year form gives; anything else is
    /// evaluated (L §2.1, §2.5).
    pub(crate) fn given_key(
        &mut self,
        m: &Reality,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Given, Stop> {
        match e.unbracketed() {
            Expr::Lit(
                lit @ (Lit::Num(_)
                | Lit::Day(_)
                | Lit::Month(..)
                | Lit::Quarter(..)
                | Lit::MonthDay(..)
                | Lit::Key(_)
                | Lit::Name(_)
                | Lit::Cur(_)),
            ) => Ok(Given::Literal(lit.clone())),
            Expr::Lit(Lit::Bool(_) | Lit::Nominal(_) | Lit::Inf { .. })
            | Expr::Str(_)
            | Expr::Name(_)
            | Expr::None
            | Expr::Never
            | Expr::Cut
            | Expr::If(_)
            | Expr::Or(_)
            | Expr::And(_)
            | Expr::Not(_)
            | Expr::Cmp(_)
            | Expr::In(_)
            | Expr::Sum(..)
            | Expr::Prod(..)
            | Expr::Neg(_)
            | Expr::Post(..)
            | Expr::Call(_)
            | Expr::LegOf(..)
            | Expr::To(..)
            | Expr::Paren(_)
            | Expr::Record(_)
            | Expr::List(_)
            | Expr::Agg(_)
            | Expr::Last(_)
            | Expr::Bal(_)
            | Expr::Flows(_)
            | Expr::Covered(_)
            | Expr::Calendar(_)
            | Expr::Value(_)
            | Expr::Query(_)
            | Expr::Low(_)
            | Expr::Crps(_)
            | Expr::DistFn(_)
            | Expr::Dist(_)
            | Expr::Prorata(_)
            | Expr::AsIf(..) => {
                let v = self.ev(m, e, env, ctx)?;
                if let Value::Word(word) = v {
                    return Ok(Given::Literal(Lit::Name(word)));
                }
                if let Some(lit) = year_literal(e, &v)? {
                    return Ok(Given::Literal(lit));
                }
                Ok(Given::Value(v))
            }
        }
    }

    /// A base with its postfixes: a read with keys and fields, or a value's fields (refcore's
    /// `e_index`, `e_field`). L §2.3, §3.1.
    fn post(
        &mut self,
        m: &Reality,
        base: &Expr,
        posts: &[Postfix],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let (mut value, rest) = match (base, posts) {
            (Expr::Name(prefix), [Postfix::Field(name), rest @ ..])
                if env.get(prefix).is_none() && self.reads_base(prefix) =>
            {
                match (self.base_read(m, (name, rest), env, ctx)?, rest) {
                    ((value, true), [_, after @ ..]) => (value, after),
                    ((value, _), _) => (value, rest),
                }
            }
            (Expr::Name(name), [Postfix::Keys(keys), rest @ ..])
                if env.get(name).is_none() && is_read(m, name) =>
            {
                (self.read_item(m, name, keys, env, ctx)?, rest)
            }
            (Expr::Name(name), [Postfix::Keys(keys), rest @ ..])
                if env.get(name).is_none() && m.places.get(name).is_some() =>
            {
                (self.place_at(m, name, keys, env, ctx)?, rest)
            }
            (Expr::Name(name), [Postfix::Keys(keys), rest @ ..])
                if env.get(name).is_none() && m.names.kind(name) == Some(NameKind::Move) =>
            {
                (self.move_ref(m, name, keys, env, ctx)?, rest)
            }
            (Expr::Name(name), [Postfix::Keys(_), ..]) if env.get(name).is_none() => {
                return Err(Stop::NotBuilt("keys of a place, a move or a member"));
            }
            _ => (self.ev(m, base, env, ctx)?, posts),
        };
        for post in rest {
            value = match post {
                Postfix::Field(name) => {
                    if let Value::Mv(mv) = &value {
                        self.mv_field(m, mv, name, ctx)?
                    } else {
                        field(m, value, name)?
                    }
                }
                Postfix::Keys(_) => return Err(Stop::NotBuilt("keys after a value")),
            };
        }
        Ok(value)
    }

    /// `if c then a else b` (refcore's `e_if`): a hole condition evaluates both branches, and
    /// the `if` is a hole unless they are equal (L §5.11 Logic, §8.1). A bare figure as a
    /// branch takes the other branch's type (L §3.2 Inline figures).
    fn choice(
        &mut self,
        m: &Reality,
        parts: &[Expr; 3],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [cond, yes, no] = parts;
        let test = self.ev(m, cond, env, ctx)?;
        if let Value::Bool(holds) = test {
            let (taken, other) = if holds { (yes, no) } else { (no, yes) };
            let value = self.ev(m, taken, env, ctx)?;
            return flex_branch(m, value, other, env, ctx);
        }
        if test == Value::None {
            return Err(refuse(Run::None, "a value is none here: if".to_owned()));
        }
        let Value::Hole(cond_hole) = test else {
            return Err(not_typed());
        };
        let first = self.ev(m, yes, env, ctx)?;
        let second = self.ev(m, no, env, ctx)?;
        let first = flex_branch(m, first, no, env, ctx)?;
        let second = flex_branch(m, second, yes, env, ctx)?;
        if first.as_hole().is_none()
            && second.as_hole().is_none()
            && same_value(m, &first, &second)?
        {
            return Ok(first);
        }
        hole_of(&[Some(&cond_hole), first.as_hole(), second.as_hole()])
    }

    /// `a or b or …` (`any`) and `a and b and …` (`all`) under Kleene logic, left to right as
    /// refcore's binary nodes: a decided term decides, a hole stays only where nothing does
    /// (refcore's `e_or`, `e_and`). L §5.11, §8.1.
    fn logic(
        &mut self,
        m: &Reality,
        terms: &[Expr],
        kind: Logic,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let (word, any) = match kind {
            Logic::Any => ("or", true),
            Logic::All => ("and", false),
        };
        if kind == Logic::All
            && !m.narrow_tests.is_empty()
            && self.narrowed_false(m, terms, env, ctx)?
        {
            return Ok(Value::Bool(false));
        }
        let mut acc: Option<Value> = None;
        for term in terms {
            if let Some(Value::Bool(b)) = &acc
                && *b == any
            {
                return Ok(Value::Bool(any));
            }
            let v = self.ev(m, term, env, ctx)?;
            let v = boolean(v, word)?;
            if v == Value::Bool(any) {
                return Ok(Value::Bool(any));
            }
            acc = Some(match acc {
                None => v,
                Some(prev) => match spread(&[prev.as_hole(), v.as_hole()]) {
                    Some(h) => Value::Hole(h),
                    None => Value::Bool(!any),
                },
            });
        }
        acc.ok_or_else(|| invariant("a logic chain of no terms"))
    }

    /// Whether a kind test of conjunction `terms` whose narrowing a field read uses is false,
    /// the tests taken first, left to right: a conjunct that reads the narrowed fields is
    /// evaluated only where the test holds, and the conjunction is false where it doesn't
    /// (refcore's `e_and`, its `_narrow`). L §5.7.
    fn narrowed_false(
        &mut self,
        m: &Reality,
        terms: &[Expr],
        env: &Env,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        let mut work: Vec<&Expr> = terms.iter().rev().collect();
        while let Some(node) = work.pop() {
            if let Expr::Paren(inner) = node {
                work.push(inner);
            }
            if let Expr::And(inner) = node {
                work.extend(inner.iter().rev());
            }
            if let Expr::In(test) = node
                && m.narrow_tests.contains(&test.site.key())
                && self.membership(m, test, env, ctx)? == Value::Bool(false)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `not e` (refcore's `e_not`). L §5.11.
    fn not(&mut self, m: &Reality, x: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let v = self.ev(m, x, env, ctx)?;
        let tested = boolean(v, "not")?;
        if let Value::Bool(holds) = tested {
            return Ok(Value::Bool(!holds));
        }
        Ok(tested)
    }

    /// `a op b` (refcore's `e_cmp`, `compare`): `none` only tests equal, a hole spreads,
    /// `never` is later than every date, a key literal resolves against the other side, a
    /// window decides or is `HOLE-IMPRECISE` naming the comparison `written`. L §3.1, §4.4,
    /// §5.11, §8.4.
    fn comparison(
        &mut self,
        m: &Reality,
        (written, lhs, op, rhs): (&Expr, &Expr, CmpOp, &Expr),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let left = self.ev(m, lhs, env, ctx)?;
        let right = self.ev(m, rhs, env, ctx)?;
        let left = year_beside(lhs, left, &right);
        let right = year_beside(rhs, right, &left);
        let (left, right) = resolve_pair(m, left, right)?;
        compare_values(m, op, (&left, &right), written)
    }
}

/// `e == f` of two values already read, as a comparison of the two expressions decides it
/// (refcore's `compare('==', …)` in `check`): true, false or the hole of an undecided
/// comparison. L §5.18, §5.11.
///
/// # Errors
///
/// The refusals of a comparison, `RUN-NONE` and `LOAD-UNIT` among them.
pub(crate) fn equal_sides(
    m: &Reality,
    (lhs, left): (&Expr, &Value),
    (rhs, right): (&Expr, &Value),
) -> Result<Value, Stop> {
    let left_year = year_beside(lhs, left.clone(), right);
    let right_year = year_beside(rhs, right.clone(), &left_year);
    let (a, b) = resolve_pair(m, left_year, right_year)?;
    let written = Expr::Cmp(Box::new((lhs.clone(), CmpOp::Eq, rhs.clone())));
    compare_values(m, CmpOp::Eq, (&a, &b), &written)
}

impl Engine {
    /// `x in K [on d]`: a thing's membership of a kind; `x in [a, b]`: a list's (refcore's
    /// `e_in`). A `d` that is a hole makes the test that hole; one that is `none`, `never` or
    /// no day is refused as `date_of` refuses it, and a day period stands for its first day.
    /// L §5.7, §8.4.
    fn membership(&mut self, m: &Reality, x: &In, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let member = self.ev(m, &x.member, env, ctx)?;
        if let Value::Hole(_) = member {
            return Ok(member);
        }
        if member == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: a membership test".to_owned(),
            ));
        }
        if let Expr::List(items) = &x.set {
            let text = member.text(m).map_err(num_stop)?;
            return Ok(Value::Bool(items.iter().any(|l| l.text() == text)));
        }
        let Expr::Name(kind) = &x.set else {
            return Err(Stop::NotBuilt("membership in a value"));
        };
        if m.kinds.get(kind).is_none() {
            return Err(Stop::NotBuilt("membership in a value"));
        }
        if let Value::Mv(mv) = &member {
            return Ok(Value::Bool(self.mv_kind_in(m, mv, kind, ctx)?));
        }
        let Value::Key(KeyVal::Thing(name)) = &member else {
            return Err(refuse_load(
                Load::Unit,
                format!(
                    "membership of {} in a kind",
                    member.text(m).map_err(num_stop)?
                ),
            ));
        };
        let Some(thing) = m.things.get(name) else {
            return Ok(Value::Bool(false));
        };
        if let Some(on) = &x.on {
            let day = self.ev(m, on, env, ctx)?;
            if let Value::Hole(h) = day {
                return Ok(Value::Hole(h));
            }
            let day = crate::ask::date_of(&day, "the day of a membership test")?;
            self.note_thing(m, thing, ctx.world);
            let alive =
                member_span(m, thing, kind, (ctx.cut, ctx.world)).is_some_and(|s| s.contains(day));
            return Ok(Value::Bool(alive));
        }
        if based_in(m, thing, kind) {
            return Ok(Value::Bool(true));
        }
        let sub = m.subtree(kind);
        if thing.entries.iter().any(|e| sub.contains(&e.kind)) {
            return Err(refuse(
                Run::Domain,
                format!(
                    "{name} in {kind} with no on: a membership with a lifespan tested without a day"
                ),
            ));
        }
        Ok(Value::Bool(false))
    }

    /// `a ± b ± …`, left to right (refcore's `e_bin`). L §3.2.
    fn sum(
        &mut self,
        m: &Reality,
        first: &Expr,
        rest: &[(AddOp, Expr)],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let mut acc = self.ev(m, first, env, ctx)?;
        let mut written = Some(first);
        for (op, x) in rest {
            let b = self.ev(m, x, env, ctx)?;
            if let Some(e) = written.take()
                && moves_a_year(&b)
                && let Some(year) = as_year(Some(e), &acc)
            {
                acc = Value::Period(year);
            }
            let op = match op {
                AddOp::Add => Op::Add,
                AddOp::Sub => Op::Sub,
            };
            acc = binop(m, op, &acc, &b)?;
        }
        Ok(acc)
    }

    /// `a × b ÷ …`, left to right (refcore's `e_bin`). L §3.2.
    fn product(
        &mut self,
        m: &Reality,
        first: &Expr,
        rest: &[(MulOp, Expr)],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let mut acc = self.ev(m, first, env, ctx)?;
        for (op, x) in rest {
            let b = self.ev(m, x, env, ctx)?;
            let op = match op {
                MulOp::Mul => Op::Mul,
                MulOp::Div => Op::Div,
            };
            acc = binop(m, op, &acc, &b)?;
        }
        Ok(acc)
    }

    /// `-e` (refcore's `e_neg`). L §3.2.
    fn minus(&mut self, m: &Reality, x: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let v = self.ev(m, x, env, ctx)?;
        if let Value::Hole(_) = v {
            return Ok(v);
        }
        if v == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: arithmetic".to_owned(),
            ));
        }
        let negated = if let Value::Window(w) = &v {
            window_negated(w)?
        } else {
            negate(&v)?
        };
        match negated {
            Some(n) => Ok(n),
            None => Err(refuse_load(
                Load::Unit,
                format!("unary minus on {}", v.text(m).map_err(num_stop)?),
            )),
        }
    }

    /// A list `[a, b]` (refcore's `e_list`): each item's value, a name evaluated as any name
    /// is (a read of an input, a bound variable, a thing, else its word), as load typed it.
    /// L §2.5, §4.4.
    fn list(&mut self, m: &Reality, items: &[Lit], env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let mut out = Vec::with_capacity(items.len());
        for lit in items {
            let v = if let Lit::Name(name) = lit {
                self.name(m, name, env, ctx)?
            } else {
                literal(m, lit, Figures::Rule)?
            };
            out.push(v);
        }
        Ok(Value::List(out))
    }

    /// A record literal `(a: x, b: y)` (refcore's `e_record`). L §3.1.
    fn record(
        &mut self,
        m: &Reality,
        fields: &[Field],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let mut out = Vec::with_capacity(fields.len());
        for f in fields {
            let Field::Named(name, value) = f else {
                // load refuses it; refcore's run has no twin of this text
                return Err(Stop::NotBuilt("a leg in a record, met at run"));
            };
            let v = match value {
                FieldValue::Hole(text) => Value::Hole(Hole::asked(text, "")),
                FieldValue::None => Value::None,
                FieldValue::Lit(lit) => literal(m, lit, env.figures)?,
                FieldValue::Expr(e) => self.ev(m, e, env, ctx)?,
            };
            out.push((name.clone(), v));
        }
        Ok(Value::Record(out))
    }
}

/// A chain of logic: `or` holds when any term does, `and` when all do (L §5.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Logic {
    /// `or`.
    Any,
    /// `and`.
    All,
}

/// A side of a comparison written in a year form, or as a month or quarter, beside a period:
/// the year it names, and beside a key of a period index of its kind the key it labels on the
/// index's grid, an anchored year or a period `by n` included (refcore's `_year` mark of
/// `compare`, and its `resolve_key_literal` of a literal beside a key of a period index).
/// L §2.1, §2.5, §4.1.
fn year_beside(e: &Expr, v: Value, other: &Value) -> Value {
    let Value::Period(like) = other else {
        return v;
    };
    if let Some(year) = as_year(Some(e), &v) {
        return Value::Period(period_like(*like, year));
    }
    if let Value::Period(written) = &v
        && period_literal(e)
    {
        return Value::Period(period_like(*like, *written));
    }
    v
}

/// Whether a value moves or measures a year in a sum: a period, or a duration. A four-digit
/// figure before one reads as a year, the one reading its arithmetic takes (refcore's
/// `ambig`). L §2.1, §4.2.
fn moves_a_year(v: &Value) -> bool {
    if let Value::Number(n) = v {
        return duration(n).is_some();
    }
    matches!(v, Value::Period(_))
}

/// A bare figure that is an `if`'s branch, in the type of the other branch (refcore's
/// `flex_to` of `t_if`). L §3.2.
fn flex_branch(m: &Reality, v: Value, other: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
    let Value::Number(Number {
        q,
        class: Class::Flex,
    }) = &v
    else {
        return Ok(v);
    };
    let Some(like) = sample(m, other, env, ctx)? else {
        return Ok(v);
    };
    Ok(figure_like(m, q, &like)?.unwrap_or(v))
}

/// Whether a thing's base kind is `kind` or below it (L §5.7).
fn based_in(m: &Reality, thing: &crate::model::Thing, kind: &str) -> bool {
    let Some(base) = thing.base.as_deref() else {
        return false;
    };
    for k in m.chain(base) {
        if k.id == kind {
            return true;
        }
    }
    false
}

/// Whether `name` names an input or a def.
pub(super) fn is_read(m: &Reality, name: &str) -> bool {
    matches!(m.names.kind(name), Some(NameKind::Input | NameKind::Def))
}

/// The hole of the operands' questions (the hole module's one rule, L §8.1).
pub(super) fn hole_of(operands: &[Option<&Hole>]) -> Result<Value, Stop> {
    spread(operands)
        .map(Value::Hole)
        .ok_or_else(|| invariant("a hole spread over operands that hold none"))
}

/// A value in a logic position: a bool, or a hole; `none` is refused (L §3.1, §5.11).
fn boolean(v: Value, word: &str) -> Result<Value, Stop> {
    if let Value::Bool(_) | Value::Hole(_) = v {
        return Ok(v);
    }
    if v == Value::None {
        return Err(refuse(Run::None, format!("a value is none here: {word}")));
    }
    Err(not_typed())
}

/// `a op b` on two values: a hole spreads, unless the other is `none` or `never`, which are
/// refused whatever the hole holds (refcore's `binop`). L §3.1, §8.4.
pub(super) fn binop(m: &Reality, op: Op, lhs: &Value, rhs: &Value) -> Result<Value, Stop> {
    for v in [lhs, rhs] {
        if *v == Value::Never {
            return Err(refuse(Run::None, "never in arithmetic".to_owned()));
        }
        if *v == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: arithmetic".to_owned(),
            ));
        }
        if let Value::Word(_) = v {
            // load resolves a key word; refcore's run refuses it with its place, which a
            // value here doesn't carry
            return Err(Stop::NotBuilt("an ambiguous key met at run"));
        }
    }
    if lhs.as_hole().is_some() || rhs.as_hole().is_some() {
        return hole_of(&[lhs.as_hole(), rhs.as_hole()]);
    }
    if as_window(lhs).is_some() || as_window(rhs).is_some() {
        return window_arith(m, op, lhs, rhs);
    }
    arith(m, op, lhs, rhs)
}

/// Two values that meet, a key literal on either side resolved in the key on the other's
/// index or kind (refcore's static resolution of a key literal by what it meets). L §2.5.
pub(super) fn resolve_pair(m: &Reality, lhs: Value, rhs: Value) -> Result<(Value, Value), Stop> {
    if let Value::Word(word) = &lhs {
        let resolved = resolve_word(m, word, &rhs)?;
        return Ok((resolved, rhs));
    }
    if let Value::Word(word) = &rhs {
        let resolved = resolve_word(m, word, &lhs)?;
        return Ok((lhs, resolved));
    }
    Ok((lhs, rhs))
}

/// Key literal `word` as a key of the index or kind of value `other`, an option of its axis
/// (L §5.5), or a code beside a code.
fn resolve_word(m: &Reality, word: &str, other: &Value) -> Result<Value, Stop> {
    if let Value::Key(KeyVal::Key { index, .. }) = other
        && m.axes.contains(index)
    {
        let key = literal_key(m, &Lit::Name(word.to_owned()), &Domain::Axis(index.clone()))?
            .ok_or_else(lacks_at_run)?;
        return Ok(Value::of_key(&key));
    }
    if let Value::Key(KeyVal::Key { index, .. }) = other {
        let key = literal_key(
            m,
            &Lit::Name(word.to_owned()),
            &Domain::Index(index.clone()),
        )?
        .ok_or_else(lacks_at_run)?;
        return Ok(Value::of_key(&key));
    }
    if let Value::Key(KeyVal::Thing(_)) = other {
        return Ok(Value::Key(KeyVal::Thing(word.to_owned())));
    }
    if let Value::Code(_) = other {
        return Ok(Value::Code(Code::Word(word.to_owned())));
    }
    Ok(Value::Word(word.to_owned()))
}

/// The comparison `a op b` of two values, keys resolved, as `written` (refcore's `compare`).
/// L §3.1, §4.4, §8.4.
pub(super) fn compare_values(
    m: &Reality,
    op: CmpOp,
    (a, b): (&Value, &Value),
    written: &Expr,
) -> Result<Value, Stop> {
    let eq = matches!(op, CmpOp::Eq | CmpOp::Ne);
    let holds = |same: bool| Value::Bool(same == (op == CmpOp::Eq));
    if *a == Value::None || *b == Value::None {
        if !eq {
            return Err(refuse(
                Run::None,
                "a value is none here: an ordering".to_owned(),
            ));
        }
        let other = if *a == Value::None { b } else { a };
        if let Value::Hole(h) = other {
            return Ok(Value::Hole(h.clone()));
        }
        return Ok(holds(a == b));
    }
    if a.as_hole().is_some() || b.as_hole().is_some() {
        return hole_of(&[a.as_hole(), b.as_hole()]);
    }
    if *a == Value::Never || *b == Value::Never {
        if eq {
            return Ok(holds(a == b));
        }
        if a == b {
            return Ok(Value::Bool(matches!(op, CmpOp::Le | CmpOp::Ge)));
        }
        let later = *a == Value::Never;
        return Ok(Value::Bool(match op {
            CmpOp::Lt | CmpOp::Le => !later,
            CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne => later,
        }));
    }
    if as_window(a).is_some() || as_window(b).is_some() {
        return window_compare(m, op, (a, b), &crate::load::expr_text(written));
    }
    if let (Value::Word(x), Value::Word(y)) = (a, b) {
        return Ok(holds(x == y));
    }
    if let (Value::Mv(x), Value::Mv(y), true) = (a, b, eq) {
        return Ok(holds(x.id == y.id));
    }
    let test = if eq { Test::Equality } else { Test::Order };
    let order = compare(m, a, b, test)?;
    Ok(Value::Bool(match op {
        CmpOp::Eq => order == Ordering::Equal,
        CmpOp::Ne => order != Ordering::Equal,
        CmpOp::Lt => order == Ordering::Less,
        CmpOp::Le => order != Ordering::Greater,
        CmpOp::Gt => order == Ordering::Greater,
        CmpOp::Ge => order != Ordering::Less,
    }))
}

/// A record's field `name` (refcore's `e_field`). L §3.1.
fn field(m: &Reality, v: Value, name: &str) -> Result<Value, Stop> {
    if let Value::Hole(_) = v {
        return Ok(v);
    }
    if v == Value::None {
        return Err(refuse(
            Run::None,
            format!("a value is none here: a field read of it ({name})"),
        ));
    }
    let Value::Record(fields) = v else {
        return Err(refuse_load(
            Load::Field,
            format!("{name} is not a field of {}", v.text(m).map_err(num_stop)?),
        ));
    };
    fields
        .into_iter()
        .find(|(f, _)| f == name)
        .map(|(_, value)| value)
        // load's types refuse it; refcore's run names the record by its id
        .ok_or(Stop::NotBuilt("a field a record lacks, met at run"))
}

/// A literal's value (refcore's `e_num` and the literal nodes): a number with a currency is
/// money, `19 %` a `num %`; a bare `0`, `1`, `-1` (any bare literal in a band's row) is a
/// figure its context types (L §3.2 Inline figures, §7.1).
///
/// # Errors
///
/// `LOAD-PRECISION` for money finer than its minor unit; `NotBuilt` for `inf` in an
/// expression.
pub(super) fn literal(m: &Reality, lit: &Lit, figures: Figures) -> Result<Value, Stop> {
    match lit {
        Lit::Num(n) => number(m, n, figures),
        Lit::Day(d) => Ok(Value::Day(*d)),
        Lit::Month(y, mo) => Period::month(*y, *mo)
            .map(Value::Period)
            .ok_or_else(|| invariant("a month literal the parser read")),
        Lit::Quarter(y, q) => Period::quarter(*y, *q)
            .map(Value::Period)
            .ok_or_else(|| invariant("a quarter literal the parser read")),
        Lit::MonthDay(mo, d) => Ok(Value::MonthDay(*mo, *d)),
        Lit::Key(t) | Lit::Name(t) | Lit::Cur(t) => Ok(Value::Word(t.clone())),
        Lit::Bool(b) => Ok(Value::Bool(*b)),
        // refcore takes any size: it answers where the compounding divides the period
        // (`periodic(r, month, nominal 120000000000000000000)`) and refuses the power
        // `LOAD-TO` where it doesn't; the engine keeps a compounding in 64 bits and stops
        // past them as not built
        Lit::Nominal(n) => n
            .to_i64()
            .map(|n| Value::Code(Code::Nominal(n)))
            .ok_or(Stop::NotBuilt("a nominal compounding past 64 bits")),
        Lit::Inf { .. } => Err(Stop::NotBuilt("inf in an expression")),
    }
}

/// A number literal's value. L §3.2, §7.1.
fn number(m: &Reality, n: &crate::syntax::Number, figures: Figures) -> Result<Value, Stop> {
    let Some(unit) = &n.unit else {
        let unit_figure = n.value.is_zero()
            || n.value == crate::num::Num::int(1)
            || n.value == crate::num::Num::int(-1);
        let figure = figures == Figures::Band || (n.whole && unit_figure);
        let class = if figure {
            Class::Flex
        } else if n.whole {
            Class::Int
        } else {
            Class::Num(Unit::one(), Shown::Plain)
        };
        return Ok(Value::Number(Number {
            q: n.value.clone(),
            class,
        }));
    };
    let terms: Vec<(String, crate::num::Num)> = unit
        .terms
        .iter()
        .map(|t| (t.base.clone(), t.power.clone()))
        .collect();
    let percent = terms == [("%".to_owned(), crate::num::Num::int(1))];
    let (u, factor) = Unit::from_syntax(terms).map_err(unit_stop)?;
    let q = n.value.checked_mul(&factor).map_err(num_stop)?;
    if percent {
        return Ok(Value::Number(Number {
            q,
            class: Class::Num(Unit::one(), Shown::Percent),
        }));
    }
    if u.is_currency()
        && let [(cur, _)] = u.terms()
    {
        let id = currency_id(m, cur).map_err(num_stop)?;
        // load refuses a figure finer than the minor unit with its place
        return num_money(m, &q, id)
            .map(Value::Money)
            .map_err(|_| Stop::NotBuilt("money finer than its minor unit, met at run"));
    }
    Ok(Value::Number(Number {
        q,
        class: Class::Num(u, Shown::Plain),
    }))
}

/// A bare figure `q` in type `ty` (refcore's `typed`): money, an int, `1` as 1 % of a
/// `num %`, a `num u`; `None` for a type a figure doesn't take. L §3.2.
///
/// # Errors
///
/// A number error; `LOAD-PRECISION` for money finer than its minor unit.
pub(super) fn typed(m: &Reality, q: &crate::num::Num, ty: &Ty) -> Result<Option<Value>, Stop> {
    match ty {
        Ty::Money(cur) => {
            let id = currency_id(m, cur).map_err(num_stop)?;
            // load refuses a figure finer than the minor unit with its place
            let money = num_money(m, q, id)
                .map_err(|_| Stop::NotBuilt("a figure finer than the minor unit, met at run"))?;
            Ok(Some(Value::Money(money)))
        }
        Ty::Int => Ok(Some(Value::Number(Number {
            q: q.clone(),
            class: Class::Int,
        }))),
        Ty::Num(_, Shown::Percent) => {
            let hundredth = crate::num::Num::ratio(1, 100).map_err(num_stop)?;
            Ok(Some(Value::Number(Number {
                q: q.checked_mul(&hundredth).map_err(num_stop)?,
                class: Class::Num(Unit::one(), Shown::Percent),
            })))
        }
        Ty::Num(unit, shown) => Ok(Some(Value::Number(Number {
            q: q.clone(),
            class: Class::Num(unit.clone(), *shown),
        }))),
        Ty::Bool
        | Ty::Text
        | Ty::Date
        | Ty::Dates
        | Ty::Period(_)
        | Ty::Key(_)
        | Ty::Rate
        | Ty::Dist(_)
        | Ty::Calendar
        | Ty::Code(_)
        | Ty::MonthDay
        | Ty::Record(_)
        | Ty::List(_)
        | Ty::Tuple(_)
        | Ty::Flex(_)
        | Ty::Year4
        | Ty::KeyLit(_)
        | Ty::KeyList
        | Ty::Dyn
        | Ty::UnitKey
        | Ty::Unseen
        | Ty::None
        | Ty::Hole
        | Ty::Never
        | Ty::DistKeys(_)
        | Ty::KindName => Ok(None),
    }
}
