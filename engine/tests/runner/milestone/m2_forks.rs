//! Milestone M2's fence F6, the forks, on T's realities (chunk C8; pick P9; R-130, R-84;
//! `MILESTONE-M2.md`'s forks row; issue #167). Every vector of `kernel/vectors.refcore`,
//! `m1/vectors.refcore`, `m2/vectors.refcore` and `m2/spec.vectors`, and every record of the four
//! corpora (`properties/kernel.corpus`, `m0.corpus`, `m1.corpus`, `m2.corpus`), runs through
//! `tally_cli::tally::run_with_forks` twice (the CLI re-exports the library, pick P59), with
//! `Forks::On` (fence F6: timelines that agree up to a day walk it once, each continued from the
//! last checkpoint before it, PERFORMANCE §2.5) and with `Forks::Off` (the unforked walk, its
//! plain twin), and the two print the same lines. A difference is the fence's fault. A store's
//! groups and a corpus run in parts by place modulo the parts (as `m1_lows`' C11–C13), each part
//! about 0.4 s of the quick engine's time at 2026-11-15, so it stays well inside the mutants
//! profile's time per test; each part asks at least as many runs answered by both as the engine
//! answered then (no vacuous pass). Written by T's `gen_forks.py` (`tmp/m2/t/`) from each item's
//! time, re-timed once G7 built the switch (9f98aa56); a fence's equivalence test (R-130), it runs
//! by default since then.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_cli::tally::walk::Forks;
    use tally_cli::tally::{Asks, Days, Given, NotImplemented, Pin, Request, run_with_forks};
    use tally_runner::corpus::{KERNEL_CORPUS, netstrings, records};
    use tally_runner::kernel::vectors;
    use tally_runner::laws_m0::M0_CORPUS;
    use tally_runner::laws_m1::M1_CORPUS;
    use tally_runner::laws_m2::M2_CORPUS;

    /// The four vector stores, as the runner embeds them.
    const KERNEL_VECTORS: &[u8] = tally_runner::kernel::VECTORS;
    const M1_VECTORS: &[u8] = tally_runner::m1::VECTORS;
    const M2_VECTORS: &[u8] = tally_runner::m2::VECTORS;
    const M2_SPEC: &[u8] = tally_runner::m2::SPEC;

    /// A command line's paths and request, as the CLI reads `run`'s words (INTERFACE §4); every
    /// item of the eight stores is a `run` with at most `--ask` (2026-11-15).
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
            } else if word.starts_with("--") {
                return Err(format!("an option this target doesn't read: {word}"));
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

    /// The lines of one run with `forks`, or `None` where it stops not implemented.
    fn lines(files: &[(String, Vec<u8>)], request: &Request, forks: Forks) -> Option<Vec<String>> {
        let given = files
            .iter()
            .map(|(path, bytes)| Given {
                path: path.clone(),
                bytes: Some(bytes.clone()),
            })
            .collect();
        match run_with_forks(given, request, forks) {
            Ok(answer) => Some(answer.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    /// Runs `files` and `request` with forks on and off; `Err` names the first line they
    /// differ at. `Ok(true)` where both answered, `Ok(false)` where both stopped.
    fn same(files: &[(String, Vec<u8>)], request: &Request) -> Result<bool, String> {
        let on = lines(files, request, Forks::On);
        let off = lines(files, request, Forks::Off);
        match (on, off) {
            (None, None) => Ok(false),
            (Some(a), Some(b)) if a == b => Ok(true),
            (Some(a), Some(b)) => {
                let line = match a.iter().zip(&b).position(|(x, y)| x != y) {
                    Some(i) => i,
                    None => a.len().min(b.len()),
                };
                Err(format!(
                    "line {}: on {:?}, off {:?}",
                    line.checked_add(1).expect("a line number"),
                    a.get(line),
                    b.get(line)
                ))
            }
            (a, b) => Err(format!(
                "one stops: on {}, off {}",
                a.is_some(),
                b.is_some()
            )),
        }
    }

    /// One item's command line and file table, run with forks on and off.
    fn item_same(args: &[String], table: &[u8]) -> Result<bool, String> {
        request_of(args)
            .and_then(|(paths, request)| Ok((given_of(&paths, table)?, request)))
            .and_then(|(files, request)| same(&files, &request))
    }

    /// Fails with every difference, and where fewer than `answered` runs were answered by both.
    fn judge(what: &str, results: Vec<(String, Result<bool, String>)>, answered: usize) {
        let n = results.len();
        let mut both = 0_usize;
        let mut failed = Vec::new();
        for (name, r) in results {
            match r {
                Ok(true) => both = both.checked_add(1).expect("a count"),
                Ok(false) => {}
                Err(e) => failed.push(format!("{name}: {e}")),
            }
        }
        assert!(
            failed.is_empty(),
            "{} of {n} runs of {what} differ with forks on and off:\n{}",
            failed.len(),
            failed.join("\n")
        );
        assert!(
            both >= answered,
            "{both} of {n} runs of {what} answered by both, {answered} expected"
        );
    }

    /// Every record of `corpus` whose place is `part` modulo `parts` alike with forks on and
    /// off; the corpus has `records_in` records; at least `answered` answered by both.
    fn corpus_holds(
        corpus: &[u8],
        (part, parts): (usize, usize),
        records_in: usize,
        answered: usize,
    ) {
        let all = records(corpus).expect("the corpus reads");
        assert_eq!(all.len(), records_in, "the corpus's records");
        let results = all
            .iter()
            .enumerate()
            .filter(|(i, _)| i.checked_rem(parts) == Some(part))
            .map(|(_, r)| {
                let name = format!("{}/{} ({})", r.case, r.variant, r.law);
                (name, item_same(&r.args, &r.files))
            })
            .collect();
        judge(&format!("part {part} of {parts}"), results, answered);
    }

    /// Every vector of `group` in `store` whose place in the group is `part` modulo `parts`
    /// alike with forks on and off; at least `answered` answered by both.
    fn group_holds(store: &[u8], group: &str, (part, parts): (usize, usize), answered: usize) {
        let all = vectors(store).expect("the store reads");
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
        let results = mine
            .iter()
            .map(|v| (v.id.clone(), item_same(&v.args, &v.files)))
            .collect();
        judge(
            &format!("{group}, part {part} of {parts}"),
            results,
            answered,
        );
    }

    /// The groups of `store`, as `vectors` reads them.
    fn groups_of(store: &[u8]) -> BTreeSet<String> {
        vectors(store)
            .expect("the store reads")
            .into_iter()
            .map(|v| v.group)
            .collect()
    }

    /// The groups of `kernel/vectors.refcore`, each tested below.
    const KERNEL_GROUPS: [&str; 15] = [
        "checks", "g1", "g2", "g3", "g4", "g5", "g9", "g6", "g7", "g8", "g10", "g11", "g12", "g13",
        "g14",
    ];

    /// The groups of `m1/vectors.refcore`, each tested below.
    const M1_GROUPS: [&str; 55] = [
        "checks-g1",
        "checks-g2",
        "checks-g3",
        "checks-g4",
        "checks-g5",
        "checks-g6",
        "checks-g7",
        "checks-g8",
        "checks-g10",
        "checks-g11",
        "checks-green",
        "goldens-pins",
        "goldens-g6",
        "goldens-g7",
        "goldens-g10",
        "goldens-y22",
        "goldens-business",
        "goldens-g8",
        "goldens-g9",
        "carried-5",
        "carried-6",
        "carried-1",
        "carried-3",
        "carried-7",
        "carried-2",
        "carried-4",
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
        "fixes-r1",
        "fixes-r2",
        "fixes-r3",
        "fixes-r4",
        "fixes-r5",
        "fixes-r6",
        "fixes-r7",
        "fixes-r8",
        "fixes-r9",
        "fixes-r10",
        "fixes-r11",
        "fixes-b",
        "fixes-s",
    ];

    /// The groups of `m2/vectors.refcore`, each tested below.
    const M2_GROUPS: [&str; 34] = [
        "checks-green",
        "checks-records",
        "checks-forms",
        "items-green",
        "items-forms",
        "items-explain",
        "items-windows",
        "fixes-forms",
        "fixes-sweep",
        "fixes-explain",
        "fixes-windows",
        "items-sweep",
        "goldens-records",
        "goldens-ledger",
        "fixes-records",
        "fixes-ledger",
        "fixes-r11",
        "fixes-r12",
        "fixes-r13",
        "fixes-r14",
        "fixes-r15",
        "fixes-r16",
        "fixes-s",
        "fixes-d",
        "fixes-cut",
        "fixes-rv",
        "hindcasts-h0",
        "hindcasts-h1",
        "hindcasts-h2",
        "hindcasts-h3",
        "hindcasts-h4",
        "hindcasts-h5",
        "hindcasts-h8",
        "hindcasts-h9",
    ];

    /// The groups of `m2/spec.vectors`, each tested below.
    const SPEC_GROUPS: [&str; 3] = ["spec-forms", "spec-records", "spec-sweep"];

    /// Each group of the four stores has its tests here, so a new group is never left out.
    #[test]
    fn m2_forks_every_group_is_tested() {
        assert_eq!(
            groups_of(KERNEL_VECTORS),
            KERNEL_GROUPS.iter().map(|g| (*g).to_owned()).collect(),
            "each group of kernel/vectors.refcore has its tests here"
        );
        assert_eq!(
            groups_of(M1_VECTORS),
            M1_GROUPS.iter().map(|g| (*g).to_owned()).collect(),
            "each group of m1/vectors.refcore has its tests here"
        );
        assert_eq!(
            groups_of(M2_VECTORS),
            M2_GROUPS.iter().map(|g| (*g).to_owned()).collect(),
            "each group of m2/vectors.refcore has its tests here"
        );
        assert_eq!(
            groups_of(M2_SPEC),
            SPEC_GROUPS.iter().map(|g| (*g).to_owned()).collect(),
            "each group of m2/spec.vectors has its tests here"
        );
    }

    /// The kernel corpus (`properties/kernel.corpus`), part 1 of 2 (records at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_corpus_part_1() {
        corpus_holds(KERNEL_CORPUS, (0, 2), 312, 156);
    }

    /// The kernel corpus (`properties/kernel.corpus`), part 2 of 2 (records at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_corpus_part_2() {
        corpus_holds(KERNEL_CORPUS, (1, 2), 312, 156);
    }

    /// M0's corpus (`properties/m0.corpus`), part 1 of 2 (records at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m0_corpus_part_1() {
        corpus_holds(M0_CORPUS, (0, 2), 285, 143);
    }

    /// M0's corpus (`properties/m0.corpus`), part 2 of 2 (records at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m0_corpus_part_2() {
        corpus_holds(M0_CORPUS, (1, 2), 285, 142);
    }

    /// M1's corpus (`properties/m1.corpus`), part 1 of 7 (records at 0 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_1() {
        corpus_holds(M1_CORPUS, (0, 7), 648, 93);
    }

    /// M1's corpus (`properties/m1.corpus`), part 2 of 7 (records at 1 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_2() {
        corpus_holds(M1_CORPUS, (1, 7), 648, 93);
    }

    /// M1's corpus (`properties/m1.corpus`), part 3 of 7 (records at 2 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_3() {
        corpus_holds(M1_CORPUS, (2, 7), 648, 93);
    }

    /// M1's corpus (`properties/m1.corpus`), part 4 of 7 (records at 3 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_4() {
        corpus_holds(M1_CORPUS, (3, 7), 648, 93);
    }

    /// M1's corpus (`properties/m1.corpus`), part 5 of 7 (records at 4 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_5() {
        corpus_holds(M1_CORPUS, (4, 7), 648, 92);
    }

    /// M1's corpus (`properties/m1.corpus`), part 6 of 7 (records at 5 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_6() {
        corpus_holds(M1_CORPUS, (5, 7), 648, 92);
    }

    /// M1's corpus (`properties/m1.corpus`), part 7 of 7 (records at 6 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_corpus_part_7() {
        corpus_holds(M1_CORPUS, (6, 7), 648, 92);
    }

    /// M2's corpus (`properties/m2.corpus`), part 1 of 3 (records at 0 modulo 3).
    #[test]
    fn m2_forks_on_prints_off_on_m2_corpus_part_1() {
        corpus_holds(M2_CORPUS, (0, 3), 168, 56);
    }

    /// M2's corpus (`properties/m2.corpus`), part 2 of 3 (records at 1 modulo 3).
    #[test]
    fn m2_forks_on_prints_off_on_m2_corpus_part_2() {
        corpus_holds(M2_CORPUS, (1, 3), 168, 56);
    }

    /// M2's corpus (`properties/m2.corpus`), part 3 of 3 (records at 2 modulo 3).
    #[test]
    fn m2_forks_on_prints_off_on_m2_corpus_part_3() {
        corpus_holds(M2_CORPUS, (2, 3), 168, 56);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 1 of 22 (at 0 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_1() {
        group_holds(KERNEL_VECTORS, "checks", (0, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 2 of 22 (at 1 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_2() {
        group_holds(KERNEL_VECTORS, "checks", (1, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 3 of 22 (at 2 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_3() {
        group_holds(KERNEL_VECTORS, "checks", (2, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 4 of 22 (at 3 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_4() {
        group_holds(KERNEL_VECTORS, "checks", (3, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 5 of 22 (at 4 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_5() {
        group_holds(KERNEL_VECTORS, "checks", (4, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 6 of 22 (at 5 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_6() {
        group_holds(KERNEL_VECTORS, "checks", (5, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 7 of 22 (at 6 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_7() {
        group_holds(KERNEL_VECTORS, "checks", (6, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 8 of 22 (at 7 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_8() {
        group_holds(KERNEL_VECTORS, "checks", (7, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 9 of 22 (at 8 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_9() {
        group_holds(KERNEL_VECTORS, "checks", (8, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 10 of 22 (at 9 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_10() {
        group_holds(KERNEL_VECTORS, "checks", (9, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 11 of 22 (at 10 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_11() {
        group_holds(KERNEL_VECTORS, "checks", (10, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 12 of 22 (at 11 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_12() {
        group_holds(KERNEL_VECTORS, "checks", (11, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 13 of 22 (at 12 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_13() {
        group_holds(KERNEL_VECTORS, "checks", (12, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 14 of 22 (at 13 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_14() {
        group_holds(KERNEL_VECTORS, "checks", (13, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 15 of 22 (at 14 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_15() {
        group_holds(KERNEL_VECTORS, "checks", (14, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 16 of 22 (at 15 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_16() {
        group_holds(KERNEL_VECTORS, "checks", (15, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 17 of 22 (at 16 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_17() {
        group_holds(KERNEL_VECTORS, "checks", (16, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 18 of 22 (at 17 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_18() {
        group_holds(KERNEL_VECTORS, "checks", (17, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 19 of 22 (at 18 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_19() {
        group_holds(KERNEL_VECTORS, "checks", (18, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 20 of 22 (at 19 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_20() {
        group_holds(KERNEL_VECTORS, "checks", (19, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 21 of 22 (at 20 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_21() {
        group_holds(KERNEL_VECTORS, "checks", (20, 22), 2);
    }

    /// `kernel/vectors.refcore`'s group `checks`, part 22 of 22 (at 21 modulo 22).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_checks_part_22() {
        group_holds(KERNEL_VECTORS, "checks", (21, 22), 1);
    }

    /// `kernel/vectors.refcore`'s group `g1`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g1() {
        group_holds(KERNEL_VECTORS, "g1", (0, 1), 15);
    }

    /// `kernel/vectors.refcore`'s group `g2`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g2() {
        group_holds(KERNEL_VECTORS, "g2", (0, 1), 3);
    }

    /// `kernel/vectors.refcore`'s group `g3`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g3() {
        group_holds(KERNEL_VECTORS, "g3", (0, 1), 2);
    }

    /// `kernel/vectors.refcore`'s group `g4`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g4() {
        group_holds(KERNEL_VECTORS, "g4", (0, 1), 1);
    }

    /// `kernel/vectors.refcore`'s group `g5`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g5() {
        group_holds(KERNEL_VECTORS, "g5", (0, 1), 13);
    }

    /// `kernel/vectors.refcore`'s group `g9`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g9() {
        group_holds(KERNEL_VECTORS, "g9", (0, 1), 5);
    }

    /// `kernel/vectors.refcore`'s group `g6`, every vector (17 since tally 2.2 dropped
    /// `g6/refusal_order_later_leg`: a leg has no day of its own, S-6).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g6() {
        group_holds(KERNEL_VECTORS, "g6", (0, 1), 17);
    }

    /// `kernel/vectors.refcore`'s group `g7`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g7() {
        group_holds(KERNEL_VECTORS, "g7", (0, 1), 10);
    }

    /// `kernel/vectors.refcore`'s group `g8`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g8() {
        group_holds(KERNEL_VECTORS, "g8", (0, 1), 5);
    }

    /// `kernel/vectors.refcore`'s group `g10`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g10() {
        group_holds(KERNEL_VECTORS, "g10", (0, 1), 6);
    }

    /// `kernel/vectors.refcore`'s group `g11`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g11() {
        group_holds(KERNEL_VECTORS, "g11", (0, 1), 11);
    }

    /// `kernel/vectors.refcore`'s group `g12`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g12() {
        group_holds(KERNEL_VECTORS, "g12", (0, 1), 2);
    }

    /// `kernel/vectors.refcore`'s group `g13`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g13() {
        group_holds(KERNEL_VECTORS, "g13", (0, 1), 11);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 1 of 20 (at 0 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_1() {
        group_holds(KERNEL_VECTORS, "g14", (0, 20), 2);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 2 of 20 (at 1 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_2() {
        group_holds(KERNEL_VECTORS, "g14", (1, 20), 2);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 3 of 20 (at 2 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_3() {
        group_holds(KERNEL_VECTORS, "g14", (2, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 4 of 20 (at 3 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_4() {
        group_holds(KERNEL_VECTORS, "g14", (3, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 5 of 20 (at 4 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_5() {
        group_holds(KERNEL_VECTORS, "g14", (4, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 6 of 20 (at 5 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_6() {
        group_holds(KERNEL_VECTORS, "g14", (5, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 7 of 20 (at 6 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_7() {
        group_holds(KERNEL_VECTORS, "g14", (6, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 8 of 20 (at 7 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_8() {
        group_holds(KERNEL_VECTORS, "g14", (7, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 9 of 20 (at 8 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_9() {
        group_holds(KERNEL_VECTORS, "g14", (8, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 10 of 20 (at 9 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_10() {
        group_holds(KERNEL_VECTORS, "g14", (9, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 11 of 20 (at 10 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_11() {
        group_holds(KERNEL_VECTORS, "g14", (10, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 12 of 20 (at 11 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_12() {
        group_holds(KERNEL_VECTORS, "g14", (11, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 13 of 20 (at 12 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_13() {
        group_holds(KERNEL_VECTORS, "g14", (12, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 14 of 20 (at 13 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_14() {
        group_holds(KERNEL_VECTORS, "g14", (13, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 15 of 20 (at 14 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_15() {
        group_holds(KERNEL_VECTORS, "g14", (14, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 16 of 20 (at 15 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_16() {
        group_holds(KERNEL_VECTORS, "g14", (15, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 17 of 20 (at 16 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_17() {
        group_holds(KERNEL_VECTORS, "g14", (16, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 18 of 20 (at 17 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_18() {
        group_holds(KERNEL_VECTORS, "g14", (17, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 19 of 20 (at 18 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_19() {
        group_holds(KERNEL_VECTORS, "g14", (18, 20), 1);
    }

    /// `kernel/vectors.refcore`'s group `g14`, part 20 of 20 (at 19 modulo 20).
    #[test]
    fn m2_forks_on_prints_off_on_kernel_g14_part_20() {
        group_holds(KERNEL_VECTORS, "g14", (19, 20), 1);
    }

    /// `m1/vectors.refcore`'s group `checks-g1`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g1_part_1() {
        group_holds(M1_VECTORS, "checks-g1", (0, 2), 5);
    }

    /// `m1/vectors.refcore`'s group `checks-g1`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g1_part_2() {
        group_holds(M1_VECTORS, "checks-g1", (1, 2), 4);
    }

    /// `m1/vectors.refcore`'s group `checks-g2`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g2() {
        group_holds(M1_VECTORS, "checks-g2", (0, 1), 7);
    }

    /// `m1/vectors.refcore`'s group `checks-g3`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g3() {
        group_holds(M1_VECTORS, "checks-g3", (0, 1), 6);
    }

    /// `m1/vectors.refcore`'s group `checks-g4`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g4() {
        group_holds(M1_VECTORS, "checks-g4", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `checks-g5`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g5() {
        group_holds(M1_VECTORS, "checks-g5", (0, 1), 4);
    }

    /// `m1/vectors.refcore`'s group `checks-g6`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g6() {
        group_holds(M1_VECTORS, "checks-g6", (0, 1), 4);
    }

    /// `m1/vectors.refcore`'s group `checks-g7`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g7() {
        group_holds(M1_VECTORS, "checks-g7", (0, 1), 4);
    }

    /// `m1/vectors.refcore`'s group `checks-g8`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g8() {
        group_holds(M1_VECTORS, "checks-g8", (0, 1), 4);
    }

    /// `m1/vectors.refcore`'s group `checks-g10`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g10() {
        group_holds(M1_VECTORS, "checks-g10", (0, 1), 4);
    }

    /// `m1/vectors.refcore`'s group `checks-g11`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_g11() {
        group_holds(M1_VECTORS, "checks-g11", (0, 1), 5);
    }

    /// `m1/vectors.refcore`'s group `checks-green`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_checks_green() {
        group_holds(M1_VECTORS, "checks-green", (0, 1), 17);
    }

    /// `m1/vectors.refcore`'s group `goldens-pins`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_pins() {
        group_holds(M1_VECTORS, "goldens-pins", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-g6`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_g6() {
        group_holds(M1_VECTORS, "goldens-g6", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-g7`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_g7() {
        group_holds(M1_VECTORS, "goldens-g7", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-g10`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_g10() {
        group_holds(M1_VECTORS, "goldens-g10", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-y22`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_y22() {
        group_holds(M1_VECTORS, "goldens-y22", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-business`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_business() {
        group_holds(M1_VECTORS, "goldens-business", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-g8`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_g8() {
        group_holds(M1_VECTORS, "goldens-g8", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `goldens-g9`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_goldens_g9() {
        group_holds(M1_VECTORS, "goldens-g9", (0, 1), 2);
    }

    /// `m1/vectors.refcore`'s group `carried-5`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_5() {
        group_holds(M1_VECTORS, "carried-5", (0, 1), 11);
    }

    /// `m1/vectors.refcore`'s group `carried-6`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_6() {
        group_holds(M1_VECTORS, "carried-6", (0, 1), 12);
    }

    /// `m1/vectors.refcore`'s group `carried-1`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_1() {
        group_holds(M1_VECTORS, "carried-1", (0, 1), 6);
    }

    /// `m1/vectors.refcore`'s group `carried-3`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_3() {
        group_holds(M1_VECTORS, "carried-3", (0, 1), 4);
    }

    /// `m1/vectors.refcore`'s group `carried-7`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_7() {
        group_holds(M1_VECTORS, "carried-7", (0, 1), 1);
    }

    /// `m1/vectors.refcore`'s group `carried-2`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_2() {
        group_holds(M1_VECTORS, "carried-2", (0, 1), 6);
    }

    /// `m1/vectors.refcore`'s group `carried-4`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_carried_4() {
        group_holds(M1_VECTORS, "carried-4", (0, 1), 12);
    }

    /// `m1/vectors.refcore`'s group `fixes-g1`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g1() {
        group_holds(M1_VECTORS, "fixes-g1", (0, 1), 17);
    }

    /// `m1/vectors.refcore`'s group `fixes-g2`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g2() {
        group_holds(M1_VECTORS, "fixes-g2", (0, 1), 22);
    }

    /// `m1/vectors.refcore`'s group `fixes-g3`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g3() {
        group_holds(M1_VECTORS, "fixes-g3", (0, 1), 27);
    }

    /// `m1/vectors.refcore`'s group `fixes-g4`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g4() {
        group_holds(M1_VECTORS, "fixes-g4", (0, 1), 7);
    }

    /// `m1/vectors.refcore`'s group `fixes-g5`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g5() {
        group_holds(M1_VECTORS, "fixes-g5", (0, 1), 17);
    }

    /// `m1/vectors.refcore`'s group `fixes-g6`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g6() {
        group_holds(M1_VECTORS, "fixes-g6", (0, 1), 14);
    }

    /// `m1/vectors.refcore`'s group `fixes-g7`, part 1 of 6 (at 0 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g7_part_1() {
        group_holds(M1_VECTORS, "fixes-g7", (0, 6), 6);
    }

    /// `m1/vectors.refcore`'s group `fixes-g7`, part 2 of 6 (at 1 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g7_part_2() {
        group_holds(M1_VECTORS, "fixes-g7", (1, 6), 6);
    }

    /// `m1/vectors.refcore`'s group `fixes-g7`, part 3 of 6 (at 2 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g7_part_3() {
        group_holds(M1_VECTORS, "fixes-g7", (2, 6), 6);
    }

    /// `m1/vectors.refcore`'s group `fixes-g7`, part 4 of 6 (at 3 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g7_part_4() {
        group_holds(M1_VECTORS, "fixes-g7", (3, 6), 5);
    }

    /// `m1/vectors.refcore`'s group `fixes-g7`, part 5 of 6 (at 4 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g7_part_5() {
        group_holds(M1_VECTORS, "fixes-g7", (4, 6), 5);
    }

    /// `m1/vectors.refcore`'s group `fixes-g7`, part 6 of 6 (at 5 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g7_part_6() {
        group_holds(M1_VECTORS, "fixes-g7", (5, 6), 5);
    }

    /// `m1/vectors.refcore`'s group `fixes-g8`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g8() {
        group_holds(M1_VECTORS, "fixes-g8", (0, 1), 95);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 1 of 7 (at 0 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_1() {
        group_holds(M1_VECTORS, "fixes-g9", (0, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 2 of 7 (at 1 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_2() {
        group_holds(M1_VECTORS, "fixes-g9", (1, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 3 of 7 (at 2 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_3() {
        group_holds(M1_VECTORS, "fixes-g9", (2, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 4 of 7 (at 3 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_4() {
        group_holds(M1_VECTORS, "fixes-g9", (3, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 5 of 7 (at 4 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_5() {
        group_holds(M1_VECTORS, "fixes-g9", (4, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 6 of 7 (at 5 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_6() {
        group_holds(M1_VECTORS, "fixes-g9", (5, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g9`, part 7 of 7 (at 6 modulo 7).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g9_part_7() {
        group_holds(M1_VECTORS, "fixes-g9", (6, 7), 30);
    }

    /// `m1/vectors.refcore`'s group `fixes-g10`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g10() {
        group_holds(M1_VECTORS, "fixes-g10", (0, 1), 101);
    }

    /// `m1/vectors.refcore`'s group `fixes-g11`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g11() {
        group_holds(M1_VECTORS, "fixes-g11", (0, 1), 91);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 1 of 9 (at 0 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_1() {
        group_holds(M1_VECTORS, "fixes-g12", (0, 9), 27);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 2 of 9 (at 1 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_2() {
        group_holds(M1_VECTORS, "fixes-g12", (1, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 3 of 9 (at 2 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_3() {
        group_holds(M1_VECTORS, "fixes-g12", (2, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 4 of 9 (at 3 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_4() {
        group_holds(M1_VECTORS, "fixes-g12", (3, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 5 of 9 (at 4 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_5() {
        group_holds(M1_VECTORS, "fixes-g12", (4, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 6 of 9 (at 5 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_6() {
        group_holds(M1_VECTORS, "fixes-g12", (5, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 7 of 9 (at 6 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_7() {
        group_holds(M1_VECTORS, "fixes-g12", (6, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 8 of 9 (at 7 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_8() {
        group_holds(M1_VECTORS, "fixes-g12", (7, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g12`, part 9 of 9 (at 8 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g12_part_9() {
        group_holds(M1_VECTORS, "fixes-g12", (8, 9), 26);
    }

    /// `m1/vectors.refcore`'s group `fixes-g13`, part 1 of 6 (at 0 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g13_part_1() {
        group_holds(M1_VECTORS, "fixes-g13", (0, 6), 58);
    }

    /// `m1/vectors.refcore`'s group `fixes-g13`, part 2 of 6 (at 1 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g13_part_2() {
        group_holds(M1_VECTORS, "fixes-g13", (1, 6), 58);
    }

    /// `m1/vectors.refcore`'s group `fixes-g13`, part 3 of 6 (at 2 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g13_part_3() {
        group_holds(M1_VECTORS, "fixes-g13", (2, 6), 58);
    }

    /// `m1/vectors.refcore`'s group `fixes-g13`, part 4 of 6 (at 3 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g13_part_4() {
        group_holds(M1_VECTORS, "fixes-g13", (3, 6), 58);
    }

    /// `m1/vectors.refcore`'s group `fixes-g13`, part 5 of 6 (at 4 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g13_part_5() {
        group_holds(M1_VECTORS, "fixes-g13", (4, 6), 57);
    }

    /// `m1/vectors.refcore`'s group `fixes-g13`, part 6 of 6 (at 5 modulo 6).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g13_part_6() {
        group_holds(M1_VECTORS, "fixes-g13", (5, 6), 57);
    }

    /// `m1/vectors.refcore`'s group `fixes-g14`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g14_part_1() {
        group_holds(M1_VECTORS, "fixes-g14", (0, 2), 96);
    }

    /// `m1/vectors.refcore`'s group `fixes-g14`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_g14_part_2() {
        group_holds(M1_VECTORS, "fixes-g14", (1, 2), 96);
    }

    /// `m1/vectors.refcore`'s group `fixes-open`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_open() {
        group_holds(M1_VECTORS, "fixes-open", (0, 1), 3);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 1 of 11 (at 0 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_1() {
        group_holds(M1_VECTORS, "fixes-gate8", (0, 11), 29);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 2 of 11 (at 1 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_2() {
        group_holds(M1_VECTORS, "fixes-gate8", (1, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 3 of 11 (at 2 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_3() {
        group_holds(M1_VECTORS, "fixes-gate8", (2, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 4 of 11 (at 3 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_4() {
        group_holds(M1_VECTORS, "fixes-gate8", (3, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 5 of 11 (at 4 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_5() {
        group_holds(M1_VECTORS, "fixes-gate8", (4, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 6 of 11 (at 5 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_6() {
        group_holds(M1_VECTORS, "fixes-gate8", (5, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 7 of 11 (at 6 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_7() {
        group_holds(M1_VECTORS, "fixes-gate8", (6, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 8 of 11 (at 7 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_8() {
        group_holds(M1_VECTORS, "fixes-gate8", (7, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 9 of 11 (at 8 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_9() {
        group_holds(M1_VECTORS, "fixes-gate8", (8, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 10 of 11 (at 9 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_10() {
        group_holds(M1_VECTORS, "fixes-gate8", (9, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-gate8`, part 11 of 11 (at 10 modulo 11).
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_gate8_part_11() {
        group_holds(M1_VECTORS, "fixes-gate8", (10, 11), 28);
    }

    /// `m1/vectors.refcore`'s group `fixes-r1`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r1() {
        group_holds(M1_VECTORS, "fixes-r1", (0, 1), 10);
    }

    /// `m1/vectors.refcore`'s group `fixes-r2`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r2() {
        group_holds(M1_VECTORS, "fixes-r2", (0, 1), 7);
    }

    /// `m1/vectors.refcore`'s group `fixes-r3`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r3() {
        group_holds(M1_VECTORS, "fixes-r3", (0, 1), 8);
    }

    /// `m1/vectors.refcore`'s group `fixes-r4`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r4() {
        group_holds(M1_VECTORS, "fixes-r4", (0, 1), 10);
    }

    /// `m1/vectors.refcore`'s group `fixes-r5`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r5() {
        group_holds(M1_VECTORS, "fixes-r5", (0, 1), 19);
    }

    /// `m1/vectors.refcore`'s group `fixes-r6`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r6() {
        group_holds(M1_VECTORS, "fixes-r6", (0, 1), 11);
    }

    /// `m1/vectors.refcore`'s group `fixes-r7`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r7() {
        group_holds(M1_VECTORS, "fixes-r7", (0, 1), 14);
    }

    /// `m1/vectors.refcore`'s group `fixes-r8`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r8() {
        group_holds(M1_VECTORS, "fixes-r8", (0, 1), 6);
    }

    /// `m1/vectors.refcore`'s group `fixes-r9`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r9() {
        group_holds(M1_VECTORS, "fixes-r9", (0, 1), 18);
    }

    /// `m1/vectors.refcore`'s group `fixes-r10`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r10() {
        group_holds(M1_VECTORS, "fixes-r10", (0, 1), 5);
    }

    /// `m1/vectors.refcore`'s group `fixes-r11`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_r11() {
        group_holds(M1_VECTORS, "fixes-r11", (0, 1), 2);
    }

    /// `m1/vectors.refcore`'s group `fixes-b`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_b() {
        group_holds(M1_VECTORS, "fixes-b", (0, 1), 18);
    }

    /// `m1/vectors.refcore`'s group `fixes-s`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m1_fixes_s() {
        group_holds(M1_VECTORS, "fixes-s", (0, 1), 23);
    }

    /// `m2/vectors.refcore`'s group `checks-green`, part 1 of 5 (at 0 modulo 5).
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_green_part_1() {
        group_holds(M2_VECTORS, "checks-green", (0, 5), 4);
    }

    /// `m2/vectors.refcore`'s group `checks-green`, part 2 of 5 (at 1 modulo 5).
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_green_part_2() {
        group_holds(M2_VECTORS, "checks-green", (1, 5), 4);
    }

    /// `m2/vectors.refcore`'s group `checks-green`, part 3 of 5 (at 2 modulo 5).
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_green_part_3() {
        group_holds(M2_VECTORS, "checks-green", (2, 5), 4);
    }

    /// `m2/vectors.refcore`'s group `checks-green`, part 4 of 5 (at 3 modulo 5).
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_green_part_4() {
        group_holds(M2_VECTORS, "checks-green", (3, 5), 4);
    }

    /// `m2/vectors.refcore`'s group `checks-green`, part 5 of 5 (at 4 modulo 5).
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_green_part_5() {
        group_holds(M2_VECTORS, "checks-green", (4, 5), 4);
    }

    /// `m2/vectors.refcore`'s group `checks-records`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_records() {
        group_holds(M2_VECTORS, "checks-records", (0, 1), 1);
    }

    /// `m2/vectors.refcore`'s group `checks-forms`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_checks_forms() {
        group_holds(M2_VECTORS, "checks-forms", (0, 1), 2);
    }

    /// `m2/vectors.refcore`'s group `items-green`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_items_green() {
        group_holds(M2_VECTORS, "items-green", (0, 1), 3);
    }

    /// `m2/vectors.refcore`'s group `items-forms`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_items_forms() {
        group_holds(M2_VECTORS, "items-forms", (0, 1), 1);
    }

    /// `m2/vectors.refcore`'s group `items-explain`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_items_explain() {
        group_holds(M2_VECTORS, "items-explain", (0, 1), 4);
    }

    /// `m2/vectors.refcore`'s group `items-windows`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_items_windows() {
        group_holds(M2_VECTORS, "items-windows", (0, 1), 2);
    }

    /// `m2/vectors.refcore`'s group `fixes-forms`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_forms() {
        group_holds(M2_VECTORS, "fixes-forms", (0, 1), 3);
    }

    /// `m2/vectors.refcore`'s group `fixes-sweep`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_sweep() {
        group_holds(M2_VECTORS, "fixes-sweep", (0, 1), 45);
    }

    /// `m2/vectors.refcore`'s group `fixes-explain`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_explain() {
        group_holds(M2_VECTORS, "fixes-explain", (0, 1), 9);
    }

    /// `m2/vectors.refcore`'s group `fixes-windows`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_windows() {
        group_holds(M2_VECTORS, "fixes-windows", (0, 1), 14);
    }

    /// `m2/vectors.refcore`'s group `items-sweep`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_items_sweep() {
        group_holds(M2_VECTORS, "items-sweep", (0, 1), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 1 of 9 (at 0 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_1() {
        group_holds(M2_VECTORS, "goldens-records", (0, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 2 of 9 (at 1 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_2() {
        group_holds(M2_VECTORS, "goldens-records", (1, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 3 of 9 (at 2 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_3() {
        group_holds(M2_VECTORS, "goldens-records", (2, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 4 of 9 (at 3 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_4() {
        group_holds(M2_VECTORS, "goldens-records", (3, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 5 of 9 (at 4 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_5() {
        group_holds(M2_VECTORS, "goldens-records", (4, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 6 of 9 (at 5 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_6() {
        group_holds(M2_VECTORS, "goldens-records", (5, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 7 of 9 (at 6 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_7() {
        group_holds(M2_VECTORS, "goldens-records", (6, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 8 of 9 (at 7 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_8() {
        group_holds(M2_VECTORS, "goldens-records", (7, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-records`, part 9 of 9 (at 8 modulo 9).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_records_part_9() {
        group_holds(M2_VECTORS, "goldens-records", (8, 9), 1);
    }

    /// `m2/vectors.refcore`'s group `goldens-ledger`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_ledger_part_1() {
        group_holds(M2_VECTORS, "goldens-ledger", (0, 2), 2);
    }

    /// `m2/vectors.refcore`'s group `goldens-ledger`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_goldens_ledger_part_2() {
        group_holds(M2_VECTORS, "goldens-ledger", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `fixes-records`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_records() {
        group_holds(M2_VECTORS, "fixes-records", (0, 1), 14);
    }

    /// `m2/vectors.refcore`'s group `fixes-ledger`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_ledger() {
        group_holds(M2_VECTORS, "fixes-ledger", (0, 1), 37);
    }

    /// `m2/vectors.refcore`'s group `fixes-r11`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_r11() {
        group_holds(M2_VECTORS, "fixes-r11", (0, 1), 14);
    }

    /// `m2/vectors.refcore`'s group `fixes-r12`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_r12() {
        group_holds(M2_VECTORS, "fixes-r12", (0, 1), 12);
    }

    /// `m2/vectors.refcore`'s group `fixes-r13`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_r13() {
        group_holds(M2_VECTORS, "fixes-r13", (0, 1), 17);
    }

    /// `m2/vectors.refcore`'s group `fixes-r14`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_r14() {
        group_holds(M2_VECTORS, "fixes-r14", (0, 1), 16);
    }

    /// `m2/vectors.refcore`'s group `fixes-r15`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_r15() {
        group_holds(M2_VECTORS, "fixes-r15", (0, 1), 9);
    }

    /// `m2/vectors.refcore`'s group `fixes-r16`, every vector (18 since tally 2.2 dropped
    /// `r16/projection_leg_day` and `r16/low_from_leg_day`: a leg has no day of its own, S-6).
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_r16() {
        group_holds(M2_VECTORS, "fixes-r16", (0, 1), 18);
    }

    /// `m2/vectors.refcore`'s group `fixes-s`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_s() {
        group_holds(M2_VECTORS, "fixes-s", (0, 1), 8);
    }

    /// `m2/vectors.refcore`'s group `fixes-d`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_d() {
        group_holds(M2_VECTORS, "fixes-d", (0, 1), 16);
    }

    /// `m2/vectors.refcore`'s group `fixes-cut`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_cut() {
        group_holds(M2_VECTORS, "fixes-cut", (0, 1), 5);
    }

    /// `m2/vectors.refcore`'s group `fixes-rv`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_m2_fixes_rv() {
        group_holds(M2_VECTORS, "fixes-rv", (0, 1), 25);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h0`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h0_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h0", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h0`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h0_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h0", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h1`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h1_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h1", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h1`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h1_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h1", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h2`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h2_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h2", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h2`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h2_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h2", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h3`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h3_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h3", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h3`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h3_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h3", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h4`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h4_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h4", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h4`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h4_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h4", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h5`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h5_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h5", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h5`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h5_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h5", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h8`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h8_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h8", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h8`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h8_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h8", (1, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h9`, part 1 of 2 (at 0 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h9_part_1() {
        group_holds(M2_VECTORS, "hindcasts-h9", (0, 2), 1);
    }

    /// `m2/vectors.refcore`'s group `hindcasts-h9`, part 2 of 2 (at 1 modulo 2).
    #[test]
    fn m2_forks_on_prints_off_on_m2_hindcasts_h9_part_2() {
        group_holds(M2_VECTORS, "hindcasts-h9", (1, 2), 1);
    }

    /// `m2/spec.vectors`'s group `spec-forms`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_spec_spec_forms() {
        group_holds(M2_SPEC, "spec-forms", (0, 1), 2);
    }

    /// `m2/spec.vectors`'s group `spec-records`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_spec_spec_records() {
        group_holds(M2_SPEC, "spec-records", (0, 1), 2);
    }

    /// `m2/spec.vectors`'s group `spec-sweep`, every vector.
    #[test]
    fn m2_forks_on_prints_off_on_spec_spec_sweep() {
        group_holds(M2_SPEC, "spec-sweep", (0, 1), 8);
    }
}
