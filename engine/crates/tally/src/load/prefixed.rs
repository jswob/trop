//! A joint model's member lines: each member's lines under its prefix, the joined declarations dropped, its currencies and calendar, its names renamed.
//! Spec: L §6.1 (a member's items read as `a.item`, its keys written as literals `a.x[k]`;
//! currencies one by their code, two minor units `LOAD-SAME`), §2.2 and §4.1 (a member's own
//! calendar where its run differs), §8.3 (`LOAD-SAME`).
//! Twin: engine/tools/refcore/joint.py (`member_lines`, `drop_blocks`, `drop_entry_keys`,
//! `dedupe_units`, `own_calendar_lines`, `rename_tokens`, `_rename_ok`, `rewrite_member_refs`,
//! `rewrite_member_names`, `retok`).
//! Invariants: only a token a member's own parse read as an id is renamed, its place and mark
//! kept; a line is named by its file and place, never by address; a dropped block is a
//! declaration joined to another's.

use super::check::invariant;
use super::joint::{SEP, owner_of};
use super::merge::{CALENDAR, Drop, FileLines, LineAt, Plan, blocks, line_word};
use crate::base::{Code, Load, Refusal};
use crate::syntax::{Kind, Line, Read, Stop, Tok, split_lines};
use crate::time::Day;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// A member's lines under its prefix (refcore's `member_lines`): its header a part of the joint
/// model, its asks and the declarations joined to another's dropped, its names renamed and
/// the joins applied. L §6.1.
pub(super) fn member_lines(
    alias: &str,
    path: &str,
    lines: &[Line],
    names: &BTreeSet<String>,
    plan: &Plan,
    drops: &[Drop],
    keys: Option<&BTreeMap<LineAt, BTreeSet<String>>>,
) -> Vec<Line> {
    let mut keep: Vec<(Option<usize>, Line)> = Vec::new();
    for (n, (b0, b1)) in blocks(lines).into_iter().enumerate() {
        let Some((first, under)) = lines.get(b0..b1).and_then(<[Line]>::split_first) else {
            continue;
        };
        if n == 0 {
            keep.push((None, part_header(alias, first)));
            let kept = under
                .iter()
                .filter(|l| matches!(line_word(l), "tally" | "law"));
            keep.extend(kept.map(|l| (None, l.clone())));
            continue;
        }
        let word = line_word(first);
        if word == "ask" || dropped(word, first, path, b0, drops) {
            continue;
        }
        let block = lines.iter().enumerate().take(b1).skip(b0);
        keep.extend(block.map(|(at, line)| (Some(at), line.clone())));
    }
    let mut out = Vec::with_capacity(keep.len());
    for (at, line) in keep {
        let gone = keys.zip(at).and_then(|(k, i)| k.get(&(path.to_owned(), i)));
        let toks = match gone {
            Some(gone) => match drop_entry_keys(&line.toks, gone) {
                Some(toks) => toks,
                None => continue,
            },
            None => line.toks,
        };
        out.push(Line {
            toks: rename_line(alias, &toks, names, &plan.jmap),
            ..line
        });
    }
    out
}

/// A member file's header as a part of the joint model: `part a·member` for its model file,
/// `part a·p` for its part `p` (refcore's `member_lines`, its header). L §6.1.
fn part_header(alias: &str, first: &Line) -> Line {
    let pid = match first.toks.get(1) {
        Some(id) if line_word(first) == "part" => id.text.as_str(),
        Some(_) | None => "member",
    };
    let word = Tok {
        kind: Kind::Id,
        text: "part".to_owned(),
        ws: false,
        seq: 0,
        mark: Cell::default(),
    };
    let id = Tok {
        kind: Kind::Id,
        text: format!("{alias}{SEP}{pid}"),
        ws: true,
        seq: 1,
        mark: Cell::default(),
    };
    Line {
        level: 0,
        toks: vec![word, id],
        number: first.number,
    }
}

/// Whether a block is a declaration joined to another's, and dropped.
fn dropped(word: &str, first: &Line, path: &str, start: usize, drops: &[Drop]) -> bool {
    if drops.contains(&Drop::Block((path.to_owned(), start))) {
        return true;
    }
    word == "kind"
        && first
            .toks
            .get(1)
            .is_some_and(|id| drops.contains(&Drop::Kind(id.text.clone())))
}

/// The joint model's own lines with the declarations joined to another's dropped (refcore's
/// `drop_blocks`), each line with its place among the file's lines as split.
pub(super) fn drop_blocks(
    path: &str,
    pairs: Vec<(usize, Line)>,
    drops: &[Drop],
    keys: Option<&BTreeMap<LineAt, BTreeSet<String>>>,
) -> Vec<Line> {
    if drops.is_empty() && keys.is_none() {
        return pairs.into_iter().map(|(_, line)| line).collect();
    }
    let levels: Vec<Line> = pairs.iter().map(|(_, l)| l.clone()).collect();
    let mut out = Vec::with_capacity(pairs.len());
    for (b0, b1) in blocks(&levels) {
        let Some(block) = pairs.get(b0..b1) else {
            continue;
        };
        let Some((start, first)) = block.first() else {
            continue;
        };
        if dropped(line_word(first), first, path, *start, drops) {
            continue;
        }
        for (at, line) in block {
            match keys.and_then(|k| k.get(&(path.to_owned(), *at))) {
                Some(gone) => out.extend(drop_entry_keys(&line.toks, gone).map(|toks| Line {
                    toks,
                    ..line.clone()
                })),
                None => out.push(line.clone()),
            }
        }
    }
    out
}

/// An entry line `K + [k1, k2] …` without the keys named; `None` when no key is left
/// (refcore's `drop_entry_keys`).
fn drop_entry_keys(toks: &[Tok], names: &BTreeSet<String>) -> Option<Vec<Tok>> {
    let open = toks.iter().position(|t| t.text == "[");
    let close = toks.iter().position(|t| t.text == "]");
    let (Some(i), Some(j)) = (open, close) else {
        return Some(toks.to_vec());
    };
    let none: &[Tok] = &[];
    let inside = toks
        .get(i..j)
        .and_then(<[Tok]>::split_first)
        .map_or(none, |(_, rest)| rest);
    let left: Vec<&Tok> = inside
        .iter()
        .filter(|t| t.text != "," && !names.contains(&t.text))
        .collect();
    if left.is_empty() {
        return None;
    }
    let mut out: Vec<Tok> = toks.iter().take(i).cloned().collect();
    out.extend(toks.get(i).cloned());
    for (k, t) in left.iter().enumerate() {
        if k > 0 {
            out.push(Tok {
                kind: Kind::Op,
                text: ",".to_owned(),
                ws: false,
                seq: t.seq,
                mark: Cell::default(),
            });
        }
        out.push((*t).clone());
    }
    out.extend(toks.iter().skip(j).cloned());
    Some(out)
}

/// A currency declared by several files is one when the declarations agree (refcore's
/// `dedupe_units`): its later declarations dropped. L §6.1 (Currencies).
///
/// # Errors
///
/// `LOAD-SAME currency <C> is declared with different minor units by <a> and <b>`.
pub(super) fn dedupe_units(
    files: Vec<FileLines>,
    owners: &BTreeMap<String, String>,
) -> Result<Vec<FileLines>, Stop> {
    let mut seen: BTreeMap<String, (Vec<String>, String)> = BTreeMap::new();
    let mut out = Vec::with_capacity(files.len());
    for (path, lines) in files {
        let who = owners
            .get(&path)
            .cloned()
            .unwrap_or_else(|| "the joint model".to_owned());
        let mut keep = Vec::with_capacity(lines.len());
        for (n, (b0, b1)) in blocks(&lines).into_iter().enumerate() {
            let block = lines.get(b0..b1).unwrap_or(&[]);
            let Some((code, sig)) = currency_block(n, block) else {
                keep.extend(block.iter().cloned());
                continue;
            };
            match seen.get(&code) {
                Some((before, by)) if *before != sig => {
                    let message = format!(
                        "currency {code} is declared with different minor units by {by} and {who}"
                    );
                    return Err(Stop::Refused(Refusal::new(Code::Load(Load::Same), message)));
                }
                Some(_) => {}
                None => {
                    seen.insert(code, (sig, who.clone()));
                    keep.extend(block.iter().cloned());
                }
            }
        }
        out.push((path, keep));
    }
    Ok(out)
}

/// A block that declares a currency, `unit XYZ: …`, past a file's header: its code and its
/// first line's words (refcore's `dedupe_units`, its test). L §6.1 (Currencies).
fn currency_block(n: usize, block: &[Line]) -> Option<(String, Vec<String>)> {
    let first = block.first()?;
    let [unit, code, ..] = first.toks.as_slice() else {
        return None;
    };
    if n == 0 || unit.text != "unit" || code.kind != Kind::Cur {
        return None;
    }
    let sig = first.toks.iter().map(|t| t.text.clone()).collect();
    Some((code.text.clone(), sig))
}

/// A member's own calendar indexes, `a·days`, `a·months` and `a·years` over its run (refcore's
/// `own_calendar_lines`). L §2.2, §4.1.
///
/// # Errors
///
/// A split refusal, which the text written can't meet.
pub(super) fn own_calendar(
    alias: &str,
    path: &str,
    start: Day,
    end: Day,
) -> Result<Vec<Line>, Stop> {
    let text = format!(
        "index zzcaldays = day {}..{}\nindex zzcalmonths = month {:04}-{:02}..{:04}-{:02}\nindex zzcalyears = year {:04}..{:04}\n",
        start.text(),
        end.text(),
        start.year(),
        start.month(),
        end.year(),
        end.month(),
        start.year(),
        end.year()
    );
    let mut lines = split_lines(&text, path)?;
    for (line, name) in lines.iter_mut().zip(CALENDAR) {
        line.toks = line
            .toks
            .iter()
            .map(|t| {
                if t.kind == Kind::Id && t.text.starts_with("zzcal") {
                    retok(t, format!("{alias}{SEP}{name}"))
                } else {
                    t.clone()
                }
            })
            .collect();
    }
    Ok(lines)
}

/// A line's tokens with a member's names under its prefix and the joins applied (refcore's
/// `rename_tokens`): only a token its own parse read as an id that names an item.
pub(super) fn rename_line(
    alias: &str,
    toks: &[Tok],
    names: &BTreeSet<String>,
    jmap: &BTreeMap<String, String>,
) -> Vec<Tok> {
    let mut out = Vec::with_capacity(toks.len());
    for (j, t) in toks.iter().enumerate() {
        if t.kind == Kind::Id
            && names.contains(&t.text)
            && t.mark.get().read == Read::Name
            && rename_ok(toks, j)
        {
            let full = format!("{alias}{SEP}{}", t.text);
            out.push(retok(t, jmap.get(&full).cloned().unwrap_or(full)));
        } else {
            out.push(t.clone());
        }
    }
    out
}

/// Whether the id token at `j` names an item, not a field, a rule label, a leg's or an
/// attribute's name, a record field or a band's argument (refcore's `_rename_ok`).
fn rename_ok(toks: &[Tok], j: usize) -> bool {
    let Some(t) = toks.get(j) else {
        return false;
    };
    if t.kind != Kind::Id {
        return false;
    }
    let prev = j.checked_sub(1).and_then(|i| toks.get(i));
    let next = j.checked_add(1).and_then(|i| toks.get(i));
    if prev.is_some_and(|p| p.text == ".") && !t.ws {
        return false;
    }
    if prev.is_some_and(|p| matches!(p.text.as_str(), "rule" | "leg" | "attr")) {
        return false;
    }
    if next.is_some_and(|n| n.text == ":")
        && prev.is_some_and(|p| matches!(p.text.as_str(), "(" | ","))
    {
        return false;
    }
    !in_beats_list(toks, j)
}

/// Whether the token at `j` is a label of a `beats` list (refcore's `_rename_ok`, its walk
/// back over the list).
fn in_beats_list(toks: &[Tok], j: usize) -> bool {
    let mut k = j;
    while let Some(i) = k.checked_sub(1) {
        let Some(c) = toks.get(i) else {
            break;
        };
        let listed = c.text == "," || (c.kind == Kind::Id && c.text != "beats");
        if !listed {
            break;
        }
        let before = i.checked_sub(1).and_then(|b| toks.get(b));
        if c.kind == Kind::Id && before.is_some_and(|b| b.text != "," && b.text != "beats") {
            break;
        }
        k = i;
    }
    k.checked_sub(1)
        .and_then(|i| toks.get(i))
        .is_some_and(|c| c.text == "beats")
}

/// The joint model's reads of a member, `a . x` touching, as one name `a·x` (`a . b . x` for a
/// member's member), and a key written as a literal right after a member's item as that
/// member's thing (refcore's `rewrite_member_refs`). L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a name past its line.
pub(super) fn rewrite_member_refs(
    toks: &[Tok],
    rename: &BTreeMap<String, BTreeSet<String>>,
    keynames: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<Tok>, Stop> {
    let mut res = rewrite_member_names(toks, rename)?;
    let empty = BTreeSet::new();
    for j in 0..res.len() {
        let Some(t) = res.get(j) else {
            break;
        };
        let opens = j
            .checked_add(1)
            .and_then(|n| res.get(n))
            .is_some_and(|n| n.text == "[" && !n.ws);
        if t.kind != Kind::Id || !t.text.contains(SEP) || !opens {
            continue;
        }
        let member = owner_of(&t.text).to_owned();
        let pre = t.text.rsplit_once(SEP).map_or("", |(p, _)| p);
        let rel = pre
            .split_once(SEP)
            .map_or_else(String::new, |(_, r)| format!("{r}{SEP}"));
        let names = keynames.get(&member).unwrap_or(&empty);
        member_keys(&mut res, j, (&member, &rel), names);
    }
    Ok(res)
}

/// The keys written as literals in the `[…]` after the member's item at `j`, each a thing or a
/// place the member declares, renamed under its prefix (refcore's `rewrite_member_refs`, its
/// walk over the keys). L §6.1.
fn member_keys(res: &mut [Tok], j: usize, (member, rel): (&str, &str), names: &BTreeSet<String>) {
    let mut depth: u32 = 0;
    let Some(from) = j.checked_add(1) else {
        return;
    };
    for k in from..res.len() {
        let Some(at) = res.get(k) else {
            return;
        };
        let s = at.text.as_str();
        if s == "[" || s == "(" {
            let Some(deeper) = depth.checked_add(1) else {
                return;
            };
            depth = deeper;
            continue;
        }
        if s == "]" || s == ")" {
            match depth.checked_sub(1) {
                Some(0) | None => return,
                Some(shallower) => depth = shallower,
            }
            continue;
        }
        if depth != 1 || at.kind != Kind::Id {
            continue;
        }
        let before = k.checked_sub(1).and_then(|b| res.get(b));
        let after = k.checked_add(1).and_then(|a| res.get(a));
        let listed = before.is_some_and(|b| b.text == "[" || b.text == ",")
            && after.is_some_and(|a| a.text == "," || a.text == "]");
        let full = format!("{rel}{}", at.text);
        if listed && names.contains(&full) {
            let renamed = retok_named(at, format!("{member}{SEP}{full}"));
            if let Some(slot) = res.get_mut(k) {
                *slot = renamed;
            }
        }
    }
}

/// `a . x` as one name `a·x`, the longest chain a member declares (refcore's
/// `rewrite_member_names`). L §6.1.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a name past its line.
fn rewrite_member_names(
    toks: &[Tok],
    rename: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<Tok>, Stop> {
    let past = || invariant("a member's name past its line".to_owned());
    let mut out = Vec::with_capacity(toks.len());
    let mut j: usize = 0;
    while let Some(t) = toks.get(j) {
        let after_dot = j
            .checked_sub(1)
            .and_then(|i| toks.get(i))
            .is_some_and(|p| p.text == ".")
            && !t.ws;
        let names = rename
            .get(&t.text)
            .filter(|_| t.kind == Kind::Id && !after_dot);
        let Some((taken, joined)) = names.and_then(|names| longest_chain(toks, j, names)) else {
            out.push(t.clone());
            j = j.checked_add(1).ok_or_else(past)?;
            continue;
        };
        out.push(retok_named(t, format!("{}{SEP}{joined}", t.text)));
        j = j.checked_add(taken).ok_or_else(past)?;
    }
    Ok(out)
}

/// The longest chain `. x . y` after the member's alias at `j` that it declares, joined by its
/// separator, with the number of tokens the alias and the chain take (refcore's
/// `rewrite_member_names`, its walk). L §6.1.
fn longest_chain(toks: &[Tok], j: usize, names: &BTreeSet<String>) -> Option<(usize, String)> {
    let mut chain: Vec<&str> = Vec::new();
    let mut k = j.checked_add(1)?;
    while let (Some(dot), Some(id)) = (toks.get(k), k.checked_add(1).and_then(|n| toks.get(n))) {
        if dot.text != "." || dot.ws || id.kind != Kind::Id || id.ws {
            break;
        }
        chain.push(&id.text);
        k = k.checked_add(2)?;
    }
    let sep = SEP.to_string();
    for m in (1..=chain.len()).rev() {
        let joined = chain.get(..m)?.join(&sep);
        if names.contains(&joined) {
            return Some((m.checked_mul(2)?.checked_add(1)?, joined));
        }
    }
    None
}

/// A token renamed: an id with the text given, its place and mark kept (refcore's `retok`).
pub(super) fn retok(t: &Tok, text: String) -> Tok {
    Tok {
        kind: Kind::Id,
        text,
        ws: t.ws,
        seq: t.seq,
        mark: Cell::new(t.mark.get()),
    }
}

/// A token renamed and read as an id (refcore's `retok` and its `x.nm = True`).
fn retok_named(t: &Tok, text: String) -> Tok {
    let mut mark = t.mark.get();
    mark.read = Read::Name;
    Tok {
        kind: Kind::Id,
        text,
        ws: t.ws,
        seq: t.seq,
        mark: Cell::new(mark),
    }
}
