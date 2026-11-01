//! Observations the cut sees, the places versions cover, and the links that tie an
//! observation to the instance it fulfils.
//! Spec: L §5.19 (observations: each row's latest restatement, `none` removes it; `covers`;
//! links written, judged by existence at the ask's cut, `RUN-RECORD` for a link to nothing or
//! a record that lacks the instance's leg); E §4.1 step 3 (an instance an observation fulfils,
//! or one on a covered day before K, is modelled past), §6.1 (each end of an observed leg booked
//! from the observation of the version that covers its place that day; `WARN-LEGS`).
//! Twin: engine/tools/refcore/moves.py (`ObsMv`, `coverage`, `covering`, `covered_on`,
//! `observations`, `obs_links`, `link_targets`, `obs_links_for`, `link_tuple`, `link_places`,
//! `touches_covered`) and walk.py (`obs_end_on`, `prepare`'s `WARN-LEGS`).
//! Invariants: observations and coverage are computed once per cut; a link is judged once per
//! cut, and what judging it read goes with every observation that fulfils it; an observation's
//! link is never inferred, only read from its row.

use std::collections::{BTreeMap, BTreeSet};

use crate::base::{Code, Refusal, Run, Warn};
use crate::eval::{Collected, Engine};
use crate::model::{Fulfils, KeyVal, Reality, Span, Value};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{Amount, Dated, End, Inst, LegVal, invariant};

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
    /// What reading it met: its row, and what judging its link read.
    pub coll: Collected,
}

impl Obs {
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

/// The versions that cover each place under one cut: the span and the version (refcore's
/// `coverage`). L §5.19 Coverage.
pub type Coverage = BTreeMap<String, Vec<(Span, String)>>;

impl Engine {
    /// The observations cut `cut` sees, each row's latest restatement, a row restated `none`
    /// left out, sorted by day and id (refcore's `observations`). L §5.19, §4.6.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a row load left without its day, kind or version; `NotBuilt` for
    /// a row known in a window.
    pub(crate) fn observations(&mut self, m: &Reality, cut: Day) -> Result<&[Obs], Stop> {
        match self.observed.entry(cut) {
            std::collections::btree_map::Entry::Occupied(seen) => Ok(seen.into_mut().as_slice()),
            std::collections::btree_map::Entry::Vacant(slot) => {
                Ok(slot.insert(seen_observations(m, cut)?).as_slice())
            }
        }
    }

    /// The versions cut `cut` sees that cover each place, with the span each covers
    /// (refcore's `coverage`). L §5.19 Coverage.
    pub(crate) fn coverage(&mut self, m: &Reality, cut: Day) -> &Coverage {
        self.covered
            .entry(cut)
            .or_insert_with(|| seen_coverage(m, cut))
    }

    /// The versions that cover `place` on `day` under cut `cut` (refcore's `covering`).
    /// L §5.19 Coverage.
    pub(crate) fn covering(&mut self, m: &Reality, place: &str, day: Day, cut: Day) -> Vec<String> {
        let Some(spans) = self.coverage(m, cut).get(place) else {
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
    /// for a hole or `none` place, a hole span or one with an open end, never a hole (refcore's
    /// `c_covered`). L §5.19 Coverage.
    ///
    /// # Errors
    ///
    /// The refusals reading the place or the span meets.
    pub(crate) fn covered_call(
        &mut self,
        m: &Reality,
        (place, span): (&crate::syntax::Expr, &crate::syntax::Span),
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
        let (Some(mut d), Some(end)) = (sp.start(), sp.end()) else {
            return Ok(Value::Bool(false));
        };
        let spans: Vec<Span> = match self.coverage(m, ctx.cut).get(&name) {
            Some(list) => list.iter().map(|(s, _)| s.clone()).collect(),
            None => Vec::new(),
        };
        while d < end {
            let mut next: Option<Day> = None;
            for s in spans.iter().filter(|s| s.contains(d)) {
                let Some(e) = s.end() else {
                    return Ok(Value::Bool(true));
                };
                if next.is_none_or(|n| e > n) {
                    next = Some(e);
                }
            }
            let Some(n) = next else {
                return Ok(Value::Bool(false));
            };
            d = n;
        }
        Ok(Value::Bool(true))
    }

    /// Whether an instance touches a place a version covers on its leg's day, or the
    /// instance's (refcore's `touches_covered`). E §4.1 step 3, L §5.19 Coverage.
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
        if let Some(hit) = self.links.get(&key) {
            let (obs, coll) = (hit.obs.clone(), hit.coll.clone());
            self.absorb(&coll);
            return Ok(obs);
        }
        if self.link_active.contains(&key) {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("the link to {tid} reads its own observations"),
            )));
        }
        self.link_active.insert(key.clone());
        self.push();
        let judged = self.judge_links(m, tid, cut);
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
        let mut errs = Vec::new();
        for o in naming {
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
                errs.push((
                    o.id.clone(),
                    record(format!(
                        "observation {} fulfils {tid} and lacks its leg {}",
                        o.id, lg.name
                    )),
                ));
                continue;
            }
            good.push(o.id.clone());
        }
        Ok((good, errs))
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
            self.judged(m, &tid, cut)?;
        }
        Ok(self.obs_errs.get(&(cut, o.id.clone())).cloned())
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
        match self.obs_links_for(m, tid, cut) {
            Ok(good) => Ok(Some(good)),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => Ok(None),
            Err(stop) => Err(stop),
        }
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
    /// disagree on a leg's amount (refcore's `prepare`, its links). E §6.1.
    ///
    /// # Errors
    ///
    /// As [`Engine::obs_links_for`].
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
        for tid in targets {
            let good = self.obs_links_for(m, &tid, cut)?;
            let mut all: Vec<Obs> = Vec::new();
            for o in self.observations(m, cut)? {
                if good.contains(&o.id) {
                    all.push(o.clone());
                }
            }
            let mut rest = all.as_slice();
            while let [a, tail @ ..] = rest {
                rest = tail;
                pairs_warned(m, &tid, a, tail, &mut out)?;
            }
        }
        Ok(out)
    }
}

/// `WARN-LEGS` for observation `a` against each later one of another version that fulfils
/// instance `tid` and disagrees with it, each warning once (refcore's `prepare`). E §6.1.
fn pairs_warned(
    m: &Reality,
    tid: &str,
    a: &Obs,
    later: &[Obs],
    out: &mut Vec<(Warn, String)>,
) -> Result<(), Stop> {
    for b in later {
        if a.version == b.version || !disagree(a, b) {
            continue;
        }
        let text = format!(
            "the two observations of {tid} disagree: {} ({}) and {} ({})",
            a.id,
            amounts(m, a)?,
            b.id,
            amounts(m, b)?
        );
        let w = (Warn::Legs, text);
        if !out.contains(&w) {
            out.push(w);
        }
    }
    Ok(())
}

/// Whether two observations disagree on the amount of a leg both state (refcore's
/// `prepare`). E §6.1.
fn disagree(a: &Obs, b: &Obs) -> bool {
    a.legs.iter().any(|la| differs(b, la))
}

/// Whether observation `b` states leg `la`'s name with another amount. E §6.1.
fn differs(b: &Obs, la: &LegVal) -> bool {
    b.legs
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

/// The observations cut `cut` sees: per observed input, each row id's latest restatement known
/// by the cut, `none` removing it; sorted by day and id (refcore's `observations`). L §5.19.
fn seen_observations(m: &Reality, cut: Day) -> Result<Vec<Obs>, Stop> {
    let mut out = Vec::new();
    for (_, input) in m.inputs.entries() {
        let mut latest: BTreeMap<&str, (Day, &crate::model::Observation)> = BTreeMap::new();
        for o in &input.observations {
            let version = m
                .versions
                .get(&o.version)
                .ok_or_else(|| invariant("an observation of a version load didn't build"))?;
            let known = o.own_known.unwrap_or(version.known);
            if known > cut {
                continue;
            }
            if latest.get(o.key.as_str()).is_none_or(|(k, _)| known > *k) {
                latest.insert(o.key.as_str(), (known, o));
            }
        }
        for (_, (_, o)) in latest {
            if o.removed {
                continue;
            }
            out.push(observation(o)?);
        }
    }
    out.sort_by(|a, b| (a.day, &a.id).cmp(&(b.day, &b.id)));
    Ok(out)
}

/// One observation as a movement: its legs between places, each on its own day or the
/// observation's (refcore's `ObsMv`). L §5.19.
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
            day: Dated::Day(lg.day.unwrap_or(day)),
        })
        .collect();
    let mut coll = Collected::default();
    coll.versions.insert(o.version.clone());
    Ok(Obs {
        id: o.sid(),
        input: o.input.clone(),
        version: o.version.clone(),
        kind,
        day,
        legs,
        values: o.values.clone(),
        fulfils: o.fulfils.clone(),
        coll,
    })
}

/// The versions cut `cut` sees that cover each place, by version id: each place with its span,
/// the one written or the version's `governs` (refcore's `coverage`). L §5.19 Coverage.
fn seen_coverage(m: &Reality, cut: Day) -> Coverage {
    let mut out = Coverage::new();
    let mut versions: Vec<&crate::model::VersionInfo> =
        m.versions.entries().map(|(_, v)| v).collect();
    versions.sort_by(|a, b| a.id.cmp(&b.id));
    for v in versions {
        if v.covers.is_empty() || v.known > cut {
            continue;
        }
        for (place, span) in &v.covers {
            let Some(span) = span.as_ref().or_else(|| v.span_at(cut)) else {
                continue;
            };
            out.entry(place.clone())
                .or_insert_with(Vec::new)
                .push((span.clone(), v.id.clone()));
        }
    }
    out
}
