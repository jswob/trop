//! Milestone M2's in-scope items on the engine (issue #167; `MILESTONE-M2.md` row 6): the
//! forms the run's brief puts in scope (#147, #101, #42, #132, #163, #164, #158, item 9's
//! pair, P6's known windows and `moves` at run), each a text of T's under `m2/items/` with
//! refcore's output in `m2/vectors.refcore`, the engine's held to it byte for byte. One test
//! per builder group (the `for` column of `m2/vectors.tsv`), so a group's test turns green
//! with its group, and one for the items the engine answered before M2 (`items-green`);
//! every failure of a group is collected before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m2::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 4] = [
        "items-green",
        "items-forms",
        "items-explain",
        "items-windows",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m2/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m2_items", &v.id))
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
    fn m2_items_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m2/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("items-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each items- group of m2/vectors.tsv has its test here"
        );
    }

    /// The items the engine answers byte for byte on arrival: #42, an ask's `for` over a
    /// year index (L §5.18: LAW-09's `each_year` on its own); R8's reach (#147, L §5.5:
    /// `solve` whose F reads an axis, `inputs-of` across it, a balance and its register per
    /// combination, `residuals` pinned to one option); item 9 (P5): `add_business` with a
    /// hole calendar inside its edge, and past the edge over a calendar governed always
    /// (`RUN-LIMIT`, L §8). It keeps them so.
    #[test]
    fn m2_items_green() {
        group_holds("items-green");
    }

    /// G1 `forms`: #101's other forms (L §2.3, §5.10, §4.6): a record input's field as a
    /// price, unkeyed; a field of `last(…)`, keyed and unkeyed, read past its reading and
    /// before the first; `value … by` a keyed field, on a reading's day and after it.
    #[test]
    fn m2_items_forms() {
        group_holds("items-forms");
    }

    /// G2 `explain`: `readers-of` a kind and its subkind (#132); `explain` of a balance whose
    /// last reading pads, anchors, or has a hole movement since it (#132, E §6.3); `last` of
    /// readings, an `after m` one before its day (#163, #164, L §4.6); `explain` of a def
    /// bound by the walk; a place of unit `any` with an opening row per unit (#163); `before
    /// from:` under `flows` and `bal`, for a leg dated before `from` with its law date after,
    /// and for a leg whose day is a window, under `windows each` (#164 R7, #163, L §6.4).
    #[test]
    fn m2_items_explain() {
        group_holds("items-explain");
    }

    /// G3 `windows`: `windows each` on an ask over `add_business`, over a balance whose
    /// move's `at` is a window, under an axis and under a pin, and on `low … from` (#158,
    /// #164 R6, L §4.4); a `known` window at run on a version, a row and an observation's
    /// version, and a `moves` line at run (P6, L §4.5, §5.3).
    #[test]
    fn m2_items_windows() {
        group_holds("items-windows");
    }
}
