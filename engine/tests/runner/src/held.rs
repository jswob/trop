//! The vectors T holds "for #n" (`engine/tests/held.tsv`): each a gating vector whose
//! expected bytes are refcore's and settled, which the engine doesn't yet give because the
//! work to give them is planned under issue n (tally 2.1.6: the coordinator's pick P34,
//! #97, M1's first builder group). The gating targets leave a held vector out of their
//! comparison; its stored bytes stay as they are, so deleting its row un-holds it, unchanged.
//! A store's own held files (`codes/m0/held.tsv`, `m0/format/held.tsv`) are another kind: a
//! vector whose expected bytes aren't settled, kept out of its store.

/// `held.tsv`: id, the target that would run it, the issue it waits for, why.
pub const HELD: &str = include_str!("../../held.tsv");

/// The targets a row may name: the gating ones, M1's vector stores (issue #108), whose
/// targets gate from M1's marker, M2's (issue #167), whose targets gate from M2's, M3's
/// (issue #226), whose targets gate from M3's, and M4's (issue #252), whose targets gate from
/// M4's.
pub const TARGETS: [&str; 26] = [
    "s_kernel",
    "m0_codes",
    "m0_format",
    "m0_load",
    "m0_properties",
    "m1_checks",
    "m1_carried",
    "m1_goldens",
    "m1_codes",
    "m1_fixes",
    "m1_properties",
    "m2_checks",
    "m2_items",
    "m2_fixes",
    "m2_goldens",
    "m2_codes",
    "m2_hindcasts",
    "m2_properties",
    "m3_check",
    "m3_changes",
    "m3_run",
    "m3_fixes",
    "m4_checks",
    "m4_constructs",
    "m4_items",
    "m4_fixes",
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
    use crate::codes::{M0, M1 as M1_CODES, M2 as M2_CODES, m0_fixtures};
    use crate::corpus::records;
    use crate::kernel::{VECTORS, vectors};
    use crate::laws_m0::M0_CORPUS;
    use crate::laws_m1::M1_CORPUS;
    use crate::laws_m2::M2_CORPUS;
    use crate::m0::{FORMAT, LOAD, format_vectors, load_vectors};
    use crate::m1::VECTORS as M1_VECTORS;
    use crate::m2::VECTORS as M2_VECTORS;
    use crate::m3::{CHECK, RUNS, SPEC, check_vectors};
    use crate::m4::{SPEC as M4_SPEC, VECTORS as M4_VECTORS};
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
        let m2_cases: BTreeSet<String> = records(M2_CORPUS)
            .expect("m2 corpus")
            .into_iter()
            .map(|r| r.case)
            .collect();
        let m1_cases: BTreeSet<String> = records(M1_CORPUS)
            .expect("m1 corpus")
            .into_iter()
            .map(|r| r.case)
            .collect();
        let m1: BTreeSet<String> = vectors(M1_VECTORS)
            .expect("m1")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let m1_codes: BTreeSet<String> = m0_fixtures(M1_CODES)
            .expect("m1 codes")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let m2_codes: BTreeSet<String> = m0_fixtures(M2_CODES)
            .expect("m2 codes")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let m2: BTreeSet<String> = vectors(M2_VECTORS)
            .expect("m2")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let m3_check: BTreeSet<String> = check_vectors(CHECK)
            .expect("m3 check")
            .into_iter()
            .map(|v| v.id)
            .collect();
        let m3_runs: BTreeSet<String> = check_vectors(RUNS)
            .expect("m3 runs")
            .into_iter()
            .chain(check_vectors(SPEC).expect("m3 spec"))
            .map(|v| v.id)
            .collect();
        // M4's spec answers are held too where a later wording changes them (P26), as M3's are
        let m4: BTreeSet<String> = vectors(M4_VECTORS)
            .expect("m4")
            .into_iter()
            .chain(vectors(M4_SPEC).expect("m4 spec"))
            .map(|v| v.id)
            .collect();
        for h in &rs {
            let store = match h.target.as_str() {
                "s_kernel" => &kernel,
                "m0_codes" => &codes,
                "m0_format" => &format,
                "m0_load" => &load,
                "m1_checks" | "m1_carried" | "m1_goldens" | "m1_fixes" => &m1,
                "m1_codes" => &m1_codes,
                "m1_properties" => &m1_cases,
                "m2_properties" => &m2_cases,
                "m2_checks" | "m2_items" | "m2_fixes" | "m2_goldens" | "m2_hindcasts" => &m2,
                "m2_codes" => &m2_codes,
                "m3_check" => &m3_check,
                "m3_changes" | "m3_run" | "m3_fixes" => &m3_runs,
                "m4_checks" | "m4_constructs" | "m4_items" | "m4_fixes" => &m4,
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
