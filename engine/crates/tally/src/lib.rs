//! The Tally engine: a reality goes in as bytes, a result comes out (ENGINE §1).
//! Spec: none, a stub.
//! Twin: none, a stub.
//! Invariants: none yet; every entry point answers `NotImplemented`.
//!
//! A stub. Every entry point answers [`NotImplemented`], and the build session
//! replaces it. Nothing here is a design choice the spec owns.

#![forbid(unsafe_code)]

#[cfg(all(feature = "fault-injection", not(debug_assertions)))]
compile_error!("the fault-injection feature exists only in builds with debug assertions (R-51)");
#[cfg(all(feature = "counters", not(debug_assertions)))]
compile_error!("the counters feature exists only in builds with debug assertions (R-51)");

/// The answer of every entry point until the engine is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotImplemented;

/// Takes a source text as bytes (the library reads no files, R-39) and answers
/// [`NotImplemented`]. The fuzz target `parse` calls it.
pub fn parse(_source: &[u8]) -> Result<(), NotImplemented> {
    Err(NotImplemented)
}

/// The work units PERFORMANCE §1.1 counts, behind the `counters` feature (D4-8,
/// BUILDER §13.2). The stub counts nothing, so every count is 0.
#[cfg(feature = "counters")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counters {
    /// Expression nodes evaluated (ev).
    pub ev: u64,
    /// Def cell reads that found the memo's value.
    pub def_hits: u64,
    /// Def cell reads that computed it.
    pub def_misses: u64,
    /// Input reads by the read rule.
    pub reads: u64,
    /// Distinct input reads.
    pub distinct_reads: u64,
    /// Days the day walk stepped through (walk days).
    pub days_stepped: u64,
    /// Stepped days with at least one act.
    pub nonempty_days: u64,
    /// Movement legs written to a place's log.
    pub legs_booked: u64,
    /// Groups of parts work at places that keep parts.
    pub parts_groups: u64,
    /// Timeline prefixes taken from a checkpoint instead of walked again.
    pub walks_shared: u64,
    /// Shared-layer cells a worker computed for itself.
    pub recomputed_locally: u64,
    /// Exact values made.
    pub num_results: u64,
}

#[cfg(feature = "counters")]
impl Counters {
    /// The `--stats` line on standard error: `stats: ` then `key=value` pairs, the form
    /// `engine/tools/perf counts` reads.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "stats: ev={} def_hits={} def_misses={} reads={} distinct_reads={} days_stepped={} \
             nonempty_days={} legs_booked={} parts_groups={} walks_shared={} recomputed_locally={} \
             num_results={}\n",
            self.ev,
            self.def_hits,
            self.def_misses,
            self.reads,
            self.distinct_reads,
            self.days_stepped,
            self.nonempty_days,
            self.legs_booked,
            self.parts_groups,
            self.walks_shared,
            self.recomputed_locally,
            self.num_results,
        )
    }
}
