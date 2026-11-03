//! Load's indexes: the built-in period indexes over the run, written lists and ranges, and open indexes' keys.
//! Spec: L §4.1 (indexes: lists, integer and period ranges with `by` and `from`, open indexes,
//! the built-in `days`, `months`, `years`, `bank_account`), §5.6 (keys and their order).
//! Twin: engine/tools/refcore/loader.py (`build_indexes`, `build_range`, `prescan_open_keys`).
//! Invariants: a range's keys are in its order and each period is `by` long; a built-in period
//! index holds the periods that meet the run [S, E]; an open index's keys are in text order.

use super::keys::{KeyLit, int_of, key_source_text, lit_text};
use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{Domain, IndexForm as Form, IndexInfo, KeyVal, NameKind};
use crate::syntax::{IndexForm, IndexFrom, ItemForm, Lit, Row, Section, Stop};
use crate::time::{Day, Period, PeriodKind, Step};

/// A range's written ends, step and anchor.
struct Range {
    /// The index's id.
    id: String,
    /// The period word.
    period: Option<String>,
    /// The first key.
    lo: Lit,
    /// The last key.
    hi: Lit,
    /// `by n`.
    by: i64,
    /// `from …`.
    from: Option<IndexFrom>,
    /// The file it is in.
    file: String,
    /// Its line.
    line: u32,
}

impl Loader {
    /// The built-in indexes, then each `index` section (refcore's `build_indexes`). L §4.1.
    pub(super) fn build_indexes(&mut self) -> Result<(), Stop> {
        let not_built = Stop::NotBuilt("a run past 9999-12-31");
        let mut days = IndexInfo::new("days", Form::Builtin);
        days.period = Some(PeriodKind::Day);
        let mut day = Some(self.m.start);
        while let Some(d) = day.filter(|x| *x <= self.m.end) {
            days.keys.push(KeyVal::Day(d));
            day = d.next();
        }
        let mut months = IndexInfo::new("months", Form::Builtin);
        months.period = Some(PeriodKind::Month);
        let mut years = IndexInfo::new("years", Form::Builtin);
        years.period = Some(PeriodKind::Year);
        for (index, first) in [
            (&mut months, Period::month_of(self.m.start)),
            (&mut years, Period::year_of(self.m.start)),
        ] {
            let mut period = first.ok_or(not_built.clone())?;
            while period.start() <= self.m.end {
                index.keys.push(KeyVal::Period(period));
                period = period.shift(1).ok_or(not_built.clone())?;
            }
        }
        for index in [
            days,
            months,
            years,
            IndexInfo::new("bank_account", Form::Open),
        ] {
            let id = index.id.clone();
            self.m.indexes.push(&id, index);
        }
        for (file, section) in &self.sections {
            let Section::Index(decl) = section else {
                continue;
            };
            let mut index = match &decl.form {
                IndexForm::Open => IndexInfo::new(&decl.id, Form::Open),
                IndexForm::List(keys) => list_index(&decl.id, keys, file, decl.line)?,
                IndexForm::Range {
                    period,
                    lo,
                    hi,
                    by,
                    from,
                } => {
                    let step = match by {
                        None => 1,
                        Some(n) => n
                            .to_i64()
                            .filter(|s| *s > 0)
                            .ok_or(Stop::NotBuilt("a range's by that isn't a positive integer"))?,
                    };
                    build_range(&Range {
                        id: decl.id.clone(),
                        period: period.clone(),
                        lo: lo.clone(),
                        hi: hi.clone(),
                        by: step,
                        from: from.clone(),
                        file: file.clone(),
                        line: decl.line,
                    })?
                }
            };
            index.assumed.clone_from(&decl.assumed);
            index.v.clone_from(&decl.v);
            index.line = Some(decl.line);
            index.finish();
            self.m.indexes.push(&decl.id, index);
        }
        Ok(())
    }

    /// The keys of open indexes the rows of inputs keyed by them bring, before places keyed by
    /// such an index are built (refcore's `prescan_open_keys`). L §4.1.
    pub(super) fn prescan_open_keys(&mut self) {
        if !self
            .m
            .indexes
            .entries()
            .any(|(_, ix)| ix.form == Form::Open)
        {
            return;
        }
        let sections = core::mem::take(&mut self.sections);
        for (file, section) in &sections {
            let Section::Version(version) = section else {
                continue;
            };
            for item in &version.items {
                if let ItemForm::Row(row) = &item.form {
                    self.prescan_row(&version.id, row, file, item.line);
                }
            }
        }
        self.sections = sections;
    }

    /// The open keys one row of an input brings; a key refused here is refused again at its
    /// row (refcore's `pass`).
    fn prescan_row(&mut self, vid: &str, row: &Row, file: &str, line: u32) {
        if row.given || row.qual.is_some() {
            return;
        }
        let Some(keys) = &row.keys else {
            return;
        };
        if self.m.names.get(&row.target) != Some(&NameKind::Input) {
            return;
        }
        let Some(input) = self.m.inputs.get(&row.target).filter(|i| !i.observed) else {
            return;
        };
        let binds = input.binds.clone();
        for (key, bind) in keys.iter().zip(&binds) {
            let Domain::Index(id) = &bind.domain else {
                continue;
            };
            let open = self
                .m
                .indexes
                .get(id)
                .is_some_and(|ix| ix.form == Form::Open);
            let Some(written) = KeyLit::of_row_key(key).filter(|_| open) else {
                continue;
            };
            if self
                .pattern_key(&written, bind, Some(vid), file, line)
                .is_err()
            {
                break;
            }
        }
    }
}

/// A list index's keys, each named once. L §4.1.
fn list_index(id: &str, keys: &[Lit], file: &str, line: u32) -> Result<IndexInfo, Stop> {
    let mut list = IndexInfo::new(id, Form::List);
    for key in keys {
        let text = key_source_text(key, file, line)?;
        if list.by_text(&text).is_some() {
            return Err(refuse(
                Load::Dup,
                format!("a key entered twice: {text}"),
                file,
                line,
            ));
        }
        let value = KeyVal::Key {
            index: id.to_owned(),
            name: text.clone(),
        };
        list.texts.push((text, value.clone()));
        list.keys.push(value);
    }
    Ok(list)
}

/// A range's keys: integers, or periods of a word, `by` long, anchored by `from` (refcore's
/// `build_range`). L §4.1.
fn build_range(range: &Range) -> Result<IndexInfo, Stop> {
    let (lo, hi) = (&range.lo, &range.hi);
    let word = if let Some(word) = range.period.as_deref() {
        word
    } else {
        if int_of(lo).is_none() || int_of(hi).is_none() {
            return Err(range.cannot("a range without a period word is of integers"));
        }
        if is_year(lo) && is_year(hi) {
            return Err(refuse(
                Load::Ambig,
                format!(
                    "{}..{} reads as a year and as an integer here",
                    lit_text(lo),
                    lit_text(hi)
                ),
                &range.file,
                range.line,
            ));
        }
        "int"
    };
    let mut index = IndexInfo::new(&range.id, Form::Range);
    match word {
        "int" => {
            let (Some(a), Some(b)) = (int_of(lo), int_of(hi)) else {
                return Err(range.cannot("an integer range"));
            };
            if range.from.is_some() {
                return Err(range.cannot("from on an integer range"));
            }
            if a > b {
                return Err(out_of_order(
                    range,
                    &format!("range {a}..{b} is out of order"),
                ));
            }
            index.is_int = true;
            let mut at = a;
            while at <= b {
                index.keys.push(KeyVal::Int(at));
                at = at
                    .checked_add(range.by)
                    .ok_or(Stop::NotBuilt("a range past i64"))?;
            }
        }
        "year" => {
            index.period = Some(PeriodKind::Year);
            index.keys = year_keys(range)?;
        }
        "month" => {
            index.period = Some(PeriodKind::Month);
            index.keys = month_keys(range)?;
        }
        "day" => {
            index.period = Some(PeriodKind::Day);
            index.keys = day_keys(range)?;
        }
        _ => return Err(Stop::NotBuilt("a range of that period word")),
    }
    Ok(index)
}

/// `LOAD-KEY` for a range whose ends are out of order.
fn out_of_order(range: &Range, message: &str) -> Stop {
    refuse(Load::Key, message.to_owned(), &range.file, range.line)
}

impl Range {
    /// `LOAD-PARSE cannot parse: <what>` at the range's line.
    fn cannot(&self, what: &str) -> Stop {
        refuse(
            Load::Parse,
            format!("cannot parse: {what}"),
            &self.file,
            self.line,
        )
    }
}

/// A four-digit whole number with no unit.
fn is_year(lit: &Lit) -> bool {
    let Lit::Num(n) = lit else {
        return false;
    };
    n.whole && n.unit.is_none() && n.four
}

/// The years of a year range, each `by` years long, from a month-day.
fn year_keys(range: &Range) -> Result<Vec<KeyVal>, Stop> {
    let years =
        (is_year(&range.lo) && is_year(&range.hi)).then(|| (int_of(&range.lo), int_of(&range.hi)));
    let Some((Some(a), Some(b))) = years else {
        return Err(range.cannot("a year range"));
    };
    if a > b {
        return Err(out_of_order(
            range,
            &format!("range {a}..{b} is out of order"),
        ));
    }
    let (month, day) = match &range.from {
        None => (1, 1),
        Some(IndexFrom::MonthDay(month, day)) => {
            if Day::from_ymd(2001, *month, *day).is_none() {
                return Err(range.cannot(&format!(
                    "{month:02}-{day:02} is not a month-day every year has"
                )));
            }
            (*month, *day)
        }
        Some(IndexFrom::Day(_)) => {
            return Err(range.cannot("a year range is anchored by a month-day"));
        }
    };
    let months = range
        .by
        .checked_mul(12)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or(Stop::NotBuilt("a range past u32"))?;
    let mut keys = Vec::new();
    let mut year = a;
    while year <= b {
        let next = year
            .checked_add(range.by)
            .ok_or(Stop::NotBuilt("a range past i64"))?;
        let start = ymd(year, month, day)?;
        let end = ymd(next, month, day)?;
        let period = Period::new(PeriodKind::Year, start, end, Step::Months(months))
            .ok_or(Stop::NotBuilt("a period out of order"))?;
        keys.push(KeyVal::Period(period));
        year = next;
    }
    Ok(keys)
}

/// The months of a month range, each `by` months long, from a day of the month.
fn month_keys(range: &Range) -> Result<Vec<KeyVal>, Stop> {
    let (Lit::Month(y0, m0), Lit::Month(y1, m1)) = (&range.lo, &range.hi) else {
        return Err(range.cannot("a month range"));
    };
    let day = match &range.from {
        None => 1,
        Some(IndexFrom::Day(n)) => n
            .to_i64()
            .filter(|d| (1..=28).contains(d))
            .and_then(|d| u8::try_from(d).ok())
            .ok_or_else(|| range.cannot("a month range is anchored by a day 1-28"))?,
        Some(IndexFrom::MonthDay(..)) => {
            return Err(range.cannot("a month range is anchored by a day 1-28"));
        }
    };
    if (y0, m0) > (y1, m1) {
        return Err(out_of_order(range, "range out of order"));
    }
    let months = u32::try_from(range.by).map_err(|_| Stop::NotBuilt("a range past u32"))?;
    let last = Day::from_ymd(*y1, *m1, day).ok_or(Stop::NotBuilt("a month past 9999"))?;
    let mut keys = Vec::new();
    let mut start = Day::from_ymd(*y0, *m0, day).ok_or(Stop::NotBuilt("a month past 9999"))?;
    while start <= last {
        let end = start
            .add_months(range.by)
            .ok_or(Stop::NotBuilt("a month past 9999"))?;
        let period = Period::new(PeriodKind::Month, start, end, Step::Months(months))
            .ok_or(Stop::NotBuilt("a period out of order"))?;
        keys.push(KeyVal::Period(period));
        start = end;
    }
    Ok(keys)
}

/// The days of a day range: days, or periods `by` days long.
fn day_keys(range: &Range) -> Result<Vec<KeyVal>, Stop> {
    let (Lit::Day(first), Lit::Day(last)) = (&range.lo, &range.hi) else {
        return Err(range.cannot("a day range"));
    };
    if range.from.is_some() {
        return Err(range.cannot("from on a day range"));
    }
    if first > last {
        return Err(out_of_order(range, "range out of order"));
    }
    let days = u32::try_from(range.by).map_err(|_| Stop::NotBuilt("a range past u32"))?;
    let mut keys = Vec::new();
    let mut day = *first;
    while day <= *last {
        let next = day
            .add_days(range.by)
            .ok_or(Stop::NotBuilt("a day past 9999-12-31"))?;
        if range.by == 1 {
            keys.push(KeyVal::Day(day));
        } else {
            let period = Period::new(PeriodKind::Day, day, next, Step::Days(days))
                .ok_or(Stop::NotBuilt("a period out of order"))?;
            keys.push(KeyVal::Period(period));
        }
        day = next;
    }
    Ok(keys)
}

/// A day from an integer year, a month and a day.
fn ymd(year: i64, month: u8, day: u8) -> Result<Day, Stop> {
    u16::try_from(year)
        .ok()
        .and_then(|y| Day::from_ymd(y, month, day))
        .ok_or(Stop::NotBuilt("a year past 9999"))
}
