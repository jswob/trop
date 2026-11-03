//! Static types: what a declaration holds and what an expression gives, as load checks them.
//! Spec: L §3.1 (types), §3.2 (`num %`, units in types), §3.3 (rates), §2.1 (four-digit
//! figures), §3.2 Inline figures.
//! Twin: engine/tools/refcore/ty.py (`Ty`, `show`, `CODE_WORDS`, `code_kind_of`).
//! Invariants: `num %` is `num` as a type and differs only in how it prints; two types are
//! equal when refcore's `Ty.key` is.

use super::unit::Unit;

/// The words of a code type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeKind {
    /// `down`, `up`, `floor`, `ceil`, `half-up`, `half-down`, `half-even`.
    Rounding,
    /// `act365`.
    Daycount,
    /// `following`, `preceding`, `modified_following`, `unadjusted`.
    Roll,
    /// `simple`, `effective`, `discount`, `nominal`.
    Compounding,
}

impl CodeKind {
    /// The type's name as it prints.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            CodeKind::Rounding => "rounding",
            CodeKind::Daycount => "daycount",
            CodeKind::Roll => "roll",
            CodeKind::Compounding => "compounding",
        }
    }

    /// The code type a word belongs to (refcore's `code_kind_of`). L §3.1.
    #[must_use]
    pub fn of_word(word: &str) -> Option<CodeKind> {
        match word {
            "down" | "up" | "floor" | "ceil" | "half-up" | "half-down" | "half-even" => {
                Some(CodeKind::Rounding)
            }
            "act365" => Some(CodeKind::Daycount),
            "following" | "preceding" | "modified_following" | "unadjusted" => Some(CodeKind::Roll),
            "simple" | "effective" | "discount" | "nominal" => Some(CodeKind::Compounding),
            _ => None,
        }
    }
}

/// A period word of a period type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodWord {
    /// `month`.
    Month,
    /// `quarter`.
    Quarter,
    /// `year`.
    Year,
}

/// How a number prints: plain, or with `%` (`num %`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shown {
    /// As its value.
    Plain,
    /// As a percentage.
    Percent,
}

/// A static type (L §3.1).
#[derive(Debug, Clone)]
pub enum Ty {
    /// `num u`; `num %` is `num` shown with `%`.
    Num(Unit, Shown),
    /// `money CUR`.
    Money(String),
    /// `int`.
    Int,
    /// `bool`.
    Bool,
    /// `text`.
    Text,
    /// `date`.
    Date,
    /// `dates`.
    Dates,
    /// `month`, `quarter`, `year`.
    Period(PeriodWord),
    /// `key ix`: a key of an index, a kind or an axis.
    Key(String),
    /// `rate`.
    Rate,
    /// `dist u`.
    Dist(Unit),
    /// `calendar`.
    Calendar,
    /// A code type.
    Code(CodeKind),
    /// `monthday`.
    MonthDay,
    /// A record of named fields.
    Record(Vec<(String, Ty)>),
    /// A list.
    List(Box<Ty>),
    /// A tuple.
    Tuple(Vec<Ty>),
    /// A bare 0, 1 or -1, whose type the context decides (L §3.2 Inline figures).
    Flex(i64),
    /// A four-digit integer: a year or an int (L §2.1).
    Year4,
    /// A key literal not yet resolved by position (L §2.5).
    KeyLit,
    /// `none`.
    None,
    /// A rule value `?"…"`.
    Hole,
    /// `never`, a date.
    Never,
    /// A kind's name where a kind is read (`count(x in k)`).
    KindName,
}

impl PartialEq for Ty {
    fn eq(&self, other: &Ty) -> bool {
        match (self, other) {
            (Ty::Num(a, _), Ty::Num(b, _)) | (Ty::Dist(a), Ty::Dist(b)) => a == b,
            (Ty::Money(a), Ty::Money(b)) | (Ty::Key(a), Ty::Key(b)) => a == b,
            (Ty::Period(a), Ty::Period(b)) => a == b,
            (Ty::Code(a), Ty::Code(b)) => a == b,
            (Ty::Record(a), Ty::Record(b)) => a == b,
            (Ty::List(a), Ty::List(b)) => a == b,
            (Ty::Tuple(a), Ty::Tuple(b)) => a == b,
            (Ty::Flex(a), Ty::Flex(b)) => a == b,
            _ => self.word() == other.word(),
        }
    }
}

impl Eq for Ty {}

impl Ty {
    /// `num` with no unit.
    #[must_use]
    pub fn plain_num() -> Ty {
        Ty::Num(Unit::one(), Shown::Plain)
    }

    /// The type's kind as refcore names it (`Ty.k`).
    #[must_use]
    pub fn word(&self) -> &'static str {
        match self {
            Ty::Num(..) => "num",
            Ty::Money(_) => "money",
            Ty::Int => "int",
            Ty::Bool => "bool",
            Ty::Text => "text",
            Ty::Date => "date",
            Ty::Dates => "dates",
            Ty::Period(_) => "period",
            Ty::Key(_) => "key",
            Ty::Rate => "rate",
            Ty::Dist(_) => "dist",
            Ty::Calendar => "calendar",
            Ty::Code(_) => "code",
            Ty::MonthDay => "mmdd",
            Ty::Record(_) => "record",
            Ty::List(_) => "list",
            Ty::Tuple(_) => "tuple",
            Ty::Flex(_) => "flex",
            Ty::Year4 => "year4",
            Ty::KeyLit => "keylit",
            Ty::None => "none",
            Ty::Hole => "hole",
            Ty::Never => "never",
            Ty::KindName => "kindname",
        }
    }

    /// A record's field type.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&Ty> {
        let Ty::Record(fields) = self else {
            return None;
        };
        fields.iter().find(|(f, _)| f == name).map(|(_, t)| t)
    }

    /// The type as it prints in a message (refcore's `show`). L §3.1.
    #[must_use]
    pub fn show(&self) -> String {
        match self {
            Ty::Num(_, Shown::Percent) => "num %".to_owned(),
            Ty::Num(unit, Shown::Plain) => with_unit("num", unit),
            Ty::Dist(unit) => with_unit("dist", unit),
            Ty::Money(cur) => format!("money {cur}"),
            Ty::Period(PeriodWord::Month) => "month".to_owned(),
            Ty::Period(PeriodWord::Quarter) => "quarter".to_owned(),
            Ty::Period(PeriodWord::Year) => "year".to_owned(),
            Ty::MonthDay => "monthday".to_owned(),
            Ty::Key(ix) => format!("key {ix}"),
            Ty::Code(code) => code.name().to_owned(),
            Ty::Record(fields) => {
                let shown: Vec<String> = fields
                    .iter()
                    .map(|(f, t)| format!("{f} {}", t.show()))
                    .collect();
                format!("({})", shown.join(", "))
            }
            Ty::List(elem) => format!("list of {}", elem.show()),
            Ty::Tuple(items) => {
                let shown: Vec<String> = items.iter().map(Ty::show).collect();
                format!("({})", shown.join(", "))
            }
            Ty::Flex(n) => format!("the figure {n}"),
            Ty::Int
            | Ty::Bool
            | Ty::Text
            | Ty::Date
            | Ty::Dates
            | Ty::Rate
            | Ty::Calendar
            | Ty::Year4
            | Ty::KeyLit
            | Ty::None
            | Ty::Hole
            | Ty::Never
            | Ty::KindName => self.word().to_owned(),
        }
    }
}

/// `word` followed by the unit, when it has one.
fn with_unit(word: &str, unit: &Unit) -> String {
    if unit.is_one() {
        word.to_owned()
    } else {
        format!("{word} {}", unit.text())
    }
}

#[cfg(test)]
mod tests {
    use super::{CodeKind, Shown, Ty};
    use crate::model::Unit;

    #[test]
    fn percent_is_num_as_a_type() {
        let pct = Ty::Num(Unit::one(), Shown::Percent);
        assert_eq!(pct, Ty::plain_num(), "equal");
        assert_eq!(pct.show(), "num %", "shown with %");
        assert_eq!(Ty::Money("PLN".to_owned()).show(), "money PLN", "money");
        assert_eq!(
            CodeKind::of_word("half-up"),
            Some(CodeKind::Rounding),
            "a code"
        );
        assert_ne!(Ty::Int, Ty::Bool, "different words");
    }
}
