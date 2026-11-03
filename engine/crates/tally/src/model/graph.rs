//! The item graph: who reads what, the running reads, the projections, and the order of decisions within a day.
//! Spec: E §3.1 steps 5–8 (the item graph, its checks, the classes of moves, the strata);
//! L §5.14.1 (plans and decisions, running reads, the written projection), §5.14.2 (order
//! within a day).
//! Twin: engine/tools/refcore/check.py (`Checker.edges`, `marks`, `balnow_items`, `projs`);
//! check3.py (`m_check_moves_graph`, `m_strata`, `_meets`).
//! Invariants: a node orders by its kind's word, then its id, as refcore's tuples sort, never
//! by a derived order (R-47); an edge's relations and an item's marks hold each value once;
//! what a projection leaves out depends only on the item that holds it.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// What a node of the graph is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// An input.
    Input,
    /// A def.
    Def,
    /// A band.
    Band,
    /// A move: its legs, amounts, `when` and attributes.
    Move,
    /// A move's day: its `on` and `at`.
    MoveOn,
    /// A place section.
    Place,
    /// An ask.
    Ask,
    /// A price clause, read where a place is valued.
    Price,
    /// The timeline's balances, which every move books into.
    Balances,
}

impl NodeKind {
    /// The kind's word, which orders nodes (refcore's tuple heads).
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            NodeKind::Input => "input",
            NodeKind::Def => "def",
            NodeKind::Band => "band",
            NodeKind::Move => "move",
            NodeKind::MoveOn => "moveon",
            NodeKind::Place => "place",
            NodeKind::Ask => "ask",
            NodeKind::Price => "price",
            NodeKind::Balances => "balances",
        }
    }
}

/// A node of the item graph: a kind and an id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// What it is.
    pub kind: NodeKind,
    /// Its id (empty for the balances).
    pub id: String,
}

impl Node {
    /// The node of `kind` named `id`.
    #[must_use]
    pub fn new(kind: NodeKind, id: &str) -> Node {
        Node {
            kind,
            id: id.to_owned(),
        }
    }

    /// The timeline's balances.
    #[must_use]
    pub fn balances() -> Node {
        Node::new(NodeKind::Balances, "")
    }

    /// The node as the builder's tests print it: its kind's word and its id.
    #[must_use]
    pub fn text(&self) -> String {
        format!("{} {}", self.kind.word(), self.id)
    }
}

impl Ord for Node {
    /// By the kind's word, then the id, both by code point: refcore's `sorted` on its
    /// `(kind, id)` tuples, which orders its cycle messages.
    fn cmp(&self, other: &Node) -> Ordering {
        (self.kind.word(), self.id.as_str()).cmp(&(other.kind.word(), other.id.as_str()))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Which way a shifted key reads: `k - n` or `k + n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
    /// `k - n`, an earlier key.
    Earlier,
    /// `k + n`, a later key.
    Later,
}

/// How a read's keys relate to its reader's (E §3.1 step 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rel {
    /// At the reader's own keys.
    Same,
    /// At a key variable of the reader shifted by a number.
    Shifted(String, Shift),
    /// At a computed key.
    Computed,
    /// From a reader with no keys of its own to relate (a move, a band, an ask).
    Any,
    /// A binding over a kind or an observed input's rows.
    Bind,
    /// An observed input named by a binding.
    Named,
    /// `flows` of a move.
    Flows,
    /// A balance read.
    Bal,
    /// A price, or a read through `last` or `since`, at a day the reader states.
    Explicit,
    /// A calendar read at each day asked about.
    Calendar,
    /// `next(K, d)`, over the moves of K.
    Next,
    /// `observed(m[k])`, over the observed inputs.
    Observed,
}

impl Rel {
    /// The relation's word, as refcore writes it.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Rel::Same => "same".to_owned(),
            Rel::Shifted(var, Shift::Earlier) => format!("shift:{var}:-"),
            Rel::Shifted(var, Shift::Later) => format!("shift:{var}:+"),
            Rel::Computed => "computed".to_owned(),
            Rel::Any => "any".to_owned(),
            Rel::Bind => "bind".to_owned(),
            Rel::Named => "named".to_owned(),
            Rel::Flows => "flows".to_owned(),
            Rel::Bal => "bal".to_owned(),
            Rel::Explicit => "explicit".to_owned(),
            Rel::Calendar => "calendar".to_owned(),
            Rel::Next => "next".to_owned(),
            Rel::Observed => "observed".to_owned(),
        }
    }
}

/// A running read or a projection, the mark E §3.1 step 7 propagates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// `bal(x)`, with no day.
    BalNow,
    /// `bal(x, d)`, or `value(x, d)`.
    Bal,
    /// `bal(x, d) from t`.
    Proj,
    /// `flows` of a place or a decision.
    Flows,
    /// A decision's legs, day or attributes, or a binding over its movements or over `pads`.
    Legs,
    /// A leg's amount read of a party or batch.
    LegOf,
}

impl Mark {
    /// The mark's word, as refcore writes it.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Mark::BalNow => "balnow",
            Mark::Bal => "bal",
            Mark::Proj => "proj",
            Mark::Flows => "flows",
            Mark::Legs => "legs",
            Mark::LegOf => "legof",
        }
    }
}

/// What a running read, a declared read or a leg names: a place, all of a keyed place, a
/// move, a kind, or a place load can't see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// One place, by its thing's name.
    Thing(String),
    /// Every instance of a keyed place.
    PlaceSec(String),
    /// A move's movements.
    Move(String),
    /// The things or movements of a kind.
    Kind(String),
    /// A place given by an expression load doesn't see.
    Unknown,
}

impl Target {
    /// The target as refcore writes it.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Target::Thing(name) => format!("thing {name}"),
            Target::PlaceSec(id) => format!("placesec {id}"),
            Target::Move(id) => format!("move {id}"),
            Target::Kind(id) => format!("kind {id}"),
            Target::Unknown => "unknown".to_owned(),
        }
    }
}

/// The item graph and what load decides from it (refcore's `Checker` graph and the
/// reality's `from_decisions`, `walk_bound`, `after`, `order_groups`, `written_order`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
    /// Each reader's reads, with their relations.
    pub edges: BTreeMap<Node, BTreeMap<Node, Vec<Rel>>>,
    /// Each item's own running reads and projections, with what they read.
    pub marks: BTreeMap<Node, Vec<(Mark, Target)>>,
    /// The items that hold `bal(x)` with no day.
    pub balnow: BTreeSet<Node>,
    /// The items that hold a written projection.
    pub holders: BTreeSet<Node>,
    /// Each holder's left-out moves: those that depend on it (L §5.14.1).
    pub left_out: BTreeMap<Node, BTreeSet<String>>,
    /// The decisions and the defs and days they reach.
    pub from_decisions: BTreeSet<Node>,
    /// The walk-bound defs: they reach a mark and a decision reaches them (E §3.1 step 7).
    pub walk_bound: BTreeSet<String>,
    /// Each decision's decisions to act after (E §3.1 step 8).
    pub after: BTreeMap<String, BTreeSet<String>>,
    /// The cycles of decisions no written order breaks, each one stratum, ordered at run.
    pub order_groups: Vec<BTreeSet<String>>,
    /// The orders `before` and `after` write: `(a, b)` is a after b.
    pub written_order: BTreeSet<(String, String)>,
}

impl Graph {
    /// Adds the edge `reader` → `target` with `rel` (refcore's `Checker.edge`).
    pub fn edge(&mut self, reader: &Node, target: Node, rel: Rel) {
        let reads = self
            .edges
            .entry(reader.clone())
            .or_insert_with(BTreeMap::new);
        let rels = reads.entry(target).or_insert_with(Vec::new);
        if !rels.contains(&rel) {
            rels.push(rel);
        }
    }

    /// Marks `item` with a running read or a projection of `target` (refcore's `mark`).
    pub fn mark(&mut self, item: &Node, mark: Mark, target: Target) {
        let marks = self.marks.entry(item.clone()).or_insert_with(Vec::new);
        let pair = (mark, target);
        if !marks.contains(&pair) {
            marks.push(pair);
        }
    }

    /// What `reader` reads, with the relations.
    #[must_use]
    pub fn reads(&self, reader: &Node) -> Option<&BTreeMap<Node, Vec<Rel>>> {
        self.edges.get(reader)
    }

    /// Whether `from` reaches `to` along the edges (refcore's `reaches`).
    #[must_use]
    pub fn reaches(&self, from: &Node, to: &Node) -> bool {
        let mut seen = BTreeSet::new();
        let mut stack = vec![from.clone()];
        while let Some(next) = stack.pop() {
            if next == *to {
                return true;
            }
            if let Some(reads) = self.edges.get(&next)
                && seen.insert(next)
            {
                stack.extend(reads.keys().cloned());
            }
        }
        false
    }

    /// The moves a projection held by `holder` leaves out, if it holds one.
    #[must_use]
    pub fn left_out_of(&self, holder: &Node) -> Option<&BTreeSet<String>> {
        self.left_out.get(holder)
    }
}

#[cfg(test)]
mod tests {
    use super::{Graph, Mark, Node, NodeKind, Rel, Shift, Target};

    #[test]
    fn nodes_order_by_their_kind_word_then_id() {
        let mut nodes = [
            Node::new(NodeKind::Def, "b"),
            Node::new(NodeKind::Band, "z"),
            Node::new(NodeKind::Def, "a"),
            Node::new(NodeKind::MoveOn, "a"),
            Node::new(NodeKind::Move, "a"),
        ];
        nodes.sort();
        let texts: Vec<String> = nodes.iter().map(Node::text).collect();
        assert_eq!(
            texts,
            ["band z", "def a", "def b", "move a", "moveon a"],
            "refcore's tuple order"
        );
    }

    #[test]
    fn an_edge_keeps_each_relation_once() {
        let mut graph = Graph::default();
        let reader = Node::new(NodeKind::Def, "d");
        graph.edge(&reader, Node::new(NodeKind::Input, "x"), Rel::Same);
        graph.edge(&reader, Node::new(NodeKind::Input, "x"), Rel::Same);
        graph.edge(
            &reader,
            Node::new(NodeKind::Input, "x"),
            Rel::Shifted("k".to_owned(), Shift::Earlier),
        );
        let input = Node::new(NodeKind::Input, "x");
        let rels = graph.reads(&reader).and_then(|reads| reads.get(&input));
        let texts: Option<Vec<String>> = rels.map(|rels| rels.iter().map(Rel::text).collect());
        assert_eq!(
            texts,
            Some(vec!["same".to_owned(), "shift:k:-".to_owned()]),
            "one of each"
        );
        graph.mark(&reader, Mark::Bal, Target::Thing("konto".to_owned()));
        graph.mark(&reader, Mark::Bal, Target::Thing("konto".to_owned()));
        assert_eq!(graph.marks.get(&reader).map(Vec::len), Some(1), "one mark");
    }

    #[test]
    fn reaches_follows_every_edge() {
        let mut graph = Graph::default();
        let a = Node::new(NodeKind::Move, "a");
        let b = Node::new(NodeKind::Def, "b");
        let c = Node::new(NodeKind::Input, "c");
        graph.edge(&a, b.clone(), Rel::Any);
        graph.edge(&b, c.clone(), Rel::Same);
        assert!(graph.reaches(&a, &c), "a reads b, b reads c");
        assert!(!graph.reaches(&c, &a), "c reads nothing");
    }
}
