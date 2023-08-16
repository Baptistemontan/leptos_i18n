use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    key::Key,
    plural::{Plural, PluralSeed},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedValue {
    Plural(Vec<(Plural, Self)>),
    String(String),
    Variable(Key),
    Component { key: Key, inner: Box<Self> },
    Bloc(Vec<Self>),
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum InterpolateKey<'a> {
    Count,
    Variable(&'a Key),
    Component(&'a Key),
}

impl ParsedValue {
    pub fn get_keys_inner<'a>(&'a self, keys: &mut Option<HashSet<InterpolateKey<'a>>>) {
        match self {
            ParsedValue::String(_) => {}
            ParsedValue::Variable(key) => {
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKey::Variable(key));
            }
            ParsedValue::Component { key, inner } => {
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKey::Component(key));
                inner.get_keys_inner(keys);
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.get_keys_inner(keys)
                }
            }
            ParsedValue::Plural(plurals) => {
                for (_, value) in plurals {
                    value.get_keys_inner(keys);
                }
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKey::Count);
            }
        }
    }

    pub fn get_keys(&self) -> Option<HashSet<InterpolateKey>> {
        let mut keys = None;
        self.get_keys_inner(&mut keys);
        keys
    }

    pub fn is_string(&self) -> Option<&str> {
        match self {
            ParsedValue::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn new(value: &str) -> Self {
        // look for component
        if let Some(component) = Self::find_component(value) {
            return component;
        }
        // else look for variables
        if let Some(variable) = Self::find_variable(value) {
            return variable;
        }

        // else it's just a string
        ParsedValue::String(value.to_string())
    }

    fn find_variable(value: &str) -> Option<Self> {
        let (before, rest) = value.split_once("{{")?;
        let (ident, after) = rest.split_once("}}")?;

        let ident = Key::try_new(&format!("var_{}", ident.trim()))?;

        let before = Self::new(before);
        let after = Self::new(after);
        let this = ParsedValue::Variable(ident);

        Some(ParsedValue::Bloc(vec![before, this, after]))
    }

    fn find_component(value: &str) -> Option<Self> {
        let (before, key, after) = Self::find_opening_tag(value)?;

        let (beetween, after) = Self::find_closing_tag(after, key)?;

        let before = ParsedValue::new(before);
        let beetween = ParsedValue::new(beetween);
        let after = ParsedValue::new(after);

        let key = Key::try_new(&format!("comp_{}", key)).unwrap();

        let this = ParsedValue::Component {
            key,
            inner: beetween.into(),
        };

        Some(ParsedValue::Bloc(vec![before, this, after]))
    }

    fn find_closing_tag<'a>(value: &'a str, key: &str) -> Option<(&'a str, &'a str)> {
        let mut indices = None;
        let mut depth = 0;
        let iter = value.match_indices('<').filter_map(|(i, _)| {
            value[i + 1..]
                .split_once('>')
                .map(|(ident, _)| (i, ident.trim()))
        });
        for (i, ident) in iter {
            if let Some(closing_tag) = ident.strip_prefix('/').map(str::trim_start) {
                if closing_tag != key {
                    continue;
                }
                if depth == 0 {
                    let end_i = i + ident.len() + 2;
                    indices = Some((i, end_i))
                } else {
                    depth -= 1;
                }
            } else if ident == key {
                depth += 1;
            }
        }

        let (start, end) = indices?;

        let before = &value[..start];
        let after = &value[end..];

        Some((before, after))
    }

    fn find_opening_tag(value: &str) -> Option<(&str, &str, &str)> {
        let (before, rest) = value.split_once('<')?;
        let (ident, after) = rest.split_once('>')?;

        Some((before, ident.trim(), after))
    }
}

impl<'a> InterpolateKey<'a> {
    pub fn as_ident(self) -> syn::Ident {
        match self {
            InterpolateKey::Variable(key) | InterpolateKey::Component(key) => key.ident.clone(),
            InterpolateKey::Count => format_ident!("var_count"),
        }
    }

    pub fn as_key(self) -> Option<&'a Key> {
        match self {
            InterpolateKey::Variable(key) | InterpolateKey::Component(key) => Some(key),
            InterpolateKey::Count => None,
        }
    }
}

impl<'a> ToTokens for InterpolateKey<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_ident().to_tokens(tokens)
    }
}

impl ToTokens for ParsedValue {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            // TODO: found way to remove code for empty strings
            ParsedValue::String(s) => quote!(__leptos__::IntoView::into_view(#s, cx)),
            ParsedValue::Variable(key) => {
                quote!(__leptos__::IntoView::into_view(core::clone::Clone::clone(&#key), cx))
            }
            ParsedValue::Bloc(values) => {
                quote!(__leptos__::CollectView::collect_view([#(#values,)*], cx))
            }
            ParsedValue::Component { key, inner } => {
                let captured_keys = inner.get_keys().map(|keys| {
                    let keys = keys
                        .into_iter()
                        .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
                    quote!(#(#keys)*)
                });

                let f = quote!({
                    #captured_keys
                    move |cx| Into::into(#inner)
                });
                let boxed_fn = quote!(Box::new(#f));
                quote!(__leptos__::IntoView::into_view(core::clone::Clone::clone(&#key)(cx, #boxed_fn), cx))
            }
            ParsedValue::Plural(plurals) => {
                let match_arms = plurals
                    .iter()
                    .map(|(plural, value)| quote!(#plural => #value));

                let mut captured_values = None;

                for (_, value) in plurals {
                    value.get_keys_inner(&mut captured_values);
                }

                let captured_values = captured_values.map(|keys| {
                    let keys = keys
                        .into_iter()
                        .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
                    quote!(#(#keys)*)
                });
                let match_statement = quote! {
                    match core::clone::Clone::clone(&count)() {
                        #(
                            #match_arms,
                        )*
                    }
                };

                let f = quote!({
                    #captured_values
                    move || #match_statement
                });

                quote! (__leptos__::IntoView::into_view(#f, cx))
            }
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_token_stream().to_tokens(tokens)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedValueSeed<'a> {
    pub in_plural: bool,
    pub locale_name: &'a str,
    pub locale_key: &'a str,
}

impl<'de> serde::de::DeserializeSeed<'de> for ParsedValueSeed<'_> {
    type Value = ParsedValue;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for ParsedValueSeed<'_> {
    type Value = ParsedValue;

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::new(v))
    }

    fn visit_map<A>(mut self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // nested plurals are not allowed, the code technically supports it,
        // but it's pointless and probably nobody will ever needs it.
        if std::mem::replace(&mut self.in_plural, true) {
            return Err(serde::de::Error::custom(format!(
                "in locale {:?} at key {:?}: nested plurals are not allowed",
                self.locale_name, self.locale_key
            )));
        }
        let plural_seed = PluralSeed {
            locale_name: self.locale_name,
            locale_key: self.locale_key,
        };
        let mut plurals = vec![];

        while let Some(plural) = map.next_key_seed(plural_seed)? {
            let value = map.next_value_seed(self)?;
            plurals.push((plural, value));
        }

        // easy to avoid compile warning, check if a fallback is not at the end position
        let invalid_fallback = plurals
            .iter()
            .rev()
            .skip(1)
            .any(|(plural, _)| matches!(plural, Plural::Fallback));
        // also check if multiple fallbacks exist
        let fallback_count = plurals
            .iter()
            .filter(|(plural, _)| matches!(plural, Plural::Fallback))
            .count();

        if invalid_fallback {
            Err(serde::de::Error::custom(format!(
                "in locale {:?} at key {:?}: fallback is only allowed in last position",
                self.locale_name, self.locale_key
            )))
        } else if fallback_count > 1 {
            Err(serde::de::Error::custom(format!(
                "in locale {:?} at key {:?}: multiple fallbacks are not allowed",
                self.locale_name, self.locale_key
            )))
        } else {
            Ok(ParsedValue::Plural(plurals))
        }
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "either a string or a map of string:string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_normal_string() {
        let value = ParsedValue::new("test");

        assert_eq!(value, ParsedValue::String("test".to_string()));
    }

    #[test]
    fn parse_variable() {
        let value = ParsedValue::new("before {{ var }} after");

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::String("before ".to_string()),
                ParsedValue::Variable(Key::try_new("var_var").unwrap()),
                ParsedValue::String(" after".to_string())
            ])
        )
    }

    #[test]
    fn parse_comp() {
        let value = ParsedValue::new("before <comp>inner</comp> after");

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::String("before ".to_string()),
                ParsedValue::Component {
                    key: Key::try_new("comp_comp").unwrap(),
                    inner: Box::new(ParsedValue::String("inner".to_string()))
                },
                ParsedValue::String(" after".to_string())
            ])
        )
    }

    #[test]
    fn parse_nested_comp() {
        let value = ParsedValue::new(
            "before <comp>inner before<comp>inner inner</comp>inner after</comp> after",
        );

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::String("before ".to_string()),
                ParsedValue::Component {
                    key: Key::try_new("comp_comp").unwrap(),
                    inner: Box::new(ParsedValue::Bloc(vec![
                        ParsedValue::String("inner before".to_string()),
                        ParsedValue::Component {
                            key: Key::try_new("comp_comp").unwrap(),
                            inner: Box::new(ParsedValue::String("inner inner".to_string()))
                        },
                        ParsedValue::String("inner after".to_string()),
                    ]))
                },
                ParsedValue::String(" after".to_string())
            ])
        )
    }
}
