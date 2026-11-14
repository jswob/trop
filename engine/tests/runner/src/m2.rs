//! Milestone M2's vectors (issue #167; `MILESTONE-M2.md`): realities run on refcore by
//! `m2/vectors.py`, stored in `m2/vectors.refcore` in `kernel/vectors.refcore`'s six
//! netstrings and read by [`crate::kernel::vectors`]. The milestone target
//! `milestone/m2_checks.rs` (the checks of BUILD-ORDER § M2 that S's `s_kernel` doesn't
//! hold) holds the engine's whole output to them, byte for byte; the later chunks' targets
//! (`m2_goldens`, `m2_hindcasts`, `m2_codes`, `m2_items`, `m2_fixes`) read the same store.
//!
//! - [`VECTORS`]: the stored vectors.

/// The vectors `m2/vectors.py` writes from refcore's runs (`kernel/vectors.refcore`'s form).
pub const VECTORS: &[u8] = include_bytes!("../../m2/vectors.refcore");

#[cfg(test)]
mod tests {
    use super::VECTORS;
    use crate::corpus::paths;
    use crate::kernel::vectors;
    use std::collections::BTreeSet;

    /// The runner tests' groups, `<target>-<group>`: `checks-…` for `m2_checks`.
    const TARGETS: [&str; 1] = ["checks-"];

    #[test]
    fn the_m2_vectors_read_and_their_tables_hold_every_path_they_name() {
        let vs = vectors(VECTORS).expect("m2/vectors.refcore reads");
        assert!(!vs.is_empty(), "m2/vectors.refcore holds vectors");
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert!(
                TARGETS.iter().any(|t| v.group.starts_with(t)),
                "{}: the group {} names no M2 target",
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

    /// No vector of M2's store is also one of M1's or S's: a check is held by one target.
    #[test]
    fn no_m2_vector_is_held_by_an_earlier_store() {
        let earlier: BTreeSet<String> = vectors(crate::m1::VECTORS)
            .expect("m1/vectors.refcore reads")
            .into_iter()
            .chain(vectors(crate::kernel::VECTORS).expect("kernel/vectors.refcore reads"))
            .map(|v| v.id)
            .collect();
        for v in vectors(VECTORS).expect("m2/vectors.refcore reads") {
            assert!(
                !earlier.contains(&v.id),
                "{}: held by an earlier store",
                v.id
            );
        }
    }
}
