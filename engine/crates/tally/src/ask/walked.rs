//! `explain`'s forms the walk answers: a balance by its reading and the movements since, a projection by its own walk, a `low`.
//! Spec: E §10 (`explain` of a balance: its last reading with its gap role, version, source
//! and gap, or the opening, and every movement since, up to that day, grouped by day, with
//! each movement's status and id; of a projection: its holder and day t, the start balance,
//! the movements it counted with the clauses applied, the movements it left out), §6.1
//! (readings, their gaps; a pad, an anchor), §4.2 (the walk's logs: what each entry books),
//! §4.3 (a projection's walk), §4.1 step 3 (a movement's status); L §5.18 (the explain row;
//! `explain low(x, span)` without `from` explains the first day its lowest balance is reached,
//! as a balance), §5.10 (a place's balance: zero from its lifespan's start, else unknown until
//! a reading), §5.14 (`with uncertain`), §5.14.1 (an ask's line leaves out nothing), §8.2 (the
//! line's register is the read's).
//! Twin: engine/tools/refcore/ask.py (`explain_balance`, `explain_low_plain`,
//! `_explain_bal_lines`, `_explain_projection`, `_reading_text`, `_entry_text`,
//! `_reading_text_plain`, `_amount_text`, `_by_day`).
//! Invariants: the lines read the walks' logs as the read left them, a projection's its plain
//! twin's walk, never F5's tree; a line's register is its value's read alone, what the lines
//! read brings nothing; days print in order, a day's movements in walk order; a movement's
//! status is the walk's at its cut; a reading's row is its stable id (L §2.5).

use core::cmp::Ordering;

use super::answer::unresolved_word;
use super::check::Printed;
use super::explain::{invariant, plain_text};
use crate::base::Code;
use crate::book::{gap_text, instant_text};
use crate::eval::{Ctx, Engine, Env, Item};
use crate::model::{GapRole, Reality, RowInstant, Value};
use crate::num::Num;
use crate::syntax::{Ask, Bal, Expr, Low, Proj, Stop};
use crate::time::Day;
use crate::walk::{
    ALL, Amount, At, Change, Ent, LegVal, Status, Walk, own_unit, qty_value, quantity, span_last,
    state_value,
};

/// A balance read `explain` explains: `bal(…)`, plain or a projection, or `low(…)`.
#[derive(Debug, Clone, Copy)]
pub(super) enum Read<'a> {
    /// `bal(x, d)`, `bal(x, d start)`, `bal(x, u, d)`, `bal(x, d) from t`.
    Bal(&'a Bal),
    /// `low(x, a .. b)`, with or without `from t`.
    Low(&'a Low),
}

impl Read<'_> {
    /// The read's projection clauses. L §5.14.1.
    fn proj(&self) -> &Proj {
        match self {
            Read::Bal(b) => &b.proj,
            Read::Low(l) => &l.proj,
        }
    }

    /// The place read. L §5.10.
    fn place(&self) -> &Expr {
        match self {
            Read::Bal(b) => &b.place,
            Read::Low(l) => &l.place,
        }
    }
}

/// The line of `explain bal(…)` or `explain low(…)`: its value as the read gives it, then,
/// but for a refusal or a balance of no day, its derivation: a balance's from the main walk,
/// a projection's from its own walk, a `low` with no `from` as the balance of its first
/// lowest day (refcore's `explain_balance`, `explain_low_plain`). L §5.18, E §10.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a form the walk doesn't build.
pub(super) fn balance_line(
    m: &Reality,
    (ask, read, reference): (&Ask, Read<'_>, &Expr),
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    let (items, met) = engine.line(m, core::slice::from_ref(reference), env, ctx)?;
    // a bare key literal no position resolved is refused where it is read (P39)
    if let Some(place) = m.asks_unresolved.get(&ask.id) {
        unresolved_word(&items, place)?;
    }
    let Ok([item]) = <[Item; 1]>::try_from(items) else {
        return Err(invariant("an explained line with other than one value"));
    };
    let text = plain_text(m, &item)?;
    let body = match (&item, read) {
        (Item::Refused(_), _) => Vec::new(),
        (Item::Value(_), Read::Bal(b)) if b.args.is_empty() => Vec::new(),
        // L §5.18: a `low` without `from` explains the first day its lowest balance is reached
        (Item::Value(Value::Hole(_)), Read::Low(l)) if l.proj.from.is_none() => Vec::new(),
        (Item::Value(v), _) => {
            // what the lines read was read for the value already: it brings nothing more
            engine.push();
            let lines = match read {
                Read::Low(l) if l.proj.from.is_none() => low_lines(m, engine, l, v, (env, ctx)),
                Read::Bal(_) | Read::Low(_) => balance_lines(m, engine, (ask, read), (env, ctx)),
            };
            engine.pop_alone();
            match lines {
                Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                    vec![format!("refused: {}", r.text())]
                }
                other => other?,
            }
        }
    };
    Ok(Printed {
        text,
        body: body.into_iter().map(|l| format!("  {l}")).collect(),
        bare: false,
        met,
        laws: None,
    })
}

/// A balance read's place and unit; the line where either is a hole (refcore's
/// `_explain_bal_lines`, its head). L §5.10.
fn place_unit(
    m: &Reality,
    engine: &mut Engine,
    read: Read<'_>,
    (env, ctx): (&Env, Ctx),
) -> Result<Result<(String, String), String>, Stop> {
    let place = match engine.place_arg(m, read.place(), env, ctx)? {
        Err(h) => return Ok(Err(format!("the place is a hole: {}", h.text()))),
        Ok(p) => p,
    };
    let unit = match read {
        Read::Bal(b) => engine.bal_unit(m, b, &place, env, ctx)?,
        Read::Low(_) => Ok(own_unit(m, &place)?),
    };
    match unit {
        Err(h) => Ok(Err(format!("the unit is a hole: {}", h.text()))),
        Ok(u) => Ok(Ok((place, u))),
    }
}

/// A balance read's derivation: a plain balance's place and instant, where it starts (its
/// last reading, or the opening), then every movement the walk booked there since, through
/// that instant, by day; a projection's from its own walk (refcore's `_explain_bal_lines`).
/// E §10, §6.1.
fn balance_lines(
    m: &Reality,
    engine: &mut Engine,
    (ask, read): (&Ask, Read<'_>),
    (env, ctx): (&Env, Ctx),
) -> Result<Vec<String>, Stop> {
    let (place, unit) = match place_unit(m, engine, read, (env, ctx))? {
        Err(line) => return Ok(vec![line]),
        Ok(pu) => pu,
    };
    let (first, end) = match read {
        Read::Bal(b) => {
            let Some(day_arg) = b.args.last() else {
                return Ok(Vec::new());
            };
            let d = engine.ev(m, day_arg, env, ctx)?;
            if matches!(d, Value::Hole(_) | Value::Window(_)) {
                return Ok(Vec::new());
            }
            let end = super::date_of(&d, "the day of bal")?;
            (end, end)
        }
        Read::Low(l) => {
            let Ok(span) = engine.span_value(m, &l.span, env, ctx)? else {
                return Ok(Vec::new());
            };
            let first = span
                .start()
                .ok_or_else(|| invariant("a span with no start"))?;
            (first, span_last(m, &span)?)
        }
    };
    if let Some(from) = &read.proj().from {
        let t = engine.ev(m, from, env, ctx)?;
        if matches!(t, Value::Hole(_)) {
            return Ok(Vec::new());
        }
        let t = super::date_of(&t, "the day of a projection")?;
        let low = matches!(read, Read::Low(_)).then_some(first);
        let unc = read.proj().uncertain || ctx.unc;
        return projection_lines(m, engine, (ask, &place, &unit), (t, low, end), (unc, ctx));
    }
    let start = matches!(read, Read::Bal(b) if b.start);
    let uncertain = read.proj().uncertain;
    let id = engine.main_walk(m, ctx.cut)?;
    plain_lines(m, engine, id, (&place, &unit), (end, start, uncertain))
}

/// `explain low(x, a .. b)` with no `from`: the lowest of the plain end-of-day balances
/// `lowest` and the first run of days it is reached on, then the balance at that run's first
/// day (refcore's `explain_low_plain`). L §5.18, §5.14.1; E §10.
fn low_lines(
    m: &Reality,
    engine: &mut Engine,
    l: &Low,
    lowest: &Value,
    (env, ctx): (&Env, Ctx),
) -> Result<Vec<String>, Stop> {
    let Ok((place, unit)) = place_unit(m, engine, Read::Low(l), (env, ctx))? else {
        return Ok(Vec::new());
    };
    let Ok(span) = engine.span_value(m, &l.span, env, ctx)? else {
        return Ok(Vec::new());
    };
    let first = span
        .start()
        .ok_or_else(|| invariant("a span with no start"))?;
    let last = span_last(m, &span)?;
    let (target, _) = quantity(m, lowest)?;
    let id = engine.main_walk(m, ctx.cut)?;
    let walk = engine.walk(id)?;
    // the first run of days whose balance is the lowest, no day between them off it
    let mut run: Option<(Day, Day)> = None;
    let mut day = Some(first);
    while let Some(d) = day.filter(|d| *d <= last) {
        let st = walk.pstate(m, &place, &unit, At::end(d));
        let on = match st.known() {
            Some(q) => q.compare(&target).map_err(crate::eval::num_stop)? == Ordering::Equal,
            None => false,
        };
        match (on, run) {
            (true, None) => run = Some((d, d)),
            (true, Some((a, _))) => run = Some((a, d)),
            (false, Some(_)) => break,
            (false, None) => {}
        }
        day = d.next();
    }
    let Some((d1, d2)) = run else {
        return Err(Stop::NotBuilt(
            "explain of a low whose lowest balance is on no day of its span",
        ));
    };
    let through = if d2 == d1 {
        String::new()
    } else {
        format!(" (through {})", d2.text())
    };
    let mut out = vec![format!(
        "lowest end-of-day balance of {place} on {} .. {}: {}, on {}{through}",
        first.text(),
        last.text(),
        plain_text(m, &Item::Value(lowest.clone()))?,
        d1.text()
    )];
    let uncertain = l.proj.uncertain;
    out.extend(plain_lines(
        m,
        engine,
        id,
        (&place, &unit),
        (d1, false, uncertain),
    )?);
    Ok(out)
}

/// A plain balance's derivation at the start or the end of day `end`, from main walk `id`:
/// where it starts, then the movements since, by day; an uncertain movement's legs too under
/// `with uncertain` (refcore's `_explain_bal_lines`, its plain form). E §10, §6.1; L §5.14.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a walk the engine lacks; `NotBuilt` for a reading at an instant
/// other than a day's start or end.
pub(super) fn plain_lines(
    m: &Reality,
    engine: &Engine,
    id: usize,
    (place, unit): (&str, &str),
    (end, start, uncertain): (Day, bool, bool),
) -> Result<Vec<String>, Stop> {
    let walk = engine.walk(id)?;
    let (key, edge) = if start {
        (At::start(end), "start")
    } else {
        (At::end(end), "end")
    };
    let mut out = vec![format!(
        "balance of {place} ({unit}) at the {edge} of {}",
        end.text()
    )];
    let (st, last) = walk.state(m, place, unit, key);
    let log = walk.entries(place, unit);
    let reset = match st.rst {
        Some(r) => log.get(r).map(|e| (r, e)),
        None => None,
    };
    let since = reset.map(|(_, e)| e.at);
    out.push(match reset {
        Some((_, e)) => format!("from: {}", reading_text(m, walk, (place, unit), e)?),
        None => opening_text(m, walk, (place, unit)),
    });
    // the entries after the last reset, through the last the state counts
    let first = match reset {
        Some((r, _)) => r.checked_add(1).ok_or_else(|| invariant("a log's place"))?,
        None => 0,
    };
    let mut ents: Vec<(&Ent, &str)> = Vec::new();
    if let Some(last) = last {
        let counted = last
            .checked_add(1)
            .ok_or_else(|| invariant("a log's length"))?;
        ents.extend(log.iter().take(counted).skip(first).map(|e| (e, unit)));
    }
    for e in walk.entries(place, ALL) {
        let after = match since {
            Some(s) => e.at > s,
            None => true,
        };
        if after && e.at <= key {
            ents.push((e, ALL));
        }
    }
    let mut rows = entry_rows(m, engine, walk, ents)?;
    if uncertain {
        uncertain_rows(m, walk, (place, end, since), &mut rows)?;
    }
    out.extend(by_day(rows));
    Ok(out)
}

/// A projection's derivation from its own walk, the plain twin's, never F5's tree: its place,
/// day t and last day (a `low`'s first day `low`), its holder, its clauses, where it starts,
/// the movements it counted by day, those it left out, and a `low`'s lowest day (refcore's
/// `_explain_projection`). E §10, §4.3; L §5.14.1.
fn projection_lines(
    m: &Reality,
    engine: &mut Engine,
    (ask, place, unit): (&Ask, &str, &str),
    (t, low, end): (Day, Option<Day>, Day),
    (unc, ctx): (bool, Ctx),
) -> Result<Vec<String>, Stop> {
    if t < m.start {
        // no point of the walk it continues to start from: nothing to explain, the walk
        // read as the projection reads it (refcore's `proj_walk`, its hole)
        engine.main_walk(m, ctx.cut)?;
        return Ok(Vec::new());
    }
    let id = engine.projection_walk(m, ctx, t, end)?;
    let walk = engine.walk(id)?;
    let head = match low {
        Some(a) => format!(
            "projection of {place} ({unit}) from {}, lowest end-of-day balance on {} .. {}",
            t.text(),
            a.text(),
            end.text()
        ),
        None => format!(
            "projection of {place} ({unit}) from {} to the end of {}",
            t.text(),
            end.text()
        ),
    };
    let with = if unc { "with" } else { "without" };
    let opened = walk.default_state(m, place, unit);
    let start = match state_value(m, &opened, unit, &walk.anchor_question(place)) {
        Ok(v) => v.text(m).map_err(crate::eval::num_stop)?,
        Err(Stop::Refused(r)) => format!("refused: {}", r.text()),
        Err(other) => return Err(other),
    };
    let mut out = vec![
        head,
        format!(
            "holder: ask {} (an ask line leaves out nothing, L §5.14.1)",
            ask.id
        ),
        format!("clauses: windows none, random none, {with} uncertain"),
        format!("start: {start} at the start of {}", t.text()),
    ];
    let key = At::end(end);
    let mut ents: Vec<(&Ent, &str)> = Vec::new();
    for u in [unit, ALL] {
        ents.extend(
            walk.entries(place, u)
                .iter()
                .filter(|e| e.at <= key)
                .map(|e| (e, u)),
        );
    }
    let mut rows = entry_rows(m, engine, walk, ents)?;
    if unc {
        // the days from t on: a leg's day whose end is after t's start
        uncertain_rows(m, walk, (place, end, Some(At::start(t))), &mut rows)?;
    }
    out.push("counted:".to_owned());
    out.extend(by_day(rows).into_iter().map(|l| format!("  {l}")));
    let left = walk
        .proj
        .iter()
        .flat_map(|p| p.counted.values())
        .flatten()
        .any(|d| t <= *d && *d <= end);
    if left {
        // an ask's line leaves out nothing; only a def's projection lists instances
        return Err(Stop::NotBuilt(
            "explain of a projection that leaves movements out",
        ));
    }
    out.push("left out: none".to_owned());
    if let Some(a) = low {
        let mut best: Option<(Day, Num)> = None;
        let mut day = Some(a);
        while let Some(d) = day.filter(|d| *d <= end) {
            if let Some(q) = walk.pstate(m, place, unit, At::end(d)).known() {
                let lower = match &best {
                    None => true,
                    Some((_, b)) => q.compare(b).map_err(crate::eval::num_stop)? == Ordering::Less,
                };
                if lower {
                    best = Some((d, q.clone()));
                }
            }
            day = d.next();
        }
        if let Some((d, q)) = best {
            out.push(format!(
                "lowest: {} at the end of {}",
                quantity_text(m, &q, unit)?,
                d.text()
            ));
        }
    }
    Ok(out)
}

/// The entries of a balance's logs, each with its log's unit, in walk order, as `(day, text)`
/// rows; an entry that only marks the register makes none (refcore's `_entry_text` over the
/// sorted entries). E §10.
fn entry_rows(
    m: &Reality,
    engine: &Engine,
    walk: &Walk,
    mut ents: Vec<(&Ent, &str)>,
) -> Result<Vec<(Day, String)>, Stop> {
    ents.sort_by_key(|(e, _)| e.at);
    let mut rows = Vec::new();
    for (e, u) in ents {
        if let Some(t) = entry_text(m, engine, walk, e, u)? {
            rows.push((e.at.day, t));
        }
    }
    Ok(rows)
}

/// Where a balance with no reading before it starts: zero from its lifespan's start, when
/// the walk opens it so, else nothing (refcore's `_explain_bal_lines`, its `default`).
/// L §5.10.
fn opening_text(m: &Reality, walk: &Walk, (place, unit): (&str, &str)) -> String {
    if walk.default_state(m, place, unit).val.is_none() {
        return "from: no reading or opening before it".to_owned();
    }
    let start = m
        .things
        .get(place)
        .and_then(|th| m.lifespan(th, (walk.cut, walk.world)))
        .and_then(|sp| sp.start());
    let start = start.map_or_else(|| "None".to_owned(), Day::text);
    format!("from: 0 at the start of its lifespan, {start}")
}

/// What a reset entry is, by what booked it: an opening, an anchor, or a pad's reading end
/// (refcore's `Ent.note`). E §6.1.
fn reading_word(e: &Ent) -> &'static str {
    if e.reg.iter().any(|r| r.word == "pad") {
        "pad"
    } else if e.reg.iter().any(|r| r.word == "anchor") {
        "anchor"
    } else {
        "opening"
    }
}

/// A reading a balance starts from: `<what> <row> = <value>, <instant> (version v, source s[,
/// locator "l"])`, an anchor's or a pad's gap after (refcore's `_reading_text`). E §10, §6.1.
fn reading_text(
    m: &Reality,
    walk: &Walk,
    (place, unit): (&str, &str),
    e: &Ent,
) -> Result<String, Stop> {
    let value = reset_value(m, e, unit)?;
    let word = reading_word(e);
    let Some((sid, seen)) = e.coll.as_ref().and_then(|c| c.rows.iter().next()) else {
        return Ok(format!("{word} = {value}"));
    };
    let cite = &m
        .versions
        .get(&seen.version)
        .ok_or_else(|| invariant("a reading of a version load didn't build"))?
        .cite;
    let mut s = format!(
        "{word} {sid} = {value}, {} (version {}, source {cite}",
        reading_instant(m, sid)?,
        seen.version
    );
    if let Some(l) = &seen.locator {
        s = format!("{s}, locator \"{l}\"");
    }
    s.push(')');
    if word == "opening" {
        return Ok(s);
    }
    // the gap the walk met at the reading's instant (an anchor's at its point, a pad's
    // reading end just after it)
    let gaps = walk.gaps.get(&(place.to_owned(), e.at.day));
    let Some(gap) = gaps
        .into_iter()
        .flatten()
        .find(|g| g.at.step == e.at.step && g.unit == unit)
    else {
        return Ok(s);
    };
    let gap_shown = gap_text(m, &gap.gap, unit)?;
    let role = match m.versions.get(&seen.version) {
        Some(v) => v
            .gap_roles
            .iter()
            .find(|(p, _)| p == place)
            .map(|(_, role)| role),
        None => None,
    };
    Ok(match role {
        Some(GapRole::Pad(other)) => {
            let pad = e.mv.as_deref().unwrap_or("pad");
            format!("{s}, gap role pad from {other}, gap {gap_shown} ({pad})")
        }
        Some(GapRole::Anchor) => format!("{s}, gap role anchor, gap {gap_shown}"),
        None => s,
    })
}

/// A reading row's instant as text, `2026-01-01 start` (refcore's `instant_text`). L §4.6.
fn reading_instant(m: &Reality, sid: &str) -> Result<String, Stop> {
    // an opening's row, else a stated reading's
    let mut found = None;
    for input in ["opening", "stated"] {
        if found.is_none()
            && let Some(info) = m.inputs.get(input)
        {
            found = info.rows.iter().find(|r| r.sid == sid);
        }
    }
    let row = found.ok_or_else(|| invariant("a reading's row load didn't build"))?;
    match &row.instant {
        Some(RowInstant::At(day, Some(edge))) => Ok(instant_text(*day, *edge)),
        Some(_) | None => Err(Stop::NotBuilt(
            "explain of a reading at an instant other than a day's start or end",
        )),
    }
}

/// A reset entry's value: the balance it states, or the hole it is (refcore's `fmt(e.val)`,
/// `fmt_hole`). L §5.18.
fn reset_value(m: &Reality, e: &Ent, unit: &str) -> Result<String, Stop> {
    match &e.change {
        Change::Reset { qs: Some(h), .. } => Ok(h.text()),
        Change::Reset { val, qs: None } => quantity_text(m, val, unit),
        Change::Delta(_) | Change::Hole(_) | Change::Perm(_) | Change::Bad(_) | Change::Mark => {
            Err(invariant("a reading that is no reset"))
        }
    }
}

/// One entry of a place's log in unit `unit`: its movement's id (with the leg, for a booked
/// leg or a hole leg but a pad's), its status, and what it moved; none for an entry that only
/// marks the register (refcore's `_entry_text`). E §10.
fn entry_text(
    m: &Reality,
    engine: &Engine,
    walk: &Walk,
    e: &Ent,
    unit: &str,
) -> Result<Option<String>, Stop> {
    let pad = e.reg.iter().any(|r| r.word == "pad");
    let mut head = e.mv.clone().unwrap_or_else(|| "the walk".to_owned());
    if let (Change::Delta(_) | Change::Hole(_), Some(leg), false) = (&e.change, &e.leg, pad) {
        head = format!("{head}.{leg}");
    }
    let status = match e.mv.as_deref() {
        Some(_) if pad => Some("pad"),
        Some(id) => status_word(engine, walk, id),
        None => None,
    };
    if let Some(status) = status {
        head = format!("{head} ({status})");
    }
    let text = match &e.change {
        Change::Delta(f) => {
            let sign = if f.is_negative() || f.is_zero() {
                ""
            } else {
                "+"
            };
            format!("{head}: {sign}{}", quantity_text(m, f, unit)?)
        }
        Change::Hole(h) | Change::Perm(h) => format!("{head}: hole: {}", h.text()),
        Change::Bad(r) => format!("{head}: refused: {}", r.text()),
        Change::Reset { .. } => {
            let sid = e
                .coll
                .as_ref()
                .and_then(|c| c.rows.keys().next())
                .map_or("", String::as_str);
            format!("{} {sid} = {}", reading_word(e), reset_value(m, e, unit)?)
        }
        Change::Mark => return Ok(None),
    };
    Ok(Some(text))
}

/// The status of movement `id` of walk `walk`: a decision's run, a plan's instance, an
/// observation or an adjustment; none for one the walk doesn't name (refcore's
/// `mv.status`). E §4.1 step 3, §4.2.
fn status_word(engine: &Engine, walk: &Walk, id: &str) -> Option<&'static str> {
    let run = walk.run_ids.get(id).and_then(|i| walk.runs.get(*i));
    let status = match run {
        Some(r) => Some(r.inst.status),
        None => walk.insts.iter().find(|i| i.id == id).map(|i| i.status),
    };
    if let Some(status) = status {
        return status_text(status);
    }
    if walk.obs.iter().any(|o| o.obs.id == id) {
        return Some("observed");
    }
    let adjusted = engine.adjustments.values().flatten().any(|a| a.id == id);
    adjusted.then_some("adjustment")
}

/// A status's word (refcore's status texts). E §4.1 step 3.
pub(super) fn status_text(status: Status) -> Option<&'static str> {
    match status {
        Status::Open => None,
        Status::None => Some("none"),
        Status::LeftOut => Some("leftout"),
        Status::Refused => Some("refused"),
        Status::Hole => Some("hole"),
        Status::Decision => Some("decision"),
        Status::Uncertain => Some("uncertain"),
        Status::Modelled => Some("modelled"),
        Status::Past => Some("past"),
        Status::Plan => Some("plan"),
    }
}

/// The uncertain instances' legs at `place` a balance `with uncertain` counts through `end`,
/// on days whose end is after `since`: a plain balance's reading, a projection's start
/// (refcore's `_explain_bal_lines`, its `extra`, and `_explain_projection`). L §5.14.
fn uncertain_rows(
    m: &Reality,
    walk: &Walk,
    (place, end, since): (&str, Day, Option<At>),
    rows: &mut Vec<(Day, String)>,
) -> Result<(), Stop> {
    for inst in &walk.uncertain {
        let day = inst.day.as_ref().and_then(crate::walk::Dated::day);
        for lg in &inst.legs {
            let Some(gd) = lg.day.day().or(day) else {
                continue;
            };
            if gd > end || since.is_some_and(|s| At::end(gd) <= s) {
                continue;
            }
            for (side, at_end) in [(Side::Out, &lg.from), (Side::In, &lg.to)] {
                if at_end.place() == Some(place) {
                    let amount = amount_text(m, lg, side)?;
                    rows.push((gd, format!("{}.{} (uncertain): {amount}", inst.id, lg.name)));
                }
            }
        }
    }
    Ok(())
}

/// The end of a leg a place is at: the one it leaves, or the one it enters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// The leg's `from`.
    Out,
    /// The leg's `to`.
    In,
}

/// A leg's amount at its end `side`, signed for the place there: `+x` in, `-x` out, its
/// hole, its refusal (refcore's `_amount_text`). E §10.
fn amount_text(m: &Reality, lg: &LegVal, side: Side) -> Result<String, Stop> {
    match &lg.amount {
        Amount::Hole(h) => Ok(format!("hole: {}", h.text())),
        Amount::Refused(r) => Ok(format!("refused: {}", r.text())),
        Amount::Qty(q) => {
            let signed = match side {
                Side::In => q.clone(),
                Side::Out => q.checked_neg().map_err(crate::eval::num_stop)?,
            };
            let sign = if signed.is_negative() || signed.is_zero() {
                ""
            } else {
                "+"
            };
            let unit = lg
                .unit
                .as_deref()
                .ok_or_else(|| invariant("a leg's amount with no unit"))?;
            Ok(format!("{sign}{}", quantity_text(m, &signed, unit)?))
        }
    }
}

/// A quantity of unit `unit` as `explain` prints it. L §5.18.
pub(super) fn quantity_text(m: &Reality, q: &Num, unit: &str) -> Result<String, Stop> {
    if unit == ALL {
        return Err(invariant("a quantity booked in every unit"));
    }
    qty_value(m, q, unit)?
        .text(m)
        .map_err(crate::eval::num_stop)
}

/// Rows of `(day, text)` grouped by day, in day order, each day's in the order given; `no
/// movement` for none (refcore's `_by_day`). E §10.
fn by_day(mut rows: Vec<(Day, String)>) -> Vec<String> {
    if rows.is_empty() {
        return vec!["no movement".to_owned()];
    }
    rows.sort_by_key(|r| r.0);
    let mut out = Vec::new();
    let mut current = None;
    for (day, text) in rows {
        if current != Some(day) {
            out.push(format!("{}:", day.text()));
            current = Some(day);
        }
        out.push(format!("  {text}"));
    }
    out
}
