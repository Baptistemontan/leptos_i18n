/// Trait implemented the enum representing the supported locales of the application
///
/// Appart from maybe `as_str` you will probably never need to use it has it only serves the internals of the library.
pub trait LocaleVariant: 'static + Default + Clone + Copy {
    /// The associated struct containing the translations
    type Keys: LocaleKeys<Variants = Self>;

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
        LocaleKeys::from_variant(self)
    }
}

/// Trait implemented the struct representing the translation keys
///
/// You will probably never need to use it has it only serves the internals of the library.
pub trait LocaleKeys: 'static + Clone + Copy {
    /// The associated enum representing the supported locales
    type Variants: LocaleVariant<Keys = Self>;

    /// Create self according to the given locale.
    fn from_variant(variant: Self::Variants) -> &'static Self;
}

/// This trait servers as a bridge beetween the locale enum and the keys struct
pub trait Locales: 'static + Clone + Copy {
    /// The struct that represent the translations keys.
    type LocaleKeys: LocaleKeys<Variants = Self::Variants>;
    /// The enum that represent the different locales supported.
    type Variants: LocaleVariant<Keys = Self::LocaleKeys>;

    /// Create the keys according to the given locale.
    #[inline]
    fn get_keys(locale: Self::Variants) -> &'static Self::LocaleKeys {
        locale.get_keys()
    }
}

/// This is used to call `.build` on `&str` when building interpolations
///
/// if it's a `&str` it will just return the str,
/// but if it's a builder `.build` will either emit an error for a missing key or if all keys
/// are supplied it will return the correct value
#[doc(hidden)]
pub trait BuildStr: Sized {
    #[inline]
    fn build(self) -> Self {
        self
    }
}

impl<'a> BuildStr for &'a str {}
