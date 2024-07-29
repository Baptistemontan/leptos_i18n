use std::borrow::Cow;

/// Trait implemented the enum representing the supported locales of the application
///
/// Most functions of this crate are generic of type implementing this trait
pub trait Locale: 'static + Default + Clone + Copy {
    /// The associated struct containing the translations
    type Keys: LocaleKeys<Locale = Self>;

    /// Try to exact match the given str to a locale and returns it.
    fn from_str(s: &str) -> Option<Self>;

    /// Try to match the given locale parts to a locale and returns it.
    ///
    /// The implentation should allow "*-FR" to match "fr-FR".
    fn from_parts(s: &[&str]) -> Option<Self>;

    /// Return a static str that represent the locale.
    fn as_str(self) -> &'static str;

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    ///
    /// This implementation should follows most of the section 3.4 of the best current practice of the "Matching of Language Tags" memo.
    /// see: <https://datatracker.ietf.org/doc/html/rfc4647#section-3.4>
    fn find_locale<T: AsRef<str>>(accepted_langs: &[T]) -> Self {
        // FIXME: This implementation is not exactly complient.
        let langs: Vec<Vec<_>> = accepted_langs
            .iter()
            .map(AsRef::as_ref)
            .filter(|l| *l != "*")
            .map(|s| s.split('-').map(str::trim).collect())
            .collect();

        Self::find_locale_from_parts(langs)
    }

    /// Same as `find_locale` but takes a Vec of locale identifier parts
    fn find_locale_from_parts(mut langs: Vec<Vec<&str>>) -> Self {
        while !langs.is_empty() {
            for mut parts in std::mem::take(&mut langs) {
                if let Some(locale) = Self::from_parts(&parts) {
                    return locale;
                }
                parts.pop();
                if !parts.is_empty() {
                    langs.push(parts);
                }
            }
        }
        Default::default()
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
