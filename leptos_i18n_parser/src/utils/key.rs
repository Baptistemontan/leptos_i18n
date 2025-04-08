use crate::parse_locales::error::{Error, Result};
use crate::parse_locales::VAR_COUNT_KEY;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::rc::Rc;

use super::UnwrapAt;

#[derive(Clone)]
pub struct Key {
    pub name: Rc<str>,
    pub ident: Rc<syn::Ident>,
}

impl Key {
    pub fn new(name: &str) -> Option<Self> {
        let name = name.trim();
        let ident_repr = name.replace('-', "_");
        let ident = syn::parse_str::<syn::Ident>(&ident_repr).ok()?;
        Some(Key {
            name: Rc::from(name),
            ident: Rc::new(ident),
        })
    }

    pub fn try_new(name: &str) -> Result<Self> {
        Self::new(name).ok_or_else(|| Error::InvalidKey(name.to_string()).into())
    }

    pub fn from_ident(ident: syn::Ident) -> Self {
        let s = ident.to_string();
        Key {
            ident: Rc::new(ident),
            name: Rc::from(s.as_str()),
        }
    }

    pub fn count() -> Self {
        Self::new(VAR_COUNT_KEY).unwrap_at("VAR_COUNT_KEY")
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.name, f)
    }
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

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl quote::ToTokens for Key {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::ToTokens::to_tokens(&*self.ident, tokens);
    }
}

impl quote::IdentFragment for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        quote::IdentFragment::fmt(&*self.ident, f)
    }

    fn span(&self) -> Option<proc_macro2::Span> {
        quote::IdentFragment::span(&*self.ident)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyPath {
    pub namespace: Option<Key>,
    pub path: Vec<Key>,
}

impl Display for KeyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}::", namespace.name)?;
        }
        let mut iter = self.path.iter();
        if let Some(first) = iter.next() {
            f.write_str(&first.name)?;
            for key in iter {
                write!(f, ".{}", key.name)?;
            }
        }
        Ok(())
    }
}

impl KeyPath {
    pub const fn new(namespace: Option<Key>) -> Self {
        KeyPath {
            namespace,
            path: vec![],
        }
    }

    pub fn push_key(&mut self, key: Key) {
        self.path.push(key);
    }

    pub fn pop_key(&mut self) -> Option<Key> {
        self.path.pop()
    }

    pub fn to_string_with_key(&self, key: &Key) -> String {
        if self.namespace.is_none() && self.path.is_empty() {
            return key.name.to_string();
        }
        let mut s = self.to_string();
        if self.namespace.is_none() || !self.path.is_empty() {
            s.push('.');
        }
        s.push_str(&key.name);
        s
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

impl serde::de::Visitor<'_> for KeyVisitor {
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
