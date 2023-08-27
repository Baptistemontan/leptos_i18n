// use serde::{de::DeserializeOwned, Serialize};

/// Trait implemented the enum representing the supported locales of the application
///
/// Appart from maybe `as_str` you will probably never need to use it has it only serves the internals of the library.
pub trait LocaleVariant: 'static + Default + Clone + Copy {
    /// Try to match the given str to a locale and returns it.
    fn from_str(s: &str) -> Option<Self>;

    /// Return a static str that represent the locale.
    fn as_str(&self) -> &'static str;

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    fn find_locale<T: AsRef<str>>(accepted_langs: &[T]) -> Self {
        accepted_langs
            .iter()
            .find_map(|l| Self::from_str(l.as_ref()))
            .unwrap_or_default()
    }
}

/// Trait implemented the struct representing the translation keys
///
/// You will probably never need to use it has it only serves the internals of the library.
pub trait LocaleKeys: 'static + Clone + Copy {
    /// The associated `Locales` types that serves as a bridge beetween the locale enum and the keys.
    type Locales: Locales;

    /// Create self according to the given locale.
    fn from_variant(variant: <Self::Locales as Locales>::Variants) -> &'static Self;
}

/// This trait servers as a bridge beetween the locale enum and the keys struct
pub trait Locales: 'static + Clone + Copy {
    /// The enum that represent the different locales supported.
    type Variants: LocaleVariant;
    /// The struct that represent the translations keys.
    type LocaleKeys: LocaleKeys<Locales = Self>;

    /// Create the keys according to the given locale.
    #[inline]
    fn get_keys(locale: Self::Variants) -> &'static Self::LocaleKeys {
        <Self::LocaleKeys as LocaleKeys>::from_variant(locale)
    }
}
