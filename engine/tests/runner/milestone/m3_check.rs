//! Milestone M3's `check` (`BUILD-ORDER.md` § M3; `OPERATIONS.md` §2.3; `MILESTONE-M3.md` rows
//! 2 and 3): each vector of `m3/check.vectors` run in process as `check <files>`. M0's 315
//! text sets, one test per M0 corpus group (split as `m0_load` splits them, pick P70), and the
//! fixtures of `m3/check/`, one test per line kind: `ok` and its warnings, a base that doesn't
//! load, refused scenarios, asks refused alone, the three sections together, `ok` where `run`
//! wouldn't answer cleanly, paths and arguments, a usage error. The engine must exit with the
//! vector's code and print its whole output (a usage error: exit 2, its words the engine's).
//! The bytes are refcore's `check` (chunk C4), which confirmed what its `run` predicted
//! (pick P8). Red until the engine builds `check`, so this target has `test =
//! false`; it gates once M3 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::m3::{check_group_failures, check_groups};

    /// Every group of the store, each run by a test below.
    const GROUPS: [&str; 24] = [
        "probes",
        "probe-sets",
        "models",
        "light",
        "blind",
        "household-A",
        "household-B",
        "household-C",
        "household-D",
        "household-E1",
        "household-E2",
        "household-X",
        "ok",
        "base",
        "scenario",
        "alone",
        "sections",
        "notclean",
        "paths",
        "usage",
        "changes",
        "operations",
        "reviews",
        "t221-gd",
    ];

    /// Holds every vector of `group` at `part` modulo `parts` to its stored run.
    fn checks_part_as_stored(group: &str, part: (usize, usize)) {
        let (failed, of) = check_group_failures(group, part).expect("m3/check.vectors reads");
        assert!(
            failed.is_empty(),
            "{} of {of} check vectors of {group} fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// Holds every vector of `group` to its stored run.
    fn checks_as_stored(group: &str) {
        checks_part_as_stored(group, (0, 1));
    }

    #[test]
    fn m3_check_every_group_is_tested() {
        let stored = check_groups().expect("m3/check.vectors reads");
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(stored, tested, "each group of the store has its test");
    }

    // M0's 315 text sets (MILESTONE-M3.md row 2).

    /// The unit probes' texts: `ok`, or row 1's load refusals.
    #[test]
    fn m3_check_probes() {
        checks_as_stored("probes");
    }

    /// The probe sets: RULE-09's `inpath` and SURF-04's `redefine`, scenarios refused alone.
    #[test]
    fn m3_check_probe_sets() {
        checks_as_stored("probe-sets");
    }

    /// The 77 models; MP-10 refused (`LOAD-BAND`).
    #[test]
    fn m3_check_models() {
        checks_as_stored("models");
    }

    /// The light twins of the heavy models.
    #[test]
    fn m3_check_light() {
        checks_as_stored("light");
    }

    /// The blind texts and session 7's cards.
    #[test]
    fn m3_check_blind() {
        checks_as_stored("blind");
    }

    /// Household A's texts, the first half.
    #[test]
    fn m3_check_household_a_part_1() {
        checks_part_as_stored("household-A", (0, 2));
    }

    /// Household A's texts, the second half.
    #[test]
    fn m3_check_household_a_part_2() {
        checks_part_as_stored("household-A", (1, 2));
    }

    /// Household B's texts, the first half; `b10_v3`'s scenario `hr2_v3` refused (`LOAD-CYCLE`)
    /// with its warnings.
    #[test]
    fn m3_check_household_b_part_1() {
        checks_part_as_stored("household-B", (0, 2));
    }

    /// Household B's texts, the second half.
    #[test]
    fn m3_check_household_b_part_2() {
        checks_part_as_stored("household-B", (1, 2));
    }

    /// Household C's texts, a third.
    #[test]
    fn m3_check_household_c_part_1() {
        checks_part_as_stored("household-C", (0, 3));
    }

    /// Household C's texts, a third.
    #[test]
    fn m3_check_household_c_part_2() {
        checks_part_as_stored("household-C", (1, 3));
    }

    /// Household C's texts, a third.
    #[test]
    fn m3_check_household_c_part_3() {
        checks_part_as_stored("household-C", (2, 3));
    }

    /// Household D's texts.
    #[test]
    fn m3_check_household_d() {
        checks_as_stored("household-D");
    }

    /// E1's joint model and its B side.
    #[test]
    fn m3_check_household_e1() {
        checks_as_stored("household-E1");
    }

    /// E2's joint models.
    #[test]
    fn m3_check_household_e2() {
        checks_as_stored("household-E2");
    }

    /// Household X: its base, cuts, changes and minimal texts.
    #[test]
    fn m3_check_household_x() {
        checks_as_stored("household-X");
    }

    // The fixtures of m3/check/, one test per line kind (MILESTONE-M3.md row 3).

    /// `ok` alone, and `ok` with load's warnings (each subject of L §8.5, an `outside` place, a
    /// joint member's as `run` prints it), with a scenario that loads.
    #[test]
    fn m3_check_ok() {
        checks_as_stored("ok");
    }

    /// A base that doesn't load: its one line and nothing more (a duplicate, a path that names
    /// nothing, no path, a read of a scenario's item, beside a refused scenario, a member's, a
    /// pin).
    #[test]
    fn m3_check_base() {
        checks_as_stored("base");
    }

    /// Refused scenarios, in their files' full-path order (pick P4): one, two, one over a
    /// refused one, `over` naming nothing, a cycle, deeper than 4, declared twice.
    #[test]
    fn m3_check_scenario() {
        checks_as_stored("scenario");
    }

    /// Asks refused alone, by id (pick P5): plain, through `diff`, a scenario pin and `as if`,
    /// beside a scenario, covered by a refused scenario's line.
    #[test]
    fn m3_check_alone() {
        checks_as_stored("alone");
    }

    /// The scenario lines, then the asks refused alone, then the warnings; exit 1.
    #[test]
    fn m3_check_sections() {
        checks_as_stored("sections");
    }

    /// `ok` isn't a clean run: `REQ-CLOCK`, a hole, a per-value refusal, `REQ-PATHS` come
    /// only from `run`.
    #[test]
    fn m3_check_notclean() {
        checks_as_stored("notclean");
    }

    /// A directory's files by full path; the household as a directory (#226's example);
    /// `--pin=bytes`.
    #[test]
    fn m3_check_paths() {
        checks_as_stored("paths");
    }

    /// `check` with an ask: a usage error, exit 2 (pick P3).
    #[test]
    fn m3_check_usage() {
        checks_as_stored("usage");
    }

    /// The seven changes through OPERATIONS §6.1's step 2 (MILESTONE-M3.md row 6): each stage
    /// with its change `ok`, and each without its advance of `on` refused `LOAD-KNOWN`.
    #[test]
    fn m3_check_changes() {
        checks_as_stored("changes");
    }

    /// OPERATIONS §3 and §7's examples on the small model `mini`: the files §3 prints load, CHG-7's
    /// `LOAD-KNOWN`, TXT-3's `LOAD-KEY`, ASK-2's `LOAD-DUP`, §7's worked scenario line and warning.
    #[test]
    fn m3_check_operations() {
        checks_as_stored("operations");
    }

    /// The group reviews' cases (M3 C3): one id refused alone in two scenario files (P5), a
    /// scenario declared in two files (P12), a scenario's narrowed coverage (P19); all three
    /// held while refcore differs (#228, #232).
    #[test]
    fn m3_check_reviews() {
        checks_as_stored("reviews");
    }

    /// tally 2.2.1's guards (#241, T's plan §1): a joint model's `check` holds no member's
    /// scenario lines (228.Q12, R20 option A); a scenario file's literal key is load's (229.2).
    #[test]
    fn m3_check_t221_guards() {
        checks_as_stored("t221-gd");
    }
}
