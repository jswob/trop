//! Load's checker, its prices: each priced place's price input, an asset that names none, a price named twice, and `value(x, d) in CUR [by p]`.
//! Spec: L §5.10 (Value; A priced place: one home for a price input), §2.3 (`price`), §4.6
//! (`last`), §8.3 (`LOAD-PRICE`, `-UNIT`, `-NAME`, `-KEY`).
//! Twin: engine/tools/refcore/check3.py (`m_check_priced`, `m_resolve_price`, `m_t_value`,
//! `_all_priced`, `m_price_edges`); model.py (`price_of`, its `LOAD-PRICE`); loader.py
//! (`Loader.build`, its loop over `price_of` before the checker).
//! Invariants: the price clauses are resolved before any item is checked, the kinds' by id and
//! then the places' by thing name, a keyed place's clause once; each refusal is put at the
//! line its clause or its value is written on; a value reads its places' prices at its day.

use super::check::{
    Checker, Files, Scope, Stands, literal_place, named_parts, place_section, place_target,
};
use super::types::{Typed, is_date, name_word, refused, text, uerr};
use crate::base::{Code, Load, Place as At, Refusal};
use crate::model::{
    Entry, Mark, NameKind, Node, NodeKind, PlaceInfo, Reality, Rel, Shown, Thing, Ty, Unit,
};
use crate::syntax::{Expr, Postfix, Price, Stop, Valuation};

/// How a price input values a place (refcore's `pn._mode`, `pn._cur`). L §5.10.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Priced {
    /// `num CUR` or `money CUR`: the value of the place's one position.
    Whole(String),
    /// `num CUR/u`: a price per unit of its positions.
    PerUnit,
}

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
    let place = entry.and_then(|e| entry_place(m, files, thing, e));
    let code = Code::Load(code);
    Stop::Refused(match place {
        Some(place) => Refusal::at(code, message, place),
        None => Refusal::new(code, message),
    })
}

/// The file and line an entry of `thing` is written on: its version's, or its place
/// section's.
fn entry_place(m: &Reality, files: &Files, thing: &Thing, entry: &Entry) -> Option<At> {
    let line = entry.line?;
    let file = match &entry.version {
        Some(version) => files.of_version(version).ok()?,
        None => files.get("place", &section_of(m, thing)?).ok()?,
    };
    Some(At::Line(file.to_owned(), line))
}

/// The place section a thing comes from: its keyed place's, or the unkeyed place it is.
pub(super) fn section_of(m: &Reality, thing: &Thing) -> Option<String> {
    if let Some(keys) = &thing.place {
        return Some(keys.place.clone());
    }
    let unkeyed = m.names.get(&thing.name) == Some(&NameKind::Place);
    unkeyed.then(|| thing.name.clone())
}

/// How a price input of type `ty` values a place: `num CUR` (or `money CUR`) whole, `num
/// CUR/u` per unit, else not a price (refcore's `m_resolve_price`, its type test). L §5.10.
fn price_mode(m: &Reality, ty: Option<&Ty>) -> Option<Priced> {
    let ty = ty?;
    let terms: Vec<(String, i64)> = if let Ty::Money(cur) = ty {
        vec![(cur.clone(), 1)]
    } else if let Ty::Num(unit, _) = ty {
        unit.terms().to_vec()
    } else {
        return None;
    };
    let curs: Vec<&String> = terms
        .iter()
        .filter(|(base, exp)| *exp == 1 && m.currencies.contains_key(base))
        .map(|(base, _)| base)
        .collect();
    let per = terms.iter().any(|(_, exp)| *exp == -1);
    match (terms.as_slice(), curs.as_slice()) {
        ([_], [cur]) => Some(Priced::Whole((*cur).clone())),
        ([_, _], [_]) if per => Some(Priced::PerUnit),
        _ => None,
    }
}

/// How a price clause values its place, from its input's declared type.
fn clause_mode(m: &Reality, price: &Price) -> Option<Priced> {
    let (id, _) = named_parts(&price.input)?;
    let (_, ty, _) = input_or_def_info(m, id)?;
    price_mode(m, ty.as_ref())
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
    let kind = *m.names.get(id)?;
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
            let at = files.get("kind", id).ok().zip(line);
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
                let at = entry_place(m, files, thing, entry).and_then(|place| match place {
                    At::Line(file, line) => Some((file, line)),
                    At::File(_) => None,
                });
                let at = at.as_ref().map(|(file, line)| (file.as_str(), *line));
                let what = format!("place {}", thing.name);
                self.resolve_price(m, price, &what, section, at)?;
            }
        }
        let assets = m.subtree("assets");
        for thing in things {
            let asset = thing
                .base
                .as_ref()
                .is_some_and(|base| assets.contains(base));
            if asset && m.price_of(thing).is_none() {
                let last = thing
                    .entries
                    .iter()
                    .rev()
                    .find(|e| entry_place(m, files, thing, e).is_some());
                let message = format!("asset {} names no price input", thing.name);
                return Err(entry_refusal(m, files, thing, last, Load::Price, message));
            }
        }
        Ok(())
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
            return Err(Stop::NotBuilt("a priced reference with fields"));
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
            let read = m.names.get(id).filter(|_| !scope.is_var(id)).copied();
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
        let Some(kind) = m.names.get(id).copied() else {
            return Err(at_clause(refused(Load::Name, format!("unknown id {id}"))));
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
        if price_mode(m, ty.as_ref()).is_none() {
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
        let literal = literal_place(m, scope, &value.place);
        let cands = candidates(m, literal.as_deref(), &t);
        for thing in cands.iter().filter_map(|name| m.things.get(name)) {
            if let Some(node) = m.price_of(thing).and_then(|price| price_node(m, &price)) {
                self.edge(&scope.item, node, Rel::Explicit);
            }
        }
        match &value.by {
            None => no_by(m, scope, value, literal.as_deref(), &t, &cands)?,
            Some(by) => self.by(m, scope, by, value, depth)?,
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        let target = place_target(m, scope, &value.place);
        self.mark(&scope.item, Mark::Bal, target);
        if let Some(of) = &value.of {
            self.check_of(m, scope, of, Some(&value.place), depth)?;
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
    /// read through `last` at keys, or an expression of type `num CUR/u` (refcore's
    /// `m_t_value`, its `by` branches). L §5.10, §4.6.
    fn by(
        &mut self,
        m: &Reality,
        scope: &Scope,
        by: &Price,
        value: &Valuation,
        depth: u32,
    ) -> Result<(), Stop> {
        let cur = &value.currency;
        let name = if let Expr::Name(id) = &by.input {
            Some(id.as_str())
        } else {
            None
        };
        // refcore's `hit`: the id the name looks up, and the input or def it is.
        let looked = name.filter(|id| !scope.is_var(id) && m.names.contains_key(*id));
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
            return Ok(());
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
            self.edge(&scope.item, item_node(id, *kind), Rel::Explicit);
            return Ok(());
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
                return Ok(());
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
        Ok(())
    }
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
        && m.names.get(id) == Some(&NameKind::Input)
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
        && let Some(Priced::Whole(whole)) = clause_mode(m, price)
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
            !matches!(clause_mode(m, &price), Some(Priced::Whole(whole)) if whole != cur)
        }
    })
}
