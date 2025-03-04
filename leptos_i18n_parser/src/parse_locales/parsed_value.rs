use std::{cell::RefCell, collections::BTreeMap, fmt::Display};

use serde::{
    de::{value::MapAccessDeserializer, DeserializeSeed, Visitor},
    Deserialize,
};

use crate::utils::{formatter::Formatter, Key, KeyPath, UnwrapAt};

use super::{
    error::{Error, Result},
    locale::{
        DefaultTo, DefaultedLocales, InterpolOrLit, InterpolationKeys, LiteralType, Locale,
        LocaleSeed, LocaleValue, LocalesOrNamespaces, RangeOrPlural,
    },
    plurals::Plurals,
    ranges::Ranges,
    warning::Warnings,
    ForeignKeysPaths, StringIndexer,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    Default,
    ForeignKey(RefCell<ForeignKey>),
    Ranges(Ranges),
    Literal(Literal),
    Variable { key: Key, formatter: Formatter },
    Component { key: Key, inner: Box<Self> },
    Bloc(Vec<Self>),
    Subkeys(Option<Locale>),
    Plurals(Plurals),
}

impl Default for ParsedValue {
    fn default() -> Self {
        ParsedValue::Literal(Literal::String(String::new(), usize::MAX))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForeignKey {
    NotSet(KeyPath, BTreeMap<String, ParsedValue>),
    Set(Box<ParsedValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String, usize),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

macro_rules! nested_result_try {
    ($value:expr) => {
        match $value {
            Ok(v) => v,
            Err(err) => return Some(Err(err)),
        }
    };
}

impl Literal {
    pub fn index_strings(&mut self, strings: &mut StringIndexer) {
        if let Literal::String(s, index) = self {
            *index = strings.push_str(s);
        }
    }

    pub fn is_string(&self) -> Option<&str> {
        match self {
            Literal::String(s, _) => Some(s),
            _ => None,
        }
    }

    pub fn join(&mut self, other: &Self) {
        match self {
            Literal::String(s, _) => s.push_str(&other.to_string()),
            Literal::Signed(v) => {
                let s = format!("{}{}", v, other);
                *self = Literal::String(s, usize::MAX);
            }
            Literal::Unsigned(v) => {
                let s = format!("{}{}", v, other);
                *self = Literal::String(s, usize::MAX);
            }
            Literal::Float(v) => {
                let s = format!("{}{}", v, other);
                *self = Literal::String(s, usize::MAX);
            }
            Literal::Bool(v) => {
                let s = format!("{}{}", v, other);
                *self = Literal::String(s, usize::MAX);
            }
        }
    }

    pub fn get_type(&self) -> LiteralType {
        match self {
            Literal::String(_, _) => LiteralType::String,
            Literal::Signed(_) => LiteralType::Signed,
            Literal::Unsigned(_) => LiteralType::Unsigned,
            Literal::Float(_) => LiteralType::Float,
            Literal::Bool(_) => LiteralType::Bool,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(v, _) => Display::fmt(v, f),
            Literal::Signed(v) => Display::fmt(v, f),
            Literal::Unsigned(v) => Display::fmt(v, f),
            Literal::Float(v) => Display::fmt(v, f),
            Literal::Bool(v) => Display::fmt(v, f),
        }
    }
}

impl ParsedValue {
    pub fn new(
        value: &str,
        key_path: &KeyPath,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Result<Self> {
        let parsed_value = [
            Self::find_foreign_key,
            Self::find_component,
            Self::find_variable,
        ]
        .into_iter()
        .find_map(|f| f(value, key_path, locale, foreign_keys_paths));
        if let Some(parsed_value) = parsed_value {
            parsed_value
        } else {
            Ok(ParsedValue::Literal(Literal::String(
                value.to_string(),
                usize::MAX,
            )))
        }
    }

    fn parse_formatter_args(s: &str) -> (&str, Option<Vec<(&str, &str)>>) {
        let Some((name, rest)) = s.split_once('(') else {
            return (s.trim(), None);
        };
        let Some((args, rest)) = rest.rsplit_once(')') else {
            return (s.trim(), None);
        };

        // TODO: what should we do with it ?
        let _ = rest;

        let args = args
            .split(';')
            .filter_map(|s| s.split_once(':'))
            .map(|(a, b)| (a.trim(), b.trim()));

        (name.trim(), Some(args.collect()))
    }

    fn parse_formatter(s: &str, locale: &Key, key_path: &KeyPath) -> Result<Formatter> {
        let (name, args) = Self::parse_formatter_args(s);
        match Formatter::from_name_and_args(name, args.as_deref()) {
            Ok(Some(formatter)) => Ok(formatter),
            Ok(None) => Err(Error::UnknownFormatter {
                name: name.to_string(),
                locale: locale.clone(),
                key_path: key_path.clone(),
            }),
            Err(formatter) => Err(Error::DisabledFormatter {
                formatter,
                locale: locale.clone(),
                key_path: key_path.clone(),
            }),
        }
    }

    fn parse_key_path(path: &str) -> Option<KeyPath> {
        let (mut key_path, path) = if let Some((namespace, rest)) = path.split_once(':') {
            let namespace = Key::new(namespace)?;

            (KeyPath::new(Some(namespace)), rest)
        } else {
            (KeyPath::new(None), path)
        };

        for key in path.split('.') {
            let key = Key::new(key)?;
            key_path.push_key(key);
        }

        Some(key_path)
    }

    fn parse_foreign_key_args_inner(
        s: &str,
        key_path: &KeyPath,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Result<BTreeMap<String, ParsedValue>> {
        let args = match serde_json::from_str::<BTreeMap<String, Literal>>(s) {
            Ok(args) => args,
            Err(err) => {
                return Err(Error::InvalidForeignKeyArgs {
                    locale: locale.clone(),
                    key_path: key_path.clone(),
                    err,
                })
            }
        };
        let mut parsed_args = BTreeMap::new();

        for (key, arg) in args {
            let parsed_value = match arg {
                Literal::String(s, _) => Self::new(&s, key_path, locale, foreign_keys_paths)?,
                other => ParsedValue::Literal(other),
            };
            let key = format!("var_{}", key.trim());
            parsed_args.insert(key, parsed_value);
        }

        Ok(parsed_args)
    }

    fn parse_foreign_key_args<'a>(
        s: &'a str,
        key_path: &KeyPath,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Result<(BTreeMap<String, ParsedValue>, &'a str)> {
        let mut depth = 0usize;
        let mut index = 0usize;

        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = match depth.checked_sub(1) {
                        Some(v) => v,
                        None => {
                            return Err(Error::UnexpectedToken {
                                locale: locale.clone(),
                                key_path: key_path.clone(),
                                message: "malformed foreign key".to_string(),
                            })
                        }
                    };
                    if depth == 0 {
                        index = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let (before, after) = s.split_at(index + '}'.len_utf8());

        let Some(after) = after.trim_start().strip_prefix(')') else {
            return Err(Error::UnexpectedToken {
                locale: locale.clone(),
                key_path: key_path.clone(),
                message: "malformed foreign key".to_string(),
            });
        };

        let args =
            Self::parse_foreign_key_args_inner(before, key_path, locale, foreign_keys_paths)?;

        Ok((args, after))
    }

    fn find_foreign_key(
        value: &str,
        key_path: &KeyPath,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Option<Result<Self>> {
        let (before, rest) = value.split_once("$t(")?;
        let next_split = rest.find([',', ')'])?;
        let keypath = rest.get(..next_split)?;
        let sep = rest[next_split..].chars().next()?;
        let after = rest.get(next_split + sep.len_utf8()..)?;
        let target_key_path = Self::parse_key_path(keypath)?;

        let (args, after) = if sep == ',' {
            nested_result_try!(Self::parse_foreign_key_args(
                after,
                key_path,
                locale,
                foreign_keys_paths
            ))
        } else {
            (BTreeMap::new(), after)
        };

        let this = ParsedValue::ForeignKey(RefCell::new(ForeignKey::new(
            key_path.clone(),
            target_key_path,
            args,
            locale,
            foreign_keys_paths,
        )));
        let before = nested_result_try!(Self::new(before, key_path, locale, foreign_keys_paths));
        let after = nested_result_try!(Self::new(after, key_path, locale, foreign_keys_paths));

        Some(Ok(ParsedValue::Bloc(vec![before, this, after])))
    }

    fn find_variable(
        value: &str,
        key_path: &KeyPath,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Option<Result<Self>> {
        let (before, rest) = value.split_once("{{")?;
        let (ident, after) = rest.split_once("}}")?;

        let ident = ident.trim();

        let before = nested_result_try!(Self::new(before, key_path, locale, foreign_keys_paths));
        let after = nested_result_try!(Self::new(after, key_path, locale, foreign_keys_paths));

        let this = if let Some((ident, formatter)) = ident.split_once(',') {
            let formatter = nested_result_try!(Self::parse_formatter(formatter, locale, key_path));
            let key = Key::new(&format!("var_{}", ident.trim()))?;
            ParsedValue::Variable { key, formatter }
        } else {
            let key = Key::new(&format!("var_{}", ident))?;
            ParsedValue::Variable {
                key,
                formatter: Formatter::None,
            }
        };

        Some(Ok(ParsedValue::Bloc(vec![before, this, after])))
    }

    fn find_valid_component(value: &str) -> Option<(Key, &str, &str, &str)> {
        let mut skip_sum = 0;
        loop {
            let (before, key, after, skip) = Self::find_opening_tag(&value[skip_sum..])?;
            if let Some((key, beetween, after)) = Self::find_closing_tag(after, key) {
                let before_len = skip_sum + before.len();
                let before = &value[..before_len];
                break Some((key, before, beetween, after));
            } else {
                skip_sum += skip;
            }
        }
    }

    fn find_component(
        value: &str,
        key_path: &KeyPath,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Option<Result<Self>> {
        let (key, before, beetween, after) = Self::find_valid_component(value)?;

        let before = nested_result_try!(ParsedValue::new(
            before,
            key_path,
            locale,
            foreign_keys_paths
        ));
        let beetween = nested_result_try!(ParsedValue::new(
            beetween,
            key_path,
            locale,
            foreign_keys_paths
        ));
        let after = nested_result_try!(ParsedValue::new(
            after,
            key_path,
            locale,
            foreign_keys_paths
        ));

        let this = ParsedValue::Component {
            key,
            inner: beetween.into(),
        };

        Some(Ok(ParsedValue::Bloc(vec![before, this, after])))
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

    fn resolve_foreign_key_inner(
        foreign_key: &mut ForeignKey,
        values: &LocalesOrNamespaces,
        top_locale: &Key,
        default_locale: &Key,
        key_path: &KeyPath,
    ) -> Result<()> {
        let ForeignKey::NotSet(foreign_key_path, args) = &*foreign_key else {
            // already set, I don't know how we got here but whatever
            return Ok(());
        };

        let Some(value) = values.get_value_at(top_locale, foreign_key_path) else {
            return Err(Error::MissingForeignKey {
                foreign_key: foreign_key_path.to_owned(),
                locale: top_locale.clone(),
                key_path: key_path.to_owned(),
            });
        };

        if matches!(value, ParsedValue::Default) {
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

        // possibility that the foreign key must be resolved too
        value.resolve_foreign_key(values, top_locale, default_locale, foreign_key_path)?;

        // possibility that args must resolve too
        for arg in args.values() {
            arg.resolve_foreign_key(values, top_locale, default_locale, foreign_key_path)?;
        }

        let value = value.populate(args, foreign_key_path, top_locale, key_path)?;

        let _ = std::mem::replace(foreign_key, ForeignKey::Set(Box::new(value)));

        Ok(())
    }

    pub fn resolve_foreign_key(
        &self,
        values: &LocalesOrNamespaces,
        top_locale: &Key,
        default_locale: &Key,
        path: &KeyPath,
    ) -> Result<()> {
        match self {
            ParsedValue::Variable { .. } | ParsedValue::Literal(_) | ParsedValue::Default => Ok(()),
            ParsedValue::Subkeys(_) => Ok(()), // unreachable ?
            ParsedValue::Ranges(inner) => {
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
                        locale: top_locale.clone(),
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
            ParsedValue::Plurals(Plurals { forms, other, .. }) => {
                for value in forms.values() {
                    value.resolve_foreign_key(values, top_locale, default_locale, path)?;
                }
                other.resolve_foreign_key(values, top_locale, default_locale, path)
            }
        }
    }

    pub fn populate(
        &self,
        args: &BTreeMap<String, ParsedValue>,
        foreign_key: &KeyPath,
        locale: &Key,
        key_path: &KeyPath,
    ) -> Result<Self> {
        match self {
            ParsedValue::Default | ParsedValue::ForeignKey(_) | ParsedValue::Literal(_) => {
                Ok(self.clone())
            }
            ParsedValue::Variable { key, formatter } => match args.get(&*key.name) {
                Some(value) => Ok(value.clone()),
                None => Ok(ParsedValue::Variable {
                    key: key.clone(),
                    formatter: *formatter,
                }),
            },
            ParsedValue::Component { key, inner } => Ok(ParsedValue::Component {
                key: key.clone(),
                inner: Box::new(inner.populate(args, foreign_key, locale, key_path)?),
            }),
            ParsedValue::Bloc(bloc) => bloc
                .iter()
                .map(|value| value.populate(args, foreign_key, locale, key_path))
                .collect::<Result<_>>()
                .map(ParsedValue::Bloc),
            ParsedValue::Ranges(ranges) => ranges.populate(args, foreign_key, locale, key_path),
            ParsedValue::Plurals(plurals) => plurals.populate(args, foreign_key, locale, key_path),
            ParsedValue::Subkeys(_) => Err(Error::InvalidForeignKey {
                foreign_key: foreign_key.to_owned(),
                locale: locale.clone(),
                key_path: key_path.to_owned(),
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn merge(
        &mut self,
        keys: &mut LocaleValue,
        top_locale: Key,
        default_to: DefaultTo,
        key_path: &mut KeyPath,
        strings: &mut StringIndexer,
        warnings: &Warnings,
    ) -> Result<()> {
        self.reduce();
        match (&mut *self, &mut *keys) {
            (this @ ParsedValue::Default, LocaleValue::Subkeys { locales, keys }) => {
                let default_locale = locales.first().unwrap_at("merge_1");
                let dummy_keys = default_locale
                    .keys
                    .keys()
                    .cloned()
                    .map(|k| (k, ParsedValue::Default))
                    .collect();
                let mut dummy_local = Locale {
                    top_locale_name: top_locale.clone(),
                    name: default_locale.name.clone(),
                    keys: dummy_keys,
                    strings: vec![],
                    top_locale_string_count: 0,
                };
                *this = ParsedValue::Subkeys(None);

                dummy_local.merge(keys, top_locale, default_to, key_path, strings, warnings)?;
                locales.push(dummy_local);
                Ok(())
            }
            (ParsedValue::Default, LocaleValue::Value { defaults, .. }) => {
                defaults.push(top_locale, default_to.get_key());
                Ok(())
            }
            // Both subkeys
            (ParsedValue::Subkeys(loc), LocaleValue::Subkeys { locales, keys }) => {
                let Some(mut loc) = loc.take() else {
                    unreachable!("merge called twice on Subkeys. If you got this error please open a issue on github.");
                };
                loc.merge(keys, top_locale, default_to, key_path, strings, warnings)?;
                locales.push(loc);
                Ok(())
            }
            (
                ParsedValue::Literal(lit),
                LocaleValue::Value {
                    value: interpol_or_lit,
                    ..
                },
            ) => {
                lit.index_strings(strings);
                let other_lit_type = match interpol_or_lit {
                    InterpolOrLit::Interpol(_) => return Ok(()),
                    InterpolOrLit::Lit(lit_type) => *lit_type,
                };
                if lit.get_type() == other_lit_type {
                    Ok(())
                } else {
                    // make builder with 0 fields.
                    *interpol_or_lit = InterpolOrLit::Interpol(InterpolationKeys::default());
                    Ok(())
                }
            }
            (
                ParsedValue::Bloc(_)
                | ParsedValue::Component { .. }
                | ParsedValue::Ranges(_)
                | ParsedValue::Variable { .. }
                | ParsedValue::Plurals(_)
                | ParsedValue::ForeignKey(_),
                LocaleValue::Value {
                    value: interpol_or_lit,
                    ..
                },
            ) => {
                self.index_strings(strings);
                self.get_keys_inner(key_path, interpol_or_lit, false)
            }

            // not compatible
            _ => Err(Error::SubKeyMissmatch {
                locale: top_locale,
                key_path: std::mem::take(key_path),
            }),
        }
    }

    pub fn reduce(&mut self) {
        match self {
            ParsedValue::Literal(Literal::String(s, _)) if s.is_empty() => {
                // skip empty strings
            }
            ParsedValue::Variable { .. } | ParsedValue::Literal(_) | ParsedValue::Default => {}
            ParsedValue::ForeignKey(foreign_key) => {
                let value = foreign_key.get_mut().as_inner_mut("reduce");
                value.reduce();
                let value = std::mem::take(value);
                *self = value;
            }
            ParsedValue::Ranges(ranges) => {
                ranges
                    .try_for_each_value_mut::<_, core::convert::Infallible>(|value| {
                        value.reduce();
                        Ok(())
                    })
                    .unwrap_at("reduce_1");
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
                    [] => *self = ParsedValue::default(),
                    [one] => *self = std::mem::take(one),
                    _ => {}
                }
            }
            ParsedValue::Plurals(Plurals { forms, other, .. }) => {
                for value in forms.values_mut().chain(Some(&mut **other)) {
                    value.reduce();
                }
            }
        }
    }

    pub fn reduce_into(self, bloc: &mut Vec<Self>) {
        match self {
            ParsedValue::Default => {}    // default in a bloc ? skip
            ParsedValue::Subkeys(_) => {} // same for subkeys
            mut plurals_like @ (ParsedValue::Ranges(_) | ParsedValue::Plurals(_)) => {
                plurals_like.reduce();
                bloc.push(plurals_like);
            }
            ParsedValue::ForeignKey(foreign_key) => {
                foreign_key
                    .into_inner()
                    .into_inner("reduce_into")
                    .reduce_into(bloc);
            }
            ParsedValue::Literal(s) => {
                if s.is_string().is_some_and(str::is_empty) {
                    // skip empty strings
                } else if let Some(ParsedValue::Literal(last)) = bloc.last_mut() {
                    // if last in the bloc is a literal join them instead of 2 literal next to each others
                    last.join(&s);
                } else {
                    bloc.push(ParsedValue::Literal(s));
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

    pub fn make_locale_value(
        &mut self,
        default_locale: &Key,
        key_path: &mut KeyPath,
        strings: &mut StringIndexer,
    ) -> Result<LocaleValue> {
        match self {
            ParsedValue::Subkeys(locale) => {
                let Some(mut locale) = locale.take() else {
                    unreachable!("make_locale_value called twice on Subkeys. If you got this error please open a issue on github.")
                };
                let keys = locale.make_builder_keys(key_path, strings)?;
                Ok(LocaleValue::Subkeys {
                    keys,
                    locales: vec![locale],
                })
            }
            ParsedValue::Default => Err(Error::ExplicitDefaultInDefault(std::mem::take(key_path))),
            this => {
                this.index_strings(strings);
                this.get_keys(key_path).map(|value| LocaleValue::Value {
                    value,
                    defaults: DefaultedLocales::new(default_locale.clone()),
                })
            }
        }
    }

    pub fn get_keys_inner(
        &self,
        key_path: &mut KeyPath,
        keys: &mut InterpolOrLit,
        is_top: bool,
    ) -> Result<()> {
        match self {
            ParsedValue::Literal(lit_type) if is_top => {
                *keys = InterpolOrLit::Lit(lit_type.get_type());
            }
            ParsedValue::Literal(_) | ParsedValue::Subkeys(_) | ParsedValue::Default => {}
            ParsedValue::Variable { key, formatter } => {
                keys.get_interpol_keys_mut()
                    .push_var(key.clone(), *formatter);
            }
            ParsedValue::Component { key, inner } => {
                keys.get_interpol_keys_mut().push_comp(key.clone());
                inner.get_keys_inner(key_path, keys, false)?;
            }
            ParsedValue::Bloc(values) => {
                for value in values {
                    value.get_keys_inner(key_path, keys, false)?;
                }
            }
            ParsedValue::Ranges(ranges) => {
                ranges.get_keys_inner(key_path, keys)?;
                let range_type = ranges.get_type();
                keys.get_interpol_keys_mut().push_count(
                    key_path,
                    RangeOrPlural::Range(range_type),
                    ranges.count_key.clone(),
                )?;
            }
            ParsedValue::ForeignKey(foreign_key) => {
                foreign_key
                    .borrow()
                    .as_inner("get_keys_inner")
                    .get_keys_inner(key_path, keys, false)?;
            }
            ParsedValue::Plurals(Plurals {
                forms,
                other,
                count_key,
                ..
            }) => {
                keys.get_interpol_keys_mut().push_count(
                    key_path,
                    RangeOrPlural::Plural,
                    count_key.clone(),
                )?;
                for value in forms.values() {
                    value.get_keys_inner(key_path, keys, false)?;
                }
                other.get_keys_inner(key_path, keys, false)?;
            }
        }
        Ok(())
    }

    pub fn get_keys(&self, key_path: &mut KeyPath) -> Result<InterpolOrLit> {
        let mut keys = InterpolOrLit::Lit(LiteralType::String);

        self.get_keys_inner(key_path, &mut keys, true)?;
        Ok(keys)
    }

    pub fn index_strings(&mut self, strings: &mut StringIndexer) {
        match self {
            ParsedValue::Literal(lit) => {
                lit.index_strings(strings);
            }
            ParsedValue::Ranges(ranges) => ranges.index_strings(strings),
            ParsedValue::Component { inner, .. } => {
                inner.index_strings(strings);
            }
            ParsedValue::Plurals(plurals) => plurals.index_strings(strings),
            ParsedValue::Bloc(vec) => {
                for value in vec {
                    value.index_strings(strings);
                }
            }
            ParsedValue::Default
            | ParsedValue::ForeignKey(_)
            | ParsedValue::Variable { .. }
            | ParsedValue::Subkeys(_) => {}
        }
    }

    pub fn update_top_locale_name(&mut self, top_locale_name: &Key) {
        if let ParsedValue::Subkeys(Some(locale)) = self {
            locale.update_top_locale_name(top_locale_name);
        }
    }
}

impl ForeignKey {
    pub fn new(
        current_key_path: KeyPath,
        target_key_path: KeyPath,
        args: BTreeMap<String, ParsedValue>,
        locale: &Key,
        foreign_keys_paths: &ForeignKeysPaths,
    ) -> Self {
        foreign_keys_paths.push_path(locale.clone(), current_key_path);
        ForeignKey::NotSet(target_key_path, args)
    }

    pub fn into_inner(self, call_site: &str) -> ParsedValue {
        match self {
            ForeignKey::NotSet(_, _) => unreachable!("called {} on unresolved foreign key. If you got this error please open an issue on github (into_inner).", call_site),
            ForeignKey::Set(inner) => *inner,
        }
    }

    pub fn as_inner(&self, call_site: &str) -> &ParsedValue {
        match self {
            ForeignKey::NotSet(_, _) => unreachable!("called {} on unresolved foreign key. If you got this error please open an issue on github (as_inner).", call_site),
            ForeignKey::Set(inner) => inner,
        }
    }

    pub fn as_inner_mut(&mut self, call_site: &str) -> &mut ParsedValue {
        match self {
            ForeignKey::NotSet(_, _) => unreachable!("called {} on unresolved foreign key. If you got this error please open an issue on github (as_inner_mut).", call_site),
            ForeignKey::Set(inner) => inner,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedValueSeed<'a> {
    pub top_locale_name: &'a Key,
    pub in_range: bool,
    pub key_path: &'a KeyPath,
    pub key: &'a Key,
    pub foreign_keys_paths: &'a ForeignKeysPaths,
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
        ParsedValue::new(
            v,
            self.key_path,
            self.top_locale_name,
            self.foreign_keys_paths,
        )
        .map_err(|err| serde::de::Error::custom(err))
    }

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::Literal(Literal::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::Literal(Literal::Signed(v)))
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::Literal(Literal::Float(v)))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ParsedValue::Literal(Literal::Unsigned(v)))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.in_range {
            return Err(serde::de::Error::custom(Error::RangeSubkeys));
        }

        let map_de = MapAccessDeserializer::new(map);

        let seed = LocaleSeed {
            name: self.key.clone(),
            top_locale_name: self.top_locale_name.clone(),
            key_path: self.key_path.to_owned(),
            foreign_keys_paths: self.foreign_keys_paths,
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
        // nested ranges are not allowed, the code technically supports it,
        // but it's pointless and probably nobody will ever needs it.
        if std::mem::replace(&mut self.in_range, true) {
            return Err(serde::de::Error::custom(Error::NestedRanges));
        }
        let ranges = Ranges::from_serde_seq(map, self)?;

        let (invalid_fallback, fallback_count, should_have_fallback) =
            ranges.check_deserialization();

        if invalid_fallback {
            Err(serde::de::Error::custom(Error::InvalidFallback))
        } else if fallback_count > 1 {
            Err(serde::de::Error::custom(Error::MultipleFallbacks))
        } else if fallback_count == 0 && should_have_fallback {
            Err(serde::de::Error::custom(Error::MissingFallback(
                ranges.get_type(),
            )))
        } else {
            Ok(ParsedValue::Ranges(ranges))
        }
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "either a string, a sequence of ranges or a map of subkeys"
        )
    }
}

struct LiteralVisitor;

impl<'de> Deserialize<'de> for Literal {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(LiteralVisitor)
    }
}

impl Visitor<'_> for LiteralVisitor {
    type Value = Literal;

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Literal::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Literal::Signed(v))
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Literal::Float(v))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Literal::Unsigned(v))
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Literal::String(v, usize::MAX))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Literal::String(v.to_string(), usize::MAX))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a litteral such as a number, a string or a boolean"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_parsed_value(value: &str) -> ParsedValue {
        let key_path = KeyPath::new(None);
        let locale = new_key("locale_key");
        let foreign_keys_paths = ForeignKeysPaths::new();

        ParsedValue::new(value, &key_path, &locale, &foreign_keys_paths).unwrap()
    }

    fn new_key(key: &str) -> Key {
        Key::new(key).unwrap()
    }

    #[test]
    fn parse_normal_string() {
        let value = new_parsed_value("test");

        assert_eq!(
            value,
            ParsedValue::Literal(Literal::String("test".to_string(), usize::MAX))
        );
    }

    #[test]
    fn parse_variable() {
        let value = new_parsed_value("before {{ var }} after");

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::Literal(Literal::String("before ".to_string(), usize::MAX)),
                ParsedValue::Variable {
                    key: new_key("var_var"),
                    formatter: Formatter::None
                },
                ParsedValue::Literal(Literal::String(" after".to_string(), usize::MAX))
            ])
        )
    }

    #[test]
    fn parse_comp() {
        let value = new_parsed_value("before <comp>inner</comp> after");

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::Literal(Literal::String("before ".to_string(), usize::MAX)),
                ParsedValue::Component {
                    key: new_key("comp_comp"),
                    inner: Box::new(ParsedValue::Literal(Literal::String(
                        "inner".to_string(),
                        usize::MAX
                    )))
                },
                ParsedValue::Literal(Literal::String(" after".to_string(), usize::MAX))
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
                ParsedValue::Literal(Literal::String("before ".to_string(), usize::MAX)),
                ParsedValue::Component {
                    key: new_key("comp_comp"),
                    inner: Box::new(ParsedValue::Bloc(vec![
                        ParsedValue::Literal(Literal::String(
                            "inner before".to_string(),
                            usize::MAX
                        )),
                        ParsedValue::Component {
                            key: new_key("comp_comp"),
                            inner: Box::new(ParsedValue::Literal(Literal::String(
                                "inner inner".to_string(),
                                usize::MAX
                            )))
                        },
                        ParsedValue::Literal(Literal::String(
                            "inner after".to_string(),
                            usize::MAX
                        )),
                    ]))
                },
                ParsedValue::Literal(Literal::String(" after".to_string(), usize::MAX))
            ])
        )
    }

    #[test]
    fn parse_skipped_tag() {
        let value = new_parsed_value("<p>test<h3>this is a h3</h3>not closing p");

        assert_eq!(
            value,
            ParsedValue::Bloc(vec![
                ParsedValue::Literal(Literal::String("<p>test".to_string(), usize::MAX)),
                ParsedValue::Component {
                    key: new_key("comp_h3"),
                    inner: Box::new(ParsedValue::Literal(Literal::String(
                        "this is a h3".to_string(),
                        usize::MAX
                    )))
                },
                ParsedValue::Literal(Literal::String("not closing p".to_string(), usize::MAX))
            ])
        )
    }
}
