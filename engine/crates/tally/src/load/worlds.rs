//! Worlds: the base loaded alone, and each scenario with the base and its stack, so a scenario's
//! refusal refuses only that scenario.
//! Spec: L §6.4 (scenarios: `scenario s over b`, a stack over the model at most 4 deep; an id a
//! scenario declares, read outside it), §6.1 (one model; a joint model, M0's ninth group), §8.3
//! (which refusal prints: every file split, by full path, before any is parsed); E §3.1 step 1;
//! E §11 (`REQ-ASK`: an ask a refused scenario's file declares exists).
//! Twin: engine/tools/refcore/cli.py (`_run_texts`, `run_worlds` up to its asks, `declared_ids`,
//! `declared_asks`); loader.py (`scenario_header`, `load_texts`); joint.py (`header_uses`,
//! `header_has_sames`). The base's own refusals of a scenario's ids, a key literal's read and
//! a rule's `v`, have no twin: refcore refuses them at run or as `LOAD-NAME` (issue #75).
//! Invariants: every file is split, in the order given, before any is parsed; the base loads
//! first, and its refusal is the run's; each scenario loads with the base's files, then its
//! stack from the model up; a world that stops `NotBuilt` stops the run.

use super::steps::load_among;
use crate::api::Source;
use crate::base::{Code, Load, Place, Refusal};
use crate::model::Reality;
use crate::syntax::{Line, Stop, cannot, parse_split, split_lines};
use std::collections::{BTreeMap, BTreeSet};

/// The deepest stack of scenarios over the model (L §6.4).
const MAX_STACK: usize = 4;

/// The words that open a section whose id a refusal of the base may name (refcore's
/// `declared_ids`).
const SECTION_WORDS: [&str; 12] = [
    "def", "input", "band", "fill", "move", "place", "kind", "index", "unit", "axis", "source",
    "version",
];

/// A scenario file's header line: its id, the world it is over, and its line (refcore's
/// `scenario_header`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Head {
    /// The scenario's id.
    id: String,
    /// The world it is over: the model, or another scenario.
    over: String,
    /// The header's line.
    line: u32,
}

/// One file of a run, split into lines, with its scenario header when it is a scenario's file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split {
    /// The path, as the run names it.
    path: String,
    /// The text, for the ids and asks a refusal names (refcore reads them from it).
    text: String,
    /// The lines.
    lines: Vec<Line>,
    /// The scenario header, or `None` for a model or part file.
    head: Option<Head>,
}

/// What a run's files load into: one reality, or a base and its scenarios' worlds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loaded {
    /// A reality with no scenario file.
    One(Box<Reality>),
    /// A reality with scenarios, each loaded as its own world.
    Worlds(Box<Worlds>),
}

/// A reality with scenarios, loaded as worlds (refcore's `run_worlds`, its loads). L §6.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worlds {
    /// The base: the files that are no scenario's, loaded alone.
    pub base: Reality,
    /// Each scenario that loaded, with the base and the scenarios under it.
    pub loaded: BTreeMap<String, Reality>,
    /// Each scenario whose load was refused, and its refusal.
    pub refused: BTreeMap<String, Refusal>,
    /// The asks the refused scenarios' files declare: they exist, refused with their scenario
    /// (E §11 `REQ-ASK`).
    pub refused_asks: BTreeSet<String>,
}

/// Splits each file in the order given, the read step's by full path, and reads its scenario
/// header (refcore's `_run_texts`, its `heads`): a split refusal or a malformed scenario header,
/// whichever file comes first. L §8.3, §6.4.
///
/// # Errors
///
/// `LOAD-PARSE` for a file that can't be split, or for a scenario header that isn't
/// `scenario ID over ID`.
pub fn split_files(sources: Vec<Source>) -> Result<Vec<Split>, Refusal> {
    let mut out = Vec::with_capacity(sources.len());
    for source in sources {
        let lines = split_lines(&source.text, &source.path)?;
        let head = scenario_head(&lines, &source.path)?;
        out.push(Split {
            path: source.path,
            text: source.text,
            lines,
            head,
        });
    }
    Ok(out)
}

/// A file's scenario header, or `None` when its first line isn't one (refcore's
/// `scenario_header`). L §6.4.
fn scenario_head(lines: &[Line], path: &str) -> Result<Option<Head>, Refusal> {
    let Some(first) = lines.first() else {
        return Ok(None);
    };
    match first.toks.as_slice() {
        [] => Ok(None),
        [word, ..] if word.text != "scenario" => Ok(None),
        [_, id, over, base, ..] if over.text == "over" => Ok(Some(Head {
            id: id.text.clone(),
            over: base.text.clone(),
            line: first.number,
        })),
        [..] => Err(cannot(
            "scenario ID over ID",
            Place::Line(path.to_owned(), first.number),
        )),
    }
}

/// Loads a run's split files (refcore's `_run_texts`): one reality, or its worlds when a file
/// is a scenario's. L §6.1, §6.4.
///
/// # Errors
///
/// The base's load refusal, or `NotBuilt` for a joint model (M0's ninth group) and for a form
/// a world's load doesn't build yet.
pub fn load_run(files: &[Split]) -> Result<Loaded, Stop> {
    if files.iter().any(|file| joint_header(&file.text)) {
        return Err(Stop::NotBuilt("joint models"));
    }
    if files.iter().all(|file| file.head.is_none()) {
        let all: Vec<&Split> = files.iter().collect();
        let none = (BTreeSet::new(), BTreeMap::new());
        return Ok(Loaded::One(Box::new(load_world(&all, &none)?)));
    }
    Ok(Loaded::Worlds(Box::new(load_worlds(files)?)))
}

/// Whether a model file's header has a `use` or a `same` line: a joint model (refcore's
/// `header_uses` and `header_has_sames`, any such line). L §6.1.
fn joint_header(text: &str) -> bool {
    let mut lines = text.split('\n');
    if !lines.next().is_some_and(|line| line.starts_with("model ")) {
        return false;
    }
    lines
        .take_while(|line| line.starts_with([' ', '\t']))
        .any(|line| matches!(line.split_whitespace().next(), Some("use" | "same")))
}

/// The run's worlds an ask may name, and the ids the scenarios' files declare, each with its
/// scenario (for the base's load; empty for any other).
type Among = (BTreeSet<String>, BTreeMap<String, String>);

/// One world's reality: its files parsed together, then loaded among the run's worlds
/// (refcore's `load_texts`). E §3.1.
fn load_world(files: &[&Split], among: &Among) -> Result<Reality, Stop> {
    let split: Vec<(&str, &[Line])> = files
        .iter()
        .map(|file| (file.path.as_str(), file.lines.as_slice()))
        .collect();
    load_among(parse_split(&split)?, &among.0, &among.1)
}

/// The base alone, then each scenario, by id, with the base and its stack (refcore's
/// `run_worlds`, its loads). L §6.4, §8.3.
fn load_worlds(files: &[Split]) -> Result<Worlds, Stop> {
    let base: Vec<&Split> = files.iter().filter(|file| file.head.is_none()).collect();
    let mut scenarios: BTreeMap<&str, (&Split, &Head)> = BTreeMap::new();
    let mut twice = BTreeSet::new();
    for file in files {
        if let Some(head) = &file.head
            && scenarios.insert(&head.id, (file, head)).is_some()
        {
            twice.insert(head.id.as_str());
        }
    }
    let run: BTreeSet<String> = scenarios.keys().map(|id| (*id).to_owned()).collect();
    let mut elsewhere = BTreeMap::new();
    for (id, (file, _)) in scenarios.iter().rev() {
        for name in section_ids(&file.text) {
            elsewhere.insert(name.to_owned(), (*id).to_owned());
        }
    }
    let mut among = (run, elsewhere);
    let alone = match load_world(&base, &among) {
        Err(Stop::Refused(refusal)) => {
            return Err(Stop::Refused(read_outside(refusal, &among.1)));
        }
        other => other?,
    };
    among.1.clear();
    let model = alone.header.id.clone();
    let mut worlds = Worlds {
        base: alone,
        loaded: BTreeMap::new(),
        refused: BTreeMap::new(),
        refused_asks: BTreeSet::new(),
    };
    for (id, (file, head)) in &scenarios {
        let loaded = match stack(file, head, &scenarios, &twice, &model) {
            Err(refusal) => Err(Stop::Refused(refusal)),
            Ok(chain) => {
                let mut world = base.clone();
                for over in chain.iter().rev() {
                    let (under, _) = scenarios.get(over.as_str()).ok_or_else(no_stack)?;
                    world.push(under);
                }
                load_world(&world, &among)
            }
        };
        match loaded {
            Ok(reality) => {
                worlds.loaded.insert((*id).to_owned(), reality);
            }
            Err(Stop::Refused(refusal)) => {
                worlds.refused_asks.extend(declared_asks(&file.text));
                worlds.refused.insert((*id).to_owned(), refusal);
            }
            Err(stop @ (Stop::NotBuilt(_) | Stop::Deep)) => return Err(stop),
        }
    }
    Ok(worlds)
}

/// The stack of the scenario whose file and header are given: its id, then each scenario it is
/// over, down to the one over the model (refcore's `run_worlds`, its chain). L §6.4.
///
/// # Errors
///
/// `LOAD-SCENARIO` at the scenario's header when its stack names a world that is neither the
/// model nor a scenario, cycles or is deeper than 4, or holds a scenario declared twice.
fn stack(
    file: &Split,
    head: &Head,
    scenarios: &BTreeMap<&str, (&Split, &Head)>,
    twice: &BTreeSet<&str>,
    model: &str,
) -> Result<Vec<String>, Refusal> {
    let deep = format!(
        "over cycle or deeper than {MAX_STACK} (scenario {})",
        head.id
    );
    let mut chain: Vec<String> = Vec::new();
    let mut world = head.id.as_str();
    while world != model {
        if chain.iter().any(|seen| seen == world) {
            return Err(at_head(file, head, deep));
        }
        let Some((_, next)) = scenarios.get(world) else {
            let under = chain.last().map_or(head.id.as_str(), String::as_str);
            let message =
                format!("scenario {under} is over {world}, neither the model nor a scenario");
            return Err(at_head(file, head, message));
        };
        chain.push(world.to_owned());
        world = &next.over;
    }
    if chain.len() > MAX_STACK {
        return Err(at_head(file, head, deep));
    }
    if let Some(dup) = chain.iter().find(|id| twice.contains(id.as_str())) {
        return Err(at_head(
            file,
            head,
            format!("scenario {dup} declared twice"),
        ));
    }
    Ok(chain)
}

/// `LOAD-SCENARIO` at a scenario's header line. L §6.4, §8.3.
fn at_head(file: &Split, head: &Head, message: String) -> Refusal {
    Refusal::at(
        Code::Load(Load::Scenario),
        message,
        Place::Line(file.path.clone(), head.line),
    )
}

/// A scenario of a stack that isn't among the run's scenarios: `stack` keeps only theirs.
fn no_stack() -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "a scenario's stack names a scenario no file declares".to_owned(),
    ))
}

/// A refusal of the base that names an id no base file declares, renamed when a scenario's
/// file declares it, the first such scenario by id (refcore's `run_worlds`): `LOAD-SCENARIO
/// <x> of scenario <s> is read outside it`, at the same place. L §6.4.
fn read_outside(refusal: Refusal, elsewhere: &BTreeMap<String, String>) -> Refusal {
    let message = refusal.message();
    let name = if refusal.code() == Code::Load(Load::Name) {
        message.strip_prefix("unknown id ").map(first_word)
    } else if refusal.code() == Code::Load(Load::Key) {
        message
            .strip_prefix("ambiguous key ")
            .map(|rest| first_word(rest).trim_end_matches(':'))
    } else {
        None
    };
    let Some(name) = name.map(str::to_owned) else {
        return refusal;
    };
    match elsewhere.get(&name) {
        Some(id) => refusal.renamed(
            Code::Load(Load::Scenario),
            format!("{name} of scenario {id} is read outside it"),
        ),
        None => refusal,
    }
}

/// The text up to its first space.
fn first_word(text: &str) -> &str {
    text.split_once(' ').map_or(text, |(word, _)| word)
}

/// The section ids a file's text declares: each line at the left margin whose first word
/// opens a section gives its second, up to a `[`, `:` or `(` (refcore's `declared_ids`).
fn section_ids(text: &str) -> Vec<&str> {
    text.split('\n')
        .filter(|line| !line.starts_with([' ', '\t']))
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let (Some(word), Some(id)) = (words.next(), words.next()) else {
                return None;
            };
            SECTION_WORDS.contains(&word).then_some(section_name(id))
        })
        .collect()
}

/// A rule's or a `takes`' `v` that names no version of the world: `LOAD-SCENARIO <item> of
/// the base names <v> of scenario <s>` when a scenario's file declares it, since the base is
/// whole without its scenarios (L §6.4, a rule's `v` included; issue #75, where refcore
/// prints the `LOAD-NAME`), else `LOAD-NAME` with `message`. L §8.3.
pub(super) fn no_version(m: &Reality, item: &str, v: &str, message: String) -> (Load, String) {
    match m.elsewhere.get(v) {
        Some(scenario) => (
            Load::Scenario,
            format!("{item} of the base names {v} of scenario {scenario}"),
        ),
        None => (Load::Name, message),
    }
}

/// A name the world doesn't declare, read where it would be a key literal, that a scenario's
/// file declares and no key of the world holds: `LOAD-SCENARIO <x> of scenario <s> is read
/// outside it` at load, since a read of an item a scenario's file declares stands only inside
/// that scenario (L §6.4; issue #75, where refcore refuses it at run as an ambiguous key or
/// as a key literal in arithmetic). L §8.3.
///
/// # Errors
///
/// That refusal.
pub(super) fn read_elsewhere(m: &Reality, name: &str) -> Result<(), Stop> {
    let Some(scenario) = m.elsewhere.get(name) else {
        return Ok(());
    };
    if holds_key(m, name) {
        return Ok(());
    }
    Err(Stop::Refused(Refusal::new(
        Code::Load(Load::Scenario),
        format!("{name} of scenario {scenario} is read outside it"),
    )))
}

/// An id as a section line writes it, up to its keys, its type or its parameters.
fn section_name(word: &str) -> &str {
    let cut = word.find(['[', ':', '(']).unwrap_or(word.len());
    word.get(..cut).unwrap_or(word)
}

/// Whether `name` is a key of an index, an option of an axis, or a thing of the world.
fn holds_key(m: &Reality, name: &str) -> bool {
    for (_, index) in m.indexes.entries() {
        if index.texts.iter().any(|(text, _)| text == name) {
            return true;
        }
    }
    for (_, axis) in m.axes.entries() {
        if axis.options.iter().any(|(option, _)| option == name) {
            return true;
        }
    }
    m.things.contains(name)
}

/// The ids of the asks a file declares, `ask <id>:` at the start of a line, read from its text,
/// so a scenario file whose load was refused still declares them (refcore's `declared_asks`).
/// E §11 `REQ-ASK`.
fn declared_asks(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    loop {
        if let Some(id) = ask_at(rest) {
            out.insert(id);
        }
        match rest.split_once('\n') {
            Some((_, next)) => rest = next,
            None => return out,
        }
    }
}

/// The ask a line opens, `ask <id>` with `:` after it and any white space, the id a small
/// letter, then small letters, digits and `_`.
fn ask_at(text: &str) -> Option<String> {
    let after = text.strip_prefix("ask ")?;
    after.chars().next().filter(char::is_ascii_lowercase)?;
    let end = after
        .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
        .unwrap_or(after.len());
    let (id, tail) = after.split_at_checked(end)?;
    tail.trim_start().starts_with(':').then(|| id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Loaded, declared_asks, load_run, section_ids, split_files};
    use crate::api::Source;
    use crate::base::Refusal;
    use crate::syntax::Stop;

    const MODEL: &str = "model m\n  tally 2\n  on 2026-03-01\n  start 2026-01-01\n  end 2027-12-31\n\nsource card \"a card\"\n\nunit PLN: currency minor 0.01\n\ninput amt: money PLN\n\nversion v\n  cite card\n  governs always\n  known 2026-01-01\n  amt = 100.00 PLN \"the card\"\n";

    fn scenario(id: &str, over: &str) -> String {
        format!(
            "scenario {id} over {over}\n  tally 2\n  from 2027-01-01\n\nversion {id}_v\n  cite card\n  governs 2027..\n  known 2026-02-01\n  replace amt = 150.00 PLN \"a raise\"\n\nask q_{id}: amt in {id} law 2027-06-01\n"
        )
    }

    fn sources(files: &[(&str, &str)]) -> Vec<Source> {
        files
            .iter()
            .map(|(path, text)| Source {
                path: (*path).to_owned(),
                text: (*text).to_owned(),
            })
            .collect()
    }

    fn loaded(files: &[(&str, &str)]) -> Result<Loaded, Stop> {
        let split = split_files(sources(files)).map_err(Stop::Refused)?;
        load_run(&split)
    }

    fn refusal(files: &[(&str, &str)]) -> String {
        match loaded(files) {
            Err(Stop::Refused(r)) => r.text(),
            Ok(_) | Err(Stop::NotBuilt(_) | Stop::Deep) => String::new(),
        }
    }

    #[test]
    fn a_scenario_header_is_read_before_any_file_is_parsed() {
        assert_eq!(
            refusal(&[
                ("a.tally", "scenario s\n  tally 2\n"),
                ("b.tally", "model\tm\n")
            ]),
            "LOAD-PARSE cannot parse: scenario ID over ID (a.tally:1)",
            "a's header before b's tab"
        );
        assert_eq!(
            refusal(&[("s.tally", &scenario("s", "m"))]),
            "LOAD-HEADER 0 files have a model header; exactly one does",
            "a scenario alone: its base has no model"
        );
    }

    #[test]
    fn each_scenario_loads_with_its_stack_or_is_refused_alone() {
        let lower = scenario("lo", "m");
        let upper = scenario("up", "lo");
        let lost = scenario("x", "nowhere");
        let Ok(Loaded::Worlds(worlds)) = loaded(&[
            ("m.tally", MODEL),
            ("s1.tally", &lower),
            ("s2.tally", &upper),
            ("s3.tally", &lost),
        ]) else {
            panic!("the base loads");
        };
        let ids: Vec<&String> = worlds.loaded.keys().collect();
        assert_eq!(ids, ["lo", "up"], "both of the stack load");
        let up = worlds.loaded.get("up").expect("up loads");
        assert_eq!(up.scenarios.of("up_v"), Some("up"), "up's version is up's");
        assert_eq!(up.scenarios.of("lo_v"), Some("lo"), "lo's under it");
        assert_eq!(up.scenarios.of("v"), None, "the base's");
        let refused: Vec<String> = worlds.refused.values().map(Refusal::text).collect();
        assert_eq!(
            refused,
            [
                "LOAD-SCENARIO scenario x is over nowhere, neither the model nor a scenario (s3.tally:1)"
            ],
            "x alone is refused"
        );
        assert!(worlds.refused_asks.contains("q_x"), "x's ask still exists");
    }

    #[test]
    fn a_stack_that_cycles_or_a_scenario_declared_twice_is_refused() {
        let a = scenario("a", "b");
        let b = scenario("b", "a");
        let Ok(Loaded::Worlds(worlds)) =
            loaded(&[("m.tally", MODEL), ("a.tally", &a), ("b.tally", &b)])
        else {
            panic!("the base loads");
        };
        let refused: Vec<String> = worlds.refused.values().map(Refusal::text).collect();
        assert_eq!(
            refused,
            [
                "LOAD-SCENARIO over cycle or deeper than 4 (scenario a) (a.tally:1)",
                "LOAD-SCENARIO over cycle or deeper than 4 (scenario b) (b.tally:1)"
            ],
            "a cycle"
        );
        let one = scenario("s", "m");
        let Ok(Loaded::Worlds(twice)) =
            loaded(&[("m.tally", MODEL), ("a.tally", &one), ("b.tally", &one)])
        else {
            panic!("the base loads");
        };
        let again: Vec<String> = twice.refused.values().map(Refusal::text).collect();
        assert_eq!(
            again,
            ["LOAD-SCENARIO scenario s declared twice (b.tally:1)"],
            "the later file names it"
        );
    }

    #[test]
    fn ids_and_asks_are_read_from_the_text() {
        let text = "def total[k in ks]: num = 1\n  def inner: num\nask q1 : x\nask Q: x\nask q2:\n";
        assert_eq!(
            section_ids(text),
            ["total"],
            "keys cut, the indented line left out"
        );
        let asks: Vec<String> = declared_asks(text).into_iter().collect();
        assert_eq!(
            asks,
            ["q1", "q2"],
            "small letters, any space before the colon"
        );
    }
}
