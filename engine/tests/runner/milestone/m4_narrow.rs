//! Milestone M4's fence F7, the narrow reads, against its plain twin (chunk C6; picks P17, U1; R-130,
//! R-84; `MILESTONE-M4.md` row 6; issue #252). Every `run` vector of the runner's stores
//! (`kernel/vectors.refcore`, `m1/`, `m2/` and `m4/vectors.refcore` with their `spec.vectors`,
//! M3's `runs.vectors` and `spec.vectors`) and every record of the four corpora
//! (`properties/kernel.corpus`, `m0.corpus`, `m1.corpus`, `m2.corpus`) runs through
//! `tally_cli::tally::run_with_how` twice, with `Plans::Narrow` (fence F7, `walk/narrow.rs`: at a
//! past booking cut, `o.fulfils` reads the one instance it names and `flows(mv, span)` the instances
//! its span holds) and with `Plans::Whole` (the move's whole list at that cut, its plain twin), every
//! other fence in its form (`How::FENCES`), and the two print the same lines. A difference is the
//! fence's fault. Then F7's own realities (`m4/narrow/`, each with every ask and with its asks in
//! reverse order, so another instance reads first), P8-L10's `c1_r2` (its six asks) and P8-14's
//! check (`cut11/b_r1`), the CLI's `narrow_oracle` realities, which T's role can't write under
//! `crates/`. A store's groups, M3's stores and a corpus run in parts by place modulo the parts (as
//! `m2_forks`), each part about 0.4 s of build/m4's quick engine's time (00143f5a, Narrow, one
//! CLI run per item, 2026-11-23); each part asks at least as many runs answered by both as the engine
//! answered then (no vacuous pass). Written by T's `gen_narrow.py` (`tmp/m4/t/`); M4's groups
//! brought up to tally 2.3.1's stores at its second pin (#259: the `fixes-t231*` groups, and the
//! spec groups its released holds shrank, one part each, every run answered by both, at 3e724a82).
//! A fence's equivalence test (R-130): it runs by default, outside `check quick`
//! (`quick_leaves_out`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use tally_cli::tally::walk::Plans;
    use tally_cli::tally::{Asks, How, NotImplemented, Pin, Request, run_with_how};
    use tally_runner::corpus::{KERNEL_CORPUS, netstrings, records};
    use tally_runner::kernel::vectors;
    use tally_runner::laws_m0::M0_CORPUS;
    use tally_runner::laws_m1::M1_CORPUS;
    use tally_runner::laws_m2::M2_CORPUS;
    use tally_runner::m3::{check_vectors, library_files, request_words, texts};

    /// The six-netstring stores, as the runner embeds them.
    const KERNEL: &[u8] = tally_runner::kernel::VECTORS;
    const M1: &[u8] = tally_runner::m1::VECTORS;
    const M1_SPEC: &[u8] = tally_runner::m1::SPEC;
    const M2: &[u8] = tally_runner::m2::VECTORS;
    const M2_SPEC: &[u8] = tally_runner::m2::SPEC;
    const M4: &[u8] = tally_runner::m4::VECTORS;
    const M4_SPEC: &[u8] = tally_runner::m4::SPEC;

    /// M3's eight-netstring stores of `run` vectors.
    const M3_RUNS: &[u8] = tally_runner::m3::RUNS;
    const M3_SPEC: &[u8] = tally_runner::m3::SPEC;

    /// F7's own reality `m4/narrow/n1_fulfils.tally`.
    const N1_FULFILS: &str = include_str!("../../m4/narrow/n1_fulfils.tally");

    /// F7's own reality `m4/narrow/n2_flows.tally`.
    const N2_FLOWS: &str = include_str!("../../m4/narrow/n2_flows.tally");

    /// F7's own reality `m4/narrow/n3_window.tally`.
    const N3_WINDOW: &str = include_str!("../../m4/narrow/n3_window.tally");

    /// F7's own reality `m4/narrow/n4_own.tally`.
    const N4_OWN: &str = include_str!("../../m4/narrow/n4_own.tally");

    /// P8-L10's `c1_r2` (`m4/load/runs.tsv`): change 1's walk run at cut 11, its files as the run names them.
    const C1_R2: [(&str, &str); 14] = [
        (
            "cut11/c1_r2/asks/cut.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/asks/cut.tally"),
        ),
        (
            "cut11/c1_r2/facts/2026.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/facts/2026.tally"),
        ),
        (
            "cut11/c1_r2/facts/2027.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/facts/2027.tally"),
        ),
        (
            "cut11/c1_r2/law/pl/benefits.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/law/pl/benefits.tally"),
        ),
        (
            "cut11/c1_r2/law/pl/business.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/law/pl/business.tally"),
        ),
        (
            "cut11/c1_r2/law/pl/common.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/law/pl/common.tally"),
        ),
        (
            "cut11/c1_r2/law/pl/employment.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/law/pl/employment.tally"),
        ),
        (
            "cut11/c1_r2/law/pl/wrappers.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/law/pl/wrappers.tally"),
        ),
        (
            "cut11/c1_r2/loans.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/loans.tally"),
        ),
        (
            "cut11/c1_r2/pays.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/pays.tally"),
        ),
        (
            "cut11/c1_r2/people.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/people.tally"),
        ),
        (
            "cut11/c1_r2/places.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/places.tally"),
        ),
        (
            "cut11/c1_r2/reality.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/reality.tally"),
        ),
        (
            "cut11/c1_r2/records/mbank/2026-09.tally",
            include_str!("../../integration/amendments/cut11/c1_r2/records/mbank/2026-09.tally"),
        ),
    ];

    /// P8-14's check (`I:X:replay_september@b_r1`): cut 11, change b, read 1, whose projections are read day by day.
    const B_R1: [(&str, &str); 13] = [
        (
            "cut11/b_r1/asks/cut.tally",
            include_str!("../../integration/amendments/cut11/b_r1/asks/cut.tally"),
        ),
        (
            "cut11/b_r1/facts/2026.tally",
            include_str!("../../integration/amendments/cut11/b_r1/facts/2026.tally"),
        ),
        (
            "cut11/b_r1/law/pl/benefits.tally",
            include_str!("../../integration/amendments/cut11/b_r1/law/pl/benefits.tally"),
        ),
        (
            "cut11/b_r1/law/pl/business.tally",
            include_str!("../../integration/amendments/cut11/b_r1/law/pl/business.tally"),
        ),
        (
            "cut11/b_r1/law/pl/common.tally",
            include_str!("../../integration/amendments/cut11/b_r1/law/pl/common.tally"),
        ),
        (
            "cut11/b_r1/law/pl/employment.tally",
            include_str!("../../integration/amendments/cut11/b_r1/law/pl/employment.tally"),
        ),
        (
            "cut11/b_r1/law/pl/wrappers.tally",
            include_str!("../../integration/amendments/cut11/b_r1/law/pl/wrappers.tally"),
        ),
        (
            "cut11/b_r1/loans.tally",
            include_str!("../../integration/amendments/cut11/b_r1/loans.tally"),
        ),
        (
            "cut11/b_r1/pays.tally",
            include_str!("../../integration/amendments/cut11/b_r1/pays.tally"),
        ),
        (
            "cut11/b_r1/people.tally",
            include_str!("../../integration/amendments/cut11/b_r1/people.tally"),
        ),
        (
            "cut11/b_r1/places.tally",
            include_str!("../../integration/amendments/cut11/b_r1/places.tally"),
        ),
        (
            "cut11/b_r1/reality.tally",
            include_str!("../../integration/amendments/cut11/b_r1/reality.tally"),
        ),
        (
            "cut11/b_r1/records/mbank/2026-09.tally",
            include_str!("../../integration/amendments/cut11/b_r1/records/mbank/2026-09.tally"),
        ),
    ];

    /// A file table (netstring pairs) as a map from path to bytes.
    fn table_of(table: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
        let mut out = BTreeMap::new();
        let mut name: Option<&[u8]> = None;
        for part in netstrings(table)? {
            match name.take() {
                None => name = Some(part),
                Some(n) => {
                    let key = String::from_utf8(n.to_vec())
                        .map_err(|_| "a path that isn't UTF-8".to_owned())?;
                    out.insert(key, part.to_vec());
                }
            }
        }
        Ok(out)
    }

    /// Embedded texts as a file table.
    fn table_of_texts(files: &[(&str, &str)]) -> BTreeMap<String, Vec<u8>> {
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec()))
            .collect()
    }

    /// The words after `run` for `files` and `asks`.
    fn words(files: &[(&str, &str)], asks: &[&str]) -> Vec<String> {
        let mut rest: Vec<String> = files.iter().map(|(path, _)| (*path).to_owned()).collect();
        if !asks.is_empty() {
            rest.push("--ask".to_owned());
            rest.extend(asks.iter().map(|a| (*a).to_owned()));
        }
        rest
    }

    /// The lines of one run under `plans`, or `None` where it stops not implemented.
    fn lines(
        paths: &[String],
        table: &BTreeMap<String, Vec<u8>>,
        (asks, pin): (&Asks, Pin),
        plans: Plans,
    ) -> Option<Vec<String>> {
        let files = library_files(paths, table.clone(), pin);
        let how = How {
            plans,
            ..How::FENCES
        };
        match run_with_how(files, &Request::new(asks.clone()), how) {
            Ok(answer) => Some(answer.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    /// Runs the words after `run` under both forms; `Err` names the first line they differ at.
    /// `Ok(true)` where both answered, `Ok(false)` where both stopped.
    fn same(rest: &[String], table: &BTreeMap<String, Vec<u8>>) -> Result<bool, String> {
        let (paths, asks, pin) = request_words(rest);
        let narrow = lines(&paths, table, (&asks, pin), Plans::Narrow);
        let whole = lines(&paths, table, (&asks, pin), Plans::Whole);
        match (narrow, whole) {
            (None, None) => Ok(false),
            (Some(a), Some(b)) if a == b => Ok(true),
            (Some(a), Some(b)) => {
                let line = match a.iter().zip(&b).position(|(x, y)| x != y) {
                    Some(i) => i,
                    None => a.len().min(b.len()),
                };
                Err(format!(
                    "line {}: narrow {:?}, whole {:?}",
                    line.checked_add(1).expect("a line number"),
                    a.get(line),
                    b.get(line)
                ))
            }
            (a, b) => Err(format!(
                "one stops: narrow {}, whole {}",
                a.is_some(),
                b.is_some()
            )),
        }
    }

    /// One item's command line (`run` first) and file table under both forms.
    fn item_same(args: &[String], table: &BTreeMap<String, Vec<u8>>) -> Result<bool, String> {
        match args.split_first() {
            Some((first, rest)) if first == "run" => same(rest, table),
            _ => Err(format!("not a run: {args:?}")),
        }
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
            "{} of {n} runs of {what} differ under Plans::Narrow and Plans::Whole:\n{}",
            failed.len(),
            failed.join("\n")
        );
        assert!(
            both >= answered,
            "{both} of {n} runs of {what} answered by both, {answered} expected"
        );
    }

    /// Every vector of `group` in `store` (six netstrings) whose place in the group is `part`
    /// modulo `parts` alike under both forms; at least `answered` answered by both.
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
            .map(|v| {
                let r = table_of(&v.files).and_then(|t| item_same(&v.args, &t));
                (v.id.clone(), r)
            })
            .collect();
        judge(
            &format!("{group}, part {part} of {parts}"),
            results,
            answered,
        );
    }

    /// M3's `run` vectors of `store` (eight netstrings; `items_in` of them, a usage error's aside)
    /// whose place is `part` modulo `parts` alike under both forms; at least `answered` answered.
    fn m3_holds(store: &[u8], (part, parts): (usize, usize), items_in: usize, answered: usize) {
        let all = check_vectors(store).expect("the store reads");
        let texts = texts();
        let runs: Vec<_> = all
            .iter()
            .filter(|v| v.source != "rule" && v.args.first().is_some_and(|w| w == "run"))
            .collect();
        assert_eq!(runs.len(), items_in, "the store's run vectors");
        let results = runs
            .iter()
            .enumerate()
            .filter(|(i, _)| i.checked_rem(parts) == Some(part))
            .map(|(_, v)| {
                let table: Result<BTreeMap<String, Vec<u8>>, String> = v
                    .table
                    .iter()
                    .map(|f| match texts.get(f.path.as_str()) {
                        Some(text) => Ok((f.key.clone(), text.as_bytes().to_vec())),
                        None => Err(format!("{} has no embedded text", f.path)),
                    })
                    .collect();
                (v.id.clone(), table.and_then(|t| item_same(&v.args, &t)))
            })
            .collect();
        judge(&format!("part {part} of {parts}"), results, answered);
    }

    /// Every record of `corpus` whose place is `part` modulo `parts` alike under both forms; the
    /// corpus has `records_in` records; at least `answered` answered by both.
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
                (
                    name,
                    table_of(&r.files).and_then(|t| item_same(&r.args, &t)),
                )
            })
            .collect();
        judge(&format!("part {part} of {parts}"), results, answered);
    }

    /// A reality of its own alike under both forms: every ask (answered or both stopped), then
    /// `reversed`, its asks in reverse order, which both must answer.
    fn own_holds(name: &str, files: &[(&str, &str)], reversed: &[&str]) {
        let table = table_of_texts(files);
        let every = same(&words(files, &[]), &table);
        assert!(every.is_ok(), "{name}, every ask: {every:?}");
        let r = same(&words(files, reversed), &table);
        judge(
            &format!("{name}, its asks reversed"),
            vec![(name.to_owned(), r)],
            1,
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

    /// Each group of the six-netstring stores has its test here or waits on a named fix, so a new
    /// group is never left out.
    #[test]
    fn m4_narrow_every_group_is_tested() {
        assert_eq!(
            groups_of(KERNEL),
            [
                "checks", "g1", "g10", "g11", "g12", "g13", "g14", "g2", "g3", "g4", "g5", "g6",
                "g7", "g8", "g9"
            ]
            .iter()
            .map(|g| (*g).to_owned())
            .collect::<BTreeSet<String>>(),
            "each group of kernel has its test here"
        );
        assert_eq!(
            groups_of(M1),
            [
                "carried-1",
                "carried-2",
                "carried-3",
                "carried-4",
                "carried-5",
                "carried-6",
                "carried-7",
                "checks-g1",
                "checks-g10",
                "checks-g11",
                "checks-g2",
                "checks-g3",
                "checks-g4",
                "checks-g5",
                "checks-g6",
                "checks-g7",
                "checks-g8",
                "checks-green",
                "fixes-b",
                "fixes-g1",
                "fixes-g10",
                "fixes-g11",
                "fixes-g12",
                "fixes-g13",
                "fixes-g14",
                "fixes-g2",
                "fixes-g3",
                "fixes-g4",
                "fixes-g5",
                "fixes-g6",
                "fixes-g7",
                "fixes-g8",
                "fixes-g9",
                "fixes-gate8",
                "fixes-open",
                "fixes-r1",
                "fixes-r10",
                "fixes-r11",
                "fixes-r2",
                "fixes-r3",
                "fixes-r4",
                "fixes-r5",
                "fixes-r6",
                "fixes-r7",
                "fixes-r8",
                "fixes-r9",
                "fixes-s",
                "goldens-business",
                "goldens-g10",
                "goldens-g6",
                "goldens-g7",
                "goldens-g8",
                "goldens-g9",
                "goldens-pins",
                "goldens-y22"
            ]
            .iter()
            .map(|g| (*g).to_owned())
            .collect::<BTreeSet<String>>(),
            "each group of m1 has its test here"
        );
        assert_eq!(
            groups_of(M1_SPEC),
            ["spec"]
                .iter()
                .map(|g| (*g).to_owned())
                .collect::<BTreeSet<String>>(),
            "each group of m1_spec has its test here"
        );
        assert_eq!(
            groups_of(M2),
            [
                "checks-forms",
                "checks-green",
                "checks-records",
                "fixes-22",
                "fixes-cut",
                "fixes-d",
                "fixes-explain",
                "fixes-forms",
                "fixes-ledger",
                "fixes-r11",
                "fixes-r12",
                "fixes-r13",
                "fixes-r15",
                "fixes-r16",
                "fixes-records",
                "fixes-rv",
                "fixes-s",
                "fixes-sweep",
                "fixes-windows",
                "goldens-ledger",
                "goldens-records",
                "hindcasts-h0",
                "hindcasts-h1",
                "hindcasts-h2",
                "hindcasts-h3",
                "hindcasts-h4",
                "hindcasts-h5",
                "hindcasts-h8",
                "hindcasts-h9",
                "items-explain",
                "items-forms",
                "items-green",
                "items-sweep",
                "items-windows"
            ]
            .iter()
            .map(|g| (*g).to_owned())
            .collect::<BTreeSet<String>>(),
            "each group of m2 has its test here"
        );
        assert_eq!(
            groups_of(M2_SPEC),
            ["spec-forms", "spec-sweep"]
                .iter()
                .map(|g| (*g).to_owned())
                .collect::<BTreeSet<String>>(),
            "each group of m2_spec has its test here"
        );
        assert_eq!(
            groups_of(M4),
            [
                "checks-accrue",
                "checks-green",
                "checks-pins",
                "constructs-accrue",
                "constructs-aggregates",
                "constructs-argm",
                "constructs-beats",
                "constructs-cal",
                "constructs-fx",
                "constructs-keydiff",
                "constructs-logic",
                "constructs-paper",
                "constructs-pins",
                "constructs-ratepin",
                "constructs-round",
                "constructs-tuple",
                "fixes-g1",
                "fixes-g2",
                "fixes-g2fix",
                "fixes-g3",
                "fixes-g3fix",
                "fixes-g4",
                "fixes-g6",
                "fixes-g6c",
                "fixes-g6cfix",
                "fixes-gate",
                "fixes-gate8",
                "fixes-gatefix",
                "fixes-gatelead",
                "fixes-narrow",
                "fixes-p12",
                "fixes-p21",
                "fixes-p22",
                "fixes-p23",
                "fixes-p30",
                "fixes-p32",
                "fixes-p40",
                "fixes-p40fix",
                "fixes-p43",
                "fixes-t231g1",
                "fixes-t231g2",
                "fixes-t231rv1",
                "fixes-t231rv2",
                "fixes-traced",
                "items-backlog",
                "items-paper",
                "items-walked"
            ]
            .iter()
            .map(|g| (*g).to_owned())
            .collect::<BTreeSet<String>>(),
            "each group of m4 has its test here"
        );
        assert_eq!(
            groups_of(M4_SPEC),
            [
                "spec-g1",
                "spec-g2",
                "spec-g2fix",
                "spec-g3",
                "spec-g3fix",
                "spec-gate",
                "spec-p12",
                "spec-p21",
                "spec-p22",
                "spec-p26",
                "spec-p31"
            ]
            .iter()
            .map(|g| (*g).to_owned())
            .collect::<BTreeSet<String>>(),
            "each group of m4_spec has its test here"
        );
    }

    /// `kernel`'s group `checks`, part 1 of 7 (its vectors at 0 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_1() {
        group_holds(KERNEL, "checks", (0, 7), 7);
    }

    /// `kernel`'s group `checks`, part 2 of 7 (its vectors at 1 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_2() {
        group_holds(KERNEL, "checks", (1, 7), 6);
    }

    /// `kernel`'s group `checks`, part 3 of 7 (its vectors at 2 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_3() {
        group_holds(KERNEL, "checks", (2, 7), 6);
    }

    /// `kernel`'s group `checks`, part 4 of 7 (its vectors at 3 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_4() {
        group_holds(KERNEL, "checks", (3, 7), 6);
    }

    /// `kernel`'s group `checks`, part 5 of 7 (its vectors at 4 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_5() {
        group_holds(KERNEL, "checks", (4, 7), 6);
    }

    /// `kernel`'s group `checks`, part 6 of 7 (its vectors at 5 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_6() {
        group_holds(KERNEL, "checks", (5, 7), 6);
    }

    /// `kernel`'s group `checks`, part 7 of 7 (its vectors at 6 modulo 7).
    #[test]
    fn m4_narrow_kernel_checks_part_7() {
        group_holds(KERNEL, "checks", (6, 7), 6);
    }

    /// `kernel`'s group `g1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g1_part_1() {
        group_holds(KERNEL, "g1", (0, 1), 15);
    }

    /// `kernel`'s group `g2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g2_part_1() {
        group_holds(KERNEL, "g2", (0, 1), 3);
    }

    /// `kernel`'s group `g3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g3_part_1() {
        group_holds(KERNEL, "g3", (0, 1), 2);
    }

    /// `kernel`'s group `g4`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g4_part_1() {
        group_holds(KERNEL, "g4", (0, 1), 1);
    }

    /// `kernel`'s group `g5`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g5_part_1() {
        group_holds(KERNEL, "g5", (0, 1), 13);
    }

    /// `kernel`'s group `g9`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g9_part_1() {
        group_holds(KERNEL, "g9", (0, 1), 5);
    }

    /// `kernel`'s group `g6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g6_part_1() {
        group_holds(KERNEL, "g6", (0, 1), 17);
    }

    /// `kernel`'s group `g7`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g7_part_1() {
        group_holds(KERNEL, "g7", (0, 1), 10);
    }

    /// `kernel`'s group `g8`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g8_part_1() {
        group_holds(KERNEL, "g8", (0, 1), 5);
    }

    /// `kernel`'s group `g10`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g10_part_1() {
        group_holds(KERNEL, "g10", (0, 1), 6);
    }

    /// `kernel`'s group `g11`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g11_part_1() {
        group_holds(KERNEL, "g11", (0, 1), 11);
    }

    /// `kernel`'s group `g12`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g12_part_1() {
        group_holds(KERNEL, "g12", (0, 1), 2);
    }

    /// `kernel`'s group `g13`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_kernel_g13_part_1() {
        group_holds(KERNEL, "g13", (0, 1), 11);
    }

    /// `kernel`'s group `g14`, part 1 of 13 (its vectors at 0 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_1() {
        group_holds(KERNEL, "g14", (0, 13), 2);
    }

    /// `kernel`'s group `g14`, part 2 of 13 (its vectors at 1 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_2() {
        group_holds(KERNEL, "g14", (1, 13), 2);
    }

    /// `kernel`'s group `g14`, part 3 of 13 (its vectors at 2 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_3() {
        group_holds(KERNEL, "g14", (2, 13), 2);
    }

    /// `kernel`'s group `g14`, part 4 of 13 (its vectors at 3 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_4() {
        group_holds(KERNEL, "g14", (3, 13), 2);
    }

    /// `kernel`'s group `g14`, part 5 of 13 (its vectors at 4 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_5() {
        group_holds(KERNEL, "g14", (4, 13), 2);
    }

    /// `kernel`'s group `g14`, part 6 of 13 (its vectors at 5 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_6() {
        group_holds(KERNEL, "g14", (5, 13), 2);
    }

    /// `kernel`'s group `g14`, part 7 of 13 (its vectors at 6 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_7() {
        group_holds(KERNEL, "g14", (6, 13), 2);
    }

    /// `kernel`'s group `g14`, part 8 of 13 (its vectors at 7 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_8() {
        group_holds(KERNEL, "g14", (7, 13), 2);
    }

    /// `kernel`'s group `g14`, part 9 of 13 (its vectors at 8 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_9() {
        group_holds(KERNEL, "g14", (8, 13), 2);
    }

    /// `kernel`'s group `g14`, part 10 of 13 (its vectors at 9 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_10() {
        group_holds(KERNEL, "g14", (9, 13), 1);
    }

    /// `kernel`'s group `g14`, part 11 of 13 (its vectors at 10 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_11() {
        group_holds(KERNEL, "g14", (10, 13), 1);
    }

    /// `kernel`'s group `g14`, part 12 of 13 (its vectors at 11 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_12() {
        group_holds(KERNEL, "g14", (11, 13), 1);
    }

    /// `kernel`'s group `g14`, part 13 of 13 (its vectors at 12 modulo 13).
    #[test]
    fn m4_narrow_kernel_g14_part_13() {
        group_holds(KERNEL, "g14", (12, 13), 1);
    }

    /// `m1`'s group `checks-g1`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m1_checks_g1_part_1() {
        group_holds(M1, "checks-g1", (0, 2), 5);
    }

    /// `m1`'s group `checks-g1`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m1_checks_g1_part_2() {
        group_holds(M1, "checks-g1", (1, 2), 4);
    }

    /// `m1`'s group `checks-g2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g2_part_1() {
        group_holds(M1, "checks-g2", (0, 1), 7);
    }

    /// `m1`'s group `checks-g3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g3_part_1() {
        group_holds(M1, "checks-g3", (0, 1), 6);
    }

    /// `m1`'s group `checks-g4`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g4_part_1() {
        group_holds(M1, "checks-g4", (0, 1), 1);
    }

    /// `m1`'s group `checks-g5`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g5_part_1() {
        group_holds(M1, "checks-g5", (0, 1), 4);
    }

    /// `m1`'s group `checks-g6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g6_part_1() {
        group_holds(M1, "checks-g6", (0, 1), 4);
    }

    /// `m1`'s group `checks-g7`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g7_part_1() {
        group_holds(M1, "checks-g7", (0, 1), 4);
    }

    /// `m1`'s group `checks-g8`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g8_part_1() {
        group_holds(M1, "checks-g8", (0, 1), 4);
    }

    /// `m1`'s group `checks-g10`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g10_part_1() {
        group_holds(M1, "checks-g10", (0, 1), 4);
    }

    /// `m1`'s group `checks-g11`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_g11_part_1() {
        group_holds(M1, "checks-g11", (0, 1), 5);
    }

    /// `m1`'s group `checks-green`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_checks_green_part_1() {
        group_holds(M1, "checks-green", (0, 1), 17);
    }

    /// `m1`'s group `goldens-pins`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_pins_part_1() {
        group_holds(M1, "goldens-pins", (0, 1), 1);
    }

    /// `m1`'s group `goldens-g6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_g6_part_1() {
        group_holds(M1, "goldens-g6", (0, 1), 1);
    }

    /// `m1`'s group `goldens-g7`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_g7_part_1() {
        group_holds(M1, "goldens-g7", (0, 1), 1);
    }

    /// `m1`'s group `goldens-g10`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_g10_part_1() {
        group_holds(M1, "goldens-g10", (0, 1), 1);
    }

    /// `m1`'s group `goldens-y22`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_y22_part_1() {
        group_holds(M1, "goldens-y22", (0, 1), 1);
    }

    /// `m1`'s group `goldens-business`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_business_part_1() {
        group_holds(M1, "goldens-business", (0, 1), 1);
    }

    /// `m1`'s group `goldens-g8`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_g8_part_1() {
        group_holds(M1, "goldens-g8", (0, 1), 1);
    }

    /// `m1`'s group `goldens-g9`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_goldens_g9_part_1() {
        group_holds(M1, "goldens-g9", (0, 1), 2);
    }

    /// `m1`'s group `carried-5`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_5_part_1() {
        group_holds(M1, "carried-5", (0, 1), 11);
    }

    /// `m1`'s group `carried-6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_6_part_1() {
        group_holds(M1, "carried-6", (0, 1), 12);
    }

    /// `m1`'s group `carried-1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_1_part_1() {
        group_holds(M1, "carried-1", (0, 1), 6);
    }

    /// `m1`'s group `carried-3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_3_part_1() {
        group_holds(M1, "carried-3", (0, 1), 4);
    }

    /// `m1`'s group `carried-7`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_7_part_1() {
        group_holds(M1, "carried-7", (0, 1), 1);
    }

    /// `m1`'s group `carried-2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_2_part_1() {
        group_holds(M1, "carried-2", (0, 1), 6);
    }

    /// `m1`'s group `carried-4`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_carried_4_part_1() {
        group_holds(M1, "carried-4", (0, 1), 12);
    }

    /// `m1`'s group `fixes-g1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g1_part_1() {
        group_holds(M1, "fixes-g1", (0, 1), 17);
    }

    /// `m1`'s group `fixes-g2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g2_part_1() {
        group_holds(M1, "fixes-g2", (0, 1), 20);
    }

    /// `m1`'s group `fixes-g3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g3_part_1() {
        group_holds(M1, "fixes-g3", (0, 1), 27);
    }

    /// `m1`'s group `fixes-g4`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g4_part_1() {
        group_holds(M1, "fixes-g4", (0, 1), 7);
    }

    /// `m1`'s group `fixes-g5`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g5_part_1() {
        group_holds(M1, "fixes-g5", (0, 1), 17);
    }

    /// `m1`'s group `fixes-g6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g6_part_1() {
        group_holds(M1, "fixes-g6", (0, 1), 14);
    }

    /// `m1`'s group `fixes-g7`, part 1 of 3 (its vectors at 0 modulo 3).
    #[test]
    fn m4_narrow_m1_fixes_g7_part_1() {
        group_holds(M1, "fixes-g7", (0, 3), 11);
    }

    /// `m1`'s group `fixes-g7`, part 2 of 3 (its vectors at 1 modulo 3).
    #[test]
    fn m4_narrow_m1_fixes_g7_part_2() {
        group_holds(M1, "fixes-g7", (1, 3), 11);
    }

    /// `m1`'s group `fixes-g7`, part 3 of 3 (its vectors at 2 modulo 3).
    #[test]
    fn m4_narrow_m1_fixes_g7_part_3() {
        group_holds(M1, "fixes-g7", (2, 3), 11);
    }

    /// `m1`'s group `fixes-g8`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g8_part_1() {
        group_holds(M1, "fixes-g8", (0, 1), 95);
    }

    /// `m1`'s group `fixes-g9`, part 1 of 7 (its vectors at 0 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_1() {
        group_holds(M1, "fixes-g9", (0, 7), 30);
    }

    /// `m1`'s group `fixes-g9`, part 2 of 7 (its vectors at 1 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_2() {
        group_holds(M1, "fixes-g9", (1, 7), 30);
    }

    /// `m1`'s group `fixes-g9`, part 3 of 7 (its vectors at 2 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_3() {
        group_holds(M1, "fixes-g9", (2, 7), 30);
    }

    /// `m1`'s group `fixes-g9`, part 4 of 7 (its vectors at 3 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_4() {
        group_holds(M1, "fixes-g9", (3, 7), 30);
    }

    /// `m1`'s group `fixes-g9`, part 5 of 7 (its vectors at 4 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_5() {
        group_holds(M1, "fixes-g9", (4, 7), 29);
    }

    /// `m1`'s group `fixes-g9`, part 6 of 7 (its vectors at 5 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_6() {
        group_holds(M1, "fixes-g9", (5, 7), 29);
    }

    /// `m1`'s group `fixes-g9`, part 7 of 7 (its vectors at 6 modulo 7).
    #[test]
    fn m4_narrow_m1_fixes_g9_part_7() {
        group_holds(M1, "fixes-g9", (6, 7), 29);
    }

    /// `m1`'s group `fixes-g10`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m1_fixes_g10_part_1() {
        group_holds(M1, "fixes-g10", (0, 2), 51);
    }

    /// `m1`'s group `fixes-g10`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m1_fixes_g10_part_2() {
        group_holds(M1, "fixes-g10", (1, 2), 50);
    }

    /// `m1`'s group `fixes-g11`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_g11_part_1() {
        group_holds(M1, "fixes-g11", (0, 1), 91);
    }

    /// `m1`'s group `fixes-g12`, part 1 of 6 (its vectors at 0 modulo 6).
    #[test]
    fn m4_narrow_m1_fixes_g12_part_1() {
        group_holds(M1, "fixes-g12", (0, 6), 37);
    }

    /// `m1`'s group `fixes-g12`, part 2 of 6 (its vectors at 1 modulo 6).
    #[test]
    fn m4_narrow_m1_fixes_g12_part_2() {
        group_holds(M1, "fixes-g12", (1, 6), 36);
    }

    /// `m1`'s group `fixes-g12`, part 3 of 6 (its vectors at 2 modulo 6).
    #[test]
    fn m4_narrow_m1_fixes_g12_part_3() {
        group_holds(M1, "fixes-g12", (2, 6), 36);
    }

    /// `m1`'s group `fixes-g12`, part 4 of 6 (its vectors at 3 modulo 6).
    #[test]
    fn m4_narrow_m1_fixes_g12_part_4() {
        group_holds(M1, "fixes-g12", (3, 6), 36);
    }

    /// `m1`'s group `fixes-g12`, part 5 of 6 (its vectors at 4 modulo 6).
    #[test]
    fn m4_narrow_m1_fixes_g12_part_5() {
        group_holds(M1, "fixes-g12", (4, 6), 36);
    }

    /// `m1`'s group `fixes-g12`, part 6 of 6 (its vectors at 5 modulo 6).
    #[test]
    fn m4_narrow_m1_fixes_g12_part_6() {
        group_holds(M1, "fixes-g12", (5, 6), 36);
    }

    /// `m1`'s group `fixes-g13`, part 1 of 5 (its vectors at 0 modulo 5).
    #[test]
    fn m4_narrow_m1_fixes_g13_part_1() {
        group_holds(M1, "fixes-g13", (0, 5), 69);
    }

    /// `m1`'s group `fixes-g13`, part 2 of 5 (its vectors at 1 modulo 5).
    #[test]
    fn m4_narrow_m1_fixes_g13_part_2() {
        group_holds(M1, "fixes-g13", (1, 5), 69);
    }

    /// `m1`'s group `fixes-g13`, part 3 of 5 (its vectors at 2 modulo 5).
    #[test]
    fn m4_narrow_m1_fixes_g13_part_3() {
        group_holds(M1, "fixes-g13", (2, 5), 69);
    }

    /// `m1`'s group `fixes-g13`, part 4 of 5 (its vectors at 3 modulo 5).
    #[test]
    fn m4_narrow_m1_fixes_g13_part_4() {
        group_holds(M1, "fixes-g13", (3, 5), 68);
    }

    /// `m1`'s group `fixes-g13`, part 5 of 5 (its vectors at 4 modulo 5).
    #[test]
    fn m4_narrow_m1_fixes_g13_part_5() {
        group_holds(M1, "fixes-g13", (4, 5), 68);
    }

    /// `m1`'s group `fixes-g14`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m1_fixes_g14_part_1() {
        group_holds(M1, "fixes-g14", (0, 2), 93);
    }

    /// `m1`'s group `fixes-g14`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m1_fixes_g14_part_2() {
        group_holds(M1, "fixes-g14", (1, 2), 93);
    }

    /// `m1`'s group `fixes-open`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_open_part_1() {
        group_holds(M1, "fixes-open", (0, 1), 3);
    }

    /// `m1`'s group `fixes-gate8`, part 1 of 9 (its vectors at 0 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_1() {
        group_holds(M1, "fixes-gate8", (0, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 2 of 9 (its vectors at 1 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_2() {
        group_holds(M1, "fixes-gate8", (1, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 3 of 9 (its vectors at 2 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_3() {
        group_holds(M1, "fixes-gate8", (2, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 4 of 9 (its vectors at 3 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_4() {
        group_holds(M1, "fixes-gate8", (3, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 5 of 9 (its vectors at 4 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_5() {
        group_holds(M1, "fixes-gate8", (4, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 6 of 9 (its vectors at 5 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_6() {
        group_holds(M1, "fixes-gate8", (5, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 7 of 9 (its vectors at 6 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_7() {
        group_holds(M1, "fixes-gate8", (6, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 8 of 9 (its vectors at 7 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_8() {
        group_holds(M1, "fixes-gate8", (7, 9), 32);
    }

    /// `m1`'s group `fixes-gate8`, part 9 of 9 (its vectors at 8 modulo 9).
    #[test]
    fn m4_narrow_m1_fixes_gate8_part_9() {
        group_holds(M1, "fixes-gate8", (8, 9), 32);
    }

    /// `m1`'s group `fixes-r1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r1_part_1() {
        group_holds(M1, "fixes-r1", (0, 1), 10);
    }

    /// `m1`'s group `fixes-r2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r2_part_1() {
        group_holds(M1, "fixes-r2", (0, 1), 7);
    }

    /// `m1`'s group `fixes-r3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r3_part_1() {
        group_holds(M1, "fixes-r3", (0, 1), 5);
    }

    /// `m1`'s group `fixes-r4`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r4_part_1() {
        group_holds(M1, "fixes-r4", (0, 1), 10);
    }

    /// `m1`'s group `fixes-r5`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r5_part_1() {
        group_holds(M1, "fixes-r5", (0, 1), 19);
    }

    /// `m1`'s group `fixes-r6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r6_part_1() {
        group_holds(M1, "fixes-r6", (0, 1), 11);
    }

    /// `m1`'s group `fixes-r7`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r7_part_1() {
        group_holds(M1, "fixes-r7", (0, 1), 14);
    }

    /// `m1`'s group `fixes-r8`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r8_part_1() {
        group_holds(M1, "fixes-r8", (0, 1), 6);
    }

    /// `m1`'s group `fixes-r9`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r9_part_1() {
        group_holds(M1, "fixes-r9", (0, 1), 18);
    }

    /// `m1`'s group `fixes-r10`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r10_part_1() {
        group_holds(M1, "fixes-r10", (0, 1), 5);
    }

    /// `m1`'s group `fixes-b`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_b_part_1() {
        group_holds(M1, "fixes-b", (0, 1), 18);
    }

    /// `m1`'s group `fixes-s`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_s_part_1() {
        group_holds(M1, "fixes-s", (0, 1), 23);
    }

    /// `m1`'s group `fixes-r11`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_fixes_r11_part_1() {
        group_holds(M1, "fixes-r11", (0, 1), 2);
    }

    /// `m1_spec`'s group `spec`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m1_spec_spec_part_1() {
        group_holds(M1_SPEC, "spec", (0, 1), 10);
    }

    /// `m2`'s group `checks-green`, part 1 of 5 (its vectors at 0 modulo 5).
    #[test]
    fn m4_narrow_m2_checks_green_part_1() {
        group_holds(M2, "checks-green", (0, 5), 4);
    }

    /// `m2`'s group `checks-green`, part 2 of 5 (its vectors at 1 modulo 5).
    #[test]
    fn m4_narrow_m2_checks_green_part_2() {
        group_holds(M2, "checks-green", (1, 5), 4);
    }

    /// `m2`'s group `checks-green`, part 3 of 5 (its vectors at 2 modulo 5).
    #[test]
    fn m4_narrow_m2_checks_green_part_3() {
        group_holds(M2, "checks-green", (2, 5), 4);
    }

    /// `m2`'s group `checks-green`, part 4 of 5 (its vectors at 3 modulo 5).
    #[test]
    fn m4_narrow_m2_checks_green_part_4() {
        group_holds(M2, "checks-green", (3, 5), 4);
    }

    /// `m2`'s group `checks-green`, part 5 of 5 (its vectors at 4 modulo 5).
    #[test]
    fn m4_narrow_m2_checks_green_part_5() {
        group_holds(M2, "checks-green", (4, 5), 4);
    }

    /// `m2`'s group `checks-records`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_checks_records_part_1() {
        group_holds(M2, "checks-records", (0, 1), 1);
    }

    /// `m2`'s group `checks-forms`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_checks_forms_part_1() {
        group_holds(M2, "checks-forms", (0, 1), 2);
    }

    /// `m2`'s group `items-green`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_items_green_part_1() {
        group_holds(M2, "items-green", (0, 1), 3);
    }

    /// `m2`'s group `items-forms`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_items_forms_part_1() {
        group_holds(M2, "items-forms", (0, 1), 1);
    }

    /// `m2`'s group `items-explain`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_items_explain_part_1() {
        group_holds(M2, "items-explain", (0, 1), 3);
    }

    /// `m2`'s group `items-windows`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_items_windows_part_1() {
        group_holds(M2, "items-windows", (0, 1), 1);
    }

    /// `m2`'s group `fixes-forms`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_forms_part_1() {
        group_holds(M2, "fixes-forms", (0, 1), 4);
    }

    /// `m2`'s group `fixes-sweep`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_sweep_part_1() {
        group_holds(M2, "fixes-sweep", (0, 1), 34);
    }

    /// `m2`'s group `fixes-explain`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_explain_part_1() {
        group_holds(M2, "fixes-explain", (0, 1), 6);
    }

    /// `m2`'s group `fixes-windows`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_windows_part_1() {
        group_holds(M2, "fixes-windows", (0, 1), 4);
    }

    /// `m2`'s group `items-sweep`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_items_sweep_part_1() {
        group_holds(M2, "items-sweep", (0, 1), 1);
    }

    /// `m2`'s group `goldens-records`, part 1 of 9 (its vectors at 0 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_1() {
        group_holds(M2, "goldens-records", (0, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 2 of 9 (its vectors at 1 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_2() {
        group_holds(M2, "goldens-records", (1, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 3 of 9 (its vectors at 2 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_3() {
        group_holds(M2, "goldens-records", (2, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 4 of 9 (its vectors at 3 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_4() {
        group_holds(M2, "goldens-records", (3, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 5 of 9 (its vectors at 4 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_5() {
        group_holds(M2, "goldens-records", (4, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 6 of 9 (its vectors at 5 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_6() {
        group_holds(M2, "goldens-records", (5, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 7 of 9 (its vectors at 6 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_7() {
        group_holds(M2, "goldens-records", (6, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 8 of 9 (its vectors at 7 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_8() {
        group_holds(M2, "goldens-records", (7, 9), 1);
    }

    /// `m2`'s group `goldens-records`, part 9 of 9 (its vectors at 8 modulo 9).
    #[test]
    fn m4_narrow_m2_goldens_records_part_9() {
        group_holds(M2, "goldens-records", (8, 9), 1);
    }

    /// `m2`'s group `goldens-ledger`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_goldens_ledger_part_1() {
        group_holds(M2, "goldens-ledger", (0, 1), 3);
    }

    /// `m2`'s group `fixes-records`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_records_part_1() {
        group_holds(M2, "fixes-records", (0, 1), 3);
    }

    /// `m2`'s group `fixes-ledger`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_fixes_ledger_part_1() {
        group_holds(M2, "fixes-ledger", (0, 2), 17);
    }

    /// `m2`'s group `fixes-ledger`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_fixes_ledger_part_2() {
        group_holds(M2, "fixes-ledger", (1, 2), 17);
    }

    /// `m2`'s group `hindcasts-h0`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h0_part_1() {
        group_holds(M2, "hindcasts-h0", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h0`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h0_part_2() {
        group_holds(M2, "hindcasts-h0", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h1`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h1_part_1() {
        group_holds(M2, "hindcasts-h1", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h1`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h1_part_2() {
        group_holds(M2, "hindcasts-h1", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h2`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h2_part_1() {
        group_holds(M2, "hindcasts-h2", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h2`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h2_part_2() {
        group_holds(M2, "hindcasts-h2", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h3`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h3_part_1() {
        group_holds(M2, "hindcasts-h3", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h3`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h3_part_2() {
        group_holds(M2, "hindcasts-h3", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h4`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h4_part_1() {
        group_holds(M2, "hindcasts-h4", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h4`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h4_part_2() {
        group_holds(M2, "hindcasts-h4", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h5`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h5_part_1() {
        group_holds(M2, "hindcasts-h5", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h5`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h5_part_2() {
        group_holds(M2, "hindcasts-h5", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h8`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h8_part_1() {
        group_holds(M2, "hindcasts-h8", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h8`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h8_part_2() {
        group_holds(M2, "hindcasts-h8", (1, 2), 1);
    }

    /// `m2`'s group `hindcasts-h9`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h9_part_1() {
        group_holds(M2, "hindcasts-h9", (0, 2), 1);
    }

    /// `m2`'s group `hindcasts-h9`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m2_hindcasts_h9_part_2() {
        group_holds(M2, "hindcasts-h9", (1, 2), 1);
    }

    /// `m2`'s group `fixes-r11`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_r11_part_1() {
        group_holds(M2, "fixes-r11", (0, 1), 14);
    }

    /// `m2`'s group `fixes-r12`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_r12_part_1() {
        group_holds(M2, "fixes-r12", (0, 1), 6);
    }

    /// `m2`'s group `fixes-r13`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_r13_part_1() {
        group_holds(M2, "fixes-r13", (0, 1), 5);
    }

    /// `m2`'s group `fixes-r15`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_r15_part_1() {
        group_holds(M2, "fixes-r15", (0, 1), 9);
    }

    /// `m2`'s group `fixes-r16`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_r16_part_1() {
        group_holds(M2, "fixes-r16", (0, 1), 4);
    }

    /// `m2`'s group `fixes-s`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_s_part_1() {
        group_holds(M2, "fixes-s", (0, 1), 8);
    }

    /// `m2`'s group `fixes-d`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_d_part_1() {
        group_holds(M2, "fixes-d", (0, 1), 12);
    }

    /// `m2`'s group `fixes-cut`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_cut_part_1() {
        group_holds(M2, "fixes-cut", (0, 1), 5);
    }

    /// `m2`'s group `fixes-rv`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_rv_part_1() {
        group_holds(M2, "fixes-rv", (0, 1), 17);
    }

    /// `m2`'s group `fixes-22`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_fixes_22_part_1() {
        group_holds(M2, "fixes-22", (0, 1), 43);
    }

    /// `m2_spec`'s group `spec-forms`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_spec_spec_forms_part_1() {
        group_holds(M2_SPEC, "spec-forms", (0, 1), 1);
    }

    /// `m2_spec`'s group `spec-sweep`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m2_spec_spec_sweep_part_1() {
        group_holds(M2_SPEC, "spec-sweep", (0, 1), 5);
    }

    /// `m4`'s group `checks-green`, part 1 of 34 (its vectors at 0 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_1() {
        group_holds(M4, "checks-green", (0, 34), 3);
    }

    /// `m4`'s group `checks-green`, part 2 of 34 (its vectors at 1 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_2() {
        group_holds(M4, "checks-green", (1, 34), 3);
    }

    /// `m4`'s group `checks-green`, part 3 of 34 (its vectors at 2 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_3() {
        group_holds(M4, "checks-green", (2, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 4 of 34 (its vectors at 3 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_4() {
        group_holds(M4, "checks-green", (3, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 5 of 34 (its vectors at 4 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_5() {
        group_holds(M4, "checks-green", (4, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 6 of 34 (its vectors at 5 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_6() {
        group_holds(M4, "checks-green", (5, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 7 of 34 (its vectors at 6 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_7() {
        group_holds(M4, "checks-green", (6, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 8 of 34 (its vectors at 7 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_8() {
        group_holds(M4, "checks-green", (7, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 9 of 34 (its vectors at 8 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_9() {
        group_holds(M4, "checks-green", (8, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 10 of 34 (its vectors at 9 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_10() {
        group_holds(M4, "checks-green", (9, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 11 of 34 (its vectors at 10 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_11() {
        group_holds(M4, "checks-green", (10, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 12 of 34 (its vectors at 11 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_12() {
        group_holds(M4, "checks-green", (11, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 13 of 34 (its vectors at 12 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_13() {
        group_holds(M4, "checks-green", (12, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 14 of 34 (its vectors at 13 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_14() {
        group_holds(M4, "checks-green", (13, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 15 of 34 (its vectors at 14 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_15() {
        group_holds(M4, "checks-green", (14, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 16 of 34 (its vectors at 15 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_16() {
        group_holds(M4, "checks-green", (15, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 17 of 34 (its vectors at 16 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_17() {
        group_holds(M4, "checks-green", (16, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 18 of 34 (its vectors at 17 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_18() {
        group_holds(M4, "checks-green", (17, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 19 of 34 (its vectors at 18 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_19() {
        group_holds(M4, "checks-green", (18, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 20 of 34 (its vectors at 19 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_20() {
        group_holds(M4, "checks-green", (19, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 21 of 34 (its vectors at 20 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_21() {
        group_holds(M4, "checks-green", (20, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 22 of 34 (its vectors at 21 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_22() {
        group_holds(M4, "checks-green", (21, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 23 of 34 (its vectors at 22 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_23() {
        group_holds(M4, "checks-green", (22, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 24 of 34 (its vectors at 23 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_24() {
        group_holds(M4, "checks-green", (23, 34), 1);
    }

    /// `m4`'s group `checks-green`, part 25 of 34 (its vectors at 24 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_25() {
        group_holds(M4, "checks-green", (24, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 26 of 34 (its vectors at 25 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_26() {
        group_holds(M4, "checks-green", (25, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 27 of 34 (its vectors at 26 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_27() {
        group_holds(M4, "checks-green", (26, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 28 of 34 (its vectors at 27 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_28() {
        group_holds(M4, "checks-green", (27, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 29 of 34 (its vectors at 28 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_29() {
        group_holds(M4, "checks-green", (28, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 30 of 34 (its vectors at 29 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_30() {
        group_holds(M4, "checks-green", (29, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 31 of 34 (its vectors at 30 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_31() {
        group_holds(M4, "checks-green", (30, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 32 of 34 (its vectors at 31 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_32() {
        group_holds(M4, "checks-green", (31, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 33 of 34 (its vectors at 32 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_33() {
        group_holds(M4, "checks-green", (32, 34), 2);
    }

    /// `m4`'s group `checks-green`, part 34 of 34 (its vectors at 33 modulo 34).
    #[test]
    fn m4_narrow_m4_checks_green_part_34() {
        group_holds(M4, "checks-green", (33, 34), 2);
    }

    /// `m4`'s group `checks-pins`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m4_checks_pins_part_1() {
        group_holds(M4, "checks-pins", (0, 2), 1);
    }

    /// `m4`'s group `checks-pins`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m4_checks_pins_part_2() {
        group_holds(M4, "checks-pins", (1, 2), 1);
    }

    /// `m4`'s group `checks-accrue`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_checks_accrue_part_1() {
        group_holds(M4, "checks-accrue", (0, 1), 1);
    }

    /// `m4`'s group `constructs-pins`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_pins_part_1() {
        group_holds(M4, "constructs-pins", (0, 1), 15);
    }

    /// `m4`'s group `constructs-accrue`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_accrue_part_1() {
        group_holds(M4, "constructs-accrue", (0, 1), 9);
    }

    /// `m4`'s group `constructs-round`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_round_part_1() {
        group_holds(M4, "constructs-round", (0, 1), 16);
    }

    /// `m4`'s group `constructs-logic`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_logic_part_1() {
        group_holds(M4, "constructs-logic", (0, 1), 14);
    }

    /// `m4`'s group `constructs-aggregates`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_aggregates_part_1() {
        group_holds(M4, "constructs-aggregates", (0, 1), 11);
    }

    /// `m4`'s group `constructs-argm`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_argm_part_1() {
        group_holds(M4, "constructs-argm", (0, 1), 8);
    }

    /// `m4`'s group `constructs-tuple`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_tuple_part_1() {
        group_holds(M4, "constructs-tuple", (0, 1), 2);
    }

    /// `m4`'s group `constructs-beats`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_beats_part_1() {
        group_holds(M4, "constructs-beats", (0, 1), 7);
    }

    /// `m4`'s group `constructs-fx`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_fx_part_1() {
        group_holds(M4, "constructs-fx", (0, 1), 7);
    }

    /// `m4`'s group `constructs-ratepin`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_ratepin_part_1() {
        group_holds(M4, "constructs-ratepin", (0, 1), 3);
    }

    /// `m4`'s group `constructs-cal`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_cal_part_1() {
        group_holds(M4, "constructs-cal", (0, 1), 13);
    }

    /// `m4`'s group `constructs-paper`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m4_constructs_paper_part_1() {
        group_holds(M4, "constructs-paper", (0, 2), 68);
    }

    /// `m4`'s group `constructs-paper`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m4_constructs_paper_part_2() {
        group_holds(M4, "constructs-paper", (1, 2), 67);
    }

    /// `m4`'s group `constructs-keydiff`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_constructs_keydiff_part_1() {
        group_holds(M4, "constructs-keydiff", (0, 1), 2);
    }

    /// `m4`'s group `items-paper`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_items_paper_part_1() {
        group_holds(M4, "items-paper", (0, 1), 4);
    }

    /// `m4`'s group `items-walked`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_items_walked_part_1() {
        group_holds(M4, "items-walked", (0, 1), 7);
    }

    /// `m4`'s group `items-backlog`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_items_backlog_part_1() {
        group_holds(M4, "items-backlog", (0, 1), 17);
    }

    /// `m4`'s group `fixes-g1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g1_part_1() {
        group_holds(M4, "fixes-g1", (0, 1), 51);
    }

    /// `m4`'s group `fixes-g2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g2_part_1() {
        group_holds(M4, "fixes-g2", (0, 1), 55);
    }

    /// `m4`'s group `fixes-p12`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m4_fixes_p12_part_1() {
        group_holds(M4, "fixes-p12", (0, 2), 95);
    }

    /// `m4`'s group `fixes-p12`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m4_fixes_p12_part_2() {
        group_holds(M4, "fixes-p12", (1, 2), 95);
    }

    /// `m4`'s group `fixes-g2fix`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g2fix_part_1() {
        group_holds(M4, "fixes-g2fix", (0, 1), 33);
    }

    /// `m4`'s group `fixes-g3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g3_part_1() {
        group_holds(M4, "fixes-g3", (0, 1), 36);
    }

    /// `m4`'s group `fixes-g3fix`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g3fix_part_1() {
        group_holds(M4, "fixes-g3fix", (0, 1), 33);
    }

    /// `m4`'s group `fixes-p21`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p21_part_1() {
        group_holds(M4, "fixes-p21", (0, 1), 7);
    }

    /// `m4`'s group `fixes-narrow`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_narrow_part_1() {
        group_holds(M4, "fixes-narrow", (0, 1), 48);
    }

    /// `m4`'s group `fixes-traced`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_traced_part_1() {
        group_holds(M4, "fixes-traced", (0, 1), 2);
    }

    /// `m4`'s group `fixes-g6`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g6_part_1() {
        group_holds(M4, "fixes-g6", (0, 1), 19);
    }

    /// `m4`'s group `fixes-p22`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p22_part_1() {
        group_holds(M4, "fixes-p22", (0, 1), 5);
    }

    /// `m4`'s group `fixes-p23`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p23_part_1() {
        group_holds(M4, "fixes-p23", (0, 1), 4);
    }

    /// `m4`'s group `fixes-g4`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g4_part_1() {
        group_holds(M4, "fixes-g4", (0, 1), 2);
    }

    /// `m4`'s group `fixes-g6cfix`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_g6cfix_part_1() {
        group_holds(M4, "fixes-g6cfix", (0, 1), 13);
    }

    /// `m4`'s group `fixes-p30`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p30_part_1() {
        group_holds(M4, "fixes-p30", (0, 1), 17);
    }

    /// `m4`'s group `fixes-p32`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p32_part_1() {
        group_holds(M4, "fixes-p32", (0, 1), 15);
    }

    /// `m4`'s group `fixes-gate`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_gate_part_1() {
        group_holds(M4, "fixes-gate", (0, 1), 42);
    }

    /// `m4`'s group `fixes-gatefix`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_gatefix_part_1() {
        group_holds(M4, "fixes-gatefix", (0, 1), 36);
    }

    /// `m4`'s group `fixes-gatelead`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_gatelead_part_1() {
        group_holds(M4, "fixes-gatelead", (0, 1), 19);
    }

    /// `m4`'s group `fixes-gate8`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_gate8_part_1() {
        group_holds(M4, "fixes-gate8", (0, 1), 19);
    }

    /// `m4`'s group `fixes-p40`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p40_part_1() {
        group_holds(M4, "fixes-p40", (0, 1), 12);
    }

    /// `m4`'s group `fixes-p40fix`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m4_fixes_p40fix_part_1() {
        group_holds(M4, "fixes-p40fix", (0, 2), 42);
    }

    /// `m4`'s group `fixes-p40fix`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m4_fixes_p40fix_part_2() {
        group_holds(M4, "fixes-p40fix", (1, 2), 42);
    }

    /// `m4`'s group `fixes-p43`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_p43_part_1() {
        group_holds(M4, "fixes-p43", (0, 1), 10);
    }

    /// `m4`'s group `fixes-t231g1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_t231g1_part_1() {
        group_holds(M4, "fixes-t231g1", (0, 1), 105);
    }

    /// `m4`'s group `fixes-t231g2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_t231g2_part_1() {
        group_holds(M4, "fixes-t231g2", (0, 1), 68);
    }

    /// `m4`'s group `fixes-t231rv1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_fixes_t231rv1_part_1() {
        group_holds(M4, "fixes-t231rv1", (0, 1), 59);
    }

    /// `m4`'s group `fixes-t231rv2`, part 1 of 2 (its vectors at 0 modulo 2).
    #[test]
    fn m4_narrow_m4_fixes_t231rv2_part_1() {
        group_holds(M4, "fixes-t231rv2", (0, 2), 161);
    }

    /// `m4`'s group `fixes-t231rv2`, part 2 of 2 (its vectors at 1 modulo 2).
    #[test]
    fn m4_narrow_m4_fixes_t231rv2_part_2() {
        group_holds(M4, "fixes-t231rv2", (1, 2), 160);
    }

    /// `m4_spec`'s group `spec-g1`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_g1_part_1() {
        group_holds(M4_SPEC, "spec-g1", (0, 1), 17);
    }

    /// `m4_spec`'s group `spec-g2`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_g2_part_1() {
        group_holds(M4_SPEC, "spec-g2", (0, 1), 11);
    }

    /// `m4_spec`'s group `spec-p12`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_p12_part_1() {
        group_holds(M4_SPEC, "spec-p12", (0, 1), 1);
    }

    /// `m4_spec`'s group `spec-g2fix`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_g2fix_part_1() {
        group_holds(M4_SPEC, "spec-g2fix", (0, 1), 8);
    }

    /// `m4_spec`'s group `spec-g3`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_g3_part_1() {
        group_holds(M4_SPEC, "spec-g3", (0, 1), 9);
    }

    /// `m4_spec`'s group `spec-g3fix`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_g3fix_part_1() {
        group_holds(M4_SPEC, "spec-g3fix", (0, 1), 10);
    }

    /// `m4_spec`'s group `spec-p21`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_p21_part_1() {
        group_holds(M4_SPEC, "spec-p21", (0, 1), 1);
    }

    /// `m4_spec`'s group `spec-p22`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_p22_part_1() {
        group_holds(M4_SPEC, "spec-p22", (0, 1), 1);
    }

    /// `m4_spec`'s group `spec-p26`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_p26_part_1() {
        group_holds(M4_SPEC, "spec-p26", (0, 1), 11);
    }

    /// `m4_spec`'s group `spec-p31`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_p31_part_1() {
        group_holds(M4_SPEC, "spec-p31", (0, 1), 3);
    }

    /// `m4_spec`'s group `spec-gate`, part 1 of 1 (its vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m4_spec_spec_gate_part_1() {
        group_holds(M4_SPEC, "spec-gate", (0, 1), 6);
    }

    /// M3's `runs.vectors`, part 1 of 7 (its run vectors at 0 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_1() {
        m3_holds(M3_RUNS, (0, 7), 230, 33);
    }

    /// M3's `runs.vectors`, part 2 of 7 (its run vectors at 1 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_2() {
        m3_holds(M3_RUNS, (1, 7), 230, 33);
    }

    /// M3's `runs.vectors`, part 3 of 7 (its run vectors at 2 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_3() {
        m3_holds(M3_RUNS, (2, 7), 230, 33);
    }

    /// M3's `runs.vectors`, part 4 of 7 (its run vectors at 3 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_4() {
        m3_holds(M3_RUNS, (3, 7), 230, 33);
    }

    /// M3's `runs.vectors`, part 5 of 7 (its run vectors at 4 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_5() {
        m3_holds(M3_RUNS, (4, 7), 230, 33);
    }

    /// M3's `runs.vectors`, part 6 of 7 (its run vectors at 5 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_6() {
        m3_holds(M3_RUNS, (5, 7), 230, 33);
    }

    /// M3's `runs.vectors`, part 7 of 7 (its run vectors at 6 modulo 7).
    #[test]
    fn m4_narrow_m3_runs_part_7() {
        m3_holds(M3_RUNS, (6, 7), 230, 32);
    }

    /// M3's `spec.vectors`, part 1 of 1 (its run vectors at 0 modulo 1).
    #[test]
    fn m4_narrow_m3_spec_part_1() {
        m3_holds(M3_SPEC, (0, 1), 40, 40);
    }

    /// the kernel corpus, part 1 of 4 (its records at 0 modulo 4).
    #[test]
    fn m4_narrow_kernel_corpus_part_1() {
        corpus_holds(KERNEL_CORPUS, (0, 4), 312, 78);
    }

    /// the kernel corpus, part 2 of 4 (its records at 1 modulo 4).
    #[test]
    fn m4_narrow_kernel_corpus_part_2() {
        corpus_holds(KERNEL_CORPUS, (1, 4), 312, 78);
    }

    /// the kernel corpus, part 3 of 4 (its records at 2 modulo 4).
    #[test]
    fn m4_narrow_kernel_corpus_part_3() {
        corpus_holds(KERNEL_CORPUS, (2, 4), 312, 78);
    }

    /// the kernel corpus, part 4 of 4 (its records at 3 modulo 4).
    #[test]
    fn m4_narrow_kernel_corpus_part_4() {
        corpus_holds(KERNEL_CORPUS, (3, 4), 312, 78);
    }

    /// M0's corpus, part 1 of 3 (its records at 0 modulo 3).
    #[test]
    fn m4_narrow_m0_corpus_part_1() {
        corpus_holds(M0_CORPUS, (0, 3), 285, 95);
    }

    /// M0's corpus, part 2 of 3 (its records at 1 modulo 3).
    #[test]
    fn m4_narrow_m0_corpus_part_2() {
        corpus_holds(M0_CORPUS, (1, 3), 285, 95);
    }

    /// M0's corpus, part 3 of 3 (its records at 2 modulo 3).
    #[test]
    fn m4_narrow_m0_corpus_part_3() {
        corpus_holds(M0_CORPUS, (2, 3), 285, 95);
    }

    /// M1's corpus, part 1 of 9 (its records at 0 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_1() {
        corpus_holds(M1_CORPUS, (0, 9), 648, 72);
    }

    /// M1's corpus, part 2 of 9 (its records at 1 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_2() {
        corpus_holds(M1_CORPUS, (1, 9), 648, 72);
    }

    /// M1's corpus, part 3 of 9 (its records at 2 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_3() {
        corpus_holds(M1_CORPUS, (2, 9), 648, 72);
    }

    /// M1's corpus, part 4 of 9 (its records at 3 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_4() {
        corpus_holds(M1_CORPUS, (3, 9), 648, 72);
    }

    /// M1's corpus, part 5 of 9 (its records at 4 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_5() {
        corpus_holds(M1_CORPUS, (4, 9), 648, 72);
    }

    /// M1's corpus, part 6 of 9 (its records at 5 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_6() {
        corpus_holds(M1_CORPUS, (5, 9), 648, 72);
    }

    /// M1's corpus, part 7 of 9 (its records at 6 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_7() {
        corpus_holds(M1_CORPUS, (6, 9), 648, 72);
    }

    /// M1's corpus, part 8 of 9 (its records at 7 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_8() {
        corpus_holds(M1_CORPUS, (7, 9), 648, 72);
    }

    /// M1's corpus, part 9 of 9 (its records at 8 modulo 9).
    #[test]
    fn m4_narrow_m1_corpus_part_9() {
        corpus_holds(M1_CORPUS, (8, 9), 648, 72);
    }

    /// M2's corpus, part 1 of 3 (its records at 0 modulo 3).
    #[test]
    fn m4_narrow_m2_corpus_part_1() {
        corpus_holds(M2_CORPUS, (0, 3), 168, 56);
    }

    /// M2's corpus, part 2 of 3 (its records at 1 modulo 3).
    #[test]
    fn m4_narrow_m2_corpus_part_2() {
        corpus_holds(M2_CORPUS, (1, 3), 168, 56);
    }

    /// M2's corpus, part 3 of 3 (its records at 2 modulo 3).
    #[test]
    fn m4_narrow_m2_corpus_part_3() {
        corpus_holds(M2_CORPUS, (2, 3), 168, 56);
    }

    /// F7's own reality `n1_fulfils.tally`: every ask, then 5 asks in reverse order, alike under both forms.
    #[test]
    fn m4_narrow_own_n1_fulfils() {
        own_holds(
            "n1_fulfils.tally",
            &[("n1_fulfils.tally", N1_FULFILS)],
            &["i_may", "i_feb", "l", "f", "k"],
        );
    }

    /// F7's own reality `n2_flows.tally`: every ask, then 7 asks in reverse order, alike under both forms.
    #[test]
    fn m4_narrow_own_n2_flows() {
        own_holds(
            "n2_flows.tally",
            &[("n2_flows.tally", N2_FLOWS)],
            &["i_may", "i_jan", "e_jan", "k", "g", "fe", "r"],
        );
    }

    /// F7's own reality `n3_window.tally`: every ask, then 18 asks in reverse order, alike under both forms.
    #[test]
    fn m4_narrow_own_n3_window() {
        own_holds(
            "n3_window.tally",
            &[("n3_window.tally", N3_WINDOW)],
            &[
                "both", "l_pot", "x_after", "x_full", "c_after", "c_full", "w_after", "w_before",
                "w_full", "p_days", "p_min", "p_past", "p_after", "p_before", "p_hi", "p_lo",
                "p_inner", "p_full",
            ],
        );
    }

    /// F7's own reality `n4_own.tally`: every ask, then 3 asks in reverse order, alike under both forms.
    #[test]
    fn m4_narrow_own_n4_own() {
        own_holds(
            "n4_own.tally",
            &[("n4_own.tally", N4_OWN)],
            &["l", "f", "k"],
        );
    }

    /// P8-L10's `c1_r2`, its six asks in `m4/load/runs.tsv`'s order, alike under both forms.
    #[test]
    fn m4_narrow_c1_r2() {
        let rest = words(
            &C1_R2,
            &[
                "explain_room",
                "ike_left",
                "konto_balance",
                "net_worth",
                "replay_september",
                "taxes_paid",
            ],
        );
        let r = same(&rest, &table_of_texts(&C1_R2));
        judge("c1_r2", vec![("c1_r2".to_owned(), r)], 1);
    }

    /// P8-14's check, `replay_september` and `konto_balance` at `b_r1`, alike under both forms.
    #[test]
    fn m4_narrow_p8_14() {
        let results = ["replay_september", "konto_balance"]
            .iter()
            .map(|ask| {
                let rest = words(&B_R1, &[ask]);
                ((*ask).to_owned(), same(&rest, &table_of_texts(&B_R1)))
            })
            .collect();
        judge("P8-14's check", results, 2);
    }
}
