//! Fence F5: a written projection's common case read from running sums of x's counted plan legs, kept in a tree, not from the projection's own walk.
//! Spec: E §4.3 (The common case is cheap: with no decision the projection counts, `bal(x, d)
//! from t` is the start balance plus a prefix sum F of the plan movements as known on t, and
//! `low(x, a .. b) from t` a range minimum over F; F changes only with the cut min(t, K)),
//! §4.1 step 3 and §4.2 steps 2 and 5 (what the projection's walk books, refuses or makes a
//! hole at x); L §5.14.1 (`bal … from t`, `low`, `left out:`), §8.2 (a balance's register);
//! P §2.3 (F in a segment tree over the days x has a counted movement, a query O(log n)).
//! Twin: engine/tools/refcore/mreads.py (`proj_bal`, `m_e_low`) and walk.py (`proj_walk`),
//! which walk the projection day by day, as projection.rs does.
//! Invariants: a read is answered here only where the projection's walk would book nothing at
//! x in the window but whole plan legs in x's unit, inside both ends' lifespans and sign
//! rules, and run no decision; it then gives that walk's value and register, and every other
//! read is the walk's; one tree per (place, unit, moves left out) in a timeline, built from the
//! instances at the read's cut, held for reads at that cut, and gathered again at any other
//! cut, updated in place where x's days are the same; a read the projection's walk refuses on
//! a day it reaches is the walk's (pick P54, #127); the sums are whole minor units of x's unit
//! only where every position's legs are, and exact otherwise (pick P56, #128).
//! PERF: P8-5, P8-M4 (one read of a written projection in E §4.3's common case: O(log n) in
//! the n days x moves, where the walk plans and walks the window and reads each of its days;
//! S measured P8-M4's read at 72.9 ms); plain twin: projection.rs (`projection`,
//! `projected_days`: the projection's own walk); the equivalence test: the CLI's
//! `tests/walk_oracle.rs` (the kernel and M1 corpora, and a projection under axes and in a
//! scenario's world, the same bytes under `Lows::Tree` and `Lows::Walk`), and this file's tests.
//!
//! F(u) is the sum of x's counted legs dated on or before u, at the cut min(t, K). The tree's
//! positions are the days x moves on; each holds that day's legs summed, and each node the sum
//! of its positions and the least running sum inside them. A read from t through b is then
//! the start balance, plus the legs from t to the window's first day, plus the least running
//! sum after it, if below zero: two queries of the tree, each O(log n). F changes with the
//! cut alone, not only where a cut sees more facts: an instance dated before the cut is past,
//! or modelled where a version covers its place (E §4.1 step 3, L §5.19), and an amount at
//! the cut may read that (`flows`), a balance, or the cut itself (G9's fix review, B1). So a
//! tree holds for reads at its own cut only; at another cut the plan pass's instances are
//! gathered again, and where x's days are the same each position whose sum changed is updated
//! in place, O(log n) each (P §2.3's range add from a day on is one position's change here),
//! else the tree is built again. Where every position's legs are a whole number of the
//! unit's minor units, and their sizes together fit a machine integer, the nodes hold those
//! integers, so no running sum can pass it; otherwise they hold exact rationals (pick P56).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::agenda::{left_out_day, left_out_lines, one_more_left_out};
use super::day::place_sign;
use super::plan::{Amount, Dated, End, Exists, Inst, LegVal, Status, invariant, qty_value};
use super::point::At;
use super::state::Progress;
use crate::eval::{Collected, Engine, Entry, num_stop};
use crate::model::{MoveInfo, Reality, Value, World};
use crate::num::Num;
use crate::syntax::{Sign, Stop};
use crate::time::Day;

/// How the engine reads a written projection in E §4.3's common case: the switch F5's
/// equivalence test turns (P §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lows {
    /// From the running sums of x's counted legs, kept in a tree: fence F5, as `run` reads.
    Tree,
    /// Through the projection's own walk, day by day: the plain twin.
    Walk,
}

/// Where a projection starts: the cut of the walk it continues, that walk, and the point it
/// starts after (refcore's `proj_walk`). E §4.3 step 1.
pub(crate) type Base = (Day, usize, At);

/// A tree's key: the place, its unit, and the moves the projection leaves out (E §4.3 step 3).
type TreeKey = (String, String, BTreeSet<String>);

/// The most registers the legs at x may carry for a tree to read them: past it every read
/// joins too many, so the walk reads it.
const REGISTERS: usize = 64;

/// A timeline's trees, and the keys of those being built (P §2.3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Trees {
    /// The trees built, each at its cut.
    built: BTreeMap<TreeKey, Tree>,
    /// The trees being built: a read that meets one again is a read of a walk while it is
    /// walked, as the projection's walk's own plan pass meets it.
    building: BTreeSet<TreeKey>,
}

/// One place's running sums in one unit at one cut, and what keeps a read from them (E §4.3,
/// P §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Tree {
    /// The cut min(t, K) its instances are read at: the only cut it holds for, since a
    /// status, an amount reading one, and the cut itself change with the cut alone.
    cut: Day,
    /// Whether any read may take it: not where x's kind has a sign rule, a decision has no
    /// day, a hole or refusal stands from the walk's start, or the legs carry too many
    /// registers.
    open: bool,
    /// The days x moves on, the positions, in order.
    days: Vec<Day>,
    /// The running sums over the positions.
    sums: Sums,
    /// Each register the legs at x carry, with the positions where one carries it, in order.
    registers: Vec<(Collected, Vec<usize>)>,
    /// The days of acts at x the sums don't hold (a hole, a refusal, a mark, a leg the sign
    /// rule or a lifespan may refuse, a leg on another day than its movement): a read that
    /// counts one of them is the walk's.
    acts: BTreeSet<Day>,
    /// The days of the hole and refused movements that reach x or every place: a read from a
    /// day on or before one is the walk's.
    later: BTreeSet<Day>,
    /// The days the projection's walk runs a decision or ends x's lifespan: a read whose
    /// window holds one is the walk's.
    runs: BTreeSet<Day>,
    /// x's `left out:` entries: each move whose instances outside the run touch x, with its
    /// count and first day (L §8.2).
    left_out: Vec<Entry>,
    /// The days of each left-out move's instances (L §8.2 `left out:`, E §4.3 step 3).
    counted: BTreeMap<String, Vec<Day>>,
}

impl Tree {
    /// An empty tree at cut `cut`. P §2.3.
    fn new(cut: Day) -> Tree {
        Tree {
            cut,
            open: true,
            days: Vec::new(),
            sums: Sums {
                width: 1,
                minor: None,
                nodes: Vec::new(),
            },
            registers: Vec::new(),
            acts: BTreeSet::new(),
            later: BTreeSet::new(),
            runs: BTreeSet::new(),
            left_out: Vec::new(),
            counted: BTreeMap::new(),
        }
    }

    /// Whether a read from `t`, counting the days `counts` through `last`, holds nothing the
    /// sums don't: E §4.3's common case. `counts` is t, or the day after t for a read at a
    /// decision's point, after that day's plans. E §4.3, P §2.3.
    fn common(&self, t: Day, counts: Day, last: Day) -> bool {
        self.open
            && !holds_one(&self.acts, (counts, last))
            && self.later.range(t..).next().is_none()
            && !holds_one(&self.runs, (t, last))
    }

    /// The number of positions on or before `day`. P §2.3.
    fn through(&self, day: Day) -> usize {
        self.days.partition_point(|d| *d <= day)
    }

    /// The legs counted from day `counts` through `first`, then the least running sum of the
    /// days after it through `last`, if below zero: the lowest end-of-day balance of the days
    /// `first` through `last`, less the start (P §2.3: two queries of the tree), in x's unit.
    /// E §4.3.
    fn lowest_change(&self, counts: Day, (first, last): (Day, Day)) -> Result<Num, Stop> {
        let before = self.days.partition_point(|d| *d < counts);
        let (from, to) = (self.through(first), self.through(last));
        let head = self.sums.over(before, from)?;
        let tail = self.sums.over(from, to)?;
        let change = match &tail.least {
            Some(dip) if dip.is_negative() => head.sum.plus(dip)?,
            Some(_) | None => head.sum,
        };
        change.quantity(self.sums.minor.as_ref())
    }

    /// The register of the legs at x booked from day `counts` through `last`: each register
    /// carried at a position in that window (L §8.2: a balance's entries since its last
    /// reading; the projection's walk has none). E §4.3.
    fn register(&self, counts: Day, last: Day) -> Collected {
        let before = self.days.partition_point(|d| *d < counts);
        let to = self.through(last);
        let mut met = Collected::default();
        for (coll, at) in &self.registers {
            let first = at.partition_point(|p| *p < before);
            if at.get(first).is_some_and(|p| *p < to) {
                met.merge(coll);
            }
        }
        for entry in &self.left_out {
            met.entries.insert(entry.clone());
        }
        met
    }
}

/// A sum the tree holds: a whole number of x's unit's minor units, or an exact quantity in
/// the unit (pick P56). A tree's figures are all of one kind.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Fig {
    /// Minor units, never past the bound the tree was made under.
    Minor(i64),
    /// An exact rational.
    Exact(Num),
}

impl Fig {
    /// No legs: zero, of the kind a tree whose minor unit is `minor` holds. P §2.3.
    fn zero(minor: Option<&Num>) -> Fig {
        match minor {
            Some(_) => Fig::Minor(0),
            None => Fig::Exact(Num::zero()),
        }
    }

    /// The two figures added. P §2.3.
    fn plus(&self, other: &Fig) -> Result<Fig, Stop> {
        match (self, other) {
            (Fig::Minor(a), Fig::Minor(b)) => a
                .checked_add(*b)
                .map(Fig::Minor)
                .ok_or_else(|| invariant("a running sum past the bound its tree was made under")),
            (Fig::Exact(a), Fig::Exact(b)) => a.checked_add(b).map(Fig::Exact).map_err(num_stop),
            (Fig::Minor(_), Fig::Exact(_)) | (Fig::Exact(_), Fig::Minor(_)) => {
                Err(invariant("a tree's sums of two kinds"))
            }
        }
    }

    /// Whether this figure is below `other`. P §2.3.
    fn below(&self, other: &Fig) -> Result<bool, Stop> {
        match (self, other) {
            (Fig::Minor(a), Fig::Minor(b)) => Ok(a < b),
            (Fig::Exact(a), Fig::Exact(b)) => Ok(a.compare(b).map_err(num_stop)? == Ordering::Less),
            (Fig::Minor(_), Fig::Exact(_)) | (Fig::Exact(_), Fig::Minor(_)) => {
                Err(invariant("a tree's sums of two kinds"))
            }
        }
    }

    /// Whether the figure is below zero. P §2.3.
    fn is_negative(&self) -> bool {
        match self {
            Fig::Minor(n) => *n < 0,
            Fig::Exact(q) => q.is_negative(),
        }
    }

    /// The figure as a quantity in x's unit: its minor units times the minor unit `minor`, or
    /// itself where the tree is exact (pick P56). L §7.2.
    fn quantity(&self, minor: Option<&Num>) -> Result<Num, Stop> {
        match (self, minor) {
            (Fig::Minor(n), Some(unit)) => Num::int(*n).checked_mul(unit).map_err(num_stop),
            (Fig::Exact(q), None) => Ok(q.clone()),
            (Fig::Minor(_), None) | (Fig::Exact(_), Some(_)) => {
                Err(invariant("a tree's sums of two kinds"))
            }
        }
    }
}

/// The positions' sums as a tree holds them, and the minor unit they count where they are
/// whole minor units (pick P56).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Figures {
    /// The unit's minor unit, where every figure counts it; none where they are exact.
    minor: Option<Num>,
    /// One figure for each position, in order.
    figs: Vec<Fig>,
}

/// Positions whose legs sum to `steps`, in a unit whose minor unit is `minor` (none for a
/// unit without one): whole minor units where every step is a whole number of them and
/// their sizes together fit an `i64`, so no running sum of them can pass it; exact otherwise
/// (pick P56). L §7.2, P §2.3.
fn figures(steps: Vec<Num>, minor: Option<&Num>) -> Figures {
    if let Some(unit) = minor
        && let Some(counts) = in_minor_units(&steps, unit)
    {
        return Figures {
            minor: Some(unit.clone()),
            figs: counts.into_iter().map(Fig::Minor).collect(),
        };
    }
    Figures {
        minor: None,
        figs: steps.into_iter().map(Fig::Exact).collect(),
    }
}

/// Each step as a whole number of minor units `unit`; none where one isn't, or where their
/// sizes added pass an `i64` (pick P56). L §7.2.
fn in_minor_units(steps: &[Num], unit: &Num) -> Option<Vec<i64>> {
    let mut size: i64 = 0;
    let mut out = Vec::with_capacity(steps.len());
    for q in steps {
        let Ok(units) = q.checked_div(unit) else {
            return None;
        };
        let n = units.to_i64()?;
        size = size.checked_add(n.checked_abs()?)?;
        out.push(n);
    }
    Some(out)
}

/// One node of the tree: the sum of its positions' legs and the least running sum inside
/// them, none for no position (P §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    /// The sum.
    sum: Fig,
    /// The least running sum, from its first position.
    least: Option<Fig>,
}

impl Node {
    /// No position: the sum zero, no running sum. P §2.3.
    fn empty(minor: Option<&Num>) -> Node {
        Node {
            sum: Fig::zero(minor),
            least: None,
        }
    }

    /// One position whose legs sum to `q`. P §2.3.
    fn day(q: Fig) -> Node {
        Node {
            least: Some(q.clone()),
            sum: q,
        }
    }

    /// This node's positions, then `right`'s: the sums added; the least running sum this
    /// one's, or its sum plus the right's least (P §2.3).
    fn then(&self, right: &Node) -> Result<Node, Stop> {
        let sum = self.sum.plus(&right.sum)?;
        let shifted = match &right.least {
            Some(l) => Some(self.sum.plus(l)?),
            None => None,
        };
        let least = match (self.least.clone(), shifted) {
            (Some(a), Some(b)) => Some(if b.below(&a)? { b } else { a }),
            (a, None) => a,
            (None, b) => b,
        };
        Ok(Node { sum, least })
    }
}

/// The running sums over the positions: a tree whose leaves are the positions from `width`
/// on, each parent the two below it side by side (P §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Sums {
    /// The leaves' count, a power of two at least the positions'.
    width: usize,
    /// The minor unit the figures count, where they are whole minor units (pick P56).
    minor: Option<Num>,
    /// The nodes: 1 the root, `2i` and `2i + 1` the two below `i`; 0 unused.
    nodes: Vec<Node>,
}

impl Sums {
    /// The tree over the positions' figures, built from the leaves up, O(n). P §2.3.
    fn of(figures: Figures) -> Result<Sums, Stop> {
        let Figures { minor, figs } = figures;
        let width = figs
            .len()
            .max(1)
            .checked_next_power_of_two()
            .ok_or_else(|| invariant("a tree past the positions there can be"))?;
        let size = width
            .checked_mul(2)
            .ok_or_else(|| invariant("a tree past the positions there can be"))?;
        let mut sums = Sums {
            width,
            nodes: vec![Node::empty(minor.as_ref()); size],
            minor,
        };
        for (i, q) in figs.into_iter().enumerate() {
            let at = sums.leaf(i)?;
            *sums.node_mut(at)? = Node::day(q);
        }
        for i in (1..width).rev() {
            sums.join(i)?;
        }
        Ok(sums)
    }

    /// The tree with each position whose figure `figures` changes set to it and the nodes
    /// above it joined again: P §2.3's range add on F from that position on, in place,
    /// O(log n) each; built again where the figures are of another kind (pick P56).
    /// `figures` holds one for each of the tree's positions.
    fn updated(mut self, figures: Figures) -> Result<Sums, Stop> {
        if figures.minor != self.minor {
            return Sums::of(figures);
        }
        for (i, q) in figures.figs.into_iter().enumerate() {
            let at = self.leaf(i)?;
            if self.node(at)?.sum == q {
                continue;
            }
            *self.node_mut(at)? = Node::day(q);
            let mut below = at;
            while let Some(up) = below.checked_shr(1).filter(|up| *up > 0) {
                self.join(up)?;
                below = up;
            }
        }
        Ok(self)
    }

    /// Node `i` made again from the two below it. P §2.3.
    fn join(&mut self, i: usize) -> Result<(), Stop> {
        let below = i.checked_mul(2).ok_or_else(|| invariant("a node"))?;
        let right = below.checked_add(1).ok_or_else(|| invariant("a node"))?;
        let joined = self.node(below)?.then(self.node(right)?)?;
        *self.node_mut(i)? = joined;
        Ok(())
    }

    /// The node at `i`, to change. P §2.3.
    fn node_mut(&mut self, i: usize) -> Result<&mut Node, Stop> {
        self.nodes
            .get_mut(i)
            .ok_or_else(|| invariant("a node of the tree"))
    }

    /// The positions `from` up to `to` (not included) as one node, their legs in order: the
    /// nodes that cover them, met from both ends up the tree, O(log n). P §2.3.
    fn over(&self, from: usize, to: usize) -> Result<Node, Stop> {
        let empty = Node::empty(self.minor.as_ref());
        let (mut left, mut right) = (empty.clone(), empty);
        let (mut l, mut r) = (self.leaf(from)?, self.leaf(to)?);
        while l < r {
            if l & 1 == 1 {
                left = left.then(self.node(l)?)?;
                l = l.checked_add(1).ok_or_else(|| invariant("a node"))?;
            }
            if r & 1 == 1 {
                r = r.checked_sub(1).ok_or_else(|| invariant("a node"))?;
                right = self.node(r)?.then(&right)?;
            }
            l = l.checked_shr(1).ok_or_else(|| invariant("a node"))?;
            r = r.checked_shr(1).ok_or_else(|| invariant("a node"))?;
        }
        left.then(&right)
    }

    /// The place of position `i`'s leaf among the nodes. P §2.3.
    fn leaf(&self, i: usize) -> Result<usize, Stop> {
        self.width
            .checked_add(i)
            .ok_or_else(|| invariant("a leaf of the tree"))
    }

    /// The node at `i`. P §2.3.
    fn node(&self, i: usize) -> Result<&Node, Stop> {
        self.nodes
            .get(i)
            .ok_or_else(|| invariant("a node of the tree"))
    }
}

/// Fence F5's range minimum alone, the tree a served read queries, for P8-5's bench (pick P56,
/// #128): positions of x's legs summed by day, as a tree gathers them, and the lowest change of
/// x's balance over a window. P §2.3, E §4.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeMinimum {
    /// The tree, its days and sums set, nothing keeping a read from it.
    tree: Tree,
}

impl RangeMinimum {
    /// The tree over `days`, in order, one position each, whose legs sum to `steps` in a
    /// unit whose minor unit is `minor` (none for a unit without one): whole minor units
    /// where every step is a whole number of them, else exact, as a gathered tree holds them.
    /// P §2.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for no day, days out of order or repeated, or a step count other
    /// than the days'.
    pub fn of(days: Vec<Day>, steps: Vec<Num>, minor: Option<&Num>) -> Result<RangeMinimum, Stop> {
        let Some(first) = days.first().copied() else {
            return Err(invariant("a range minimum over no day"));
        };
        if steps.len() != days.len() || days.iter().zip(days.iter().skip(1)).any(|(a, b)| a >= b) {
            return Err(invariant(
                "a range minimum's days out of order, or not one step each",
            ));
        }
        let mut tree = Tree::new(first);
        tree.days = days;
        tree.sums = Sums::of(figures(steps, minor))?;
        Ok(RangeMinimum { tree })
    }

    /// The legs counted from day `counts` through `first`, then the least running sum of the
    /// days after it through `last`, if below zero: what a served `low(x, first .. last) from
    /// t` adds to its start balance, two queries of the tree. P §2.3, E §4.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a node the tree lacks; a number past the cap.
    pub fn lowest_change(&self, counts: Day, window: (Day, Day)) -> Result<Num, Stop> {
        self.tree.lowest_change(counts, window)
    }

    /// Whether the sums are whole minor units, not exact rationals (pick P56). P §2.3.
    #[must_use]
    pub fn in_minor_units(&self) -> bool {
        self.tree.sums.minor.is_some()
    }
}

/// x's counted legs and what keeps a read from them, as a tree's build gathers them.
struct Gathered {
    /// The tree, its sums not yet made.
    tree: Tree,
    /// Each counted leg's end at x: its day, its signed quantity, its register.
    legs: Vec<(Day, Num, Collected)>,
    /// Each move with instances outside the run: how many, and the first day (refcore's
    /// `note_leftout`).
    outside: BTreeMap<String, (u32, Dated)>,
    /// The moves outside the run with an instance whose leg touches x.
    touching: BTreeSet<String>,
}

impl Engine {
    /// `bal(x, d) from t` or `low(x, a .. b) from t` in E §4.3's common case, from the tree:
    /// the lowest end-of-day balance of the days `first` through `last` in the walk continued
    /// from `base` on day t (`bal`: one day), with the register that walk's reads carry; none
    /// where the read isn't the common case, for the projection's own walk to read (P §2.3).
    /// E §4.3, L §5.14.1.
    ///
    /// # Errors
    ///
    /// The refusals the cut's instances meet; `NotBuilt` for a read met again while its tree
    /// is built.
    pub(super) fn common_low(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        (t, (first, last)): (Day, (Day, Day)),
        (cut, base, point): Base,
        left_out: &BTreeSet<String>,
    ) -> Result<Option<Value>, Stop> {
        // the walk's first day holds, after a decision's point, only the decisions after it
        let counts = if point <= At::start(t) {
            t
        } else {
            match t.next() {
                Some(d) => d,
                None => return Ok(None),
            }
        };
        let c = t.min(cut);
        let world = self.walk(base)?.world;
        if world != self.timeline_world() {
            return Ok(None);
        }
        // the projection's own walk, failed on a day the read reaches, refuses it (P54)
        if let Some(id) = self.proj_walks.get(&(base, point, c, left_out.clone()))
            && fails_by(&self.walk(*id)?.progress, last)
        {
            return Ok(None);
        }
        let key = (place.to_owned(), unit.to_owned(), left_out.clone());
        self.tree_at(m, (place, unit), (c, world), &key)?;
        let tree = self
            .trees
            .built
            .get(&key)
            .ok_or_else(|| invariant("a tree just built"))?;
        if !tree.common(t, counts, last) {
            return Ok(None);
        }
        let Some(start) = self
            .walk(base)?
            .pstate(m, place, unit, point)
            .known()
            .cloned()
        else {
            return Ok(None);
        };
        let low = start
            .checked_add(&tree.lowest_change(counts, (first, last))?)
            .map_err(num_stop)?;
        let mut met = tree.register(counts, last);
        let lines = left_out_lines(&tree.counted, (t, last))?;
        for line in lines {
            self.note("left out", line, String::new());
        }
        self.chain_register(m, (base, point), (place, unit), &mut met)?;
        self.absorb(&met);
        qty_value(m, &low, unit).map(Some)
    }

    /// The tree of `key` at cut `cut`, gathered from the cut's instances unless it is already,
    /// its reads kept apart from the reading cell's as the projection's walk keeps its plan
    /// pass's (E §3.2, §4.1); the tree of another cut updated in place where x's days are the
    /// same. P §2.3.
    fn tree_at(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        (cut, world): (Day, World),
        key: &TreeKey,
    ) -> Result<(), Stop> {
        if self.trees.built.get(key).is_some_and(|t| t.cut == cut) {
            return Ok(());
        }
        if self.trees.building.contains(key) {
            return Err(Stop::NotBuilt("a read of a walk while it is walked"));
        }
        self.trees.building.insert(key.clone());
        let mark = self.reads_apart();
        self.push();
        let built = self.build_tree(m, (place, unit), (cut, world), &key.2);
        self.pop_alone();
        self.reads_back(mark);
        self.trees.building.remove(key);
        let (mut tree, steps) = built?;
        let figs = figures(steps, m.currencies.get(unit));
        // P §2.3: the same days, so each changed position is one update in place
        tree.sums = match self.trees.built.remove(key) {
            Some(old) if old.days == tree.days => old.sums.updated(figs)?,
            Some(_) | None => Sums::of(figs)?,
        };
        self.trees.built.insert(key.clone(), tree);
        Ok(())
    }

    /// A tree from every move's instances at cut `cut` in `world`, in the projection's walk's
    /// order (refcore's `prepare`, as a projection's plan pass reads it): the counted legs at
    /// x summed by day, and what the walk would book there otherwise. E §4.3, §4.1, P §2.3.
    fn build_tree(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        (cut, world): (Day, World),
        left_out: &BTreeSet<String>,
    ) -> Result<(Tree, Vec<Num>), Stop> {
        // the projection's walk reads its cut's first readings before its plan pass
        crate::book::first_readings(m, (cut, world))?;
        let mut got = Gathered {
            tree: Tree::new(cut),
            legs: Vec::new(),
            outside: BTreeMap::new(),
            touching: BTreeSet::new(),
        };
        for mv in m.moves_in(world) {
            let insts = self.move_instances(m, mv, cut)?;
            for inst in insts.iter() {
                if left_out.contains(&mv.id) {
                    count_left_out(&mut got.tree, mv, inst);
                } else if mv.decision {
                    judge_decision(&mut got.tree, inst);
                } else if inst.status == Status::None {
                    let places = self.decided_places(m, mv, inst, cut)?;
                    got.decided(inst, (place, unit), &places);
                } else {
                    got.judge_plan(m, (place, unit), (cut, world), inst)?;
                }
            }
        }
        got.finish(m, place, (cut, world))
    }
}

impl Gathered {
    /// A plan instance of a move the projection counts, as its walk schedules it by status
    /// (refcore's `prepare`, its plans): left out, a mark, a hole or a refusal, or its legs
    /// booked. E §4.1 step 3, §4.2, L §8.2.
    fn judge_plan(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        seen: (Day, World),
        inst: &Inst,
    ) -> Result<(), Stop> {
        let day = inst.day.as_ref().and_then(Dated::day);
        match inst.status {
            Status::LeftOut => self.outside_run(inst, place)?,
            Status::Modelled if inst.obs.is_empty() => self.marks(inst, place),
            Status::Refused => self.refused(inst, day, place),
            Status::Hole => self.hole(inst, day, place),
            Status::Past | Status::Plan => {
                for lg in &inst.legs {
                    self.leg(m, (place, unit), seen, (lg, day), &inst.coll)?;
                }
            }
            Status::Modelled
            | Status::None
            | Status::Decision
            | Status::Open
            | Status::Uncertain => {}
        }
        Ok(())
    }

    /// An instance that doesn't exist (a false `when` or key `where`): the reads that decided
    /// it carried at x from its day, where a leg gives x, a position that moves nothing
    /// (refcore's `note_none`). L §8.2 One read set (tally 2.1.7 R7).
    fn decided(&mut self, inst: &Inst, (place, unit): (&str, &str), places: &[(String, String)]) {
        let Some(d) = inst.day.as_ref().and_then(Dated::day) else {
            return;
        };
        if places
            .iter()
            .any(|(p, u)| p == place && (u == unit || u == super::state::ALL))
        {
            self.legs.push((d, Num::zero(), inst.coll.clone()));
        }
    }

    /// An instance outside the run: counted with its move's, which keeps a `left out:` entry
    /// at x where one of its legs touches x (refcore's `note_leftout`). L §8.2, E §4.1 step 3.
    fn outside_run(&mut self, inst: &Inst, place: &str) -> Result<(), Stop> {
        one_more_left_out(&mut self.outside, inst)?;
        if inst.legs.iter().any(|lg| at_place(lg, place)) {
            self.touching.insert(inst.move_id.clone());
        }
        Ok(())
    }

    /// A modelled-past instance nothing fulfils: its marks at x, one per leg on its day
    /// (refcore's `prepare`, `open`). L §5.19, §8.2.
    fn marks(&mut self, inst: &Inst, place: &str) {
        for lg in inst.legs.iter().filter(|lg| at_place(lg, place)) {
            if let Dated::Day(d) = lg.day {
                self.tree.acts.insert(d);
            }
        }
    }

    /// A refused instance: its legs refused at their places from its day, or every place where
    /// a leg's place can't be read; from the walk's start when it has no day. L §8.4, E §4.2.
    fn refused(&mut self, inst: &Inst, day: Option<Day>, place: &str) {
        let whole = !inst.legs.is_empty()
            && inst
                .legs
                .iter()
                .all(|lg| lg.from.place().is_some() && lg.to.place().is_some());
        let reaches = !whole || inst.legs.iter().any(|lg| at_place(lg, place));
        match day {
            Some(d) if reaches => {
                self.tree.later.insert(d);
            }
            Some(_) => {}
            None => self.tree.open = false,
        }
    }

    /// A hole movement: a hole at its legs' places from their days, from the walk's start
    /// when its day is a hole. E §4.2 Holes in the walk.
    fn hole(&mut self, inst: &Inst, day: Option<Day>, place: &str) {
        let touches = inst.legs.iter().any(|lg| at_place(lg, place));
        match day {
            Some(d) if touches => {
                self.tree.later.insert(d);
            }
            None if touches || inst.legs.is_empty() => self.tree.open = false,
            Some(_) | None => {}
        }
    }

    /// One leg of a booked plan instance dated `day`: its ends at x summed into the tree where
    /// the walk would book them whole (a quantity in x's unit, both ends places inside their
    /// lifespans, neither with a sign rule), its day an act the sums don't hold otherwise.
    /// E §4.2 steps 2 and 5, L §5.14 Legs, §5.10.
    fn leg(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        seen: (Day, World),
        (lg, day): (&LegVal, Option<Day>),
        coll: &Collected,
    ) -> Result<(), Stop> {
        if !at_place(lg, place) {
            return Ok(());
        }
        let d = match lg.day {
            Dated::Day(d) => d,
            Dated::Hole(_) => {
                if let Some(first) = day {
                    self.tree.later.insert(first);
                }
                return Ok(());
            }
        };
        let q = match (&lg.amount, lg.unit.as_deref()) {
            (Amount::Qty(q), Some(u)) if u == unit => q,
            (Amount::Qty(_), Some(_)) => return Ok(()),
            (Amount::Qty(_) | Amount::Hole(_) | Amount::Refused(_), _) => {
                self.tree.acts.insert(d);
                return Ok(());
            }
        };
        if q.is_zero() {
            return Ok(());
        }
        if day != Some(d) || !ends_book(m, lg, d, seen) {
            self.tree.acts.insert(d);
            return Ok(());
        }
        if lg.from.place() == Some(place) {
            let out = q.checked_neg().map_err(num_stop)?;
            self.legs.push((d, out, coll.clone()));
        }
        if lg.to.place() == Some(place) {
            self.legs.push((d, q.clone(), coll.clone()));
        }
        Ok(())
    }

    /// The tree made from what was gathered, with each position's legs summed, its sums still
    /// to make: x's sign rule and lifespan's end, its `left out:` entries, the positions and
    /// their registers. P §2.3, L §8.2.
    fn finish(
        mut self,
        m: &Reality,
        place: &str,
        seen: (Day, World),
    ) -> Result<(Tree, Vec<Num>), Stop> {
        let mut tree = self.tree;
        if matches!(place_sign(m, place), Some(Sign::Min0 | Sign::Max0)) {
            tree.open = false;
        }
        if let Some(last) = m
            .things
            .get(place)
            .and_then(|t| m.lifespan(t, seen))
            .and_then(|sp| sp.end())
            .and_then(Day::previous)
        {
            tree.runs.insert(last);
        }
        for (mid, (n, first)) in &self.outside {
            if self.touching.contains(mid) {
                tree.left_out.push(Entry {
                    word: "left out",
                    text: format!("{mid}, {n} instances, first {}", first.text()),
                    figure: String::new(),
                });
            }
        }
        for days in tree.counted.values_mut() {
            days.sort();
        }
        self.legs.sort_by_key(|(d, _, _)| *d);
        let mut steps: Vec<Num> = Vec::new();
        for (d, q, coll) in self.legs {
            if tree.days.last() == Some(&d) {
                let sum = steps.last_mut().ok_or_else(|| invariant("a day's sum"))?;
                *sum = sum.checked_add(&q).map_err(num_stop)?;
            } else {
                tree.days.push(d);
                steps.push(q);
            }
            let at = tree
                .days
                .len()
                .checked_sub(1)
                .ok_or_else(|| invariant("a position"))?;
            carried(&mut tree, coll, at);
        }
        Ok((tree, steps))
    }
}

/// Notes that a leg at position `at` carries register `coll`, among the tree's distinct
/// registers; past [`REGISTERS`] the tree is closed. L §8.2.
fn carried(tree: &mut Tree, coll: Collected, at: usize) {
    if let Some((_, places)) = tree.registers.iter_mut().find(|(c, _)| *c == coll) {
        if places.last() != Some(&at) {
            places.push(at);
        }
        return;
    }
    if tree.registers.len() >= REGISTERS {
        tree.open = false;
        return;
    }
    tree.registers.push((coll, vec![at]));
}

/// A decision instance of a move the projection counts: its day one the walk runs a decision
/// on; none, or a hole, keeps every read from the tree (refcore's `prepare`, its decisions).
/// E §4.2 step 3, §4.3 step 2.
fn judge_decision(tree: &mut Tree, inst: &Inst) {
    if inst.exists == Exists::No {
        return;
    }
    match inst.day.as_ref().and_then(Dated::day) {
        Some(d) => {
            tree.runs.insert(d);
        }
        None => tree.open = false,
    }
}

/// An instance of a move the projection leaves out, its day kept for the `left out:` line of
/// a read whose window holds it: a plan's when it exists, a decision's always (refcore's
/// `count_leftout`). E §4.3 step 3, L §8.2.
fn count_left_out(tree: &mut Tree, mv: &MoveInfo, inst: &Inst) {
    if mv.decision || inst.status != Status::None {
        left_out_day(&mut tree.counted, inst);
    }
}

/// Whether a projection's walk in `progress` refuses a read through day `last`: it failed on
/// that day or before it, or in its plan pass or close (pick P54, #127). E §4.3, L §8.4.
fn fails_by(progress: &Progress, last: Day) -> bool {
    match progress {
        Progress::Failed(_, on) => on.is_none_or(|d| d <= last),
        Progress::Planning | Progress::Walking | Progress::Walked | Progress::Done => false,
    }
}

/// Whether a leg has an end at `place`. E §4.2.
fn at_place(lg: &LegVal, place: &str) -> bool {
    lg.from.place() == Some(place) || lg.to.place() == Some(place)
}

/// Whether the walk books both ends of a leg on day `d` as they stand: each a place inside its
/// lifespan with no question of its end, and neither with a sign rule the sums can't test
/// (E §4.2 step 5; L §5.10, §5.14 Legs).
fn ends_book(m: &Reality, lg: &LegVal, d: Day, seen: (Day, World)) -> bool {
    end_books(m, &lg.from, d, seen) && end_books(m, &lg.to, d, seen)
}

/// Whether the walk books one end of a leg on day `d` as it stands: a place inside its
/// lifespan with no question of its end, with no sign rule the sums can't test (E §4.2 step 5;
/// L §5.10, §5.14 Legs).
fn end_books(m: &Reality, end: &End, d: Day, seen: (Day, World)) -> bool {
    let End::Place(p) = end else {
        return false;
    };
    let Some(span) = m.things.get(p).and_then(|t| m.lifespan(t, seen)) else {
        return false;
    };
    span.contains(d)
        && span.end_question().is_none()
        && !matches!(place_sign(m, p), Some(Sign::Min0 | Sign::Max0))
}

/// Whether a day of `days` lies in `from` through `last`, none where `from` is past `last`
/// (a read at a decision's point on its window's only day). P §2.3.
fn holds_one(days: &BTreeSet<Day>, (from, last): (Day, Day)) -> bool {
    days.range(from..).next().is_some_and(|d| *d <= last)
}

#[cfg(test)]
mod tests {
    use super::{Lows, Node, RangeMinimum, Sums, figures};
    use crate::api::Source;
    use crate::eval::{Ctx, Engine, Env, Memo};
    use crate::load::load;
    use crate::model::{Reality, Value};
    use crate::num::Num;
    use crate::syntax::{AskBody, parse_reality};
    use crate::time::Day;
    use core::cmp::Ordering;

    /// A place paid monthly and charged monthly, an amount nobody has stated from August,
    /// and reads the tree takes and reads it leaves to the walk.
    const REALITY: &str = r#"model lt
  tally 2
  on 2026-03-15
  start 2026-01-01
  end 2026-12-31

source card "invented"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

index month = month 2026-01..2026-12

place out: ext lives 2026-01-01..

place konto: accounts DUK lives 2026-01-01..

input pay: money DUK

input rent: money DUK

input rent_off: int

input gift: money DUK question "How much is the gift?"

input gift_day: date

move salary[m in month]: transfers
  on start_of(m)
  at start_of(m)
  leg cash from out to konto amount pay

move rent_pay[m in month]: transfers
  on add_days(start_of(m), rent_off)
  at start_of(m)
  leg cash from konto to out amount rent

move late: transfers
  on gift_day
  at gift_day
  leg cash from out to konto amount gift

ask low_from: low(konto, 2026-02-01 .. 2026-06-30) from 2026-01-20

ask bal_from: bal(konto, 2026-06-30) from 2026-02-01

ask low_hole: low(konto, 2026-04-01 .. 2026-09-30) from 2026-04-01

ask low_unc: low(konto, 2026-02-01 .. 2026-06-30) from 2026-01-20 with uncertain

ask low_feb: low(konto, 2026-02-01 .. 2026-06-30) from 2026-02-01

ask low_mar: low(konto, 2026-03-01 .. 2026-06-30) from 2026-03-01

version facts
  cite card
  governs always
  known 2026-01-01
  pay = 1000.00
  rent = 1200.00
  rent_off = 9
  gift_day = 2026-08-10
  opening[konto] = 500.00 at 2026-01-01 start

version rise
  cite card
  governs 2026-05-01..
  known 2026-02-15
  replace rent = 1250.00 "the rent rose"
"#;

    fn reality() -> Reality {
        let files = parse_reality(&[Source {
            path: "lt.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("parses");
        load(files).expect("loads")
    }

    /// Ask `id`'s value, read by an engine reading a projection's common case as `lows` says;
    /// the projection walks it made and the trees it built.
    fn read(m: &Reality, id: &str, lows: Lows) -> (Value, usize, usize) {
        let ask = m.asks.iter().find(|a| a.id == id).expect("the ask");
        let AskBody::Exprs(exprs) = &ask.body else {
            panic!("{id} asks for a value");
        };
        let expr = exprs.first().expect("a value");
        let mut engine = Engine::new(m.on)
            .with_how((Memo::Classes, lows))
            .for_ask(ask);
        let mut ctx = Ctx::new(None, m.on).in_world(m.scenarios.model_world());
        ctx.unc = ask.uncertain;
        engine.enter_timeline(m, ctx).expect("its timeline");
        let v = engine.value(m, expr, &Env::empty(), ctx).expect("a value");
        (v, engine.proj_walks.len(), engine.trees.built.len())
    }

    #[test]
    fn the_tree_reads_the_common_case_where_the_walk_isnt_walked() {
        let m = reality();
        for id in ["low_from", "bal_from"] {
            let (tree, walks, trees) = read(&m, id, Lows::Tree);
            let (walk, twin_walks, twin_trees) = read(&m, id, Lows::Walk);
            assert_eq!(tree, walk, "{id}: the same value");
            assert_eq!(
                (walks, trees),
                (0, 1),
                "{id}: the tree, no projection's walk"
            );
            assert_eq!((twin_walks, twin_trees), (1, 0), "{id}: the twin walks it");
        }
    }

    /// Asks `ids` read in order by one engine reading a projection's common case as `lows`
    /// says: each value, and the cut of the engine's one tree after it, none for no tree.
    fn read_in_order(m: &Reality, ids: &[&str], lows: Lows) -> Vec<(Value, Option<Day>)> {
        let first = ids.first().expect("an ask");
        let holder = m.asks.iter().find(|a| a.id == *first).expect("the ask");
        let mut engine = Engine::new(m.on)
            .with_how((Memo::Classes, lows))
            .for_ask(holder);
        let ctx = Ctx::new(None, m.on).in_world(m.scenarios.model_world());
        engine.enter_timeline(m, ctx).expect("its timeline");
        let mut out = Vec::new();
        for id in ids {
            let ask = m.asks.iter().find(|a| a.id == *id).expect("the ask");
            let AskBody::Exprs(exprs) = &ask.body else {
                panic!("{id} asks for a value");
            };
            let expr = exprs.first().expect("a value");
            let v = engine.value(m, expr, &Env::empty(), ctx).expect("a value");
            out.push((v, engine.trees.built.values().next().map(|t| t.cut)));
        }
        out
    }

    #[test]
    fn a_tree_is_gathered_again_at_each_new_cut() {
        let m = reality();
        // from 01-20, then 02-01 (the same facts, the January plans past), 03-01 (past
        // `rise`, known 02-15), and back
        let ids = ["low_from", "low_feb", "low_mar", "low_from"];
        let tree = read_in_order(&m, &ids, Lows::Tree);
        let walk = read_in_order(&m, &ids, Lows::Walk);
        let values = |reads: &[(Value, Option<Day>)]| -> Vec<Value> {
            reads.iter().map(|(v, _)| v.clone()).collect()
        };
        assert_eq!(values(&tree), values(&walk), "the walk's values");
        let day = |month: u8, d: u8| Day::from_ymd(2026, month, d).expect("a day");
        let cuts: Vec<Option<Day>> = tree.iter().map(|(_, cut)| *cut).collect();
        assert_eq!(
            cuts,
            [
                Some(day(1, 20)),
                Some(day(2, 1)),
                Some(day(3, 1)),
                Some(day(1, 20))
            ],
            "each read's own cut"
        );
    }

    /// The two kinds of sums a tree holds: none, exact; a minor unit of 1, whole units.
    fn kinds() -> [Option<Num>; 2] {
        [None, Some(Num::int(1))]
    }

    #[test]
    fn a_tree_updated_in_place_is_the_tree_built_anew() {
        for steps in steps(20_261_110) {
            let changed: Vec<Num> = steps
                .iter()
                .enumerate()
                .map(|(i, q)| {
                    if i.checked_rem_euclid(3) == Some(0) {
                        q.checked_add(&Num::int(-37)).expect("a sum")
                    } else {
                        q.clone()
                    }
                })
                .collect();
            for (old, new) in [(0, 0), (1, 1), (1, 0), (0, 1)] {
                let (old, new) = (&kinds()[old], &kinds()[new]);
                let updated = Sums::of(figures(steps.clone(), old.as_ref()))
                    .expect("a tree")
                    .updated(figures(changed.clone(), new.as_ref()))
                    .expect("updated");
                assert_eq!(
                    updated,
                    Sums::of(figures(changed.clone(), new.as_ref())).expect("a tree"),
                    "{} positions, {old:?} to {new:?}",
                    steps.len()
                );
            }
        }
    }

    #[test]
    fn whole_minor_units_give_the_exact_range_minimum() {
        let cent = Num::ratio(1, 100).expect("a cent");
        let first = Day::from_ymd(2026, 1, 1).expect("a day");
        for steps in steps(20_261_111) {
            let days: Vec<Day> = (0..steps.len())
                .map(|i| {
                    first
                        .add_days(i64::try_from(i).expect("a day"))
                        .expect("a day")
                })
                .collect();
            let cents: Vec<Num> = steps
                .iter()
                .map(|q| q.checked_mul(&cent).expect("cents"))
                .collect();
            let Ok(minor) = RangeMinimum::of(days.clone(), cents.clone(), Some(&cent)) else {
                continue;
            };
            let exact = RangeMinimum::of(days.clone(), cents, None).expect("a tree");
            assert!(
                minor.in_minor_units() && !exact.in_minor_units(),
                "one tree of each kind"
            );
            for (i, a) in days.iter().enumerate() {
                same_changes((&minor, &exact), &days, (first, *a, i));
            }
        }
        // a step that isn't a whole number of cents keeps the tree exact
        let tenth = Num::ratio(1, 1000).expect("a tenth of a cent");
        let odd = RangeMinimum::of(vec![first], vec![tenth], Some(&cent)).expect("a tree");
        assert!(!odd.in_minor_units(), "a tenth of a cent is exact");
    }

    /// The two trees' changes over every window from day `a`, the `i`th, counted from `first`
    /// and from `a`.
    fn same_changes(
        (minor, exact): (&RangeMinimum, &RangeMinimum),
        days: &[Day],
        (first, a, i): (Day, Day, usize),
    ) {
        for b in days.iter().skip(i) {
            for counts in [first, a] {
                assert_eq!(
                    minor.lowest_change(counts, (a, *b)).expect("a change"),
                    exact.lowest_change(counts, (a, *b)).expect("a change"),
                    "{counts:?} {a:?} {b:?}"
                );
            }
        }
    }

    #[test]
    fn a_read_the_sums_cant_hold_is_the_walks() {
        let m = reality();
        // a hole amount in the window, then `with uncertain`
        for id in ["low_hole", "low_unc"] {
            let (tree, walks, _) = read(&m, id, Lows::Tree);
            let (walk, _, _) = read(&m, id, Lows::Walk);
            assert_eq!(tree, walk, "{id}: the same value");
            assert_eq!(walks, 1, "{id}: the projection's walk reads it");
        }
    }

    /// Steps from a fixed seed, as the corpora's generators make cases: 0 to 40 positions of
    /// amounts from -500 to 500.
    fn steps(seed: u64) -> Vec<Vec<Num>> {
        // xorshift, a draw below `below`
        let mut state = seed;
        let mut next = |below: u64| {
            state ^= state.checked_shl(13).expect("a shift");
            state ^= state.checked_shr(7).expect("a shift");
            state ^= state.checked_shl(17).expect("a shift");
            state.checked_rem_euclid(below).expect("a draw")
        };
        (0..60)
            .map(|_| {
                let n = next(41);
                (0..n)
                    .map(|_| {
                        let q = i64::try_from(next(1001)).expect("a small draw");
                        Num::int(q.checked_sub(500).expect("a small draw"))
                    })
                    .collect()
            })
            .collect()
    }

    /// The positions `from..to` folded one by one, their sum and least running sum: the plain
    /// twin of [`Sums::over`].
    fn folded(steps: &[Num], from: usize, to: usize) -> (Num, Option<Num>) {
        let mut sum = Num::zero();
        let mut least: Option<Num> = None;
        for q in &steps[from..to] {
            sum = sum.checked_add(q).expect("a sum");
            let lower = least
                .as_ref()
                .is_none_or(|l| sum.compare(l).expect("compared") == Ordering::Less);
            if lower {
                least = Some(sum.clone());
            }
        }
        (sum, least)
    }

    /// A node's sum and least running sum as quantities.
    fn plain(node: &Node, minor: Option<&Num>) -> (Num, Option<Num>) {
        let least = node
            .least
            .as_ref()
            .map(|l| l.quantity(minor).expect("a quantity"));
        (node.sum.quantity(minor).expect("a quantity"), least)
    }

    #[test]
    fn the_tree_folds_each_range_as_one_by_one() {
        for steps in steps(20_261_109) {
            for minor in kinds() {
                let sums = Sums::of(figures(steps.clone(), minor.as_ref())).expect("a tree");
                for from in 0..=steps.len() {
                    folds_from(&sums, &steps, (from, minor.as_ref()));
                }
            }
        }
    }

    /// Each range of `sums` from position `from` against the positions folded one by one.
    fn folds_from(sums: &Sums, steps: &[Num], (from, minor): (usize, Option<&Num>)) {
        for to in from..=steps.len() {
            let got = sums.over(from, to).expect("a node");
            assert_eq!(
                plain(&got, minor),
                folded(steps, from, to),
                "{from}..{to} of {}, {minor:?}",
                steps.len()
            );
        }
    }
}
