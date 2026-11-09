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
//! instances at the read's cut, holding for every cut that sees no more than that one, and
//! updated in place at a cut that does; a read the projection's walk refuses on a day it
//! reaches is the walk's (pick P54, #127).
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
//! sum after it, if below zero: two queries of the tree, each O(log n). F changes only on the
//! days a cut starts to see more (a version's `known`, a row's own): a tree built at cut c
//! holds for every later cut before the next such day, and at a cut past it the plan pass's
//! instances are gathered again and each position whose sum changed is updated in place, O(log
//! n) each (P §2.3's range add from a day on is one position's change here); the tree is built
//! again only where x's days change or the cut goes back.

use core::ops::Bound;
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
    /// The days a cut starts to see more on, once a tree needs them.
    known: Option<BTreeSet<Day>>,
}

impl Trees {
    /// The first day after `cut` on which a cut sees more than `cut` does; none past the
    /// last. P §2.3.
    fn next_known(&mut self, m: &Reality, cut: Day) -> Option<Day> {
        self.known
            .get_or_insert_with(|| knowledge_days(m))
            .range((Bound::Excluded(cut), Bound::Unbounded))
            .next()
            .copied()
    }
}

/// One place's running sums in one unit at one cut, and what keeps a read from them (E §4.3,
/// P §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Tree {
    /// The cut min(t, K) its instances are read at.
    cut: Day,
    /// The first day after the cut on which a cut sees more: every cut from `cut` up to it
    /// gives the instances it counts the same, so the tree holds there too; none past the
    /// last such day.
    upto: Option<Day>,
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
            upto: None,
            open: true,
            days: Vec::new(),
            sums: Sums {
                width: 1,
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

    /// Whether the tree holds at cut `cut`: one at or after its own and before the next day a
    /// cut sees more on, so the plan pass gives the instances a read counts the same (P §2.3:
    /// the cut never goes back during a walk, so a cut before its own builds it again).
    fn holds_at(&self, cut: Day) -> bool {
        self.cut <= cut && self.upto.is_none_or(|next| cut < next)
    }

    /// The number of positions on or before `day`. P §2.3.
    fn through(&self, day: Day) -> usize {
        self.days.partition_point(|d| *d <= day)
    }

    /// The legs counted from day `counts` through `first`, then the least running sum of the
    /// days after it through `last`, if below zero: the lowest end-of-day balance of the days
    /// `first` through `last`, less the start (P §2.3: two queries of the tree). E §4.3.
    fn lowest_change(&self, counts: Day, (first, last): (Day, Day)) -> Result<Num, Stop> {
        let before = self.days.partition_point(|d| *d < counts);
        let (from, to) = (self.through(first), self.through(last));
        let head = self.sums.over(before, from)?;
        let tail = self.sums.over(from, to)?;
        let dip = match tail.least {
            Some(l) if l.is_negative() => l,
            Some(_) | None => Num::zero(),
        };
        head.sum.checked_add(&dip).map_err(num_stop)
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

/// One node of the tree: the sum of its positions' legs and the least running sum inside
/// them, none for no position (P §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    /// The sum.
    sum: Num,
    /// The least running sum, from its first position.
    least: Option<Num>,
}

impl Node {
    /// No position: the sum zero, no running sum. P §2.3.
    fn empty() -> Node {
        Node {
            sum: Num::zero(),
            least: None,
        }
    }

    /// One position whose legs sum to `q`. P §2.3.
    fn day(q: Num) -> Node {
        Node {
            least: Some(q.clone()),
            sum: q,
        }
    }

    /// This node's positions, then `right`'s: the sums added; the least running sum this
    /// one's, or its sum plus the right's least (P §2.3).
    fn then(&self, right: &Node) -> Result<Node, Stop> {
        let sum = self.sum.checked_add(&right.sum).map_err(num_stop)?;
        let shifted = match &right.least {
            Some(l) => Some(self.sum.checked_add(l).map_err(num_stop)?),
            None => None,
        };
        let least = match (self.least.clone(), shifted) {
            (Some(a), Some(b)) => Some(match b.compare(&a).map_err(num_stop)? {
                Ordering::Less => b,
                Ordering::Equal | Ordering::Greater => a,
            }),
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
    /// The nodes: 1 the root, `2i` and `2i + 1` the two below `i`; 0 unused.
    nodes: Vec<Node>,
}

impl Sums {
    /// The tree over the positions' sums `steps`, built from the leaves up, O(n). P §2.3.
    fn of(steps: Vec<Num>) -> Result<Sums, Stop> {
        let width = steps
            .len()
            .max(1)
            .checked_next_power_of_two()
            .ok_or_else(|| invariant("a tree past the positions there can be"))?;
        let size = width
            .checked_mul(2)
            .ok_or_else(|| invariant("a tree past the positions there can be"))?;
        let mut sums = Sums {
            width,
            nodes: vec![Node::empty(); size],
        };
        for (i, q) in steps.into_iter().enumerate() {
            let at = sums.leaf(i)?;
            *sums.node_mut(at)? = Node::day(q);
        }
        for i in (1..width).rev() {
            sums.join(i)?;
        }
        Ok(sums)
    }

    /// The tree with each position whose sum `steps` changes set to it and the nodes above
    /// it joined again: P §2.3's range add on F from that position on, in place, O(log n)
    /// each. `steps` holds a sum for each of the tree's positions.
    fn updated(mut self, steps: Vec<Num>) -> Result<Sums, Stop> {
        for (i, q) in steps.into_iter().enumerate() {
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
        let (mut left, mut right) = (Node::empty(), Node::empty());
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

/// x's counted legs and what keeps a read from them, as a tree's build gathers them.
struct Gathered {
    /// The tree, its sums not yet made.
    tree: Tree,
    /// Each counted leg's end at x: its day, its signed quantity, its register.
    legs: Vec<(Day, Num, Collected)>,
    /// Each move with instances outside the run: how many, and the first day (refcore's
    /// `note_leftout`).
    outside: BTreeMap<String, (u32, Option<Day>)>,
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

    /// The tree of `key` at cut `cut`, built from the cut's instances unless it is already,
    /// its reads kept apart from the reading cell's as the projection's walk keeps its plan
    /// pass's (E §3.2, §4.1). P §2.3.
    fn tree_at(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        (cut, world): (Day, World),
        key: &TreeKey,
    ) -> Result<(), Stop> {
        if self.trees.built.get(key).is_some_and(|t| t.holds_at(cut)) {
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
        tree.upto = self.trees.next_known(m, cut);
        // P §2.3: the same days, so each changed position is one update in place
        tree.sums = match self.trees.built.remove(key) {
            Some(old) if old.days == tree.days => old.sums.updated(steps)?,
            Some(_) | None => Sums::of(steps)?,
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
                let first = first.map_or_else(|| "None".to_owned(), Day::text);
                tree.left_out.push(Entry {
                    word: "left out",
                    text: format!("{mid}, {n} instances, first {first}"),
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

/// Every day on which a cut starts to see more, and the day after it: each version's `known`
/// and its window's last day, each row's, given's and observation's own (L §4.5, §5.3, §5.19;
/// E §5). Every cut between two of them sees the same versions, rows and observations, so
/// the plan pass gives the instances counted from it the same days, amounts and statuses.
/// P §2.3.
fn knowledge_days(m: &Reality) -> BTreeSet<Day> {
    let mut days = BTreeSet::new();
    for (_, v) in m.versions.entries() {
        mark(&mut days, Some(v.known));
        mark(&mut days, v.window);
        for moved in &v.moved_by {
            mark(&mut days, Some(moved.known));
            mark(&mut days, moved.window);
        }
    }
    for (_, input) in m.inputs.entries() {
        for row in input.rows.iter().chain(&input.part_rows) {
            mark(&mut days, row.own_known);
            mark(&mut days, row.own_window);
        }
        for obs in &input.observations {
            mark(&mut days, obs.own_known);
            mark(&mut days, obs.own_window);
        }
    }
    for (_, def) in m.defs.entries() {
        for row in &def.givens {
            mark(&mut days, row.own_known);
            mark(&mut days, row.own_window);
        }
    }
    days
}

/// Notes a day a cut starts to see more on, and the day after it, whichever of `known <= cut`
/// and `known < cut` a test reads. P §2.3.
fn mark(days: &mut BTreeSet<Day>, day: Option<Day>) {
    if let Some(d) = day {
        days.insert(d);
        days.extend(d.next());
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
    use super::{Lows, Node, Sums};
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
    fn a_tree_holds_until_a_cut_sees_more_then_is_updated_in_place() {
        let m = reality();
        // from 01-20, then 02-01 (the same facts), 03-01 (past `rise`, known 02-15), and back
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
                Some(day(1, 20)),
                Some(day(3, 1)),
                Some(day(1, 20))
            ],
            "kept for 02-01, updated for 03-01, built again going back"
        );
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
            let updated = Sums::of(steps.clone())
                .expect("a tree")
                .updated(changed.clone())
                .expect("updated");
            assert_eq!(
                updated,
                Sums::of(changed).expect("a tree"),
                "{} positions",
                steps.len()
            );
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

    /// The node of positions `from..to` folded one by one: the plain twin of [`Sums::over`].
    fn folded(steps: &[Num], from: usize, to: usize) -> Node {
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
        Node { sum, least }
    }

    #[test]
    fn the_tree_folds_each_range_as_one_by_one() {
        for steps in steps(20_261_109) {
            let sums = Sums::of(steps.clone()).expect("a tree");
            for from in 0..=steps.len() {
                for to in from..=steps.len() {
                    let got = sums.over(from, to).expect("a node");
                    assert_eq!(
                        got,
                        folded(&steps, from, to),
                        "{from}..{to} of {}",
                        steps.len()
                    );
                }
            }
        }
    }
}
