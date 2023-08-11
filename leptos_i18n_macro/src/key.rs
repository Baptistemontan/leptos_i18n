use crate::error::{Error, Result};
use std::hash::Hash;

pub struct Key<'a> {
    pub name: &'a str,
    pub ident: syn::Ident,
}

impl<'a> Hash for Key<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> Eq for Key<'a> {}

pub enum KeyKind<'a> {
    LocaleName,
    LocaleKey { locale_name: &'a str },
}

impl<'a> Key<'a> {
    pub fn new(name: &'a str, kind: KeyKind) -> Result<Self> {
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
        Ok(Key { name, ident })
    }

    pub fn try_new(name: &'a str) -> Option<Self> {
        let name = name.trim();
        let ident = syn::parse_str::<syn::Ident>(name).ok()?;
        Some(Key { name, ident })
    }
}

impl<'a> quote::ToTokens for Key<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens)
    }
}
