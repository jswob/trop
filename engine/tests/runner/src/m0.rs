//! Milestone M0's vectors (issue #56; `MILESTONE-M0.md`), written by `m0/vectors.py` from
//! refcore 2.1.3's runs over every text set `checks.tsv` names.
//!
//! The format (row 4): refcore's `reprint` of each set and of the format fixtures
//! (`m0/format/`), stored in `m0/format.refcore`, each file's canonical text held by its
//! length and its digest (two residues modulo 61-bit primes). The milestone target
//! `milestone/m0_format.rs` holds the engine's `reprint` to them, and its reprint of its own
//! canonical text to that text
//! (DONE gate 6: `format(parse(x)) = x` on L §2.4's canonical form, and `format` idempotent).
//!
//! The load (rows 1 and 2): refcore's whole run of the load probe, `run <files> --ask
//! zz_m0_probe` (a load refusal, or `refused: REQ-ASK no ask zz_m0_probe` alone after a clean
//! load, E §11), of a refused scenario's own asks, and of row 1's 10 checks, stored in
//! `m0/load.refcore`. The milestone target `milestone/m0_load.rs` holds the engine's run to
//! each, its exit code and its output with `warning:` lines left out on both sides (the
//! warnings are M1's, `BUILD-ORDER.md` § M1).
//!
//! - [`FORMAT`] and [`LOAD`]: the stored vectors, read by [`format_vectors`] and
//!   [`load_vectors`];
//! - [`digest`]: the texts' digest, as `vectors.py` computes it;
//! - [`printed`]: a reprint's output as its sections and the rest;
//! - [`format_failure`] and [`load_failure`]: how the engine's run of a vector fails, if it
//!   does;
//! - [`group_failures`] and [`load_group_failures`]: every failure of one group, for the
//!   milestone targets.

use crate::corpus::{Run, groups, netstrings, pack, text};
use crate::engine::run_args;
use crate::kernel::difference;
use crate::m0_texts::TEXTS;
use std::collections::BTreeMap;

/// `m0/format.refcore`: eight netstrings per vector (`m0/vectors.py`).
pub const FORMAT: &[u8] = include_bytes!("../../m0/format.refcore");

/// `m0/load.refcore`: seven netstrings per vector (`m0/vectors.py`).
pub const LOAD: &[u8] = include_bytes!("../../m0/load.refcore");

/// The ask the load probe asks, which no text declares.
pub const PROBE: &str = "zz_m0_probe";

/// The digest's two lanes, each a prime modulus and a base (`vectors.py`'s `LANES`): the
/// primes 2^61 − 1 and 2^61 − 31.
const LANES: [(u128, u128); 2] = [
    (0x1fff_ffff_ffff_ffff, 1_099_511_628_211),
    (0x1fff_ffff_ffff_ffe1, 1_540_483_477),
];

/// A text's digest: its polynomial residue in each lane.
pub type Digest = [u128; 2];

/// One file a vector's run may read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableFile {
    /// Its path as the run names it (`INTERFACE.md` §4).
    pub key: String,
    /// Its path under `engine/tests/`, a key of [`TEXTS`].
    pub path: String,
}

/// One section of refcore's reprint: a file and its canonical text, by length and digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The file's path, as the run names it.
    pub name: String,
    /// The canonical text's length in bytes.
    pub bytes: usize,
    /// The canonical text's [`digest`].
    pub digest: Digest,
}

/// One format vector: a `reprint` command line, its files and refcore's output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatVector {
    /// The set's id: the first id of the checks that run it, or `format/<fixture>`.
    pub id: String,
    /// The group its test runs it in (`probes`, `models`, `household-B`, `format/numbers`, …).
    pub group: String,
    /// The command line's words after the program name: `reprint` and the files.
    pub args: Vec<String>,
    /// Every file the run may read: those it names, and a joint model's members.
    pub table: Vec<TableFile>,
    /// refcore's exit code.
    pub code: u8,
    /// refcore's sections, in its order.
    pub sections: Vec<Section>,
    /// The rest of refcore's output: the `sha256` line.
    pub tail: String,
}

/// One load vector: a `run` command line, its files and refcore's whole run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadVector {
    /// The set's id, `<set>#<scenario>` for a refused scenario's asks, or `refusal/<check>`
    /// for row 1's checks.
    pub id: String,
    /// The group its test runs it in (`probes`, `household-B`, `refusal/U:rule/RULE-04`, …).
    pub group: String,
    /// The command line's words after the program name: `run`, the files, `--ask` and asks.
    pub args: Vec<String>,
    /// Every file the run may read: those it names, and a joint model's members.
    pub table: Vec<TableFile>,
    /// refcore's run: its whole output and its exit code.
    pub run: Run,
}

/// A reprint's output, split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printed {
    /// Each section: the file's path as the run names it, and its canonical text, every
    /// line ending in a newline.
    pub sections: Vec<(String, String)>,
    /// The lines after the last section (the `sha256` line), or the whole output when it
    /// opens no section (a refusal).
    pub tail: String,
}

/// The digest of `bytes`: in each lane, `h = (h * base + byte) mod p` from `h = 0`, with
/// checked arithmetic (R-23: no wrapping); `h < 2^61` and `base < 2^41`, so no step can
/// overflow, and `None` would only say the lanes were changed wrongly.
pub fn digest(bytes: &[u8]) -> Option<Digest> {
    let mut out = [0_u128; 2];
    for (h, (p, base)) in out.iter_mut().zip(LANES) {
        for b in bytes {
            *h = h
                .checked_mul(base)?
                .checked_add(u128::from(*b))?
                .checked_rem(p)?;
        }
    }
    Some(out)
}

/// A digest written as `vectors.py` writes it: 2 × 16 lower-case hex digits.
fn digest_of_hex(hex: &str) -> Option<Digest> {
    let first = u128::from_str_radix(hex.get(..16)?, 16).ok()?;
    let second = u128::from_str_radix(hex.get(16..)?, 16).ok()?;
    (hex.len() == 32).then_some([first, second])
}

/// The vectors of `bytes`, in `vectors.py`'s order (by id).
pub fn format_vectors(bytes: &[u8]) -> Result<Vec<FormatVector>, String> {
    let mut out = Vec::new();
    for eight in groups(&netstrings(bytes)?, 8)? {
        let [id, group, _cwd, args, table, code, sections, tail] = eight.as_slice() else {
            return Err("a format vector is eight netstrings".to_owned());
        };
        let id = text(id)?;
        out.push(FormatVector {
            group: text(group)?,
            args: text(args)?.split(' ').map(str::to_owned).collect(),
            table: table_files(&text(table)?, &id)?,
            code: text(code)?
                .parse()
                .map_err(|_| format!("{id}: an exit code"))?,
            sections: sections_of(&text(sections)?, &id)?,
            tail: text(tail)?,
            id,
        });
    }
    Ok(out)
}

/// The load vectors of `bytes`, in `vectors.py`'s order.
pub fn load_vectors(bytes: &[u8]) -> Result<Vec<LoadVector>, String> {
    let mut out = Vec::new();
    for seven in groups(&netstrings(bytes)?, 7)? {
        let [id, group, _cwd, args, table, code, whole] = seven.as_slice() else {
            return Err("a load vector is seven netstrings".to_owned());
        };
        let id = text(id)?;
        out.push(LoadVector {
            group: text(group)?,
            args: text(args)?.split(' ').map(str::to_owned).collect(),
            table: table_files(&text(table)?, &id)?,
            run: Run {
                out: text(whole)?,
                code: text(code)?
                    .parse()
                    .map_err(|_| format!("{id}: an exit code"))?,
            },
            id,
        });
    }
    Ok(out)
}

/// A vector's table: one line per file, its key and its path, tab-separated.
fn table_files(lines: &str, id: &str) -> Result<Vec<TableFile>, String> {
    lines
        .lines()
        .map(|line| match line.split_once('\t') {
            Some((key, path)) => Ok(TableFile {
                key: key.to_owned(),
                path: path.to_owned(),
            }),
            None => Err(format!("{id}: a table line without its tab")),
        })
        .collect()
}

/// A vector's sections: one line per section, its name, length and digest, tab-separated.
fn sections_of(lines: &str, id: &str) -> Result<Vec<Section>, String> {
    lines
        .lines()
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            let [name, bytes, hex] = cols.as_slice() else {
                return Err(format!("{id}: a section line is three columns"));
            };
            Ok(Section {
                name: (*name).to_owned(),
                bytes: bytes
                    .parse()
                    .map_err(|_| format!("{id}: a section's length"))?,
                digest: digest_of_hex(hex).ok_or_else(|| format!("{id}: a section's digest"))?,
            })
        })
        .collect()
}

/// `out`, a reprint's output, as its sections and the rest (`vectors.py`'s `sections`): a
/// line `--- <path>` opens a section, which runs to the next such line, or to a last line
/// `sha256 …`; an output that opens no section is all rest.
pub fn printed(out: &str) -> Printed {
    let mut lines: Vec<&str> = out.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let count = lines.len();
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut tail = String::new();
    let mut in_tail = false;
    for (at, line) in lines.iter().enumerate() {
        let hash_line = at.checked_add(1) == Some(count) && line.starts_with("sha256 ");
        let opens = line.strip_prefix("--- ").filter(|_| !in_tail);
        if let Some(name) = opens {
            sections.push((name.to_owned(), String::new()));
            continue;
        }
        let open = sections.last_mut().filter(|_| !(in_tail || hash_line));
        if let Some((_, content)) = open {
            content.push_str(line);
            content.push('\n');
            continue;
        }
        in_tail = true;
        tail.push_str(line);
        tail.push('\n');
    }
    Printed { sections, tail }
}

/// Every text of [`TEXTS`], by its path under `engine/tests/`.
pub fn texts() -> BTreeMap<&'static str, &'static str> {
    TEXTS.iter().copied().collect()
}

/// The file table of vector `id` (`INTERFACE.md` §4), its `files` read from `texts`.
pub fn table(
    id: &str,
    files: &[TableFile],
    texts: &BTreeMap<&str, &str>,
) -> Result<Vec<u8>, String> {
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    for f in files {
        let content = texts
            .get(f.path.as_str())
            .ok_or_else(|| format!("{id}: {} isn't in m0_texts.rs", f.path))?;
        pairs.push((f.key.as_str(), content));
    }
    Ok(pack(&pairs))
}

/// How the engine's `reprint` of `v` fails, or `None`: it must print refcore's output (each
/// canonical text by its length and digest, the `sha256` line verbatim, the exit code), and
/// reprinting each canonical text in place of its file must print the same output again,
/// byte for byte (the round trip).
pub fn format_failure(v: &FormatVector, texts: &BTreeMap<&str, &str>) -> Option<String> {
    let first_table = match table(&v.id, &v.table, texts) {
        Ok(t) => t,
        Err(e) => return Some(e),
    };
    let first = run_args(&v.args, &first_table);
    let got = printed(&first.out);
    if let Some(why) = against_refcore(v, &first, &got) {
        return Some(format!("{}: {why}", v.id));
    }
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    for f in &v.table {
        let canonical = got
            .sections
            .iter()
            .find(|(name, _)| *name == f.key)
            .map(|(_, content)| content.as_str());
        let Some(content) = canonical.or_else(|| texts.get(f.path.as_str()).copied()) else {
            return Some(format!("{}: {} has no text", v.id, f.path));
        };
        pairs.push((f.key.as_str(), content));
    }
    let again = run_args(&v.args, &pack(&pairs));
    difference(&first, &again).map(|d| {
        format!(
            "{}: the round trip: reprinting the canonical text prints another output: {d}",
            v.id
        )
    })
}

/// How the engine's run `run`, split as `got`, differs from refcore's stored output.
fn against_refcore(v: &FormatVector, run: &Run, got: &Printed) -> Option<String> {
    let opening: String = run.out.chars().take_while(|c| *c != '\n').collect();
    if run.code != v.code {
        return Some(format!(
            "exit {} (refcore {}), first line {opening:?}",
            run.code, v.code
        ));
    }
    if got.sections.len() != v.sections.len() {
        return Some(format!(
            "{} sections (refcore {}), first line {opening:?}",
            got.sections.len(),
            v.sections.len()
        ));
    }
    for ((name, content), want) in got.sections.iter().zip(&v.sections) {
        let bytes = content.len();
        if *name != want.name
            || bytes != want.bytes
            || digest(content.as_bytes()) != Some(want.digest)
        {
            return Some(format!(
                "section {name:?}, {bytes} bytes, differs from refcore's {:?}, {} bytes \
                 (`m0/vectors.py --show '{}'` prints refcore's)",
                want.name, want.bytes, v.id
            ));
        }
    }
    (got.tail != v.tail).then(|| format!("last lines {:?} (refcore {:?})", got.tail, v.tail))
}

/// Every vector of `group` the engine fails, each with why, and how many vectors the group
/// holds.
pub fn group_failures(group: &str) -> Result<(Vec<String>, usize), String> {
    let vs = format_vectors(FORMAT)?;
    let texts = texts();
    let mine: Vec<&FormatVector> = vs.iter().filter(|v| v.group == group).collect();
    if mine.is_empty() {
        return Err(format!("m0/format.refcore holds no vector of {group}"));
    }
    let failed = mine
        .iter()
        .filter_map(|v| format_failure(v, &texts))
        .collect();
    Ok((failed, mine.len()))
}

/// `out` without its `warning: …` lines: the warnings are M1's (`BUILD-ORDER.md` § M1, L §8.5),
/// as the household compare leaves them out (`README.md` § Compare).
fn unwarned(out: &str) -> String {
    out.split_inclusive('\n')
        .filter(|line| !line.starts_with("warning: "))
        .collect()
}

/// How the engine's run of `v` fails, or `None`: it must exit with refcore's code and print
/// refcore's whole output, `warning:` lines left out on both sides.
pub fn load_failure(v: &LoadVector, texts: &BTreeMap<&str, &str>) -> Option<String> {
    let files = match table(&v.id, &v.table, texts) {
        Ok(t) => t,
        Err(e) => return Some(e),
    };
    let got = run_args(&v.args, &files);
    let want = Run {
        out: unwarned(&v.run.out),
        code: v.run.code,
    };
    let mine = Run {
        out: unwarned(&got.out),
        code: got.code,
    };
    difference(&want, &mine).map(|d| format!("{}: {d}", v.id))
}

/// Every load vector of `group` the engine fails, each with why, and how many vectors the
/// group holds.
pub fn load_group_failures(group: &str) -> Result<(Vec<String>, usize), String> {
    let vs = load_vectors(LOAD)?;
    let texts = texts();
    let mine: Vec<&LoadVector> = vs.iter().filter(|v| v.group == group).collect();
    if mine.is_empty() {
        return Err(format!("m0/load.refcore holds no vector of {group}"));
    }
    let failed = mine
        .iter()
        .filter_map(|v| load_failure(v, &texts))
        .collect();
    Ok((failed, mine.len()))
}

#[cfg(test)]
mod tests {
    use super::{
        FORMAT, LOAD, LoadVector, PROBE, TEXTS, digest, digest_of_hex, format_vectors,
        load_vectors, printed, texts, unwarned,
    };
    use std::collections::BTreeSet;

    /// The corpus groups `vectors.py` sorts the 315 text sets into, and how many sets each
    /// holds; with the format fixtures, 334 vectors.
    const GROUPS: [(&str, usize); 12] = [
        ("blind", 18),
        ("household-A", 8),
        ("household-B", 9),
        ("household-C", 12),
        ("household-D", 3),
        ("household-E1", 2),
        ("household-E2", 2),
        ("household-X", 28),
        ("light", 3),
        ("models", 77),
        ("probe-sets", 11),
        ("probes", 142),
    ];

    /// The values `vectors.py`'s `digest` prints for three texts.
    #[test]
    fn the_digest_is_vectors_pys() {
        for (text, hex) in [
            ("", "00000000000000000000000000000000"),
            ("a", "00000000000000610000000000000061"),
            ("model a\n", "00e443b5d44239f514c67f899ee4585e"),
        ] {
            assert_eq!(
                digest(text.as_bytes()),
                digest_of_hex(hex),
                "the digest of {text:?}"
            );
        }
        assert_eq!(digest_of_hex("00"), None, "a digest is 32 hex digits");
    }

    #[test]
    fn the_format_vectors_read_and_name_their_texts() {
        let vs = format_vectors(FORMAT).expect("m0/format.refcore reads");
        assert_eq!(vs.len(), 334, "315 text sets and 19 format fixtures");
        let all = texts();
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert_eq!(
                v.args.first().map(String::as_str),
                Some("reprint"),
                "{}",
                v.id
            );
            assert_eq!(v.code, 0, "{}: refcore reprints every set", v.id);
            assert!(!v.sections.is_empty(), "{}: no section", v.id);
            assert!(v.tail.starts_with("sha256 "), "{}: {}", v.id, v.tail);
            for f in &v.table {
                assert!(all.contains_key(f.path.as_str()), "{}: {}", v.id, f.path);
            }
            for name in v.args.iter().skip(1) {
                assert!(
                    v.table.iter().any(|f| f.key == *name),
                    "{}: {name} isn't in its table",
                    v.id
                );
            }
        }
        for (group, count) in GROUPS {
            assert_eq!(
                vs.iter().filter(|v| v.group == group).count(),
                count,
                "{group}"
            );
        }
        assert_eq!(
            vs.iter().filter(|v| v.group.starts_with("format/")).count(),
            19,
            "one vector per format fixture"
        );
    }

    /// The texts are sorted and unique, so a path has one text.
    #[test]
    fn the_texts_are_sorted_once() {
        assert!(
            TEXTS
                .iter()
                .zip(TEXTS.iter().skip(1))
                .all(|(a, b)| a.0 < b.0),
            "m0_texts.rs is sorted by path, each once"
        );
    }

    /// `vectors.py`'s digest and this one agree: a section whose file is already canonical
    /// holds its source's digest, and hundreds do.
    #[test]
    fn the_digests_agree_with_vectors_py() {
        let vs = format_vectors(FORMAT).expect("m0/format.refcore reads");
        let all = texts();
        let mut agree = 0_usize;
        for v in &vs {
            for s in &v.sections {
                let path = v
                    .table
                    .iter()
                    .find(|f| f.key == s.name)
                    .map(|f| f.path.as_str());
                let source = path
                    .and_then(|p| all.get(p))
                    .expect("a section names a file");
                if source.len() == s.bytes && digest(source.as_bytes()) == Some(s.digest) {
                    agree += 1;
                }
            }
        }
        assert!(
            agree > 500,
            "only {agree} canonical sources hold their digest"
        );
    }

    /// 315 probes (one per text set, in the corpus groups), 2 refused scenarios' asks and row
    /// 1's 10 checks; every output a refusal; 304 probes print the REQ-ASK line alone, the
    /// other 11 a load refusal (MP-10's `LOAD-BAND` and row 1's 10).
    #[test]
    fn the_load_vectors_read_and_name_their_texts() {
        let vs = load_vectors(LOAD).expect("m0/load.refcore reads");
        assert_eq!(vs.len(), 327, "315 probes, 2 scenarios' asks, 10 refusals");
        let all = texts();
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert_eq!(v.args.first().map(String::as_str), Some("run"), "{}", v.id);
            for f in &v.table {
                assert!(all.contains_key(f.path.as_str()), "{}: {}", v.id, f.path);
            }
            assert!(
                v.run.out.starts_with("refused: ") || v.run.out.starts_with("scenario "),
                "{}: {}",
                v.id,
                v.run.out
            );
        }
        let probes: Vec<&LoadVector> = vs
            .iter()
            .filter(|v| !v.id.contains('#') && !v.id.starts_with("refusal/"))
            .collect();
        assert_eq!(probes.len(), 315, "a probe per text set");
        let probed = format!("refused: REQ-ASK no ask {PROBE}\n");
        for v in &probes {
            assert_eq!(v.args.last().map(String::as_str), Some(PROBE), "{}", v.id);
            assert_eq!(v.run.code, 1, "{}: the probe is refused", v.id);
        }
        assert_eq!(
            probes.iter().filter(|v| v.run.out == probed).count(),
            304,
            "the probe's REQ-ASK line alone after a clean load"
        );
        for (group, count) in GROUPS {
            assert_eq!(
                probes.iter().filter(|v| v.group == group).count(),
                count,
                "{group}"
            );
        }
        let refusals: Vec<&LoadVector> = vs
            .iter()
            .filter(|v| v.group.starts_with("refusal/"))
            .collect();
        assert_eq!(refusals.len(), 10, "row 1's checks");
        for v in refusals {
            assert_eq!(v.run.code, 1, "{}", v.id);
            assert!(v.run.out.starts_with("refused: LOAD-"), "{}", v.id);
        }
    }

    #[test]
    fn warnings_are_left_out() {
        assert_eq!(
            unwarned(
                "scenario s: refused: X\nwarning: WARN-UNREAD input a is read by nothing\nb\n"
            ),
            "scenario s: refused: X\nb\n",
            "the warning line goes, the others stay"
        );
        assert_eq!(
            unwarned("a\nwarning: w"),
            "a\n",
            "a last line without its newline"
        );
    }

    #[test]
    fn a_reprint_splits_into_sections_and_the_rest() {
        let p = printed("--- a.tally\nmodel a\n  tally 2\n--- b/c.tally\npart c\nsha256 00\n");
        assert_eq!(
            p.sections,
            vec![
                ("a.tally".to_owned(), "model a\n  tally 2\n".to_owned()),
                ("b/c.tally".to_owned(), "part c\n".to_owned()),
            ],
            "two sections"
        );
        assert_eq!(p.tail, "sha256 00\n", "the hash line");
        let r = printed("refused: LOAD-PARSE cannot parse\n");
        assert!(r.sections.is_empty(), "a refusal opens no section");
        assert_eq!(
            r.tail, "refused: LOAD-PARSE cannot parse\n",
            "it is all rest"
        );
    }
}
