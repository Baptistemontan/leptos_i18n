pub mod key;

use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

pub use key::{Key, KeyPath};

use crate::parser::options::LocaleName;

#[derive(Clone, Copy)]
pub struct Loc<'a> {
    pub key_path: &'a KeyPath,
    pub locale: &'a LocaleName,
}

pub struct LocMut<'a> {
    pub key_path: &'a mut KeyPath,
    pub locale: &'a LocaleName,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Location {
    pub locale: LocaleName,
    pub key_path: KeyPath,
}

impl Location {
    pub fn new(locale: LocaleName, key_path: KeyPath) -> Location {
        Location { locale, key_path }
    }

    pub fn push_key(&mut self, key: Key) -> LocationGuard<'_> {
        self.key_path.path.push(key);
        LocationGuard { loc: self }
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

impl<'a> From<&'a Location> for Loc<'a> {
    fn from(value: &'a Location) -> Self {
        Loc {
            key_path: &value.key_path,
            locale: &value.locale,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Location { locale, key_path } = self;
        write!(f, "{locale}/{key_path}")
    }
}

pub struct LocationGuard<'a> {
    loc: &'a mut Location,
}

impl Deref for LocationGuard<'_> {
    type Target = Location;
    fn deref(&self) -> &Self::Target {
        self.loc
    }
}

impl DerefMut for LocationGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.loc
    }
}

impl Drop for LocationGuard<'_> {
    fn drop(&mut self) {
        self.key_path.path.pop();
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
