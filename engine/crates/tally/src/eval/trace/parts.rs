//! What a read that counts a move's instances keeps of a traced plan pass, by its key and span.
//! Spec: L §8.2 One read set (a read's set is, for each due instance at its key and in its
//! span, what decided whether it counts and how much; the day that put one outside the
//! span, never its `when` or amount; nothing at another key), §5.18 Explanation order;
//! E §10 (`explain` prints that set), §3.2; picks P20, P22.
//! Twin: none: refcore traces a plan pass whole, every key's instance and every part (#255).
//! Invariants: a part is a run of the open node's children, from one mark to the next, the
//! marks taken while the pass computes the instances; a part set aside is kept by instance
//! and cut, and only a read that counts that instance takes it back, where it reads, so each
//! instance's reads print once per explain; a cell traced in a part set aside is met again as
//! a dropped one (`trace_drop`'s rule), never traced twice; a pass whose instances a later
//! version answered keeps every part where it printed.

use std::collections::BTreeMap;

use super::{Engine, Snap, Trace};
use crate::time::Day;

/// Where a mark of a traced plan pass stands in an instance's reads (E §10; P20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    /// Before its reads.
    Begin,
    /// After its day's reads (its law date and `on`), before its body's.
    Day,
    /// After its reads.
    End,
}

/// What a read that counts a move's instances keeps of one instance's traced reads (L §8.2
/// One read set; P20, P22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keep {
    /// Every read: one at the read's key and in its span, or whose day is a hole.
    Whole,
    /// Its day's reads: one at the read's key outside its span.
    Day,
    /// None: one at another key.
    Nothing,
}

/// The marks of one traced plan pass: each instance's, by its id, in the order taken.
pub(crate) type Marks = Vec<(String, Part, Snap)>;

/// The parts set aside, by instance, cut and whether it is the body's (else the day's).
pub(crate) type Stash = BTreeMap<(String, Day, bool), Vec<usize>>;

/// One instance's parts: its day's children from `begin` to `day`, its body's from `day` to
/// `end`, under node `parent`.
struct Seg {
    id: String,
    parent: usize,
    begin: Snap,
    day: Snap,
    end: Snap,
}

impl Engine {
    /// Opens the marks of a plan pass traced now; returns the marks of the pass it opens
    /// inside, for [`Engine::trace_marks_close`] (E §10).
    pub(crate) fn trace_marks_open(&mut self) -> Option<Marks> {
        let trace = self.trace.as_mut()?;
        trace.marks.replace(Vec::new())
    }

    /// Closes the marks of the pass of move `move_id` at cut `cut`, kept for the read that
    /// asked for it ([`Engine::trace_keep`]); the marks of the pass it opened inside, `outer`,
    /// open again.
    pub(crate) fn trace_marks_close(&mut self, (move_id, cut): (&str, Day), outer: Option<Marks>) {
        if let Some(trace) = self.trace.as_mut() {
            let mine = core::mem::replace(&mut trace.marks, outer);
            trace.fresh = mine.map(|marks| (move_id.to_owned(), cut, marks));
        }
    }

    /// Forgets the marks of the last pass traced: a read about to ask for a move's
    /// instances keeps only a pass it asked for itself; a pass a later version answered
    /// keeps its reads where they printed.
    pub(crate) fn trace_marks_forget(&mut self) {
        if let Some(trace) = self.trace.as_mut() {
            trace.fresh = None;
        }
    }

    /// Whether a plan pass traced now takes marks.
    pub(crate) fn trace_marking(&self) -> bool {
        self.tracing() && self.trace.as_ref().is_some_and(|t| t.marks.is_some())
    }

    /// Marks where instance `id`'s reads stand now, `part`, in the pass traced now.
    pub(crate) fn trace_part(&mut self, id: &str, part: Part) {
        let Some(snap) = self.trace_snap() else {
            return;
        };
        if let Some(marks) = self.trace.as_mut().and_then(|t| t.marks.as_mut()) {
            marks.push((id.to_owned(), part, snap));
        }
    }

    /// What a read that counts move `move_id`'s instances at cut `cut` keeps of each one's
    /// traced reads, `keep` by instance: of the pass it asked for, each part it doesn't keep
    /// set aside; of a pass another read traced, each part it keeps that was set aside taken
    /// back here (L §8.2 One read set; E §10; P20, P22).
    pub(crate) fn trace_keep(&mut self, (move_id, cut): (&str, Day), keep: &[(String, Keep)]) {
        let Some(trace) = self.trace.as_mut() else {
            return;
        };
        let fresh = trace
            .fresh
            .take()
            .filter(|(q, c, _)| q == move_id && *c == cut);
        let mut traced = Vec::new();
        if let Some((_, _, marks)) = fresh {
            let segs = segments(&marks);
            for seg in segs.iter().rev() {
                traced.push(seg.id.clone());
                let wanted = keep
                    .iter()
                    .find(|(id, _)| *id == seg.id)
                    .map_or(Keep::Whole, |(_, k)| *k);
                if wanted != Keep::Whole {
                    trace.set_aside(seg.parent, (seg.day, seg.end), (&seg.id, cut, true));
                }
                if wanted == Keep::Nothing {
                    trace.set_aside(seg.parent, (seg.begin, seg.day), (&seg.id, cut, false));
                }
            }
        }
        for (id, wanted) in keep {
            if traced.contains(id) || *wanted == Keep::Nothing {
                continue;
            }
            trace.take_back((id, cut, false));
            if *wanted == Keep::Whole {
                trace.take_back((id, cut, true));
            }
        }
    }
}

impl Trace {
    /// Sets aside the children of node `parent` between `from` and `to`, kept by `key`; the
    /// cells traced among them are met again as dropped ones.
    fn set_aside(
        &mut self,
        parent: usize,
        (from, to): (Snap, Snap),
        (id, cut, body): (&str, Day, bool),
    ) {
        if from.parent != parent || to.parent != parent || from.children > to.children {
            return;
        }
        let Ok(node) = self.node_mut(parent) else {
            return;
        };
        if to.children > node.children.len() {
            return;
        }
        let mut gone = Vec::new();
        let mut stays = Vec::with_capacity(node.children.len());
        for (n, at) in core::mem::take(&mut node.children).into_iter().enumerate() {
            if (from.children..to.children).contains(&n) {
                gone.push(at);
            } else {
                stays.push(at);
            }
        }
        node.children = stays;
        if gone.is_empty() {
            return;
        }
        let (kept, dropped) = core::mem::take(&mut self.cells)
            .into_iter()
            .partition(|(_, at)| *at < from.nodes || *at >= to.nodes);
        self.cells = kept;
        self.dropped.extend::<BTreeMap<_, usize>>(dropped);
        let key = (id.to_owned(), cut, body);
        match self.stash.get_mut(&key) {
            Some(held) => held.extend(gone),
            None => {
                self.stash.insert(key, gone);
            }
        }
    }

    /// Takes back the part set aside by `key` under the innermost open node.
    fn take_back(&mut self, (id, cut, body): (&str, Day, bool)) {
        let Some(nodes) = self.stash.remove(&(id.to_owned(), cut, body)) else {
            return;
        };
        for at in nodes {
            let done = self.reference(at);
            self.keep(done);
        }
    }
}

/// Each instance's parts from a pass's marks, in the order taken: its first `Begin`, the
/// first `Day` after it (its `End` where there is none: a decision's pass reads days only)
/// and the first `End` after that.
fn segments(marks: &Marks) -> Vec<Seg> {
    let mut out = Vec::new();
    let mut open: Option<(String, Snap, Option<Snap>)> = None;
    for (id, part, snap) in marks {
        open = match (part, open) {
            (Part::Begin, None) => Some((id.clone(), *snap, None)),
            (Part::Day, Some((at, begin, None))) if at == *id => Some((at, begin, Some(*snap))),
            (Part::End, Some((at, begin, day))) if at == *id => {
                out.push(Seg {
                    id: at,
                    parent: begin.parent,
                    begin,
                    day: day.unwrap_or(*snap),
                    end: *snap,
                });
                None
            }
            (_, kept) => kept,
        };
    }
    out
}
