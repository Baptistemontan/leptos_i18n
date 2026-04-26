use std::{
    fmt::{self, Debug},
    hash::Hash,
    str::FromStr,
};

use crate::locale_traits::{BaseLocale, Locale};

/// Represent a scope in a locale.
pub trait Scope: 'static + Send + Sync {
    type BaseLocale: BaseLocale;

    /// The keys of the scopes
    type Keys: Keys<BaseLocale = Self::BaseLocale>;

    fn get_keys() -> Self::Keys {
        Self::Keys::THIS
    }
}

pub trait Keys: 'static + Send + Sync + Copy {
    type BaseLocale: BaseLocale;
    const THIS: Self;
}

impl<K: Keys> Scope for K {
    type BaseLocale = K::BaseLocale;
    type Keys = K;
}

/// A struct representing a scoped locale
pub struct ScopedLocale<S: Scope> {
    /// Base locale
    pub locale: S::BaseLocale,
}

impl<S: Scope> ScopedLocale<S> {
    /// Create a new `ScopedLocale` with the given base locale
    pub const fn new(locale: S::BaseLocale) -> Self {
        ScopedLocale { locale }
    }

    pub const fn locale(self) -> S::BaseLocale {
        self.locale
    }
}

impl<S: Scope> Debug for ScopedLocale<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <S::BaseLocale as Debug>::fmt(&self.locale, f)
    }
}

impl<S: Scope> Default for ScopedLocale<S> {
    fn default() -> Self {
        ScopedLocale {
            locale: Default::default(),
        }
    }
}

impl<S: Scope> PartialEq for ScopedLocale<S> {
    fn eq(&self, other: &Self) -> bool {
        self.locale == other.locale
    }
}

impl<S: Scope> Eq for ScopedLocale<S> {}

impl<S: Scope> PartialOrd for ScopedLocale<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Scope> Ord for ScopedLocale<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.locale.cmp(&other.locale)
    }
}

impl<S: Scope> Clone for ScopedLocale<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Scope> Copy for ScopedLocale<S> {}

impl<S: Scope> fmt::Display for ScopedLocale<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <S::BaseLocale as fmt::Display>::fmt(&self.locale, f)
    }
}

impl<T: ?Sized, S: Scope> AsRef<T> for ScopedLocale<S>
where
    S::BaseLocale: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.locale.as_ref()
    }
}

impl<S: Scope> Hash for ScopedLocale<S> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        Hash::hash(&self.locale, state)
    }
}

impl<S: Scope> FromStr for ScopedLocale<S> {
    type Err = <S::BaseLocale as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let locale = <S::BaseLocale as FromStr>::from_str(s)?;
        Ok(ScopedLocale { locale })
    }
}

impl<S: Scope> Scope for ScopedLocale<S> {
    type BaseLocale = S::BaseLocale;
    type Keys = S::Keys;
}

impl<S: Scope> Locale for ScopedLocale<S> {
    fn to_base_locale(self) -> Self::BaseLocale {
        self.locale
    }

    fn from_base_locale(locale: Self::BaseLocale) -> Self {
        Self::new(locale)
    }
}

impl<Sc: Scope> serde::Serialize for ScopedLocale<Sc> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.to_base_locale(), serializer)
    }
}

impl<'de, S: Scope> serde::Deserialize<'de> for ScopedLocale<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let base_locale: S::BaseLocale = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_base_locale(base_locale))
    }
}
