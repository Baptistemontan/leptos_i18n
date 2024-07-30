use std::{borrow::Cow, str::FromStr};

use unic_langid::LanguageIdentifier;

use crate::langid::{convert_vec_str_to_langids_lossy, filter_matches, find_match};

/// Trait implemented the enum representing the supported locales of the application
///
/// Most functions of this crate are generic of type implementing this trait
pub trait Locale:
    'static
    + Default
    + Clone
    + Copy
    + FromStr
    + AsRef<LanguageIdentifier>
    + std::fmt::Display
    + PartialEq
{
    /// The associated struct containing the translations
    type Keys: LocaleKeys<Locale = Self>;

    /// Return a static str that represent the locale.
    fn as_str(self) -> &'static str;

    /// Return a static reference to a `LanguageIdentifier`
    fn as_langid(self) -> &'static LanguageIdentifier;

    /// Return a static reference to an array containing all variants of this enum
    fn get_all() -> &'static [Self];

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    fn find_locale<T: AsRef<[u8]>>(accepted_languages: &[T]) -> Self {
        let langids = convert_vec_str_to_langids_lossy(accepted_languages);
        find_match(&langids, Self::get_all())
    }

    /// Given a langid, return a Vec of suitables `Locale` sorted in compatibility (first one being the best match).
    ///
    /// This function does not fallback to default if no match is found.
    fn find_matchs<T: AsRef<LanguageIdentifier>>(langid: T) -> Vec<Self> {
        filter_matches(std::slice::from_ref(langid.as_ref()), Self::get_all())
    }

    /// Return the keys based on self
    #[inline]
    fn get_keys(self) -> &'static Self::Keys {
        LocaleKeys::from_locale(self)
    }
}

/// Trait implemented the struct representing the translation keys
///
/// You will probably never need to use it has it only serves the internals of the library.
pub trait LocaleKeys: 'static + Clone + Copy {
    /// The associated enum representing the supported locales
    type Locale: Locale<Keys = Self>;

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
