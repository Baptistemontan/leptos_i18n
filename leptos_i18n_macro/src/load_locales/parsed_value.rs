use std::{collections::HashSet, rc::Rc};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    error::Error,
    key::Key,
    plural::{PluralType, Plurals},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    Plural(Plurals),
    String(String),
    Variable(Rc<Key>),
    Component { key: Rc<Key>, inner: Box<Self> },
    Bloc(Vec<Self>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum InterpolateKey {
    Count(PluralType),
    Variable(Rc<Key>),
    Component(Rc<Key>),
}

impl ParsedValue {
    pub fn get_keys_inner(&self, keys: &mut Option<HashSet<InterpolateKey>>) {
        match self {
            ParsedValue::String(_) => {}
            ParsedValue::Variable(key) => {
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKey::Variable(Rc::clone(key)));
            }
            ParsedValue::Component { key, inner } => {
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKey::Component(Rc::clone(key)));
                inner.get_keys_inner(keys);
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.get_keys_inner(keys)
                }
            }
            ParsedValue::Plural(plurals) => {
                plurals.get_keys_inner(keys);
                let plural_type = plurals.get_type();
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKey::Count(plural_type));
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

        let ident = Key::new(&format!("var_{}", ident.trim()))?;

        let before = Self::new(before);
        let after = Self::new(after);
        let this = ParsedValue::Variable(Rc::new(ident));

        Some(ParsedValue::Bloc(vec![before, this, after]))
    }

    fn find_valid_component(value: &str) -> Option<(Rc<Key>, &str, &str, &str)> {
        let mut skip_sum = 0;
        loop {
            let (before, key, after, skip) = Self::find_opening_tag(&value[skip_sum..])?;
            if let Some((key, beetween, after)) = Self::find_closing_tag(after, key) {
                let before_len = skip_sum + before.len();
                let before = &value[..before_len];
                break Some((Rc::new(key), before, beetween, after));
            } else {
                skip_sum += skip;
            }
        }
    }

    fn find_component(value: &str) -> Option<Self> {
        let (key, before, beetween, after) = Self::find_valid_component(value)?;

        let before = ParsedValue::new(before);
        let beetween = ParsedValue::new(beetween);
        let after = ParsedValue::new(after);

        let this = ParsedValue::Component {
            key,
            inner: beetween.into(),
        };

        Some(ParsedValue::Bloc(vec![before, this, after]))
    }

    fn find_closing_tag<'a>(value: &'a str, key: &str) -> Option<(Key, &'a str, &'a str)> {
        let key_ident = Key::new(&format!("comp_{}", key))?;
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

        Some((key_ident, before, after))
    }

    fn find_opening_tag(value: &str) -> Option<(&str, &str, &str, usize)> {
        let (before, rest) = value.split_once('<')?;
        let (ident, after) = rest.split_once('>')?;

        let skip = before.len() + ident.len() + 2;

        Some((before, ident.trim(), after, skip))
    }

    fn flatten(&self, tokens: &mut Vec<TokenStream>) {
        match self {
            ParsedValue::String(s) if s.is_empty() => {}
            ParsedValue::String(s) => tokens.push(quote!(leptos::IntoView::into_view(#s))),
            ParsedValue::Plural(plurals) => tokens.push(plurals.to_token_stream()),
            ParsedValue::Variable(key) => {
                tokens.push(quote!(leptos::IntoView::into_view(core::clone::Clone::clone(&#key))))
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
                    move || Into::into(#inner)
                });
                let boxed_fn = quote!(Box::new(#f));
                tokens.push(quote!(leptos::IntoView::into_view(core::clone::Clone::clone(&#key)( #boxed_fn))))
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.flatten(tokens)
                }
            }
        }
    }
}

impl InterpolateKey {
    pub fn as_ident(&self) -> syn::Ident {
        match self {
            InterpolateKey::Variable(key) | InterpolateKey::Component(key) => key.ident.clone(),
            InterpolateKey::Count(_) => format_ident!("var_count"),
        }
    }

    pub fn as_key(&self) -> Option<&Key> {
        match self {
            InterpolateKey::Variable(key) | InterpolateKey::Component(key) => Some(key),
            InterpolateKey::Count(_) => None,
        }
    }

    #[cfg(feature = "debug_interpolations")]
    pub fn get_real_name(&self) -> &str {
        match self {
            InterpolateKey::Count(_) => "count",
            InterpolateKey::Variable(key) => key.name.strip_prefix("var_").unwrap(),
            InterpolateKey::Component(key) => key.name.strip_prefix("comp_").unwrap(),
        }
    }

    pub fn get_generic(&self) -> TokenStream {
        match self {
            InterpolateKey::Variable(_) => {
                quote!(leptos::IntoView + core::clone::Clone + 'static)
            }
            InterpolateKey::Count(plural_type) => {
                quote!(Fn() -> #plural_type + core::clone::Clone + 'static)
            }
            InterpolateKey::Component(_) => quote!(
                Fn(leptos::ChildrenFn) -> leptos::View
                    + core::clone::Clone
                    + 'static
            ),
        }
    }
}

impl ToTokens for InterpolateKey {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_ident().to_tokens(tokens)
    }
}

impl ToTokens for ParsedValue {
    fn to_token_stream(&self) -> TokenStream {
        let mut tokens = Vec::new();
        self.flatten(&mut tokens);

        match &tokens[..] {
            [] => quote!(leptos::View::default()),
            [value] => value.clone(),
            values => quote!(leptos::CollectView::collect_view([#(#values,)*])),
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_token_stream().to_tokens(tokens)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedValueSeed(pub bool);

impl<'de> serde::de::DeserializeSeed<'de> for ParsedValueSeed {
    type Value = ParsedValue;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for ParsedValueSeed {
    type Value = ParsedValue;

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::new(v))
    }

    fn visit_seq<A>(mut self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // nested plurals are not allowed, the code technically supports it,
        // but it's pointless and probably nobody will ever needs it.
        if std::mem::replace(&mut self.0, true) {
            return Err(serde::de::Error::custom(Error::NestedPlurals));
        }
        let plurals = Plurals::from_serde_seq(map, self)?;

        let (invalid_fallback, fallback_count, should_have_fallback) =
            plurals.check_deserialization();

        if invalid_fallback {
            Err(serde::de::Error::custom(Error::InvalidFallback))
        } else if fallback_count > 1 {
            Err(serde::de::Error::custom(Error::MultipleFallbacks))
        } else if fallback_count == 0 && should_have_fallback {
            Err(serde::de::Error::custom(Error::MissingFallback(
                plurals.get_type(),
            )))
        } else {
            Ok(ParsedValue::Plural(plurals))
        }
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "either a string, a sequence of plurals or a map of subkeys"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_key(key: &str) -> Rc<Key> {
        Rc::new(Key::new(key).unwrap())
    }

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
                ParsedValue::Variable(new_key("var_var")),
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
                    key: new_key("comp_comp"),
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
                    key: new_key("comp_comp"),
                    inner: Box::new(ParsedValue::Bloc(vec![
                        ParsedValue::String("inner before".to_string()),
                        ParsedValue::Component {
                            key: new_key("comp_comp"),
                            inner: Box::new(ParsedValue::String("inner inner".to_string()))
                        },
                        ParsedValue::String("inner after".to_string()),
                    ]))
                },
                ParsedValue::String(" after".to_string())
            ])
        )
    }

    #[test]
    fn parse_skipped_tag() {
        let value = ParsedValue::new("<p>test<h3>this is a h3</h3>not closing p");

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::String("<p>test".to_string()),
                ParsedValue::Component {
                    key: new_key("comp_h3"),
                    inner: Box::new(ParsedValue::String("this is a h3".to_string()))
                },
                ParsedValue::String("not closing p".to_string())
            ])
        )
    }
}
