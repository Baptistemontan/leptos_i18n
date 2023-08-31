use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{Bound, Not},
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::{
    error::{Error, Result},
    parsed_value::{InterpolateKey, ParsedValue, ParsedValueSeed},
    SeedBase,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum PluralType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl Default for PluralType {
    fn default() -> Self {
        Self::I64
    }
}

impl ToTokens for PluralType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let plural_type = match self {
            PluralType::I8 => quote!(i8),
            PluralType::I16 => quote!(i16),
            PluralType::I32 => quote!(i32),
            PluralType::I64 => quote!(i64),
            PluralType::U8 => quote!(u8),
            PluralType::U16 => quote!(u16),
            PluralType::U32 => quote!(u32),
            PluralType::U64 => quote!(u64),
            PluralType::F32 => quote!(f32),
            PluralType::F64 => quote!(f64),
        };
        tokens.extend(plural_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Plurals {
    I8(Vec<(Plural<i8>, ParsedValue)>),
    I16(Vec<(Plural<i16>, ParsedValue)>),
    I32(Vec<(Plural<i32>, ParsedValue)>),
    I64(Vec<(Plural<i64>, ParsedValue)>),
    U8(Vec<(Plural<u8>, ParsedValue)>),
    U16(Vec<(Plural<u16>, ParsedValue)>),
    U32(Vec<(Plural<u32>, ParsedValue)>),
    U64(Vec<(Plural<u64>, ParsedValue)>),
    F32(Vec<(Plural<f32>, ParsedValue)>),
    F64(Vec<(Plural<f64>, ParsedValue)>),
}

impl Plurals {
    pub fn get_keys_inner<'a>(&'a self, keys: &mut Option<HashSet<InterpolateKey<'a>>>) {
        match self {
            Plurals::I8(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::I16(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::I32(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::I64(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::U8(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::U16(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::U32(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::U64(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::F32(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
            Plurals::F64(v) => v.iter().for_each(|(_, value)| value.get_keys_inner(keys)),
        }
    }

    pub fn get_type(&self) -> PluralType {
        match self {
            Plurals::I8(_) => PluralType::I8,
            Plurals::I16(_) => PluralType::I16,
            Plurals::I32(_) => PluralType::I32,
            Plurals::I64(_) => PluralType::I64,
            Plurals::U8(_) => PluralType::U8,
            Plurals::U16(_) => PluralType::U16,
            Plurals::U32(_) => PluralType::U32,
            Plurals::U64(_) => PluralType::U64,
            Plurals::F32(_) => PluralType::F32,
            Plurals::F64(_) => PluralType::F64,
        }
    }

    fn to_tokens_integers<T: PluralInteger>(plurals: &[(Plural<T>, ParsedValue)]) -> TokenStream {
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
            match var_count() {
                #(
                    #match_arms,
                )*
            }
        };

        quote! {
            leptos::IntoView::into_view(
                {
                    #captured_values
                    move || #match_statement
                },
                cx
            )
        }
    }

    fn to_tokens_floats<T: PluralFloats>(plurals: &[(Plural<T>, ParsedValue)]) -> TokenStream {
        fn to_condition<T: PluralFloats>(plural: &Plural<T>) -> Option<TokenStream> {
            match plural {
                Plural::Exact(exact) => Some(quote!(plural_count == #exact)),
                Plural::Range { .. } => {
                    Some(quote!(core::ops::RangeBounds::contains(&(#plural), &plural_count)))
                }
                Plural::Multiple(conditions) => {
                    let mut conditions = conditions.iter().filter_map(to_condition);
                    let first = conditions.next();
                    Some(quote!(#first #(|| #conditions)*))
                }
                Plural::Fallback => None,
            }
        }

        let mut ifs = plurals
            .iter()
            .map(|(plural, value)| match to_condition(plural) {
                None => quote!({ #value }),
                Some(condition) => quote!(if #condition { #value }),
            });
        let first = ifs.next();
        let ifs = quote! {
            #first
            #(else #ifs)*
        };

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

        quote! {
            leptos::IntoView::into_view(
                {
                    #captured_values
                    move || {
                        let plural_count = var_count();
                        #ifs
                    }
                },
                cx
            )
        }
    }

    fn deserialize_all_pairs<'de, A, T>(
        mut seq: A,
        plurals: &mut Vec<(Plural<T>, ParsedValue)>,
        parsed_value_seed: ParsedValueSeed,
    ) -> Result<(), A::Error>
    where
        A: serde::de::SeqAccess<'de>,
        T: PluralNumber,
    {
        let plural_seed = PluralStructSeed::<T>(parsed_value_seed, PhantomData);
        while let Some(pair) = seq.next_element_seed(plural_seed)? {
            plurals.push(pair)
        }
        Ok(())
    }

    fn deserialize_inner<'de, A>(
        &mut self,
        seq: A,
        parsed_value_seed: ParsedValueSeed,
    ) -> Result<(), A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        match self {
            Plurals::I8(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::I16(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::I32(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::I64(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::U8(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::U16(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::U32(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::U64(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::F32(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
            Plurals::F64(plurals) => Self::deserialize_all_pairs(seq, plurals, parsed_value_seed),
        }
    }

    pub fn from_type(plural_type: PluralType) -> Self {
        match plural_type {
            PluralType::I8 => Self::I8(vec![]),
            PluralType::I16 => Self::I16(vec![]),
            PluralType::I32 => Self::I32(vec![]),
            PluralType::I64 => Self::I64(vec![]),
            PluralType::U8 => Self::U8(vec![]),
            PluralType::U16 => Self::U16(vec![]),
            PluralType::U32 => Self::U32(vec![]),
            PluralType::U64 => Self::U64(vec![]),
            PluralType::F32 => Self::F32(vec![]),
            PluralType::F64 => Self::F64(vec![]),
        }
    }

    pub fn from_serde_seq<'de, A>(
        mut seq: A,
        parsed_value_seed: ParsedValueSeed,
    ) -> Result<Self, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let SeedBase {
            locale_name,
            locale_key,
            namespace,
        } = parsed_value_seed.base;
        let type_or_plural = seq
            .next_element_seed(TypeOrPluralSeed(parsed_value_seed))?
            .ok_or_else(|| {
                let err = match namespace {
                    Some(namespace) => format!(
                    "in locale {:?} at namespace {:?} at key {:?}: empty plurals are not allowed",
                    locale_name, namespace, locale_key
                ),
                    None => format!(
                        "in locale {:?} at key {:?}: empty plurals are not allowed",
                        locale_name, locale_key
                    ),
                };
                serde::de::Error::custom(err)
            })?;

        let mut plurals = match type_or_plural {
            TypeOrPlural::Type(plural_type) => Self::from_type(plural_type),
            TypeOrPlural::Plural(plural) => Plurals::I64(vec![plural]),
        };

        plurals.deserialize_inner(seq, parsed_value_seed)?;
        Ok(plurals)
    }

    fn check_de_inner<T: PluralNumber>(
        plurals: &[(Plural<T>, ParsedValue)],
    ) -> (bool, usize, bool) {
        // easy to avoid compile warning, check if a fallback is not at the end position
        let invalid_fallback = plurals
            .iter()
            .rev()
            .skip(1)
            .any(|(plural, _)| match plural {
                Plural::Fallback => true,
                // "n | _" is kind of pointless but still supported, but still check if a fallback is put outside the end
                Plural::Multiple(multi) => multi
                    .iter()
                    .any(|plural| matches!(plural, Plural::Fallback)),
                _ => false,
            });
        // also check if multiple fallbacks exist
        let fallback_count = plurals
            .iter()
            .filter(|(plural, _)| matches!(plural, Plural::Fallback))
            .count();

        (invalid_fallback, fallback_count, T::should_have_fallback())
    }

    pub fn check_deserialization(&self) -> (bool, usize, bool) {
        match self {
            Plurals::I8(plurals) => Self::check_de_inner(plurals),
            Plurals::I16(plurals) => Self::check_de_inner(plurals),
            Plurals::I32(plurals) => Self::check_de_inner(plurals),
            Plurals::I64(plurals) => Self::check_de_inner(plurals),
            Plurals::U8(plurals) => Self::check_de_inner(plurals),
            Plurals::U16(plurals) => Self::check_de_inner(plurals),
            Plurals::U32(plurals) => Self::check_de_inner(plurals),
            Plurals::U64(plurals) => Self::check_de_inner(plurals),
            Plurals::F32(plurals) => Self::check_de_inner(plurals),
            Plurals::F64(plurals) => Self::check_de_inner(plurals),
        }
    }
}

impl ToTokens for Plurals {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Plurals::I8(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::I16(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::I32(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::I64(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::U8(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::U16(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::U32(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::U64(plurals) => Self::to_tokens_integers(plurals).to_tokens(tokens),
            Plurals::F32(plurals) => Self::to_tokens_floats(plurals).to_tokens(tokens),
            Plurals::F64(plurals) => Self::to_tokens_floats(plurals).to_tokens(tokens),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Plural<T> {
    Exact(T),
    Range { start: Option<T>, end: Bound<T> },
    Multiple(Vec<Self>),
    Fallback,
}

pub trait PluralNumber: FromStr + ToTokens + PartialOrd + Copy {
    fn range_end_bound(self) -> Option<Bound<Self>>;

    fn plural_type() -> PluralType;

    fn should_have_fallback() -> bool {
        matches!(Self::plural_type(), PluralType::F32 | PluralType::F64)
    }
}

pub trait PluralInteger: PluralNumber {}

pub trait PluralFloats: PluralNumber {}

impl<T: PluralNumber> Plural<T> {
    pub fn new(
        locale_name: &str,
        locale_key: &str,
        namespace: Option<&str>,
        s: &str,
    ) -> Result<Self> {
        let parse = |s: &str| {
            s.parse::<T>().map_err(|_| Error::InvalidPlural {
                locale_name: locale_name.to_string(),
                locale_key: locale_key.to_string(),
                namespace: namespace.map(str::to_string),
                plural: s.to_string(),
                plural_type: T::plural_type(),
            })
        };
        let s = s.trim();
        if matches!(s, "_" | "..") {
            return Ok(Self::Fallback);
        };

        if s.contains('|') {
            return s
                .split('|')
                .map(|s| Self::new(locale_name, locale_key, namespace, s))
                .collect::<Result<_>>()
                .map(Self::Multiple);
        }

        if let Some((start, end)) = s.split_once("..") {
            let start = start.trim();
            let start = start.is_empty().not().then(|| parse(start)).transpose()?;
            let end = end.trim();
            let end = if end.is_empty() {
                Bound::Unbounded
            } else if let Some(end) = end.strip_prefix('=').map(str::trim_start) {
                Bound::Included(parse(end)?)
            } else {
                let end = parse(end)?;

                end.range_end_bound()
                    .ok_or_else(|| Error::InvalidBoundEnd {
                        locale_name: locale_name.to_string(),
                        locale_key: locale_key.to_string(),
                        namespace: namespace.map(str::to_string),
                        range: s.to_string(),
                        plural_type: T::plural_type(),
                    })?
            };

            if let Some(start) = start {
                match end {
                    Bound::Excluded(end) if end <= start => {
                        return Err(Error::ImpossibleRange {
                            locale_name: locale_name.to_string(),
                            locale_key: locale_key.to_string(),
                            namespace: namespace.map(str::to_string),
                            range: s.to_string(),
                        })
                    }
                    Bound::Included(end) if end < start => {
                        return Err(Error::ImpossibleRange {
                            locale_name: locale_name.to_string(),
                            locale_key: locale_key.to_string(),
                            namespace: namespace.map(str::to_string),
                            range: s.to_string(),
                        })
                    }
                    _ => {}
                }
            }

            Ok(Self::Range { start, end })
        } else {
            parse(s).map(Self::Exact)
        }
    }
}

impl<T: PluralNumber> ToTokens for Plural<T> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            Plural::Exact(num) => quote!(#num),
            Plural::Range {
                start,
                end: Bound::Included(end),
            } => {
                quote!(#start..=#end)
            }
            Plural::Range {
                start,
                end: Bound::Unbounded,
            } => {
                quote!(#start..)
            }
            // unreachable normally but mal
            Plural::Range {
                start,
                end: Bound::Excluded(end),
            } => {
                quote!(#start..#end)
            }
            Plural::Fallback => quote!(_),
            Plural::Multiple(matchs) => {
                let mut matchs = matchs.iter().map(Self::to_token_stream);
                if let Some(first) = matchs.next() {
                    quote!(#first #(| #matchs)*)
                } else {
                    quote!()
                }
            }
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        Self::to_token_stream(self).to_tokens(tokens)
    }
}

#[derive(Debug, Clone, Copy)]
struct PluralSeed<'a, T>(pub SeedBase<'a>, PhantomData<T>);

impl<'de, T: PluralNumber> serde::de::DeserializeSeed<'de> for PluralSeed<'_, T> {
    type Value = Plural<T>;
    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, T: PluralNumber> serde::de::Visitor<'de> for PluralSeed<'_, T> {
    type Value = Plural<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string representing a plural or a sequence of string representing a plural"
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut plurals = vec![];

        while let Some(plural) = seq.next_element_seed(self)? {
            plurals.push(plural)
        }

        Ok(Plural::Multiple(plurals))
    }

    fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let SeedBase {
            locale_name,
            locale_key,
            namespace,
        } = self.0;
        Plural::new(locale_name, locale_key, namespace, s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy)]
struct PluralStructSeed<'a, T>(pub ParsedValueSeed<'a>, PhantomData<T>);

impl<'de, T: PluralNumber> serde::de::DeserializeSeed<'de> for PluralStructSeed<'_, T> {
    type Value = (Plural<T>, ParsedValue);
    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, T: PluralNumber> serde::de::Visitor<'de> for PluralStructSeed<'_, T> {
    type Value = (Plural<T>, ParsedValue);

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a struct representing a plural with the count and the value"
        )
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        fn deser_field<'de, A, S, T>(
            option: &mut Option<T>,
            map: &mut A,
            seed: S,
            field_name: &'static str,
        ) -> Result<(), A::Error>
        where
            A: serde::de::MapAccess<'de>,
            S: serde::de::DeserializeSeed<'de, Value = T>,
        {
            if option.replace(map.next_value_seed(seed)?).is_some() {
                Err(serde::de::Error::duplicate_field(field_name))
            } else {
                Ok(())
            }
        }
        fn unwrap_field<T, E>(field: Option<T>, field_name: &'static str) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            field.ok_or_else(|| serde::de::Error::missing_field(field_name))
        }
        let mut plural = None;
        let mut value = None;
        let seed_base = self.0.base;
        while let Some(field) = map.next_key_seed(PluralFieldSeed(seed_base))? {
            match field {
                PluralField::Plural => deser_field(
                    &mut plural,
                    &mut map,
                    PluralSeed(seed_base, PhantomData),
                    "count",
                )?,
                PluralField::Value => deser_field(&mut value, &mut map, self.0, "count")?,
            }
        }

        let plural = plural.unwrap_or(Plural::Fallback); // if no count, fallback
        let value = unwrap_field(value, "value")?;

        Ok((plural, value))
    }
}

enum PluralField {
    Plural,
    Value,
}

impl PluralField {
    pub const FIELDS: &'static [&'static str] = &["count", "value"];
}

struct PluralFieldSeed<'a>(pub SeedBase<'a>);

impl<'de> serde::de::DeserializeSeed<'de> for PluralFieldSeed<'_> {
    type Value = PluralField;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

impl<'de> serde::de::Visitor<'de> for PluralFieldSeed<'_> {
    type Value = PluralField;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "an identifier for fields {:?}",
            PluralField::FIELDS
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "count" => Ok(PluralField::Plural),
            "value" => Ok(PluralField::Value),
            _ => Err(serde::de::Error::unknown_field(v, PluralField::FIELDS)),
        }
    }
}

enum TypeOrPlural {
    Type(PluralType),
    Plural((Plural<i64>, ParsedValue)),
}

struct TypeOrPluralSeed<'a>(pub ParsedValueSeed<'a>);

impl<'de> serde::de::DeserializeSeed<'de> for TypeOrPluralSeed<'_> {
    type Value = TypeOrPlural;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for TypeOrPluralSeed<'_> {
    type Value = TypeOrPlural;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "either a string describing a numerical type or a plural"
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let SeedBase {
            locale_name,
            locale_key,
            namespace,
        } = self.0.base;

        match v.trim() {
            "i8" => Ok(TypeOrPlural::Type(PluralType::I8)),
            "i16" => Ok(TypeOrPlural::Type(PluralType::I16)),
            "i32" => Ok(TypeOrPlural::Type(PluralType::I32)),
            "i64" => Ok(TypeOrPlural::Type(PluralType::I64)),
            "u8" => Ok(TypeOrPlural::Type(PluralType::U8)),
            "u16" => Ok(TypeOrPlural::Type(PluralType::U16)),
            "u32" => Ok(TypeOrPlural::Type(PluralType::U32)),
            "u64" => Ok(TypeOrPlural::Type(PluralType::U64)),
            "f32" => Ok(TypeOrPlural::Type(PluralType::F32)),
            "f64" => Ok(TypeOrPlural::Type(PluralType::F64)),
            _ => Err(serde::de::Error::custom(match namespace {
                Some(namespace) => format!(
                    "in locale {:?} at namespace {:?} at key {:?}: {:?} is not a valid number type",
                    locale_name, namespace, locale_key, v
                ),
                None => format!(
                    "in locale {:?} at key {:?}: {:?} is not a valid number type",
                    locale_name, locale_key, v
                ),
            })),
        }
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let plural_seed = PluralStructSeed::<i64>(self.0, PhantomData);
        plural_seed.visit_map(map).map(TypeOrPlural::Plural)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact() {
        let plural = Plural::new("", "", None, "0").unwrap();

        assert_eq!(plural, Plural::Exact(0));
    }

    #[test]
    fn test_fallback() {
        let plural = Plural::<i32>::new("", "", None, "_").unwrap();

        assert_eq!(plural, Plural::Fallback);
    }

    #[test]
    fn test_range() {
        let plural = Plural::new("", "", None, "0..6").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: Some(0),
                end: Bound::Included(5)
            }
        );
    }

    #[test]
    fn test_range_unbounded_end() {
        let plural = Plural::new("", "", None, "0..").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: Some(0),
                end: Bound::Unbounded
            }
        );
    }

    #[test]
    fn test_range_included_end() {
        let plural = Plural::new("", "", None, "0..=6").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: Some(0),
                end: Bound::Included(6)
            }
        );
    }

    #[test]
    fn test_range_unbounded_start() {
        let plural = Plural::new("", "", None, "..=6").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: None,
                end: Bound::Included(6)
            }
        );
    }

    #[test]
    fn test_range_full() {
        let plural = Plural::<i32>::new("", "", None, "..").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: None,
                end: Bound::Unbounded
            }
        );
    }

    #[test]
    fn test_multiple() {
        let plural = Plural::<i32>::new("", "", None, "5 | 5..8 | 70..=80 | _").unwrap();

        assert_eq!(
            plural,
            Plural::Multiple(vec![
                Plural::Exact(5),
                Plural::Range {
                    start: Some(5),
                    end: Bound::Included(7)
                },
                Plural::Range {
                    start: Some(70),
                    end: Bound::Included(80)
                },
                Plural::Fallback
            ])
        );
    }
}

mod plural_number_impl {
    use super::{Bound, PluralFloats, PluralInteger, PluralNumber, PluralType};
    macro_rules! impl_num {
        ($(($num_type:ty, $plural_type:ident))*) => {
            $(
                impl PluralNumber for $num_type {
                    fn range_end_bound(self) -> Option<Bound<Self>> {
                        self.checked_sub(1).map(Bound::Included)
                    }

                    fn plural_type() -> PluralType {
                        PluralType::$plural_type
                    }
                }

                impl PluralInteger for $num_type {}
            )*
        };
    }

    macro_rules! impl_floats {
        ($(($num_type:ty, $plural_type:ident))*) => {
            $(
                impl PluralNumber for $num_type {
                    fn range_end_bound(self) -> Option<Bound<Self>> {
                        Some(Bound::Excluded(self))
                    }

                    fn plural_type() -> PluralType {
                        PluralType::$plural_type
                    }
                }

                impl PluralFloats for $num_type {}
            )*
        };
    }

    impl_num!((i8, I8)(i16, I16)(i32, I32)(i64, I64)(u8, U8)(u16, U16)(
        u32, U32
    )(u64, U64));

    impl_floats!((f32, F32)(f64, F64));
}
