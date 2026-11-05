//! A line's canonical text: its tokens, each in its canonical spelling, spaced by L §2.4's rules.
//! Spec: L §2.4 (Expressions: "one space around binary operators and after commas, none inside
//! brackets"; Numbers: "the shortest exact decimal, … thousands grouped by a space", "a reduced
//! a/b"); L §2.1 (units "written without spaces inside them").
//! Twin: engine/tools/refcore/canon.py (`line_text`, `spaced`) and parser.py (`canon_number`,
//! the fraction's `canon` in `lit`).
//! Invariants: reads only the tokens and the marks the last parse of their line gave them;
//! every slice is checked; a fraction's parts are reduced with checked arithmetic.
//!
//! What the spec leaves open keeps the spacing of its examples (`f(x)`, `x[k]`, `a: T`, `?"q"`,
//! `8 PLN/hour`, `19 %`), as refcore does.

use super::expr::is_builtin;
use super::lex::{Canon, Kind, Role, Tok};
use super::parse::{Stop, is_keyword};
use crate::base::{Code, Refusal};

/// The words a `[` after them is spaced from: a calendar's lists (`weekends [sat, sun]`).
const LIST_WORDS: [&str; 3] = ["weekends", "yearly", "added"];

/// The canonical text of `toks`, one line's tokens.
///
/// # Errors
///
/// `ENGINE-INVARIANT` when a fraction's parts aren't two integers of 64 bits.
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
        let call = p.kind == Kind::Id && (is_builtin(&p.text) || pm.name);
        return !(p.text == "]" || call);
    }
    if t.text == "[" && t_op {
        let keyed =
            p.kind == Kind::Id && !is_keyword(&p.text) && !LIST_WORDS.contains(&p.text.as_str());
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
            Ok(reduced(Some(tok), denom)?.0.to_string())
        }
        Canon::Denominator => {
            let numer = at.checked_sub(2).and_then(|n| toks.get(n));
            Ok(reduced(numer, Some(tok))?.1.to_string())
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

/// A fraction's numerator and denominator, reduced (L §2.4: "a reduced a/b").
fn reduced(numer: Option<&Tok>, denom: Option<&Tok>) -> Result<(i64, i64), Stop> {
    let (Some(a), Some(b)) = (int_of(numer), int_of(denom)) else {
        return Err(not_a_fraction());
    };
    let g = gcd(a, b).ok_or_else(not_a_fraction)?;
    let a = a.checked_div_euclid(g).ok_or_else(not_a_fraction)?;
    let b = b.checked_div_euclid(g).ok_or_else(not_a_fraction)?;
    Ok((a, b))
}

/// An integer token's value, if it fits 64 bits.
fn int_of(tok: Option<&Tok>) -> Option<i64> {
    if let Some(Tok {
        kind: Kind::Int(n), ..
    }) = tok
    {
        return n.to_i64();
    }
    None
}

/// The greatest common divisor of two integers of which `b` isn't 0.
fn gcd(a: i64, b: i64) -> Option<i64> {
    let (mut a, mut b) = (a.checked_abs()?, b.checked_abs()?);
    while b != 0 {
        let r = a.checked_rem_euclid(b)?;
        a = b;
        b = r;
    }
    (a != 0).then_some(a)
}

/// The refusal of a fraction the reprint can't reduce.
fn not_a_fraction() -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "a fraction's parts are not two integers of 64 bits".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{gcd, number_text};

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

    #[test]
    fn a_fraction_reduces() {
        assert_eq!(gcd(6, 4), Some(2), "6/4 is 3/2");
        assert_eq!(gcd(1, 3), Some(1), "1/3 stays");
    }
}
