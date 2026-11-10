//! Milestone M1's fence F5 on T's realities (pick P57; R-130, R-84; `MILESTONE-M1.md` row 9;
//! issue #108). Every ask of T's corpora (`properties/kernel.corpus`, 312 records;
//! `properties/m1.corpus`, 648), of every fix vector of `m1/vectors.refcore` (groups
//! `fixes-…`) and of F5's own realities (`m1/lows/`, G9's `tally-cli/tests/lows/` copied) runs
//! through `tally::run_with_lows` twice, with `Lows::Tree` (fence F5's tree of running sums,
//! P §2.3, as `run` reads) and with `Lows::Walk` (the projection's own walk, its plain twin,
//! E §4.3), and the two print the same lines. A difference is the fence's fault
//! (`run_with_lows`'s doc). Since F5 answers T's projection reads, T's vectors reach the twin
//! only where the tree declines; this target reaches it on every read, so the twin's mutants
//! are T's to catch (R-84 counts T's tests at gate 8; `mutants-local --next` counts this
//! target). One test per source, so a failure names it. `test = false` until M1's marker
//! (pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::walk::Lows;
    use tally::{Asks, Days, Given, NotImplemented, Pin, Request, run_with_lows};
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

    /// Every record of `corpus` alike under the tree and the twin; at least `answered` of them
    /// answered by both (no vacuous pass).
    fn corpus_holds(corpus: &[u8], records_in: usize, answered: usize) {
        let all = records(corpus).expect("the corpus reads");
        assert_eq!(all.len(), records_in, "the corpus's records");
        let mut answered_by_both = Vec::new();
        let mut failed = Vec::new();
        for r in &all {
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
            all.len(),
            failed.join("\n")
        );
        assert!(both >= answered, "{both} records answered by both");
    }

    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_the_kernel_corpus() {
        corpus_holds(KERNEL_CORPUS, 312, 312);
    }

    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_the_m1_corpus() {
        corpus_holds(M1_CORPUS, 648, 480);
    }

    /// Every fix vector of `m1/vectors.refcore` (groups `fixes-g1` to `fixes-g9`, G9's review
    /// probes among them), each ask alike under the tree and the twin.
    #[test]
    fn m1_lows_the_tree_prints_the_walk_on_the_fix_vectors() {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all
            .iter()
            .filter(|v| v.group.starts_with("fixes-"))
            .collect();
        assert!(mine.len() >= 400, "the fix vectors are in the store");
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
            "{} of {} vectors differ:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
        assert!(both >= 380, "{both} vectors answered by both");
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
}
