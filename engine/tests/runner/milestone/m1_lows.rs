//! Milestone M1's fence F5 on T's realities (pick P57; R-130, R-84; `MILESTONE-M1.md` row 9;
//! issue #108). Every ask of T's corpora (`properties/kernel.corpus`, 312 records;
//! `properties/m1.corpus`, 648), of every fix vector of `m1/vectors.refcore` (groups
//! `fixes-…`) and of F5's own realities (`m1/lows/`, G9's `tally-cli/tests/lows/` copied) runs
//! through `tally_cli::tally::run_with_lows` twice (the CLI re-exports the library, pick
//! P59), with `Lows::Tree` (fence F5's tree of running sums, P §2.3, as `run` reads) and with
//! `Lows::Walk` (the projection's own walk, its plain twin, E §4.3), and the two print the
//! same lines. A difference is the fence's fault
//! (`run_with_lows`'s doc). Since F5 answers T's projection reads, T's vectors reach the twin
//! only where the tree declines; this target reaches it on every read, so the twin's mutants
//! are T's to catch (R-84 counts T's tests at gate 8; `mutants-local --next` counts this
//! target). One test per source, and the fix vectors one per group (pick P69), so a failure
//! names it. `test = false` until M1's marker (pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_cli::tally::num::Num;
    use tally_cli::tally::time::Day;
    use tally_cli::tally::walk::{Lows, RangeMinimum};
    use tally_cli::tally::{Asks, Days, Given, NotImplemented, Pin, Request, run_with_lows};
    use tally_runner::corpus::{KERNEL_CORPUS, netstrings, records};
    use tally_runner::kernel::vectors;
    use tally_runner::laws_m1::M1_CORPUS;
    use tally_runner::m1::VECTORS;

    /// F5's reality: a pay that differs by an axis's option, a rent a later version raises, a
    /// monthly look-ahead decision reading `low` from its own day, projections from days before
    /// and after the cut (G9's `lows.tally`), with its scenario `richer` from June.
    const LOWS: [(&str, &str); 2] = [
        ("lows.tally", include_str!("../../m1/lows/lows.tally")),
        ("richer.tally", include_str!("../../m1/lows/richer.tally")),
    ];

    /// F5's other realities, one file each: projections from many cuts across a version that
    /// raises the rent and one that moves a bonus (`cuts`); a projection's walk refused on a
    /// decision's day read on both sides of it (`failed`, P54); a covered rent read from two
    /// cuts (`cov_x`, `cov_nocov`); a rent's day that moves between the cuts (`inv_min`).
    const ALONE: [(&str, &str); 5] = [
        ("cuts.tally", include_str!("../../m1/lows/cuts.tally")),
        ("failed.tally", include_str!("../../m1/lows/failed.tally")),
        ("cov_x.tally", include_str!("../../m1/lows/cov_x.tally")),
        (
            "cov_nocov.tally",
            include_str!("../../m1/lows/cov_nocov.tally"),
        ),
        ("inv_min.tally", include_str!("../../m1/lows/inv_min.tally")),
    ];

    /// A command line's paths and request, as the CLI reads `run`'s words (INTERFACE §4).
    fn request_of(args: &[String]) -> Result<(Vec<String>, Request), String> {
        let Some((first, rest)) = args.split_first() else {
            return Err("an empty command line".to_owned());
        };
        if first != "run" {
            return Err(format!("not a run: {first}"));
        }
        let mut paths = Vec::new();
        let mut asks = Vec::new();
        let mut pin = Pin::Canonical;
        let mut after_ask = false;
        for word in rest {
            if word == "--ask" {
                after_ask = true;
            } else if let Some(form) = word.strip_prefix("--pin=") {
                pin = if form == "bytes" {
                    Pin::Bytes
                } else {
                    Pin::Canonical
                };
            } else if after_ask {
                asks.push(word.clone());
            } else {
                paths.push(word.clone());
            }
        }
        let asks = if asks.is_empty() {
            Asks::Every
        } else {
            Asks::Named(asks)
        };
        let request = Request {
            asks,
            pin,
            days: Days::Events,
        };
        Ok((paths, request))
    }

    /// The files the paths name, read from the file table `table` (netstring pairs).
    fn given_of(paths: &[String], table: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut pairs = Vec::new();
        let mut name: Option<&[u8]> = None;
        for part in netstrings(table)? {
            match name.take() {
                None => name = Some(part),
                Some(n) => pairs.push((n, part)),
            }
        }
        let mut out = Vec::new();
        for path in paths {
            let Some((_, bytes)) = pairs.iter().find(|(n, _)| *n == path.as_bytes()) else {
                return Err(format!("{path} is not in the file table"));
            };
            out.push((path.clone(), bytes.to_vec()));
        }
        Ok(out)
    }

    /// The lines of one run with `lows`, or `None` where it stops not implemented.
    fn lines(files: &[(String, Vec<u8>)], request: &Request, lows: Lows) -> Option<Vec<String>> {
        let given = files
            .iter()
            .map(|(path, bytes)| Given {
                path: path.clone(),
                bytes: Some(bytes.clone()),
            })
            .collect();
        match run_with_lows(given, request, lows) {
            Ok(answer) => Some(answer.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    /// Runs `files` and `request` with the tree and with the twin; `Err` names the first line
    /// they differ at. `Ok(true)` where both answered, `Ok(false)` where both stopped.
    fn same(files: &[(String, Vec<u8>)], request: &Request) -> Result<bool, String> {
        let tree = lines(files, request, Lows::Tree);
        let walk = lines(files, request, Lows::Walk);
        match (tree, walk) {
            (None, None) => Ok(false),
            (Some(t), Some(w)) if t == w => Ok(true),
            (Some(t), Some(w)) => {
                let line = match t.iter().zip(&w).position(|(a, b)| a != b) {
                    Some(i) => i,
                    None => t.len().min(w.len()),
                };
                Err(format!(
                    "line {}: tree {:?}, walk {:?}",
                    line.checked_add(1).expect("a line number"),
                    t.get(line),
                    w.get(line)
                ))
            }
            (t, w) => Err(format!(
                "one stops: tree {}, walk {}",
                t.is_some(),
                w.is_some()
            )),
        }
    }

    /// Every record of `corpus` whose place is `part` modulo `parts` alike under the tree and
    /// the twin; at least `answered` of them answered by both (no vacuous pass).
    fn corpus_holds(
        corpus: &[u8],
        (part, parts): (usize, usize),
        records_in: usize,
        answered: usize,
    ) {
        let all = records(corpus).expect("the corpus reads");
        assert_eq!(all.len(), records_in, "the corpus's records");
        let mine: Vec<_> = all
            .iter()
            .enumerate()
            .filter(|(i, _)| i.checked_rem(parts) == Some(part))
            .map(|(_, r)| r)
            .collect();
        let mut answered_by_both = Vec::new();
        let mut failed = Vec::new();
        for r in &mine {
            let run = request_of(&r.args)
                .and_then(|(paths, request)| Ok((given_of(&paths, &r.files)?, request)))
                .and_then(|(files, request)| same(&files, &request));
            match run {
                Ok(both_answered) => answered_by_both.push(both_answered),
                Err(e) => failed.push(format!("{}/{} ({}): {e}", r.case, r.variant, r.law)),
            }
        }
        let both = answered_by_both.iter().filter(|a| **a).count();
        assert!(
            failed.is_empty(),
            "{} of {} records differ:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
        assert!(both >= answered, "{both} records answered by both");
    }

    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_the_kernel_corpus() {
        corpus_holds(KERNEL_CORPUS, (0, 1), 312, 312);
    }

    /// The m1 corpus, part 1 of 2 (records at 0 modulo 2, pick P70).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_the_m1_corpus_part_1() {
        corpus_holds(M1_CORPUS, (0, 2), 648, 324);
    }

    /// Part 2 of 2.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_the_m1_corpus_part_2() {
        corpus_holds(M1_CORPUS, (1, 2), 648, 324);
    }

    /// The fix vectors' groups of `m1/vectors.refcore`, one test each below, so that each runs
    /// well inside the mutants profile's time per test (pick P69: the whole store in one test
    /// passed 120 s on CI's runner).
    const FIX_GROUPS: [&str; 16] = [
        "fixes-g1",
        "fixes-g2",
        "fixes-g3",
        "fixes-g4",
        "fixes-g5",
        "fixes-g6",
        "fixes-g7",
        "fixes-g8",
        "fixes-g9",
        "fixes-g10",
        "fixes-g11",
        "fixes-g12",
        "fixes-g13",
        "fixes-g14",
        "fixes-open",
        "fixes-gate8",
    ];

    /// Every fix vector of `group` in `m1/vectors.refcore` whose place in the group is `part`
    /// modulo `parts` (a large group runs as several tests, together every vector), each ask
    /// alike under the tree and the twin; at least `answered` of them answered by both (no
    /// vacuous pass).
    fn fixes_hold(group: &str, (part, parts): (usize, usize), answered: usize) {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all
            .iter()
            .filter(|v| v.group == group)
            .enumerate()
            .filter(|(i, _)| i.checked_rem(parts) == Some(part))
            .map(|(_, v)| v)
            .collect();
        assert!(
            !mine.is_empty(),
            "the group {group} has vectors in part {part} of {parts}"
        );
        let mut answered_by_both = Vec::new();
        let mut failed = Vec::new();
        for v in &mine {
            let run = request_of(&v.args)
                .and_then(|(paths, request)| Ok((given_of(&paths, &v.files)?, request)))
                .and_then(|(files, request)| same(&files, &request));
            match run {
                Ok(both_answered) => answered_by_both.push(both_answered),
                Err(e) => failed.push(format!("{}: {e}", v.id)),
            }
        }
        let both = answered_by_both.iter().filter(|a| **a).count();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
        assert!(
            both >= answered,
            "{both} of {} vectors of {group} answered by both",
            mine.len()
        );
    }

    #[test]
    fn m1_lows_every_fix_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m1/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("fixes-"))
            .collect();
        let tested: BTreeSet<String> = FIX_GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each fixes- group of m1/vectors.tsv has its test here"
        );
    }

    /// G1's fix vectors (`m1/fixes/g1/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g1() {
        fixes_hold("fixes-g1", (0, 1), 17);
    }

    /// G2's fix vectors (`m1/fixes/g2/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g2() {
        fixes_hold("fixes-g2", (0, 1), 22);
    }

    /// G3's fix vectors (`m1/fixes/g3/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g3() {
        fixes_hold("fixes-g3", (0, 1), 27);
    }

    /// G4's fix vectors (`m1/fixes/g4/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g4() {
        fixes_hold("fixes-g4", (0, 1), 7);
    }

    /// G5's fix vectors (`m1/fixes/g5/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g5() {
        fixes_hold("fixes-g5", (0, 1), 17);
    }

    /// G6's fix vectors (`m1/fixes/g6/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g6() {
        fixes_hold("fixes-g6", (0, 1), 14);
    }

    /// G7's fix vectors, part 1 of 4 (at 0 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g7_part_1() {
        fixes_hold("fixes-g7", (0, 4), 8);
    }

    /// G7's fix vectors, part 2 of 4 (at 1 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g7_part_2() {
        fixes_hold("fixes-g7", (1, 4), 7);
    }

    /// G7's fix vectors, part 3 of 4 (at 2 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g7_part_3() {
        fixes_hold("fixes-g7", (2, 4), 7);
    }

    /// G7's fix vectors, part 4 of 4 (at 3 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g7_part_4() {
        fixes_hold("fixes-g7", (3, 4), 8);
    }

    /// G8's fix vectors (`m1/fixes/g8/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g8() {
        fixes_hold("fixes-g8", (0, 1), 95);
    }

    /// G9's fix vectors, part 1 of 36 (at 0 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_1() {
        fixes_hold("fixes-g9", (0, 36), 6);
    }

    /// G9's fix vectors, part 2 of 36 (at 1 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_2() {
        fixes_hold("fixes-g9", (1, 36), 6);
    }

    /// G9's fix vectors, part 3 of 36 (at 2 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_3() {
        fixes_hold("fixes-g9", (2, 36), 6);
    }

    /// G9's fix vectors, part 4 of 36 (at 3 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_4() {
        fixes_hold("fixes-g9", (3, 36), 6);
    }

    /// G9's fix vectors, part 5 of 36 (at 4 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_5() {
        fixes_hold("fixes-g9", (4, 36), 6);
    }

    /// G9's fix vectors, part 6 of 36 (at 5 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_6() {
        fixes_hold("fixes-g9", (5, 36), 6);
    }

    /// G9's fix vectors, part 7 of 36 (at 6 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_7() {
        fixes_hold("fixes-g9", (6, 36), 6);
    }

    /// G9's fix vectors, part 8 of 36 (at 7 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_8() {
        fixes_hold("fixes-g9", (7, 36), 6);
    }

    /// G9's fix vectors, part 9 of 36 (at 8 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_9() {
        fixes_hold("fixes-g9", (8, 36), 6);
    }

    /// G9's fix vectors, part 10 of 36 (at 9 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_10() {
        fixes_hold("fixes-g9", (9, 36), 6);
    }

    /// G9's fix vectors, part 11 of 36 (at 10 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_11() {
        fixes_hold("fixes-g9", (10, 36), 6);
    }

    /// G9's fix vectors, part 12 of 36 (at 11 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_12() {
        fixes_hold("fixes-g9", (11, 36), 6);
    }

    /// G9's fix vectors, part 13 of 36 (at 12 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_13() {
        fixes_hold("fixes-g9", (12, 36), 6);
    }

    /// G9's fix vectors, part 14 of 36 (at 13 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_14() {
        fixes_hold("fixes-g9", (13, 36), 6);
    }

    /// G9's fix vectors, part 15 of 36 (at 14 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_15() {
        fixes_hold("fixes-g9", (14, 36), 6);
    }

    /// G9's fix vectors, part 16 of 36 (at 15 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_16() {
        fixes_hold("fixes-g9", (15, 36), 6);
    }

    /// G9's fix vectors, part 17 of 36 (at 16 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_17() {
        fixes_hold("fixes-g9", (16, 36), 6);
    }

    /// G9's fix vectors, part 18 of 36 (at 17 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_18() {
        fixes_hold("fixes-g9", (17, 36), 6);
    }

    /// G9's fix vectors, part 19 of 36 (at 18 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_19() {
        fixes_hold("fixes-g9", (18, 36), 6);
    }

    /// G9's fix vectors, part 20 of 36 (at 19 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_20() {
        fixes_hold("fixes-g9", (19, 36), 6);
    }

    /// G9's fix vectors, part 21 of 36 (at 20 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_21() {
        fixes_hold("fixes-g9", (20, 36), 6);
    }

    /// G9's fix vectors, part 22 of 36 (at 21 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_22() {
        fixes_hold("fixes-g9", (21, 36), 6);
    }

    /// G9's fix vectors, part 23 of 36 (at 22 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_23() {
        fixes_hold("fixes-g9", (22, 36), 6);
    }

    /// G9's fix vectors, part 24 of 36 (at 23 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_24() {
        fixes_hold("fixes-g9", (23, 36), 6);
    }

    /// G9's fix vectors, part 25 of 36 (at 24 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_25() {
        fixes_hold("fixes-g9", (24, 36), 6);
    }

    /// G9's fix vectors, part 26 of 36 (at 25 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_26() {
        fixes_hold("fixes-g9", (25, 36), 6);
    }

    /// G9's fix vectors, part 27 of 36 (at 26 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_27() {
        fixes_hold("fixes-g9", (26, 36), 6);
    }

    /// G9's fix vectors, part 28 of 36 (at 27 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_28() {
        fixes_hold("fixes-g9", (27, 36), 6);
    }

    /// G9's fix vectors, part 29 of 36 (at 28 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_29() {
        fixes_hold("fixes-g9", (28, 36), 6);
    }

    /// G9's fix vectors, part 30 of 36 (at 29 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_30() {
        fixes_hold("fixes-g9", (29, 36), 6);
    }

    /// G9's fix vectors, part 31 of 36 (at 30 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_31() {
        fixes_hold("fixes-g9", (30, 36), 5);
    }

    /// G9's fix vectors, part 32 of 36 (at 31 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_32() {
        fixes_hold("fixes-g9", (31, 36), 5);
    }

    /// G9's fix vectors, part 33 of 36 (at 32 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_33() {
        fixes_hold("fixes-g9", (32, 36), 5);
    }

    /// G9's fix vectors, part 34 of 36 (at 33 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_34() {
        fixes_hold("fixes-g9", (33, 36), 5);
    }

    /// G9's fix vectors, part 35 of 36 (at 34 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_35() {
        fixes_hold("fixes-g9", (34, 36), 5);
    }

    /// G9's fix vectors, part 36 of 36 (at 35 modulo 36).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g9_part_36() {
        fixes_hold("fixes-g9", (35, 36), 5);
    }

    /// G10's fix vectors (`m1/fixes/g10/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g10() {
        fixes_hold("fixes-g10", (0, 1), 101);
    }

    /// G11's fix vectors (`m1/fixes/g11/`), alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g11() {
        fixes_hold("fixes-g11", (0, 1), 91);
    }

    /// G12's fix vectors, part 1 of 47 (at 0 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_1() {
        fixes_hold("fixes-g12", (0, 47), 5);
    }

    /// G12's fix vectors, part 2 of 47 (at 1 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_2() {
        fixes_hold("fixes-g12", (1, 47), 5);
    }

    /// G12's fix vectors, part 3 of 47 (at 2 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_3() {
        fixes_hold("fixes-g12", (2, 47), 5);
    }

    /// G12's fix vectors, part 4 of 47 (at 3 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_4() {
        fixes_hold("fixes-g12", (3, 47), 5);
    }

    /// G12's fix vectors, part 5 of 47 (at 4 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_5() {
        fixes_hold("fixes-g12", (4, 47), 5);
    }

    /// G12's fix vectors, part 6 of 47 (at 5 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_6() {
        fixes_hold("fixes-g12", (5, 47), 5);
    }

    /// G12's fix vectors, part 7 of 47 (at 6 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_7() {
        fixes_hold("fixes-g12", (6, 47), 5);
    }

    /// G12's fix vectors, part 8 of 47 (at 7 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_8() {
        fixes_hold("fixes-g12", (7, 47), 5);
    }

    /// G12's fix vectors, part 9 of 47 (at 8 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_9() {
        fixes_hold("fixes-g12", (8, 47), 5);
    }

    /// G12's fix vectors, part 10 of 47 (at 9 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_10() {
        fixes_hold("fixes-g12", (9, 47), 5);
    }

    /// G12's fix vectors, part 11 of 47 (at 10 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_11() {
        fixes_hold("fixes-g12", (10, 47), 5);
    }

    /// G12's fix vectors, part 12 of 47 (at 11 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_12() {
        fixes_hold("fixes-g12", (11, 47), 5);
    }

    /// G12's fix vectors, part 13 of 47 (at 12 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_13() {
        fixes_hold("fixes-g12", (12, 47), 5);
    }

    /// G12's fix vectors, part 14 of 47 (at 13 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_14() {
        fixes_hold("fixes-g12", (13, 47), 5);
    }

    /// G12's fix vectors, part 15 of 47 (at 14 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_15() {
        fixes_hold("fixes-g12", (14, 47), 5);
    }

    /// G12's fix vectors, part 16 of 47 (at 15 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_16() {
        fixes_hold("fixes-g12", (15, 47), 5);
    }

    /// G12's fix vectors, part 17 of 47 (at 16 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_17() {
        fixes_hold("fixes-g12", (16, 47), 5);
    }

    /// G12's fix vectors, part 18 of 47 (at 17 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_18() {
        fixes_hold("fixes-g12", (17, 47), 5);
    }

    /// G12's fix vectors, part 19 of 47 (at 18 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_19() {
        fixes_hold("fixes-g12", (18, 47), 5);
    }

    /// G12's fix vectors, part 20 of 47 (at 19 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_20() {
        fixes_hold("fixes-g12", (19, 47), 5);
    }

    /// G12's fix vectors, part 21 of 47 (at 20 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_21() {
        fixes_hold("fixes-g12", (20, 47), 5);
    }

    /// G12's fix vectors, part 22 of 47 (at 21 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_22() {
        fixes_hold("fixes-g12", (21, 47), 5);
    }

    /// G12's fix vectors, part 23 of 47 (at 22 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_23() {
        fixes_hold("fixes-g12", (22, 47), 5);
    }

    /// G12's fix vectors, part 24 of 47 (at 23 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_24() {
        fixes_hold("fixes-g12", (23, 47), 5);
    }

    /// G12's fix vectors, part 25 of 47 (at 24 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_25() {
        fixes_hold("fixes-g12", (24, 47), 5);
    }

    /// G12's fix vectors, part 26 of 47 (at 25 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_26() {
        fixes_hold("fixes-g12", (25, 47), 5);
    }

    /// G12's fix vectors, part 27 of 47 (at 26 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_27() {
        fixes_hold("fixes-g12", (26, 47), 5);
    }

    /// G12's fix vectors, part 28 of 47 (at 27 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_28() {
        fixes_hold("fixes-g12", (27, 47), 5);
    }

    /// G12's fix vectors, part 29 of 47 (at 28 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_29() {
        fixes_hold("fixes-g12", (28, 47), 5);
    }

    /// G12's fix vectors, part 30 of 47 (at 29 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_30() {
        fixes_hold("fixes-g12", (29, 47), 5);
    }

    /// G12's fix vectors, part 31 of 47 (at 30 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_31() {
        fixes_hold("fixes-g12", (30, 47), 5);
    }

    /// G12's fix vectors, part 32 of 47 (at 31 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_32() {
        fixes_hold("fixes-g12", (31, 47), 5);
    }

    /// G12's fix vectors, part 33 of 47 (at 32 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_33() {
        fixes_hold("fixes-g12", (32, 47), 5);
    }

    /// G12's fix vectors, part 34 of 47 (at 33 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_34() {
        fixes_hold("fixes-g12", (33, 47), 5);
    }

    /// G12's fix vectors, part 35 of 47 (at 34 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_35() {
        fixes_hold("fixes-g12", (34, 47), 5);
    }

    /// G12's fix vectors, part 36 of 47 (at 35 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_36() {
        fixes_hold("fixes-g12", (35, 47), 5);
    }

    /// G12's fix vectors, part 37 of 47 (at 36 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_37() {
        fixes_hold("fixes-g12", (36, 47), 5);
    }

    /// G12's fix vectors, part 38 of 47 (at 37 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_38() {
        fixes_hold("fixes-g12", (37, 47), 5);
    }

    /// G12's fix vectors, part 39 of 47 (at 38 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_39() {
        fixes_hold("fixes-g12", (38, 47), 5);
    }

    /// G12's fix vectors, part 40 of 47 (at 39 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_40() {
        fixes_hold("fixes-g12", (39, 47), 5);
    }

    /// G12's fix vectors, part 41 of 47 (at 40 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_41() {
        fixes_hold("fixes-g12", (40, 47), 5);
    }

    /// G12's fix vectors, part 42 of 47 (at 41 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_42() {
        fixes_hold("fixes-g12", (41, 47), 5);
    }

    /// G12's fix vectors, part 43 of 47 (at 42 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_43() {
        fixes_hold("fixes-g12", (42, 47), 5);
    }

    /// G12's fix vectors, part 44 of 47 (at 43 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_44() {
        fixes_hold("fixes-g12", (43, 47), 5);
    }

    /// G12's fix vectors, part 45 of 47 (at 44 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_45() {
        fixes_hold("fixes-g12", (44, 47), 5);
    }

    /// G12's fix vectors, part 46 of 47 (at 45 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_46() {
        fixes_hold("fixes-g12", (45, 47), 5);
    }

    /// G12's fix vectors, part 47 of 47 (at 46 modulo 47).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g12_part_47() {
        fixes_hold("fixes-g12", (46, 47), 5);
    }

    /// G13's fix vectors, part 1 of 4 (at 0 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g13_part_1() {
        fixes_hold("fixes-g13", (0, 4), 85);
    }

    /// G13's fix vectors, part 2 of 4 (at 1 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g13_part_2() {
        fixes_hold("fixes-g13", (1, 4), 84);
    }

    /// G13's fix vectors, part 3 of 4 (at 2 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g13_part_3() {
        fixes_hold("fixes-g13", (2, 4), 84);
    }

    /// G13's fix vectors, part 4 of 4 (at 3 modulo 4).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g13_part_4() {
        fixes_hold("fixes-g13", (3, 4), 84);
    }

    /// G14's fix vectors, part 1 of 6 (at 0 modulo 6).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g14_part_1() {
        fixes_hold("fixes-g14", (0, 6), 32);
    }

    /// G14's fix vectors, part 2 of 6 (at 1 modulo 6).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g14_part_2() {
        fixes_hold("fixes-g14", (1, 6), 32);
    }

    /// G14's fix vectors, part 3 of 6 (at 2 modulo 6).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g14_part_3() {
        fixes_hold("fixes-g14", (2, 6), 32);
    }

    /// G14's fix vectors, part 4 of 6 (at 3 modulo 6).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g14_part_4() {
        fixes_hold("fixes-g14", (3, 6), 31);
    }

    /// G14's fix vectors, part 5 of 6 (at 4 modulo 6).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g14_part_5() {
        fixes_hold("fixes-g14", (4, 6), 31);
    }

    /// G14's fix vectors, part 6 of 6 (at 5 modulo 6).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_g14_part_6() {
        fixes_hold("fixes-g14", (5, 6), 31);
    }

    /// The reviews' findings kept apart from their groups (`fixes-open`).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_open() {
        fixes_hold("fixes-open", (0, 1), 3);
    }

    /// Gate 8's open survivors' texts (`m1/fixes/gate8/`).
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_fixes_gate8() {
        fixes_hold("fixes-gate8", (0, 1), 7);
    }

    /// F5's own realities, every ask of each.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_f5s_realities() {
        let every = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let pair: Vec<(String, Vec<u8>)> = LOWS
            .iter()
            .map(|(p, t)| ((*p).to_owned(), t.as_bytes().to_vec()))
            .collect();
        let mut sets = vec![pair];
        for (p, t) in ALONE {
            sets.push(vec![(p.to_owned(), t.as_bytes().to_vec())]);
        }
        let failed: Vec<String> = sets
            .iter()
            .filter_map(|files| match same(files, &every) {
                Ok(true) => None,
                Ok(false) => Some(format!("{:?}: not answered", files.first().map(|f| &f.0))),
                Err(e) => Some(format!("{:?}: {e}", files.first().map(|f| &f.0))),
            })
            .collect();
        assert!(failed.is_empty(), "F5's realities:\n{}", failed.join("\n"));
    }

    /// Fence F5's range minimum through its public entry (`tally::walk::RangeMinimum`, P8-5's
    /// bench's, pick P56): built over days in order, one step each, it answers every window's
    /// lowest change as the plain running sums give it (the least balance change from `counts`
    /// on a day `first` through `last`, P §2.3, E §4.3); days out of order, repeated, or a step
    /// count other than the days' are refused (gate 8's open survivor, `RangeMinimum::of`).
    #[test]
    fn m1_lows_the_range_minimum_answers_as_the_running_sums() {
        let cents: [i64; 6] = [-500, 300, -1000, 250, 400, -50];
        let days: Vec<Day> = (1..=6)
            .map(|d| Day::from_ymd(2026, 1, d).expect("a day of January"))
            .collect();
        let num = |c: i64| Num::ratio(c, 100).expect("a figure in cents");
        let steps: Vec<Num> = cents.iter().map(|c| num(*c)).collect();
        let cent = num(1);
        let tree = RangeMinimum::of(days.clone(), steps.clone(), Some(&cent))
            .expect("a range minimum over days in order");
        assert!(tree.in_minor_units(), "steps of whole cents sum in cents");
        for c in 0..days.len() {
            for f in c..days.len() {
                for l in f..days.len() {
                    let running = |to: usize| -> i64 { cents[c..=to].iter().sum() };
                    let plain = (f..=l).map(running).min().expect("a window's days");
                    let got = tree
                        .lowest_change(days[c], (days[f], days[l]))
                        .expect("a window's lowest change");
                    assert_eq!(got, num(plain), "counts day {c}, window {f}..{l}");
                }
            }
        }
        let mut swapped = days.clone();
        swapped.swap(1, 2);
        assert!(
            RangeMinimum::of(swapped, steps.clone(), Some(&cent)).is_err(),
            "days out of order"
        );
        let mut repeated = days.clone();
        repeated[2] = repeated[1];
        assert!(
            RangeMinimum::of(repeated, steps.clone(), Some(&cent)).is_err(),
            "a day repeated"
        );
        assert!(
            RangeMinimum::of(days, steps[1..].to_vec(), Some(&cent)).is_err(),
            "a step short"
        );
    }
}
