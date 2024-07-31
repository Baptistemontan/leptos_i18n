use std::{marker::PhantomData, str::FromStr};

use unic_langid::LanguageIdentifier;

use crate::{I18nContext, Locale, LocaleKeys};

pub struct ScopedLocale<L: Locale, K: LocaleKeys<Locale = L> = <L as Locale>::RootKeys> {
    locale: L,
    keys_marker: PhantomData<K>,
}

impl<L: Locale, K: LocaleKeys<Locale = L>> ScopedLocale<L, K> {
    pub fn new(locale: L) -> ScopedLocale<L, K> {
        ScopedLocale {
            locale,
            keys_marker: PhantomData,
        }
    }
}

impl<L: Locale, K: LocaleKeys<Locale = L>> Default for ScopedLocale<L, K> {
    fn default() -> Self {
        Self {
            locale: Default::default(),
            keys_marker: PhantomData,
        }
    }
}

impl<L: Locale, K: LocaleKeys<Locale = L>> Clone for ScopedLocale<L, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Locale, K: LocaleKeys<Locale = L>> Copy for ScopedLocale<L, K> {}
impl<L: Locale, K: LocaleKeys<Locale = L>> FromStr for ScopedLocale<L, K> {
    type Err = <L as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let locale = L::from_str(s)?;
        Ok(ScopedLocale {
            locale,
            keys_marker: PhantomData,
        })
    }
}
impl<L: Locale, K: LocaleKeys<Locale = L>> AsRef<LanguageIdentifier> for ScopedLocale<L, K> {
    fn as_ref(&self) -> &LanguageIdentifier {
        self.locale.as_ref()
    }
}
impl<L: Locale, K: LocaleKeys<Locale = L>> std::fmt::Display for ScopedLocale<L, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <L as std::fmt::Display>::fmt(&self.locale, f)
    }
}
impl<L: Locale, K: LocaleKeys<Locale = L>> PartialEq for ScopedLocale<L, K> {
    fn eq(&self, other: &Self) -> bool {
        self.locale == other.locale
    }
}

impl<L: Locale, K: LocaleKeys<Locale = L>> Locale<L> for ScopedLocale<L, K> {
    type RootKeys = K;

    fn as_str(self) -> &'static str {
        self.locale.as_str()
    }

    fn as_langid(self) -> &'static LanguageIdentifier {
        self.locale.as_langid()
    }

    fn get_all() -> &'static [L] {
        L::get_all()
    }

    fn to_underlying_locale(self) -> L {
        self.locale
    }
}

#[doc(hidden)]
#[inline]
pub fn scope_locale_util<L: Locale, SP: Locale<L>, S: LocaleKeys<Locale = L>>(
    loc: SP,
    _: impl FnOnce(&SP::RootKeys) -> &S,
) -> ScopedLocale<L, S> {
    loc.scope()
}

#[doc(hidden)]
#[inline]
pub fn scope_ctx_util<L: Locale, P: LocaleKeys<Locale = L>, S: LocaleKeys<Locale = L>>(
    ctx: I18nContext<L, P>,
    _: impl FnOnce(&P) -> &S,
) -> I18nContext<L, S> {
    ctx.scope()
}
