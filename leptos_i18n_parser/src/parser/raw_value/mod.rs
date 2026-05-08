use serde::{Deserialize, de::Visitor};

use crate::{error::Result, parser::dummy::Dummy};

use super::{ParseContext, ParseFn};

pub mod component;
pub mod foreign_key;
pub mod variable;

use component::Component;
use foreign_key::ForeignKey;
use variable::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum RawLiteral {
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawValue {
    ForeignKey(ForeignKey),
    Literal(RawLiteral),
    Variable(Variable),
    Component(Component),
    Bloc(Vec<Self>),
    Dummy(Dummy),
}

impl Default for RawValue {
    fn default() -> Self {
        //
        RawValue::Literal(RawLiteral::String(String::default()))
    }
}

impl RawValue {
    pub const DEFAULT_FNS: &[ParseFn] = &[Component::parse, ForeignKey::parse, Variable::parse];

    pub fn parse(ctx: &ParseContext, s: &str) -> Result<Self, ()> {
        let parsed_value = ctx.parse_fns.iter().find_map(|f| f(ctx, s));
        match parsed_value {
            None => Ok(RawValue::Literal(RawLiteral::String(s.to_string()))),
            Some(Ok(v)) => Ok(v),
            Some(Err(())) => Err(()),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            RawValue::Literal(RawLiteral::String(s)) => s.is_empty(),
            RawValue::Bloc(bloc) => bloc.is_empty(),
            _ => false,
        }
    }
}

struct LiteralVisitor;

impl<'de> Deserialize<'de> for RawLiteral {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(LiteralVisitor)
    }
}

impl Visitor<'_> for LiteralVisitor {
    type Value = RawLiteral;

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawLiteral::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawLiteral::Signed(v))
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawLiteral::Float(v))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawLiteral::Unsigned(v))
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawLiteral::String(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawLiteral::String(v.to_string()))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a litteral such as a number, a string or a boolean"
        )
    }
}
