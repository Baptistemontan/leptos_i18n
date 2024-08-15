use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::utils::formatter::Formatter;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::de::{value::MapAccessDeserializer, DeserializeSeed};

use super::{
    error::{Error, Result},
    interpolate::CACHED_LOCALE_FIELD_KEY,
    locale::{Locale, LocaleSeed, LocaleValue, LocalesOrNamespaces},
    plural::{PluralType, Plurals},
};

use crate::utils::key::{Key, KeyPath, CACHED_PLURAL_COUNT_KEY};

thread_local! {
    pub static FOREIGN_KEYS: RefCell<HashSet<(Rc<Key>, KeyPath)>> = RefCell::new(HashSet::new());
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForeignKey {
    NotSet(KeyPath, HashMap<String, String>),
    Set(Box<ParsedValue>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ParsedValue {
    #[default]
    Default,
    ForeignKey(RefCell<ForeignKey>),
    Plural(Plurals),
    String(String),
    Variable {
        key: Rc<Key>,
        formatter: Formatter,
    },
    Component {
        key: Rc<Key>,
        inner: Box<Self>,
    },
    Bloc(Vec<Self>),
    Subkeys(Option<Locale>),
}

#[derive(Debug, Default)]
pub struct VarInfo {
    pub formatters: HashSet<Formatter>,
    pub plural_count: Option<PluralType>,
}

#[derive(Debug, Default)]
pub struct InterpolationKeys {
    components: HashSet<Rc<Key>>,
    variables: HashMap<Rc<Key>, VarInfo>,
}

impl InterpolationKeys {
    pub fn push_var(&mut self, key: Rc<Key>, formatter: Formatter) {
        let var_infos = self.variables.entry(key).or_default();
        var_infos.formatters.insert(formatter);
    }

    pub fn push_comp(&mut self, key: Rc<Key>) {
        self.components.insert(key);
    }

    pub fn push_count(&mut self, key_path: &mut KeyPath, ty: PluralType) -> Result<()> {
        let key = CACHED_PLURAL_COUNT_KEY.with(Clone::clone);
        let var_infos = self.variables.entry(key).or_default();
        match var_infos.plural_count.replace(ty) {
            Some(old) if old != ty => Err(Error::PluralTypeMissmatch {
                key_path: std::mem::take(key_path),
                type1: old,
                type2: ty,
            }),
            _ => Ok(()),
        }
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Key> {
        let comps = self.components.iter().map(|k| &**k);
        let vars = self.variables.keys().map(|k| &**k);
        comps.chain(vars)
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = (Rc<Key>, &VarInfo)> {
        self.variables
            .iter()
            .map(|(key, value)| (key.clone(), value))
    }

    pub fn iter_comps(&self) -> impl Iterator<Item = Rc<Key>> + '_ {
        self.components.iter().cloned()
    }
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
        key_path: &KeyPath,
    ) -> Result<()> {
        let ForeignKey::NotSet(foreign_key_path, args) = &*foreign_key else {
            // already set, I don't know how we got here but whatever
            return Ok(());
        };

        let Some(value) = values.get_value_at(top_locale, foreign_key_path) else {
            return Err(Error::MissingForeignKey {
                foreign_key: foreign_key_path.to_owned(),
                locale: Rc::clone(top_locale),
                key_path: key_path.to_owned(),
            });
        };

        match value {
            ParsedValue::Default => {
                // this check is normally done in a later step for optimisations (Locale::make_builder_keys),
                // but we still need to do it here to avoid infinite loop
                // this case happen if a foreign key point to an explicit default in the default locale
                // pretty niche, but would cause a rustc stack overflow if not done.
                if top_locale == default_locale {
                    return Err(Error::ExplicitDefaultInDefault(key_path.to_owned()));
                } else {
                    return Self::resolve_foreign_key_inner(
                        foreign_key,
                        values,
                        default_locale,
                        default_locale,
                        key_path,
                    );
                }
            }
            ParsedValue::Plural(_) => {
                return Err(Error::Custom(format!(
                    "foreign key to plurals is not supported yet, at key {} in locale {:?}",
                    key_path, top_locale
                )))
            }
            _ => {}
        }

        // possibility that the foreign key must be resolved too
        value.resolve_foreign_key(values, top_locale, default_locale, foreign_key_path)?;

        let value = value.populate(args, foreign_key_path, top_locale, key_path)?;

        let _ = std::mem::replace(foreign_key, ForeignKey::Set(Box::new(value)));

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
            ParsedValue::Variable { .. } | ParsedValue::String(_) | ParsedValue::Default => Ok(()),
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

    pub fn populate(
        &self,
        args: &HashMap<String, String>,
        foreign_key: &KeyPath,
        locale: &Rc<Key>,
        key_path: &KeyPath,
    ) -> Result<Self> {
        match self {
            ParsedValue::Default | ParsedValue::ForeignKey(_) | ParsedValue::String(_) => {
                Ok(self.clone())
            }
            ParsedValue::Variable { key, formatter } => match args.get(&key.name) {
                Some(value) => Ok(ParsedValue::String(value.to_owned())),
                None => Ok(ParsedValue::Variable {
                    key: Rc::clone(key),
                    formatter: *formatter,
                }),
            },
            ParsedValue::Component { key, inner } => Ok(ParsedValue::Component {
                key: Rc::clone(key),
                inner: Box::new(inner.populate(args, foreign_key, locale, key_path)?),
            }),
            ParsedValue::Bloc(bloc) => bloc
                .iter()
                .map(|value| value.populate(args, foreign_key, locale, key_path))
                .collect::<Result<_>>()
                .map(ParsedValue::Bloc),
            ParsedValue::Subkeys(_) | ParsedValue::Plural(_) => Err(Error::InvalidForeignKey {
                foreign_key: foreign_key.to_owned(),
                locale: Rc::clone(locale),
                key_path: key_path.to_owned(),
            }),
        }
    }

    pub fn get_keys_inner(
        &self,
        key_path: &mut KeyPath,
        keys: &mut Option<InterpolationKeys>,
    ) -> Result<()> {
        match self {
            ParsedValue::String(_) | ParsedValue::Subkeys(_) | ParsedValue::Default => {}
            ParsedValue::Variable { key, formatter } => {
                keys.get_or_insert_with(Default::default)
                    .push_var(key.clone(), *formatter);
            }
            ParsedValue::Component { key, inner } => {
                keys.get_or_insert_with(Default::default)
                    .push_comp(key.clone());
                inner.get_keys_inner(key_path, keys)?;
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.get_keys_inner(key_path, keys)?;
                }
            }
            ParsedValue::Plural(plurals) => {
                plurals.get_keys_inner(key_path, keys)?;
                let plural_type = plurals.get_type();
                keys.get_or_insert_with(Default::default)
                    .push_count(key_path, plural_type)?;
            }
            ParsedValue::ForeignKey(foreign_key) => {
                foreign_key
                    .borrow()
                    .as_inner("get_keys_inner")
                    .get_keys_inner(key_path, keys)?;
            }
        }
        Ok(())
    }

    pub fn get_keys(&self, key_path: &mut KeyPath) -> Result<Option<InterpolationKeys>> {
        let mut keys = None;
        self.get_keys_inner(key_path, &mut keys)?;
        Ok(keys)
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

    pub fn make_locale_value(&mut self, key_path: &mut KeyPath) -> Result<LocaleValue> {
        match self {
            ParsedValue::Subkeys(locale) => {
                let Some(mut locale) = locale.take() else {
                    unreachable!("make_locale_value called twice on Subkeys. If you got this error please open a issue on github.")
                };
                let keys = locale.make_builder_keys(key_path)?;
                Ok(LocaleValue::Subkeys {
                    keys,
                    locales: vec![locale],
                })
            }
            ParsedValue::Default => Err(Error::ExplicitDefaultInDefault(std::mem::take(key_path))),
            this => this.get_keys(key_path).map(LocaleValue::Value),
        }
    }

    pub fn merge(
        &mut self,
        keys: &mut LocaleValue,
        default_locale: &str,
        top_locale: Rc<Key>,
        key_path: &mut KeyPath,
    ) -> Result<()> {
        self.reduce();
        match (&mut *self, keys) {
            // Default, do nothing
            (ParsedValue::Default, _) => Ok(()),
            // Both subkeys
            (ParsedValue::Subkeys(loc), LocaleValue::Subkeys { locales, keys }) => {
                let Some(mut loc) = loc.take() else {
                    unreachable!("merge called twice on Subkeys. If you got this error please open a issue on github.");
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
                | ParsedValue::Variable { .. }
                | ParsedValue::ForeignKey(_),
                LocaleValue::Value(keys),
            ) => self.get_keys_inner(key_path, keys),
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

    fn parse_foreign_key(ident: &str, locale: &Rc<Key>, key_path: &KeyPath) -> Option<Self> {
        let ident = ident.strip_prefix('@')?;
        let mut splitted = ident.split(',');
        let path = splitted.next()?;

        let foreign_key_path = Self::parse_key_path(path)?;
        FOREIGN_KEYS.with(|foreign_keys| {
            foreign_keys
                .borrow_mut()
                .insert((Rc::clone(locale), key_path.clone()))
        });

        let mut args = HashMap::new();
        const QUOTES: &[char] = &['"', '\''];

        for arg in splitted {
            let (ident, value) = arg.split_once('=')?;
            let mut key = String::from("var_");
            key.push_str(ident.trim());

            let value = value.trim().strip_prefix(QUOTES)?;
            let value = value.strip_suffix(QUOTES)?;
            args.insert(key, value.to_owned());
        }

        Some(ParsedValue::ForeignKey(RefCell::new(ForeignKey::NotSet(
            foreign_key_path,
            args,
        ))))
    }

    fn parse_formatter_args(s: &str) -> (&str, Option<Vec<(&str, &str)>>) {
        let Some((name, rest)) = s.split_once('(') else {
            return (s.trim(), None);
        };
        let Some((args, rest)) = rest.rsplit_once(')') else {
            return (s.trim(), None);
        };
        if !rest.trim().is_empty() {
            todo!()
        }
        let args = args
            .split(';')
            .filter_map(|s| s.split_once(':'))
            .map(|(a, b)| (a.trim(), b.trim()));

        (name.trim(), Some(args.collect()))
    }

    fn parse_formatter(s: &str) -> Formatter {
        let (name, args) = Self::parse_formatter_args(s);
        Formatter::from_name_and_args(name, args.as_deref())
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
            '@' => Self::parse_foreign_key(ident, locale, key_path)?,
            // variable key
            _ => {
                if let Some((ident, formatter)) = ident.split_once(',') {
                    let formatter = Self::parse_formatter(formatter);
                    let key = Rc::new(Key::new(&format!("var_{}", ident))?);
                    ParsedValue::Variable { key, formatter }
                } else {
                    let key = Rc::new(Key::new(&format!("var_{}", ident))?);
                    ParsedValue::Variable {
                        key,
                        formatter: Formatter::None,
                    }
                }
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

    pub fn reduce(&mut self) {
        match self {
            ParsedValue::Variable { .. } | ParsedValue::String(_) | ParsedValue::Default => {}
            ParsedValue::ForeignKey(foreign_key) => {
                let value = foreign_key.get_mut().as_inner_mut("reduce");
                value.reduce();
                let value = std::mem::take(value);
                *self = value;
            }
            ParsedValue::Plural(plurals) => {
                let _: Result<_, ()> = plurals.try_for_each_value_mut(|value| {
                    value.reduce();
                    Ok(())
                });
            }
            ParsedValue::Component { inner, .. } => inner.reduce(),
            ParsedValue::Subkeys(Some(subkeys)) => {
                for value in subkeys.keys.values_mut() {
                    value.reduce();
                }
            }
            ParsedValue::Subkeys(None) => {
                unreachable!("called reduce on empty subkeys. If you got this error please open an issue on github.")
            }
            ParsedValue::Bloc(values) => {
                for value in std::mem::take(values) {
                    value.reduce_into(values);
                }

                match values.as_mut_slice() {
                    [] => *self = ParsedValue::String(String::new()),
                    [one] => *self = std::mem::take(one),
                    _ => {}
                }
            }
        }
    }

    pub fn reduce_into(self, bloc: &mut Vec<Self>) {
        match self {
            ParsedValue::Default => {}    // default in a bloc ? skip
            ParsedValue::Plural(_) => {}  // same for plural, can't be in a bloc
            ParsedValue::Subkeys(_) => {} // same for subkeys
            ParsedValue::ForeignKey(foreign_key) => {
                foreign_key
                    .into_inner()
                    .into_inner("reduce_into")
                    .reduce_into(bloc);
            }
            ParsedValue::String(s) => {
                if s.is_empty() {
                    // skip empty strings
                } else if let Some(ParsedValue::String(last)) = bloc.last_mut() {
                    // if last in the bloc is a string push into it instead of 2 strings next to each others
                    last.push_str(&s);
                } else {
                    bloc.push(ParsedValue::String(s));
                }
            }
            ParsedValue::Variable { key, formatter } => {
                bloc.push(ParsedValue::Variable { key, formatter })
            }
            ParsedValue::Component { key, mut inner } => {
                inner.reduce();
                bloc.push(ParsedValue::Component { key, inner });
            }
            ParsedValue::Bloc(inner) => {
                for value in inner {
                    value.reduce_into(bloc);
                }
            }
        }
    }

    fn flatten(&self, tokens: &mut Vec<TokenStream>, locale_field: &Key) {
        match self {
            ParsedValue::Subkeys(_) | ParsedValue::Default => {}
            ParsedValue::String(s) if s.is_empty() => {}
            ParsedValue::String(s) => tokens.push(quote!(leptos::IntoView::into_view(#s))),
            ParsedValue::Plural(plurals) => tokens.push(plurals.to_token_stream()),
            ParsedValue::Variable { key, formatter } => {
                let ts = formatter.var_into_view(key, locale_field);
                tokens.push(ts);
            }
            ParsedValue::Component { key, inner } => {
                let mut key_path = KeyPath::new(None);
                let captured_keys = inner.get_keys(&mut key_path).unwrap().map(|keys| {
                    let keys = keys
                        .iter_keys()
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
                    value.flatten(tokens, locale_field)
                }
            }
            ParsedValue::ForeignKey(foreign_key) => foreign_key
                .borrow()
                .as_inner("flatten")
                .flatten(tokens, locale_field),
        }
    }

    fn flatten_string(&self, tokens: &mut Vec<TokenStream>, locale_field: &Key) {
        match self {
            ParsedValue::Subkeys(_) | ParsedValue::Default => {}
            ParsedValue::String(s) if s.is_empty() => {}
            ParsedValue::String(s) => tokens.push(quote!(core::fmt::Display::fmt(#s, __formatter))),
            ParsedValue::Plural(plurals) => tokens.push(plurals.as_string_impl()),
            ParsedValue::Variable { key, formatter } => {
                let ts = formatter.var_fmt(key, locale_field);
                tokens.push(ts);
            }
            ParsedValue::Component { key, inner } => {
                let inner = inner.as_string_impl();
                tokens.push(quote!(l_i18n_crate::display::DisplayComponent::fmt(#key, __formatter, |__formatter| #inner)))
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.flatten_string(tokens, locale_field)
                }
            }
            ParsedValue::ForeignKey(foreign_key) => foreign_key
                .borrow()
                .as_inner("flatten_string")
                .flatten_string(tokens, locale_field),
        }
    }

    pub fn as_string_impl(&self) -> TokenStream {
        let mut tokens = Vec::new();
        let locale_field = CACHED_LOCALE_FIELD_KEY.with(Clone::clone);
        self.flatten_string(&mut tokens, &locale_field);

        match &tokens[..] {
            [] => quote!(Ok(())),
            [value] => value.clone(),
            values => quote!({ #(#values?;)* Ok(()) }),
        }
    }
}

impl ForeignKey {
    pub fn into_inner(self, call_site: &str) -> ParsedValue {
        match self {
            ForeignKey::NotSet(_, _) => unreachable!("called {} on unresolved foreign key. If you got this error please open an issue on github.", call_site),
            ForeignKey::Set(inner) => *inner,
        }
    }

    pub fn as_inner(&self, call_site: &str) -> &ParsedValue {
        match self {
            ForeignKey::NotSet(_, _) => unreachable!("called {} on unresolved foreign key. If you got this error please open an issue on github.", call_site),
            ForeignKey::Set(inner) => inner,
        }
    }

    pub fn as_inner_mut(&mut self, call_site: &str) -> &mut ParsedValue {
        match self {
            ForeignKey::NotSet(_, _) => unreachable!("called {} on unresolved foreign key. If you got this error please open an issue on github.", call_site),
            ForeignKey::Set(inner) => inner,
        }
    }
}

impl ToTokens for ParsedValue {
    fn to_token_stream(&self) -> TokenStream {
        let mut tokens = Vec::new();
        let locale_field = CACHED_LOCALE_FIELD_KEY.with(Clone::clone);
        self.flatten(&mut tokens, &locale_field);

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

        seed.deserialize(map_de).map(Some).map(ParsedValue::Subkeys)
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
                ParsedValue::Variable {
                    key: new_key("var_var"),
                    formatter: Formatter::None
                },
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
