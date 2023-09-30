use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    ops::Deref,
    rc::Rc,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use serde::de::{value::MapAccessDeserializer, DeserializeSeed};

use super::{
    error::{Error, Result},
    key::{Key, KeyPath},
    locale::{Locale, LocaleSeed, LocaleValue, LocalesOrNamespaces},
    plural::{PluralType, Plurals},
};

thread_local! {
    pub static FOREIGN_KEYS: RefCell<HashSet<(Rc<Key>, KeyPath)>> = RefCell::new(HashSet::new());
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForeignKey {
    NotSet(KeyPath),
    Set(Box<ParsedValue>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ParsedValue {
    #[default]
    Default,
    ForeignKey(RefCell<ForeignKey>),
    Plural(Plurals),
    String(String),
    Variable(Rc<Key>),
    Component {
        key: Rc<Key>,
        inner: Box<Self>,
    },
    Bloc(Vec<Self>),
    Subkeys(Locale),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum InterpolateKey {
    Count(PluralType),
    Variable(Rc<Key>),
    Component(Rc<Key>),
}

impl ParsedValue {
    pub fn resolve_foreign_keys(
        values: &LocalesOrNamespaces,
        default_locale: &Rc<Key>,
    ) -> Result<()> {
        FOREIGN_KEYS.with(|foreign_keys| {
            let set = foreign_keys.borrow();
            for (locale, value_path) in &*set {
                let value = values.get_value_at(locale, value_path).unwrap();
                value.resolve_foreign_key(values, locale, default_locale, value_path)?;
            }
            Ok(())
        })
    }

    fn resolve_foreign_key_inner(
        foreign_key: &mut ForeignKey,
        values: &LocalesOrNamespaces,
        top_locale: &Rc<Key>,
        default_locale: &Rc<Key>,
        path: &KeyPath,
    ) -> Result<()> {
        let ForeignKey::NotSet(key_path) = &*foreign_key else {
            // already set, I don't know how we got here but whatever
            return Ok(());
        };

        let Some(value) = values.get_value_at(top_locale, key_path) else {
            return Err(Error::InvalidForeignKey {
                foreign_key: key_path.to_owned(),
                locale: Rc::clone(top_locale),
                key_path: path.to_owned(),
            });
        };

        match value {
            ParsedValue::Default => {
                return Self::resolve_foreign_key_inner(
                    foreign_key,
                    values,
                    default_locale,
                    default_locale,
                    path,
                );
            }
            ParsedValue::Plural(_) => {
                return Err(Error::Custom(format!(
                    "foreign key to plurals is not supported yet, at key {} in locale {:?}",
                    path, top_locale
                )))
            }
            _ => {}
        }

        // possibility that the foreign key must be resolved too
        value.resolve_foreign_key(values, top_locale, default_locale, key_path)?;

        let _ = std::mem::replace(foreign_key, ForeignKey::Set(Box::new(value.clone())));

        Ok(())
    }

    pub fn resolve_foreign_key(
        &self,
        values: &LocalesOrNamespaces,
        top_locale: &Rc<Key>,
        default_locale: &Rc<Key>,
        path: &KeyPath,
    ) -> Result<()> {
        match self {
            ParsedValue::Variable(_) | ParsedValue::String(_) | ParsedValue::Default => Ok(()),
            ParsedValue::Subkeys(_) => Ok(()), // unreachable ?
            ParsedValue::Plural(inner) => {
                inner.resolve_foreign_keys(values, top_locale, default_locale, path)
            }
            ParsedValue::Component { inner, .. } => {
                inner.resolve_foreign_key(values, top_locale, default_locale, path)
            }
            ParsedValue::Bloc(bloc) => {
                for value in bloc {
                    value.resolve_foreign_key(values, top_locale, default_locale, path)?;
                }
                Ok(())
            }
            ParsedValue::ForeignKey(foreign_key) => {
                let Ok(mut foreign_key) = foreign_key.try_borrow_mut() else {
                    return Err(Error::RecursiveForeignKey {
                        locale: Rc::clone(top_locale),
                        key_path: path.to_owned(),
                    });
                };

                Self::resolve_foreign_key_inner(
                    &mut foreign_key,
                    values,
                    top_locale,
                    default_locale,
                    path,
                )
            }
        }
    }

    pub fn get_keys_inner(&self, keys: &mut Option<HashSet<InterpolateKey>>) {
        match self {
            ParsedValue::String(_) | ParsedValue::Subkeys(_) | ParsedValue::Default => {}
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
            ParsedValue::ForeignKey(foreign_key) => match &*foreign_key.borrow() {
                ForeignKey::Set(inner) => inner.get_keys_inner(keys),
                ForeignKey::NotSet(_) => unreachable!("dafuk?"),
            },
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

    pub fn new(value: &str, key_path: &KeyPath, locale: &Rc<Key>) -> Self {
        // look for component
        if let Some(component) = Self::find_component(value, key_path, locale) {
            return component;
        }
        // else look for variables
        if let Some(variable) = Self::find_variable(value, key_path, locale) {
            return variable;
        }

        // else it's just a string
        ParsedValue::String(value.to_string())
    }

    pub fn make_locale_value(&mut self) -> LocaleValue {
        if let ParsedValue::Subkeys(_) = self {
            let ParsedValue::Subkeys(mut locale) = core::mem::take(self) else {
                unreachable!();
            };
            LocaleValue::Subkeys {
                keys: locale.make_builder_keys(),
                locales: vec![locale],
            }
        } else {
            LocaleValue::Value(self.get_keys())
        }
    }

    fn merge_inner(
        &self,
        keys: &mut Option<HashSet<InterpolateKey>>,
        top_locale: Rc<Key>,
        key_path: &mut KeyPath,
    ) -> Result<()> {
        self.get_keys_inner(keys);
        let Some(keys) = keys else {
            return Ok(());
        };
        let mut iter = keys.iter();
        let Some(count_type) = iter.find_map(|key| match key {
            InterpolateKey::Count(plural_type) => Some(*plural_type),
            _ => None,
        }) else {
            return Ok(());
        };

        let other_type = iter.find_map(|key| match key {
            InterpolateKey::Count(plural_type) if *plural_type != count_type => Some(*plural_type),
            _ => None,
        });

        if let Some(other_type) = other_type {
            return Err(Error::PluralTypeMissmatch {
                locale: top_locale,
                key_path: std::mem::take(key_path),
                type1: count_type,
                type2: other_type,
            });
        }

        // if the set contains InterpolateKey::Count, remove variable keys with name "count"
        // ("var_count" with the rename)
        keys.retain(|key| !matches!(key, InterpolateKey::Variable(key) if key.name == "var_count"));

        Ok(())
    }

    pub fn merge(
        &mut self,
        keys: &mut LocaleValue,
        default_locale: &str,
        top_locale: Rc<Key>,
        key_path: &mut KeyPath,
    ) -> Result<()> {
        self.reduce();
        match (&self, keys) {
            // Default, do nothing
            (ParsedValue::Default, _) => Ok(()),
            // Both subkeys
            (ParsedValue::Subkeys(_), LocaleValue::Subkeys { locales, keys }) => {
                let ParsedValue::Subkeys(mut loc) = core::mem::take(self) else {
                    unreachable!();
                };
                loc.merge(keys, default_locale, top_locale, key_path)?;
                locales.push(loc);
                Ok(())
            }
            // Both value
            (
                ParsedValue::Bloc(_)
                | ParsedValue::Component { .. }
                | ParsedValue::Plural(_)
                | ParsedValue::String(_)
                | ParsedValue::Variable(_)
                | ParsedValue::ForeignKey(_),
                LocaleValue::Value(keys),
            ) => self.merge_inner(keys, top_locale, key_path),
            // not compatible
            _ => Err(Error::SubKeyMissmatch {
                locale: top_locale,
                key_path: std::mem::take(key_path),
            }),
        }
    }

    fn parse_key_path(path: &str) -> Option<KeyPath> {
        let (mut key_path, path) = if let Some((namespace, rest)) = path.split_once("::") {
            let namespace = Key::new(namespace)?;

            (KeyPath::new(Some(Rc::new(namespace))), rest)
        } else {
            (KeyPath::new(None), path)
        };

        for key in path.split('.') {
            let key = Key::new(key)?;
            key_path.push_key(Rc::new(key));
        }

        Some(key_path)
    }

    fn find_variable(value: &str, key_path: &KeyPath, locale: &Rc<Key>) -> Option<Self> {
        let (before, rest) = value.split_once("{{")?;
        let (ident, after) = rest.split_once("}}")?;

        let ident = ident.trim();

        let first_char = ident.chars().next()?;

        let before = Self::new(before, key_path, locale);
        let after = Self::new(after, key_path, locale);

        let this = match first_char {
            // foreign key
            '@' => {
                let path = ident.strip_prefix('@')?;
                let foreign_key_path = Self::parse_key_path(path)?;
                FOREIGN_KEYS.with(|foreign_keys| {
                    foreign_keys
                        .borrow_mut()
                        .insert((Rc::clone(locale), key_path.clone()))
                });
                ParsedValue::ForeignKey(RefCell::new(ForeignKey::NotSet(foreign_key_path)))
            }
            // variable key
            _ => {
                let ident = Key::new(&format!("var_{}", ident))?;
                ParsedValue::Variable(Rc::new(ident))
            }
        };

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

    fn find_component(value: &str, key_path: &KeyPath, locale: &Rc<Key>) -> Option<Self> {
        let (key, before, beetween, after) = Self::find_valid_component(value)?;

        let before = ParsedValue::new(before, key_path, locale);
        let beetween = ParsedValue::new(beetween, key_path, locale);
        let after = ParsedValue::new(after, key_path, locale);

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

    fn reduce_bloc(values: &mut Vec<ParsedValue>) -> Option<ParsedValue> {
        let mut iter = values.iter_mut();
        let Some(mut acc) = iter.next() else {
            return Some(ParsedValue::String(String::new()));
        };
        acc.reduce();

        for value in iter {
            value.reduce();
            match (&mut *acc, value) {
                (ParsedValue::String(acc_str), ParsedValue::String(s)) => {
                    acc_str.push_str(s.as_str());
                    s.clear();
                }
                (_, new_acc @ ParsedValue::String(_)) => {
                    acc = new_acc;
                }
                _ => continue,
            }
        }

        values.retain(|value| !matches!(value, ParsedValue::String(s) if s.is_empty()));

        match &mut **values {
            [] => Some(ParsedValue::String(String::new())),
            [one] => Some(std::mem::take(one)),
            _ => None,
        }
    }

    pub fn reduce(&mut self) {
        match self {
            ParsedValue::Variable(_) | ParsedValue::String(_) | ParsedValue::Default => {}
            ParsedValue::ForeignKey(foreign_key) => {
                let fk = foreign_key.get_mut();
                match fk {
                    ForeignKey::NotSet(_) => unreachable!(),
                    ForeignKey::Set(value) => {
                        value.reduce();
                        let value = std::mem::take(&mut **value);
                        *self = value;
                    }
                }
            }
            ParsedValue::Plural(plurals) => {
                let _: Result<_, ()> = plurals.try_for_each_value_mut(|value| {
                    value.reduce();
                    Ok(())
                });
            }
            ParsedValue::Component { inner, .. } => inner.reduce(),
            ParsedValue::Subkeys(subkeys) => {
                for value in subkeys.keys.values_mut() {
                    value.reduce();
                }
            }
            ParsedValue::Bloc(values) => {
                if let Some(value) = Self::reduce_bloc(values) {
                    *self = value;
                }
            }
        }
    }

    fn flatten(&self, tokens: &mut Vec<TokenStream>) {
        match self {
            ParsedValue::Subkeys(_) | ParsedValue::Default => {}
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
                let boxed_fn = quote!(leptos::ToChildren::to_children(#f));
                tokens.push(quote!(leptos::IntoView::into_view(core::clone::Clone::clone(&#key)(#boxed_fn))))
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.flatten(tokens)
                }
            }
            ParsedValue::ForeignKey(foreign_key) => match &*foreign_key.borrow() {
                ForeignKey::Set(inner) => inner.flatten(tokens),
                ForeignKey::NotSet(_) => unreachable!(),
            },
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

    #[cfg(feature = "debug_interpolations")]
    pub fn get_default(&self) -> TokenStream {
        match self {
            InterpolateKey::Variable(_) => {
                quote!(())
            }
            InterpolateKey::Count(plural_type) => match plural_type {
                PluralType::F32 | PluralType::F64 => quote!(|| 0.0),
                _ => quote!(|| 0),
            },
            InterpolateKey::Component(_) => {
                quote!(|_: leptos::ChildrenFn| core::default::Default::default())
            }
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
pub struct ParsedValueSeed<'a> {
    pub top_locale_name: &'a Rc<Key>,
    pub in_plural: bool,
    pub key_path: &'a KeyPath,
    pub key: &'a Rc<Key>,
}

impl<'de> serde::de::DeserializeSeed<'de> for ParsedValueSeed<'_> {
    type Value = ParsedValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for ParsedValueSeed<'_> {
    type Value = ParsedValue;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::new(v, self.key_path, self.top_locale_name))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.in_plural {
            return Err(serde::de::Error::custom(Error::PluralSubkeys));
        }

        let map_de = MapAccessDeserializer::new(map);

        let seed = LocaleSeed {
            name: Rc::clone(self.key),
            top_locale_name: Rc::clone(self.top_locale_name),
            key_path: self.key_path.to_owned(),
        };

        seed.deserialize(map_de).map(ParsedValue::Subkeys)
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::Default)
    }

    fn visit_seq<A>(mut self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // nested plurals are not allowed, the code technically supports it,
        // but it's pointless and probably nobody will ever needs it.
        if std::mem::replace(&mut self.in_plural, true) {
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

    fn new_parsed_value(value: &str) -> ParsedValue {
        let key_path = KeyPath::new(None);
        let locale = Rc::new(Key::new("locale_key").unwrap());

        ParsedValue::new(value, &key_path, &locale)
    }

    fn new_key(key: &str) -> Rc<Key> {
        Rc::new(Key::new(key).unwrap())
    }

    #[test]
    fn parse_normal_string() {
        let value = new_parsed_value("test");

        assert_eq!(value, ParsedValue::String("test".to_string()));
    }

    #[test]
    fn parse_variable() {
        let value = new_parsed_value("before {{ var }} after");

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
        let value = new_parsed_value("before <comp>inner</comp> after");

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
        let value = new_parsed_value(
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
        let value = new_parsed_value("<p>test<h3>this is a h3</h3>not closing p");

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
