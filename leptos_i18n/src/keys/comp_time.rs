use super::args::Args;
use super::builder::ArgsBuilder;

pub trait ConstArgsMarker: ArgsBuilder {
    type Builded;
    type Value: LiteralValue;
    type Args: ConstArgs<Locale = Self::Locale, Id = Self::Id, Value = Self::Value>;
}

pub trait ConstArgs: Args + Copy + 'static {
    const THIS: Self;
    type Value: LiteralValue;

    fn value(id: Self::Id, locale: Self::Locale) -> Self::Value;
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum NoRecurse {}

pub enum NoArgs {}

#[doc(hidden)]
pub trait LiteralValue: Copy + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Literal<M = &'static [Literal<NoRecurse>]>
where
    M: LiteralValue,
{
    String(&'static str),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
    Multiple(M),
}

impl LiteralValue for bool {}
impl LiteralValue for i64 {}
impl LiteralValue for u64 {}
impl LiteralValue for f64 {}
impl LiteralValue for &'static str {}
impl LiteralValue for &'static [Literal<NoRecurse>] {}
impl LiteralValue for Literal {}
impl LiteralValue for NoRecurse {}

impl<M: LiteralValue> Literal<M> {
    pub const fn str(self) -> Option<&'static str> {
        if let Literal::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn signed(self) -> Option<i64> {
        if let Literal::Signed(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn unsigned(self) -> Option<u64> {
        if let Literal::Unsigned(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn float(self) -> Option<f64> {
        if let Literal::Float(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn bool(self) -> Option<bool> {
        if let Literal::Bool(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn multiple(self) -> Option<M> {
        if let Literal::Multiple(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
