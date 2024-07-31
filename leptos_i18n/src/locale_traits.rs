use std::{borrow::Cow, str::FromStr};

use unic_langid::LanguageIdentifier;

use crate::{
    langid::{convert_vec_str_to_langids_lossy, filter_matches, find_match},
    scoped::ScopedLocale,
};

/// Trait implemented the enum representing the supported locales of the application
///
/// Most functions of this crate are generic of type implementing this trait
pub trait Locale<L: Locale = Self>:
    'static
    + Default
    + Clone
    + Copy
    + FromStr
    + AsRef<LanguageIdentifier>
    + std::fmt::Display
    + PartialEq
{
    /// The associated type containing the entire translations
    type RootKeys: LocaleKeys<Locale = L>;

    /// Return a static str that represent the locale.
    fn as_str(self) -> &'static str;

    /// Return a static reference to a `LanguageIdentifier`
    fn as_langid(self) -> &'static LanguageIdentifier;

    /// Return a static reference to an array containing all variants of this enum
    fn get_all() -> &'static [L];

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    fn find_locale<T: AsRef<[u8]>>(accepted_languages: &[T]) -> L {
        let langids = convert_vec_str_to_langids_lossy(accepted_languages);
        find_match(&langids, Self::get_all())
    }

    /// Given a langid, return a Vec of suitables `Locale` sorted in compatibility (first one being the best match).
    ///
    /// This function does not fallback to default if no match is found.
    fn find_matchs<T: AsRef<LanguageIdentifier>>(langid: T) -> Vec<L> {
        filter_matches(std::slice::from_ref(langid.as_ref()), Self::get_all())
    }

    /// Return the values of the given Keys
    #[inline]
    fn get_keys<K: LocaleKeys<Locale = L>>(self) -> &'static K {
        LocaleKeys::from_locale(self.to_underlying_locale())
    }

    /// Return the root keys
    #[inline]
    fn get_root_keys(self) -> &'static Self::RootKeys {
        Self::get_keys(self)
    }

    /// Returns the underlying locale, this for wrappers implementing `Locale` to return the base value, such as `ScopedLocale`
    fn to_underlying_locale(self) -> L;

    /// Scope the locale to rthe given keys.
    fn scope<K: LocaleKeys<Locale = L>>(self) -> ScopedLocale<L, K> {
        ScopedLocale::new(self.to_underlying_locale())
    }
}

/// Trait implemented the struct representing the translation keys
///
/// You will probably never need to use it has it only serves the internals of the library.
pub trait LocaleKeys: 'static + Clone + Copy {
    /// The associated enum representing the supported locales
    type Locale: Locale;

    /// Return a static ref to Self containing the translations for the given locale
    fn from_locale(locale: Self::Locale) -> &'static Self;
}

/// This is used to call `.build` on `&str` when building interpolations.
///
/// If it's a `&str` it will just return the str,
/// but if it's a builder `.build` will either emit an error for a missing key or if all keys
/// are supplied it will return the correct value
///
/// It has no uses outside of the internals of the `t!` macro.
#[doc(hidden)]
pub trait BuildStr: Sized {
    #[inline]
    fn build(self) -> Self {
        self
    }

    #[inline]
    fn build_display(self) -> Self {
        self
    }

    fn build_string(self) -> Cow<'static, str>;
}

impl BuildStr for &'static str {
    #[inline]
    fn build_string(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

#[cfg(test)]
mod test {
    leptos_i18n_macro::declare_locales! {
        path: crate,
        default: "en",
        locales: ["en", "fr"],
        en: {},
        fr: {},
    }

    use super::Locale as _;
    use i18n::Locale;

    #[test]
    fn test_find_locale() {
        let res = Locale::find_locale(&["de"]);
        assert_eq!(res, Locale::default());

        let res = Locale::find_locale(&["fr"]);
        assert_eq!(res, Locale::fr);

        let res = Locale::find_locale(&["en"]);
        assert_eq!(res, Locale::en);

        let res = Locale::find_locale(&["fr-FR"]);
        assert_eq!(res, Locale::fr);

        let res = Locale::find_locale(&["de", "fr-FR", "fr"]);
        assert_eq!(res, Locale::fr);
    }
}
