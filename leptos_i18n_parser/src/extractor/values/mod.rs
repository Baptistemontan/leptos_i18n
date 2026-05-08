use std::collections::BTreeMap;

use crate::error::{Diagnostics, Error, Warning};
use crate::extractor::StringIndexer;
use crate::extractor::values::attributes::{Attribute, AttributeValue};
use crate::extractor::values::foreign_key::{ResolvedLocale, ResolvedValue};
use crate::parser::dummy::Dummy;
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Keys {
    pub values: BTreeMap<Key, ValuesOrSubkeys>,
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
    Dummy(Dummy),
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
    let current_locale_name = values.name;
    let is_default = current_locale_name.key == *default_locale;

    if !is_default {
        for (key, values_or_sk) in keys.values.iter_mut() {
            if !values.values.values.contains_key(key) {
                let loc = loc.push_key(key.clone());
                diag.emit_warning(Warning::MissingKey { loc: loc.clone() });
                default_locale_for_values_or_sk(values_or_sk, &current_locale_name.key, cfg);
            }
        }
    }

    for (key, value) in values.values.values {
        let mut loc = loc.push_key(key.clone());
        match value {
            foreign_key::ResolvedValueOrSubkeys::Value(resolved_value) => {
                let keys = keys
                    .values
                    .entry(key.clone())
                    .or_insert(ValuesOrSubkeys::Values(Values::new(default_locale.clone())));
                let values = match keys {
                    ValuesOrSubkeys::Values(values) => values,
                    ValuesOrSubkeys::Subkeys(keys) => {
                        diag.emit_error(Error::SubKeyMissmatch { loc: loc.clone() });
                        default_locale_for_subkeys(keys, &current_locale_name.key, cfg);
                        continue;
                    }
                };
                let value = reduce_and_index_value(resolved_value, str_indexer);
                values.values.insert(current_locale_name.key.clone(), value);
            }
            foreign_key::ResolvedValueOrSubkeys::Subkeys(sk) => {
                let keys = keys
                    .values
                    .entry(key.clone())
                    .or_insert(ValuesOrSubkeys::Subkeys(Keys::default()));
                let keys = match keys {
                    ValuesOrSubkeys::Subkeys(keys) => keys,
                    ValuesOrSubkeys::Values(values) => {
                        diag.emit_error(Error::SubKeyMissmatch { loc: loc.clone() });
                        default_locale_for_values(values, &current_locale_name.key, cfg);
                        continue;
                    }
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
                    diag.emit_error(Error::ExplicitDefaultInDefault(loc.key_path.clone()));
                    continue;
                }

                let Some(values) = keys.values.get_mut(&key) else {
                    diag.emit_warning(Warning::SurplusKey { loc: loc.clone() });
                    continue;
                };

                default_locale_for_values_or_sk(values, &current_locale_name.key, cfg);
            }
        }
    }
}

fn default_locale_for_values_or_sk(
    values: &mut ValuesOrSubkeys,
    locale_to_default: &Key,
    cfg: &Config,
) {
    match values {
        ValuesOrSubkeys::Values(values) => {
            default_locale_for_values(values, locale_to_default, cfg)
        }
        ValuesOrSubkeys::Subkeys(keys) => default_locale_for_subkeys(keys, locale_to_default, cfg),
    }
}

fn default_locale_for_values(values: &mut Values, locale_to_default: &Key, cfg: &Config) {
    values.defaults.push(locale_to_default.clone(), cfg);
}

fn default_locale_for_subkeys(keys: &mut Keys, locale_to_default: &Key, cfg: &Config) {
    for values in keys.values.values_mut() {
        default_locale_for_values_or_sk(values, locale_to_default, cfg);
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
        ResolvedValue::Literal(RawLiteral::String(s)) if s.is_empty() => {}
        ResolvedValue::Literal(RawLiteral::String(s)) => {
            // merge strings together
            if let Some(ResolvedValue::Literal(RawLiteral::String(buff))) = bloc.last_mut() {
                buff.push_str(&s);
            } else {
                bloc.push(ResolvedValue::Literal(RawLiteral::String(s)));
            }
        }
        ResolvedValue::Literal(raw_literal) => {
            bloc.push(ResolvedValue::Literal(raw_literal));
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
        ResolvedValue::Dummy(dummy) => bloc.push(ResolvedValue::Dummy(dummy)),
    }
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
        ResolvedValue::Dummy(dummy) => Value::Dummy(dummy),
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
