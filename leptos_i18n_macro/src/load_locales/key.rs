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

impl Key {
    pub fn new(name: &str) -> Option<Self> {
        let name = name.trim();
        let ident_repr = name.replace('-', "_");
        let ident = syn::parse_str::<syn::Ident>(&ident_repr).ok()?;
        Some(Key {
            name: name.to_string(),
            ident,
        })
    }

    pub fn try_new(name: &str) -> Result<Self> {
        Self::new(name).ok_or_else(|| Error::InvalidKey(name.to_string()))
    }
}

impl quote::ToTokens for Key {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens)
    }
}

struct KeyVisitor;

impl<'de> serde::de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor)
    }
}

impl<'de> serde::de::Visitor<'de> for KeyVisitor {
    type Value = Key;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string that can be used as a valid rust identifier"
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Key::try_new(v).map_err(E::custom)
    }
}
