pub mod key;

use std::fmt::{Debug, Display};

pub use key::{Key, KeyPath};

use crate::{
    formatters::Formatters,
    parse_locales::{
        ForeignKeysPaths,
        error::{Diagnostics, Result},
        parsed_value::ParsedValue,
    },
};

pub type ParseFn = fn(&ParseContext, &str) -> Option<Result<ParsedValue>>;

#[derive(Clone, Copy)]
pub struct ParseContext<'a> {
    pub loc: Loc<'a>,
    pub foreign_keys_paths: &'a ForeignKeysPaths,
    pub formatters: &'a Formatters,
    pub diag: &'a Diagnostics,
    pub parse_fns: &'a [ParseFn],
}

#[derive(Clone, Copy)]
pub struct Loc<'a> {
    pub key_path: &'a KeyPath,
    pub locale: &'a Key,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub locale: Key,
    pub key_path: KeyPath,
}

impl Location {
    pub fn new(locale: Key, key_path: KeyPath) -> Location {
        Location { locale, key_path }
    }
}

impl From<&'_ ParseContext<'_>> for Location {
    fn from(ctx: &'_ ParseContext) -> Self {
        ctx.loc.into()
    }
}

impl From<ParseContext<'_>> for Location {
    fn from(ctx: ParseContext<'_>) -> Self {
        ctx.loc.into()
    }
}

impl From<&'_ Loc<'_>> for Location {
    fn from(loc: &'_ Loc) -> Self {
        Self::new(loc.locale.clone(), loc.key_path.clone())
    }
}

impl From<Loc<'_>> for Location {
    fn from(loc: Loc<'_>) -> Self {
        Self::new(loc.locale.clone(), loc.key_path.clone())
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Location { locale, key_path } = self;
        write!(f, "key \"{key_path}\" in locale {locale:?}")
    }
}

/// We should avoid to panic as much as possible, and return the Error enum instead,
/// but there is cases where unwrap *should* be good, like when accessing a value in a Map where the keys are already known
/// This trait serves as a easy unwrap where the code position can be given.
pub trait UnwrapAt {
    type Value;

    fn unwrap_at(self, location: &str) -> Self::Value;
}

impl<T> UnwrapAt for Option<T> {
    type Value = T;

    #[track_caller]
    fn unwrap_at(self, location: &str) -> Self::Value {
        let msg = format!(
            "Unexpected None value at {location}. If you got this error please open an issue on the leptos_i18n github repo."
        );
        self.expect(&msg)
    }
}

impl<T, E: Debug> UnwrapAt for Result<T, E> {
    type Value = T;

    #[track_caller]
    fn unwrap_at(self, location: &str) -> Self::Value {
        let msg = format!(
            "Unexpected Err value at {location}. If you got this error please open an issue on the leptos_i18n github repo."
        );
        self.expect(&msg)
    }
}
