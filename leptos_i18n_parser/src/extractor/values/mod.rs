use std::collections::BTreeMap;

use crate::error::Diagnostics;
use crate::extractor::StringIndexer;
use crate::extractor::values::attributes::{Attribute, AttributeValue};
use crate::extractor::values::foreign_key::{ResolvedLocale, ResolvedValue};
use crate::parser::options::Config;
use crate::parser::raw_value::RawLiteral;
use crate::parser::raw_value::component::{
    Component, RawAttribute, RawAttributeValue, RawAttributes,
};
use crate::parser::raw_value::variable::Variable;
use crate::utils::Key;
use crate::utils::Location;

pub mod attributes;
pub mod foreign_key;
pub mod plurals;

use super::BuilderId;
use super::defaults::DefaultedLocales;

use attributes::Attributes;
use plurals::Plurals;

#[derive(Debug, Clone, PartialEq)]
pub struct Keys {
    pub values: BTreeMap<Key, ValuesOrSubkeys>,
    pub defaults: DefaultedLocales,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValuesOrSubkeys {
    Values(Values),
    Subkeys(Keys),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Values {
    pub builder_id: BuilderId,
    pub values: BTreeMap<Key, Value>,
    pub defaults: DefaultedLocales,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Literal(Literal),
    Variable(Variable),
    Component(Component<Self, Attributes>),
    Bloc(Vec<Self>),
    Plurals(Plurals),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(usize),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

pub fn merge_and_index_keys(
    values: ResolvedLocale,
    default_locale: &Key,
    keys: &mut Keys,
    loc: &mut Location,
    cfg: &Config,
    str_indexer: &mut StringIndexer,
    diag: &Diagnostics,
) {
    // TODO: diag
    let _ = diag;
    let current_locale_name = values.name;
    let is_default = current_locale_name.key == *default_locale;
    for (key, value) in values.values.values {
        let mut loc = loc.push_key(key.clone());
        match value {
            foreign_key::ResolvedValueOrSubkeys::Value(resolved_value) => {
                let keys = keys
                    .values
                    .entry(key.clone())
                    .or_insert(ValuesOrSubkeys::Values(Values::new(default_locale.clone())));
                let ValuesOrSubkeys::Values(keys) = keys else {
                    todo!("missmatch")
                };
                let value = reduce_and_index_value(resolved_value, str_indexer);
                keys.values.insert(current_locale_name.key.clone(), value);
            }
            foreign_key::ResolvedValueOrSubkeys::Subkeys(sk) => {
                let keys = keys
                    .values
                    .entry(key.clone())
                    .or_insert(ValuesOrSubkeys::Subkeys(Keys::new(default_locale.clone())));
                let ValuesOrSubkeys::Subkeys(keys) = keys else {
                    todo!("missmatch")
                };
                let values = ResolvedLocale {
                    name: current_locale_name.clone(),
                    values: sk,
                };

                merge_and_index_keys(
                    values,
                    default_locale,
                    keys,
                    &mut loc,
                    cfg,
                    str_indexer,
                    diag,
                );
            }
            foreign_key::ResolvedValueOrSubkeys::Defaulted => {
                if is_default {
                    todo!("default in default locale");
                }

                let Some(values) = keys.values.get_mut(&key) else {
                    todo!("key not present in default locale");
                };

                let defaults = match values {
                    ValuesOrSubkeys::Values(values) => &mut values.defaults,
                    ValuesOrSubkeys::Subkeys(keys) => &mut keys.defaults,
                };

                defaults.push(current_locale_name.key.clone(), cfg);
            }
            foreign_key::ResolvedValueOrSubkeys::Dummy(_) => todo!(),
        }
    }
}

fn reduce_value(value: ResolvedValue) -> ResolvedValue {
    let mut bloc = Vec::new();
    reduce_value_into(value, &mut bloc);
    match &mut *bloc {
        [] => ResolvedValue::Literal(RawLiteral::String(String::new())),
        [value] => core::mem::replace(
            value,
            ResolvedValue::Literal(RawLiteral::String(String::new())),
        ),
        _ => ResolvedValue::Bloc(bloc),
    }
}

fn reduce_value_into(value: ResolvedValue, bloc: &mut Vec<ResolvedValue>) {
    match value {
        ResolvedValue::Literal(raw_literal) => {
            if let Some(ResolvedValue::Literal(lit)) = bloc.last_mut() {
                merge_literals(lit, raw_literal);
            } else {
                bloc.push(ResolvedValue::Literal(raw_literal));
            }
        }
        ResolvedValue::Variable(variable) => {
            bloc.push(ResolvedValue::Variable(variable));
        }
        ResolvedValue::Component(component) => {
            let inner = component
                .inner
                .map(|inner| reduce_value(*inner))
                .map(Box::new);
            let comp = ResolvedValue::Component(Component {
                key: component.key,
                inner,
                attributes: component.attributes,
            });
            bloc.push(comp);
        }
        ResolvedValue::Bloc(values) => {
            for value in values {
                reduce_value_into(value, bloc);
            }
        }
        ResolvedValue::Plurals(plurals) => {
            let forms = plurals.forms.map(reduce_value);
            let plurals = ResolvedValue::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: plurals.count_key,
                forms,
            });
            bloc.push(plurals);
        }
    }
}

fn merge_literals(dest: &mut RawLiteral, lit: RawLiteral) {
    use core::fmt::Write;
    let mut buff = match dest {
        RawLiteral::String(s) => core::mem::take(s),
        RawLiteral::Signed(n) => format!("{n}"),
        RawLiteral::Unsigned(n) => format!("{n}"),
        RawLiteral::Float(n) => format!("{n}"),
        RawLiteral::Bool(n) => format!("{n}"),
    };

    match lit {
        RawLiteral::String(s) => buff.push_str(&s),
        RawLiteral::Signed(n) => write!(&mut buff, "{n}").unwrap(),
        RawLiteral::Unsigned(n) => write!(&mut buff, "{n}").unwrap(),
        RawLiteral::Float(n) => write!(&mut buff, "{n}").unwrap(),
        RawLiteral::Bool(n) => write!(&mut buff, "{n}").unwrap(),
    }

    *dest = RawLiteral::String(buff);
}

fn reduce_and_index_value(value: ResolvedValue, str_indexer: &mut StringIndexer) -> Value {
    let value = reduce_value(value);
    match value {
        ResolvedValue::Literal(RawLiteral::Bool(v)) => Value::Literal(Literal::Bool(v)),
        ResolvedValue::Literal(RawLiteral::Unsigned(v)) => Value::Literal(Literal::Unsigned(v)),
        ResolvedValue::Literal(RawLiteral::Signed(v)) => Value::Literal(Literal::Signed(v)),
        ResolvedValue::Literal(RawLiteral::Float(v)) => Value::Literal(Literal::Float(v)),
        ResolvedValue::Literal(RawLiteral::String(v)) => {
            let index = str_indexer.push_str(&v);
            Value::Literal(Literal::String(index))
        }
        ResolvedValue::Variable(variable) => Value::Variable(variable),
        ResolvedValue::Component(component) => {
            let inner = component
                .inner
                .map(|inner| reduce_and_index_value(*inner, str_indexer))
                .map(Box::new);
            let attributes = index_attributes(component.attributes, str_indexer);
            Value::Component(Component {
                key: component.key,
                inner,
                attributes,
            })
        }
        ResolvedValue::Bloc(resolved_values) => {
            let bloc = resolved_values
                .into_iter()
                .map(|v| reduce_and_index_value(v, str_indexer))
                .collect();
            Value::Bloc(bloc)
        }
        ResolvedValue::Plurals(plurals) => {
            let forms = plurals
                .forms
                .map(|form| reduce_and_index_value(form, str_indexer));
            Value::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: plurals.count_key,
                forms,
            })
        }
    }
}

fn index_attributes(raw_attrs: RawAttributes, str_indexer: &mut StringIndexer) -> Attributes {
    let attrs = raw_attrs
        .attrs
        .into_iter()
        .map(|attr| index_attribute(attr, str_indexer))
        .collect();
    Attributes { attrs }
}

fn index_attribute(raw_attr: RawAttribute, str_indexer: &mut StringIndexer) -> Attribute {
    let key_index = str_indexer.push_str(&raw_attr.key);
    Attribute {
        key_index,
        value: raw_attr
            .value
            .map(|v| index_attribute_value(v, str_indexer)),
    }
}

fn index_attribute_value(
    value: RawAttributeValue,
    str_indexer: &mut StringIndexer,
) -> AttributeValue {
    match value {
        RawAttributeValue::Literal(RawLiteral::Bool(v)) => {
            AttributeValue::Literal(Literal::Bool(v))
        }
        RawAttributeValue::Literal(RawLiteral::Unsigned(v)) => {
            AttributeValue::Literal(Literal::Unsigned(v))
        }
        RawAttributeValue::Literal(RawLiteral::Signed(v)) => {
            AttributeValue::Literal(Literal::Signed(v))
        }
        RawAttributeValue::Literal(RawLiteral::Float(v)) => {
            AttributeValue::Literal(Literal::Float(v))
        }
        RawAttributeValue::Literal(RawLiteral::String(v)) => {
            let index = str_indexer.push_str(&v);
            AttributeValue::Literal(Literal::String(index))
        }
        RawAttributeValue::Variable(key) => AttributeValue::Variable(key),
    }
}

impl Values {
    pub fn new(default_locale: Key) -> Self {
        Values {
            builder_id: BuilderId::default(),
            values: BTreeMap::default(),
            defaults: DefaultedLocales::new(default_locale),
        }
    }
}

impl Keys {
    pub fn new(default_locale: Key) -> Self {
        Keys {
            values: BTreeMap::default(),
            defaults: DefaultedLocales::new(default_locale),
        }
    }
}
