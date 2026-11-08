//! Milestone M1's vectors (issue #108; `MILESTONE-M1.md`): realities run on refcore by
//! `m1/vectors.py`, stored in `m1/vectors.refcore` in `kernel/vectors.refcore`'s six
//! netstrings and read by [`crate::kernel::vectors`]. The milestone targets
//! `milestone/m1_checks.rs` (the checks of BUILD-ORDER § M1 that S's `s_kernel` doesn't
//! hold) and `milestone/m1_carried.rs` (the items the pre-M1 run left for M1) hold the
//! engine's whole output to them, byte for byte; `milestone/m1_format.rs` holds #109's law.
//!
//! - [`VECTORS`]: the stored vectors;
//! - [`KEY_KINDS`]: #109's text, rows keyed by literals of different kinds;
//! - [`fixed_point`]: the law it is held to, a canonical text reprints as itself.

use crate::corpus::{Run, pack};
use crate::engine::run_args;
use crate::laws_m0::again_of;

/// The vectors `m1/vectors.py` writes from refcore's runs (`kernel/vectors.refcore`'s form).
pub const VECTORS: &[u8] = include_bytes!("../../m1/vectors.refcore");

/// #109, minimised from the format fuzz target's find (`tmp/m1-rules/crash-1fc61e46…`): a
/// part whose input keyed by days has a row keyed by the integer `20240` beside day keys and
/// a day range. The engine and refcore print it alike, and each moves that row when it
/// reprints its own output (refcore's half: #111). The coordinator's pick P3 orders keys of
/// different literal kinds by kind first, then by value (L §2.4 at M2's round).
pub const KEY_KINDS: (&str, &str) = (
    "key_kinds.tally",
    include_str!("../../m1/format/key_kinds.tally"),
);

/// The law a canonical text is held to (L §2 "one canonical form"; `PROPERTIES.md` § M0,
/// `m0_format_idempotent`): `path`'s text, reprinted, then its reprint reprinted, prints the
/// same bytes both times, or the first reprint is refused. The expected value is the law,
/// never an output: refcore breaks it on #109's text too (#111). `run` is the engine (or a
/// planted fault in this module's tests).
pub fn fixed_point(path: &str, text: &str, run: fn(&[String], &[u8]) -> Run) -> Result<(), String> {
    let first = run(
        &["reprint".to_owned(), path.to_owned()],
        &pack(&[(path, text)]),
    );
    let Some((args, files)) = again_of(&first) else {
        return Ok(());
    };
    let second = run(&args, &files);
    if second == first {
        return Ok(());
    }
    Err(format!(
        "{path}: its canonical text reprints otherwise:\n--- the first reprint\n{}--- its reprint\n{}",
        first.out, second.out
    ))
}

/// [`fixed_point`] on the engine.
pub fn engine_fixed_point(path: &str, text: &str) -> Result<(), String> {
    fixed_point(path, text, run_args)
}

#[cfg(test)]
mod tests {
    use super::{KEY_KINDS, VECTORS, fixed_point};
    use crate::corpus::{Run, paths};
    use crate::kernel::vectors;
    use std::collections::BTreeSet;

    /// The runner tests' groups, `<target>-<group>`: `checks-…` for `m1_checks`, `carried-…`
    /// for `m1_carried`.
    const TARGETS: [&str; 2] = ["checks-", "carried-"];

    #[test]
    fn the_m1_vectors_read_and_their_tables_hold_every_path_they_name() {
        let vs = vectors(VECTORS).expect("m1/vectors.refcore reads");
        assert!(!vs.is_empty(), "m1/vectors.refcore holds vectors");
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert!(
                TARGETS.iter().any(|t| v.group.starts_with(t)),
                "{}: the group {} names no M1 target",
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

    /// The one file a stand-in printer is given: its path and its text.
    fn single(args: &[String], files: &[u8]) -> (String, String) {
        let parts = crate::corpus::netstrings(files).expect("a table");
        let path = args.get(1).cloned().expect("a path");
        let text = parts
            .get(1)
            .map(|t| crate::corpus::text(t).expect("a UTF-8 text"))
            .expect("a text");
        (path, text)
    }

    /// A stand-in printer that is a fixed point: it prints its one file under `--- `, as
    /// `reprint` does, with a constant hash line.
    fn stable(args: &[String], files: &[u8]) -> Run {
        let (path, text) = single(args, files);
        Run {
            out: format!("--- {path}\n{text}sha256 0\n"),
            code: 0,
        }
    }

    /// A planted fault: a printer that moves the last line of its file to the top each time.
    fn moving(args: &[String], files: &[u8]) -> Run {
        let (path, text) = single(args, files);
        let lines: Vec<&str> = text.lines().collect();
        let lines: Vec<&str> = match lines.split_last() {
            Some((last, rest)) => std::iter::once(*last).chain(rest.iter().copied()).collect(),
            None => Vec::new(),
        };
        Run {
            out: format!("--- {path}\n{}\nsha256 0\n", lines.join("\n")),
            code: 0,
        }
    }

    /// A refused reprint passes: the law holds a printed text only.
    fn refusing(_: &[String], _: &[u8]) -> Run {
        Run {
            out: "refused: LOAD-PARSE cannot parse\n".to_owned(),
            code: 1,
        }
    }

    #[test]
    fn the_fixed_point_law_holds_a_stable_printer_and_fails_a_moving_one() {
        let (path, text) = KEY_KINDS;
        assert_eq!(
            fixed_point(path, text, stable),
            Ok(()),
            "a printer that prints its text as given holds the law"
        );
        assert_eq!(
            fixed_point(path, text, refusing),
            Ok(()),
            "a refused reprint holds the law"
        );
        assert!(
            fixed_point(path, text, moving).is_err(),
            "a printer that moves a line fails the law"
        );
    }
}
