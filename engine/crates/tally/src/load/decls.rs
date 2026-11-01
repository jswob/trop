//! Load's declarations: inputs, defs, bands, places and moves, with their key binds, types and units.
//! Spec: L §3.1 (types), §3.2 (units, `num %`), §3.4 (`prices Y by ix`), §5.6 (key binds), §5.8
//! (inputs), §5.9 (bands), §5.10 (places), §5.11 (defs, rules, `v`), §5.14 (moves);
//! E §3.1 step 4.
//! Twin: engine/tools/refcore/loader.py (`build_decls`, `decl_binds`, `resolve_domain`,
//! `decl_type`, `unit_of`, `time_pos`); check3.py (`resolve_move_kind`).
//! Invariants: a declaration's bound variables are distinct and never a top-level id; a type's
//! currency is declared; a unit's terms are units, currencies, period words, `%`, the keys of
//! kinds of unit, or the declaration's own bound variables (as `$var`).

use super::spans::lspan_to_span;
use super::steps::{Loader, num_stop, refuse};
use crate::base::Load;
use crate::model::{
    BandInfo, BindInfo, CodeKind, DefInfo, Domain, InputInfo, KindSpec, MoveInfo, NameKind,
    PERIODS, PeriodWord, PlaceInfo, RuleVer, Shown, Span, TimePos, Ty, Unit, is_currency_code,
};
use crate::num::Num;
use crate::syntax::{
    Band, Bind, Def, Domain as Written, Expr, Input, InputForm, Lit, Move, Place, Section, Stop,
    Type,
};
use crate::syntax::{Unit as WrittenUnit, UnitTerm};
use std::collections::BTreeSet;

/// What a declaration with key binds is: an `on` or `over` clause is a move's alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Declares {
    /// An input.
    Input,
    /// A def.
    Def,
    /// A band.
    Band,
    /// A place.
    Place,
    /// A move.
    Move,
}

impl Declares {
    /// The word a message names it by.
    fn word(self) -> &'static str {
        match self {
            Declares::Input => "input",
            Declares::Def => "def",
            Declares::Band => "band",
            Declares::Place => "place",
            Declares::Move => "move",
        }
    }
}

impl Loader {
    /// Each declaration in section order (refcore's `build_decls`). L §5.8–§5.14.
    pub(super) fn build_decls(&mut self) -> Result<(), Stop> {
        let sections = core::mem::take(&mut self.sections);
        let built = self.declare_all(&sections);
        self.sections = sections;
        built
    }

    /// The declarations of every section.
    fn declare_all(&mut self, sections: &[(String, Section)]) -> Result<(), Stop> {
        for (file, section) in sections {
            match section {
                Section::Input(decl) => self.declare_input(decl, file)?,
                Section::Def(decl) => self.declare_def(decl, file)?,
                Section::Band(decl) => self.declare_band(decl, file)?,
                Section::Place(decl) => self.declare_place(decl, file)?,
                Section::Move(decl) => self.declare_move(decl, file)?,
                Section::Source(_)
                | Section::Version(_)
                | Section::Unit(_)
                | Section::Kind(_)
                | Section::Index(_)
                | Section::Ask(_) => {}
            }
        }
        Ok(())
    }

    /// An input: its key binds, type, `prices` and question. L §5.8, §3.4.
    fn declare_input(&mut self, decl: &Input, file: &str) -> Result<(), Stop> {
        let line = decl.line;
        let mut input = InputInfo::new(&decl.id, Some(line));
        input.binds = self.decl_binds(&decl.binds, Declares::Input, file, line)?;
        match &decl.form {
            InputForm::Observed => input.observed = true,
            InputForm::Typed(typed) => {
                let (ty, factor) = self.decl_type(&typed.ty, &input.binds, file, line)?;
                input.reading = typed.reading;
                input.question.clone_from(&typed.question);
                if let Some(year) = &typed.prices {
                    input.prices = Some(prices_year(year, &ty, file, line)?);
                    input.prices_by = typed
                        .prices_by
                        .as_ref()
                        .map(|by| prices_by(by, file, line))
                        .transpose()?;
                }
                input.ty = Some(ty);
                input.unit_factor = factor;
            }
        }
        input.time_pos = self.time_pos(&input.binds);
        self.m.inputs.push(&decl.id, input);
        Ok(())
    }

    /// A def: its key binds, type and rules, each label once. L §5.11.
    fn declare_def(&mut self, decl: &Def, file: &str) -> Result<(), Stop> {
        let binds = self.decl_binds(&decl.binds, Declares::Def, file, decl.line)?;
        let (ty, unit_factor) = self.decl_type(&decl.ty, &binds, file, decl.line)?;
        let mut rules: Vec<(String, Vec<RuleVer>)> = Vec::new();
        for rule in &decl.rules {
            if rules.iter().any(|(label, _)| *label == rule.label) {
                return Err(refuse(
                    Load::Label,
                    format!(
                        "rule {} already exists in {}; write replace",
                        rule.label, decl.id
                    ),
                    file,
                    rule.line,
                ));
            }
            if let Some(v) = &rule.v
                && !self.m.versions.contains(v)
            {
                return Err(refuse(
                    Load::Name,
                    format!("rule {}.{}: v {v} names no version", decl.id, rule.label),
                    file,
                    rule.line,
                ));
            }
            let span = rule
                .governs
                .as_ref()
                .map(|g| lspan_to_span(g, file, rule.line))
                .transpose()?;
            let first = RuleVer {
                label: rule.label.clone(),
                rule: rule.clone(),
                version: None,
                v: rule.v.clone(),
                span,
                replace: false,
            };
            rules.push((rule.label.clone(), vec![first]));
        }
        let time_pos = self.time_pos(&binds);
        let def = DefInfo {
            id: decl.id.clone(),
            binds,
            ty,
            unit_factor,
            at: decl.at.clone(),
            rules,
            givens: Vec::new(),
            time_pos,
            decl: decl.clone(),
        };
        self.m.defs.push(&decl.id, def);
        Ok(())
    }

    /// A band: at most one key, its argument's type (never things), its value's type. L §5.9.
    fn declare_band(&mut self, decl: &Band, file: &str) -> Result<(), Stop> {
        let binds = self.decl_binds(&decl.binds, Declares::Band, file, decl.line)?;
        if binds.len() > 1 {
            return Err(refuse(
                Load::Parse,
                "cannot parse: a keyed band has one key".to_owned(),
                file,
                decl.line,
            ));
        }
        let (arg_type, _) = self.decl_type(&decl.arg_type, &binds, file, decl.line)?;
        if let Ty::Key(over) = &arg_type
            && self.m.kinds.contains(over)
        {
            return Err(refuse(
                Load::Unit,
                format!("band {} over things of {over}", decl.id),
                file,
                decl.line,
            ));
        }
        let (ty, unit_factor) = self.decl_type(&decl.ty, &binds, file, decl.line)?;
        let band = BandInfo {
            id: decl.id.clone(),
            binds,
            arg: decl.arg.clone(),
            arg_type,
            ty,
            unit_factor,
            tables: Vec::new(),
        };
        self.m.bands.push(&decl.id, band);
        Ok(())
    }

    /// A place section: its key binds, defined for every key; its kind and things come later.
    /// L §5.10.
    fn declare_place(&mut self, decl: &Place, file: &str) -> Result<(), Stop> {
        let binds = self.decl_binds(&decl.binds, Declares::Place, file, decl.line)?;
        if binds.iter().any(|b| b.node.filter.is_some()) {
            return Err(refuse(
                Load::Domain,
                format!(
                    "the key bind of place {} is defined for every key (L §5.7)",
                    decl.id
                ),
                file,
                decl.line,
            ));
        }
        let place = PlaceInfo {
            id: decl.id.clone(),
            kind: String::new(),
            binds,
            unit: decl.unit.clone(),
            priced: decl.priced.clone(),
            lives: Span::always(),
            things: Vec::new(),
            decl: decl.clone(),
        };
        self.m.places.push(&decl.id, place);
        Ok(())
    }

    /// A move: its key binds and event kind; a decision when it states `reads`. L §5.14.
    fn declare_move(&mut self, decl: &Move, file: &str) -> Result<(), Stop> {
        let binds = self.decl_binds(&decl.binds, Declares::Move, file, decl.line)?;
        let kind = self.move_kind(&decl.id, &decl.kind, file, decl.line)?;
        let info = MoveInfo {
            id: decl.id.clone(),
            kind,
            binds,
            decision: !decl.reads.is_empty(),
            stratum: 0,
            decl: decl.clone(),
        };
        self.m.moves.push(&decl.id, info);
        Ok(())
    }

    /// A move's event kind: declared, an event kind, not `adjustments` (refcore's
    /// `resolve_move_kind`). L §5.14, §5.14.3.
    fn move_kind(&self, id: &str, kind: &str, file: &str, line: u32) -> Result<String, Stop> {
        if kind.contains('.') {
            return Err(Stop::NotBuilt("a member's prefix"));
        }
        let Some(info) = self
            .m
            .kinds
            .get(kind)
            .filter(|_| self.m.names.get(kind) == Some(&NameKind::Kind))
        else {
            return Err(refuse(Load::Name, format!("unknown id {kind}"), file, line));
        };
        if info.spec != KindSpec::Event {
            return Err(refuse(
                Load::Kind,
                format!("move {id} of {kind}, not an event kind"),
                file,
                line,
            ));
        }
        if self.m.chain(kind).iter().any(|k| k.id == "adjustments") {
            return Err(refuse(
                Load::Kind,
                format!("move {id} of {kind}: only the engine makes adjustments"),
                file,
                line,
            ));
        }
        Ok(kind.to_owned())
    }

    /// Where a declaration's one period key is (refcore's `time_pos`). L §4.6 step 1.
    fn time_pos(&self, binds: &[BindInfo]) -> TimePos {
        let mut positions: Vec<usize> = Vec::new();
        for (at, bind) in binds.iter().enumerate() {
            let Domain::Index(id) = &bind.domain else {
                continue;
            };
            if self.m.indexes.get(id).is_some_and(|ix| ix.period.is_some()) {
                positions.push(at);
            }
        }
        match positions.as_slice() {
            [] => TimePos::None,
            [one] => TimePos::At(*one),
            [_, _, ..] => TimePos::Multi,
        }
    }

    /// A declaration's key binds (refcore's `decl_binds`). L §5.6.
    fn decl_binds(
        &self,
        binds: &[Bind],
        what: Declares,
        file: &str,
        line: u32,
    ) -> Result<Vec<BindInfo>, Stop> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for bind in binds {
            let var = &bind.var;
            if !seen.insert(var.clone()) {
                return Err(refuse(
                    Load::Dup,
                    format!("bound variable {var} is declared twice"),
                    file,
                    line,
                ));
            }
            if self.m.names.contains_key(var) {
                return Err(refuse(
                    Load::Shadow,
                    format!("bound variable {var} has the name of a top-level id"),
                    file,
                    line,
                ));
            }
            if (bind.on.is_some() || bind.over.is_some()) && what != Declares::Move {
                return Err(refuse(
                    Load::Domain,
                    format!(
                        "an on or over clause on the key bind of a {}, which is defined for every key",
                        what.word()
                    ),
                    file,
                    line,
                ));
            }
            let domain = match &bind.domain {
                Written::Name { id, qual } => {
                    if qual.is_some() {
                        return Err(Stop::NotBuilt("a member's prefix"));
                    }
                    let domain = self.resolve_domain(id, file, line)?;
                    self.check_key_shadow(var, &domain, file, line)?;
                    domain
                }
                Written::List(keys) => Domain::List(keys.clone()),
                Written::Range(_) => Domain::Range,
            };
            out.push(BindInfo {
                var: var.clone(),
                domain,
                node: bind.clone(),
            });
        }
        Ok(out)
    }

    /// An index, a kind or an axis by name (refcore's `resolve_domain`). L §5.6.
    fn resolve_domain(&self, id: &str, file: &str, line: u32) -> Result<Domain, Stop> {
        match self.m.names.get(id) {
            Some(NameKind::Index) => Ok(Domain::Index(id.to_owned())),
            Some(NameKind::Kind) => Ok(Domain::Kind(id.to_owned())),
            Some(NameKind::Axis) => Ok(Domain::Axis(id.to_owned())),
            Some(_) => Err(refuse(
                Load::Key,
                format!("{id} is not an index, a kind or an axis"),
                file,
                line,
            )),
            None => Err(refuse(Load::Name, format!("unknown id {id}"), file, line)),
        }
    }

    /// A bound variable spelled as a key of its own domain is refused. L §2.5.
    fn check_key_shadow(
        &self,
        var: &str,
        domain: &Domain,
        file: &str,
        line: u32,
    ) -> Result<(), Stop> {
        let shadows = match domain {
            Domain::Index(id) => self
                .m
                .indexes
                .get(id)
                .is_some_and(|ix| ix.by_text(var).is_some()),
            Domain::Kind(kind) => self.m.thing_in(var, kind),
            Domain::Axis(_) | Domain::List(_) | Domain::Range => false,
        };
        if shadows {
            let id = match domain {
                Domain::Index(id) | Domain::Kind(id) | Domain::Axis(id) => id.as_str(),
                Domain::List(_) | Domain::Range => "",
            };
            return Err(refuse(
                Load::Shadow,
                format!("bound variable {var} has the spelling of a key of {id}"),
                file,
                line,
            ));
        }
        Ok(())
    }

    /// A written type as a type, with the factor a `%` in its unit brings (refcore's
    /// `decl_type`). L §3.1, §3.2.
    pub(super) fn decl_type(
        &self,
        ty: &Type,
        binds: &[BindInfo],
        file: &str,
        line: u32,
    ) -> Result<(Ty, Num), Stop> {
        match ty {
            Type::Money(code) => {
                if !self.m.currencies.contains_key(code) {
                    return Err(undeclared(code, file, line));
                }
                Ok((Ty::Money(code.clone()), Num::int(1)))
            }
            Type::Num(unit) => {
                let (scaled, factor) = self.unit_of(unit.as_ref(), binds, file, line)?;
                if is_percent(unit.as_ref()) {
                    return Ok((Ty::Num(Unit::one(), Shown::Percent), factor));
                }
                Ok((Ty::Num(scaled, Shown::Plain), factor))
            }
            Type::Dist(unit) => {
                let (scaled, factor) = self.unit_of(unit.as_ref(), binds, file, line)?;
                Ok((Ty::Dist(scaled), factor))
            }
            Type::Key(name) => {
                if name.contains('.') {
                    return Err(Stop::NotBuilt("a member's prefix"));
                }
                match self.m.names.get(name) {
                    Some(NameKind::Kind | NameKind::Index | NameKind::Axis) => {
                        Ok((Ty::Key(name.clone()), Num::int(1)))
                    }
                    Some(_) => Err(refuse(
                        Load::Key,
                        format!("{name} is not an index or a kind"),
                        file,
                        line,
                    )),
                    None => Err(refuse(Load::Name, format!("unknown id {name}"), file, line)),
                }
            }
            Type::Record(fields) => {
                let mut out = Vec::new();
                let mut names = BTreeSet::new();
                for (name, field) in fields {
                    if !names.insert(name.clone()) {
                        return Err(refuse(Load::Dup, format!("field {name} twice"), file, line));
                    }
                    let (field_ty, _) = self.decl_type(field, binds, file, line)?;
                    out.push((name.clone(), field_ty));
                }
                Ok((Ty::Record(out), Num::int(1)))
            }
            Type::Word(word) => word_type(word)
                .map(|t| (t, Num::int(1)))
                .ok_or(Stop::NotBuilt("a type of that word")),
        }
    }

    /// A written unit with its terms resolved (refcore's `unit_of`). L §3.2.
    pub(super) fn unit_of(
        &self,
        unit: Option<&WrittenUnit>,
        binds: &[BindInfo],
        file: &str,
        line: u32,
    ) -> Result<(Unit, Num), Stop> {
        let Some(written) = unit else {
            return Ok((Unit::one(), Num::int(1)));
        };
        let mut terms = Vec::new();
        for UnitTerm { base, power } in &written.terms {
            let term = if base == "%"
                || self.m.currencies.contains_key(base)
                || self.m.units.contains(base)
                || PERIODS.contains(&base.as_str())
            {
                base.clone()
            } else if binds.iter().any(|b| b.var == *base) {
                format!("${base}")
            } else if is_currency_code(base) {
                return Err(undeclared(base, file, line));
            } else if self.m.unit_kind_keys.contains_key(base) {
                base.clone()
            } else {
                return Err(refuse(
                    Load::Name,
                    format!("unknown id {base} (a unit)"),
                    file,
                    line,
                ));
            };
            terms.push((term, *power));
        }
        Unit::from_syntax(terms).map_err(num_stop)
    }
}

/// `LOAD-UNIT money in an undeclared currency C`.
fn undeclared(code: &str, file: &str, line: u32) -> Stop {
    refuse(
        Load::Unit,
        format!("money in an undeclared currency {code}"),
        file,
        line,
    )
}

/// Whether a written unit is exactly `%`.
fn is_percent(unit: Option<&WrittenUnit>) -> bool {
    let Some(written) = unit else {
        return false;
    };
    let [UnitTerm { base, power }] = written.terms.as_slice() else {
        return false;
    };
    base == "%" && *power == 1
}

/// The type a one-word type names.
fn word_type(word: &str) -> Option<Ty> {
    let ty = match word {
        "int" => Ty::Int,
        "bool" => Ty::Bool,
        "text" => Ty::Text,
        "date" => Ty::Date,
        "dates" => Ty::Dates,
        "month" => Ty::Period(PeriodWord::Month),
        "quarter" => Ty::Period(PeriodWord::Quarter),
        "year" => Ty::Period(PeriodWord::Year),
        "monthday" => Ty::MonthDay,
        "rate" => Ty::Rate,
        "calendar" => Ty::Calendar,
        other => Ty::Code(CodeKind::of_word(other)?),
    };
    Some(ty)
}

/// `prices Y`: a four-digit year, on money. L §3.4.
fn prices_year(year: &Lit, ty: &Ty, file: &str, line: u32) -> Result<i64, Stop> {
    if !matches_money(ty) {
        return Err(refuse(
            Load::Unit,
            format!(
                "prices on an input of type {}; prices is for money",
                ty.show()
            ),
            file,
            line,
        ));
    }
    let Lit::Num(n) = year else {
        return Err(cannot_prices(file, line));
    };
    if !(n.whole && n.unit.is_none() && n.text.len() == 4) {
        return Err(cannot_prices(file, line));
    }
    n.value.to_i64().ok_or_else(|| cannot_prices(file, line))
}

/// `LOAD-PARSE cannot parse: prices takes a year`.
fn cannot_prices(file: &str, line: u32) -> Stop {
    refuse(
        Load::Parse,
        "cannot parse: prices takes a year".to_owned(),
        file,
        line,
    )
}

/// Whether a type is money.
fn matches_money(ty: &Ty) -> bool {
    let Ty::Money(_) = ty else {
        return false;
    };
    true
}

/// `prices Y by ix`: the index input, by name. L §3.4.
fn prices_by(by: &Expr, file: &str, line: u32) -> Result<String, Stop> {
    let Expr::Name(name) = by else {
        return Err(refuse(
            Load::Parse,
            "cannot parse: prices by an index input".to_owned(),
            file,
            line,
        ));
    };
    Ok(name.clone())
}
