//! Milestone M1's review fixes on the engine (issue #108; `MILESTONE-M1.md` row 8; the
//! engine-coordinator skill: "T then turns each fix into tests"): each blocking finding a
//! builder group's review fixed, and each mutant its `mutants-local` left for T, as a fixture
//! under `m1/fixes/g<n>/`, refcore's whole output in `m1/vectors.refcore` (groups `fixes-g<n>`),
//! the engine's held to it byte for byte. Where refcore and the spec differ, the vector keeps
//! refcore's bytes and is held "for #n" in `held.tsv` (#112 to #127). One test per group, so a
//! failure names the group whose fix it undoes. `test = false` until M1's marker (pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m1::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 9] = [
        "fixes-g1", "fixes-g2", "fixes-g3", "fixes-g4", "fixes-g5", "fixes-g6", "fixes-g7",
        "fixes-g8", "fixes-g9",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m1_fixes", &v.id))
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
    fn m1_fixes_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m1/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("fixes-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each fixes- group of m1/vectors.tsv has its test here"
        );
    }

    /// G1's review fixes (`m1/fixes/g1/`): a conjunction narrows the binding inside an `if`
    /// (`g in big and g.size > 0` and its twins, L §5.10); a rule's `when` narrows it, and a
    /// def keyed by a movement's binding is read (L §5.11); `unreplaced`'s survivors: a replace
    /// row beats only the rows at the same keys known before it, a scenario's beats its base's
    /// whatever their known dates and a base's replace never beats a scenario's (L §4.6 steps
    /// 2 and 3, §6.4); and the load refusals that leave two `mutants-local` survivors no input
    /// at run (two rows of one layer known one day, `LOAD-OVERLAP`; a dated row read with no
    /// law date, `LOAD-LAWDATE`).
    #[test]
    fn m1_fixes_g1_narrowing_and_replacement() {
        group_holds("fixes-g1");
    }

    /// G2's review fixes (`m1/fixes/g2/`): a window's identity, two reads of one item at the
    /// same keys being one value and nothing else (L §4.4; held for #115 where refcore
    /// differs); a window at 9999-12-31; `pow` of a fraction to a negative power, of zero, and
    /// of ten under and over the 4 096-digit cap (`num/rational.rs`'s mutants, the review's
    /// note 9).
    #[test]
    fn m1_fixes_g2_windows_and_powers() {
        group_holds("fixes-g2");
    }

    /// G3's review fixes (`m1/fixes/g3/`): a window in a `ties by` (a hole where undecided,
    /// P33, #115); a later `ties by` key read only among the still tied (#116); `first` and
    /// `last` stop at a key (#117); `last(x, d)` over an always or open-start row (#118); one
    /// window's argmin (#115); `since(x, d)`; a `last` at an instant (`read/rows.rs`'s
    /// mutant); the fix review's `where` holes and register.
    #[test]
    fn m1_fixes_g3_order_and_last() {
        group_holds("fixes-g3");
    }

    /// G4's (`m1/fixes/g4/`): MB1's eight missed `num/split.rs` mutants, a `rest last` split
    /// and a `rest largest` one with distinct remainders, steps left over and a negative total
    /// (L §5.13); the G5 review's sign case, a positive and a negative `rest largest` total in
    /// one ask (its blocking 1, G4's touch-up).
    #[test]
    fn m1_fixes_g4_prorata() {
        group_holds("fixes-g4");
    }

    /// G5's (`m1/fixes/g5/`): MB2's six wording-pin mutants (`bands.rs:67`, `rules.rs:43`);
    /// the review's blocking 2 (a deep chain under a top-level `as if`), 3 (`solve` to a step
    /// that is `none`), 4 (a step that isn't positive), 5 (#120, held); P38's two pins at one
    /// cell, P39's bare word, P41's `with known` after `on` (#112, held).
    #[test]
    fn m1_fixes_g5_pins_and_solve() {
        group_holds("fixes-g5");
    }

    /// G6's (`m1/fixes/g6/`): the review's blocking 1, a scenario's move in the model's world
    /// (an adjustment, a binding, a diff); 2 and 3, E §9 item 3's listed movement (#121,
    /// held); MB2's two survivors, a scenario's `replace rule` and band table read in the
    /// model's world (`read/rules.rs:39`, `read/bands.rs:65`); and C6's find, a decision
    /// before `from` beside a listed movement (#121, held; red for G6 in `m1_properties`).
    #[test]
    fn m1_fixes_g6_worlds() {
        group_holds("fixes-g6");
    }

    /// G7's (`m1/fixes/g7/`): the review's B1 and the fix reviews' texts (`fl`, `fl2`, `tl`: a
    /// move's flows beside another move's axis; `wn`: a warning kept where the walk reads no
    /// unset axis; `c64`, `c64d`: 64 combinations by month and by day; `pr1`, `pr2` pinned);
    /// a diff over an axis (P47) and a leg refused under some options read outside the fan-out
    /// (the fix review's NB2, NB3), both held for #122; a bracketed period literal beside a key
    /// of a `by 3` index (#113, held).
    #[test]
    fn m1_fixes_g7_axes() {
        group_holds("fixes-g7");
    }

    /// G8's (`m1/fixes/g8/`): the fixes' `f1` and the review's forms no probe reached: `low`
    /// plain, with its clauses, one day, in a decision on its own day (`RUN-CYCLE`); `gap` and a
    /// pad's flows, in a scenario's world beside its own reading; `value` by a clause keyed by
    /// a kind holding neither the place nor its unit (`LOAD-KEY`), per another unit
    /// (`LOAD-UNIT`), a price def, a law-dated price, a carry before its first reading, an
    /// unkeyed price past its row, a place alive before the start with no reading. Held:
    /// `held:` on a price read through its carry (P49, #123); `last` over an `always` row
    /// (P50, #118); `low … with uncertain` without `from` (P51, #125); a whole-priced `value`
    /// by a wrong unit or a written expression (P53, #126); a price keyed by unit in another
    /// currency (the fix review's NB1, #126, red for G14 below).
    #[test]
    fn m1_fixes_g8_projections_and_prices() {
        group_holds("fixes-g8");
    }

    /// G9's (`m1/fixes/g9/`): the fix reviews' texts: a lifespan's end in a projection
    /// (`f_life`), a rent a version covers read from two cuts (`cov_x`, `cov_nocov`, B1), a
    /// booked A0 without its instance's day (`inv_min`, B2), a tree updated in place (`u_*`);
    /// a failed projection walk read through windows on both sides of its day, in either order
    /// (P54, #127, held where refcore differs).
    #[test]
    fn m1_fixes_g9_the_tree_and_its_twin() {
        group_holds("fixes-g9");
    }

    /// A price of another currency is refused, never a figure in the wrong unit (L 5.10: `by p`
    /// is a `num CUR/u` in the asked CUR; L 3.2; pick P53, "a number from mismatched units is
    /// never right"): every vector of `fixes/g8/units_*` and `fixes/g8/keyed_*` refuses its ask,
    /// `LOAD-UNIT` or `RUN-DOMAIN`, the spec's reading, not refcore's bytes. The clause, the
    /// written price and the carried one, and a whole-priced place, refuse on the engine; a price
    /// keyed by unit (`by fxs`, `by fxd`, `by last(fxu)`, G8's fix review NB1) prints a PLN figure
    /// labelled USD on both, held for #126 in its group: red for G14.
    #[test]
    fn m1_fixes_g8_a_price_in_another_currency_is_refused() {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all
            .iter()
            .filter(|v| v.id.starts_with("fixes/g8/units_") || v.id.starts_with("fixes/g8/keyed_"))
            .collect();
        assert!(mine.len() >= 8, "the unit vectors are in the store");
        let failed: Vec<String> = mine
            .iter()
            .filter_map(|v| {
                let out = run_args(&v.args, &v.files).out;
                let Some(line) = out.lines().find(|l| !l.starts_with("warning: ")) else {
                    return Some(format!("{}: no answer line", v.id));
                };
                let refused = ["refused: LOAD-UNIT ", "refused: RUN-DOMAIN "]
                    .iter()
                    .any(|code| line.contains(code));
                (!refused).then(|| format!("{}: {line}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} asks answer a price of another currency:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }
}
