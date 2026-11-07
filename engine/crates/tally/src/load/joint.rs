//! A joint model: its members found, pinned and loaded alone, then merged under their prefixes and loaded as one reality.
//! Spec: L §6.1 (Joint models: `use a = "path" sha256 "…"`; a member is closed, loaded alone
//! for its asks, `a.q`), E §3.1 step 1 (a member found relative to the joint model's file, a
//! file or a directory tree, the files given first; its pin the sha256 of its canonical text,
//! or of its bytes with `--pin=bytes`; a member that is a joint model pins its own members),
//! L §8.3 (`LOAD-PIN`, `LOAD-DUP`, `LOAD-HEADER`), E §11 (`REQ-ASK`: a member's ask).
//! Twin: engine/tools/refcore/joint.py (`split_reality`, `header_uses`, `header_has_sames`,
//! `member_texts`, `pin_check`, `member_hash`, `joint_texts` up to its merge, `MemberRefused`,
//! `owner_of`, `owner_deep`, `shown`, `pinned_members`); cli.py (`_run_texts`, `joint`,
//! `reprint_lines`: a reprint's members pinned first, tally 2.1.6); model.py (`lookup` on a
//! member, for the asks its namespace holds).
//! Invariants: every member of a model is found, then every one pinned, in use order, before
//! any loads; a member that is a joint model is a frame of a worklist, never a recursion
//! (R-16); a member's refusal names its aliases from the run's model down; the files given are
//! looked up before the disk's.

use super::check::invariant;
use super::merge::{Member, merge};
use super::steps::load_among;
use super::worlds::{Loaded, Split, declared_asks, load_split};
use crate::api::{Pin, Source};
use crate::base::{Code, Load, Refusal};
use crate::model::{Joint, MemberAsks, NameKind, Reality};
use crate::syntax::{Line, Stop, parse_split, printed_hash, reprint, split_lines};
use core::ops::Bound;
use std::collections::{BTreeMap, BTreeSet};

/// The character that joins a member's alias to a name it declares inside the merge (`a·x`),
/// printed as a dot (refcore's `SEP`). L §6.1.
pub(super) const SEP: char = '\u{b7}';

/// A `use` line of a model's header: the alias, the path and the pin as written.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Use {
    /// The alias.
    alias: String,
    /// The path, relative to the model's file.
    path: String,
    /// The pin.
    pin: String,
}

/// One file of a member: its path as the run names it, its path in the member's tree, and its
/// text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct MemberFile {
    /// The path.
    path: String,
    /// The path under the member's directory, or the file's name for a member that is a file.
    rel: String,
    /// The text.
    text: String,
}

/// A joint model being loaded: its members found and pinned, and those loaded so far.
struct Frame {
    /// Its aliases from the run's model down; empty for the run's own.
    chain: Vec<String>,
    /// Its model file's path.
    jfile: String,
    /// Its own files.
    own: Vec<Source>,
    /// The scenarios of a member that is a joint model, which its base's asks may name.
    worlds: BTreeSet<String>,
    /// The asks a member's scenario files declare.
    scenario_asks: BTreeSet<String>,
    /// Each member's files, in use order (taken as each loads).
    files: Vec<(String, Vec<MemberFile>)>,
    /// The members loaded alone so far, in use order.
    done: Vec<Member>,
    /// What a run may ask of each member loaded.
    asks: BTreeMap<String, MemberAsks>,
}

/// What every frame of a run looks members up in before the disk: the files given, and the
/// pin's form.
struct Pool {
    /// The files the run is given.
    given: Vec<Source>,
    /// How a `use` line's pin is taken.
    pin: Pin,
}

/// A member's next step: loaded alone, or a joint model whose own members load first.
enum Next {
    /// Loaded alone, with its asks.
    Alone(Member, MemberAsks),
    /// A joint model: its frame.
    Nested(Box<Frame>),
}

/// The member a name belongs to, `""` for the joint model's own (refcore's `owner_of`).
pub(super) fn owner_of(name: &str) -> &str {
    name.split_once(SEP).map_or("", |(owner, _)| owner)
}

/// The innermost member a name belongs to: `a` for `a·x`, `a·b` for `a·b·x` (refcore's
/// `owner_deep`). L §6.1.
pub(super) fn owner_deep(name: &str) -> &str {
    name.rsplit_once(SEP).map_or("", |(owner, _)| owner)
}

/// A name as a message prints it, each prefix with its dot (refcore's `shown`). L §6.1.
pub(super) fn shown(name: &str) -> String {
    name.replace(SEP, ".")
}

/// The run's own joint model among its files, when it is one (refcore's `_run_texts`, its
/// `split_reality`, `header_uses` and `header_has_sames`): the one model file no `use` line
/// of a model among them names, whose header has a `use` or a `same` line. L §6.1.
pub(super) fn joint_root(files: &[Split]) -> Option<usize> {
    let pairs: Vec<(&str, &str)> = files
        .iter()
        .map(|f| (f.path.as_str(), f.text.as_str()))
        .collect();
    let mut roots: Vec<(usize, &str)> = Vec::new();
    for at in own_files(&pairs) {
        if let Some((_, text)) = pairs.get(at)
            && is_model(text)
        {
            roots.push((at, text));
        }
    }
    let [(root, text)] = roots.as_slice() else {
        return None;
    };
    is_joint(text).then_some(*root)
}

/// The files that are the reality's own, not inside a path some model's `use` line names
/// (refcore's `split_reality`). L §6.1.
fn own_files(files: &[(&str, &str)]) -> Vec<usize> {
    let mut targets = Vec::new();
    for (path, text) in files {
        if is_model(text) {
            let dir = dirname(path);
            targets.extend(
                header_uses(text)
                    .iter()
                    .map(|u| normpath(&join(dir, &u.path))),
            );
        }
    }
    let mut own = Vec::with_capacity(files.len());
    for (at, (path, _)) in files.iter().enumerate() {
        let n = normpath(path);
        if !targets.iter().any(|t| within(&n, t)) {
            own.push(at);
        }
    }
    own
}

/// Whether a folded path is a target's, or under it.
fn within(path: &str, target: &str) -> bool {
    path == target
        || path
            .strip_prefix(target)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Loads a run whose model is a joint model (refcore's `joint` and `joint_texts`): each member
/// found and pinned, loaded alone, and merged under its prefix; a member that is a joint model
/// first does the same with its own; then the merged lines load as the run's reality, its
/// scenarios as worlds. E §3.1 step 1; L §6.1, §6.4.
///
/// # Errors
///
/// `LOAD-PIN`, `LOAD-DUP` or the merge's refusal of the run's own model, each member's prefix
/// printed with its dot; `member <a>: …` for a member's; `NotBuilt` for a form not built yet.
pub(super) fn load_joint(
    files: &[Split],
    root: usize,
    disk: &BTreeMap<&str, &[u8]>,
    pin: Pin,
) -> Result<Loaded, Stop> {
    let pool = Pool {
        given: files
            .iter()
            .map(|f| Source {
                path: f.path.clone(),
                text: f.text.clone(),
            })
            .collect(),
        pin,
    };
    let pairs: Vec<(&str, &str)> = pool
        .given
        .iter()
        .map(|f| (f.path.as_str(), f.text.as_str()))
        .collect();
    let own: Vec<Source> = own_files(&pairs)
        .into_iter()
        .filter_map(|at| pool.given.get(at).cloned())
        .collect();
    let jfile = files
        .get(root)
        .map(|f| f.path.clone())
        .ok_or_else(|| invariant("the joint model's file past the files".to_owned()))?;
    let first = start(frame(Vec::new(), jfile, own), &pool, disk)?;
    let mut stack = vec![first];
    loop {
        let Some(top) = stack.last_mut() else {
            return Err(invariant("a joint model's worklist ran empty".to_owned()));
        };
        let at = top.done.len();
        if let Some((alias, found)) = top.files.get_mut(at) {
            let mut chain = top.chain.clone();
            chain.push(alias.clone());
            let found = core::mem::take(found);
            match member(alias.clone(), &found, chain, &pool, disk)? {
                Next::Alone(done, asks) => {
                    top.asks.insert(done.alias.clone(), asks);
                    top.done.push(done);
                }
                Next::Nested(frame) => stack.push(*frame),
            }
            continue;
        }
        let Some(frame) = stack.pop() else {
            return Err(invariant("a joint model's worklist ran empty".to_owned()));
        };
        let merged =
            merge(&frame.jfile, &frame.own, frame.done).map_err(|s| tagged(s, &frame.chain))?;
        let joint = Joint {
            members: frame.asks,
        };
        let Some(parent) = stack.last_mut() else {
            return finish(files, merged.files, &joint).map_err(|s| tagged(s, &[]));
        };
        let reality = load_lines(&merged.files, &frame.worlds, Some(&joint))
            .map_err(|s| tagged(s, &frame.chain))?;
        let alias = frame
            .chain
            .last()
            .cloned()
            .ok_or_else(|| invariant("a member's frame without its alias".to_owned()))?;
        let mut asks = ask_ids(&reality);
        asks.extend(frame.scenario_asks);
        parent.asks.insert(
            alias.clone(),
            MemberAsks {
                asks,
                named: asks_named(&reality),
                members: joint.members,
            },
        );
        parent.done.push(Member {
            alias,
            files: merged.files,
            mfile: frame.jfile,
            own_alias: merged.own_alias,
            nested: true,
        });
    }
}

/// The members of each joint model among a reprint's files, found and pinned as `run` finds
/// and pins them, before anything prints (tally 2.1.6, #74; the coordinator's P17, #105): each
/// file named by its path as the run keys it, a member's paths relative to its model's file;
/// a member that is itself a joint model has its own members found and pinned too, in use
/// order, as `run` meets them before it loads each. For each direct member, by its alias, the
/// paths of its files, which a reprint takes as a reality of its own. E §3.1 step 1;
/// INTERFACE §1.
///
/// # Errors
///
/// `LOAD-PIN` for a member absent or changed, `LOAD-DUP` for a `use` twice, `LOAD-HEADER` for
/// a member with no model file or two; `member <a>: …` for a member's other refusal, as `run`
/// prints each.
pub(crate) fn reprint_members(
    given: &[Source],
    disk: &BTreeMap<&str, &[u8]>,
) -> Result<Vec<BTreeSet<String>>, Stop> {
    let pool = Pool {
        given: given.to_vec(),
        pin: Pin::Canonical,
    };
    let mut members: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for source in given {
        if !is_model(&source.text) || header_uses(&source.text).is_empty() {
            continue;
        }
        let model = frame(Vec::new(), source.path.clone(), vec![source.clone()]);
        let found = start(model, &pool, disk)?;
        for (alias, files) in &found.files {
            members.insert(
                alias.clone(),
                files.iter().map(|f| f.path.clone()).collect(),
            );
        }
        pin_nested(found, &pool, disk)?;
    }
    Ok(members.into_values().collect())
}

/// Each member of `first` that is itself a joint model, its own members found and pinned, in
/// use order, depth first, as `run` meets them (refcore's `joint_texts`, recursing), its
/// members loaded by none: a worklist, never a recursion (R-16). E §3.1 step 1; L §6.1.
fn pin_nested(first: Frame, pool: &Pool, disk: &BTreeMap<&str, &[u8]>) -> Result<(), Stop> {
    let mut stack = vec![(first, 0_usize)];
    while let Some((top, at)) = stack.last_mut() {
        let Some((alias, files)) = top.files.get(*at) else {
            stack.pop();
            continue;
        };
        *at = at
            .checked_add(1)
            .ok_or_else(|| invariant("a joint model's members past counting".to_owned()))?;
        let mut chain = top.chain.clone();
        chain.push(alias.clone());
        let (base, model) = member_base(files).map_err(|r| tagged(Stop::Refused(r), &chain))?;
        if is_joint(&model.text) {
            let nested = nested_frame(&base, model, chain, pool, disk)?;
            stack.push((nested, 0));
        }
    }
    Ok(())
}

/// A joint model's frame before its members are found: its aliases, its model file and its
/// own files.
fn frame(chain: Vec<String>, jfile: String, own: Vec<Source>) -> Frame {
    Frame {
        chain,
        jfile,
        own,
        worlds: BTreeSet::new(),
        scenario_asks: BTreeSet::new(),
        files: Vec::new(),
        done: Vec::new(),
        asks: BTreeMap::new(),
    }
}

/// A joint model's frame with its members found and pinned (refcore's `joint_texts`, before
/// its members load): each member's files found, in use order, a use twice refused there;
/// then each one's pin checked. E §3.1 step 1; L §6.1, §8.3.
fn start(mut frame: Frame, pool: &Pool, disk: &BTreeMap<&str, &[u8]>) -> Result<Frame, Stop> {
    let chain = frame.chain.clone();
    let text = frame
        .own
        .iter()
        .find(|f| f.path == frame.jfile)
        .map_or("", |f| f.text.as_str());
    let uses = header_uses(text);
    let dir = dirname(&frame.jfile);
    let reach = disk.contains_key(frame.jfile.as_str()).then_some(disk);
    let mut files: Vec<(String, Vec<MemberFile>)> = Vec::with_capacity(uses.len());
    for u in &uses {
        if files.iter().any(|(alias, _)| *alias == u.alias) {
            let twice = Refusal::new(Code::Load(Load::Dup), format!("use {} twice", u.alias));
            return Err(tagged(Stop::Refused(twice), &chain));
        }
        let place = normpath(&join(dir, &u.path));
        let found = member_texts(u, &place, pool, reach).map_err(|s| tagged(s, &chain))?;
        files.push((u.alias.clone(), found));
    }
    for (u, (alias, found)) in uses.iter().zip(&files) {
        match pin_check(u, found, pool.pin) {
            Ok(()) => {}
            Err(Stop::Refused(r)) if r.code() == Code::Load(Load::Pin) => {
                return Err(tagged(Stop::Refused(r), &chain));
            }
            Err(other) => {
                let mut under = chain.clone();
                under.push(alias.clone());
                return Err(tagged(other, &under));
            }
        }
    }
    frame.files = files;
    Ok(frame)
}

/// A member's step (refcore's `joint_texts`, its loop over members): its base, the files that
/// are no scenario's, with exactly one model file; a joint model's frame, or the base loaded
/// alone among its scenarios. L §6.1, §6.4.
fn member(
    alias: String,
    files: &[MemberFile],
    chain: Vec<String>,
    pool: &Pool,
    disk: &BTreeMap<&str, &[u8]>,
) -> Result<Next, Stop> {
    let mut worlds = BTreeSet::new();
    let mut scenario_asks = BTreeSet::new();
    for file in files.iter().filter(|f| is_scenario(&f.text)) {
        worlds.extend(scenario_id(&file.text));
        scenario_asks.extend(declared_asks(&file.text));
    }
    let (base, model) = member_base(files).map_err(|r| tagged(Stop::Refused(r), &chain))?;
    if is_joint(&model.text) {
        let mut nested = nested_frame(&base, model, chain, pool, disk)?;
        nested.worlds = worlds;
        nested.scenario_asks = scenario_asks;
        return Ok(Next::Nested(Box::new(nested)));
    }
    let reality = alone(&base, &worlds).map_err(|s| tagged(s, &chain))?;
    let mut asks = ask_ids(&reality);
    asks.extend(scenario_asks);
    let mut lines = Vec::with_capacity(base.len());
    for file in &base {
        let split =
            split_lines(&file.text, &file.path).map_err(|r| tagged(Stop::Refused(r), &chain))?;
        lines.push((file.path.clone(), split));
    }
    let done = Member {
        alias,
        files: lines,
        mfile: model.path.clone(),
        own_alias: BTreeMap::new(),
        nested: false,
    };
    Ok(Next::Alone(
        done,
        MemberAsks {
            asks,
            named: asks_named(&reality),
            members: BTreeMap::new(),
        },
    ))
}

/// A member's base, the files that are no scenario's, and its one model file (refcore's
/// `joint_texts`, its loop over members). L §6.1, §6.4.
///
/// # Errors
///
/// `LOAD-HEADER` for a base with no model file, or two.
fn member_base(files: &[MemberFile]) -> Result<(Vec<&MemberFile>, &MemberFile), Refusal> {
    let base: Vec<&MemberFile> = files.iter().filter(|f| !is_scenario(&f.text)).collect();
    let models: Vec<&MemberFile> = base.iter().copied().filter(|f| is_model(&f.text)).collect();
    let [model] = models.as_slice() else {
        let message = if models.is_empty() {
            "no file has a model header"
        } else {
            "two files have a model header"
        };
        return Err(Refusal::new(Code::Load(Load::Header), message.to_owned()));
    };
    Ok((base, *model))
}

/// The frame of a member that is itself a joint model, its own members found and pinned: its
/// model file and its base's files.
fn nested_frame(
    base: &[&MemberFile],
    model: &MemberFile,
    chain: Vec<String>,
    pool: &Pool,
    disk: &BTreeMap<&str, &[u8]>,
) -> Result<Frame, Stop> {
    let own = base
        .iter()
        .map(|f| Source {
            path: f.path.clone(),
            text: f.text.clone(),
        })
        .collect();
    start(frame(chain, model.path.clone(), own), pool, disk)
}

/// A member that is no joint model, loaded alone (refcore's `load_texts`): its files split in
/// the order of their full paths, then loaded in the order of their paths in its tree, among
/// its scenarios. L §6.1, §8.3.
fn alone(files: &[&MemberFile], worlds: &BTreeSet<String>) -> Result<Reality, Stop> {
    let mut order: Vec<&&MemberFile> = files.iter().collect();
    order.sort_by(|a, b| a.path.cmp(&b.path));
    let mut split: BTreeMap<&str, Vec<Line>> = BTreeMap::new();
    for file in order {
        split.insert(&file.path, split_lines(&file.text, &file.path)?);
    }
    let mut lines: Vec<(String, Vec<Line>)> = Vec::with_capacity(files.len());
    for f in files {
        if let Some(l) = split.remove(f.path.as_str()) {
            lines.push((f.path.clone(), l));
        }
    }
    load_lines(&lines, worlds, None)
}

/// Lines loaded as one reality among `worlds` (refcore's `load_lines`). E §3.1.
fn load_lines(
    files: &[(String, Vec<Line>)],
    worlds: &BTreeSet<String>,
    joint: Option<&Joint>,
) -> Result<Reality, Stop> {
    let split: Vec<(&str, &[Line])> = files
        .iter()
        .map(|(path, lines)| (path.as_str(), lines.as_slice()))
        .collect();
    load_among(parse_split(&split)?, worlds, &BTreeMap::new(), joint)
}

/// The run's merged lines loaded (refcore's `joint`, after `joint_texts`): one reality, or
/// worlds when the joint model has scenario files of its own. L §6.1, §6.4.
fn finish(
    given: &[Split],
    merged: Vec<(String, Vec<Line>)>,
    joint: &Joint,
) -> Result<Loaded, Stop> {
    let mut splits = Vec::with_capacity(merged.len());
    for (path, lines) in merged {
        let mut split = Split {
            path,
            text: String::new(),
            lines,
            head: None,
        };
        if let Some(file) = given.iter().find(|f| f.path == split.path) {
            split.text.clone_from(&file.text);
            split.head.clone_from(&file.head);
        }
        splits.push(split);
    }
    load_split(&splits, Some(joint))
}

/// A stop of a joint model's load named for the member it was met in, `chain` its aliases
/// from the run's model down; the run's own refusal with each prefix printed with its dot
/// (refcore's `MemberRefused` and `joint`'s `replace(SEP, '.')`). L §6.1; INTERFACE §2.
fn tagged(stop: Stop, chain: &[String]) -> Stop {
    match stop {
        Stop::Refused(refusal) if refusal.is_member() => Stop::Refused(refusal),
        Stop::Refused(refusal) if chain.is_empty() => Stop::Refused(refusal.shown()),
        Stop::Refused(refusal) => Stop::Refused(refusal.of_member(chain.join("."))),
        other @ (Stop::NotBuilt(_) | Stop::Deep) => other,
    }
}

/// The asks a member's base answers by their ids: those not refused alone (refcore's `asks`
/// after `asks_alone` is taken out). E §11.
fn ask_ids(m: &Reality) -> BTreeSet<String> {
    m.asks
        .iter()
        .filter(|ask| !m.asks_alone.contains_key(&ask.id))
        .map(|ask| ask.id.clone())
        .collect()
}

/// The asks a member's namespace holds, refused alone or not: what refcore's `lookup` on the
/// member finds as an ask, read or called by the joint model as `a.q`. L §6.1, §5.18.
fn asks_named(m: &Reality) -> BTreeSet<String> {
    let mut named = BTreeSet::new();
    for (id, kind) in &m.names {
        if *kind == NameKind::Ask {
            named.insert(id.clone());
        }
    }
    named
}

/// A member's files (refcore's `member_texts`): the file given at its path, else every file
/// given under it, then, where the disk is reached (the joint model's file is on it), every
/// `.tally` file the disk holds under it, in the order of their full paths, or the file the
/// disk holds at it; in the order of their paths in the member's tree. E §3.1 step 1; L §6.1.
///
/// # Errors
///
/// `LOAD-PIN … (no such file)` when none is found; `LOAD-PARSE` for a file of the disk that
/// isn't UTF-8.
fn member_texts(
    u: &Use,
    place: &str,
    pool: &Pool,
    disk: Option<&BTreeMap<&str, &[u8]>>,
) -> Result<Vec<MemberFile>, Stop> {
    if let Some(file) = pool.given.iter().find(|f| normpath(&f.path) == place) {
        return Ok(vec![MemberFile {
            path: file.path.clone(),
            rel: basename(&file.path).to_owned(),
            text: file.text.clone(),
        }]);
    }
    let under = format!("{place}/");
    let mut got: BTreeMap<String, (String, String)> = BTreeMap::new();
    for file in &pool.given {
        let n = normpath(&file.path);
        if n.starts_with(&under) {
            got.insert(n, (file.path.clone(), file.text.clone()));
        }
    }
    let from = (Bound::Included(under.as_str()), Bound::Unbounded);
    if let Some(dir) = disk.filter(|d| holds_under(d, &under)) {
        for (key, bytes) in dir.range::<str, _>(from) {
            if !key.starts_with(&under) {
                break;
            }
            let n = normpath(key);
            if !is_tally(key) || got.contains_key(&n) {
                continue;
            }
            let text = String::from_utf8(bytes.to_vec())
                .map_err(|_| Stop::Refused(Refusal::not_utf8(key)))?;
            got.insert(n, ((*key).to_owned(), text));
        }
    } else if got.is_empty()
        && let Some(bytes) = disk.and_then(|d| d.get(place))
    {
        let text = String::from_utf8(bytes.to_vec())
            .map_err(|_| Stop::Refused(Refusal::not_utf8(place)))?;
        return Ok(vec![MemberFile {
            path: place.to_owned(),
            rel: basename(place).to_owned(),
            text,
        }]);
    }
    if got.is_empty() {
        let message = format!(
            "{} doesn't match its sha256 in use {} (no such file)",
            u.path, u.alias
        );
        return Err(Stop::Refused(Refusal::new(Code::Load(Load::Pin), message)));
    }
    let mut out: Vec<MemberFile> = got
        .into_iter()
        .map(|(n, (path, text))| MemberFile {
            rel: n
                .strip_prefix(under.as_str())
                .unwrap_or(n.as_str())
                .to_owned(),
            path,
            text,
        })
        .collect();
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(out)
}

/// Whether the disk holds a file under a directory, `under` ending with its `/`.
fn holds_under(disk: &BTreeMap<&str, &[u8]>, under: &str) -> bool {
    disk.range::<str, _>((Bound::Included(under), Bound::Unbounded))
        .next()
        .is_some_and(|(key, _)| key.starts_with(under))
}

/// A member's pin against its files (refcore's `pin_check` and `member_hash`): 64 lower-case
/// hexadecimal digits, equal to the sha256 of its files' canonical texts, or of their bytes
/// with `--pin=bytes`, taken in the order of their paths in its tree. E §3.1 step 1; L §8.3.
///
/// # Errors
///
/// `LOAD-PIN <path> doesn't match its sha256 in use <a>`; a reprint's refusal.
fn pin_check(u: &Use, files: &[MemberFile], pin: Pin) -> Result<(), Stop> {
    let wrong = || {
        let message = format!("{} doesn't match its sha256 in use {}", u.path, u.alias);
        Stop::Refused(Refusal::new(Code::Load(Load::Pin), message))
    };
    let hex = u.pin.len() == 64
        && u.pin
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if !hex {
        return Err(wrong());
    }
    let mut sorted: Vec<&MemberFile> = files.iter().collect();
    sorted.sort_by(|a, b| a.rel.cmp(&b.rel));
    let got = match pin {
        Pin::Bytes => {
            let texts: Vec<(String, String)> = sorted
                .iter()
                .map(|f| (f.rel.clone(), f.text.clone()))
                .collect();
            printed_hash(&texts)
        }
        Pin::Canonical => {
            let sources: Vec<Source> = sorted
                .iter()
                .map(|f| Source {
                    path: f.rel.clone(),
                    text: f.text.clone(),
                })
                .collect();
            printed_hash(&reprint(&sources)?)
        }
    };
    if got != u.pin {
        return Err(wrong());
    }
    Ok(())
}

/// Whether a file's text opens with a model header (refcore's `is_model`).
fn is_model(text: &str) -> bool {
    text.split('\n')
        .next()
        .is_some_and(|line| line.starts_with("model "))
}

/// Whether a file's text opens with a scenario header (refcore's `is_scenario`).
fn is_scenario(text: &str) -> bool {
    text.split('\n')
        .next()
        .is_some_and(|line| line.starts_with("scenario "))
}

/// Whether a model file's header has a `use` or a `same` line: a joint model (refcore's
/// `header_uses` and `header_has_sames`). L §6.1.
fn is_joint(text: &str) -> bool {
    !header_uses(text).is_empty() || header_has_sames(text)
}

/// The scenario id a scenario file's text opens with (refcore's `scenario ([a-z][a-z0-9_]*)`).
fn scenario_id(text: &str) -> Option<String> {
    let after = text.strip_prefix("scenario ")?;
    after.chars().next().filter(char::is_ascii_lowercase)?;
    let end = after
        .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
        .unwrap_or(after.len());
    after.get(..end).map(str::to_owned)
}

/// A model file's header lines, as text: the indented lines after its first (refcore's loop
/// over `lines[1:]` up to the first line that isn't indented).
fn header_lines(text: &str) -> Vec<&str> {
    let mut lines = text.split('\n');
    if !lines.next().is_some_and(|line| line.starts_with("model ")) {
        return Vec::new();
    }
    lines
        .take_while(|line| line.starts_with([' ', '\t']))
        .collect()
}

/// The `use a = "path" sha256 "…"` lines of a model file's header (refcore's `header_uses`).
/// L §6.1.
fn header_uses(text: &str) -> Vec<Use> {
    header_lines(text)
        .into_iter()
        .filter_map(use_line)
        .collect()
}

/// Whether a model file's header has a `same` line (refcore's `header_has_sames`). L §6.1.
fn header_has_sames(text: &str) -> bool {
    header_lines(text).into_iter().any(is_same_line)
}

/// Whether a header line is a `same` line: its word, then white space.
fn is_same_line(line: &str) -> bool {
    line.trim_start()
        .strip_prefix("same")
        .is_some_and(|rest| rest.starts_with(char::is_whitespace))
}

/// One header line read as `use <alias> = "<path>" sha256 "<pin>"`, white space around each
/// part as refcore's pattern allows it.
fn use_line(line: &str) -> Option<Use> {
    let rest = line.trim_start().strip_prefix("use")?;
    let rest = rest.strip_prefix(char::is_whitespace)?.trim_start();
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    let (alias, rest) = rest.split_at_checked(end)?;
    if alias.is_empty() {
        return None;
    }
    let rest = rest.trim_start().strip_prefix('=')?.trim_start();
    let (path, rest) = quoted(rest)?;
    let rest = rest.strip_prefix(char::is_whitespace)?.trim_start();
    let rest = rest.strip_prefix("sha256")?;
    let rest = rest.strip_prefix(char::is_whitespace)?.trim_start();
    let (pin, rest) = quoted(rest)?;
    rest.trim().is_empty().then(|| Use {
        alias: alias.to_owned(),
        path: path.to_owned(),
        pin: pin.to_owned(),
    })
}

/// A `"…"` at the start of `text`, no quote inside: its inside and what follows it.
fn quoted(text: &str) -> Option<(&str, &str)> {
    let inner = text.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some((inner.get(..end)?, inner.get(end.checked_add(1)?..)?))
}

/// The paths of the members a file's `use` lines name, found relative to it: what the binary
/// reads beside the paths it is given (INTERFACE §2, refcore's `use_member_files`).
#[must_use]
pub fn member_paths(path: &str, text: &str) -> Vec<String> {
    let dir = dirname(path);
    header_uses(text)
        .iter()
        .map(|u| normpath(&join(dir, &u.path)))
        .collect()
}

/// Whether a path names a reality's file: it ends `.tally` (refcore's `collect`).
fn is_tally(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "tally")
}

/// A path's directory, as Python's `os.path.dirname` gives it.
fn dirname(path: &str) -> &str {
    let Some((_, name)) = path.rsplit_once('/') else {
        return "";
    };
    let Some(head) = path.strip_suffix(name) else {
        return "";
    };
    let trimmed = head.trim_end_matches('/');
    if trimmed.is_empty() { head } else { trimmed }
}

/// A path's last part, as Python's `os.path.basename` gives it.
fn basename(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

/// Two paths joined, as Python's `os.path.join` joins them.
fn join(dir: &str, path: &str) -> String {
    if path.starts_with('/') || dir.is_empty() {
        return path.to_owned();
    }
    if dir.ends_with('/') {
        format!("{dir}{path}")
    } else {
        format!("{dir}/{path}")
    }
}

/// A path with `.`, `..` and repeated separators folded, as Python's `os.path.normpath`
/// folds them on a POSIX system.
fn normpath(path: &str) -> String {
    if path.is_empty() {
        return ".".to_owned();
    }
    let slashes = if path.starts_with("//") && !path.starts_with("///") {
        "//"
    } else if path.starts_with('/') {
        "/"
    } else {
        ""
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        let up = part == "..";
        let keep = !up || (slashes.is_empty() && parts.is_empty()) || parts.last() == Some(&"..");
        if keep {
            parts.push(part);
        } else if !parts.is_empty() {
            parts.pop();
        }
    }
    let joined = format!("{slashes}{}", parts.join("/"));
    if joined.is_empty() {
        ".".to_owned()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::{dirname, join, normpath, owner_deep, owner_of, use_line};

    #[test]
    fn paths_fold_as_python_folds_them() {
        assert_eq!(normpath(".."), "..", "a parent kept");
        assert_eq!(normpath("a/b/.."), "a", "a parent folded");
        assert_eq!(normpath("E2/.."), ".", "folded to nothing");
        assert_eq!(normpath("a/./b//c/"), "a/b/c", "dots and slashes");
        assert_eq!(normpath("/.."), "/", "above the root");
        assert_eq!(normpath(""), ".", "nothing");
        assert_eq!(dirname("E2/family.tally"), "E2", "a directory");
        assert_eq!(dirname("family.tally"), "", "no directory");
        assert_eq!(join("", "A"), "A", "no directory to join");
        assert_eq!(join("E2", "A"), "E2/A", "joined");
        assert_eq!(join("E2", "/A"), "/A", "an absolute path");
    }

    #[test]
    fn a_use_line_reads_as_refcore_reads_it() {
        let u = use_line("  use a = \"a/a.tally\" sha256 \"0f\"");
        assert_eq!(
            u.map(|u| (u.alias, u.path, u.pin)),
            Some(("a".to_owned(), "a/a.tally".to_owned(), "0f".to_owned())),
            "a use line"
        );
        assert!(
            use_line("  use a = \"x\" sha256 \"0f\" more").is_none(),
            "words after"
        );
        assert!(
            use_line("  usea = \"x\" sha256 \"0f\"").is_none(),
            "no space"
        );
        assert_eq!(owner_of("a\u{b7}b\u{b7}x"), "a", "the outer member");
        assert_eq!(
            owner_deep("a\u{b7}b\u{b7}x"),
            "a\u{b7}b",
            "the inner member"
        );
        assert_eq!(owner_of("x"), "", "the joint model's own");
    }
}
