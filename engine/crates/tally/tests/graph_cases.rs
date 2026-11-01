//! The builder's item graph tests: a small reality loads to refcore's edges, marks, decisions' reach, walk-bound defs, left-out moves and strata, and the graph's refusals print refcore's text.
//! Spec: E §3.1 steps 5–8; L §5.11 (a same-key cycle, a recurrence's shifted read),
//! §5.14.1 (decisions, running reads, the projection and what it leaves out), §5.14.2 (the
//! order within a day: `after`, `before`, cycles), §8.5 (`WARN-UNREAD`).
//! Twin: engine/tools/refcore/check.py and check3.py; each expected line is refcore's graph
//! of the same text, printed in the same normalized form.
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally::Source;
    use tally::load::load;
    use tally::model::{Node, NodeKind, Reality, Target};
    use tally::syntax::{Stop, parse_reality};

    /// A recurrence, an aggregate, a projection a decision reaches, a plan, three decisions
    /// ordered by a written `after` and a read of a decision's legs, `next`, and an input
    /// and a def nothing reads.
    const REALITY: &str = "model g
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"a card\"

unit PLN: currency minor 0.01

kind ext: place PLN signed outside

index month = month 2026-01..2026-03

place konto: accounts PLN lives 2026-01-01..

place save: accounts PLN lives 2026-01-01..

place shop: ext lives 2026-01-01..

input pay[m in month]: money PLN

input pay_day[m in month]: date

input floor: money PLN

input spare: money PLN

def running[m in month]: money PLN
  rule first = pay[m] when pos(m) == 1
  rule later = running[m - 1] + pay[m] when pos(m) > 1

def total: money PLN = sum(pay[m] for m in month)

def unused: money PLN = sum(running[m] for m in month)

def room[m in month]: money PLN = bal(konto, pay_day[m]) from pay_day[m] - floor

move salary[m in month]: transfers
  on pay_day[m]
  leg cash from shop to konto amount pay[m]

move save_it[m in month]: transfers
  on pay_day[m]
  reads konto
  after sweep
  leg cash from konto to save amount max(0, room[m])

move sweep[m in month]: transfers
  on pay_day[m]
  reads save
  leg cash from save to konto amount max(0, bal(save) - floor)

move top[m in month]: transfers
  on pay_day[m]
  reads save_it
  leg cash from shop to konto amount save_it[m].cash.amount

version facts
  cite card
  governs always
  known 2026-01-01
  pay[*] = 1000.00
  pay_day[2026-01] = 2026-01-10
  pay_day[2026-02] = 2026-02-10
  pay_day[2026-03] = 2026-03-10
  floor = 100.00
  opening[konto] = 50.00 at 2026-01-01 start

ask total_paid: total, flows(salary, 2026-01-01 .. 2026-03-31)

ask ahead: bal(konto, 2026-03-31) from 2026-02-01, next(transfers, 2026-01-05)
";

    /// The places the refusal cases share.
    const HEAD: &str = "model g
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

unit PLN: currency minor 0.01

kind ext: place PLN signed outside

index month = month 2026-01..2026-03

place konto: accounts PLN lives 2026-01-01..

place save: accounts PLN lives 2026-01-01..

place shop: ext lives 2026-01-01..
";

    /// The load of one file `a.tally` holding `text`.
    fn load_one(text: &str) -> Result<Reality, Stop> {
        let files = parse_reality(&[Source {
            path: "a.tally".to_owned(),
            text: text.to_owned(),
        }])?;
        load(files)
    }

    /// The refusal `HEAD` and `tail` meet, as the run prints it.
    fn refusal(tail: &str) -> String {
        match load_one(&format!("{HEAD}{tail}")) {
            Err(Stop::Refused(r)) => r.text(),
            Err(Stop::NotBuilt(what)) => format!("not built: {what}"),
            Ok(_) => "loaded".to_owned(),
        }
    }

    /// A node as refcore's probe prints it.
    fn node(n: &Node) -> String {
        match n.kind {
            NodeKind::Balances => "balances".to_owned(),
            NodeKind::Price => "price *".to_owned(),
            NodeKind::Input
            | NodeKind::Def
            | NodeKind::Band
            | NodeKind::Move
            | NodeKind::MoveOn
            | NodeKind::Place
            | NodeKind::Ask => n.text(),
        }
    }

    /// The reality's graph facts, one per line, sorted, in refcore's form (`refgraph.py`).
    fn facts(reality: &Reality) -> Vec<String> {
        let graph = &reality.graph;
        let mut lines = BTreeSet::new();
        for (reader, reads) in &graph.edges {
            for (target, rels) in reads {
                let mut words: Vec<String> = rels.iter().map(tally::model::Rel::text).collect();
                words.sort();
                let (from, to) = (node(reader), node(target));
                lines.insert(format!("edge {from} -> {to} : {}", words.join(" ")));
            }
        }
        for (item, marks) in &graph.marks {
            for (mark, target) in marks {
                let (holder, word, what) = (node(item), mark.word(), target.text());
                lines.insert(format!("mark {holder} : {word} {what}"));
            }
        }
        for item in &graph.balnow {
            lines.insert(format!("balnow {}", node(item)));
        }
        for (holder, moves) in &graph.left_out {
            let ids: Vec<&str> = moves.iter().map(String::as_str).collect();
            lines.insert(format!("leftout {} : {}", node(holder), ids.join(" ")));
        }
        for item in &graph.from_decisions {
            lines.insert(format!("fromdec {}", node(item)));
        }
        for def in &graph.walk_bound {
            lines.insert(format!("walkbound {def}"));
        }
        for (decision, firsts) in &graph.after {
            let ids: Vec<&str> = firsts.iter().map(String::as_str).collect();
            lines.insert(format!("after {decision} : {}", ids.join(" ")));
        }
        for group in &graph.order_groups {
            let ids: Vec<&str> = group.iter().map(String::as_str).collect();
            lines.insert(format!("group {}", ids.join(" ")));
        }
        for (later, earlier) in &graph.written_order {
            lines.insert(format!("written {later} {earlier}"));
        }
        for (id, mv) in reality.moves.entries() {
            let reads: Vec<String> = mv.read_targets.iter().map(Target::text).collect();
            let mut changes: Vec<String> = mv.changes.iter().map(Target::text).collect();
            changes.sort();
            let decision = if mv.decision { "True" } else { "False" };
            let reads_own = if mv.self_reads { "True" } else { "False" };
            lines.insert(format!(
                "move {id} : decision {decision} stratum {} self {reads_own} reads [{}] changes [{}]",
                mv.stratum,
                reads.join(" "),
                changes.join(" ")
            ));
        }
        for (code, text) in &reality.warnings {
            lines.insert(format!("warning {} {text}", code.as_str()));
        }
        lines.into_iter().collect()
    }

    #[test]
    fn the_graph_is_refcores() {
        let m = load_one(REALITY).expect("the reality loads");
        let expected = [
            "after save_it : sweep",
            "after sweep : ",
            "after top : save_it",
            "balnow move sweep",
            "edge ask ahead -> balances : bal",
            "edge ask ahead -> move salary : next",
            "edge ask ahead -> move save_it : next",
            "edge ask ahead -> move sweep : next",
            "edge ask ahead -> move top : next",
            "edge ask ahead -> place konto : any",
            "edge ask total_paid -> def total : any",
            "edge ask total_paid -> move salary : flows",
            "edge def room -> balances : bal",
            "edge def room -> input floor : same",
            "edge def room -> input pay_day : same",
            "edge def room -> place konto : any",
            "edge def running -> def running : shift:m:-",
            "edge def running -> input pay : same",
            "edge def total -> input pay : computed",
            "edge def unused -> def running : computed",
            "edge move salary -> input pay : any",
            "edge move salary -> moveon salary : any",
            "edge move salary -> place konto : any",
            "edge move salary -> place shop : any",
            "edge move save_it -> def room : any",
            "edge move save_it -> moveon save_it : any",
            "edge move save_it -> place konto : any",
            "edge move save_it -> place save : any",
            "edge move sweep -> balances : bal",
            "edge move sweep -> input floor : any",
            "edge move sweep -> moveon sweep : any",
            "edge move sweep -> place konto : any",
            "edge move sweep -> place save : any",
            "edge move top -> move save_it : any",
            "edge move top -> moveon top : any",
            "edge move top -> place konto : any",
            "edge move top -> place shop : any",
            "edge moveon salary -> input pay_day : any",
            "edge moveon save_it -> input pay_day : any",
            "edge moveon sweep -> input pay_day : any",
            "edge moveon top -> input pay_day : any",
            "fromdec def room",
            "fromdec move save_it",
            "fromdec move sweep",
            "fromdec move top",
            "fromdec moveon save_it",
            "fromdec moveon sweep",
            "fromdec moveon top",
            "leftout ask ahead : ",
            "leftout def room : save_it top",
            "mark ask ahead : proj thing konto",
            "mark def room : proj thing konto",
            "mark move sweep : balnow thing save",
            "mark move top : legs move save_it",
            "move salary : decision False stratum 0 self False reads [] changes [thing konto thing shop]",
            "move save_it : decision True stratum 2 self True reads [thing konto] changes [thing konto thing save]",
            "move sweep : decision True stratum 1 self True reads [thing save] changes [thing konto thing save]",
            "move top : decision True stratum 3 self False reads [move save_it] changes [thing konto thing shop]",
            "walkbound room",
            "warning WARN-UNREAD def unused is read by nothing",
            "warning WARN-UNREAD input spare is read by nothing",
            "written save_it sweep",
        ];
        assert_eq!(facts(&m), expected, "refcore's graph of the same text");
    }

    #[test]
    fn a_projection_leaves_out_what_depends_on_its_holder() {
        let m = load_one(REALITY).expect("the reality loads");
        let room = Node::new(NodeKind::Def, "room");
        let left: Vec<&str> = m
            .graph
            .left_out_of(&room)
            .map(|moves| moves.iter().map(String::as_str).collect())
            .expect("room holds a projection");
        assert_eq!(
            left,
            ["save_it", "top"],
            "save_it reads room; top reads save_it"
        );
        let ask = Node::new(NodeKind::Ask, "ahead");
        assert_eq!(
            m.graph.left_out_of(&ask).map(BTreeSet::len),
            Some(0),
            "an ask's projection leaves out none"
        );
    }

    #[test]
    fn the_graph_refuses_as_refcore_does() {
        let cases = [
            (
                "\ndef a[m in month]: money PLN = b[m]\n\ndef b[m in month]: money PLN = a[m]\n",
                "LOAD-CYCLE a depends on itself at the same key: a -> b -> a",
            ),
            (
                "\nmove d1: transfers\n  on cut\n  reads konto\n  leg cash from save to konto amount bal(konto)\n\nmove d2: transfers\n  on cut\n  reads save\n  leg cash from konto to save amount bal(save)\n",
                "LOAD-ORDER decisions d1 and d2 read each other with no before or after (a.tally:19)",
            ),
            (
                "\nmove salary: transfers\n  on pay_day\n  leg cash from shop to konto amount pay\n\nmove d1: transfers\n  on pay_day\n  reads konto\n  before salary\n  leg cash from konto to save amount bal(konto)\n\ninput pay_day: date\n\ninput pay: money PLN\n",
                "LOAD-ORDER d1 before salary: salary is a plan or an observation (a.tally:23)",
            ),
            (
                "\nmove d1: transfers\n  on pay_day\n  reads konto\n  leg cash from konto to save amount bal(konto)\n\nmove d2: transfers\n  on pay_day\n  reads d1\n  before d1\n  leg cash from shop to konto amount d1.cash.amount\n\ninput pay_day: date\n\ninput pay: money PLN\n",
                "LOAD-ORDER d2 before d1 contradicts a read of d1's legs (a.tally:24)",
            ),
            (
                "\nmove d1: transfers\n  on pay_day\n  reads konto\n  after nope\n  leg cash from konto to save amount bal(konto)\n\ninput pay_day: date\n\ninput pay: money PLN\n",
                "LOAD-NAME unknown move nope (after nope) (a.tally:19)",
            ),
        ];
        for (tail, expected) in cases {
            assert_eq!(
                refusal(tail),
                expected,
                "refcore's refusal of the same text"
            );
        }
    }
}
