//! Milestone M4's vectors (issue #252; `MILESTONE-M4.md`): realities run on refcore by
//! `m4/vectors.py`, stored in `m4/vectors.refcore` in `kernel/vectors.refcore`'s six
//! netstrings and read by [`crate::kernel::vectors`]. The milestone targets
//! `milestone/m4_checks.rs` (BUILD-ORDER § M4's 77 checks) and `milestone/m4_constructs.rs`
//! (the corners of M4's constructs) hold the engine's whole output to them, byte for byte.
//!
//! - [`VECTORS`]: the stored vectors.

/// The vectors `m4/vectors.py` writes from refcore's runs (`kernel/vectors.refcore`'s form).
pub const VECTORS: &[u8] = include_bytes!("../../m4/vectors.refcore");

#[cfg(test)]
mod tests {
    use super::VECTORS;
    use crate::corpus::paths;
    use crate::kernel::vectors;
    use std::collections::BTreeSet;

    /// The runner tests' groups, `<target>-<group>`: `checks-…` for `m4_checks`,
    /// `constructs-…` for `m4_constructs`.
    const TARGETS: [&str; 2] = ["checks-", "constructs-"];

    #[test]
    fn the_m4_vectors_read_and_their_tables_hold_every_path_they_name() {
        let vs = vectors(VECTORS).expect("m4/vectors.refcore reads");
        assert!(!vs.is_empty(), "m4/vectors.refcore holds vectors");
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert!(
                TARGETS.iter().any(|t| v.group.starts_with(t)),
                "{}: the group {} names no M4 target",
                v.id,
                v.group
            );
            assert_eq!(v.args.first().map(String::as_str), Some("run"), "{}", v.id);
            let table = paths(&v.files).expect("the file table reads");
            let named = v
                .args
                .iter()
                .skip(1)
                .take_while(|a| a.as_str() != "--ask")
                .filter(|a| !a.starts_with("--"));
            for path in named {
                assert!(table.contains(path), "{}: {path} isn't in its table", v.id);
            }
        }
    }

    /// BUILD-ORDER § M4 lists 77 checks: 73 are vectors of this store, under their own ids,
    /// and the other 4 have gated since S as kernel vectors (`kernel/vectors.tsv`).
    #[test]
    fn the_m4_store_and_s_hold_build_orders_77_checks() {
        let checks = vectors(VECTORS)
            .expect("m4/vectors.refcore reads")
            .into_iter()
            .filter(|v| v.group.starts_with("checks-"))
            .count();
        assert_eq!(checks, 73, "BUILD-ORDER § M4's Green (77), less S's 4");
        let kernel: BTreeSet<String> = vectors(crate::kernel::VECTORS)
            .expect("kernel/vectors.refcore reads")
            .into_iter()
            .map(|v| v.id)
            .collect();
        for id in ["U:cur/CUR-01", "U:cur/CUR-02", "U:law/LAW-12", "I:M:RC-07"] {
            assert!(kernel.contains(id), "{id}: a kernel vector since S");
        }
    }

    /// No vector of M4's store is also one of S's, M1's or M2's: a check is held by one target.
    #[test]
    fn no_m4_vector_is_held_by_an_earlier_store() {
        let earlier: BTreeSet<String> = vectors(crate::m2::VECTORS)
            .expect("m2/vectors.refcore reads")
            .into_iter()
            .chain(vectors(crate::m1::VECTORS).expect("m1/vectors.refcore reads"))
            .chain(vectors(crate::kernel::VECTORS).expect("kernel/vectors.refcore reads"))
            .map(|v| v.id)
            .collect();
        for v in vectors(VECTORS).expect("m4/vectors.refcore reads") {
            assert!(
                !earlier.contains(&v.id),
                "{}: held by an earlier store",
                v.id
            );
        }
    }
}
