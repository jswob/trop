//! Load's checker, its prices: each priced place's price input, an asset that names none, a price named twice, and `value(x, d) in CUR [by p]`.
//! Spec: L §5.10 (Value; A priced place: one home for a price input), §2.3 (`price`), §4.6
//! (`last`), §8.3 (`LOAD-PRICE`, `-UNIT`, `-NAME`, `-KEY`).
//! Twin: engine/tools/refcore/check3.py (`m_check_priced`, `m_resolve_price`, `m_t_value`,
//! `_all_priced`, `m_price_edges`); model.py (`price_of`, its `LOAD-PRICE`); loader.py
//! (`Loader.build`, its loop over `price_of` before the checker).
//! Invariants: the price clauses are resolved before any item is checked, the kinds' by id and
//! then the places' by thing name, a keyed place's clause once; each refusal is put at the
//! line its clause or its value is written on; a value reads its places' prices at its day.

use super::check::{Checker, Files, Scope, Stands, invariant};
use super::targets::{literal_place, named_parts, place_section, place_target};
use super::texts::text;
use super::types::{Typed, is_date, name_word, refused, uerr};
use crate::base::{Code, Load, Place as At, Refusal};
use crate::model::{
    Domain, Entry, Mark, NameKind, Node, NodeKind, PlaceInfo, Priced, Reality, Rel, Shown, Thing,
    Ty, Unit,
};
use crate::syntax::{Expr, Postfix, Price, Stop, Valuation};

/// What a value with no `by` finds its place priced by (refcore's `priced`).
enum Found {
    /// No price input.
    None,
    /// A price input.
    Price(Price),
    /// Every place the value can give is priced in one of its homes, or holds the currency.
    All,
}

/// `LOAD-PRICE` when a place names a price input on its kind and on itself, at the entry
/// that names it, every thing in the order load made them (refcore's `price_of`, run over
/// every thing before the checker). L §5.10.
///
/// # Errors
///
/// That refusal.
pub(super) fn price_twice(m: &Reality, files: &Files) -> Result<(), Stop> {
    for (_, thing) in m.things.entries() {
        let Some(base) = thing.base.as_deref() else {
            continue;
        };
        let own = thing
            .entries
            .iter()
            .rev()
            .find(|e| e.kind == base && e.priced.is_some());
        let Some(own) = own else {
            continue;
        };
        let Some(kind) = m.chain(base).into_iter().find(|k| k.priced.is_some()) else {
            continue;
        };
        let message = format!(
            "{} names a price input on its kind {} and on itself",
            thing.name, kind.id
        );
        return Err(entry_refusal(
            m,
            files,
            thing,
            Some(own),
            Load::Price,
            message,
        ));
    }
    Ok(())
}

/// A refusal at the line an entry of `thing` is written on, where load knows it.
fn entry_refusal(
    m: &Reality,
    files: &Files,
    thing: &Thing,
    entry: Option<&Entry>,
    code: Load,
    message: String,
) -> Stop {
    let place = match entry.map(|e| entry_place(m, files, thing, e)) {
        Some(Err(stop)) => return stop,
        Some(Ok(place)) => place,
        None => None,
    };
    let code = Code::Load(code);
    Stop::Refused(match place {
        Some(place) => Refusal::at(code, message, place),
        None => Refusal::new(code, message),
    })
}

/// The file and line an entry of `thing` is written on: its version's, or its place
/// section's; none for an entry with no line (the prelude's). A section with no file is
/// `ENGINE-INVARIANT`.
fn entry_place(
    m: &Reality,
    files: &Files,
    thing: &Thing,
    entry: &Entry,
) -> Result<Option<At>, Stop> {
    let Some(line) = entry.line else {
        return Ok(None);
    };
    if let Some(version) = &entry.version {
        let file = files.of_version(version)?;
        return Ok(Some(At::Line(file.to_owned(), line)));
    }
    let Some(section) = section_of(m, thing) else {
        return Ok(None);
    };
    let file = files.get("place", &section)?;
    Ok(Some(At::Line(file.to_owned(), line)))
}

/// The last entry of `thing` written on a line load knows (refcore's `m_check_priced`, its
/// asset's last entry). L §5.10.
fn last_placed(m: &Reality, files: &Files, thing: &Thing) -> Result<Option<Entry>, Stop> {
    for entry in thing.entries.iter().rev() {
        if entry_place(m, files, thing, entry)?.is_some() {
            return Ok(Some(entry.clone()));
        }
    }
    Ok(None)
}

/// The place section a thing comes from: its keyed place's, or the unkeyed place it is.
pub(super) fn section_of(m: &Reality, thing: &Thing) -> Option<String> {
    if let Some(keys) = &thing.place {
        return Some(keys.place.clone());
    }
    let unkeyed = m.names.kind(&thing.name) == Some(NameKind::Place);
    unkeyed.then(|| thing.name.clone())
}

/// Whether `t` is `num CUR/u`: a currency to the first power over one other unit.
fn per_unit_of(t: &Ty, cur: &str) -> bool {
    if matches!(t, Ty::Unseen) {
        return true;
    }
    let Ty::Num(unit, _) = t else {
        return false;
    };
    let terms = unit.terms();
    terms.len() == 2 && terms.iter().any(|(base, exp)| base == cur && *exp == 1)
}

/// An input or a def `id`: its kind, its declared type and how many keys it takes.
fn input_or_def_info(m: &Reality, id: &str) -> Option<(NameKind, Option<Ty>, usize)> {
    let kind = m.names.kind(id)?;
    if kind == NameKind::Input {
        let input = m.inputs.get(id)?;
        return Some((kind, input.ty.clone(), input.binds.len()));
    }
    if kind == NameKind::Def {
        let def = m.defs.get(id)?;
        return Some((kind, Some(def.ty.clone()), def.binds.len()));
    }
    None
}

/// An input's or def's node.
fn item_node(id: &str, kind: NameKind) -> Node {
    if kind == NameKind::Input {
        Node::new(NodeKind::Input, id)
    } else {
        Node::new(NodeKind::Def, id)
    }
}

impl Checker {
    /// Each price clause resolved, the kinds' by id then the places' by thing name, a keyed
    /// place's clause once; then `LOAD-PRICE` for a thing of `assets` that names no price
    /// input (refcore's `m_check_priced`). L §5.10.
    pub(super) fn check_priced(&mut self, m: &Reality, files: &Files) -> Result<(), Stop> {
        let mut kinds: Vec<(&String, Option<u32>, &Price)> = Vec::new();
        for (id, kind) in m.kinds.entries() {
            if let Some(price) = &kind.priced {
                kinds.push((id, kind.line, price));
            }
        }
        kinds.sort_by(|a, b| a.0.cmp(b.0));
        for (id, line, price) in kinds {
            let at = match line {
                Some(line) => Some((files.get("kind", id)?, line)),
                None => None,
            };
            self.resolve_price(m, price, &format!("kind {id}"), None, at)?;
        }
        let mut things: Vec<&Thing> = m.things.entries().map(|(_, thing)| thing).collect();
        things.sort_by(|a, b| a.name.cmp(&b.name));
        let mut sections = std::collections::BTreeSet::new();
        for thing in &things {
            for entry in &thing.entries {
                let Some(price) = &entry.priced else {
                    continue;
                };
                let section = if entry.version.is_none() {
                    section_of(m, thing).and_then(|id| m.places.get(&id))
                } else {
                    None
                };
                if section.is_some_and(|place| !sections.insert(place.id.clone())) {
                    continue;
                }
                let at = entry_place(m, files, thing, entry)?.and_then(|place| match place {
                    At::Line(file, line) => Some((file, line)),
                    At::File(_) => None,
                });
                let at = at.as_ref().map(|(file, line)| (file.as_str(), *line));
                let what = format!("place {}", thing.name);
                self.resolve_price(m, price, &what, section, at)?;
            }
        }
        let assets = m.subtree("assets");
        for thing in &things {
            let asset = thing
                .base
                .as_ref()
                .is_some_and(|base| assets.contains(base));
            if asset && m.price_of(thing).is_none() {
                let last = last_placed(m, files, thing)?;
                let message = format!("asset {} names no price input", thing.name);
                let last = last.as_ref();
                return Err(entry_refusal(m, files, thing, last, Load::Price, message));
            }
        }
        price_fits(m, files, &things)
    }

    /// A price clause: an input or a def of type `num CUR` or `num CUR/u`, keyed by one index
    /// at most, read with the keys it states from the clause's own node (refcore's
    /// `m_resolve_price`). L §5.10, §2.3, §4.6.
    fn resolve_price(
        &mut self,
        m: &Reality,
        price: &Price,
        what: &str,
        section: Option<&PlaceInfo>,
        at: Option<(&str, u32)>,
    ) -> Result<(), Stop> {
        let (file, line) = at.unwrap_or(("", 0));
        let at_clause = |stop: Stop| match (stop, at) {
            (Stop::Refused(refusal), Some(_)) => {
                Stop::Refused(refusal.placed(At::Line(file.to_owned(), line)))
            }
            (other, _) => other,
        };
        let Some((id, posts)) = named_parts(&price.input) else {
            return Err(invariant(
                "unreachable: the parser reads a price as L §2.3's `ref`, so a price that isn't \
                 a reference is LOAD-PARSE before load (T's `written/price-paren`)"
                    .to_owned(),
            ));
        };
        let keyed = matches!(posts, [Postfix::Keys(_)]);
        if !keyed && !posts.is_empty() {
            return Err(Stop::NotBuilt("a priced reference with fields"));
        }
        if keyed {
            let scope = Scope::new(Node::new(NodeKind::Price, what), Stands::Rule, file, line);
            let scope = match section {
                Some(info) => self.keyed(m, scope, &info.binds)?,
                None => scope,
            };
            self.expr(m, &scope, &price.input, None, 0)?;
            let read = m.names.kind(id).filter(|_| !scope.is_var(id));
            if !matches!(read, Some(NameKind::Input | NameKind::Def)) {
                let message = format!(
                    "{what} is priced by {}, not an input or a def",
                    text(&price.input)
                );
                return Err(at_clause(uerr(message)));
            }
            if price.last && read != Some(NameKind::Input) {
                let message = format!(
                    "{what} is priced by last({}): last reads an input (L §4.6)",
                    text(&price.input)
                );
                return Err(at_clause(uerr(message)));
            }
        }
        let Some(kind) = m.names.kind(id) else {
            return Err(at_clause(
                refused(Load::Name, format!("unknown id {id}")).naming(id),
            ));
        };
        let Some((_, ty, keys)) = input_or_def_info(m, id) else {
            let message = format!(
                "{what} is priced by {id}, a {}, not an input or a def",
                name_word(kind)
            );
            return Err(at_clause(uerr(message)));
        };
        if price.last && kind != NameKind::Input {
            let message = format!("{what} is priced by last({id}): last reads an input (L §4.6)");
            return Err(at_clause(uerr(message)));
        }
        if m.price_mode(ty.as_ref()).is_none() {
            let shown = ty.as_ref().map_or_else(|| "?".to_owned(), Ty::show);
            let message = format!(
                "{what} is priced by {id} of type {shown}, not num CUR (a value) or num CUR/u (a price per unit)"
            );
            return Err(at_clause(uerr(message)));
        }
        if keys > 1 {
            let message =
                format!("wrong number of keys: {what} is priced by {id}, keyed by {keys} indexes");
            return Err(at_clause(refused(Load::Key, message)));
        }
        Ok(())
    }

    /// `value(x, d) in CUR [by p] [of q]`: a place, a date, a declared currency; the prices
    /// its places are priced by, read at its day; a `by` where a position may be in another
    /// unit; a balance read (refcore's `m_t_value`, `m_price_edges`). L §5.10.
    pub(super) fn value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        value: &Valuation,
        depth: u32,
    ) -> Typed {
        let t = self.place_arg(m, scope, &value.place, "value", depth)?;
        let dt = self.expr(m, scope, &value.day, None, depth)?;
        if !is_date(m, &dt) {
            return Err(uerr(format!(
                "the day of value is {}, not a date",
                dt.show()
            )));
        }
        let cur = &value.currency;
        if !m.currencies.contains_key(cur) {
            return Err(uerr(format!("money in an undeclared currency {cur}")));
        }
        let literal = literal_place(m, scope, &value.place)?;
        let cands = candidates(m, literal.as_deref(), &t);
        for thing in cands.iter().filter_map(|name| m.things.get(name)) {
            if let Some(node) = m.price_of(thing).and_then(|price| price_node(m, &price)) {
                self.edge(&scope.item, node, Rel::Explicit);
            }
        }
        let per = match &value.by {
            None => {
                no_by(m, scope, value, literal.as_deref(), &t, &cands)?;
                None
            }
            Some(by) => self.by(m, scope, by, value, depth)?,
        };
        for name in seen_places(m, scope, &value.place, literal.as_deref(), &cands) {
            if let Some(thing) = m.things.get(&name) {
                fits_value(m, value, thing, per.as_deref())?;
            }
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        let target = place_target(m, scope, &value.place)?;
        self.mark(&scope.item, Mark::Bal, target);
        if let Some(of) = &value.of {
            self.check_of(m, scope, of, Some((&value.place, &t)), depth)?;
        }
        let unit = Unit::make(vec![(cur.clone(), 1)]).map_err(|_| {
            Stop::Refused(Refusal::new(
                Code::EngineInvariant,
                "a currency's unit".to_owned(),
            ))
        })?;
        Ok(Ty::Num(unit, Shown::Plain))
    }

    /// A value's `by p`: an unkeyed input read through `last`, a keyed input or def, an input
    /// read through `last` at keys, or an expression of type `num CUR/u`; the unit u its price
    /// is per where the `by` itself gives it, none for a price keyed by unit (refcore's
    /// `m_t_value`, its `by` branches and `per`). L §5.10, §4.6.
    fn by(
        &mut self,
        m: &Reality,
        scope: &Scope,
        by: &Price,
        value: &Valuation,
        depth: u32,
    ) -> Result<Option<String>, Stop> {
        let cur = &value.currency;
        let name = if let Expr::Name(id) = &by.input {
            Some(id.as_str())
        } else {
            None
        };
        // refcore's `hit`: the id the name looks up, and the input or def it is.
        let looked = name.filter(|id| !scope.is_var(id) && m.names.contains_key(id));
        // An unkeyed input or def with no `last` is read as an expression, below.
        let mut read = None;
        if let Some(id) = looked
            && let Some(info) = input_or_def_info(m, id)
            && (info.2 != 0 || by.last)
        {
            read = Some((id, info));
        }
        if let Some((id, (NameKind::Input, pt, 0))) = &read
            && by.last
        {
            if !pt.as_ref().is_some_and(|t| per_unit_of(t, cur)) {
                let shown = pt.as_ref().map_or_else(|| "?".to_owned(), Ty::show);
                return Err(uerr(format!(
                    "value by last({id}): the price is {shown}, not num {cur}/u"
                )));
            }
            self.edge(&scope.item, item_node(id, NameKind::Input), Rel::Explicit);
            return Ok(pt.as_ref().and_then(per_of));
        }
        if let Some((id, (kind, ty, keys))) = &read {
            if *keys != 1 || !matches!(ty, Some(Ty::Num(..))) {
                return Err(uerr(format!(
                    "value by {id}: a price is a num CUR/u keyed by unit, or an expression of that type with its keys"
                )));
            }
            if by.last && *kind != NameKind::Input {
                return Err(uerr(format!(
                    "value by last({id}): last reads an input (L §4.6)"
                )));
            }
            // a price keyed by unit is a num CUR/u in the value's own currency, as the
            // written form is (L §5.10; pick P61, #130)
            if !ty.as_ref().is_some_and(|t| per_unit_of(t, cur)) {
                let shown = ty.as_ref().map_or_else(|| "?".to_owned(), Ty::show);
                let named = if by.last {
                    format!("last({id})")
                } else {
                    (*id).to_owned()
                };
                return Err(uerr(format!(
                    "value by {named}: the price is {shown}, not num {cur}/u"
                )));
            }
            // keyed by unit: a price keyed by places is load's to refuse, which sees its
            // key's kind (L §5.10, R5)
            if !m.unit_keyed(id) {
                return Err(uerr(format!(
                    "value by {id}: a price is a num CUR/u keyed by unit, or an expression of that type with its keys"
                )));
            }
            self.edge(&scope.item, item_node(id, *kind), Rel::Explicit);
            return Ok(None);
        }
        if let Some(id) = name
            && looked.is_none()
            && by.last
        {
            return Err(refused(Load::Name, format!("unknown price input {id}")));
        }
        if by.last {
            if keyed_input(m, scope, &by.input) {
                let pt = self.last(m, scope, &by.input, &value.day, depth)?;
                if !per_unit_of(&pt, cur) {
                    return Err(uerr(format!(
                        "value by last({}): the price is {}, not num {cur}/u",
                        text(&by.input),
                        pt.show()
                    )));
                }
                return Ok(per_of(&pt));
            }
            return Err(uerr(format!(
                "value by last({}): last reads an input (L §4.6)",
                text(&by.input)
            )));
        }
        let pt = self.expr(m, scope, &by.input, None, depth)?;
        if !per_unit_of(&pt, cur) {
            return Err(uerr(format!(
                "value by {}: the price is {}, not num {cur}/u",
                text(&by.input),
                pt.show()
            )));
        }
        Ok(per_of(&pt))
    }
}

/// The unit a `num CUR/u` is per: its term of power −1.
fn per_of(t: &Ty) -> Option<String> {
    let Ty::Num(unit, _) = t else {
        return None;
    };
    unit.terms()
        .iter()
        .find(|(_, power)| *power == -1)
        .map(|(base, _)| base.clone())
}

/// The places load sees a value's reference give, by name: a literal one, a keyed place's
/// own things, or every thing of the reference's kind (refcore's `_seen_places`). L §5.10.
fn seen_places(
    m: &Reality,
    scope: &Scope,
    place: &Expr,
    literal: Option<&str>,
    cands: &[String],
) -> Vec<String> {
    let section = literal
        .is_none()
        .then(|| place_section(m, scope, place))
        .flatten()
        .and_then(|id| m.places.get(&id));
    let mut out: Vec<String> = match section {
        Some(info) => info.things.iter().map(|(_, name)| name.clone()).collect(),
        None => cands.to_vec(),
    };
    out.sort();
    out
}

/// A value's place against the price it is read by (refcore's `m_t_value`, its places): the
/// place's own price per unit in another currency than the value's (`LOAD-UNIT`); with a
/// `by`, a position the `by`'s unit isn't (`LOAD-UNIT`), or one the place's own price
/// already gives (`LOAD-PRICE`). Load sees both prices. L §5.10, §8.3.
fn fits_value(
    m: &Reality,
    value: &Valuation,
    thing: &Thing,
    by_per: Option<&str>,
) -> Result<(), Stop> {
    let (cur, name) = (&value.currency, &thing.name);
    let priced = m
        .price_of(thing)
        .and_then(|price| m.clause_mode(&price))
        .map(|(_, mode)| mode);
    let Some(by) = &value.by else {
        if let Some(Priced::PerUnit(pc, per)) = &priced
            && pc != cur
        {
            let u = unit_name(m, thing, per);
            return Err(uerr(format!(
                "value({name}) in {cur}: {name} is priced in {pc}/{u}, not {cur}/{u}"
            )));
        }
        return Ok(());
    };
    // the unit a `by` prices is the position's: a place valued whole in another currency
    // holds that currency, a place of one unit that unit
    let want = match &priced {
        Some(Priced::Whole(whole)) => Some(whole.clone()),
        Some(Priced::PerUnit(..)) => None,
        None => m.place_unit(thing),
    };
    if let (Some(per), Some(want)) = (by_per, want.as_deref())
        && !matches!(want, "any" | "one")
        && want != cur
        && per != want
    {
        return Err(uerr(format!(
            "value({name}) in {cur}: {name} is priced in {cur}/{per}, not {cur}/{want}"
        )));
    }
    if let Some(Priced::PerUnit(_, per)) = &priced
        && per != cur
    {
        let named = match by_per {
            Some(bp) if per.starts_with('$') => bp,
            Some(_) | None => per.as_str(),
        };
        let u = unit_name(m, thing, named);
        let written = if by.last {
            format!("last({})", text(&by.input))
        } else {
            text(&by.input)
        };
        return Err(refused(
            Load::Price,
            format!("value({name}) by {written}: {name}'s price already gives its {u}"),
        ));
    }
    Ok(())
}

/// A per-unit price's unit as a message names it: a price keyed by unit (`$u`) names the
/// place's own unit (refcore's `_unit_name`). L §2.4, §3.2.
fn unit_name(m: &Reality, thing: &Thing, u: &str) -> String {
    if u.starts_with('$')
        && let Some(own) = m
            .place_unit(thing)
            .filter(|pu| !matches!(pu.as_str(), "any" | "one"))
    {
        return own;
    }
    u.to_owned()
}

/// Where a thing's price clause is written: its own entry's line, else the line of the kind
/// that states it (refcore's `pn.loc`).
fn price_place(m: &Reality, files: &Files, thing: &Thing) -> Result<Option<(String, u32)>, Stop> {
    let Some(base) = thing.base.as_deref() else {
        return Ok(None);
    };
    let own = thing
        .entries
        .iter()
        .rev()
        .find(|e| e.kind == base && e.priced.is_some());
    if let Some(entry) = own {
        return Ok(match entry_place(m, files, thing, entry)? {
            Some(At::Line(file, line)) => Some((file, line)),
            Some(At::File(_)) | None => None,
        });
    }
    let Some(kind) = m.chain(base).into_iter().find(|k| k.priced.is_some()) else {
        return Ok(None);
    };
    let Some(line) = kind.line else {
        return Ok(None);
    };
    Ok(Some((files.get("kind", &kind.id)?.to_owned(), line)))
}

/// Each place's price input against the place, its key and its unit, with the priced kinds,
/// whatever reads them; the first by file and line prints (refcore's `m_check_priced`, its
/// pass 4). E §3.1 pass 4, L §5.10, §8.3 `LOAD-KEY`, `LOAD-UNIT`.
fn price_fits(m: &Reality, files: &Files, things: &[&Thing]) -> Result<(), Stop> {
    let mut first: Option<((String, u32), Stop)> = None;
    for thing in things {
        let Some((pid, mode)) = m.price_of(thing).and_then(|p| m.clause_mode(&p)) else {
            continue;
        };
        let Some(pu) = m
            .place_unit(thing)
            .filter(|u| !matches!(u.as_str(), "any" | "one"))
        else {
            continue;
        };
        let name = &thing.name;
        let (code, message) = match (m.first_domain(&pid), &mode) {
            (Some(Domain::Kind(k)), _) if !m.thing_in(name, &k) && !m.thing_in(&pu, &k) => (
                Load::Key,
                format!(
                    "{name} is priced by {pid}, keyed by {k}, which holds neither {name} nor its unit"
                ),
            ),
            (_, Priced::PerUnit(pc, per)) if !per.starts_with('$') && *per != pu => (
                Load::Unit,
                format!("value({name}) in {pc}: {name} is priced in {pc}/{per}, not {pc}/{pu}"),
            ),
            _ => continue,
        };
        let at = price_place(m, files, thing)?;
        let key = at.clone().unwrap_or_else(|| (String::new(), 0));
        if first.as_ref().is_none_or(|(k, _)| key < *k) {
            let refusal = Refusal::new(Code::Load(code), message);
            let refusal = match at {
                Some((file, line)) => refusal.placed(At::Line(file, line)),
                None => refusal,
            };
            first = Some((key, Stop::Refused(refusal)));
        }
    }
    first.map_or(Ok(()), |(_, stop)| Err(stop))
}

/// Whether `e` is an input that is no bound variable, read with keys: `px[k]`.
fn keyed_input(m: &Reality, scope: &Scope, e: &Expr) -> bool {
    let Expr::Post(base, posts) = e else {
        return false;
    };
    let Expr::Name(id) = base.as_ref() else {
        return false;
    };
    matches!(posts.as_slice(), [Postfix::Keys(_)])
        && !scope.is_var(id)
        && m.names.kind(id) == Some(NameKind::Input)
}

/// The names of the things a value's place can give: the place it names, or every thing of
/// the kind its type names (refcore's `m_t_value`, its `cands`).
fn candidates(m: &Reality, literal: Option<&str>, t: &Ty) -> Vec<String> {
    if let Some(name) = literal {
        return m
            .things
            .get(name)
            .map(|_| name.to_owned())
            .into_iter()
            .collect();
    }
    let Ty::Key(kind) = t else {
        return Vec::new();
    };
    let sub = m.subtree(kind);
    let mut names = Vec::new();
    for (name, thing) in m.things.entries() {
        if let Some(base) = &thing.base
            && sub.contains(base)
        {
            names.push(name.clone());
        }
    }
    names
}

/// The input or def a price clause names.
fn price_node(m: &Reality, price: &Price) -> Option<Node> {
    let (id, _) = named_parts(&price.input)?;
    let (kind, _, _) = input_or_def_info(m, id)?;
    Some(item_node(id, kind))
}

/// A value with no `by`: `LOAD-PRICE` where its place holds another unit than `CUR` and is
/// priced by nothing, or is valued whole in another currency (refcore's `m_t_value`, its
/// `n.by is None` arm). L §5.10.
fn no_by(
    m: &Reality,
    scope: &Scope,
    value: &Valuation,
    literal: Option<&str>,
    t: &Ty,
    cands: &[String],
) -> Result<(), Stop> {
    let cur = &value.currency;
    let mut pu: Option<String> = None;
    let mut found = Found::None;
    if let Some(thing) = literal.and_then(|name| m.things.get(name)) {
        pu = m.place_unit(thing);
        if let Some(price) = m.price_of(thing) {
            found = Found::Price(price);
        }
    } else if let Ty::Key(kind) = t
        && m.kinds.get(kind).is_some()
    {
        pu = m.kind_unit(kind).map(str::to_owned);
        if let Some(price) = m.kind_priced(kind) {
            found = Found::Price(price.clone());
        }
        let holds_other = pu
            .as_deref()
            .is_some_and(|u| !matches!(u, "any" | "one") && u != cur);
        if holds_other
            && matches!(found, Found::None)
            && all_priced(m, scope, &value.place, cands, cur)
        {
            found = Found::All;
        }
    }
    let other = pu
        .as_deref()
        .filter(|u| !matches!(*u, "any" | "one") && u != cur);
    if let Some(held) = other
        && matches!(found, Found::None)
    {
        return Err(refused(
            Load::Price,
            format!(
                "value({}) in {cur} holds units in {held} and states no by",
                text(&value.place)
            ),
        ));
    }
    if let Found::Price(price) = &found
        && let Some((_, Priced::Whole(whole))) = m.clause_mode(price)
        && whole != *cur
    {
        return Err(refused(
            Load::Price,
            format!(
                "value({}) in {cur} holds units in {whole} and states no by",
                text(&value.place)
            ),
        ));
    }
    Ok(())
}

/// Whether every place a value's place can give is priced in one of its homes, or holds
/// `cur`: a keyed place's own things, else every thing of its kind; a place valued whole in
/// another currency is not (refcore's `_all_priced`). L §5.10.
fn all_priced(m: &Reality, scope: &Scope, place: &Expr, cands: &[String], cur: &str) -> bool {
    let mut things: Vec<&Thing> = cands.iter().filter_map(|name| m.things.get(name)).collect();
    if let Some(section) = place_section(m, scope, place)
        && let Some(info) = m.places.get(&section)
    {
        things = info
            .things
            .iter()
            .filter_map(|(_, name)| m.things.get(name))
            .collect();
    }
    if things.is_empty() {
        return false;
    }
    things.iter().all(|thing| match m.price_of(thing) {
        None => m.place_unit(thing).as_deref() == Some(cur),
        Some(price) => {
            !matches!(m.clause_mode(&price), Some((_, Priced::Whole(whole))) if whole != cur)
        }
    })
}
