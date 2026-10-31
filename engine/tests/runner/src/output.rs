//! Reading what an ask prints (L §5.16): its block of lines, its values, a sum of money in
//! minor units, a hole. The same reading as `properties/laws.py`'s, so a law means one
//! thing on refcore's output and on the engine's.

use std::collections::BTreeMap;

/// `{ask name: its lines}`: each line that doesn't start with a space opens a block, named
/// by what precedes its first `:`; a `warning:` line is a block of its own.
pub fn blocks(out: &str) -> BTreeMap<String, Vec<String>> {
    let mut res: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut cur = String::new();
    for line in out.split('\n').filter(|l| !l.is_empty()) {
        if !line.starts_with(' ') {
            line.split_once(':')
                .map_or(line, |(name, _)| name)
                .clone_into(&mut cur);
            res.insert(cur.clone(), Vec::new());
        }
        res.entry(cur.clone())
            .or_insert_with(Vec::new)
            .push(line.to_owned());
    }
    res
}

/// The first line of the block of ask `name`.
pub fn ask_line<'a>(
    blocks: &'a BTreeMap<String, Vec<String>>,
    name: &str,
) -> Result<&'a str, String> {
    blocks
        .get(name)
        .and_then(|lines| lines.first())
        .map(String::as_str)
        .ok_or_else(|| format!("no line for ask {name}"))
}

/// The values of an ask line `name: v` or `name: (v1, v2, …)`, split at the top-level
/// `, ` (a hole's question is quoted and may hold commas and parentheses).
pub fn values(line: &str) -> Vec<String> {
    let value = line.split_once(": ").map_or("", |(_, v)| v);
    let Some(inner) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) else {
        return vec![value.to_owned()];
    };
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut open: Vec<char> = Vec::new();
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => quoted = !quoted,
            '(' if !quoted => open.push(ch),
            ')' if !quoted => {
                open.pop();
            }
            ',' if !quoted && open.is_empty() && chars.peek() == Some(&' ') => {
                chars.next();
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    out.push(cur);
    out
}

/// A value printed as a sum of DUK (`-1 234.56 DUK`, the thousands grouped by spaces) in
/// minor units; `None` for a hole or anything else.
pub fn cents(value: &str) -> Option<i128> {
    let number = value.strip_suffix(" DUK")?.replace(' ', "");
    let (negative, digits) = number
        .strip_prefix('-')
        .map_or((false, number.as_str()), |d| (true, d));
    let (whole, fraction) = digits.split_once('.')?;
    if fraction.len() != 2
        || whole.is_empty()
        || !whole
            .chars()
            .chain(fraction.chars())
            .all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let units = whole
        .parse::<i128>()
        .ok()?
        .checked_mul(100)?
        .checked_add(fraction.parse::<i128>().ok()?)?;
    if negative {
        units.checked_neg()
    } else {
        Some(units)
    }
}

/// A hole prints `?` and its questions (L §8.1).
pub fn is_hole(value: &str) -> bool {
    value.starts_with('?')
}

#[cfg(test)]
mod tests {
    use super::{blocks, cents, values};

    #[test]
    fn reads_an_ask_line_as_refcore_prints_it() {
        let b = blocks(
            "q: (-1 234.56 DUK, ?\"a, (b)?\" (x), 0.00 DUK)\n  cut 2025-02-01\nwarning: W\n",
        );
        let line = b
            .get("q")
            .and_then(|l| l.first())
            .cloned()
            .expect("the ask q");
        let v = values(&line);
        assert_eq!(v.len(), 3, "three values: {v:?}");
        assert_eq!(v.first().and_then(|x| cents(x)), Some(-123_456), "{v:?}");
        assert_eq!(
            v.get(1).map(String::as_str),
            Some("?\"a, (b)?\" (x)"),
            "{v:?}"
        );
        assert!(b.contains_key("warning"), "a warning is its own block");
        assert_eq!(cents("12.3 DUK"), None, "two decimals only");
    }
}
