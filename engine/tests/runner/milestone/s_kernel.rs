//! Milestone S's kernel tests on the engine (issue #41; `MILESTONE-S.md` row 5, DONE gate 8):
//! every vector of `kernel/vectors.refcore` and every record of the kernel corpus run in
//! process, the engine's whole output and exit code held to refcore's, byte for byte. One test
//! per group of T's plan, so a failure names its group; every failure of a group is collected
//! before it fails, so one run shows them all.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::corpus::{KERNEL_CORPUS, KERNEL_REFCORE, records, refcore_runs};
    use tally_runner::engine::{run_args, run_record};
    use tally_runner::held::is_held;
    use tally_runner::kernel::{VECTORS, difference, vectors};

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 15] = [
        "checks", "g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8", "g9", "g10", "g11", "g12", "g13",
        "g14",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("kernel/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("s_kernel", &v.id))
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from refcore:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn s_kernel_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("kernel/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each group of kernel/vectors.tsv has its test here"
        );
    }

    /// The kernel corpus's records (`properties/gen.py`), each the engine's whole output
    /// against refcore's (`properties/laws.py`).
    #[test]
    fn s_kernel_corpus_whole_output() {
        let runs = refcore_runs(KERNEL_REFCORE).expect("refcore's runs read");
        let recs = records(KERNEL_CORPUS).expect("the corpus reads");
        let failed: Vec<String> = recs
            .iter()
            .filter_map(|r| {
                let key = (r.case.clone(), r.variant.clone());
                match runs.get(&key) {
                    None => Some(format!("{}/{}: no refcore run", r.case, r.variant)),
                    Some(expected) => difference(expected, &run_record(r))
                        .map(|d| format!("{}/{}: {d}", r.case, r.variant)),
                }
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} corpus records differ from refcore:\n{}",
            failed.len(),
            recs.len(),
            failed.join("\n")
        );
    }

    /// The checks of `checks.tsv` the engine at S answers: 33 unit probes, 5 models and 5 of
    /// household X's checks, each compared whole (a household check's warnings included).
    #[test]
    fn s_kernel_checks_whole_output() {
        group_holds("checks");
    }

    /// G1, exact numbers, their text and rounding (L §7.1, §7.2, §5.18): every rounding mode
    /// at ties and off them, both signs, in the big tier too; a/b and the reading aid; money
    /// past the machine word; the 4 096-digit cap and a step that isn't positive, each refused
    /// beside an answered twin; a sum of a whole number and a fraction, a num in lowest terms,
    /// and what prints with no decimals (a minor unit of 1, an int), so a mutant in them fails
    /// here, not only in M0's load tests (tally 2.1.6) (`kernel/g1/`).
    #[test]
    fn s_kernel_g1_numbers() {
        group_holds("g1");
    }

    /// G2, the read rule (L §4.6, §4.5, §5.3, §5.11): rows by literal, range, `*` and a
    /// left-out key; known dates under `as known` cuts; spans; `replace`; precedence; keys and
    /// things a later version adds; dated rules; `held` past the cut (`kernel/g2/`).
    #[test]
    fn s_kernel_g2_read_rule() {
        group_holds("g2");
    }

    /// G3, bands (L §5.9): open, closed and infinite ends, a hole row, patches chained by
    /// known date, an argument no row holds, a band governing always (`kernel/g3/`).
    #[test]
    fn s_kernel_g3_bands() {
        group_holds("g3");
    }

    /// G4, holes in the answer (L §8.1, §4.6 step 5): one question at two figures, two
    /// questions sorted, a stated hole, the no-value texts, keys in brackets, a thing outside
    /// its lifespan read as none (`kernel/g4/`).
    #[test]
    fn s_kernel_g4_holes() {
        group_holds("g4");
    }

    /// G5, the day walk's agenda and the order within a day (E §4.2 steps 0–4, L §5.10,
    /// §5.14.2, §5.19): openings before the start and on it; readings at a day's start and end
    /// around the cut; observations before, inside and after the run, two on a day; a covered
    /// month; plans outside the run; decisions in strata, one due twice with `ties by` a
    /// position and one by a money value; a decision computed less its own earlier
    /// observation; decisions on the cut's day; a walk-bound def read at two points; ties with
    /// no `ties by` (RUN-TIE) beside an untied twin; a decision refused in its `when`; a
    /// projection read by a decision on its own day and by an ask, from a day with a reading
    /// at its start; lifespans that end inside the run; a cut after the run's end
    /// (`kernel/g5/`).
    #[test]
    fn s_kernel_g5_agenda() {
        group_holds("g5");
    }

    /// G6, the plan pass (E §4.1, L §5.14): a daily move's ledger lines in a covered month; a
    /// `when` that holds, fails, is a hole and is refused; an attribute refused; an uncertain
    /// kind with and without `with uncertain`; legs in money and units, of a kind of unit
    /// `any`, and of 0; money in two declared currencies on a leg of unit kind `currencies`
    /// beside a leg whose unit is the currency itself (tally 2.1.2, #43); which refusal a
    /// place keeps when two of a day's plan legs refuse there: the first in the typed movement
    /// order (tally 2.1.6, #57: move `a` before `ab`; one move's keys `[x, f3]` before
    /// `[y, f11]` and `[x, f3]` before `[x, f11]`), each beside a twin where one leg refuses
    /// alone; a refused `when`, attribute or key's `where` refusing only the places its legs
    /// give, from its day (tally 2.1.6, #51), beside a twin whose `where` holds (`kernel/g6/`).
    #[test]
    fn s_kernel_g6_plan_pass() {
        group_holds("g6");
    }

    /// G7, reads inside the walk and at the ask (L §5.10, §5.14, §5.14.1, §5.14.3, §8.2): a
    /// decision's read of its own day (RUN-CYCLE), of a later one (RUN-FUTURE), of its day's
    /// start and of the day before; flows of a place and of a decision move through a later
    /// day, the reader's own and the day before; flows read that tie two instances (RUN-TIE)
    /// beside a twin that doesn't; flows of a place beside a refused `when` whose legs don't
    /// touch it, which stands (tally 2.1.6, #51), a hole
    /// amount, a hole day and the run's start; flows of a move by leg and keys, and over a
    /// span with no movement; `bal(x, u, d)`; `with uncertain` around a reading; flows of a
    /// move whose leg the walk refuses, asked first while the walk it needs reads a def deeper
    /// than the evaluation's depth limit, in a decision and in a plan (`kernel/g7/`).
    #[test]
    fn s_kernel_g7_reads() {
        group_holds("g7");
    }

    /// G8, projections (L §5.14.1, E §4.3, D-6): `bal(x, d) from t` from before the run's
    /// start, from it, around the cut; the read's own `with uncertain` and the ask line's;
    /// a place first stated after t; a decision's projection from a later day (RUN-FUTURE)
    /// and an earlier one; #35, a decision reading a projection, asked with and without the
    /// line's `with uncertain`; #37 T1 and N2; a decision reading its own move's flows,
    /// projected from before its day; a projection from a decision's own point, alone, nested
    /// in another, and with a later decision that day refused in its `when`; tally 2.1.2: a
    /// projection at a decision's point counting a later decision of its stratum whose id
    /// extends its own (`sw`, `sw_b`), refused beside a twin that answers (#32), and `flows` of
    /// a place over days before a projection's start, from the walk it continues (#38)
    /// (`kernel/g8/`).
    #[test]
    fn s_kernel_g8_projections() {
        group_holds("g8");
    }

    /// G9, booking a leg and the sign rule (L §5.10, §5.14, E §4.2 steps 4–5): claims, debts
    /// and a holding broken by a first-batch plan and by a decision (RUN-PLACE), each beside a
    /// twin that stays at 0 or nets in its batch; a claim stated below zero at a close; legs
    /// out of and into places outside their lifespans (RUN-LIFESPAN) beside a leg of 0;
    /// `WARN-NEGATIVE` twice; a hole amount and a hole day; tally 2.1.2: a leg whose place is a
    /// hole, booking its known side, beside a twin between known places (#31); a reading's
    /// own place tested at the close (a pad and an anchor below 0, the twins at 0), at a day's
    /// start (a pad, a twin that nets in its batch) and at an opening (#46) (`kernel/g9/`).
    #[test]
    fn s_kernel_g9_sign_rule() {
        group_holds("g9");
    }

    /// G10, observations, readings and the residual ledger (L §5.19, E §6.1–§6.3): a transfer
    /// two statements cover, each booking its own end; links refused (RUN-RECORD); rows that
    /// disagree (WARN-LEGS) beside rows that agree; a restated row and a restated reading;
    /// `covered` over versions in turn and at a cut on a version's known day; a link whose
    /// instance's `when` reads an assumed row; pads up, by nothing and down, two places padding
    /// each other at one instant, an anchor; a place's first reading as a cut sees it; ledgers
    /// with matched, partly fulfilled, missing, undated, unexplained and balance lines, a link
    /// to an instance that doesn't exist beside one of its move that does, ties broken by the
    /// movement order, typed: key positions 2, 3 and 11 (tally 2.1.2, #33), and in the year
    /// 9999 a dated line before a hole-day one (tally 2.1.6, #48); an observed attribute naming
    /// a move instance of its kind (tally 2.1.6, #86) (`kernel/g10/`).
    #[test]
    fn s_kernel_g10_observations() {
        group_holds("g10");
    }

    /// G11, adjustments and their settlement (L §5.14.3, E §5): a version's change measured on
    /// instances by amount, by places, from a hole, into a hole, into a refusal and by a key it
    /// adds; settlements that run the leg's way, the other way, both and with one end, inside
    /// and outside a span; one nothing settles (HOLE-CHANGED) before and after its day; a
    /// waiver and a settlement written as moves bound over adjustments; rows naming
    /// adjustments no version made; a leg turned round and a leg touching neither place
    /// (RUN-RECORD) beside twins that settle; tally 2.1.2: a refused settlement read twice in
    /// one ask, in both orders, beside a twin that settles (#44); re-measurements refused by
    /// an amount and by a `when`, a booking refused at its first measure that stays refused,
    /// and `when`s made holes, then ended (#34), each beside reads that don't reach them; tally
    /// 2.1.6: under a hole `when`, a leg whose amount is refused has its adjustment refused, the
    /// other leg's a hole (#52), and each leg is measured against its own hole state
    /// (`kernel/g11/`).
    #[test]
    fn s_kernel_g11_adjustments() {
        group_holds("g11");
    }

    /// G12, bindings over movements and things (L §5.19 Bindings, §5.7, §8.4): a `for` over an
    /// event kind with members on several days and one on a hole day, which sorts last (#37
    /// N1); pairs of two spans; a `where` over a leg's field; an instance and the rows that
    /// fulfil it counted once, by the row where it is booked, else by the instance; an
    /// uncertain instance left out; a binding whose day is a hole; attributes; what rows
    /// fulfil; `x in K on d` with d known, before a lifespan, a hole and never (#37 N3); moves
    /// bound over things on an assumed day and on a hole day; tally 2.1.2: movements by day,
    /// then their keys in index order, positions 2, 3 and 11 (#39) (`kernel/g12/`).
    #[test]
    fn s_kernel_g12_members() {
        group_holds("g12");
    }

    /// G13, what load checks at S (tally 2.1.2): `LOAD-DEPTH` counts a `from` term one level
    /// (L §2.3, #29), refused at 9 beside a twin at 8 that answers; `WARN-UNREAD` for a place
    /// no line names (L §8.5, #30): an opening, a reading, an anchor and a `covers` line name
    /// none; a read, a leg, an observed row's leg, a pad, a binding over its kind and a row
    /// whose value it is each name one; any spacing between tokens reads (tally 2.1.6, #81): a
    /// field spaced at its `.`, an id at its `[`, a call at its `(`, beside its touching twin
    /// (`kernel/g13/`).
    #[test]
    fn s_kernel_g13_load() {
        group_holds("g13");
    }

    /// G14, capacity at run (L §7.6; tally 2.1.6, #89 and #48): a day computed past
    /// 9999-12-31 or before 0001-01-01, and the end of time read as a day, refused at its key
    /// (`RUN-LIMIT`) beside the edges that answer; a movement whose day passes 9999-12-31
    /// refusing its legs' places on every day while another place stands; a range from rows of
    /// more than 1 000 000 keys beside one of 1 000 000; the edges load allows (1 000 000 keys,
    /// a period ending on 9999-12-31, 18 digits, unit powers of 64); a calendar with no
    /// business day (`RUN-DOMAIN`) beside six weekend days and every month-day; and the
    /// review's seven findings in 0bbf95b (P35): the end of time as a span's end or a key, a
    /// day, month or year shifted by the machine word's least or greatest value or past 64
    /// bits, `add_business` by the least value, ranges near ±9e18, an integer key past 64
    /// bits (`RUN-KEY`), and a list index of 1 000 001 keys (`LOAD-LIMIT`) beside its twin of
    /// 1 000 000, texts of 8.9 MB generated at test time from their sources (`.gen`, P36),
    /// each beside its answered twin; and the second review's six (P37): an integer past 64
    /// bits as a business-day count (a hole's question, `RUN-DOMAIN`), a duration, a key's
    /// shift (`RUN-KEY`), a day number, a power on base 1, 0 or -1 and a calendar's first
    /// year, ranges near the 4 096-digit cap, and the refusal texts refcore says
    /// (`kernel/g14/`).
    #[test]
    fn s_kernel_g14_capacity() {
        group_holds("g14");
    }
}
