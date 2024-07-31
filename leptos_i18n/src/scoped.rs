use std::{marker::PhantomData, str::FromStr};

use unic_langid::LanguageIdentifier;

use crate::{I18nContext, Locale, LocaleKeys};

/// Represent a scope into a `Locale`, this is the mechanism to scope the context or a locale to a namespace or subkeys.
pub trait Scope<L: Locale>: 'static {
    /// The Scoped Keys
    type Keys: LocaleKeys<Locale = L>;
}

impl<BL: Locale, L: Locale<BL>> Scope<BL> for L {
    type Keys = L::RootKeys;
}

/// Represent a `Locale` with a given Scope.
pub struct ScopedLocale<L: Locale, S: Scope<L>> {
    locale: L,
    scope_marker: PhantomData<S>,
}

impl<L: Locale, S: Scope<L>> ScopedLocale<L, S> {
    /// Create a new Scoped `Locale`
    pub fn new(locale: L) -> Self {
        ScopedLocale {
            locale,
            scope_marker: PhantomData,
        }
    }
}

impl<L: Locale, S: Scope<L>> Default for ScopedLocale<L, S> {
    fn default() -> Self {
        Self {
            locale: Default::default(),
            scope_marker: PhantomData,
        }
    }
}

impl<L: Locale, S: Scope<L>> Clone for ScopedLocale<L, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Locale, S: Scope<L>> Copy for ScopedLocale<L, S> {}
impl<L: Locale, S: Scope<L>> FromStr for ScopedLocale<L, S> {
    type Err = <L as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let locale = L::from_str(s)?;
        Ok(ScopedLocale {
            locale,
            scope_marker: PhantomData,
        })
    }
}

impl<L: Locale, S: Scope<L>> AsRef<LanguageIdentifier> for ScopedLocale<L, S> {
    fn as_ref(&self) -> &LanguageIdentifier {
        self.locale.as_ref()
    }
}

impl<L: Locale, S: Scope<L>> std::fmt::Display for ScopedLocale<L, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <L as std::fmt::Display>::fmt(&self.locale, f)
    }
}

impl<L: Locale, S: Scope<L>> PartialEq for ScopedLocale<L, S> {
    fn eq(&self, other: &Self) -> bool {
        self.locale == other.locale
    }
}

impl<L: Locale, S: Scope<L>> Locale<L> for ScopedLocale<L, S> {
    type RootKeys = S::Keys;

    fn as_str(self) -> &'static str {
        self.locale.as_str()
    }

    fn as_langid(self) -> &'static LanguageIdentifier {
        self.locale.as_langid()
    }

    fn to_base_locale(self) -> L {
        self.locale
    }

    fn get_all_base() -> &'static [L] {
        L::get_all()
    }

    fn get_all() -> &'static [Self] {
        todo!()
    }
}

#[doc(hidden)]
#[doc(hidden)]
#[inline]
pub fn scope_locale_util<L: Locale, SP: Locale<L>, S: Scope<L>>(
    loc: SP,
    _: impl FnOnce(&SP::RootKeys) -> &S,
) -> ScopedLocale<L, S> {
    loc.scope()
}

#[doc(hidden)]
#[inline]
pub fn scope_ctx_util<L: Locale, S: Scope<L>, NS: Scope<L>>(
    ctx: I18nContext<L, S>,
    _: impl FnOnce(&S) -> &NS,
) -> I18nContext<L, NS> {
    ctx.scope()
}

#[cfg(test)]
mod test {
    leptos_i18n_macro::declare_locales! {
        path: crate,
        default: "en",
        locales: ["en", "fr"],
        en: {
            subkeys: {
                value: "en"
            },
        },
        fr: {
            subkeys: {
                value: "fr"
            }
        },
    }

    use crate::Locale as _;

    use super::scope_locale_util;
    use crate as leptos_i18n;
    use i18n::*;

    #[test]
    fn test_scoped() {
        let en = scope_locale_util(Locale::en, |k| &k.subkeys);
        let fr = scope_locale_util(Locale::fr, |k| &k.subkeys);

        assert_eq!(en.get_keys().value, Locale::en.get_keys().subkeys.value);
        assert_eq!(fr.get_keys().value, Locale::fr.get_keys().subkeys.value);

        let en = scope_locale!(Locale::en, subkeys);
        let fr = scope_locale_util(Locale::fr, |k| &k.subkeys);

        assert_eq!(en.get_keys().value, Locale::en.get_keys().subkeys.value);
        assert_eq!(fr.get_keys().value, Locale::fr.get_keys().subkeys.value);
        // let EN = scope_locale!(Locale::en, subkeys);
    }
}
