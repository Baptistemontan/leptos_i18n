use super::error::{Error, Result};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};

#[derive(Clone)]
pub struct Key {
    pub name: String,
    pub ident: syn::Ident,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyPath {
    namespace: Option<Rc<Key>>,
    path: Vec<Rc<Key>>,
}

impl KeyPath {
    pub fn new(namespace: Option<Rc<Key>>) -> Self {
        KeyPath {
            namespace,
            path: vec![],
        }
    }

    pub fn push_key(&mut self, key: Rc<Key>) {
        self.path.push(key);
    }

    pub fn pop_key(&mut self) {
        self.path.pop();
    }
}

impl Display for KeyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\"")?;
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
        f.write_str("\"")
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
