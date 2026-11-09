//! An ask's fan-out over the axes it reaches: every combination of their options evaluated, the combinations that give one result printed as one line, each line a product of options.
//! Spec: L §5.5 (Fan-out: an ask evaluates every combination of the axes it reaches, one line
//! per distinct result; an axis prints `*` when every option of it gives the line's result with
//! the line's other options held, else the options that give it joined by ` | `, with its
//! `reading:` entry; a set that isn't a product is split by the axes in id order; `with a =
//! opt` fixes an axis, which then doesn't fan out), §5.18 (one line per binding tuple and per
//! distinct result across the combinations where the tuple is bound; a tuple whose `where` is
//! a hole prints its line as that hole, its value unevaluated), §8.2 (`reading:`); E §3.2 (the
//! axis options in a cell's context, trimmed to the axes the item reaches), §3.3 (a timeline
//! per combination of the axes the moves reach).
//! Twin: engine/tools/refcore/ask.py (`_answer_in` with axes, `binds_over_axes`, `group`,
//! `split`, `print_line`).
//! Invariants: the combinations run in refcore's order, the first axis slowest; two
//! combinations give one result when their lines print the same text; a tuple first bound in a
//! later combination stands after the tuple bound before it there; every line names a product
//! of options, and the lines of a tuple are sorted by each axis's first option, `*` first,
//! ties in the order made; the split is a loop over an explicit stack (R-16); a combination's
//! layer is open only while its combination is evaluated.

use std::collections::BTreeSet;

use super::answer::{env_text, form_line};
use super::check::Printed;
use super::register::{check_register_lines, register_lines};
use crate::base::{Code, Refusal};
use crate::eval::{Collected, Ctx, Engine, Entry, Env, Member, combinations};
use crate::hole::{Hole, spread};
use crate::model::{Reality, Value};
use crate::syntax::{Ask, Stop};
use crate::time::Day;

/// The axes an ask fans out over, in id order, each with its options' names in the order
/// written (refcore's `a._axes` and `m.axes[ax].options`). L §5.5.
pub(super) struct Axes {
    /// Each axis's id and its options.
    axes: Vec<(String, Vec<String>)>,
}

/// One combination of the axes' options: each axis's option by its position, the axes in id
/// order.
type Combo = Vec<usize>;

/// Each axis's options a line names, by their positions, the axes in id order: one product.
type Product = Vec<BTreeSet<usize>>;

impl Axes {
    /// The axes ask `ask` reaches that its pins don't fix, as load counted them (`LOAD-FORKS`);
    /// none when it reaches none. L §5.5.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an axis load didn't build.
    pub(super) fn of(m: &Reality, ask: &Ask) -> Result<Option<Axes>, Stop> {
        let Some(ids) = m.graph.ask_axes.get(&ask.id) else {
            return Ok(None);
        };
        if ids.is_empty() {
            return Ok(None);
        }
        let mut axes = Vec::with_capacity(ids.len());
        for id in ids {
            let axis = m
                .axes
                .get(id)
                .ok_or_else(|| invariant("an axis load didn't build"))?;
            let options = axis.options.iter().map(|(name, _)| name.clone()).collect();
            axes.push((id.clone(), options));
        }
        Ok(Some(Axes { axes }))
    }

    /// Every combination of the options, the first axis slowest (refcore's `combos`), at most
    /// load's 64 (`LOAD-FORKS`).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for more combinations than load lets through.
    fn combinations(&self) -> Result<Vec<Combo>, Stop> {
        let counts: Vec<usize> = self.axes.iter().map(|(_, options)| options.len()).collect();
        combinations(&counts, 64).ok_or_else(|| invariant("more combinations than load's 64"))
    }

    /// Combination `combo` as each axis's id with its option's name.
    fn options_of(&self, combo: &[usize]) -> Result<Vec<(String, String)>, Stop> {
        let mut out = Vec::with_capacity(combo.len());
        for ((id, options), at) in self.axes.iter().zip(combo) {
            let option = options
                .get(*at)
                .ok_or_else(|| invariant("a combination's option its axis lacks"))?;
            out.push((id.clone(), option.clone()));
        }
        Ok(out)
    }

    /// A line's options as its head prints them, `a = *, b = x | y` (refcore's `print_line`).
    /// L §5.5.
    fn head(&self, product: &[BTreeSet<usize>]) -> Result<String, Stop> {
        let mut parts = Vec::with_capacity(product.len());
        for ((id, options), named) in self.axes.iter().zip(product) {
            let shown = if named.len() == options.len() {
                "*".to_owned()
            } else {
                names(options, named)?
            };
            parts.push(format!("{id} = {shown}"));
        }
        Ok(parts.join(", "))
    }

    /// A line's `reading:` entries: each axis it doesn't print `*`, with the options it
    /// names, in id order (refcore's `group`, its `reading`). L §5.5, §8.2.
    fn readings(&self, product: &[BTreeSet<usize>]) -> Result<Vec<String>, Stop> {
        let mut out = Vec::new();
        for ((id, options), named) in self.axes.iter().zip(product) {
            if named.len() != options.len() {
                out.push(format!("{id} = {}", names(options, named)?));
            }
        }
        Ok(out)
    }

    /// The key a tuple's lines are sorted by: for each axis, none for `*`, else its first
    /// option's position (refcore's `group`, its `sk`). L §5.5.
    fn order_key(&self, product: &[BTreeSet<usize>]) -> Vec<Option<usize>> {
        let mut out = Vec::with_capacity(product.len());
        for ((_, options), named) in self.axes.iter().zip(product) {
            if named.len() == options.len() {
                out.push(None);
            } else {
                out.push(named.first().copied());
            }
        }
        out
    }
}

/// The names of the options at `named` of an axis whose options are `options`, in the order
/// written, joined by ` | `.
fn names(options: &[String], named: &BTreeSet<usize>) -> Result<String, Stop> {
    let mut out = Vec::with_capacity(named.len());
    for at in named {
        let name = options
            .get(*at)
            .ok_or_else(|| invariant("a line's option its axis lacks"))?;
        out.push(name.as_str());
    }
    Ok(out.join(" | "))
}

/// One binding tuple of an ask that fans out (refcore's `per[key]`): its key and scope, the
/// combinations it is bound in, the holes of its `where` where it is one, and its line in each
/// combination once evaluated.
struct Bound {
    /// The values of its variables, in the order bound (refcore's `vkey`s).
    key: Vec<Value>,
    /// Its scope, as the first combination binding it made it.
    env: Env,
    /// The combinations it is bound in, by their place, each with the registers of the
    /// movements it binds there.
    combos: Vec<(usize, Collected)>,
    /// The holes of its `where`, in the combinations where it is one.
    unknown: Vec<Hole>,
    /// Its line in each combination it is bound in, by the combination's place.
    results: Vec<(usize, Printed)>,
}

impl Bound {
    /// A tuple of key `key` and scope `env`, bound in no combination yet.
    fn new(key: Vec<Value>, env: Env) -> Bound {
        Bound {
            key,
            env,
            combos: Vec::new(),
            unknown: Vec::new(),
            results: Vec::new(),
        }
    }
}

/// The lines of ask `ask`, which fans out over `axes`, in context `ctx` with the `with`
/// pins' register `pinned` under each: every combination evaluated, each binding tuple's lines
/// in turn (refcore's `_answer_in` with axes, `binds_over_axes`). L §5.5, §5.18.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
pub(super) fn fanned_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
    axes: &Axes,
) -> Result<Vec<String>, Stop> {
    let combos = axes.combinations()?;
    let mut options = Vec::with_capacity(combos.len());
    for combo in &combos {
        options.push(axes.options_of(combo)?);
    }
    let mut bound = if ask.binds.is_empty() {
        let mut one = Bound::new(Vec::new(), Env::empty());
        for at in 0..combos.len() {
            one.combos.push((at, Collected::default()));
        }
        vec![one]
    } else {
        match bind_tuples(m, ask, engine, ctx, &options)? {
            Err(hole) => return Ok(vec![format!("{}: {}", ask.id, hole.text())]),
            Ok(bound) => bound,
        }
    };
    for (at, set) in options.iter().enumerate() {
        let inside = engine.with_options(ctx, set)?;
        engine.open_layer(m, inside)?;
        let evaluated = evaluate_tuples(m, ask, engine, (inside, at), &mut bound);
        engine.end_layer(inside.layer, evaluated.as_ref().err())?;
        evaluated?;
    }
    let mut out = Vec::new();
    for tuple in &bound {
        out.extend(tuple_lines(m, (ask, axes, &combos), tuple, (ctx, pinned))?);
    }
    Ok(out)
}

/// The binding tuples of the ask's `for` across the combinations of `options`: a tuple first
/// bound in a later combination stands after the tuple bound before it there (refcore's
/// `binds_over_axes`, its `order.insert(prev + 1, key)`); or the hole of the first domain that
/// is one, the ask's one line. L §5.5, §5.18.
fn bind_tuples(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    ctx: Ctx,
    options: &[Vec<(String, String)>],
) -> Result<Result<Vec<Bound>, Hole>, Stop> {
    let mut bound: Vec<Bound> = Vec::new();
    for (at, set) in options.iter().enumerate() {
        let inside = engine.with_options(ctx, set)?;
        engine.open_layer(m, inside)?;
        let members = engine.tuples(m, &ask.binds, inside);
        engine.end_layer(inside.layer, members.as_ref().err())?;
        let members = match members? {
            Err(hole) => return Ok(Err(hole)),
            Ok(members) => members,
        };
        let mut before: Option<usize> = None;
        for member in members {
            let place = tuple_place(ask, &mut bound, &member.env, before)?;
            let tuple = bound
                .get_mut(place)
                .ok_or_else(|| invariant("a tuple's place past the tuples"))?;
            bound_in(tuple, at, member);
            before = Some(place);
        }
    }
    Ok(Ok(bound))
}

/// The place among `bound` of the tuple of scope `env`: where it stands, or, new, right after
/// the tuple at `before`, the first place when none (refcore's `order.insert(prev + 1, key)`).
fn tuple_place(
    ask: &Ask,
    bound: &mut Vec<Bound>,
    env: &Env,
    before: Option<usize>,
) -> Result<usize, Stop> {
    let mut key = Vec::with_capacity(ask.binds.len());
    for bind in &ask.binds {
        let var = env
            .get(&bind.var)
            .ok_or_else(|| invariant("an ask's tuple without one of its variables"))?;
        key.push(var.value.clone());
    }
    if let Some(place) = bound.iter().position(|b| b.key == key) {
        return Ok(place);
    }
    let place = match before {
        Some(at) => at
            .checked_add(1)
            .ok_or_else(|| invariant("a count of tuples"))?,
        None => 0,
    };
    let mut new = Some(Bound::new(key, env.clone()));
    for (at, tuple) in core::mem::take(bound).into_iter().enumerate() {
        if at == place
            && let Some(first) = new.take()
        {
            bound.push(first);
        }
        bound.push(tuple);
    }
    if let Some(last) = new {
        bound.push(last);
    }
    Ok(place)
}

/// Notes that `tuple` is bound in combination `at` as `member` binds it: its `where`'s hole
/// if one, and what its movements brought.
fn bound_in(tuple: &mut Bound, at: usize, member: Member) {
    if let Some(hole) = member.unknown {
        tuple.unknown.push(hole);
    }
    tuple.combos.push((at, member.brought));
}

/// Each tuple's line in the combination at `at`, of context `inside`; a tuple whose `where` is
/// a hole in some combination is left unevaluated, its line that hole (refcore's
/// `binds_over_axes`). L §5.18.
fn evaluate_tuples(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (inside, at): (Ctx, usize),
    bound: &mut [Bound],
) -> Result<(), Stop> {
    for tuple in bound.iter_mut().filter(|t| t.unknown.is_empty()) {
        let Some((_, brought)) = tuple.combos.iter().find(|(combo, _)| *combo == at) else {
            continue;
        };
        let mut line = form_line(m, ask, engine, &tuple.env, inside)?;
        line.met.merge(brought);
        tuple.results.push((at, line));
    }
    Ok(())
}

/// A tuple's lines: its `where`'s hole, every question once; or one line per product of
/// options that give one result, the register under it with the `with` pins' `pinned` and
/// the line's `reading:` entries (refcore's `binds_over_axes`, `print_line`). L §5.5, §5.18,
/// §8.2.
fn tuple_lines(
    m: &Reality,
    (ask, axes, combos): (&Ask, &Axes, &[Combo]),
    tuple: &Bound,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let head = format!("{}{}", ask.id, env_text(m, &ask.binds, &tuple.env)?);
    if !tuple.unknown.is_empty() {
        let operands: Vec<Option<&Hole>> = tuple.unknown.iter().map(Some).collect();
        let hole = spread(&operands).ok_or_else(|| invariant("a where's hole that is none"))?;
        return Ok(vec![format!("{head}: {}", hole.text())]);
    }
    let mut out = Vec::new();
    for line in grouped(axes, combos, &tuple.results)? {
        out.push(format!(
            "{head} [{}]: {}",
            axes.head(&line.product)?,
            line.text
        ));
        let mut met = line.met;
        met.merge(pinned);
        for reading in axes.readings(&line.product)? {
            met.entries.insert(Entry {
                word: "reading",
                text: reading,
                figure: String::new(),
            });
        }
        let register = match line.laws {
            Some(laws) => check_register_lines(&met, ctx, laws)?,
            None => register_lines(&met, ctx)?,
        };
        out.extend(register.into_iter().map(|l| format!("  {l}")));
    }
    Ok(out)
}

/// One line of a fan-out: the product of options it names and the result they give.
struct Grouped {
    /// Each axis's options the line names.
    product: Product,
    /// The result's text after the line's head.
    text: String,
    /// What the line's combinations met, together.
    met: Collected,
    /// A check's sides' law dates, as its first combination read them.
    laws: Option<(Option<Day>, Option<Day>)>,
}

/// The lines of one tuple's `results`, each by its combination's place in `combos`: the
/// combinations that give one result, the results in the order first met, each set split into
/// products by the axes in id order; then every line sorted by each axis's first option, `*`
/// first, the sort keeping the order made between equal keys (refcore's `group`). L §5.5.
fn grouped(
    axes: &Axes,
    combos: &[Combo],
    results: &[(usize, Printed)],
) -> Result<Vec<Grouped>, Stop> {
    let mut by_text: Vec<(&str, Vec<usize>)> = Vec::new();
    for (at, (_, printed)) in results.iter().enumerate() {
        match by_text.iter_mut().find(|(text, _)| *text == printed.text) {
            Some((_, members)) => members.push(at),
            None => by_text.push((printed.text.as_str(), vec![at])),
        }
    }
    let mut lines = Vec::new();
    for (_, members) in &by_text {
        let mut given = Vec::with_capacity(members.len());
        for at in members {
            given.push(combo_of(combos, results, *at)?);
        }
        for product in split(axes, given) {
            lines.push(product_line(product, members, (combos, results))?);
        }
    }
    lines.sort_by_key(|line| axes.order_key(&line.product));
    Ok(lines)
}

/// The combination of the result at `at` of `results`.
fn combo_of(combos: &[Combo], results: &[(usize, Printed)], at: usize) -> Result<Combo, Stop> {
    let (combo, _) = results
        .get(at)
        .ok_or_else(|| invariant("a result past the results"))?;
    combos
        .get(*combo)
        .cloned()
        .ok_or_else(|| invariant("a result's combination past the combinations"))
}

/// The line of `product`, whose combinations are those of `members` (places in `results`)
/// inside it: the first one's result, every one's register together (refcore's `group`, its
/// `coll` and `first`). L §5.5.
fn product_line(
    product: Product,
    members: &[usize],
    (combos, results): (&[Combo], &[(usize, Printed)]),
) -> Result<Grouped, Stop> {
    let mut met = Collected::default();
    let mut first: Option<&Printed> = None;
    for at in members {
        let combo = combo_of(combos, results, *at)?;
        if !product
            .iter()
            .zip(&combo)
            .all(|(named, o)| named.contains(o))
        {
            continue;
        }
        let (_, printed) = results
            .get(*at)
            .ok_or_else(|| invariant("a result past the results"))?;
        met.merge(&printed.met);
        if first.is_none() {
            first = Some(printed);
        }
    }
    let first = first.ok_or_else(|| invariant("a product of options no combination gives"))?;
    Ok(Grouped {
        product,
        text: first.text.clone(),
        met,
        laws: first.laws,
    })
}

/// The combinations `given` split into products by the axes in id order: the first axis's
/// options grouped by the set of the other axes' combinations they give the result with, in
/// the options' order, each group a product's first set, each split again on the next axis
/// (refcore's `split`; L §5.5 "A set that isn't a product"). A loop over an explicit stack,
/// the first group's products first (R-16).
fn split(axes: &Axes, given: Vec<Combo>) -> Vec<Product> {
    let mut out = Vec::new();
    let mut stack: Vec<(Product, Vec<Combo>)> = vec![(Vec::new(), given)];
    while let Some((named, rests)) = stack.pop() {
        let Some((_, options)) = axes.axes.get(named.len()) else {
            out.push(named);
            continue;
        };
        for (tails, opts) in by_rest(options.len(), &rests).into_iter().rev() {
            let mut longer = named.clone();
            longer.push(opts);
            stack.push((longer, tails.into_iter().collect()));
        }
    }
    out
}

/// The options of the next axis, positions below `count`, grouped by the set of the rests of
/// the combinations `rests` they head; the groups in the order of their first option
/// (refcore's `split`, its `groups` and `order`).
fn by_rest(count: usize, rests: &[Combo]) -> Vec<(BTreeSet<Combo>, BTreeSet<usize>)> {
    let mut groups: Vec<(BTreeSet<Combo>, BTreeSet<usize>)> = Vec::new();
    for option in 0..count {
        let mut tails = BTreeSet::new();
        for combo in rests {
            if let [head, tail @ ..] = combo.as_slice()
                && *head == option
            {
                tails.insert(tail.to_vec());
            }
        }
        if tails.is_empty() {
            continue;
        }
        match groups.iter_mut().find(|(seen, _)| *seen == tails) {
            Some((_, opts)) => {
                opts.insert(option);
            }
            None => groups.push((tails, BTreeSet::from([option]))),
        }
    }
    groups
}

/// `ENGINE-INVARIANT` for a fan-out's broken invariant.
fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{Axes, split};
    use std::collections::BTreeSet;

    fn axes(counts: &[usize]) -> Axes {
        Axes {
            axes: counts
                .iter()
                .enumerate()
                .map(|(i, n)| (format!("a{i}"), (0..*n).map(|o| format!("o{o}")).collect()))
                .collect(),
        }
    }

    fn set(xs: &[usize]) -> BTreeSet<usize> {
        xs.iter().copied().collect()
    }

    #[test]
    fn a_product_stays_one_line() {
        let two = axes(&[2, 3]);
        let all = two.combinations().expect("six combinations");
        assert_eq!(all.len(), 6, "every combination");
        assert_eq!(all.first(), Some(&vec![0, 0]), "the first axis slowest");
        assert_eq!(all.get(1), Some(&vec![0, 1]), "the last axis fastest");
        let lines = split(&two, all);
        assert_eq!(
            lines,
            vec![vec![set(&[0, 1]), set(&[0, 1, 2])]],
            "one product"
        );
    }

    #[test]
    fn a_set_that_isnt_a_product_splits_by_the_first_axis() {
        // (x1, y1) and (x2, y2) give A; the split groups x by the y's it comes with
        let two = axes(&[2, 2]);
        let lines = split(&two, vec![vec![0, 0], vec![1, 1]]);
        assert_eq!(
            lines,
            vec![vec![set(&[0]), set(&[0])], vec![set(&[1]), set(&[1])]],
            "two lines, x1's first"
        );
    }
}
