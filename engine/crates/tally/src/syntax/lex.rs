//! The lexical rules: a file split into lines with indent levels, each line into tokens.
//! Spec: L §2.1 (lexical rules); L §8.3 (`LOAD-PARSE`: tabs, bad indentation).
//! Twin: engine/tools/refcore/lexer.py (`split_lines`, `lex_line`, `_number`).
//! Invariants: every loop consumes at least one character or stops; a token remembers whether
//! white space came before it, since a touching `*` or `/` may belong to a unit (L §2.1).
//!
//! The error texts are refcore's, word for word, since a check may print them.

use crate::base::{Code, Load, Place, Refusal};
use crate::num::Num;
use crate::time::Day;

/// What a token is, with its value where it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// An id: `[a-z][a-z0-9_]*`, at most 48 characters (keywords included).
    Id,
    /// A key that is no id: it holds `-`, or starts with a digit and is no number or date.
    Key,
    /// An integer, `40000` or `96 000`.
    Int(Num),
    /// A decimal, `1190.05`.
    Dec(Num),
    /// A day, `2026-07-14`.
    Day(Day),
    /// A month, `2026-07`: its year and month.
    Month(u16, u8),
    /// A quarter, `2026-Q1`: its year and quarter.
    Quarter(u16, u8),
    /// A month-day, `04-30`: its month and day, unchecked here.
    MonthDay(u8, u8),
    /// A currency code: three upper-case letters.
    Cur,
    /// A string, its text with the escapes undone.
    Str(String),
    /// An operator or punctuation: `==`, `..`, `(`, `%`, ….
    Op,
}

/// One token of a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tok {
    /// What it is.
    pub kind: Kind,
    /// Its source text, as written (a string with its quotes).
    pub text: String,
    /// Whether white space came before it on its line.
    pub ws: bool,
}

/// One non-blank line of a file: its indent level and its tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// Its indent level: two spaces a level.
    pub level: u32,
    /// Its tokens, in order.
    pub toks: Vec<Tok>,
    /// Its line number in the file, from 1.
    pub number: u32,
}

/// The two-character operators.
const TWO: [&str; 5] = ["==", "!=", "<=", ">=", ".."];
/// The one-character operators and punctuation.
const ONE: &str = "<>+-*/()[]{},:=.?^%";
/// The longest id.
const ID_MAX: usize = 48;

/// A `LOAD-PARSE` refusal, `cannot parse: <what>`, at a place. L §8.3.
pub(crate) fn cannot(what: &str, place: Place) -> Refusal {
    Refusal::at(
        Code::Load(Load::Parse),
        format!("cannot parse: {what}"),
        place,
    )
}

/// Splits a file into its non-blank lines with their indent levels, and each line into tokens.
/// L §2.1.
///
/// # Errors
///
/// `LOAD-PARSE` for a `\r`, a tab, an odd indentation or a token that can't be read.
pub fn split_lines(src: &str, file: &str) -> Result<Vec<Line>, Refusal> {
    if src.contains('\r') {
        return Err(cannot(
            "lines end with \\n only",
            Place::File(file.to_owned()),
        ));
    }
    let mut out = Vec::new();
    for (i, raw) in src.split('\n').enumerate() {
        let number = i
            .checked_add(1)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| cannot("too many lines", Place::File(file.to_owned())))?;
        let place = || Place::Line(file.to_owned(), number);
        if raw.contains('\t') {
            return Err(cannot("tab", place()));
        }
        if raw.trim().is_empty() {
            continue;
        }
        let spaces = raw.bytes().take_while(|b| *b == b' ').count();
        let body = raw
            .get(spaces..)
            .ok_or_else(|| cannot("indentation", place()))?;
        if !spaces.is_multiple_of(2) {
            return Err(cannot(&format!("indentation of {spaces} spaces"), place()));
        }
        let level = spaces
            .checked_div(2)
            .and_then(|l| u32::try_from(l).ok())
            .ok_or_else(|| cannot("indentation", place()))?;
        let toks = lex_line(body).map_err(|what| cannot(&what, place()))?;
        out.push(Line {
            level,
            toks,
            number,
        });
    }
    Ok(out)
}

/// The tokens of one line's text, after its indentation. The error is the text after
/// `cannot parse: `. L §2.1.
fn lex_line(text: &str) -> Result<Vec<Tok>, String> {
    let mut toks = Vec::new();
    let mut rest = text;
    let mut ws = false;
    while let Some(c) = rest.chars().next() {
        if c == ' ' {
            ws = true;
            rest = rest.get(1..).ok_or("a space")?;
            continue;
        }
        let (kind, len) = match c {
            '"' => string(rest)?,
            '0'..='9' => number(rest)?,
            'a'..='z' => word(rest)?,
            'A'..='Z' => currency(rest)?,
            _ => operator(rest, c)?,
        };
        let (written, after) = rest.split_at_checked(len).ok_or("a token")?;
        toks.push(Tok {
            kind,
            text: written.to_owned(),
            ws,
        });
        rest = after;
        ws = false;
    }
    Ok(toks)
}

/// Whether `c` may stand in a word: `[a-z0-9_-]`.
fn is_word(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
}

/// The length in bytes of the run of word characters at the start of `text`.
fn word_len(text: &str) -> usize {
    text.find(|c: char| !is_word(c)).unwrap_or(text.len())
}

/// A string: `\"` and `\\` are its only escapes. L §2.1.
fn string(text: &str) -> Result<(Kind, usize), String> {
    let mut value = String::new();
    let mut chars = text.char_indices().skip(1);
    while let Some((at, c)) = chars.next() {
        match c {
            '"' => {
                let len = at.checked_add(1).ok_or("a string")?;
                return Ok((Kind::Str(value), len));
            }
            '\\' => match chars.next() {
                Some((_, escaped @ ('"' | '\\'))) => value.push(escaped),
                Some(_) | None => return Err("bad escape in string".to_owned()),
            },
            _ => value.push(c),
        }
    }
    Err("unterminated string".to_owned())
}

/// A token that starts with a digit: a grouped number, a quarter, a decimal, an integer, a
/// day, a month, a month-day or a key. L §2.1.
fn number(text: &str) -> Result<(Kind, usize), String> {
    if let Some(len) = grouped(text) {
        let written = text.get(..len).ok_or("a number")?;
        let digits: String = written.chars().filter(|c| *c != ' ').collect();
        let value = Num::parse_decimal(&digits).map_err(|_| written.to_owned())?;
        let kind = if digits.contains('.') {
            Kind::Dec(value)
        } else {
            Kind::Int(value)
        };
        return Ok((kind, len));
    }
    let len = word_len(text);
    let (word, after) = text.split_at_checked(len).ok_or("a number")?;
    if let Some(quarter) = quarter(word, after) {
        return Ok(quarter);
    }
    if let Some(fraction) = after.strip_prefix('.')
        && word.bytes().all(|b| b.is_ascii_digit())
        && fraction.starts_with(|c: char| c.is_ascii_digit())
    {
        let places = fraction
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(fraction.len());
        let end = len
            .checked_add(1)
            .and_then(|n| n.checked_add(places))
            .ok_or("a decimal")?;
        let written = text.get(..end).ok_or("a decimal")?;
        if let Some(next) = fraction.get(places..).and_then(|r| r.chars().next())
            && is_word(next)
        {
            return Err(format!("{written}{next}"));
        }
        let value = Num::parse_decimal(written).map_err(|_| written.to_owned())?;
        return Ok((Kind::Dec(value), end));
    }
    Ok((classify(word)?, len))
}

/// `\d{4}-` followed by `Q1` to `Q4` and no word character: a quarter. L §2.1.
fn quarter(word: &str, after: &str) -> Option<(Kind, usize)> {
    let year = word.strip_suffix('-')?;
    if year.len() != 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut chars = after.chars();
    let (Some('Q'), Some(q @ '1'..='4')) = (chars.next(), chars.next()) else {
        return None;
    };
    if chars.next().is_some_and(is_word) {
        return None;
    }
    let year = year.parse::<u16>().ok()?;
    let q = u8::try_from(q.to_digit(10)?).ok()?;
    Some((Kind::Quarter(year, q), word.len().checked_add(2)?))
}

/// Whether `s` is exactly `n` ASCII digits. L §2.1.
fn all_digits(s: &str, n: usize) -> bool {
    s.len() == n && s.bytes().all(|b| b.is_ascii_digit())
}

/// A word of digits and dashes: an integer, a day, a month, a month-day, else a key
/// (refcore's `_number`). L §2.1.
fn classify(word: &str) -> Result<Kind, String> {
    let parts: Vec<&str> = word.split('-').collect();

    match parts.as_slice() {
        [whole] if !whole.is_empty() && whole.bytes().all(|b| b.is_ascii_digit()) => {
            let value = Num::parse_decimal(whole).map_err(|_| (*whole).to_owned())?;
            Ok(Kind::Int(value))
        }
        [y, m, d] if all_digits(y, 4) && all_digits(m, 2) && all_digits(d, 2) => {
            let not_a_day = || format!("{word} is not a day");
            let day = Day::from_ymd(
                y.parse().map_err(|_| not_a_day())?,
                m.parse().map_err(|_| not_a_day())?,
                d.parse().map_err(|_| not_a_day())?,
            );
            day.map(Kind::Day).ok_or_else(not_a_day)
        }
        [y, m] if all_digits(y, 4) && all_digits(m, 2) => {
            let not_a_month = || format!("{word} is not a month");
            let month: u8 = m.parse().map_err(|_| not_a_month())?;
            if !(1..=12).contains(&month) {
                return Err(not_a_month());
            }
            Ok(Kind::Month(y.parse().map_err(|_| not_a_month())?, month))
        }
        [m, d] if all_digits(m, 2) && all_digits(d, 2) => {
            let bad = || format!("{word} is not a month-day");
            Ok(Kind::MonthDay(
                m.parse().map_err(|_| bad())?,
                d.parse().map_err(|_| bad())?,
            ))
        }
        _ => Ok(Kind::Key),
    }
}

/// The length of a number with its thousands grouped by spaces at the start of `text`, as
/// refcore's `\d{1,3}(?: \d{3})+(?:\.\d+)?(?![0-9a-z_\-])` matches it, backtracking
/// included; `None` when it doesn't match. L §2.1.
fn grouped(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let digit_at = |at: usize| bytes.get(at).is_some_and(u8::is_ascii_digit);
    let lead = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    if lead == 0 || lead > 3 {
        return None;
    }
    let mut ends = Vec::new();
    let mut at = lead;
    loop {
        let group = (1..=3).all(|k| at.checked_add(k).is_some_and(digit_at));
        if bytes.get(at) != Some(&b' ') || !group {
            break;
        }
        at = at.checked_add(4)?;
        ends.push(at);
    }
    let free = |end: usize| ends_word(text, end);
    for end in ends.into_iter().rev() {
        let point = end.checked_add(1)?;
        if bytes.get(end) == Some(&b'.') && digit_at(point) {
            let places = bytes
                .get(point..)?
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .count();
            let with_decimals = point.checked_add(places)?;
            if free(with_decimals) {
                return Some(with_decimals);
            }
        }
        if free(end) {
            return Some(end);
        }
    }
    None
}

/// An id or a key that starts with a letter. L §2.1.
fn word(text: &str) -> Result<(Kind, usize), String> {
    let len = word_len(text);
    let word = text.get(..len).ok_or("a word")?;
    if word.contains('-') {
        return Ok((Kind::Key, len));
    }
    if len > ID_MAX {
        return Err(format!(
            "identifier {word} is longer than {ID_MAX} characters"
        ));
    }
    Ok((Kind::Id, len))
}

/// A currency code: exactly three upper-case letters, not followed by a word character.
/// L §2.1.
fn currency(text: &str) -> Result<(Kind, usize), String> {
    let len = text
        .find(|c: char| !c.is_ascii_uppercase())
        .unwrap_or(text.len());
    let (code, after) = text.split_at_checked(len).ok_or("a currency code")?;
    let next = after.chars().next();
    if len != 3 || next.is_some_and(is_word) {
        let shown: String = code.chars().chain(next).collect();
        return Err(format!("{shown} is not a currency code"));
    }
    Ok((Kind::Cur, len))
}

/// An operator: two characters first, then one. L §2.3.
fn operator(text: &str, c: char) -> Result<(Kind, usize), String> {
    if TWO.iter().any(|two| text.starts_with(two)) {
        return Ok((Kind::Op, 2));
    }
    if ONE.contains(c) {
        return Ok((Kind::Op, 1));
    }
    Err(format!("character {}", python_repr(c)))
}

/// A character as Python's `repr` prints it, for refcore's message. L §8.3.
fn python_repr(c: char) -> String {
    let code = u32::from(c);
    match c {
        '\'' => "\"'\"".to_owned(),
        '\\' => "'\\\\'".to_owned(),
        '\n' => "'\\n'".to_owned(),
        '\r' => "'\\r'".to_owned(),
        '\t' => "'\\t'".to_owned(),
        _ if c.is_control() || (c.is_whitespace() && c != ' ') || is_format(c) => {
            if code < 0x100 {
                format!("'\\x{code:02x}'")
            } else if code < 0x1_0000 {
                format!("'\\u{code:04x}'")
            } else {
                format!("'\\U{code:08x}'")
            }
        }
        _ => format!("'{c}'"),
    }
}

/// The invisible format characters Python's `repr` escapes (category Cf, the common ones).
/// L §8.3.
fn is_format(c: char) -> bool {
    c == '\u{ad}'
        || c == '\u{feff}'
        || ('\u{200b}'..='\u{200f}').contains(&c)
        || ('\u{2028}'..='\u{202e}').contains(&c)
        || ('\u{2060}'..='\u{2064}').contains(&c)
}

/// Whether the text ends at `end`, or the character there can't continue a word (the
/// regex's `(?![0-9a-z_\-])`). L §2.1.
fn ends_word(text: &str, end: usize) -> bool {
    match text.get(end..) {
        Some(after) => !after.chars().next().is_some_and(is_word),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, Line, Tok, lex_line, python_repr, split_lines};
    use crate::num::Num;

    fn kinds(text: &str) -> Vec<(String, bool)> {
        lex_line(text)
            .expect("lexes")
            .into_iter()
            .map(|t: Tok| (t.text, t.ws))
            .collect()
    }

    fn error(src: &str) -> String {
        split_lines(src, "f.tally").expect_err("refused").text()
    }

    #[test]
    fn tokens_remember_white_space() {
        assert_eq!(
            kinds("a  =b*2"),
            [
                ("a".to_owned(), false),
                ("=".to_owned(), true),
                ("b".to_owned(), false),
                ("*".to_owned(), false),
                ("2".to_owned(), false)
            ],
            "touching and spaced"
        );
    }

    #[test]
    fn numbers_dates_and_keys() {
        let toks =
            lex_line("96 000 1 190.05 0.0023 2026-07-14 2026-07 2026-Q1 04-30 ty2014-15 1e-4")
                .expect("lexes");
        let k: Vec<&Kind> = toks.iter().map(|t| &t.kind).collect();
        assert_eq!(*k[0], Kind::Int(Num::int(96_000)), "grouped");
        assert_eq!(
            *k[1],
            Kind::Dec(Num::parse_decimal("1190.05").expect("n")),
            "grouped decimal"
        );
        assert_eq!(
            *k[2],
            Kind::Dec(Num::parse_decimal("0.0023").expect("n")),
            "decimal"
        );
        assert_eq!(
            *k[3],
            Kind::Day(crate::time::Day::from_ymd(2026, 7, 14).expect("d")),
            "day"
        );
        assert_eq!(*k[4], Kind::Month(2026, 7), "month");
        assert_eq!(*k[5], Kind::Quarter(2026, 1), "quarter");
        assert_eq!(*k[6], Kind::MonthDay(4, 30), "month-day");
        assert_eq!(*k[7], Kind::Key, "a key with a dash");
        assert_eq!(*k[8], Kind::Key, "a step's key");
        assert_eq!(toks.len(), 9, "nine tokens");
    }

    #[test]
    fn grouping_backtracks_as_the_regex_does() {
        assert_eq!(
            kinds("1 000 000x").first().map(|t| t.0.as_str()),
            Some("1 000"),
            "one group"
        );
        assert_eq!(
            kinds("1 0001").first().map(|t| t.0.as_str()),
            Some("1"),
            "no group"
        );
        assert_eq!(
            kinds("1 000.5x").first().map(|t| t.0.as_str()),
            Some("1 000"),
            "no decimals"
        );
        assert_eq!(
            kinds("1234 567").first().map(|t| t.0.as_str()),
            Some("1234"),
            "four digits"
        );
    }

    #[test]
    fn strings_ids_currencies_and_operators() {
        let toks = lex_line(r#"x "a \"b\" \\ c" PLN .. == ?"#).expect("lexes");
        assert_eq!(toks[0].kind, Kind::Id, "id");
        assert_eq!(
            toks[1].kind,
            Kind::Str(r#"a "b" \ c"#.to_owned()),
            "escapes"
        );
        assert_eq!(toks[2].kind, Kind::Cur, "currency");
        assert_eq!(
            toks.iter()
                .skip(3)
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>(),
            ["..", "==", "?"],
            "operators"
        );
    }

    #[test]
    fn lines_have_levels() {
        let lines = split_lines("model m\n  tally 2\n\n   \nkind k", "f.tally").expect("splits");
        let levels: Vec<(u32, u32)> = lines.iter().map(|l: &Line| (l.level, l.number)).collect();
        assert_eq!(levels, [(0, 1), (1, 2), (0, 5)], "blank lines are skipped");
    }

    #[test]
    fn refusals_word_for_word() {
        assert_eq!(
            error("a\r\n"),
            "LOAD-PARSE cannot parse: lines end with \\n only (f.tally)",
            "a carriage return"
        );
        assert_eq!(
            error("a\n\tb"),
            "LOAD-PARSE cannot parse: tab (f.tally:2)",
            "tab"
        );
        assert_eq!(
            error("   a"),
            "LOAD-PARSE cannot parse: indentation of 3 spaces (f.tally:1)",
            "odd"
        );
        assert_eq!(
            error("\"abc"),
            "LOAD-PARSE cannot parse: unterminated string (f.tally:1)",
            "string"
        );
        assert_eq!(
            error("\"a\\n\""),
            "LOAD-PARSE cannot parse: bad escape in string (f.tally:1)",
            "escape"
        );
        assert_eq!(
            error("2026-02-30"),
            "LOAD-PARSE cannot parse: 2026-02-30 is not a day (f.tally:1)",
            "day"
        );
        assert_eq!(
            error("2026-13"),
            "LOAD-PARSE cannot parse: 2026-13 is not a month (f.tally:1)",
            "month"
        );
        assert_eq!(
            error("1.5x"),
            "LOAD-PARSE cannot parse: 1.5x (f.tally:1)",
            "decimal"
        );
        assert_eq!(
            error("PLNX"),
            "LOAD-PARSE cannot parse: PLNX is not a currency code (f.tally:1)",
            "four letters"
        );
        assert_eq!(
            error("PLNa"),
            "LOAD-PARSE cannot parse: PLNa is not a currency code (f.tally:1)",
            "a letter after"
        );
        assert_eq!(
            error("a ! b"),
            "LOAD-PARSE cannot parse: character '!' (f.tally:1)",
            "a character"
        );
        let long = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(
            error(long),
            format!(
                "LOAD-PARSE cannot parse: identifier {long} is longer than 48 characters (f.tally:1)"
            ),
            "id"
        );
    }

    #[test]
    fn characters_print_as_python_does() {
        assert_eq!(python_repr('!'), "'!'", "plain");
        assert_eq!(python_repr('\''), "\"'\"", "a quote");
        assert_eq!(python_repr('\u{0}'), "'\\x00'", "a control");
        assert_eq!(python_repr('\u{a0}'), "'\\xa0'", "no-break space");
        assert_eq!(python_repr('ą'), "'ą'", "a letter");
    }
}
