/// Trait implemented the enum representing the supported locales of the application
///
/// Most functions of this crate are generic of type implementing this trait
pub trait Locale: 'static + Default + Clone + Copy {
    /// The associated struct containing the translations
    type Keys: LocaleKeys<Locale = Self>;

    /// Try to match the given str to a locale and returns it.
    fn from_str(s: &str) -> Option<Self>;

    /// Return a static str that represent the locale.
    fn as_str(self) -> &'static str;

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    fn find_locale<T: AsRef<str>>(accepted_langs: &[T]) -> Self {
        accepted_langs
            .iter()
            .find_map(|l| Self::from_str(l.as_ref()))
            .unwrap_or_default()
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
