//! Balance readings: the `opening` and `stated` rows the cut sees, and what a stated reading does to its place, a pad or an anchor, with its gap.
//! Spec: L §5.19 (balance readings and their gaps: `pad x from y`, `anchor x`; a reading whose
//! value is `none` states nothing; the read rule decides who sees it), §4.6 (a reading's
//! instant), §5.10 (`HOLE-ANCHOR`); E §6.1 (Readings: the gap is the reading less the booked
//! balance at its instant; a pad is a movement of `pads`, an anchor sets the balance; holes),
//! §4.2 step 0 and step 4 (where readings apply).
//! Twin: engine/tools/refcore/moves.py (`readings`, `instant_text`, `PadMv`) and walk.py
//! (`apply_reading`, `booked_coll`, `sub`, `st_value`, `gtext`).
//! Invariants: a reading applies only before the cut but an opening at the run's start; after
//! a reading the place's balance is the reading; a pad books only its other end, the reading's
//! end is the reset; every gap is kept for the ledger.

use std::collections::BTreeMap;

use crate::base::{Code, Load, Refusal};
use crate::eval::{Collected, Engine, Entry};
use crate::hole::{Hole, spread};
use crate::model::{GapRole, Reality, RowInstant, Shown, Ty, Unit};
use crate::num::Num;
use crate::read::row_known;
use crate::syntax::{Edge, Stop};
use crate::time::Day;
use crate::walk::{
    Amount, At, Change, Dated, End, Ends, Ent, LegVal, Reading, St, invariant, qty_value,
};

/// A reading the cut sees: its day, its edge, its place, and what it states (none for a row
/// whose value is `none`).
pub type Seen = (Day, Edge, String, Option<Reading>);

/// A pad: a movement of `pads` that takes up a statement's gap (refcore's `PadMv`). E §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pad {
    /// Its id, `pads:<version>:<place>@<instant>`.
    pub id: String,
    /// Its one leg, from the pad's other place or to it.
    pub leg: LegVal,
    /// What its amount rests on: the booked balance's register and the reading's.
    pub coll: Collected,
    /// Its point in the walk.
    pub point: At,
}

/// A reading's gap, kept for the ledger's balance line (refcore's `Walk.gaps`). E §6.1, §6.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    /// The place.
    pub place: String,
    /// The reading's point.
    pub point: At,
    /// The gap: the reading less the booked balance, a hole, or the refusal the balance is.
    pub amount: Amount,
    /// The reading.
    pub reading: Reading,
    /// Whether the version anchors the place (else it pads it).
    pub anchor: bool,
    /// What the gap rests on.
    pub coll: Collected,
}

/// A reading row kept as the latest of its key: when it became known, its day and edge, the
/// row (L §5.19).
type Kept = (Day, Day, Edge, crate::model::Row);

/// The readings cut `cut` sees: each `opening` and `stated` row known by the cut, a restated
/// one replaced by the latest, sorted by place, instant and input (refcore's `readings`).
/// L §5.19, §4.6.
///
/// # Errors
///
/// A row load left without its version; `NotBuilt` for a reading `after` a movement or at a
/// day with no edge, and for a place of unit `any`.
pub fn readings(m: &Reality, cut: Day) -> Result<Vec<Seen>, Stop> {
    let mut keep: BTreeMap<(String, String, String), Kept> = BTreeMap::new();
    for input in ["opening", "stated"] {
        let Some(info) = m.inputs.get(input) else {
            continue;
        };
        for row in &info.rows {
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
            if keep.get(&key).is_none_or(|(k, ..)| known > *k) {
                keep.insert(key, (known, day, edge, row.clone()));
            }
        }
    }
    let mut out = Vec::with_capacity(keep.len());
    for ((place, instant, input), (_, day, edge, row)) in keep {
        let reading = reading_of(m, &input, &place, &instant, &row)?;
        out.push((day, edge, place, reading));
    }
    Ok(out)
}

/// Each place's first reading day among the readings cut `cut` sees, for `HOLE-ANCHOR`
/// (refcore's `anchor_question`, its days). L §8.1.
///
/// # Errors
///
/// As [`readings`].
pub fn first_readings(m: &Reality, cut: Day) -> Result<BTreeMap<String, Day>, Stop> {
    let mut out: BTreeMap<String, Day> = BTreeMap::new();
    for (day, _, place, _) in readings(m, cut)? {
        let first = out.entry(place).or_insert(day);
        *first = (*first).min(day);
    }
    Ok(out)
}

/// A reading's instant as text, `2026-09-30 end` (refcore's `instant_text`). L §4.6.
fn instant_text(day: Day, edge: Edge) -> String {
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
    /// A `stated` reading at its point: its gap against the booked balance kept for the
    /// ledger, then, by its version's gap role, an anchor that sets the balance or a pad that
    /// books the gap from the other place (refcore's `apply_reading`). E §6.1, L §5.19.
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
        let mut bc = self.booked_coll(m, id, &r.place, &r.unit, at)?;
        bc.merge(&r.coll);
        let anchor = role == GapRole::Anchor;
        self.walk_mut(id)?.gaps.push(Gap {
            place: r.place.clone(),
            point: at,
            amount: gap.clone(),
            reading: r.clone(),
            anchor,
            coll: bc.clone(),
        });
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
                self.walk_mut(id)?.add(m, &r.place, &r.unit, ent)
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
                }
                let sub = at
                    .sub
                    .checked_add(1)
                    .ok_or_else(|| invariant("a reading's point"))?;
                let mut ent = Ent::new(at.with_sub(sub), Change::Reset { val, qs });
                ent.coll = Some(r.coll.clone());
                ent.reg = reg;
                ent.mv = Some(pad.id.clone());
                padded.pads.push(pad);
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
        point: at,
    })
}

/// A state's value as an amount: the refusal it holds, a hole with every question it rests
/// on (`HOLE-ANCHOR` before the place's first reading), or the balance (refcore's
/// `st_value`). L §5.10, §8.1.
fn booked_amount(st: &St, anchor: &Hole) -> Amount {
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
