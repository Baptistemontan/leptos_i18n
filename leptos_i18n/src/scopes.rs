use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    str::FromStr,
};

use unic_langid::LanguageIdentifier;

use crate::{I18nContext, Locale, LocaleKeys};

/// Represent a scope in a locale.
pub trait Scope<L: Locale>: 'static {
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
    pub const fn map<NS: Scope<L>>(self, map_fn: fn(&S) -> &NS) -> ConstScope<L, NS> {
        let _ = map_fn;
        ConstScope(PhantomData)
    }
}

#[doc(hidden)]
pub const fn scope_ctx_util<L: Locale, OS: Scope<L>, NS: Scope<L>>(
    ctx: I18nContext<L, OS>,
    map_fn: fn(&OS) -> &NS,
) -> I18nContext<L, NS> {
    let old_scope = ConstScope::<L, OS>::new();
    let new_scope = old_scope.map(map_fn);
    ctx.scope(new_scope)
}

#[doc(hidden)]
pub fn scope_locale_util<BL: Locale, L: Locale<BL>, NS: Scope<BL>>(
    locale: L,
    map_fn: fn(&<L as Locale<BL>>::Keys) -> &NS,
) -> ScopedLocale<BL, NS> {
    let _ = map_fn;
    ScopedLocale {
        locale: locale.to_base_locale(),
        scope_marker: PhantomData,
    }
}

pub struct ScopedLocale<L: Locale, S: Scope<L> = <L as Locale>::Keys> {
    locale: L,
    scope_marker: PhantomData<S>,
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

    fn as_str(self) -> &'static str {
        todo!()
    }

    fn as_langid(self) -> &'static LanguageIdentifier {
        todo!()
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
}
