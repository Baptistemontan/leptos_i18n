use std::{
    fmt::{self, Debug},
    hash::Hash,
    marker::PhantomData,
    str::FromStr,
};

use icu_locale::{LanguageIdentifier, Locale as IcuLocale};

use crate::{Direction, Locale, LocaleKeys};

/// Represent a scope in a locale.
pub trait Scope<L: Locale>: 'static + Send + Sync {
    /// The keys of the scopes
    type Keys: LocaleKeys<Locale = L>;
}

impl<K: LocaleKeys> Scope<K::Locale> for K {
    type Keys = K;
}

/// A struct representing a scoped locale
pub struct ScopedLocale<L: Locale, S: Scope<L> = <L as Locale>::Keys> {
    /// Base locale
    pub locale: L,
    scope_marker: PhantomData<S>,
}

impl<L: Locale, S: Scope<L>> ScopedLocale<L, S> {
    /// Create a new `ScopedLocale` with the given base locale
    pub const fn new(locale: L) -> Self {
        ScopedLocale {
            locale,
            scope_marker: PhantomData,
        }
    }
}

impl<L: Locale, S: Scope<L>> Debug for ScopedLocale<L, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <L as Debug>::fmt(&self.locale, f)
    }
}

impl<L: Locale, S: Scope<L>> Default for ScopedLocale<L, S> {
    fn default() -> Self {
        ScopedLocale {
            locale: Default::default(),
            scope_marker: PhantomData,
        }
    }
}

impl<L: Locale, S: Scope<L>> PartialEq for ScopedLocale<L, S> {
    fn eq(&self, other: &Self) -> bool {
        self.locale == other.locale
    }
}

impl<L: Locale, S: Scope<L>> Eq for ScopedLocale<L, S> {}

impl<L: Locale, S: Scope<L>> Clone for ScopedLocale<L, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Locale, S: Scope<L>> Copy for ScopedLocale<L, S> {}

impl<L: Locale, S: Scope<L>> fmt::Display for ScopedLocale<L, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <L as fmt::Display>::fmt(&self.locale, f)
    }
}

impl<L: Locale, S: Scope<L>> AsRef<LanguageIdentifier> for ScopedLocale<L, S> {
    fn as_ref(&self) -> &LanguageIdentifier {
        self.locale.as_ref()
    }
}

impl<L: Locale, S: Scope<L>> AsRef<IcuLocale> for ScopedLocale<L, S> {
    fn as_ref(&self) -> &IcuLocale {
        self.locale.as_ref()
    }
}

impl<L: Locale, S: Scope<L>> AsRef<str> for ScopedLocale<L, S> {
    fn as_ref(&self) -> &str {
        self.locale.as_ref()
    }
}

impl<L: Locale, Sc: Scope<L>> AsRef<L> for ScopedLocale<L, Sc> {
    fn as_ref(&self) -> &L {
        &self.locale
    }
}

impl<L: Locale, S: Scope<L>> Hash for ScopedLocale<L, S> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        Hash::hash(&self.locale, state)
    }
}

impl<L: Locale, S: Scope<L>> FromStr for ScopedLocale<L, S> {
    type Err = <L as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let locale = <L as FromStr>::from_str(s)?;
        Ok(ScopedLocale {
            locale,
            scope_marker: PhantomData,
        })
    }
}

impl<L: Locale, S: Scope<L>> Locale<L> for ScopedLocale<L, S> {
    type Keys = S::Keys;
    type TranslationUnitId = L::TranslationUnitId;
    #[cfg(all(feature = "dynamic_load", not(feature = "csr")))]
    type ServerFn = L::ServerFn;

    fn as_str(self) -> &'static str {
        <L as Locale>::as_str(self.locale)
    }

    fn direction(self) -> Direction {
        <L as Locale>::direction(self.locale)
    }

    fn as_icu_locale(self) -> &'static IcuLocale {
        <L as Locale>::as_icu_locale(self.locale)
    }

    fn get_all() -> &'static [L] {
        <L as Locale>::get_all()
    }

    fn to_base_locale(self) -> L {
        self.locale
    }

    fn from_base_locale(locale: L) -> Self {
        ScopedLocale {
            locale,
            scope_marker: PhantomData,
        }
    }

    #[cfg(feature = "dynamic_load")]
    fn request_translations(
        self,
        translations_id: Self::TranslationUnitId,
    ) -> impl std::future::Future<
        Output = Result<
            crate::fetch_translations::LocaleServerFnOutput,
            leptos::prelude::ServerFnError,
        >,
    > {
        L::request_translations(self.locale, translations_id)
    }

    #[cfg(all(feature = "dynamic_load", feature = "hydrate"))]
    fn init_translations(self, translations_id: Self::TranslationUnitId, values: Vec<Box<str>>) {
        L::init_translations(self.locale, translations_id, values);
    }
}

impl<L: Locale, Sc: Scope<L>> serde::Serialize for ScopedLocale<L, Sc> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.to_base_locale(), serializer)
    }
}

impl<'de, L: Locale, S: Scope<L>> serde::Deserialize<'de> for ScopedLocale<L, S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let base_locale: L = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_base_locale(base_locale))
    }
}
