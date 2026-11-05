//! Milestone M0's code fixtures on the engine (DONE gate 5; `BUILD-ORDER.md` § M0: each
//! `LOAD-*` code has its fixture; `MILESTONE-M0.md` rows 5 and 6): each fixture of
//! `codes/fixtures.tsv` and of `codes/m0.refcore` run in process, its output and exit code
//! refcore's, byte for byte. `codes/m0.refcore` holds a fixture for each `LOAD-*` code no
//! other output at S or M0 prints (`tools/xref_check.py --gate M0`), each one edit of a
//! family's base text, which loads clean and runs beside it as its twin (the load probe:
//! `refused: REQ-ASK no ask zz_m0_probe`); #45's three gaps and C-16's exchange rate among
//! them. Then, tier 2 (G7), a fixture for each of the code's message forms (L §8.3) no
//! other output prints, its refusal held to that form by `codes/m0.py`. One test per code,
//! its fixtures named. Red on the stub (exit 69, "not implemented"), so this target has
//! `test = false`; it gates once M0 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::codes::{FIXTURES, Fixture, M0, M0Fixture, fixtures, m0_fixtures};
    use tally_runner::engine::run_args;
    use tally_runner::kernel::difference;

    /// Runs every fixture of `codes/m0.refcore` that prints `code`, and the base of each of
    /// their families (the stated twin, which loads clean), on the engine against refcore's
    /// whole output and exit code.
    fn code_as_refcore(code: &str) {
        let vs = m0_fixtures(M0).expect("codes/m0.refcore reads");
        let mine: Vec<&M0Fixture> = vs
            .iter()
            .filter(|v| !v.is_base() && v.code == code)
            .collect();
        assert!(!mine.is_empty(), "{code} has fixtures");
        let families: BTreeSet<&str> = mine.iter().map(|v| v.family.as_str()).collect();
        let twins = vs
            .iter()
            .filter(|v| v.is_base() && families.contains(v.family.as_str()));
        let failed: Vec<String> = twins
            .chain(mine.iter().copied())
            .filter_map(|v| {
                difference(&v.run, &run_args(&v.args, &v.files)).map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of the vectors of {code} and their bases fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// Runs every fixture of `code` on the engine against refcore's output and exit code.
    fn refuses_as_refcore(code: &str) {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        let mine: Vec<&Fixture> = fs.iter().filter(|f| f.code == code).collect();
        assert!(!mine.is_empty(), "{code} has fixtures");
        all_as_refcore(&mine);
    }

    /// Runs every fixture under the folder `folder` of `codes/` (its id `<folder>/…`), each
    /// refusal with its control, on the engine against refcore's output and exit code.
    fn folder_as_refcore(folder: &str) {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        let prefix = format!("{folder}/");
        let mine: Vec<&Fixture> = fs.iter().filter(|f| f.id.starts_with(&prefix)).collect();
        assert!(!mine.is_empty(), "{folder} has fixtures");
        all_as_refcore(&mine);
    }

    /// Holds each fixture of `mine` to refcore's run, collecting every failure.
    fn all_as_refcore(mine: &[&Fixture]) {
        let failed: Vec<String> = mine
            .iter()
            .filter_map(|f| {
                let run = run_args(&f.args, &f.files);
                (run.out != f.out || run.code != f.exit_code).then(|| {
                    format!(
                        "{}: exit {} printing {:?}; refcore: exit {} printing {:?}",
                        f.id, run.code, run.out, f.exit_code, f.out
                    )
                })
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} fixtures fail:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m0_load_path_a_path_that_names_no_file() {
        refuses_as_refcore("LOAD-PATH");
    }

    #[test]
    fn m0_load_parse_the_load_path_fixtures_control() {
        refuses_as_refcore("LOAD-PARSE");
    }

    /// E §3.1 The order of the checks (tally 2.1.2, issue #23): the passes run in order (a
    /// second declaration refuses before a def's key, whatever the lines), and the items' own
    /// checks take moves by full path and line, not by id; each beside its control.
    #[test]
    fn m0_load_order_the_checks_past_parsing() {
        folder_as_refcore("order");
    }

    /// L §2.3 `LOAD-DEPTH` (tally 2.1.2, issue #29): a `ties by` term counts one level, so 9
    /// argmins chained by `ties by` are refused at the parse, before the declarations pass;
    /// its control, 8 deep, parses and is refused by that pass.
    #[test]
    fn m0_load_depth_a_ties_by_term_counts_one_level() {
        folder_as_refcore("LOAD-DEPTH");
    }

    /// `LOAD-ALIVE`: `basic/alive`.
    #[test]
    fn m0_code_load_alive() {
        code_as_refcore("LOAD-ALIVE");
    }

    /// `LOAD-AMBIG`: `basic/ambig`.
    #[test]
    fn m0_code_load_ambig() {
        code_as_refcore("LOAD-AMBIG");
    }

    /// `LOAD-ASIF`: `basic/asif`, `basic/asif-unknown`.
    #[test]
    fn m0_code_load_asif() {
        code_as_refcore("LOAD-ASIF");
    }

    /// `LOAD-ASKONLY`: `basic/askonly-solve`.
    #[test]
    fn m0_code_load_askonly() {
        code_as_refcore("LOAD-ASKONLY");
    }

    /// `LOAD-BAND`: `basic/band-marginal`.
    #[test]
    fn m0_code_load_band() {
        code_as_refcore("LOAD-BAND");
    }

    /// `LOAD-COVERED`: `basic/covered`.
    #[test]
    fn m0_code_load_covered() {
        code_as_refcore("LOAD-COVERED");
    }

    /// `LOAD-COVERS`: `basic/covers`, `adjust/covers-outside`.
    #[test]
    fn m0_code_load_covers() {
        code_as_refcore("LOAD-COVERS");
    }

    /// `LOAD-CYCLE`: `basic/cycle-both`, `basic/cycle-beats`.
    #[test]
    fn m0_code_load_cycle() {
        code_as_refcore("LOAD-CYCLE");
    }

    /// `LOAD-DOMAIN`: `basic/domain`, `basic/domain-discrete`, `lifepath/paths-zero`,
    /// `lifepath/path-zero`, `lifepath/path-past-n`, `lifepath/path-past-64`,
    /// `lifepath/paths-past-64` (a path count printed, of any size: G11's B6).
    #[test]
    fn m0_code_load_domain() {
        code_as_refcore("LOAD-DOMAIN");
    }

    /// `LOAD-DUP`: `basic/dup-key`, `basic/dup-band`.
    #[test]
    fn m0_code_load_dup() {
        code_as_refcore("LOAD-DUP");
    }

    /// `LOAD-FIELD`: `basic/field`, `basic/field-not`, `lifepath/lifespan-field`.
    #[test]
    fn m0_code_load_field() {
        code_as_refcore("LOAD-FIELD");
    }

    /// `LOAD-FORKS`: `basic/forks`.
    #[test]
    fn m0_code_load_forks() {
        code_as_refcore("LOAD-FORKS");
    }

    /// `LOAD-GAPROLE`: `basic/gaprole`.
    #[test]
    fn m0_code_load_gaprole() {
        code_as_refcore("LOAD-GAPROLE");
    }

    /// `LOAD-GOVERNS`: `basic/governs`.
    #[test]
    fn m0_code_load_governs() {
        code_as_refcore("LOAD-GOVERNS");
    }

    /// `LOAD-GUARD`: `basic/guard`.
    #[test]
    fn m0_code_load_guard() {
        code_as_refcore("LOAD-GUARD");
    }

    /// `LOAD-HEADER`: `basic/header-tally`, `basic/header-on`, `basic/header-end`,
    /// `basic/header-order`, `scen/header-from`.
    #[test]
    fn m0_code_load_header() {
        code_as_refcore("LOAD-HEADER");
    }

    /// `LOAD-INSTANT`: `basic/instant`, `basic/instant-side`, `basic/instant-none`.
    #[test]
    fn m0_code_load_instant() {
        code_as_refcore("LOAD-INSTANT");
    }

    /// `LOAD-KEY`: `basic/key-not`, `basic/key-id`, `basic/key-count`, `basic/key-range`,
    /// `basic/key-two`, `basic/key-across`, `scenplace/covered-unknown` (`covered` through a
    /// scenario row's base prefix: G12's N1), `kplace/same-place-unknown` (a `same` side keyed
    /// by a keyed place: G12's N3).
    #[test]
    fn m0_code_load_key() {
        code_as_refcore("LOAD-KEY");
    }

    /// `LOAD-KIND`: `basic/kind`.
    #[test]
    fn m0_code_load_kind() {
        code_as_refcore("LOAD-KIND");
    }

    /// `LOAD-KNOWN`: `basic/known`, `basic/known-after-on`, `basic/known-row`.
    #[test]
    fn m0_code_load_known() {
        code_as_refcore("LOAD-KNOWN");
    }

    /// `LOAD-LABEL`: `basic/label`, `basic/label-twice`.
    #[test]
    fn m0_code_load_label() {
        code_as_refcore("LOAD-LABEL");
    }

    /// `LOAD-LAWDATE`: `basic/lawdate`, `basic/lawdate-direct`, `basic/lawdate-def`,
    /// `adjust/lawdate-move`.
    #[test]
    fn m0_code_load_lawdate() {
        code_as_refcore("LOAD-LAWDATE");
    }

    /// `LOAD-LAWPERIOD`: `basic/lawperiod`.
    #[test]
    fn m0_code_load_lawperiod() {
        code_as_refcore("LOAD-LAWPERIOD");
    }

    /// `LOAD-LEAF`: `basic/leaf`.
    #[test]
    fn m0_code_load_leaf() {
        code_as_refcore("LOAD-LEAF");
    }

    /// `LOAD-LEG`: `basic/leg`, `basic/leg-attr`, `basic/leg-lacks`, `adjust/leg-uncertain`.
    #[test]
    fn m0_code_load_leg() {
        code_as_refcore("LOAD-LEG");
    }

    /// `LOAD-LIFESPAN`: `basic/lifespan`, `basic/lifespan-enter`.
    #[test]
    fn m0_code_load_lifespan() {
        code_as_refcore("LOAD-LIFESPAN");
    }

    /// `LOAD-LITERAL`: `basic/literal-amount`.
    #[test]
    fn m0_code_load_literal() {
        code_as_refcore("LOAD-LITERAL");
    }

    /// `LOAD-MISSING`: `basic/missing`, `basic/missing-read`.
    #[test]
    fn m0_code_load_missing() {
        code_as_refcore("LOAD-MISSING");
    }

    /// `LOAD-MOVES`: `basic/moves`, `basic/moves-twice`.
    #[test]
    fn m0_code_load_moves() {
        code_as_refcore("LOAD-MOVES");
    }

    /// `LOAD-NAME`: `basic/name`, `basic/name-binding`, `basic/name-rule-v`, `scenplace/sum` (an
    /// aggregate through a scenario row's base prefix: G12's N1).
    #[test]
    fn m0_code_load_name() {
        code_as_refcore("LOAD-NAME");
    }

    /// `LOAD-ORDER`: `basic/order`, `basic/order-plan`, `basic/order-legs`.
    #[test]
    fn m0_code_load_order() {
        code_as_refcore("LOAD-ORDER");
    }

    /// `LOAD-ORIENT`: `basic/orient`.
    #[test]
    fn m0_code_load_orient() {
        code_as_refcore("LOAD-ORIENT");
    }

    /// `LOAD-OVERLAP`: `basic/overlap`.
    #[test]
    fn m0_code_load_overlap() {
        code_as_refcore("LOAD-OVERLAP");
    }

    /// `LOAD-PARSE`: `lifepath/path-alone` (a path count printed, past 64 bits: G11's B6),
    /// `lifepath/lifespan-two`, `lifepath/lifespan-none` (`lifespan` takes one thing: G11's B2),
    /// `scenplace/covered-args`, `scenplace/covered-call` (`covered` through a scenario row's
    /// base prefix: G12's N1).
    #[test]
    fn m0_code_load_parse() {
        code_as_refcore("LOAD-PARSE");
    }

    /// `LOAD-PARTS`: `own/parts-missing`, `own/parts-under`, `own/parts-arrival`, `own/parts-owns`,
    /// `own/parts-of-input`, `own/parts-over`, `own/parts-below`, `own/parts-overlap`,
    /// `own/parts-sort`, `jown/same-parts`.
    #[test]
    fn m0_code_load_parts() {
        code_as_refcore("LOAD-PARTS");
    }

    /// `LOAD-PIN`: `joint/pin`.
    #[test]
    fn m0_code_load_pin() {
        code_as_refcore("LOAD-PIN");
    }

    /// `LOAD-PLACEKIND`: `basic/placekind`, `basic/placekind-one`.
    #[test]
    fn m0_code_load_placekind() {
        code_as_refcore("LOAD-PLACEKIND");
    }

    /// `LOAD-PRECISION`: `basic/precision`, `basic/precision-fraction`.
    #[test]
    fn m0_code_load_precision() {
        code_as_refcore("LOAD-PRECISION");
    }

    /// `LOAD-PRICE`: `basic/price`, `basic/price-twice`, `basic/price-by`.
    #[test]
    fn m0_code_load_price() {
        code_as_refcore("LOAD-PRICE");
    }

    /// `LOAD-RANDOM`: `basic/random`, `basic/random-closed`, `basic/random-cdf`,
    /// `basic/random-path`.
    #[test]
    fn m0_code_load_random() {
        code_as_refcore("LOAD-RANDOM");
    }

    /// `LOAD-READS`: `basic/reads`, `basic/reads-unread`.
    #[test]
    fn m0_code_load_reads() {
        code_as_refcore("LOAD-READS");
    }

    /// `LOAD-REPLACE`: `basic/replace`.
    #[test]
    fn m0_code_load_replace() {
        code_as_refcore("LOAD-REPLACE");
    }

    /// `LOAD-RESERVED`: `basic/reserved`.
    #[test]
    fn m0_code_load_reserved() {
        code_as_refcore("LOAD-RESERVED");
    }

    /// `LOAD-SAME`: `basic/same`, `joint/same-what`, `joint/same-unjoined`, `joint/same-minor`.
    #[test]
    fn m0_code_load_same() {
        code_as_refcore("LOAD-SAME");
    }

    /// `LOAD-SCENARIO`: `basic/scenario-row`, `basic/scenario-ask`, `scen/scenario-early`,
    /// `scen/scenario-over`, `scen/scenario-alive`.
    #[test]
    fn m0_code_load_scenario() {
        code_as_refcore("LOAD-SCENARIO");
    }

    /// `LOAD-SCHEDULE`: `basic/schedule`.
    #[test]
    fn m0_code_load_schedule() {
        code_as_refcore("LOAD-SCHEDULE");
    }

    /// `LOAD-SHADOW`: `basic/shadow`.
    #[test]
    fn m0_code_load_shadow() {
        code_as_refcore("LOAD-SHADOW");
    }

    /// `LOAD-TO`: `basic/to`, `basic/to-dist`.
    #[test]
    fn m0_code_load_to() {
        code_as_refcore("LOAD-TO");
    }

    /// `LOAD-UNIT`: `basic/unit-mixed`, `basic/unit-currency`, `basic/unit-fx`,
    /// `adjust/unit-settlement`, `basic/unit-mode`, `lifepath/lifespan-money`,
    /// `lifepath/lifespan-key`, `lifepath/lifespan-record`, `lifepath/lifespan-to-num` (`lifespan`
    /// typed at load: G11's B2), `scenplace/covered-amt` (G12's N1), `adjust/adjusts-type`,
    /// `adjust/change-type`, `adjust/lifespan-dyn` (an adjustment's fields typed at load,
    /// `lifespan` of `dyn`: G12's leftovers 2 and 3).
    #[test]
    fn m0_code_load_unit() {
        code_as_refcore("LOAD-UNIT");
    }

    /// `LOAD-VERSION`: `parts/version`, `basic/version-rule`.
    #[test]
    fn m0_code_load_version() {
        code_as_refcore("LOAD-VERSION");
    }

    /// `LOAD-WORLD`: `scen/world`.
    #[test]
    fn m0_code_load_world() {
        code_as_refcore("LOAD-WORLD");
    }
}
