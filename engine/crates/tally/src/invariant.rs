//! E §9 invariant 4, asks move no money: each main walk's signature is kept once it is walked, and checked again after each ask.
//! Spec: E §9 item 4 ("Asks move no money. The timeline's hash is the same before and after
//! the asks are answered"); E §9's opening (a failure refuses with `ENGINE-INVARIANT`, never an
//! answer). Items 1 and 2 are checked at each day's close (`walk/day.rs`), item 5 where a plan
//! instance is booked (`walk/plan.rs`); item 6 holds by the walk's `RUN-LIFESPAN` refusals
//! (`walk/day.rs`); item 3 is scenarios', not built at S.
//! Twin: engine/tools/refcore/walk.py (`Walk.signature`; `m_main_walk` keeps `w.sig` once the
//! walk is done) and ask.py (`Asker._answer`'s last step, its text).
//! Invariants: a signature is every entry of every place's log, (place, unit) in text order and
//! each log in walk order, with the entry's point, change and movement; it is taken once, when
//! a main walk is done; a projection's walk and a failed walk have none checked.

use std::collections::BTreeMap;

use crate::base::{Code, Refusal};
use crate::eval::Engine;
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{At, Change, Progress, Walk};

/// One booked entry as the signature holds it: its point, what it does, its movement.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    /// The entry's point.
    at: At,
    /// What it does to the place's state.
    change: Change,
    /// The movement it books, by id.
    mv: Option<String>,
}

/// A main walk's signature (refcore's `Walk.signature`): each (place, unit)'s booked entries,
/// in walk order. E §9 item 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Signature {
    /// The entries of each (place, unit), the keys in text order.
    logs: BTreeMap<(String, String), Vec<Line>>,
}

impl Signature {
    /// The signature of `walk` as it stands: every entry of every place's log. E §9 item 4.
    fn of(walk: &Walk) -> Signature {
        let mut logs = BTreeMap::new();
        for (key, log) in &walk.logs {
            let lines = log
                .ents
                .iter()
                .map(|e| Line {
                    at: e.at,
                    change: e.change.clone(),
                    mv: e.mv.clone(),
                })
                .collect();
            logs.insert(key.clone(), lines);
        }
        Signature { logs }
    }
}

impl Engine {
    /// Keeps the signature of walk `id` once it is done, when it is a main walk (refcore's
    /// `m_main_walk`, `w.sig`): a projection's walk is a read, not the timeline. E §9 item 4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn keep_signature(&mut self, id: usize) -> Result<(), Stop> {
        let walk = self.walk(id)?;
        if walk.proj.is_some() {
            return Ok(());
        }
        let signature = Signature::of(walk);
        self.signatures.insert(id, signature);
        Ok(())
    }

    /// Asks move no money: every main walk this engine walked, unless it failed, has the
    /// signature it had when it was done (refcore's `Asker._answer`, its last step). E §9 item 4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT asks move no money: …` for the first walk whose entries changed, by
    /// the order the walks were made.
    pub(crate) fn asks_moved_no_money(&self) -> Result<(), Stop> {
        for (id, kept) in &self.signatures {
            let walk = self.walk(*id)?;
            if matches!(walk.progress, Progress::Failed(_)) {
                continue;
            }
            if Signature::of(walk) != *kept {
                return Err(moved(walk.cut));
            }
        }
        Ok(())
    }
}

/// The refusal of a timeline that changed while an ask was answered, as refcore words it.
/// E §9 item 4.
fn moved(cut: Day) -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        format!(
            "asks move no money: the timeline at cut {} changed while the ask was answered (E §9 invariant 4)",
            cut.text()
        ),
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::Days;
    use crate::api::Source;
    use crate::eval::Engine;
    use crate::load::load;
    use crate::model::Reality;
    use crate::num::Num;
    use crate::syntax::{Stop, parse_reality};
    use crate::walk::{At, Change, Ent, Projected};

    /// A rent and a wage between two places, asked once.
    const REALITY: &str = r#"model sg
  tally 2
  on 2025-02-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

input pay: money DUK

input rent_day: date

input wage_day: date

move rent: transfers
  on rent_day
  leg cash from bank to out amount pay

move wage: transfers
  on wage_day
  leg cash from out to bank amount pay

version facts
  cite card
  governs always
  known 2025-01-01
  pay = 100.00
  rent_day = 2025-01-20
  wage_day = 2025-03-05
  opening[bank] = 1000.00 at 2025-01-01 start

ask bank_end: bal(bank, 2025-03-31)
"#;

    fn reality() -> Reality {
        let files = parse_reality(&[Source {
            path: "sg.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("parses");
        load(files).expect("loads")
    }

    #[test]
    fn a_walk_whose_entries_change_after_it_is_done_refuses_the_ask() {
        let m = reality();
        let mut engine = Engine::new(m.on).with_days(Days::Events);
        let id = engine.main_walk(&m, m.on).expect("walks");
        assert!(engine.signatures.contains_key(&id), "kept once walked");
        assert_eq!(engine.asks_moved_no_money(), Ok(()), "nothing moved");
        let again = engine.main_walk(&m, m.on).expect("walks");
        assert_eq!(again, id, "the same walk");
        assert_eq!(engine.asks_moved_no_money(), Ok(()), "a read moves nothing");
        let walk = engine.walk_mut(id).expect("the walk");
        let log = walk
            .logs
            .get_mut(&("bank".to_owned(), "DUK".to_owned()))
            .expect("bank's log");
        let first = log.ents.first().expect("an entry").at;
        log.ents
            .push(Ent::new(At::end(first.day), Change::Delta(Num::int(1))));
        let Err(Stop::Refused(r)) = engine.asks_moved_no_money() else {
            panic!("a changed timeline is refused");
        };
        assert_eq!(
            r.text(),
            "ENGINE-INVARIANT asks move no money: the timeline at cut 2025-02-10 changed while \
             the ask was answered (E §9 invariant 4)",
            "refcore's text"
        );
    }

    #[test]
    fn a_projection_keeps_no_signature() {
        let m = reality();
        let mut engine = Engine::new(m.on).with_days(Days::Events);
        let id = engine.main_walk(&m, m.on).expect("walks");
        let mut walk = engine.walk(id).expect("the walk").clone();
        walk.proj = Some(Box::new(Projected {
            base: id,
            point: At::start(m.start),
            left_out: BTreeSet::new(),
            counted: BTreeMap::new(),
            levels: Vec::new(),
            root: (m.on, m.start),
        }));
        let projected = engine.walks.len();
        engine.walks.push(walk);
        engine.keep_signature(projected).expect("kept or not");
        assert!(
            !engine.signatures.contains_key(&projected),
            "a projection's walk is a read"
        );
    }
}
