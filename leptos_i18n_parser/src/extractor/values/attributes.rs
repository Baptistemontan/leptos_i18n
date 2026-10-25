use crate::utils::Key;

use super::Literal;

#[derive(Debug, Clone, PartialEq)]
pub struct Attributes {
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub key_index: usize,
    pub value: Option<AttributeValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Literal(Literal),
    Variable(Key),
}
