//! A section's parentheses in the canonical format: a pair stays only where the parse needs it,
//! and the pairs L §2.4 always prints stay or are added.
//! Spec: L §2.4 (Expressions: "parentheses only where precedence needs them, … a `... to u`
//! call used as an operand and a converted flows or value inside a comparison (parse rule 4) are
//! always parenthesised").
//! Twin: engine/tools/refcore/canon.py (`canon_block`, `reparse`, `sig`, `forced`).
//! Invariants: a pair is taken out only when the section parsed without it is the same tree,
//! parentheses aside (a blind parse, refcore's `sig`); each try parses the section once, so the
//! work is the section's pairs times its size; tokens are named by their `seq`.

use super::item::parse_block;
use super::lex::{Kind, Line, Mark, Paren, Tok};
use super::parse::{Parens, Stop};
use super::tree::Section;
use std::cell::Cell;
use std::collections::BTreeSet;

/// One pair of parentheses around an expression: its line in the section and its two tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pair {
    /// The line's position in the section.
    line: usize,
    /// The `(`'s `seq`.
    open: u32,
    /// The `)`'s `seq`.
    close: u32,
}

/// The section `block` with its parentheses made canonical, in place, and the section parsed
/// from its final tokens, its tokens marked by that parse (refcore's `canon_block`).
///
/// # Errors
///
/// A refusal the section's own parse meets.
pub(super) fn canon_block(
    block: &mut [Line],
    file: &str,
    units: &BTreeSet<String>,
) -> Result<Section, Stop> {
    parse_block(block, file, units, Parens::Kept)?;
    let pairs = plain_pairs(block);
    let want = parse_block(block, file, units, Parens::Blind)?;
    for pair in pairs {
        let Some(line) = block.get_mut(pair.line) else {
            continue;
        };
        let saved = line.toks.clone();
        line.toks
            .retain(|tok| tok.seq != pair.open && tok.seq != pair.close);
        let same = parse_block(block, file, units, Parens::Blind).is_ok_and(|tree| tree == want);
        if !same && let Some(restored) = block.get_mut(pair.line) {
            restored.toks = saved;
        }
    }
    let node = parse_block(block, file, units, Parens::Kept)?;
    if !wrap_forced(block) {
        return Ok(node);
    }
    parse_block(block, file, units, Parens::Kept)
}

/// The pairs around an expression that L §2.4 doesn't always print, in the text's order: each
/// `(` the last parse marked plain, with the `)` that closes it.
fn plain_pairs(block: &[Line]) -> Vec<Pair> {
    let mut pairs = Vec::new();
    for (at, line) in block.iter().enumerate() {
        for (k, tok) in line.toks.iter().enumerate() {
            if tok.mark.get().paren != Paren::Plain {
                continue;
            }
            if let Some(close) = closing(&line.toks, k) {
                pairs.push(Pair {
                    line: at,
                    open: tok.seq,
                    close,
                });
            }
        }
    }
    pairs
}

/// The `seq` of the bracket that closes the one at `open`.
fn closing(toks: &[Tok], open: usize) -> Option<u32> {
    let mut depth = 0_u32;
    for tok in toks.iter().skip(open) {
        if tok.kind != Kind::Op {
            continue;
        }
        match tok.text.as_str() {
            "(" | "[" | "{" => depth = depth.checked_add(1)?,
            ")" | "]" | "}" => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(tok.seq);
                }
            }
            _ => {}
        }
    }
    None
}

/// Puts parentheses around each operand the last parse marked as always parenthesised and
/// written bare; whether any was added.
fn wrap_forced(block: &mut [Line]) -> bool {
    let mut added = false;
    for line in block.iter_mut() {
        if !line
            .toks
            .iter()
            .any(|tok| tok.mark.get().wrap_open || tok.mark.get().wrap_close)
        {
            continue;
        }
        let mut toks = Vec::new();
        for tok in core::mem::take(&mut line.toks) {
            let mark = tok.mark.get();
            if mark.wrap_open {
                toks.push(bracket("("));
            }
            toks.push(tok);
            if mark.wrap_close {
                toks.push(bracket(")"));
            }
        }
        line.toks = toks;
        added = true;
    }
    added
}

/// A bracket the reprint adds: a `(` with a space before it, a `)` with none, as refcore's.
fn bracket(text: &str) -> Tok {
    Tok {
        kind: Kind::Op,
        text: text.to_owned(),
        ws: text == "(",
        seq: u32::MAX,
        mark: Cell::new(Mark::default()),
    }
}
