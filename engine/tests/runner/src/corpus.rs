//! The kernel corpus and refcore's outputs for it, read from the bytes the runner embeds
//! (`properties/kernel.corpus`, written by `properties/gen.py`; `properties/kernel.refcore`,
//! written by `properties/laws.py`). Both are netstrings (`<decimal length>:<bytes>,`), the
//! form `INTERFACE.md` §4 gives the engine's file table.

use std::collections::BTreeMap;

/// The corpus `properties/gen.py` writes: every record of the kernel's six laws.
pub const KERNEL_CORPUS: &[u8] = include_bytes!("../../properties/kernel.corpus");

/// refcore's output and exit code for every record of [`KERNEL_CORPUS`], from `properties/laws.py`.
pub const KERNEL_REFCORE: &[u8] = include_bytes!("../../properties/kernel.refcore");

/// One reality of the corpus, in one variant of its law's comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The law it is generated for (`determinism`, `conservation`, …).
    pub law: String,
    /// Its case id, which groups the variants a law compares.
    pub case: String,
    /// Its variant (`one`, `split`, `base`, `later`, `control`, `dropped`).
    pub variant: String,
    /// The command line's words after the program name.
    pub args: Vec<String>,
    /// The file table, packed as `INTERFACE.md` §4 says: what `tally_cli::run` takes.
    pub files: Vec<u8>,
}

/// What one run printed and its exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// Standard output, then standard error, as `run.py` compares them.
    pub out: String,
    /// The exit code.
    pub code: u8,
}

/// Takes one netstring from `bytes` at `at`: its content and the position after it.
pub fn netstring(bytes: &[u8], at: usize) -> Result<(&[u8], usize), String> {
    let rest = bytes.get(at..).ok_or("a netstring past the end")?;
    let colon = rest
        .iter()
        .position(|b| *b == b':')
        .ok_or("a netstring without its ':'")?;
    let digits = rest.get(..colon).ok_or("a netstring's length")?;
    let text = std::str::from_utf8(digits).map_err(|e| e.to_string())?;
    let len: usize = text
        .parse()
        .map_err(|_| format!("a netstring length '{text}'"))?;
    let start = colon.checked_add(1).ok_or("overflow")?;
    let end = start.checked_add(len).ok_or("overflow")?;
    let body = rest
        .get(start..end)
        .ok_or("a netstring shorter than its length")?;
    if rest.get(end) != Some(&b',') {
        return Err(format!("a netstring at {at} without its ','"));
    }
    let next = at
        .checked_add(end)
        .and_then(|n| n.checked_add(1))
        .ok_or("overflow")?;
    Ok((body, next))
}

/// Every netstring of `bytes`, in order.
pub fn netstrings(bytes: &[u8]) -> Result<Vec<&[u8]>, String> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let (body, next) = netstring(bytes, at)?;
        out.push(body);
        at = next;
    }
    Ok(out)
}

fn text(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
}

/// Packs `(path, content)` pairs as `INTERFACE.md` §4's file table.
pub fn pack(files: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (path, content) in files {
        for part in [path.as_bytes(), content.as_bytes()] {
            out.extend_from_slice(part.len().to_string().as_bytes());
            out.push(b':');
            out.extend_from_slice(part);
            out.push(b',');
        }
    }
    out
}

/// The paths of a file table (`INTERFACE.md` §4), in its order: every other netstring,
/// from the first.
pub fn paths(table: &[u8]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut is_path = true;
    for part in netstrings(table)? {
        if is_path {
            out.push(text(part)?);
        }
        is_path = !is_path;
    }
    Ok(out)
}

/// Groups `items` in fours or fives: a record's netstrings, never a partial one.
fn groups<'a>(items: &[&'a [u8]], size: usize) -> Result<Vec<Vec<&'a [u8]>>, String> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for item in items {
        cur.push(*item);
        if cur.len() == size {
            out.push(std::mem::take(&mut cur));
        }
    }
    if cur.is_empty() {
        Ok(out)
    } else {
        Err("the bytes end inside a record".to_owned())
    }
}

/// The corpus's records, in the order `gen.py` wrote them.
pub fn records(bytes: &[u8]) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for five in groups(&netstrings(bytes)?, 5)? {
        let [law, case, variant, args, files] = five.as_slice() else {
            return Err("a record is five netstrings".to_owned());
        };
        out.push(Record {
            law: text(law)?,
            case: text(case)?,
            variant: text(variant)?,
            args: text(args)?.split(' ').map(str::to_owned).collect(),
            files: files.to_vec(),
        });
    }
    Ok(out)
}

/// refcore's runs of the corpus, by (case, variant).
pub fn refcore_runs(bytes: &[u8]) -> Result<BTreeMap<(String, String), Run>, String> {
    let mut out = BTreeMap::new();
    for four in groups(&netstrings(bytes)?, 4)? {
        let [case, variant, code, printed] = four.as_slice() else {
            return Err("a refcore record is four netstrings".to_owned());
        };
        let code: u8 = text(code)?
            .parse()
            .map_err(|_| "a refcore exit code".to_owned())?;
        out.insert(
            (text(case)?, text(variant)?),
            Run {
                out: text(printed)?,
                code,
            },
        );
    }
    Ok(out)
}
