use super::error::{Error, Result};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Key {
    pub name: String,
    pub ident: syn::Ident,
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Key {}

pub enum KeyKind<'a> {
    LocaleName,
    LocaleKey { locale_name: &'a str },
}

impl Key {
    pub fn new(name: &str, kind: KeyKind) -> Result<Self> {
        let name = name.trim();
        let Ok(ident) = syn::parse_str::<syn::Ident>(name) else {
            return Err(match kind {
                KeyKind::LocaleName => Error::InvalidLocaleName(name.to_string()),
                KeyKind::LocaleKey { locale_name } => Error::InvalidLocaleKey {
                    key: name.to_string(),
                    locale: locale_name.to_string(),
                },
            });
        };
        Ok(Key {
            name: name.to_string(),
            ident,
        })
    }

    pub fn try_new(name: &str) -> Option<Self> {
        let name = name.trim();
        let ident = syn::parse_str::<syn::Ident>(name).ok()?;
        Some(Key {
            name: name.to_string(),
            ident,
        })
    }
}

impl quote::ToTokens for Key {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeySeed<'a> {
    LocaleName,
    LocaleKey(&'a str),
}

impl<'a> KeySeed<'a> {
    pub fn to_key_kind(self) -> KeyKind<'a> {
        match self {
            KeySeed::LocaleName => KeyKind::LocaleName,
            KeySeed::LocaleKey(locale_name) => KeyKind::LocaleKey { locale_name },
        }
    }
}

impl<'a: 'de, 'de> serde::de::DeserializeSeed<'de> for KeySeed<'a> {
    type Value = Key;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'a, 'de> serde::de::Visitor<'de> for KeySeed<'a> {
    type Value = Key;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KeySeed::LocaleName => write!(
                formatter,
                "a string representing a locale that can be used as a valid rust identifier"
            ),
            KeySeed::LocaleKey(_) => write!(
                formatter,
                "a string representing a locale key that can be used as a valid rust identifier"
            ),
        }
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Key::new(v, self.to_key_kind()).map_err(E::custom)
    }
}

pub struct KeyVecSeed<'a>(pub KeySeed<'a>);

impl<'a: 'de, 'de> serde::de::DeserializeSeed<'de> for KeyVecSeed<'a> {
    type Value = Vec<Key>;
    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a: 'de, 'de> serde::de::Visitor<'de> for KeyVecSeed<'a> {
    type Value = Vec<Key>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an sequence of string")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut keys = Vec::new(); // json don't have size hints
        while let Some(value) = seq.next_element_seed(self.0)? {
            keys.push(value);
        }
        Ok(keys)
    }
}
