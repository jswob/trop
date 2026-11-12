//! Milestone M0's load (`BUILD-ORDER.md` § M0, Green, its first two lines; `MILESTONE-M0.md`
//! rows 1 and 2): each vector of `m0/load.refcore` run in process. M0's engine evaluates
//! nothing, so a text set's load is seen through the load probe, `run <files> --ask
//! zz_m0_probe`, an ask no text declares: a load refusal prints as in any run, and after a
//! clean load the run prints `refused: REQ-ASK no ask zz_m0_probe` alone, exit 1 (E §11, tally
//! 2.1.3, #58), so these tests stay green once M1 evaluates. A scenario a stored output shows
//! refused is asked its own asks (refused with it). Row 1's 10 checks run as `checks.tsv`
//! writes them, against their stored baselines. The engine must exit with refcore 2.1.3's code
//! and print its whole output, `warning:` lines left out on both sides (the warnings are M1's).
//! Red until M0's loader is built, so this target has `test = false`; it gates once M0 is
//! tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m0::load_group_part_failures;

    /// Holds every load vector of `group` to refcore's run.
    fn loads_as_refcore(group: &str) {
        loads_part_as_refcore(group, (0, 1));
    }

    /// As `loads_as_refcore`, over the vectors of `group` at `part` modulo `parts` (pick P70).
    fn loads_part_as_refcore(group: &str, part: (usize, usize)) {
        let (failed, of) = load_group_part_failures(group, part).expect("m0/load.refcore reads");
        assert!(
            failed.is_empty(),
            "{} of {of} load vectors of {group} fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    // Row 1: the 10 load refusals, each its check's command line and its stored baseline.

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_law_13_rate25() {
        loads_as_refcore("refusal/U:law/LAW-13/rate25");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_rand_04_plan_b() {
        loads_as_refcore("refusal/U:rand/RAND-04/plan_b");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_rule_09_inpath() {
        loads_as_refcore("refusal/U:rule/RULE-09/inpath");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_scen_01_amend() {
        loads_as_refcore("refusal/U:scen/SCEN-01/amend");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_scen_02_kept() {
        loads_as_refcore("refusal/U:scen/SCEN-02/kept");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_scen_03_infl8() {
        loads_as_refcore("refusal/U:scen/SCEN-03/infl8");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_scen_04_crash() {
        loads_as_refcore("refusal/U:scen/SCEN-04/crash");
    }

    /// A scenario file loaded alone: `LOAD-HEADER`.
    #[test]
    fn m0_load_refusal_surf_04_redefine() {
        loads_as_refcore("refusal/U:surf/SURF-04/redefine");
    }

    /// A same-key cycle through two defs: `LOAD-CYCLE`.
    #[test]
    fn m0_load_refusal_rule_04() {
        loads_as_refcore("refusal/U:rule/RULE-04");
    }

    /// A kind that keeps parts and is signed: `LOAD-PARTS`.
    #[test]
    fn m0_load_refusal_own_11_p21_signed() {
        loads_as_refcore("refusal/U:own/OWN-11/p21-signed");
    }

    // Row 2: every text set of checks.tsv loads with no refusal but those its stored output
    // shows, one test per corpus group.

    /// The unit probes' texts (`unit/probes/`), one set per probe; row 1's 10 among them, by
    /// the probe.
    #[test]
    fn m0_load_probes() {
        loads_as_refcore("probes");
    }

    /// The probe sets (`U:set/…`): a probe with its scenario files; RULE-09's `inpath` refused
    /// alone (`LOAD-ASKONLY`), asked its own ask.
    #[test]
    fn m0_load_probe_sets() {
        loads_as_refcore("probe-sets");
    }

    /// The 77 models of `integration/models/`; MP-10 refused (`LOAD-BAND`).
    #[test]
    fn m0_load_models() {
        loads_as_refcore("models");
    }

    /// The light twins of the heavy models.
    #[test]
    fn m0_load_light() {
        loads_as_refcore("light");
    }

    /// The blind texts (`I:hardset/…`) and session 7's blind cards S1–S6.
    #[test]
    fn m0_load_blind() {
        loads_as_refcore("blind");
    }

    /// Household A's runs, with their scenarios and law files.
    #[test]
    fn m0_load_household_a_part_1() {
        loads_part_as_refcore("household-A", (0, 2));
    }

    /// Part 2 of 2.
    #[test]
    fn m0_load_household_a_part_2() {
        loads_part_as_refcore("household-A", (1, 2));
    }

    /// Household B's runs; `b10_v3`'s scenario `hr2_v3` refused alone (`LOAD-CYCLE`), asked
    /// its own ask.
    #[test]
    fn m0_load_household_b_part_1() {
        loads_part_as_refcore("household-B", (0, 2));
    }

    /// Part 2 of 2.
    #[test]
    fn m0_load_household_b_part_2() {
        loads_part_as_refcore("household-B", (1, 2));
    }

    /// Household C's runs.
    #[test]
    fn m0_load_household_c_part_1() {
        loads_part_as_refcore("household-C", (0, 3));
    }

    /// Part 2 of 3.
    #[test]
    fn m0_load_household_c_part_2() {
        loads_part_as_refcore("household-C", (1, 3));
    }

    /// Part 3 of 3.
    #[test]
    fn m0_load_household_c_part_3() {
        loads_part_as_refcore("household-C", (2, 3));
    }

    /// Household D's runs.
    #[test]
    fn m0_load_household_d() {
        loads_as_refcore("household-D");
    }

    /// Household E1's joint model, with its members A, B and D.
    #[test]
    fn m0_load_household_e1() {
        loads_as_refcore("household-E1");
    }

    /// Household E2's joint model, with its members A and D.
    #[test]
    fn m0_load_household_e2() {
        loads_as_refcore("household-E2");
    }

    /// The amendments household X: its base, its changes, its cut copies and minimal texts.
    #[test]
    fn m0_load_household_x() {
        loads_as_refcore("household-X");
    }
}
