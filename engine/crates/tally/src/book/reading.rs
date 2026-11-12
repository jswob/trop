//! Balance readings: the `opening` and `stated` rows the cut sees, and what a stated reading does to its place, a pad or an anchor, with its gap.
//! Spec: L §5.19 (balance readings and their gaps: `pad x from y`, `anchor x`; a reading whose
//! value is `none` states nothing; the read rule decides who sees it), §4.6 (a reading's
//! instant), §5.10 (`HOLE-ANCHOR`; the sign rule tests the reading's own place at the
//! reading's point, a pad's in the point's group with its leg, an anchor's through
//! `walk/day.rs`; tally 2.1.2); E §6.1 (Readings: the gap is the reading less the booked
//! balance at its instant; a pad is a movement of `pads`, an anchor sets the balance; holes),
//! §4.2 step 0 and step 4 (where readings apply), step 5 (the sign rule at a reading's point);
//! L §6.4 (the readings a walk's world sees).
//! Twin: engine/tools/refcore/moves.py (`readings`, `instant_text`, `PadMv`) and walk.py
//! (`apply_reading`, `booked_coll`, `sub`, `st_value`, `gtext`).
//! Invariants: a reading applies only before the cut but an opening at the run's start; after
//! a reading the place's balance is the reading; a pad books only its other end, the reading's
//! end is the reset, which carries the pad's flow there; a gap is used where it is met (worth,
//! the register, a pad's amount) and kept per (place, day) for `gap(x, d)`; the ledger's
//! balance line computes its own from the reading (`book/ledger.rs`).

use std::collections::BTreeMap;

use crate::base::{Code, Load, Refusal};
use crate::eval::{Collected, Engine, Entry, Rows};
use crate::hole::{Hole, spread};
use crate::model::{GapRole, Reality, RowInstant, Shown, Ty, Unit, World};
use crate::num::Num;
use crate::read::row_known;
use crate::syntax::{Edge, Stop};
use crate::time::Day;
use crate::walk::{
    Amount, At, Change, Dated, End, Ends, Ent, GapAt, LegVal, Reading, St, invariant, qty_value,
};

/// A reading the cut sees: its day, its edge, its place, its input, and what it states (none
/// for a row whose value is `none`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    /// Its day.
    pub day: Day,
    /// Its edge: the day's start or end.
    pub edge: Edge,
    /// The place.
    pub place: String,
    /// The input: `opening` or `stated`.
    pub input: String,
    /// What it states; none for a row whose value is `none`.
    pub reading: Option<Reading>,
}

/// A pad: a movement of `pads` that takes up a statement's gap (refcore's `PadMv`). E §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pad {
    /// Its id, `pads:<version>:<place>@<instant>`.
    pub id: String,
    /// Its one leg, from the pad's other place or to it.
    pub leg: LegVal,
    /// What its amount rests on: the booked balance's register and the reading's.
    pub coll: Collected,
}

/// A reading row kept as the latest of its key: when it became known, its day and edge, the
/// row (L §5.19).
type Kept = (Day, Day, Edge, crate::model::Row);

/// The readings cut `cut` sees in world `world`: each `opening` and `stated` row known by
/// the cut, of a version the world sees, a restated one replaced as `replace` says (a
/// scenario's over its base's whatever their `known` dates, as `standing` takes it), sorted
/// by place, instant and input, each with its row among what it met where `rows` says so
/// (refcore's `readings`, `world_ok`). L §5.19, §4.6 step 2, §6.4; E §10.
///
/// # Errors
///
/// A row load left without its version; `NotBuilt` for a reading `after` a movement or at a
/// day with no edge, and for a place of unit `any`.
pub fn readings(m: &Reality, (cut, world): (Day, World), rows: Rows) -> Result<Vec<Seen>, Stop> {
    let mut cands: BTreeMap<(String, String, String), Vec<Kept>> = BTreeMap::new();
    for input in ["opening", "stated"] {
        let Some(info) = m.inputs.get(input) else {
            continue;
        };
        for row in &info.rows {
            if !m.scenarios.sees(world, Some(&row.version)) {
                continue;
            }
            let known = row_known(m, row)?;
            if known > cut {
                continue;
            }
            let Some(crate::model::Pat::Key(crate::model::KeyVal::Thing(place))) =
                row.pattern.first()
            else {
                return Err(Stop::NotBuilt("a reading of a place not named"));
            };
            let (day, edge) = match &row.instant {
                Some(RowInstant::At(day, Some(edge))) => (*day, *edge),
                Some(_) | None => {
                    return Err(Stop::NotBuilt(
                        "a reading at an instant other than a day's start or end",
                    ));
                }
            };
            let key = (place.clone(), instant_text(day, edge), input.to_owned());
            cands
                .entry(key)
                .or_insert_with(Vec::new)
                .push((known, day, edge, row.clone()));
        }
    }
    let mut out = Vec::with_capacity(cands.len());
    for ((place, instant, input), rows_at) in cands {
        let Some((_, day, edge, row)) = standing(m, rows_at) else {
            continue;
        };
        let mut reading = reading_of(m, &input, &place, &instant, &row)?;
        // the row as `inputs-of` lists it, kept by an engine answering one (E §10)
        if rows == Rows::Kept
            && let Some(r) = &mut reading
        {
            r.coll
                .note_row(&row.sid, &row.version, row.locator.as_ref());
        }
        out.push(Seen {
            day,
            edge,
            place,
            input,
            reading,
        });
    }
    Ok(out)
}

/// The row that stands among the readings `rows_at` of one place, instant and input: of the
/// rows no `replace` row beats, the latest known, the first at a tie (refcore's `choose`,
/// its replacement, as `read/rows.rs` `unreplaced` weighs other rows). L §4.6 step 2, §6.4.
fn standing(m: &Reality, rows_at: Vec<Kept>) -> Option<Kept> {
    let mut best: Option<usize> = None;
    for (at, (known, .., row)) in rows_at.iter().enumerate() {
        if replaced(m, &rows_at, (*known, row)) {
            continue;
        }
        let later = best
            .and_then(|b| rows_at.get(b))
            .is_none_or(|(k, ..)| known > k);
        if later {
            best = Some(at);
        }
    }
    best.and_then(|b| rows_at.into_iter().nth(b))
}

/// Whether a `replace` row among `rows_at` beats row `row`, known on `known`: one of a layer
/// above its own, whatever the two rows' `known` dates (a branch stands above its base), or
/// one of its own layer known after it. L §4.6 step 2, §6.4.
fn replaced(m: &Reality, rows_at: &[Kept], (known, row): (Day, &crate::model::Row)) -> bool {
    let layer = m.scenarios.layer(Some(&row.version));
    for (k, .., other) in rows_at {
        if !other.replace {
            continue;
        }
        let above = m.scenarios.layer(Some(&other.version));
        if above > layer || (above == layer && *k > known) {
            return true;
        }
    }
    false
}

/// A reading's place among the readings `walked` a walk's cut sees, as the walk ranks them on
/// their day: how many come before it by place, instant and input (refcore's walk key, its
/// text). The reading needn't be one of them: one only a later cut sees takes the place it
/// would have. E §4.2 step 0, §6.3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` past four billion readings.
pub fn reading_rank(walked: &[Seen], seen: &Seen) -> Result<u32, Stop> {
    let key = (
        seen.place.as_str(),
        instant_text(seen.day, seen.edge),
        seen.input.as_str(),
    );
    let mut before: u32 = 0;
    for other in walked {
        let theirs = (
            other.place.as_str(),
            instant_text(other.day, other.edge),
            other.input.as_str(),
        );
        if theirs < key {
            before = before
                .checked_add(1)
                .ok_or_else(|| invariant("a reading's rank"))?;
        }
    }
    Ok(before)
}

/// Each place's first reading day among the readings the cut and the world of `by` see,
/// for `HOLE-ANCHOR` (refcore's `anchor_question`, its days). L §8.1, §6.4.
///
/// # Errors
///
/// As [`readings`].
pub fn first_readings(m: &Reality, by: (Day, World)) -> Result<BTreeMap<String, Day>, Stop> {
    let mut out: BTreeMap<String, Day> = BTreeMap::new();
    for seen in readings(m, by, Rows::Unkept)? {
        let first = out.entry(seen.place).or_insert(seen.day);
        *first = (*first).min(seen.day);
    }
    Ok(out)
}

/// A reading's instant as text, `2026-09-30 end` (refcore's `instant_text`). L §4.6.
pub(crate) fn instant_text(day: Day, edge: Edge) -> String {
    let word = match edge {
        Edge::Start => "start",
        Edge::End => "end",
    };
    format!("{} {word}", day.text())
}

/// A reading's row as the walk applies it: its value in the place's unit, a stated hole, or
/// none for a row whose value is `none` (refcore's `apply_reading`, its value). L §5.19.
fn reading_of(
    m: &Reality,
    input: &str,
    place: &str,
    instant: &str,
    row: &crate::model::Row,
) -> Result<Option<Reading>, Stop> {
    let unit = m
        .things
        .get(place)
        .and_then(|t| m.place_unit(t))
        .filter(|u| u != "any")
        .ok_or(Stop::NotBuilt("a reading of a place of unit any"))?;
    let ty = if m.currencies.contains_key(&unit) {
        Ty::Money(unit.clone())
    } else {
        let u = Unit::make(vec![(unit.clone(), 1)]).map_err(crate::eval::num_stop)?;
        Ty::Num(u, Shown::Plain)
    };
    let mut coll = Collected::default();
    coll.versions.insert(row.version.clone());
    if let Some(reason) = &row.assumed {
        coll.entries.insert(Entry {
            word: "assumed",
            text: reason.clone(),
            figure: String::new(),
        });
    }
    let figure = format!("{input}[{place}]");
    let value = match crate::eval::convert_row(m, &row.value, &ty)? {
        Err(text) => Err(Hole::asked(&text, &figure)),
        Ok(crate::model::Value::None) => return Ok(None),
        Ok(v) => Ok(crate::walk::quantity(m, &v)?.0),
    };
    Ok(Some(Reading {
        input: input.to_owned(),
        place: place.to_owned(),
        unit,
        value,
        coll,
        version: row.version.clone(),
        instant: instant.to_owned(),
    }))
}

impl Engine {
    /// A `stated` reading at its point: its gap against the booked balance, then, by its
    /// version's gap role, an anchor that sets the balance or a pad that books the gap from
    /// the other place (refcore's `apply_reading`). E §6.1, L §5.19.
    ///
    /// # Errors
    ///
    /// `LOAD-GAPROLE` for a version that states the reading with neither pad nor anchor; a
    /// number error.
    pub(crate) fn apply_stated(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        r: &Reading,
    ) -> Result<(), Stop> {
        if at.day >= self.walk(id)?.cut {
            return Ok(());
        }
        let version = m
            .versions
            .get(&r.version)
            .ok_or_else(|| invariant("a reading of a version load didn't build"))?;
        let Some(role) = version
            .gap_roles
            .iter()
            .find(|(p, _)| *p == r.place)
            .map(|(_, g)| g.clone())
        else {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::GapRole),
                format!(
                    "{} states balance readings of {} with neither pad nor anchor",
                    r.version, r.place
                ),
            )));
        };
        let walk = self.walk(id)?;
        let before = walk.pstate(m, &r.place, &r.unit, at);
        let booked = booked_amount(&before, &walk.anchor_question(&r.place));
        let gap = gap_of(&r.value, &booked)?;
        // the gap rests on what the booked balance rests on, and on the reading's row (L §8.2);
        // so does a pad, whose amount it is; kept for `gap(x, d)` (L §5.19)
        let mut bc = self.booked_coll(m, id, &r.place, &r.unit, at)?;
        bc.merge(&r.coll);
        let read = self.walk_mut(id)?;
        read.gaps
            .entry((r.place.clone(), at.day))
            .or_insert_with(Vec::new)
            .push(GapAt {
                at,
                gap: gap.clone(),
                unit: r.unit.clone(),
                coll: bc.clone(),
            });
        // E §9 item 2: an anchor's gap, or a pad's from outside, moves worth; a pad from an
        // inside place moves it between two inside places
        if read.inside(m, &r.place) {
            let moves = match &role {
                GapRole::Anchor => true,
                GapRole::Pad(other) => !read.inside(m, other),
            };
            if moves {
                let q = match &gap {
                    Amount::Qty(g) => Some(g),
                    Amount::Hole(_) | Amount::Refused(_) => None,
                };
                read.worth_add(&r.unit, q)?;
            }
        }
        let (val, qs) = match &r.value {
            Ok(v) => (v.clone(), None),
            Err(h) => (Num::zero(), Some(h.clone())),
        };
        match role {
            GapRole::Anchor => {
                let stated = match &r.value {
                    Ok(v) => qty_text(m, v, &r.unit)?,
                    Err(h) => h.text(),
                };
                let text = format!(
                    "{} {}, stated {stated}, difference {}",
                    r.place,
                    r.instant,
                    gap_text(m, &gap, &r.unit)?
                );
                let mut ent = Ent::new(at, Change::Reset { val, qs });
                ent.coll = Some(r.coll.clone());
                ent.reg = vec![Entry {
                    word: "anchor",
                    text,
                    figure: String::new(),
                }];
                let anchored = self.walk_mut(id)?;
                anchored.add(m, &r.place, &r.unit, ent)?;
                anchored.sign_anchor(m, at, r)
            }
            GapRole::Pad(other) => {
                let pad = make_pad(r, &other, &gap, bc, at)?;
                let text = format!(
                    "{} {}, {} of {}",
                    r.place,
                    r.instant,
                    pad.id,
                    gap_text(m, &gap, &r.unit)?
                );
                let reg = vec![Entry {
                    word: "pad",
                    text,
                    figure: String::new(),
                }];
                let ends = Ends {
                    from: pad.leg.from.place() == Some(other.as_str()),
                    to: pad.leg.to.place() == Some(other.as_str()),
                    whole: true,
                };
                self.book_leg_ends(m, id, (&pad.id, &pad.leg), at, (&reg, &pad.coll), ends)?;
                let padded = self.walk_mut(id)?;
                let key = (pad.id.clone(), pad.leg.name.clone());
                let refused = padded.bad_legs.contains_key(&key);
                if let (Amount::Qty(g), false) = (&gap, refused) {
                    let sum = padded
                        .conserved
                        .entry(r.unit.clone())
                        .or_insert_with(Num::zero);
                    *sum = sum.checked_add(g).map_err(crate::eval::num_stop)?;
                    // L §5.10, E §4.2 step 5 (tally 2.1.2): the reading's own place is tested
                    // at its point too, in the point's group with the pad's leg
                    let Some(group) = padded.open_group(at) else {
                        return Err(Stop::NotBuilt(
                            "a reading after a decision (the sign rule's point 2)",
                        ));
                    };
                    let at_place = group.entry((r.place.clone(), r.unit.clone()));
                    at_place
                        .or_insert_with(Vec::new)
                        .push((pad.id.clone(), pad.leg.clone()));
                }
                let sub = at
                    .sub
                    .checked_add(1)
                    .ok_or_else(|| invariant("a reading's point"))?;
                let mut ent = Ent::new(at.with_sub(sub), Change::Reset { val, qs });
                ent.coll = Some(r.coll.clone());
                ent.reg = reg;
                ent.mv = Some(pad.id.clone());
                // the reading's end of the pad, which `flows` of the place counts: the gap, a
                // number or a hole (refcore's reset `flow`)
                ent.flow = match gap {
                    Amount::Refused(_) => None,
                    Amount::Qty(_) | Amount::Hole(_) => Some(gap),
                };
                padded.add(m, &r.place, &r.unit, ent)
            }
        }
    }

    /// The register of a place's booked balance at a point: its log's since the last reset,
    /// and the adjustments with a remainder there (refcore's `booked_coll`). E §6.1, L §8.2.
    fn booked_coll(
        &mut self,
        m: &Reality,
        id: usize,
        place: &str,
        unit: &str,
        at: At,
    ) -> Result<Collected, Stop> {
        let mut out = Collected::default();
        let (_, last) = self.walk(id)?.state(m, place, unit, at);
        if let Some(i) = last {
            let cum = self.walk_mut(id)?.cum(place, unit, i);
            out.merge(&cum);
        }
        let walk = self.walk(id)?;
        for (day, entry) in walk.adj_regs.get(place).into_iter().flatten() {
            if *day <= at.day {
                out.entries.insert(entry.clone());
            }
        }
        Ok(out)
    }
}

/// A pad of gap `gap` at reading `r`, from `other` into the place when the gap is positive or
/// unknown, else out of it (refcore's `PadMv`). E §6.1.
fn make_pad(r: &Reading, other: &str, gap: &Amount, coll: Collected, at: At) -> Result<Pad, Stop> {
    let (amount, into) = match gap {
        Amount::Qty(g) if g.is_negative() => (
            Amount::Qty(g.checked_neg().map_err(crate::eval::num_stop)?),
            false,
        ),
        Amount::Qty(g) => (Amount::Qty(g.clone()), !g.is_zero()),
        Amount::Hole(_) | Amount::Refused(_) => (gap.clone(), true),
    };
    let (from, to) = if into {
        (End::Place(other.to_owned()), End::Place(r.place.clone()))
    } else {
        (End::Place(r.place.clone()), End::Place(other.to_owned()))
    };
    Ok(Pad {
        id: format!("pads:{}:{}@{}", r.version, r.place, r.instant),
        leg: LegVal {
            name: "cash".to_owned(),
            from,
            to,
            amount,
            unit: Some(r.unit.clone()),
            day: Dated::Day(at.day),
        },
        coll,
    })
}

/// A state's value as an amount: the refusal it holds, a hole with every question it rests
/// on (`HOLE-ANCHOR` before the place's first reading), or the balance (refcore's
/// `st_value`). L §5.10, §8.1.
pub(super) fn booked_amount(st: &St, anchor: &Hole) -> Amount {
    if let Some(r) = &st.bad {
        return Amount::Refused(r.clone());
    }
    let missing = st.val.is_none().then_some(anchor);
    if let Some(h) = spread(&[st.hole.as_ref(), st.perm.as_ref(), missing]) {
        return Amount::Hole(h);
    }
    match &st.val {
        Some(v) => Amount::Qty(v.clone()),
        None => Amount::Hole(anchor.clone()),
    }
}

/// The gap: the reading less the booked balance; the refusal the balance is; the holes of
/// either (refcore's `sub`). E §6.1.
fn gap_of(stated: &Result<Num, Hole>, booked: &Amount) -> Result<Amount, Stop> {
    let stated_hole = stated.as_ref().err();
    match booked {
        Amount::Refused(r) => Ok(Amount::Refused(r.clone())),
        Amount::Hole(h) => Ok(Amount::Hole(
            spread(&[stated_hole, Some(h)]).ok_or_else(|| invariant("a gap's hole"))?,
        )),
        Amount::Qty(b) => match stated {
            Err(h) => Ok(Amount::Hole(h.clone())),
            Ok(s) => Ok(Amount::Qty(
                s.checked_sub(b).map_err(crate::eval::num_stop)?,
            )),
        },
    }
}

/// A quantity of a unit as it prints. L §5.18.
fn qty_text(m: &Reality, q: &Num, unit: &str) -> Result<String, Stop> {
    qty_value(m, q, unit)?
        .text(m)
        .map_err(crate::eval::num_stop)
}

/// A gap as the register prints it: the amount, its hole, or `refused: <code> <message>`
/// (refcore's `gtext`). E §6.1, L §8.2.
pub(crate) fn gap_text(m: &Reality, gap: &Amount, unit: &str) -> Result<String, Stop> {
    match gap {
        Amount::Qty(q) => qty_text(m, q, unit),
        Amount::Hole(h) => Ok(h.text()),
        Amount::Refused(r) => Ok(format!("refused: {}", r.text())),
    }
}
