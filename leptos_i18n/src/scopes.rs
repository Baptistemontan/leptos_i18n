use std::{
    fmt::{self, Debug},
    hash::Hash,
    marker::PhantomData,
    str::FromStr,
};

use icu::locid;
use leptos_router::ChooseView;

use crate::{I18nContext, Locale, LocaleKeys};

/// Represent a scope in a locale.
pub trait Scope<L: Locale>: 'static + Send + Sync {
    /// The keys of the scopes
    type Keys: LocaleKeys<Locale = L>;
}

impl<K: LocaleKeys> Scope<K::Locale> for K {
    type Keys = K;
}

/// A struct that act as a marker for a scope, can be constructed as a constant and can be used to scope a context or a locale.
pub struct ConstScope<L: Locale, S: Scope<L> = <L as Locale>::Keys>(PhantomData<(L, S)>);

impl<L: Locale, S: Scope<L>> Default for ConstScope<L, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Locale, S: Scope<L>> Clone for ConstScope<L, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Locale, S: Scope<L>> Copy for ConstScope<L, S> {}

impl<L: Locale, S: Scope<L>> ConstScope<L, S> {
    /// Create a marker for a scope
    pub const fn new() -> Self {
        ConstScope(PhantomData)
    }

    /// This function is a helper for type resolution in macros.
    ///
    /// You can use it but it's meant to be used inside `use_i18n_scoped!` and `scope_i18n`.
    pub const fn new_from_ctx(_: I18nContext<L, S>) -> Self {
        Self::new()
    }

    #[doc(hidden)]
    pub const fn map<NS: Scope<L>>(self, map_fn: fn(S) -> NS) -> ConstScope<L, NS> {
        let _ = map_fn;
        ConstScope(PhantomData)
    }
}

pub struct ScopedLocale<L: Locale, S: Scope<L> = <L as Locale>::Keys> {
    pub locale: L,
    scope_marker: PhantomData<S>,
}

impl<L: Locale, S: Scope<L>> ScopedLocale<L, S> {
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

impl<L: Locale, S: Scope<L>> AsRef<locid::LanguageIdentifier> for ScopedLocale<L, S> {
    fn as_ref(&self) -> &locid::LanguageIdentifier {
        self.locale.as_ref()
    }
}

impl<L: Locale, S: Scope<L>> AsRef<locid::Locale> for ScopedLocale<L, S> {
    fn as_ref(&self) -> &locid::Locale {
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
    type Routes<View, Chil> = L::Routes<View, Chil>;
    type TranslationUnitId = L::TranslationUnitId;
    #[cfg(feature = "dynamic_load")]
    type ServerFn = L::ServerFn;

    fn as_str(self) -> &'static str {
        <L as Locale>::as_str(self.locale)
    }

    fn as_icu_locale(self) -> &'static locid::Locale {
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

    fn make_routes<View, Chil>(
        base_route: crate::routing::BaseRoute<View, Chil>,
        base_path: &'static str,
    ) -> Self::Routes<View, Chil>
    where
        View: ChooseView,
    {
        L::make_routes(base_route, base_path)
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
    fn init_translations(self, translations_id: Self::TranslationUnitId, values: Vec<String>) {
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
