//! A line's canonical text: its tokens, each in its canonical spelling, spaced by L §2.4's rules.
//! Spec: L §2.4 (Expressions: "one space around binary operators and after commas, none inside
//! brackets"; Numbers: "the shortest exact decimal, … thousands grouped by a space", "a reduced
//! a/b"); L §2.1 (units "written without spaces inside them").
//! Twin: engine/tools/refcore/canon.py (`line_text`, `spaced`) and parser.py (`canon_number`,
//! the fraction's `canon` in `lit`).
//! Invariants: reads only the tokens and the marks the last parse of their line gave them;
//! every slice is checked; a fraction's parts are reduced by the number kernel, of any size.
//!
//! What the spec leaves open keeps the spacing of its examples (`f(x)`, `x[k]`, `a: T`, `?"q"`,
//! `8 PLN/hour`, `19 %`), as refcore does.

use super::expr::is_builtin;
use super::lex::{Canon, Kind, Read, Role, Tok};
use super::parse::{Stop, is_keyword};
use crate::base::{Code, Refusal};
use crate::num::Num;

/// The words a `[` after them is spaced from: a calendar's lists (`weekends [sat, sun]`).
const LIST_WORDS: [&str; 3] = ["weekends", "yearly", "added"];

/// The canonical text of `toks`, one line's tokens.
///
/// # Errors
///
/// `ENGINE-INVARIANT` when the tokens the parser marked a fraction's parts aren't two
/// integers with a denominator other than 0.
pub(super) fn line_text(toks: &[Tok]) -> Result<String, Stop> {
    let mut out = String::new();
    let mut prev: Option<&Tok> = None;
    for (at, tok) in toks.iter().enumerate() {
        if prev.is_some_and(|p| spaced(p, tok)) {
            out.push(' ');
        }
        out.push_str(&canonical(toks, at, tok)?);
        prev = Some(tok);
    }
    Ok(out)
}

/// Whether one space separates `p` and the token `t` after it (refcore's `spaced`).
fn spaced(p: &Tok, t: &Tok) -> bool {
    let (pm, tm) = (p.mark.get(), t.mark.get());
    let p_op = p.kind == Kind::Op;
    let t_op = t.kind == Kind::Op;
    if tm.role == Role::Unit {
        return false;
    }
    if ["(", "[", "{"].contains(&p.text.as_str()) {
        return false;
    }
    if [")", "]", "}", ",", ":"].contains(&t.text.as_str()) {
        return false;
    }
    if t.text == "." || p.text == "." {
        return false;
    }
    if t.text == ".." {
        return tm.role == Role::Bin || tm.role == Role::Lo;
    }
    if p.text == ".." {
        return pm.role == Role::Bin || pm.role == Role::Hi;
    }
    if p.text == "?" {
        return false;
    }
    if p.text == "-" && p_op && pm.role != Role::Bin {
        return false;
    }
    if t.text == "(" && t_op {
        // A token the parser marks a name is one whatever its word: a field after `.` is never
        // a keyword (L §2.3 rule 3), so `a.or(x)` is a call and prints touching (2.1.4-1, #84).
        let call = p.kind == Kind::Id && (is_builtin(&p.text) || pm.read == Read::Name);
        return !(p.text == "]" || call);
    }
    if t.text == "[" && t_op {
        // An id's or a field's `[` touches whatever its word (`mi.or[k1]`, an item named
        // `added`, `added[d]`; tally 2.1.6, #81); a calendar's list word's is spaced.
        let word = !is_keyword(&p.text) && !LIST_WORDS.contains(&p.text.as_str());
        let keyed = p.kind == Kind::Id && (pm.read != Read::Word || word);
        return !(p.text == "]" || p.text == ")" || keyed);
    }
    true
}

/// The canonical spelling of `tok`, the token at `at` of `toks`.
fn canonical(toks: &[Tok], at: usize, tok: &Tok) -> Result<String, Stop> {
    match tok.mark.get().canon {
        Canon::Keep => Ok(tok.text.clone()),
        Canon::Number => Ok(number_text(&tok.text).unwrap_or_else(|| tok.text.clone())),
        Canon::Numerator => {
            let denom = at.checked_add(2).and_then(|d| toks.get(d));
            Ok(reduced(Some(tok), denom)?.0)
        }
        Canon::Denominator => {
            let numer = at.checked_sub(2).and_then(|n| toks.get(n));
            Ok(reduced(numer, Some(tok))?.1)
        }
    }
}

/// A number's canonical text, or none where the token keeps its text: an integer written with a
/// leading zero is a key's spelling (L §2.1). L §2.4: "Numbers print as the shortest exact
/// decimal, with a leading 0 before the point (0.05) and thousands grouped by a space".
fn number_text(text: &str) -> Option<String> {
    let raw: String = text.chars().filter(|c| *c != ' ').collect();
    let (whole, part) = match raw.split_once('.') {
        Some((whole, part)) => {
            let whole = whole.trim_start_matches('0');
            (
                if whole.is_empty() { "0" } else { whole },
                part.trim_end_matches('0'),
            )
        }
        None if raw.len() > 1 && raw.starts_with('0') => return None,
        None => (raw.as_str(), ""),
    };
    let mut groups: Vec<&str> = Vec::new();
    let mut rest = whole;
    while rest.len() > 3 {
        let (head, tail) = rest.split_at_checked(rest.len().checked_sub(3)?)?;
        groups.push(tail);
        rest = head;
    }
    groups.push(rest);
    groups.reverse();
    let mut out = groups.join(" ");
    if !part.is_empty() {
        out.push('.');
        out.push_str(part);
    }
    Some(out)
}

/// A fraction's numerator and denominator, reduced by the number kernel's gcd, of any size
/// (L §2.4: "a reduced a/b"; refcore's `Fraction(a, b)`).
fn reduced(numer: Option<&Tok>, denom: Option<&Tok>) -> Result<(String, String), Stop> {
    let (Some(a), Some(b)) = (int_of(numer), int_of(denom)) else {
        return Err(not_a_fraction());
    };
    let value = a.checked_div(b).map_err(|_| not_a_fraction())?;
    Ok(value.fraction_parts())
}

/// An integer token's value.
fn int_of(tok: Option<&Tok>) -> Option<&Num> {
    if let Some(Tok {
        kind: Kind::Int(n), ..
    }) = tok
    {
        return Some(n);
    }
    None
}

/// The refusal of a fraction the reprint can't reduce: the parser marks only two integers
/// around a `/` whose denominator isn't 0 (`LOAD-PARSE` first).
fn not_a_fraction() -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "a fraction's parts are not two integers with a denominator other than 0".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::number_text;

    #[test]
    fn numbers_print_shortest_and_grouped() {
        assert_eq!(
            number_text("96 000.00").as_deref(),
            Some("96 000"),
            "zeros after the point go"
        );
        assert_eq!(
            number_text("1190.050").as_deref(),
            Some("1 190.05"),
            "grouped by a space"
        );
        assert_eq!(number_text(".5"), Some("0.5".to_owned()), "a leading 0");
        assert_eq!(number_text("007"), None, "a key's spelling stays");
        assert_eq!(number_text("0").as_deref(), Some("0"), "zero");
        assert_eq!(
            number_text("1234567").as_deref(),
            Some("1 234 567"),
            "two groups"
        );
    }
}
