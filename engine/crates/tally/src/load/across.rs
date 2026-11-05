//! Load's period across keys: a period load sees, read at a key of a time index, refused where it lies across two of the index's keys.
//! Spec: L §4.6 (a period no longer than the keys that no key's period holds is refused at
//! load where load sees it), §8.3 (`LOAD-KEY` `<period> lies across keys <a> and <b> of
//! <index>`; item 5, the keys of a declared index; item 9, what load sees).
//! Twin: engine/tools/refcore/check.py (`period_conv`, its `period_cands`); check3.py
//! (`m_check_period_keys`, `m_seen_day`, `FOLD_FNS`).
//! Invariants: a key is folded only where load sees it, from literals and the reader's own
//! keys of declared indexes through built-ins that read no fact; a fold the evaluator
//! refuses is left to the value, as refcore leaves it.

use super::steps::refuse;
use crate::base::{Code, Load};
use crate::eval::{Ctx, Engine, Env};
use crate::model::{
    BindInfo, Domain, IndexForm, IndexInfo, KeyVal, Node, NodeKind, Reality, Value,
};
use crate::syntax::{Expr, Lit, Stop};
use crate::time::Period;

/// The built-ins a seen day is folded through: they read no fact (check3.py's `FOLD_FNS`).
const FOLD_FNS: [&str; 9] = [
    "end_of",
    "start_of",
    "day_of",
    "add_days",
    "add_months",
    "add_years",
    "month_of",
    "year_of",
    "quarter_of",
];

/// A period read at a key of a time index, folded once the walk is done (refcore's
/// `period_cands` entry).
#[derive(Debug, Clone)]
pub(super) struct PeriodRead {
    /// The time index.
    pub(super) index: String,
    /// The key as written.
    pub(super) key: Expr,
    /// The item that reads it.
    pub(super) reader: Node,
    /// The file and line it is written on.
    pub(super) at: (String, u32),
}

/// `LOAD-KEY` for each period load sees that lies across two keys of its index, folded at
/// every key of the reader's bindings in their order (refcore's `m_check_period_keys`).
/// L §4.6, §8.3.
///
/// # Errors
///
/// The first such period's `LOAD-KEY`; an invariant or `NotBuilt` the fold meets.
pub(super) fn check_period_keys(m: &Reality, reads: &[PeriodRead]) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for read in reads {
        let Some(index) = m.indexes.get(&read.index) else {
            continue;
        };
        let Some(keys) = period_keys(index) else {
            continue;
        };
        let Some(binds) = reader_binds(m, &read.reader) else {
            continue;
        };
        if !seen_day(m, &read.key, &binds) {
            continue;
        }
        let Some(domains) = bind_keys(m, &binds) else {
            continue;
        };
        for env in products(&binds, &domains) {
            let Some(period) = fold(&mut engine, m, &read.key, &env)? else {
                continue;
            };
            across(&keys, period, read)?;
        }
    }
    Ok(())
}

/// The keys of an index that isn't open, when each is a period (refcore's `isinstance(k,
/// Period)` over `ix.keys`). L §4.1.
fn period_keys(index: &IndexInfo) -> Option<Vec<Period>> {
    if is_open(index) || index.keys.is_empty() {
        return None;
    }
    let mut keys = Vec::new();
    for key in &index.keys {
        let KeyVal::Period(period) = key else {
            return None;
        };
        keys.push(*period);
    }
    Some(keys)
}

/// Whether an index's keys come from rows, which load doesn't see: an open index, or the
/// built-in `bank_account`, the one built-in index of no time and no integers (refcore's
/// `form == 'open'`). L §4.1, §8.3 item 5.
fn is_open(index: &IndexInfo) -> bool {
    index.form == IndexForm::Open
        || (index.form == IndexForm::Builtin && index.period.is_none() && !index.is_int)
}

/// The reader's own bindings: an ask has none, a def's and a move's are declared; another
/// reader isn't folded (refcore's `holder` cases).
fn reader_binds(m: &Reality, reader: &Node) -> Option<Vec<BindInfo>> {
    match reader.kind {
        NodeKind::Ask => Some(Vec::new()),
        NodeKind::Def => m.defs.get(&reader.id).map(|d| d.binds.clone()),
        NodeKind::Move | NodeKind::MoveOn => m.moves.get(&reader.id).map(|mv| mv.binds.clone()),
        NodeKind::Input
        | NodeKind::Band
        | NodeKind::Place
        | NodeKind::Price
        | NodeKind::Balances
        | NodeKind::Parts => None,
    }
}

/// Whether load sees the day or period `e`: literals, the reader's own keys of a declared
/// index, built-ins that read no fact, and `+` and `-` over these (refcore's `m_seen_day`).
/// L §8.3 item 9.
pub(super) fn seen_day(m: &Reality, e: &Expr, binds: &[BindInfo]) -> bool {
    let mut stack = vec![e];
    while let Some(node) = stack.pop() {
        if let Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) | Lit::Num(_)) = node {
            continue;
        }
        if let Expr::Name(name) = node {
            if !declared_key(m, binds, name) {
                return false;
            }
            continue;
        }
        if let Expr::Call(call) = node
            && FOLD_FNS.contains(&call.name.as_str())
            && call.prefix.is_empty()
            && call.band_key.is_none()
        {
            stack.extend(call.args.iter());
            continue;
        }
        if let Expr::Sum(first, rest) = node {
            stack.push(first);
            stack.extend(rest.iter().map(|(_, term)| term));
            continue;
        }
        let Expr::Neg(inner) = node else {
            return false;
        };
        stack.push(inner);
    }
    true
}

/// Whether `name` is one of the reader's bound variables over a declared index that isn't
/// open. L §8.3 items 5 and 9.
fn declared_key(m: &Reality, binds: &[BindInfo], name: &str) -> bool {
    binds
        .iter()
        .any(|bind| bind.var == name && declared_keys(m, bind).is_some())
}

/// The keys of the index a binding ranges over, when it is declared and not open.
fn declared_keys(m: &Reality, bind: &BindInfo) -> Option<Vec<KeyVal>> {
    let Domain::Index(ix) = &bind.domain else {
        return None;
    };
    m.indexes
        .get(ix)
        .filter(|index| !is_open(index))
        .map(|index| index.keys.clone())
}

/// The keys of each binding's index, in the bindings' order, or `None` when one ranges over
/// what load doesn't see.
fn bind_keys(m: &Reality, binds: &[BindInfo]) -> Option<Vec<Vec<KeyVal>>> {
    binds.iter().map(|bind| declared_keys(m, bind)).collect()
}

/// Every choice of one key per binding, the last binding's key turning fastest (Python's
/// `itertools.product`), each as the scope it binds.
fn products(binds: &[BindInfo], domains: &[Vec<KeyVal>]) -> Vec<Env> {
    let mut out = vec![Env::empty()];
    for (bind, keys) in binds.iter().zip(domains) {
        let mut next = Vec::new();
        for env in &out {
            for key in keys {
                next.push(env.with(&bind.var, Value::of_key(key), Some(bind.domain.clone())));
            }
        }
        out = next;
    }
    out
}

/// The key's value at one choice of keys, when it is a period; a refusal leaves it to the
/// value at run (refcore's `except TallyError: continue`). E §3.2.
fn fold(engine: &mut Engine, m: &Reality, key: &Expr, env: &Env) -> Result<Option<Period>, Stop> {
    match engine.ev(m, key, env, Ctx::new(None, m.on)) {
        Ok(Value::Period(period)) => Ok(Some(period)),
        Ok(_) => Ok(None),
        Err(Stop::Refused(r)) if r.code() != Code::EngineInvariant => Ok(None),
        Err(stop) => Err(stop),
    }
}

/// `LOAD-KEY` when no key holds `period`, two keys meet it, and some key is no shorter
/// (refcore's `m_check_period_keys`, its test). L §4.6.
fn across(keys: &[Period], period: Period, read: &PeriodRead) -> Result<(), Stop> {
    let held = keys
        .iter()
        .any(|k| k.start() <= period.start() && period.end() <= k.end());
    if held {
        return Ok(());
    }
    let over: Vec<&Period> = keys
        .iter()
        .filter(|k| k.start() < period.end() && period.start() < k.end())
        .collect();
    let [first, second, ..] = over.as_slice() else {
        return Ok(());
    };
    let long = length(period)?;
    let mut shorter = true;
    for key in keys {
        shorter = shorter && length(*key)? < long;
    }
    if shorter {
        return Ok(());
    }
    let (file, line) = &read.at;
    Err(refuse(
        Load::Key,
        format!(
            "{} lies across keys {} and {} of {}",
            period.label(),
            first.label(),
            second.label(),
            read.index
        ),
        file,
        *line,
    ))
}

/// A period's days, `end - start`.
fn length(period: Period) -> Result<i64, Stop> {
    period
        .start()
        .days_to(period.end())
        .ok_or_else(|| super::check::invariant("a period's days past an i64".to_owned()))
}
