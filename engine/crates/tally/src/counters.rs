//! The work counters behind `--stats`: what one run did, in the units of P §1.1, counted in the `counters` build only.
//! Spec: P §1.1 (the units: ev, def cells hit and missed, reads and distinct reads, walk days
//! and non-empty days, legs booked, parts groups, walks shared, cells recomputed locally, `Num`
//! results), §8.2 (the count budgets read from them); BUILDER §13.2 (D4-8, K8-15).
//! Twin: `engine/tools/refcore_fast`, refcore's counted run, whose units `shape.md` reports.
//! Invariants: the module exists only in the `counters` build, which refuses to compile
//! without debug assertions (R-51); counting never changes what a run prints (gate 12); a
//! count that would pass `u64` prints `overflow`, never a clamped figure (R-23).
//!
//! Parts groups, walks shared and cells recomputed locally stay 0 at S: no place keeps parts,
//! no timeline forks and no worker runs (BUILD-ORDER § S).

use std::collections::{BTreeMap, BTreeSet};

/// One unit of P §1.1 that a run adds to one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unit {
    /// An expression node evaluated (ev).
    Ev,
    /// A def cell read that found the memo's value.
    DefHit,
    /// A def cell whose computation unwound past the counted depth, to be computed again from
    /// the work stack (R-16): work a miss pays for twice.
    DefUnwound,
    /// A day the day walk stepped through that held no act (a walk day).
    Day,
    /// A stepped day with at least one act: a walk day and a non-empty day.
    NonEmptyDay,
    /// A walk made: a cut's main walk, or a projection's own (E §3.3, §4.3).
    Walk,
    /// A movement leg written to a place's log.
    Leg,
    /// An exact value an expression node made: a number or money.
    NumResult,
}

/// One count: a whole number, or past `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Count(Option<u64>);

impl Count {
    /// Nothing counted yet.
    const ZERO: Count = Count(Some(0));

    /// One more.
    fn bump(&mut self) {
        self.0 = self.0.and_then(|n| n.checked_add(1));
    }

    /// `other` more.
    fn plus(&mut self, other: Count) {
        self.0 = match (self.0, other.0) {
            (Some(a), Some(b)) => a.checked_add(b),
            _ => None,
        };
    }

    /// The count as `--stats` prints it.
    fn text(self) -> String {
        self.0
            .map_or_else(|| "overflow".to_owned(), |n| n.to_string())
    }
}

/// The counts of one run, or of one ask's engine, gathered into the run's (BUILDER §13.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counters {
    /// Expression nodes evaluated.
    ev: Count,
    /// Def cell reads that found the memo's value.
    def_hits: Count,
    /// Def cells computed and kept.
    def_misses: Count,
    /// The misses per def, by its id: where the memo's work goes (P §3.4).
    by_def: BTreeMap<String, Count>,
    /// Def cells unwound past the counted depth.
    def_unwound: Count,
    /// Input reads by the read rule.
    reads: Count,
    /// The distinct reads: each read's key, (input, keys, law date, cut), once.
    distinct: BTreeSet<String>,
    /// Days stepped.
    days_stepped: Count,
    /// Stepped days with an act.
    nonempty_days: Count,
    /// Walks made.
    walks: Count,
    /// Legs booked.
    legs_booked: Count,
    /// Exact values made.
    num_results: Count,
}

impl Default for Counters {
    fn default() -> Counters {
        Counters {
            ev: Count::ZERO,
            def_hits: Count::ZERO,
            def_misses: Count::ZERO,
            by_def: BTreeMap::new(),
            def_unwound: Count::ZERO,
            reads: Count::ZERO,
            distinct: BTreeSet::new(),
            days_stepped: Count::ZERO,
            nonempty_days: Count::ZERO,
            walks: Count::ZERO,
            legs_booked: Count::ZERO,
            num_results: Count::ZERO,
        }
    }
}

impl Counters {
    /// One more of `unit`.
    pub(crate) fn add(&mut self, unit: Unit) {
        match unit {
            Unit::Ev => self.ev.bump(),
            Unit::DefHit => self.def_hits.bump(),
            Unit::DefUnwound => self.def_unwound.bump(),
            Unit::Day => self.days_stepped.bump(),
            Unit::NonEmptyDay => self.day_with_acts(),
            Unit::Walk => self.walks.bump(),
            Unit::Leg => self.legs_booked.bump(),
            Unit::NumResult => self.num_results.bump(),
        }
    }

    /// A day stepped that held an act: it counts as a walk day too.
    fn day_with_acts(&mut self) {
        self.days_stepped.bump();
        self.nonempty_days.bump();
    }

    /// One def cell computed and kept: a miss of def `def`.
    pub(crate) fn miss(&mut self, def: &str) {
        self.def_misses.bump();
        if let Some(n) = self.by_def.get_mut(def) {
            n.bump();
        } else {
            self.by_def.insert(def.to_owned(), Count(Some(1)));
        }
    }

    /// One read by the read rule, keyed by what makes it distinct (P §1.1 "read").
    pub(crate) fn read(&mut self, key: String) {
        self.reads.bump();
        self.distinct.insert(key);
    }

    /// Adds another engine's counts: a run's counts are its asks' engines' together, a
    /// distinct read counted once across them.
    pub(crate) fn merge(&mut self, other: &Counters) {
        self.ev.plus(other.ev);
        self.def_hits.plus(other.def_hits);
        self.def_misses.plus(other.def_misses);
        for (def, n) in &other.by_def {
            self.by_def
                .entry(def.clone())
                .or_insert(Count::ZERO)
                .plus(*n);
        }
        self.def_unwound.plus(other.def_unwound);
        self.reads.plus(other.reads);
        self.distinct.extend(other.distinct.iter().cloned());
        self.days_stepped.plus(other.days_stepped);
        self.nonempty_days.plus(other.nonempty_days);
        self.walks.plus(other.walks);
        self.legs_booked.plus(other.legs_booked);
        self.num_results.plus(other.num_results);
    }

    /// The `--stats` line for standard error: `stats: ` then `key=value` pairs, the form
    /// `engine/tools/perf counts` reads.
    #[must_use]
    pub fn line(&self) -> String {
        let distinct = match u64::try_from(self.distinct.len()) {
            Ok(n) => Count(Some(n)),
            Err(_) => Count(None),
        }
        .text();
        format!(
            "stats: ev={} def_hits={} def_misses={} def_unwound={} reads={} distinct_reads={} \
             days_stepped={} nonempty_days={} walks={} legs_booked={} parts_groups=0 \
             walks_shared=0 recomputed_locally=0 num_results={}\n",
            self.ev.text(),
            self.def_hits.text(),
            self.def_misses.text(),
            self.def_unwound.text(),
            self.reads.text(),
            distinct,
            self.days_stepped.text(),
            self.nonempty_days.text(),
            self.walks.text(),
            self.legs_booked.text(),
            self.num_results.text(),
        )
    }

    /// The second `--stats` line: the defs with the most misses, the most first, ties by id,
    /// at most `DEFS_SHOWN` of them, `stats-defs: id=misses …`.
    #[must_use]
    pub fn defs_line(&self) -> String {
        let mut defs: Vec<(&String, Count)> = self.by_def.iter().map(|(d, n)| (d, *n)).collect();
        // a count past u64 sorts first; otherwise the most misses first, then the id
        defs.sort_by(|a, b| {
            b.1.0
                .is_none()
                .cmp(&a.1.0.is_none())
                .then(b.1.0.cmp(&a.1.0))
                .then(a.0.cmp(b.0))
        });
        let shown: Vec<String> = defs
            .iter()
            .take(DEFS_SHOWN)
            .map(|(d, n)| format!("{d}={}", n.text()))
            .collect();
        format!("stats-defs: {}\n", shown.join(" "))
    }
}

/// How many defs the `stats-defs:` line names.
const DEFS_SHOWN: usize = 12;

#[cfg(test)]
mod tests {
    use super::{Count, Counters, Unit};

    #[test]
    fn a_count_past_u64_prints_overflow() {
        let mut c = Count(Some(u64::MAX));
        c.bump();
        assert_eq!(c.text(), "overflow", "never clamped");
        let mut d = Count::ZERO;
        d.plus(c);
        assert_eq!(d.text(), "overflow", "an overflow stays one");
    }

    #[test]
    fn a_distinct_read_counts_once_across_engines() {
        let mut a = Counters::default();
        a.read("x|k".to_owned());
        a.read("x|k".to_owned());
        a.miss("f");
        a.add(Unit::Ev);
        let mut b = Counters::default();
        b.read("x|k".to_owned());
        b.read("y|k".to_owned());
        b.miss("g");
        b.miss("g");
        b.miss("f");
        b.miss("f");
        b.miss("h");
        a.merge(&b);
        let line = a.line();
        assert!(line.starts_with("stats: ev=1 "), "{line}");
        assert!(line.contains(" reads=4 distinct_reads=2 "), "{line}");
        assert!(line.contains(" def_misses=6 "), "{line}");
        assert!(line.ends_with(" num_results=0\n"), "{line}");
        assert_eq!(a.defs_line(), "stats-defs: f=3 g=2 h=1\n", "most first");
    }
}
