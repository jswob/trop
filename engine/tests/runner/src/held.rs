//! The vectors T holds "for #n" (`engine/tests/held.tsv`): each a gating vector whose
//! expected bytes are refcore's and settled, which the engine doesn't yet give because the
//! work to give them is planned under issue n (tally 2.1.6: the coordinator's pick P34,
//! #97, M1's first builder group). The gating targets leave a held vector out of their
//! comparison; its stored bytes stay as they are, so deleting its row un-holds it, unchanged.
//! A store's own held files (`codes/m0/held.tsv`, `m0/format/held.tsv`) are another kind: a
//! vector whose expected bytes aren't settled, kept out of its store.

/// `held.tsv`: id, the target that would run it, the issue it waits for, why.
pub const HELD: &str = include_str!("../../held.tsv");

/// The gating targets a row may name.
pub const TARGETS: [&str; 5] = [
    "s_kernel",
    "m0_codes",
    "m0_format",
    "m0_load",
    "m0_properties",
];

/// One held vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The vector's id in its store (a case of `properties/m0.corpus` for `m0_properties`).
    pub id: String,
    /// The gating target that would run it.
    pub target: String,
    /// The issue it waits for, `#n`.
    pub issue: String,
}

/// The rows of `tsv`, in `held.tsv`'s layout.
pub fn rows(tsv: &str) -> Result<Vec<Held>, String> {
    let mut lines = tsv.lines();
    if lines.next() != Some("id\ttarget\tissue\twhy") {
        return Err("held.tsv: the header isn't id, target, issue, why".to_owned());
    }
    lines
        .map(|line| {
            let cells: Vec<&str> = line.split('\t').collect();
            let [id, target, issue, why] = cells.as_slice() else {
                return Err(format!("held.tsv: not four cells: {line}"));
            };
            if id.is_empty()
                || why.is_empty()
                || !TARGETS.contains(target)
                || !issue.starts_with('#')
            {
                return Err(format!("held.tsv: an incomplete row: {line}"));
            }
            Ok(Held {
                id: (*id).to_owned(),
                target: (*target).to_owned(),
                issue: (*issue).to_owned(),
            })
        })
        .collect()
}

/// Whether target `target` holds `id` out of its comparison.
pub fn is_held(target: &str, id: &str) -> bool {
    rows(HELD).is_ok_and(|rs| rs.iter().any(|h| h.target == target && h.id == id))
}

#[cfg(test)]
mod tests {
    use super::{HELD, rows};
    use crate::codes::{M0, m0_fixtures};
    use crate::corpus::records;
    use crate::kernel::{VECTORS, vectors};
    use crate::laws_m0::M0_CORPUS;
    use crate::m0::{FORMAT, LOAD, format_vectors, load_vectors};
    use std::collections::BTreeSet;

    /// Every row reads, names its id once, and names a vector its target's store holds, so a
    /// misspelt id can't hold nothing.
    #[test]
    fn every_held_id_is_a_vector_of_its_target() {
        let rs = rows(HELD).expect("held.tsv reads");
        let ids: BTreeSet<(&str, &str)> = rs
            .iter()
            .map(|h| (h.target.as_str(), h.id.as_str()))
            .collect();
        assert_eq!(ids.len(), rs.len(), "an id held twice");
        let kernel: BTreeSet<String> = vectors(VECTORS)
            .expect("kernel")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let codes: BTreeSet<String> = m0_fixtures(M0)
            .expect("codes")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let format: BTreeSet<String> = format_vectors(FORMAT)
            .expect("format")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let load: BTreeSet<String> = load_vectors(LOAD)
            .expect("load")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let cases: BTreeSet<String> = records(M0_CORPUS)
            .expect("corpus")
            .into_iter()
            .map(|r| r.case)
            .collect();
        for h in &rs {
            let store = match h.target.as_str() {
                "s_kernel" => &kernel,
                "m0_codes" => &codes,
                "m0_format" => &format,
                "m0_load" => &load,
                _ => &cases,
            };
            assert!(
                store.contains(&h.id),
                "{} ({}): no such vector",
                h.id,
                h.target
            );
        }
    }
}
