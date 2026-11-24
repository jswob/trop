//! Observations the cut sees, the places versions cover, and the links that tie an
//! observation to the instance it fulfils.
//! Spec: L §5.19 (observations: each row's latest restatement, a scenario's above its base's
//! whatever their `known` days as L §4.6 step 2 ranks claims, `none` removes it; `covers`;
//! links written, judged by existence at the ask's cut, `RUN-RECORD` for a link to nothing or
//! a record that lacks the instance's leg); E §4.1 step 3 (an instance an observation fulfils,
//! or one on a covered day before K, is modelled past), §6.1 (each end of an observed leg booked
//! from the observation of the version that covers its place that day; `WARN-LEGS`); L §6.4
//! (the observations and coverage the timeline's world sees); L §6.1, §5.19 (whether a cut
//! sees one observed row as the observations do: a `same` line between two observed rows has
//! no clock of its own and holds where both are seen; tally 2.2, S-5).
//! Twin: engine/tools/refcore/moves.py (`ObsMv`, `coverage`, `covering`, `covered_on`,
//! `observations`, `obs_links`, `link_targets`, `obs_links_for`, `link_tuple`, `link_places`,
//! `touches_covered`, `obs_row_seen`) and walk.py (`obs_end_on`, `prepare`'s `WARN-LEGS`).
//! Invariants: observations and coverage are computed once per cut; a link is judged once per
//! cut, and what judging it read goes with every observation that fulfils it; an observation's
//! link is never inferred, only read from its row.

use std::collections::{BTreeMap, BTreeSet};

use super::same::Groups;
use crate::base::{Code, Refusal, Run, Warn};
use crate::eval::{Collected, Engine, Rows};
use crate::model::{Fulfils, InputInfo, KeyVal, MvRef, MvSrc, Reality, Span, Value, World};
use crate::syntax::{Expr, Key, Stop};
use crate::time::Day;
use crate::walk::{Amount, Dated, End, Inst, LegVal, Term, invariant};

/// The id an observed row is read by, as written: `r5` of `bank[r5]` (refcore's
/// `resolve_text` of the read's one key). L §5.19.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for keys load refuses (`LOAD-KEY`: an observation is read by its id).
fn row_written(keys: &[Key]) -> Result<String, Stop> {
    if let [Key::Expr(e)] = keys {
        if let Expr::Name(name) = e.unbracketed() {
            return Ok(name.clone());
        }
        if let Expr::Lit(lit) = e.unbracketed() {
            return Ok(lit.text());
        }
    }
    Err(invariant(
        "an observation read by other than its id: load refuses it",
    ))
}

/// An observation the cut sees (refcore's `ObsMv`). L §5.19.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obs {
    /// Its stable id, `bank[r17]`.
    pub id: String,
    /// The observed input.
    pub input: String,
    /// The version that states it.
    pub version: String,
    /// Its event kind.
    pub kind: String,
    /// Its day.
    pub day: Day,
    /// Its legs, each between two places, its amount exact.
    pub legs: Vec<LegVal>,
    /// Its attributes, typed.
    pub values: Vec<(String, Value)>,
    /// What it fulfils.
    pub fulfils: Option<Fulfils>,
    /// Whether its `fulfils` is its class's, stated on a row it is stated `same` with: the
    /// link is judged on the row that states it (L §5.19 One movement on two statements).
    pub shared: bool,
    /// What reading it met: its row, and what judging its link read.
    pub coll: Collected,
}

impl Obs {
    /// Its movement order, `(day, -1, input, row)` (refcore's `ObsMv.order`; tally 2.1.2).
    /// E §4.1, §6.3 Order; L §5.19 Bindings.
    #[must_use]
    pub fn order(&self) -> Term {
        Term::booked(self.day, &[&self.input, super::members::row_key(self)])
    }

    /// The id of the instance it names, `<move>[<keys>]`, when it fulfils one (refcore's
    /// `link_targets`, its tid). L §5.19.
    #[must_use]
    pub fn target_id(&self) -> Option<String> {
        match &self.fulfils {
            Some(Fulfils::Instance(mv, keys)) => Some(inst_id(mv, keys)),
            Some(Fulfils::Adjustment(..)) | None => None,
        }
    }
}

/// The id of a move's instance at its keys, `<move>[<keys>]` (L §2.5).
#[must_use]
pub fn inst_id(mv: &str, keys: &[KeyVal]) -> String {
    if keys.is_empty() {
        return mv.to_owned();
    }
    let texts: Vec<String> = keys.iter().map(KeyVal::text).collect();
    format!("{mv}[{}]", texts.join(", "))
}

/// An observation's record refused: its id and the refusal (L §5.19 Links).
type Refused = (String, Refusal);

/// A link judged: the observations that fulfil an instance, by id, and what judging read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Linked {
    /// The observations whose link stands, in the observations' order.
    pub obs: Vec<String>,
    /// What judging the links read.
    pub coll: Collected,
}

/// The versions that cover each place under one cut (refcore's `coverage`). L §5.19
/// Coverage, §4.5.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Coverage {
    /// Each place's covered spans, each with the version that covers it.
    pub spans: BTreeMap<String, Vec<(Span, String)>>,
}

impl Engine {
    /// The observations cut `cut` sees, each row's latest restatement, a row restated `none`
    /// left out, sorted by day and id (refcore's `observations`). L §5.19, §4.6.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a row load left without its day, kind or version, or for a freed
    /// layer.
    pub(crate) fn observations(&mut self, m: &Reality, cut: Day) -> Result<&[Obs], Stop> {
        let world = self.timeline_world();
        if !self.observed.contains_key(&cut) {
            let mut seen = seen_observations(m, (cut, world), self.keep_rows)?;
            let groups = self.same_groups(m, cut)?;
            share_fulfils(&groups, &mut seen);
            self.observed.insert(cut, seen);
        }
        self.observed
            .get(&cut)
            .map(Vec::as_slice)
            .ok_or_else(|| invariant("observations just kept"))
    }

    /// A read of observed input `input` by a row's id, `bank[r5]`: the observation cut `cut`
    /// sees with that id as a movement, its row recorded as read; `none` where the cut sees
    /// none (refcore's `m_e_obsrow`). L §5.19 (observation rows are keyed by their ids).
    ///
    /// # Errors
    ///
    /// The observations' (`observations`); `ENGINE-INVARIANT` for a key load refuses
    /// (`LOAD-KEY`: an observation is read by its id).
    pub(crate) fn observed_row(
        &mut self,
        m: &Reality,
        (input, keys): (&str, &[Key]),
        cut: Day,
    ) -> Result<Value, Stop> {
        let id = format!("{input}[{}]", row_written(keys)?);
        let rows = self.observations(m, cut)?;
        let Some(o) = rows.iter().find(|o| o.id == id) else {
            return Ok(Value::None);
        };
        let mut met = Collected::default();
        met.note_version(&o.version);
        if let Some(seen) = o.coll.rows().and_then(|r| r.get(&id)) {
            met.note_row(&id, &seen.version, seen.locator.as_ref());
        }
        let day = Some(o.day);
        self.absorb(&met);
        Ok(Value::Mv(Box::new(MvRef {
            id,
            day,
            src: MvSrc::Obs,
        })))
    }

    /// The versions cut `cut` sees that cover each place, with the span each covers
    /// (refcore's `coverage`). L §5.19 Coverage.
    pub(crate) fn coverage(&mut self, m: &Reality, cut: Day) -> &Coverage {
        let world = self.timeline_world();
        self.covered
            .entry(cut)
            .or_insert_with(|| seen_coverage(m, (cut, world)))
    }

    /// The versions that cover `place` on `day` under cut `cut` (refcore's `covering`).
    /// L §5.19 Coverage.
    pub(crate) fn covering(&mut self, m: &Reality, place: &str, day: Day, cut: Day) -> Vec<String> {
        let Some(spans) = self.coverage(m, cut).spans.get(place) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (span, v) in spans {
            if span.contains(day) {
                out.push(v.clone());
            }
        }
        out
    }

    /// `covered(x, span)`: versions the cut knows cover `x` on every day of the span; false
    /// for a hole or `none` place, a hole span or one with an open end; never a hole
    /// (refcore's `c_covered`). L §5.19 Coverage, §8.1.
    ///
    /// # Errors
    ///
    /// The refusals reading the place or the span meets.
    pub(crate) fn covered_call(
        &mut self,
        m: &Reality,
        (place, span): (&Expr, &crate::syntax::Span),
        env: &crate::eval::Env,
        ctx: crate::eval::Ctx,
    ) -> Result<Value, Stop> {
        let p = self.ev(m, place, env, ctx)?;
        let Value::Key(KeyVal::Thing(name)) = p else {
            return Ok(Value::Bool(false));
        };
        let Ok(sp) = self.span_value(m, span, env, ctx)? else {
            return Ok(Value::Bool(false));
        };
        let (Some(start), Some(end)) = (sp.start(), sp.end()) else {
            return Ok(Value::Bool(false));
        };
        let spans: Vec<Span> = match self.coverage(m, ctx.cut).spans.get(&name) {
            Some(list) => list.iter().map(|(s, _)| s.clone()).collect(),
            None => Vec::new(),
        };
        Ok(Value::Bool(covers_all(&spans, (start, end))))
    }

    /// Whether an instance touches a place a version covers under cut `cut` on its leg's day,
    /// or the instance's (refcore's `touches_covered`). E §4.1 step 3, L §5.19 Coverage.
    pub(crate) fn touches_covered(&mut self, m: &Reality, inst: &Inst, cut: Day) -> bool {
        for leg in &inst.legs {
            let Some(day) = leg
                .day
                .day()
                .or_else(|| inst.day.as_ref().and_then(Dated::day))
            else {
                continue;
            };
            for place in [&leg.from, &leg.to].into_iter().filter_map(End::place) {
                if !self.covering(m, place, day, cut).is_empty() {
                    return true;
                }
            }
        }
        false
    }

    /// Whether observation `o` books its end at `place` on `day`: no version covers the
    /// place that day, or `o`'s own version does (refcore's `obs_end_on`). E §6.1, L §5.19.
    pub(crate) fn obs_end_on(
        &mut self,
        m: &Reality,
        o: &Obs,
        place: &str,
        day: Day,
        cut: Day,
    ) -> bool {
        let cov = self.covering(m, place, day, cut);
        cov.is_empty() || cov.contains(&o.version)
    }

    /// The observations that fulfil instance `tid` under cut `cut`, each link judged: the
    /// instance must exist at the cut by its keys, `where` and `when`, touch the observation's
    /// places and have no leg the observation lacks, else the observation is refused
    /// `RUN-RECORD` (refcore's `obs_links_for`). L §5.19 Links.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for a link whose judging reads its own observations; a load refusal met
    /// judging it.
    pub(crate) fn obs_links_for(
        &mut self,
        m: &Reality,
        tid: &str,
        cut: Day,
    ) -> Result<Vec<String>, Stop> {
        let key = (cut, tid.to_owned());
        // a link judged at a decision's walk point, read inside a plan's list: a plan reaching
        // a running read, whichever read judged it first (E §3.1 step 7; tally 2.3.1 C-9)
        if self.plans_making > 0 && self.links_at_point.contains(&key) {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(crate::base::Load::Reads),
                format!("the link to {tid} reads a running read outside a decision"),
            )));
        }
        if let Some(hit) = self.links.get(&key) {
            let (obs, coll) = (hit.obs.clone(), hit.coll.clone());
            self.absorb(&coll);
            return Ok(obs);
        }
        if self.link_active.contains(&key) {
            self.narrow_open_to_days();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("the link to {tid} reads its own observations"),
            )));
        }
        self.link_active.insert(key.clone());
        self.push();
        // what the judging reads is the links', shared by every reader (E §3.2)
        let mark = self.reads_apart();
        let judged = self.judge_links(m, tid, cut);
        self.reads_back(mark);
        let coll = self.pop_alone();
        self.link_active.remove(&key);
        let (obs, errs) = judged?;
        if let Some(seen) = self.observed.get_mut(&cut) {
            for o in seen
                .iter_mut()
                .filter(|o| o.target_id().as_deref() == Some(tid))
            {
                o.coll.merge(&coll);
            }
        }
        for (id, err) in errs {
            self.obs_errs.insert((cut, id), err);
        }
        self.absorb(&coll);
        self.links.insert(
            key,
            Linked {
                obs: obs.clone(),
                coll,
            },
        );
        Ok(obs)
    }

    /// The refusal S1 gives a read resting on movement or instance `id` under cut `cut`, or
    /// none: the links a `same` class's rows make are judged once, every one, so both
    /// instances a class fulfils and both its rows are known before any read takes one
    /// (refcore's `s1_refusal`, pick P42). L §5.19 One movement on two statements, §8.4.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
    pub(crate) fn s1_refusal(
        &mut self,
        m: &Reality,
        id: &str,
        cut: Day,
    ) -> Result<Option<Refusal>, Stop> {
        if self.s1_judged.insert(cut) {
            // what the judging reads is the refusal's, not the reader's: its register, its
            // read set and the trace's note of a walk read (E §10) go back as they stood
            let before = self.trace_walked();
            self.push();
            let mark = self.reads_apart();
            let judged = self.judge_class_links(m, cut);
            self.reads_back(mark);
            self.pop_alone();
            self.trace_walked_back(before);
            judged?;
        }
        Ok(self.s1_links.get(&(cut, id.to_owned())).cloned())
    }

    /// Each instance a row of a `same` class fulfils, its links judged, in the observations'
    /// order (refcore's `s1_refusal`, its loop). L §5.19 One movement on two statements.
    fn judge_class_links(&mut self, m: &Reality, cut: Day) -> Result<(), Stop> {
        let groups = self.same_groups(m, cut)?;
        if groups.is_empty() {
            return Ok(());
        }
        let mut targets: Vec<String> = Vec::new();
        for o in self.observations(m, cut)? {
            if let Some(t) = o.target_id()
                && groups.contains_key(&o.id)
                && !targets.contains(&t)
            {
                targets.push(t);
            }
        }
        for t in &targets {
            self.obs_links_for(m, t, cut)?;
        }
        Ok(())
    }

    /// Each observation naming `tid`, its link judged (refcore's `obs_links_for`, its loop).
    /// L §5.19 Links.
    fn judge_links(
        &mut self,
        m: &Reality,
        tid: &str,
        cut: Day,
    ) -> Result<(Vec<String>, Vec<Refused>), Stop> {
        let mut naming: Vec<Obs> = Vec::new();
        for o in self.observations(m, cut)? {
            if o.target_id().as_deref() == Some(tid) {
                naming.push(o.clone());
            }
        }
        let mut good = Vec::new();
        let mut shared = Vec::new();
        let mut errs = Vec::new();
        for o in naming {
            // a `fulfils` stated on another row of its class is judged on that row (L §5.19)
            if o.shared {
                shared.push(o.id);
                continue;
            }
            let Some(Fulfils::Instance(move_id, keys)) = &o.fulfils else {
                continue;
            };
            let mv = m
                .moves
                .get(move_id)
                .ok_or_else(|| invariant("a link to a move load didn't build"))?;
            let found = self.link_tuple(m, mv, keys, cut)?;
            let mut bad = found.is_none();
            if let Some(env) = &found {
                let places = self.link_places(m, mv, env, cut)?;
                let touched: BTreeSet<&str> = o
                    .legs
                    .iter()
                    .flat_map(|lg| [lg.from.place(), lg.to.place()])
                    .flatten()
                    .collect();
                if !places.is_empty() && places.iter().all(|p| !touched.contains(p.as_str())) {
                    bad = true;
                }
            }
            if bad {
                errs.push((
                    o.id.clone(),
                    record(format!(
                        "observation {} fulfils {tid}, which doesn't exist or doesn't touch its places",
                        o.id
                    )),
                ));
                continue;
            }
            let lack = mv.decl.legs.iter().find(|lg| !states_leg(&o, &lg.name));
            if let Some(lg) = lack
                && !m.kind_legs(&o.kind).is_empty()
            {
                let err = record(format!(
                    "observation {} fulfils {tid} and lacks its leg {}",
                    o.id, lg.name
                ));
                // what reads the instance's realised movement rests on the refused record
                // (L §8.4)
                self.bad_links
                    .entry((cut, tid.to_owned()))
                    .or_insert_with(|| err.clone());
                errs.push((o.id.clone(), err));
                continue;
            }
            good.push(o);
        }
        let differ = self.fulfils_differ(m, tid, &good, cut)?;
        let differs: BTreeSet<&str> = differ.iter().map(|(id, _)| id.as_str()).collect();
        let left: Vec<Obs> = good
            .into_iter()
            .filter(|o| !differs.contains(o.id.as_str()))
            .collect();
        errs.extend(differ);
        let unjoined = self.unjoined(m, tid, &left, cut)?;
        if let Some((_, err)) = unjoined.first() {
            self.bad_links
                .entry((cut, tid.to_owned()))
                .or_insert_with(|| err.clone());
        }
        let refused: BTreeSet<&str> = unjoined.iter().map(|(id, _)| id.as_str()).collect();
        let stands: Vec<String> = left
            .into_iter()
            .filter(|o| !refused.contains(o.id.as_str()))
            .map(|o| o.id)
            .collect();
        errs.extend(unjoined);
        // a row with no `fulfils` of its own stands in where a row of its class does
        let groups = self.same_groups(m, cut)?;
        let mut standing: Vec<String> = Vec::new();
        for id in shared {
            let Some(class) = groups.get(&id) else {
                continue;
            };
            if class.iter().any(|y| stands.contains(y)) {
                standing.push(id);
            }
        }
        let seen = self.observations(m, cut)?;
        let linked = seen
            .iter()
            .filter(|o| stands.contains(&o.id) || standing.contains(&o.id))
            .map(|o| o.id.clone())
            .collect();
        Ok((linked, errs))
    }

    /// The observations that stand in for a modelled instance on a covered day: those whose
    /// link to it stands (refcore's `flows_move`, `i.obs or obs_links_for`). L §5.14.3, §5.19.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
    pub(crate) fn standing_in(
        &mut self,
        m: &Reality,
        inst: &Inst,
        cut: Day,
    ) -> Result<Vec<Obs>, Stop> {
        let ids = if inst.obs.is_empty() {
            self.obs_links_for(m, &inst.id, cut)?
        } else {
            inst.obs.clone()
        };
        let all = self.observations(m, cut)?;
        Ok(all
            .iter()
            .filter(|o| ids.contains(&o.id))
            .cloned()
            .collect())
    }

    /// The refusal of observation `o`'s record under cut `cut`, its link judged first: a link
    /// to nothing, or a record that lacks the instance's leg (refcore's `ObsMv.err`). L §5.19.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
    pub(crate) fn obs_err(
        &mut self,
        m: &Reality,
        o: &Obs,
        cut: Day,
    ) -> Result<Option<Refusal>, Stop> {
        if let Some(tid) = o.target_id() {
            // judging the link is no read of the walk the explanation prints (E §10): a row
            // whose class states its `fulfils` explains as a row (group 4's review, N4)
            let before = self.trace_walked();
            let judged = self.judged(m, &tid, cut);
            self.trace_walked_back(before);
            judged?;
        }
        Ok(self.obs_errs.get(&(cut, o.id.clone())).cloned())
    }

    /// The refusal of the first observation, in their order, whose link to instance `tid`
    /// under cut `cut` is refused, the links to it judged first; none where none is. A read
    /// that an observation would stand for `tid` in rests on that link, so it is refused with
    /// it (L §5.19 Links, §8.4, §8.2 One read set; tally 2.3.1 R2, C-13.6).
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
    pub(crate) fn refused_link(
        &mut self,
        m: &Reality,
        tid: &str,
        cut: Day,
    ) -> Result<Option<Refusal>, Stop> {
        if self.judged(m, tid, cut)?.is_none() {
            return Ok(None);
        }
        let mut naming = Vec::new();
        for o in self.observations(m, cut)? {
            if o.target_id().as_deref() == Some(tid) {
                naming.push(o.id.clone());
            }
        }
        Ok(naming
            .into_iter()
            .find_map(|id| self.obs_errs.get(&(cut, id)).cloned()))
    }

    /// The instance observation `o` stands in for under cut `cut`, by id, its link judged
    /// first (refcore's `ObsMv.target`). L §5.19.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
    pub(crate) fn obs_target(
        &mut self,
        m: &Reality,
        o: &Obs,
        cut: Day,
    ) -> Result<Option<String>, Stop> {
        let Some(tid) = o.target_id() else {
            return Ok(None);
        };
        let Some(good) = self.judged(m, &tid, cut)? else {
            return Ok(None);
        };
        Ok(good.contains(&o.id).then_some(tid))
    }

    /// The links to `tid` judged, or `None` while judging them is refused by anything but
    /// load: the observation's verdict waits until it can be judged (refcore's
    /// `ObsMv._judge_link`). L §5.19 Links.
    fn judged(&mut self, m: &Reality, tid: &str, cut: Day) -> Result<Option<Vec<String>>, Stop> {
        let good = match self.obs_links_for(m, tid, cut) {
            Ok(good) => good,
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => return Ok(None),
            Err(stop) => return Err(stop),
        };
        // a decision's link judged at its walk point: a read from outside the walk is a
        // running read, so it takes the main walk's verdict (E §4.2 step 3, §3.1 step 7;
        // tally 2.3.1 R6, C-9); none is walked from inside a plan, which load refuses
        let key = (cut, tid.to_owned());
        if self.links_at_point.contains(&key)
            && !self.main_walks.contains_key(&cut)
            && self.plans_making == 0
            && self.plan_active.is_empty()
        {
            self.main_walk(m, cut)?;
            return Ok(self.links.get(&key).map(|hit| hit.obs.clone()));
        }
        Ok(Some(good))
    }

    /// The links to decision instance `tid` under cut `cut`, judged at its walk point, where
    /// its `when` says it doesn't exist: each observation that fulfils it refused (`RUN-RECORD`),
    /// for the walk and every later reader (L §5.19 Links, §5.14 An instance's existence is a
    /// cell; E §4.2 step 3; tally 2.3.1 R6, C-9). Returns the refusal by observation.
    pub(crate) fn unlink_at_point(&mut self, tid: &str, obs: &[String], cut: Day) -> Vec<Refused> {
        let mut out = Vec::with_capacity(obs.len());
        if !self.links_at_point.contains(&(cut, tid.to_owned())) {
            return out;
        }
        if let Some(hit) = self.links.get_mut(&(cut, tid.to_owned())) {
            hit.obs.clear();
        }
        for id in obs {
            let err = record(format!(
                "observation {id} fulfils {tid}, which doesn't exist or doesn't touch its places"
            ));
            self.obs_errs.insert((cut, id.clone()), err.clone());
            out.push((id.clone(), err));
        }
        out
    }

    /// The day of the decision instance observation `o` fulfils, its link standing; `None`
    /// for one that fulfils a plan's instance or nothing. Such an observation is booked at
    /// the instance's point on that day, not at its own (refcore's `book_obs_leg`, its
    /// test). E §4.2 step 1, L §5.19.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
    pub(crate) fn decision_fulfilled(
        &mut self,
        m: &Reality,
        o: &Obs,
        cut: Day,
    ) -> Result<Option<Day>, Stop> {
        let Some(Fulfils::Instance(move_id, _)) = &o.fulfils else {
            return Ok(None);
        };
        let Some(tid) = self.obs_target(m, o, cut)? else {
            return Ok(None);
        };
        let mv = m
            .moves
            .get(move_id)
            .ok_or_else(|| invariant("a link to a move load didn't build"))?;
        if !mv.decision {
            return Ok(None);
        }
        let insts = self.move_instances(m, mv, cut)?;
        Ok(insts
            .iter()
            .find(|i| i.id == tid)
            .and_then(|i| i.day.as_ref())
            .and_then(Dated::day))
    }

    /// `WARN-LEGS`: two observations of different versions that fulfil one instance and
    /// disagree on a leg's amount (refcore's `prepare`, its links); and two observed rows
    /// stated `same` that disagree, where no row of their group fulfils an instance, the
    /// movement named by its first row (L §5.19, l.1717; tally 2.2.1's 219.2). E §6.1.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`] and [`Engine::same_groups`].
    pub(crate) fn disagreeing_links(
        &mut self,
        m: &Reality,
        cut: Day,
    ) -> Result<Vec<(Warn, String)>, Stop> {
        let mut targets: BTreeSet<String> = BTreeSet::new();
        for o in self.observations(m, cut)? {
            if let Some(tid) = o.target_id() {
                targets.insert(tid);
            }
        }
        let mut out = Vec::new();
        let mut linked: BTreeSet<String> = BTreeSet::new();
        let groups = self.same_groups(m, cut)?;
        for tid in targets {
            let good = self.obs_links_for(m, &tid, cut)?;
            let mut all: Vec<Obs> = Vec::new();
            for o in self.observations(m, cut)? {
                if good.contains(&o.id) {
                    all.push(o.clone());
                }
            }
            linked.extend(good);
            let mut rest = all.as_slice();
            while let [a, tail @ ..] = rest {
                rest = tail;
                pairs_warned(m, (&tid, &groups), a, tail, &mut out)?;
            }
        }
        self.same_rows_warned(m, cut, &linked, &mut out)?;
        Ok(out)
    }

    /// `WARN-LEGS` for each `same` group of observed rows none of which is among `linked`,
    /// those that fulfil an instance: its rows in id order, pair by pair, the movement named
    /// by its first (refcore's `prepare`; tally 2.2.1's 219.2). L §5.19 One movement on two
    /// statements, l.1717; E §6.1.
    fn same_rows_warned(
        &mut self,
        m: &Reality,
        cut: Day,
        linked: &BTreeSet<String>,
        out: &mut Vec<(Warn, String)>,
    ) -> Result<(), Stop> {
        let groups = self.same_groups(m, cut)?;
        let mut seen: BTreeSet<&[String]> = BTreeSet::new();
        for group in groups.values() {
            if !seen.insert(group.as_slice()) || group.iter().any(|y| linked.contains(y)) {
                continue;
            }
            // the group's rows the cut sees, in its order: the order of their ids
            let mut rows: Vec<Obs> = Vec::new();
            for o in self.observations(m, cut)? {
                if group.contains(&o.id) {
                    rows.push(o.clone());
                }
            }
            rows.sort_by_cached_key(|o| m.id_order(&o.id));
            let Some(first) = rows.first() else {
                continue;
            };
            let mut rest = rows.as_slice();
            while let [a, tail @ ..] = rest {
                rest = tail;
                same_pairs_warned(m, &first.id, a, tail, out)?;
            }
        }
        Ok(())
    }
}

/// `WARN-LEGS` for row `a` against each later row of its `same` group that disagrees with it
/// on an amount or a day, in one version or two: what each states of the fact they disagree
/// on, the amounts, or the days where only the days differ; each warning once (L §5.19
/// l.1717, §8.5; B5, Q7, P36 on #241).
fn same_pairs_warned(
    m: &Reality,
    name: &str,
    a: &Obs,
    later: &[Obs],
    out: &mut Vec<(Warn, String)>,
) -> Result<(), Stop> {
    for b in later {
        let Some((fa, fb)) = disagreement(m, a, b)? else {
            continue;
        };
        let text = format!(
            "the two observations of {name} disagree: {} ({fa}) and {} ({fb})",
            a.id, b.id
        );
        let w = (Warn::Legs, text);
        if !out.contains(&w) {
            out.push(w);
        }
    }
    Ok(())
}

/// `WARN-LEGS` for observation `a` against each later one that fulfils instance `tid` and
/// disagrees with it on an amount or a day, of another version or stated `same` with it in
/// `groups`, each warning once: unjoined rows of one version are pieces of a payment
/// (refcore's `prepare`, `legs_warn`; L §5.19 l.1717, "the sentence names no version"; B5,
/// P36 on #241). E §6.1.
fn pairs_warned(
    m: &Reality,
    (tid, groups): (&str, &Groups),
    a: &Obs,
    later: &[Obs],
    out: &mut Vec<(Warn, String)>,
) -> Result<(), Stop> {
    for b in later {
        let joined = groups.get(&a.id).is_some_and(|g| g.contains(&b.id));
        if a.version == b.version && !joined {
            continue;
        }
        let Some((fa, fb)) = disagreement(m, a, b)? else {
            continue;
        };
        let text = format!(
            "the two observations of {tid} disagree: {} ({fa}) and {} ({fb})",
            a.id, b.id
        );
        let w = (Warn::Legs, text);
        if !out.contains(&w) {
            out.push(w);
        }
    }
    Ok(())
}

/// What observations `a` and `b` each state of the fact they disagree on, as `WARN-LEGS`
/// prints it: their leg amounts where an amount differs, else their days where only the days
/// differ; none where they agree on both ("disagree on an amount or a day", L §5.19 l.1717;
/// refcore's `legs_warn`; B5, Q7, P36 on #241). E §6.1.
fn disagreement(m: &Reality, a: &Obs, b: &Obs) -> Result<Option<(String, String)>, Stop> {
    if disagree(a, b) {
        return Ok(Some((amounts(m, a)?, amounts(m, b)?)));
    }
    if a.day != b.day {
        return Ok(Some((a.day.text(), b.day.text())));
    }
    Ok(None)
}

/// Whether two observations disagree on the amount of a leg both state (refcore's
/// `prepare`). E §6.1.
fn disagree(a: &Obs, b: &Obs) -> bool {
    a.legs.iter().any(|la| differs(b, la))
}

/// Whether observation `b` states leg `la`'s name with another amount; a leg of `la`'s that
/// is a hole differs from nothing (refcore's `prepare`, its links). E §6.1.
fn differs(b: &Obs, la: &LegVal) -> bool {
    !matches!(la.amount, Amount::Hole(_))
        && b.legs
            .iter()
            .any(|lb| lb.name == la.name && lb.amount != la.amount)
}

/// Whether observation `o` states a leg named `name`. L §5.19 Links.
fn states_leg(o: &Obs, name: &str) -> bool {
    o.legs.iter().any(|ol| ol.name == name)
}

/// An observation's leg amounts, by leg name, as `WARN-LEGS` prints them. E §6.1.
fn amounts(m: &Reality, o: &Obs) -> Result<String, Stop> {
    let mut legs: Vec<&LegVal> = o.legs.iter().collect();
    legs.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = Vec::with_capacity(legs.len());
    for lg in legs {
        if let Amount::Hole(h) = &lg.amount {
            out.push(h.text());
            continue;
        }
        let (Amount::Qty(q), Some(unit)) = (&lg.amount, &lg.unit) else {
            return Err(invariant("an observed leg with no amount"));
        };
        let v = crate::walk::qty_value(m, q, unit)?;
        out.push(v.text(m).map_err(crate::eval::num_stop)?);
    }
    Ok(out.join(", "))
}

/// A `RUN-RECORD` refusal (L §8.4).
fn record(message: String) -> Refusal {
    Refusal::new(Code::Run(Run::Record), message)
}

/// Input `input`'s observation at position `at` (L §5.19).
fn observation_at(input: &InputInfo, at: usize) -> Result<&crate::model::Observation, Stop> {
    input
        .observations
        .get(at)
        .ok_or_else(|| invariant("an observation past the input's"))
}

/// Whether cut `cut` in world `world` sees observed row `id` (`bank[r1]`) as the observations
/// do ([`seen_observations`]): its latest restatement known by the cut in a version the world
/// sees, a scenario's above its base's whatever their `known` days, of two in one layer known
/// the same day the first written, isn't `none`. A row no version states is seen: a line
/// naming it stands, and the read of that side refuses (`RUN-RECORD`) (refcore's
/// `obs_row_seen`). L §5.19, §6.1, §6.4, §4.6 step 2 (tally 2.2, S-5).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a version load didn't build.
pub(crate) fn observed_row_seen(
    m: &Reality,
    id: &str,
    (cut, world): (Day, World),
) -> Result<bool, Stop> {
    let Some((name, key)) = id.strip_suffix(']').and_then(|inner| inner.split_once('[')) else {
        return Ok(true);
    };
    let Some(input) = m.inputs.get(name) else {
        return Ok(true);
    };
    let mut stated = false;
    // the latest restatement's layer and `known` day, and whether it removes the row
    let mut latest: Option<((usize, Day), bool)> = None;
    for o in input.observations.iter().filter(|o| o.key == key) {
        stated = true;
        let version = m
            .versions
            .get(&o.version)
            .ok_or_else(|| invariant("an observation of a version load didn't build"))?;
        let known_on = o.own_known.unwrap_or(version.known);
        if known_on > cut || !m.scenarios.sees(world, Some(&o.version)) {
            continue;
        }
        let rank = (m.scenarios.layer(Some(&o.version)), known_on);
        if latest.is_none_or(|(kept, _)| rank > kept) {
            latest = Some((rank, o.removed));
        }
    }
    Ok(!stated || latest.is_some_and(|(_, removed)| !removed))
}

/// The observations the cut and the world of `seen` see: per observed input, each row id's
/// latest restatement known by the cut in a version the world sees, a scenario's above its
/// base's whatever their `known` days, `none` removing it; sorted by day and id, each with its
/// row among what it met where `rows` says so (refcore's `observations`). L §5.19, §6.4, §4.6
/// step 2; E §10.
fn seen_observations(
    m: &Reality,
    (cut, world): (Day, World),
    rows: Rows,
) -> Result<Vec<Obs>, Stop> {
    let mut out = Vec::new();
    for (_, input) in m.inputs.entries() {
        // each row id's latest restatement, its layer and `known` day, and its position; of
        // two in one layer known the same day, the first written
        let mut latest: BTreeMap<&str, ((usize, Day), usize)> = BTreeMap::new();
        for (at, o) in input.observations.iter().enumerate() {
            let version = m
                .versions
                .get(&o.version)
                .ok_or_else(|| invariant("an observation of a version load didn't build"))?;
            let known_on = o.own_known.unwrap_or(version.known);
            if known_on > cut || !m.scenarios.sees(world, Some(&o.version)) {
                continue;
            }
            let rank = (m.scenarios.layer(Some(&o.version)), known_on);
            latest
                .entry(o.key.as_str())
                .and_modify(|kept| {
                    if rank > kept.0 {
                        *kept = (rank, at);
                    }
                })
                .or_insert((rank, at));
        }
        for (_, (_, at)) in latest {
            let o = observation_at(input, at)?;
            if o.removed {
                continue;
            }
            let mut seen = observation(o)?;
            // the row as `inputs-of` lists it, kept by an engine answering one (E §10)
            if rows == Rows::Kept {
                seen.coll.note_row(&seen.id, &o.version, o.locator.as_ref());
            }
            out.push(seen);
        }
    }
    out.sort_by(|a, b| (a.day, &a.id).cmp(&(b.day, &b.id)));
    Ok(out)
}

/// A row of `seen` stated `same` with others that states no `fulfils` takes its group's: the
/// first of the group's seen rows, in the same lines' order, that states one. A movement on
/// two statements fulfils what either row states, so both rows stand in for it (L §5.19 One
/// movement on two statements, §6.1).
fn share_fulfils(groups: &BTreeMap<String, Vec<String>>, seen: &mut [Obs]) {
    if groups.is_empty() {
        return;
    }
    let mut stated: BTreeMap<String, Fulfils> = BTreeMap::new();
    for o in seen.iter() {
        if let Some(f) = &o.fulfils {
            stated.insert(o.id.clone(), f.clone());
        }
    }
    for o in seen.iter_mut().filter(|o| o.fulfils.is_none()) {
        let Some(group) = groups.get(&o.id) else {
            continue;
        };
        if let Some(f) = group.iter().find_map(|member| stated.get(member)) {
            o.fulfils = Some(f.clone());
            o.shared = true;
        }
    }
}

/// Whether `spans` cover every day from `start` up to `end` (refcore's `covers_all`). L §5.19
/// Coverage.
fn covers_all(spans: &[Span], (start, end): (Day, Day)) -> bool {
    let mut d = start;
    while d < end {
        let mut next: Option<Day> = None;
        for s in spans.iter().filter(|s| s.contains(d)) {
            let Some(e) = s.end() else {
                return true;
            };
            if next.is_none_or(|n| e > n) {
                next = Some(e);
            }
        }
        let Some(n) = next else {
            return false;
        };
        d = n;
    }
    true
}

/// One observation as a movement: its legs between places, on the observation's day
/// (refcore's `ObsMv`). L §5.19.
fn observation(o: &crate::model::Observation) -> Result<Obs, Stop> {
    let day = o
        .day
        .ok_or_else(|| invariant("an observation with no day"))?;
    let kind = o
        .kind
        .clone()
        .ok_or_else(|| invariant("an observation with no kind"))?;
    let legs = o
        .legs
        .iter()
        .map(|lg| LegVal {
            name: lg.name.clone(),
            from: End::Place(lg.from.clone()),
            to: End::Place(lg.to.clone()),
            amount: Amount::Qty(lg.amount.clone()),
            unit: Some(lg.unit.clone()),
            day: Dated::Day(day),
            src: None,
        })
        .collect();
    let mut coll = Collected::default();
    coll.note_version(&o.version);
    Ok(Obs {
        id: o.sid(),
        input: o.input.clone(),
        version: o.version.clone(),
        kind,
        day,
        legs,
        values: o.values.clone(),
        fulfils: o.fulfils.clone(),
        shared: false,
        coll,
    })
}

/// The versions the cut and the world of `seen` see that cover each place, by version id: each
/// place with its span, the one written or the version's `governs` (refcore's `coverage`).
/// L §5.19 Coverage, §6.4.
fn seen_coverage(m: &Reality, (cut, world): (Day, World)) -> Coverage {
    let mut out = Coverage::default();
    let mut versions: Vec<&crate::model::VersionInfo> =
        m.versions.entries().map(|(_, v)| v).collect();
    versions.sort_by(|a, b| a.id.cmp(&b.id));
    for v in versions {
        if v.covers.is_empty() || v.known > cut || !m.scenarios.sees(world, Some(&v.id)) {
            continue;
        }
        for (place, span) in &v.covers {
            let Some(span) = span
                .as_ref()
                .or_else(|| v.span_at(&m.scenarios, (cut, world)))
            else {
                continue;
            };
            out.spans
                .entry(place.clone())
                .or_insert_with(Vec::new)
                .push((span.clone(), v.id.clone()));
        }
    }
    out
}
