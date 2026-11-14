//! Load's checker, its random values: distributions, `cdf`, `pdf` and `pmf`, queries, `crps`, and how each read of a random fact is recorded.
//! Spec: L §5.16 (distributions and paths, the closed list, samples, queries), §5.15
//! (`crps`), §5.11 (queries over keys), §8.3 (`LOAD-RANDOM`, `-TO`, `-DOMAIN`, `-UNIT`).
//! Twin: engine/tools/refcore/check2.py (`m_tc_dist`, `m_dist_read_type`, `m_note_read`,
//! `m_sample_type`, `m_discrete_index`, `m_t_dist`, `m_dist_whole`, `m_dist_affine_syntax`,
//! `m_t_distfn`, `m_unit_or`, `m_t_query_random`, `m_dist_value_query`, `m_t_crps`,
//! `m_check_dist_rows`, `dist_support_max`, with `discrete`'s keys and a family's `to u`
//! seen, issue #75); check.py (`t_query`, its arm over keys).
//! Invariants: each form is typed in refcore's order with refcore's messages; a type load
//! doesn't see (`Unseen`) refuses nothing; each read of an input or a def is recorded with
//! its role for the random cone (`Graph.rand_edges`).

use super::check::{Checker, Files, ReadIn, Scope, Stands, VarDom, invariant};
use super::exact::Under;
use super::types::{Typed, converts, is_bool, num, numcls, refused, uerr, unit_of};
use crate::base::Load;
use crate::model::{IndexForm, KeyVal, NameKind, Node, NodeKind, ReadRole, Reality, Ty, Unit};
use crate::num::Num;
use crate::syntax::{
    Bind, Crps, Dist, DistFn, DistFnKind, DistKind, Domain as Written, Expr, Lit, Query, QueryFn,
    RuleValue, Stop, Value,
};
use std::collections::BTreeSet;

/// A query or a path score over random values, where it stands (refcore's `queries` entry).
#[derive(Debug, Clone)]
pub(super) struct QueryAt {
    /// The `mean`, `share`, `quantile` or `crps` form, as written.
    pub(super) form: Expr,
    /// The item that holds it.
    pub(super) item: Node,
    /// Where it stands.
    pub(super) stands: Stands,
    /// The file and line it is written on.
    pub(super) at: (String, u32),
}

/// A slot key that a range up to a value gives, at a key of an index (refcore's
/// `draw_keys` entry): a random number of items when the value is random. L §5.16.
#[derive(Debug, Clone)]
pub(super) struct DrawKey {
    /// The range's end, as written: the draw.
    pub(super) end: Expr,
    /// The index whose key it is: the slots.
    pub(super) index: String,
    /// The file and line it is read on.
    pub(super) at: (String, u32),
}

/// How far a distribution written in a row or a rule can draw, as its text shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Support {
    /// Unbounded above.
    Unbounded,
    /// Up to a value its text states.
    UpTo(Num),
    /// It rests on facts, or the text is no distribution.
    Unseen,
}

/// The largest value a distribution written in a row or a rule can draw, where its text shows
/// it (refcore's `dist_support_max`): literal bounds or, for `discrete`, the largest of its
/// keys load sees (issue #75); a continuous family is seen through its `to u`. L §5.16.
pub(super) fn support_max(m: &Reality, value: &Expr) -> Support {
    let value = if let Expr::To(inner, _) = value {
        &**inner
    } else {
        value
    };
    let Expr::Dist(dist) = value else {
        return Support::Unseen;
    };
    let literal = |at: usize| {
        if let Some(Expr::Lit(Lit::Num(n))) = dist.args.get(at) {
            Support::UpTo(n.value.clone())
        } else {
            Support::Unseen
        }
    };
    match dist.func {
        DistKind::Poisson
        | DistKind::Negbin
        | DistKind::Normal
        | DistKind::Lognormal
        | DistKind::Gamma
        | DistKind::Exponential => Support::Unbounded,
        DistKind::Binomial => literal(0),
        DistKind::Uniform => literal(1),
        DistKind::Discrete => dist
            .bind
            .as_ref()
            .and_then(|bind| discrete_max(m, bind))
            .map_or(Support::Unseen, Support::UpTo),
    }
}

/// The largest key a `discrete` draws, where load sees its keys: an integer index's. A key
/// list holds names, and a stepped range draws figures of mixed sample types, refused before
/// (L §5.6, §5.16).
fn discrete_max(m: &Reality, bind: &Bind) -> Option<Num> {
    let Written::Name { id, qual: None } = &bind.domain else {
        return None;
    };
    let index = m.indexes.get(id).filter(|index| index.is_int)?;
    let mut keys = Vec::new();
    for key in index.keys() {
        let KeyVal::Int(k) = key else {
            return None;
        };
        keys.push(*k);
    }
    keys.into_iter().max().map(Num::int)
}

/// The expression inside any brackets (refcore's `strip`).
pub(super) fn strip(e: &Expr) -> &Expr {
    let mut inner = e;
    while let Expr::Paren(nested) = inner {
        inner = nested;
    }
    inner
}

/// The distribution inside a `to u`, or the expression itself, bracket-stripped.
fn strip_to(e: &Expr) -> &Expr {
    let inner = strip(e);
    if let Expr::To(nested, _) = inner {
        return strip(nested);
    }
    inner
}

/// A distribution's name as written.
pub(super) fn dist_word(kind: DistKind) -> &'static str {
    match kind {
        DistKind::Normal => "normal",
        DistKind::Lognormal => "lognormal",
        DistKind::Exponential => "exponential",
        DistKind::Gamma => "gamma",
        DistKind::Uniform => "uniform",
        DistKind::Poisson => "poisson",
        DistKind::Binomial => "binomial",
        DistKind::Negbin => "negbin",
        DistKind::Discrete => "discrete",
    }
}

/// Whether a distribution is continuous (refcore's `CONT`). L §5.16.
pub(super) fn continuous(kind: DistKind) -> bool {
    matches!(
        kind,
        DistKind::Normal
            | DistKind::Lognormal
            | DistKind::Exponential
            | DistKind::Gamma
            | DistKind::Uniform
    )
}

/// A function's name as written.
fn fn_word(kind: DistFnKind) -> &'static str {
    match kind {
        DistFnKind::Cdf => "cdf",
        DistFnKind::Pdf => "pdf",
        DistFnKind::Pmf => "pmf",
    }
}

/// A query's name as written.
fn query_word(func: QueryFn) -> &'static str {
    match func {
        QueryFn::Mean => "mean",
        QueryFn::Share => "share",
        QueryFn::Quantile => "quantile",
    }
}

/// Whether `t` is a plain number: an int, a bare figure or a `num` with no unit; a type load
/// doesn't see passes.
fn plain(m: &Reality, t: &Ty) -> Result<bool, Stop> {
    if matches!(t, Ty::Unseen) {
        return Ok(true);
    }
    let c = numcls(m, t);
    if !matches!(c, Some("num" | "int" | "flex")) {
        return Ok(false);
    }
    Ok(c != Some("num") || unit_of(t)?.is_one())
}

/// The unit of a number, none for an int or a bare figure (refcore's `unit_or`). L §5.16.
fn unit_or(m: &Reality, t: &Ty, what: &str) -> Result<Unit, Stop> {
    let c = numcls(m, t);
    if matches!(c, Some("num" | "money")) {
        return unit_of(t);
    }
    if matches!(c, Some("flex" | "int")) || matches!(t, Ty::Unseen) {
        return Ok(Unit::one());
    }
    Err(uerr(format!("{what} is {}, not a number", t.show())))
}

/// The unit of a number type, none for anything else (refcore's `t.u if c in ('num',
/// 'money') else ()`).
fn num_unit(m: &Reality, t: &Ty) -> Result<Unit, Stop> {
    if matches!(numcls(m, t), Some("num" | "money")) {
        return unit_of(t);
    }
    Ok(Unit::one())
}

/// A unit as refcore's `fmt` prints it, `num` when it has none.
fn unit_text(unit: &Unit) -> String {
    if unit.is_one() {
        return "num".to_owned();
    }
    unit.text()
}

/// `LOAD-TO` for a continuous distribution written whole in a row, a rule or a pin, with no
/// `to u` (refcore's `dist_whole`). L §5.16.
pub(super) fn dist_whole(node: &Expr, place: &str) -> Result<(), Stop> {
    if let Expr::Dist(d) = strip(node)
        && continuous(d.func)
    {
        return Err(refused(
            Load::To,
            format!(
                "continuous distribution {} needs to u ({place})",
                dist_word(d.func)
            ),
        ));
    }
    Ok(())
}

/// The index a random fact's `discrete` rows or rules range over, if they are all one
/// index's (refcore's `discrete_index`); `LOAD-UNIT` for a mix. L §5.16.
///
/// # Errors
///
/// `LOAD-UNIT` for a mix; `ENGINE-INVARIANT` for a `discrete` with no binding, which the
/// parser refuses first.
fn discrete_index(m: &Reality, files: &Files, target: &Node) -> Result<Option<String>, Stop> {
    let mut nodes: Vec<Option<&Expr>> = Vec::new();
    if target.kind == NodeKind::Def {
        if let Some(def) = m.defs.get(&target.id) {
            for (_, versions) in &def.rules {
                for version in versions {
                    nodes.push(match &version.rule.value {
                        RuleValue::Expr(e) => Some(e),
                        RuleValue::Solve(_) | RuleValue::Hole(_) => None,
                    });
                }
            }
        }
    } else if let Some(input) = m.inputs.get(&target.id) {
        for row in &input.rows {
            if let Value::Dist(e) = &row.value {
                nodes.push(Some(e));
            }
        }
    }
    // each domain's index, none for a list or a range, and its text as written (L §2.4
    // Values, tally 2.1.7 R9)
    let mut over: BTreeSet<Option<String>> = BTreeSet::new();
    let mut shown: BTreeSet<String> = BTreeSet::new();
    let mut other = false;
    for node in nodes {
        if let Some(Expr::Dist(d)) = node.map(strip_to)
            && d.func == DistKind::Discrete
        {
            // the parser gives every `discrete` its `for` binding, refusing one written
            // without it (`dist`, PARSE)
            let domain = &d
                .bind
                .as_ref()
                .ok_or_else(|| invariant("a discrete with no binding".to_owned()))?
                .domain;
            let ix = match domain {
                Written::Name { id, .. } => Some(id.clone()),
                Written::List(_) | Written::Range(_) => None,
            };
            shown.insert(super::texts::domain_text(domain));
            over.insert(ix);
        } else {
            other = true;
        }
    }
    let Some(first) = over.first().cloned() else {
        return Ok(None);
    };
    if let (Some(ix), 1, false) = (first, over.len(), other) {
        return Ok(Some(ix));
    }
    let names: Vec<String> = shown.into_iter().collect();
    let tail = if other {
        " and other distributions"
    } else {
        ""
    };
    let message = format!(
        "random fact {} mixes sample types: discrete over {}{tail}",
        target.id,
        names.join(", ")
    );
    Err(at_decl(m, files, target, uerr(message))?)
}

/// Refusal `stop` placed at the declaration of random fact `target`, its input's or its
/// def's line (refcore's `info.decl.loc`). L §5.16, §8.3.
fn at_decl(m: &Reality, files: &Files, target: &Node, stop: Stop) -> Result<Stop, Stop> {
    let (word, line) = if target.kind == NodeKind::Def {
        ("def", m.defs.get(&target.id).map(|def| def.decl.line))
    } else {
        (
            "input",
            m.inputs.get(&target.id).and_then(|input| input.line),
        )
    };
    let Some(line) = line else {
        return Ok(stop);
    };
    let scope = Scope::new(
        target.clone(),
        Stands::Decl,
        files.get(word, &target.id)?,
        line,
    );
    Ok(scope.place(stop))
}

/// The type of a random fact's sample: `num u`, or a key of the index its `discrete`
/// ranges over (refcore's `sample_type`). L §5.16 Samples.
pub(super) fn sample_type(m: &Reality, files: &Files, target: &Node, ty: &Ty) -> Typed {
    if let Some(ix) = discrete_index(m, files, target)? {
        return Ok(Ty::Key(ix));
    }
    Ok(num(unit_of(ty)?))
}

/// Whether `e` reads an input or a def declared `dist` directly (refcore's `is_x`, its read
/// arm).
fn reads_dist_fact(m: &Reality, scope: &Scope, e: &Expr) -> bool {
    let base = if let Expr::Post(base, _) = e {
        &**base
    } else {
        e
    };
    let Expr::Name(name) = base else {
        return false;
    };
    if scope.is_var(name) {
        return false;
    }
    let kind = m.names.kind(name);
    if kind == Some(NameKind::Input) {
        return m
            .inputs
            .get(name)
            .is_some_and(|i| matches!(i.ty, Some(Ty::Dist(_))));
    }
    kind == Some(NameKind::Def)
        && m.defs
            .get(name)
            .is_some_and(|d| matches!(d.ty, Ty::Dist(_)))
}

/// Whether `e` is a random value X: a distribution, one under `to u`, or a random fact read
/// directly (refcore's `is_x`).
fn is_x(m: &Reality, scope: &Scope, e: &Expr) -> bool {
    let e = strip(e);
    if matches!(e, Expr::Dist(_)) {
        return true;
    }
    if let Expr::To(inner, _) = e
        && matches!(strip(inner), Expr::Dist(_))
    {
        return true;
    }
    reads_dist_fact(m, scope, e)
}

/// Whether some node of `e` is a random value X (refcore's `contains_x`), an explicit stack.
fn contains_x(m: &Reality, scope: &Scope, e: &Expr) -> bool {
    let mut stack = vec![e];
    while let Some(x) = stack.pop() {
        if is_x(m, scope, x) {
            return true;
        }
        stack.extend(super::cone::all_children(x));
    }
    false
}

/// The X of an affine form `a * X + b` in exactly one random value, or `None` (refcore's
/// `dist_affine_syntax`). L §5.16.
pub(super) fn affine_x(m: &Reality, scope: &Scope, node: &Expr) -> Option<Expr> {
    let mut e = node;
    loop {
        e = strip(e);
        if is_x(m, scope, e) {
            return Some(e.clone());
        }
        let terms = chain_terms(e);
        let at = one_side(m, scope, &terms)?;
        e = terms.get(at)?.1;
    }
}

/// The terms of a negation, a sum or a product, each marked when it divides; none for
/// another form.
fn chain_terms(e: &Expr) -> Vec<(bool, &Expr)> {
    let mut terms: Vec<(bool, &Expr)> = Vec::new();
    if let Expr::Neg(inner) = e {
        terms.push((false, inner));
    } else if let Expr::Sum(first, rest) = e {
        terms.push((false, first));
        terms.extend(rest.iter().map(|(_, t)| (false, t)));
    } else if let Expr::Prod(first, rest) = e {
        terms.push((false, first));
        let div = crate::syntax::MulOp::Div;
        terms.extend(rest.iter().map(|(op, t)| (*op == div, t)));
    }
    terms
}

/// Of a chain's terms, the position of the one that holds X when exactly one does and it
/// divides nothing: refcore's left-nested `+`, `-`, `*` (one side holds X) and `/` (the
/// left side only).
fn one_side(m: &Reality, scope: &Scope, terms: &[(bool, &Expr)]) -> Option<usize> {
    let mut holding = Vec::new();
    for (at, (divides, term)) in terms.iter().enumerate() {
        if contains_x(m, scope, term) {
            holding.push((at, *divides));
        }
    }
    match holding.as_slice() {
        [(at, false)] => Some(*at),
        _ => None,
    }
}

impl Checker {
    /// A read of a random fact: its distribution where one is taken, else its sample on the
    /// path, the reader random by lifting (refcore's `dist_read_type`). L §5.16 Samples.
    pub(super) fn dist_read(
        &mut self,
        m: &Reality,
        scope: &Scope,
        target: &Node,
        ty: &Ty,
        read: SampleOf,
    ) -> Typed {
        if read == SampleOf::AffineX {
            self.graph.rand_edge(&scope.item, target, ReadRole::Dist);
            return sample_type(m, &self.files, target, ty);
        }
        let role = match scope.read_in {
            ReadIn::Query => ReadRole::Query,
            ReadIn::Dist => ReadRole::Dist,
            ReadIn::Plain => ReadRole::Sample,
        };
        self.graph.rand_edge(&scope.item, target, role);
        if role == ReadRole::Dist {
            return Ok(ty.clone());
        }
        sample_type(m, &self.files, target, ty)
    }

    /// A plain read of an input or a def, recorded for the random cone (refcore's
    /// `note_read`). L §5.16 lifting.
    pub(super) fn note_read(&mut self, scope: &Scope, target: &Node) {
        let role = if scope.read_in == ReadIn::Query {
            ReadRole::Query
        } else {
            ReadRole::Value
        };
        self.graph.rand_edge(&scope.item, target, role);
    }

    /// A distribution: `discrete` over an index's keys, or one of the closed list with its
    /// parameters' units (refcore's `t_dist`). L §5.16.
    pub(super) fn dist(
        &mut self,
        m: &Reality,
        scope: &Scope,
        dist: &Dist,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        self.under_to = false;
        let affx = core::mem::take(&mut self.affx_hit);
        let d = dist;
        let want = if let Some(Ty::Dist(unit)) = expect {
            Some(unit.clone())
        } else {
            None
        };
        if d.func == DistKind::Discrete {
            return self.discrete(m, scope, d, depth);
        }
        let name = dist_word(d.func);
        let arity: usize = match d.func {
            DistKind::Exponential | DistKind::Poisson => 1,
            DistKind::Normal
            | DistKind::Lognormal
            | DistKind::Gamma
            | DistKind::Uniform
            | DistKind::Binomial
            | DistKind::Negbin
            | DistKind::Discrete => 2,
        };
        if d.args.len() != arity {
            return Err(refused(
                Load::Parse,
                format!("cannot parse: {name} takes {arity} parameters"),
            ));
        }
        let mut ts = Vec::new();
        for arg in &d.args {
            ts.push(self.expr(m, &scope.child(), arg, None, depth)?);
        }
        let missing = || invariant(format!("{name} has fewer parameters"));
        let first = ts.first().ok_or_else(missing)?;
        let fallback = if let Some(w) = &want {
            w.clone()
        } else {
            Unit::one()
        };
        let unit = match d.func {
            DistKind::Normal | DistKind::Uniform => {
                let second = ts.get(1).ok_or_else(missing)?;
                let lo = param(m, name, first, "the first parameter")?;
                let hi = param(m, name, second, "the second parameter")?;
                if let (Some(lo), Some(hi)) = (&lo, &hi)
                    && !converts(lo, hi)?
                {
                    return Err(uerr(format!(
                        "{name}({}, {}): the parameters have different units",
                        first.show(),
                        second.show()
                    )));
                }
                match (lo, hi) {
                    (Some(u), _) | (None, Some(u)) => u,
                    (None, None) => fallback,
                }
            }
            DistKind::Lognormal => {
                let second = ts.get(1).ok_or_else(missing)?;
                plain_param(m, name, first, "m")?;
                plain_param(m, name, second, "s")?;
                fallback
            }
            DistKind::Exponential => match param(m, name, first, "the rate")? {
                Some(r) if !r.is_one() => r.inv().map_err(super::steps::num_stop)?,
                Some(_) | None => fallback,
            },
            DistKind::Gamma => {
                let second = ts.get(1).ok_or_else(missing)?;
                plain_param(m, name, first, "the shape")?;
                match param(m, name, second, "the scale")? {
                    Some(u) => u,
                    None => fallback,
                }
            }
            DistKind::Poisson | DistKind::Binomial | DistKind::Negbin | DistKind::Discrete => {
                for t in &ts {
                    plain_param(m, name, t, "a parameter")?;
                }
                if let Some(w) = want.as_ref().filter(|w| !w.is_one()) {
                    return Err(uerr(format!(
                        "{name} gives counts, not a dist {}",
                        w.text()
                    )));
                }
                Unit::one()
            }
        };
        if let Some(w) = &want
            && !converts(&unit, w)?
        {
            return Err(uerr(format!(
                "{name} of {} for a dist {}",
                unit_text(&unit),
                unit_text(w)
            )));
        }
        let unit = want.unwrap_or(unit);
        if affx {
            return Ok(num(unit));
        }
        Ok(Ty::Dist(unit))
    }

    /// `discrete(w for k in IDX)`: over an index's keys in index order, a plain weight
    /// (refcore's `t_dist`, its discrete arm). L §5.16.
    fn discrete(&mut self, m: &Reality, scope: &Scope, d: &Dist, depth: u32) -> Typed {
        let (Some(bind), Some(term)) = (&d.bind, &d.term) else {
            return Err(invariant(
                "a discrete distribution with no binding".to_owned(),
            ));
        };
        let mut inner = scope.in_aggregate();
        self.bind(m, &mut inner, bind, depth)?;
        let dom = inner.vars.get(&bind.var).map(|v| v.dom.clone());
        let unordered = match &dom {
            Some(VarDom::Kind(k) | VarDom::Event(k) | VarDom::Obs(k)) => {
                Some(format!("the members of {k}"))
            }
            Some(VarDom::Index(ix))
                if m.indexes.get(ix).is_some_and(|i| i.form == IndexForm::Open) =>
            {
                Some(format!("the keys of the open index {ix}"))
            }
            Some(
                VarDom::Index(_) | VarDom::Axis(_) | VarDom::List | VarDom::Range | VarDom::Arg,
            )
            | None => None,
        };
        if let Some(what) = unordered {
            return Err(uerr(format!(
                "discrete over {what}: its quantile walks the keys in index order, and they have none"
            )));
        }
        let tt = self.expr(m, &inner, term, None, depth)?;
        if !plain(m, &tt)? {
            return Err(uerr(format!(
                "discrete: the weight is {}, not a plain number",
                tt.show()
            )));
        }
        let Some(VarDom::Index(ix)) = dom else {
            return Err(refused(
                Load::Domain,
                "discrete over a key list or a stepped range".to_owned(),
            ));
        };
        Ok(Ty::DistKeys(ix))
    }

    /// `cdf`, `pdf` or `pmf` of a distribution, a random fact read directly, or an affine
    /// form of one, at a point (refcore's `t_distfn`). L §5.16.
    pub(super) fn dist_fn(&mut self, m: &Reality, scope: &Scope, f: &DistFn, depth: u32) -> Typed {
        let under_to = Under::take(&mut self.under_to);
        let name = fn_word(f.func);
        let Some(found) = affine_x(m, scope, &f.value) else {
            return Err(refused(
                Load::Random,
                format!(
                    "{name} of a non-affine random value: it takes a distribution, a random fact read directly or an affine form of one; the question is a share over paths"
                ),
            ));
        };
        let x = &found;
        let mut xt = self.expr(m, &scope.dist_scope(), x, None, depth)?;
        if !matches!(xt, Ty::Dist(_) | Ty::DistKeys(_) | Ty::Unseen) {
            return Err(uerr(format!("{name} of {}", xt.show())));
        }
        if matches!(xt, Ty::Dist(_))
            && let Some(target) = read_target(m, scope, x)
            && let Some(ix) = discrete_index(m, &self.files, &target)?
        {
            xt = Ty::DistKeys(ix);
        }
        // X is the stripped value or a node inside it, so equal means X is the whole value.
        let whole = strip(&f.value) == x;
        if let Ty::DistKeys(ix) = &xt {
            if !whole {
                return Err(uerr(format!(
                    "{name} of an affine form of a distribution over keys"
                )));
            }
            if f.func == DistFnKind::Pdf {
                return Err(uerr("pdf of a discrete distribution".to_owned()));
            }
            let key = Ty::Key(ix.clone());
            let yt = self.expr(m, scope, &f.at, Some(&key), depth)?;
            self.assign(m, scope, &key, &f.at, &yt, &format!("the point of {name}"))?;
            return Ok(num(Unit::one()));
        }
        let unit = if whole {
            unit_of(&xt)?
        } else {
            self.aff_x = Some(x.clone());
            let typed = self.expr(m, scope, &f.value, None, depth);
            self.aff_x = None;
            self.affx_hit = false;
            let t = typed?;
            if !matches!(numcls(m, &t), Some("num" | "money" | "int" | "flex"))
                && !matches!(t, Ty::Unseen)
            {
                return Err(uerr(format!("{name} of {}", t.show())));
            }
            num_unit(m, &t)?
        };
        let yt = self.expr(m, scope, &f.at, Some(&num(unit.clone())), depth)?;
        if !matches!(yt, Ty::Flex(_)) {
            let yu = unit_or(m, &yt, &format!("{name}: the point"))?;
            if !matches!(yt, Ty::Unseen) && !converts(&yu, &unit)? {
                return Err(uerr(format!(
                    "{name} at {} for a value of unit {}",
                    yt.show(),
                    unit_text(&unit)
                )));
            }
        }
        if let Expr::Dist(d) = strip_to(x) {
            let needs_to = match f.func {
                DistFnKind::Cdf => matches!(
                    d.func,
                    DistKind::Normal
                        | DistKind::Lognormal
                        | DistKind::Exponential
                        | DistKind::Gamma
                        | DistKind::Poisson
                ),
                DistFnKind::Pdf => matches!(
                    d.func,
                    DistKind::Normal
                        | DistKind::Lognormal
                        | DistKind::Exponential
                        | DistKind::Gamma
                ),
                DistFnKind::Pmf => d.func == DistKind::Poisson,
            };
            if needs_to && under_to == Under::Bare {
                return Err(refused(
                    Load::To,
                    format!("{name} of {} needs to u", dist_word(d.func)),
                ));
            }
            let counted = matches!(
                d.func,
                DistKind::Poisson | DistKind::Binomial | DistKind::Negbin | DistKind::Discrete
            );
            if f.func == DistFnKind::Pdf && counted {
                return Err(uerr(format!(
                    "pdf of the discrete {}: a sample has no density",
                    dist_word(d.func)
                )));
            }
            if f.func == DistFnKind::Pmf && continuous(d.func) {
                return Err(uerr(format!("pmf of the continuous {}", dist_word(d.func))));
            }
        }
        if f.func == DistFnKind::Pdf {
            self.orient_items.insert(scope.item.clone());
            return Ok(num(unit.inv().map_err(super::steps::num_stop)?));
        }
        Ok(num(Unit::one()))
    }

    /// `mean`, `share` or `quantile`: over keys with `for`, else over a random value or of a
    /// distribution value (refcore's `t_query`, `t_query_random`). L §5.11, §5.16.
    pub(super) fn query(&mut self, m: &Reality, scope: &Scope, q: &Query, depth: u32) -> Typed {
        let under_to = Under::take(&mut self.under_to);
        let Some(binds) = &q.binds else {
            if matches!(strip_to(&q.arg), Expr::Dist(_)) {
                return self.dist_value_query(m, scope, q, under_to, depth);
            }
            return self.query_random(m, scope, q, depth);
        };
        let mut inner = scope.in_aggregate();
        for bind in binds {
            self.bind(m, &mut inner, bind, depth)?;
        }
        let tt = self.expr(m, &inner, &q.arg, None, depth)?;
        if matches!(tt, Ty::Unseen) {
            return Ok(tt);
        }
        match q.func {
            QueryFn::Share => {
                if !is_bool(&tt) {
                    return Err(uerr(format!("share of {}", tt.show())));
                }
                Ok(num(Unit::one()))
            }
            QueryFn::Mean => mean_type(m, &tt),
            QueryFn::Quantile => {
                self.level(m, scope, q, LevelUnit::Any, depth)?;
                Ok(tt)
            }
        }
    }

    /// A quantile's level: a number, plain where `plain`; a figure allowed in an ask.
    fn level(
        &mut self,
        m: &Reality,
        scope: &Scope,
        q: &Query,
        unit: LevelUnit,
        depth: u32,
    ) -> Result<(), Stop> {
        let Some(level) = &q.level else {
            return Err(invariant("a quantile with no level".to_owned()));
        };
        let mut levelled = scope.child();
        levelled.allow_lit = scope.stands == Stands::Ask;
        let qt = self.expr(m, &levelled, level, None, depth)?;
        let fits = match unit {
            LevelUnit::Any => {
                matches!(numcls(m, &qt), Some("num" | "flex" | "int")) || matches!(qt, Ty::Unseen)
            }
            LevelUnit::Plain => plain(m, &qt)?,
        };
        if !fits {
            return Err(uerr(format!("the level of quantile is {}", qt.show())));
        }
        Ok(())
    }

    /// A query over a random value: its argument and `where` read inside the query,
    /// recorded for the closed forms (refcore's `t_query_random`). L §5.16.
    fn query_random(&mut self, m: &Reality, scope: &Scope, q: &Query, depth: u32) -> Typed {
        let name = query_word(q.func);
        let mut inner = scope.child();
        inner.read_in = ReadIn::Query;
        let tt = self.expr(m, &inner, &q.arg, None, depth)?;
        if let Some(filter) = &q.filter {
            let wt = self.expr(m, &inner, filter, None, depth)?;
            if !is_bool(&wt) && !matches!(wt, Ty::Unseen) {
                return Err(uerr(format!("where of {name} is {}, not bool", wt.show())));
            }
        }
        self.queries.push(QueryAt {
            form: Expr::Query(Box::new(q.clone())),
            item: scope.item.clone(),
            stands: scope.stands,
            at: scope.file_line(),
        });
        if matches!(tt, Ty::Unseen) {
            return Ok(tt);
        }
        match q.func {
            QueryFn::Share => {
                if !is_bool(&tt) {
                    return Err(uerr(format!("share of {}", tt.show())));
                }
                Ok(num(Unit::one()))
            }
            QueryFn::Mean => mean_type(m, &tt),
            QueryFn::Quantile => {
                self.level(m, scope, q, LevelUnit::Plain, depth)?;
                let c = numcls(m, &tt);
                if c.is_none() && !matches!(tt, Ty::Key(_)) {
                    return Err(uerr(format!("quantile of {}", tt.show())));
                }
                if c == Some("money") {
                    return Ok(num(unit_of(&tt)?));
                }
                if matches!(c, Some("int" | "flex")) {
                    return Ok(num(Unit::one()));
                }
                Ok(tt)
            }
        }
    }

    /// `quantile(d, q)` of a distribution value, its function, exact or under `to u`;
    /// `mean`, `share` and `where` refused there (refcore's `dist_value_query`). L §5.16.
    fn dist_value_query(
        &mut self,
        m: &Reality,
        scope: &Scope,
        q: &Query,
        under_to: Under,
        depth: u32,
    ) -> Typed {
        if q.func == QueryFn::Mean {
            return Err(refused(
                Load::Random,
                "mean of a distribution value; ask it of a random fact".to_owned(),
            ));
        }
        if q.func == QueryFn::Share {
            return Err(refused(
                Load::Random,
                "share of a distribution value: share takes a random bool".to_owned(),
            ));
        }
        if q.filter.is_some() {
            return Err(refused(
                Load::Random,
                "quantile of a distribution value with where: where restricts a query over paths"
                    .to_owned(),
            ));
        }
        let dt = self.expr(m, &scope.dist_scope(), &q.arg, None, depth)?;
        if !matches!(dt, Ty::Dist(_) | Ty::DistKeys(_) | Ty::Unseen) {
            return Err(uerr(format!("quantile of {}", dt.show())));
        }
        self.level(m, scope, q, LevelUnit::Plain, depth)?;
        if let Ty::DistKeys(ix) = dt {
            return Ok(Ty::Key(ix));
        }
        if let Expr::Dist(d) = strip_to(&q.arg)
            && matches!(
                d.func,
                DistKind::Normal | DistKind::Lognormal | DistKind::Exponential | DistKind::Gamma
            )
            && under_to == Under::Bare
        {
            return Err(refused(
                Load::To,
                format!("quantile of {} needs to u", dist_word(d.func)),
            ));
        }
        Ok(num(unit_of(&dt)?))
    }

    /// `crps`: of a normal forecast in closed form, of a random value over paths, or of a
    /// sample with or without weights; the outcome of the forecast's unit (refcore's
    /// `t_crps`). L §5.15.
    pub(super) fn crps(&mut self, m: &Reality, scope: &Scope, c: &Crps, depth: u32) -> Typed {
        let under_to = Under::take(&mut self.under_to);
        self.orient_items.insert(scope.item.clone());
        let number = |t: &Ty| {
            matches!(numcls(m, t), Some("num" | "int" | "flex" | "money"))
                || matches!(t, Ty::Unseen)
        };
        let unit = if let Some(bind) = &c.bind {
            let mut inner = scope.in_aggregate();
            self.bind(m, &mut inner, bind, depth)?;
            let et = self.expr(m, &inner, &c.forecast, None, depth)?;
            if !number(&et) {
                return Err(uerr(format!("crps of {}", et.show())));
            }
            if let Some(weight) = &c.weight {
                let wt = self.expr(m, &inner, weight, None, depth)?;
                if !plain(m, &wt)? {
                    return Err(uerr(format!(
                        "crps: the weight is {}, not a plain number",
                        wt.show()
                    )));
                }
            }
            num_unit(m, &et)?
        } else if let Expr::Dist(d) = strip_to(&c.forecast) {
            if d.func != DistKind::Normal {
                return Err(uerr(format!(
                    "crps of {}: the closed form is for a normal forecast (L §5.15)",
                    dist_word(d.func)
                )));
            }
            let dt = self.expr(m, &scope.dist_scope(), strip(&c.forecast), None, depth)?;
            if under_to == Under::Bare {
                return Err(refused(
                    Load::To,
                    "crps of a normal forecast needs to u".to_owned(),
                ));
            }
            unit_of(&dt)?
        } else {
            let mut inner = scope.child();
            inner.read_in = ReadIn::Query;
            let et = self.expr(m, &inner, &c.forecast, None, depth)?;
            if !number(&et) {
                return Err(uerr(format!("crps of {}", et.show())));
            }
            self.queries.push(QueryAt {
                form: Expr::Crps(Box::new(c.clone())),
                item: scope.item.clone(),
                stands: scope.stands,
                at: scope.file_line(),
            });
            num_unit(m, &et)?
        };
        let yt = self.expr(m, scope, &c.outcome, Some(&num(unit.clone())), depth)?;
        if !matches!(yt, Ty::Flex(_) | Ty::Unseen) {
            let yu = unit_or(m, &yt, "crps: the outcome")?;
            if !converts(&yu, &unit)? {
                return Err(uerr(format!(
                    "crps: the outcome {} and the forecast ({}) have different units",
                    yt.show(),
                    unit.text()
                )));
            }
        }
        Ok(num(unit))
    }

    /// Each row of a `dist` input that states a distribution: a distribution of the input's
    /// unit, a continuous one under `to u` (refcore's `check_dist_rows`). L §5.16, §5.3.
    ///
    /// # Errors
    ///
    /// `LOAD-UNIT` or `LOAD-TO` at the row's line.
    pub(super) fn check_dist_rows(&mut self, m: &Reality, files: &Files) -> Result<(), Stop> {
        for (id, input) in m.inputs.entries() {
            let Some(declared @ Ty::Dist(want)) = &input.ty else {
                continue;
            };
            for row in &input.rows {
                let Value::Dist(node) = &row.value else {
                    continue;
                };
                let file = files.get("version", &row.version)?;
                let mut scope =
                    Scope::new(Node::new(NodeKind::Input, id), Stands::Decl, file, row.line);
                scope.allow_lit = true;
                scope.read_in = ReadIn::Dist;
                let t = self.expr(m, &scope, node, Some(declared), 0)?;
                let fits = if let Ty::Dist(unit) = &t {
                    converts(unit, want)?
                } else {
                    (matches!(t, Ty::DistKeys(_)) && want.is_one()) || matches!(t, Ty::Unseen)
                };
                if !fits {
                    let message =
                        format!("a row of {id} gives {}, not {}", t.show(), declared.show());
                    return Err(scope.place(uerr(message)));
                }
                dist_whole(node, &format!("a row of {id}")).map_err(|stop| scope.place(stop))?;
            }
        }
        Ok(())
    }
}

/// A distribution's parameter: its unit, none for an int or a bare figure (refcore's
/// `t_dist`, its `unit_of`). L §5.16.
fn param(m: &Reality, name: &str, t: &Ty, what: &str) -> Result<Option<Unit>, Stop> {
    let c = numcls(m, t);
    if matches!(c, Some("flex" | "int")) || matches!(t, Ty::Unseen) {
        return Ok(None);
    }
    if matches!(c, Some("num" | "money")) {
        return unit_of(t).map(Some);
    }
    Err(uerr(format!(
        "{name}: {what} is {}, not a number",
        t.show()
    )))
}

/// A distribution's parameter that is a plain number (refcore's `t_dist`, its `plain`).
/// L §5.16.
fn plain_param(m: &Reality, name: &str, t: &Ty, what: &str) -> Result<(), Stop> {
    if param(m, name, t, what)?.is_some_and(|u| !u.is_one()) {
        return Err(uerr(format!(
            "{name}: {what} is {}, not a plain number",
            t.show()
        )));
    }
    Ok(())
}

/// How a random fact's read is typed: as the X of an affine form under `cdf`, `pdf` or
/// `pmf` (its distribution read, typed as its values), or by its scope's role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SampleOf {
    /// The X of an affine form (refcore's `_affx`).
    AffineX,
    /// A read by its scope's role.
    Scope,
}

/// Whether a quantile's level is any number, or a plain one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelUnit {
    /// Any number (a query over keys).
    Any,
    /// A plain number (a query over a random value).
    Plain,
}

/// The type of `mean` of a value of type `t` (refcore's `t_query`, its mean arm). L §5.16.
fn mean_type(m: &Reality, t: &Ty) -> Typed {
    let c = numcls(m, t);
    if c.is_none() {
        return Err(uerr(format!("mean of {}", t.show())));
    }
    if matches!(c, Some("int" | "flex")) {
        return Ok(num(Unit::one()));
    }
    if c == Some("money") {
        return Ok(num(unit_of(t)?));
    }
    Ok(t.clone())
}

/// The input or def `e` reads, if it reads one directly (refcore's `_r` of a read node).
fn read_target(m: &Reality, scope: &Scope, e: &Expr) -> Option<Node> {
    let base = if let Expr::Post(base, _) = strip(e) {
        &**base
    } else {
        strip(e)
    };
    let Expr::Name(name) = base else {
        return None;
    };
    if scope.is_var(name) {
        return None;
    }
    let kind = m.names.kind(name)?;
    if kind == NameKind::Input {
        return Some(Node::new(NodeKind::Input, name));
    }
    (kind == NameKind::Def).then(|| Node::new(NodeKind::Def, name))
}
