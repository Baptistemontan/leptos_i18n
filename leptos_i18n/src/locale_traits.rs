use std::str::FromStr;
use std::{fmt::Debug, hash::Hash};

use icu::locid;
use leptos::prelude::Renderer;
use leptos_router::ChooseView;

use crate::langid::{convert_vec_str_to_langids_lossy, filter_matches, find_match};

/// Trait implemented the enum representing the supported locales of the application
///
/// Most functions of this crate are generic of type implementing this trait
pub trait Locale<L: Locale = Self>:
    'static
    + Default
    + Clone
    + Copy
    + FromStr
    + AsRef<locid::LanguageIdentifier>
    + AsRef<locid::Locale>
    + AsRef<str>
    + AsRef<L>
    + std::fmt::Display
    + std::fmt::Debug
    + PartialEq
    + Eq
    + Hash
    + Send
    + Sync
    + serde::Serialize
    + serde::de::DeserializeOwned
{
    /// The associated struct containing the translations
    type Keys: LocaleKeys<Locale = L>;

    /// Associated routes for routing
    type Routes<View, Chil, R>;

    /// Associated `#[server]` function type to request the translations
    #[cfg(feature = "dynamic_load")]
    type ServerFn: leptos::server_fn::ServerFn;

    /// Enum where each variants is an ID of a translation unit
    type TranslationUnitId: TranslationUnitId;

    /// Return a static str that represent the locale.
    fn as_str(self) -> &'static str;

    /// Return a static reference to a icu `Locale`
    fn as_icu_locale(self) -> &'static locid::Locale;

    /// Return a static reference to a `LanguageIdentifier`
    fn as_langid(self) -> &'static locid::LanguageIdentifier {
        Locale::as_icu_locale(self).as_ref()
    }

    /// Return a static reference to an array containing all variants of this enum
    fn get_all() -> &'static [L];

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    fn find_locale<T: AsRef<[u8]>>(accepted_languages: &[T]) -> Self {
        let langids = convert_vec_str_to_langids_lossy(accepted_languages);
        let l = find_match(&langids, Self::get_all());
        Self::from_base_locale(l)
    }

    /// Given a langid, return a Vec of suitables `Locale` sorted in compatibility (first one being the best match).
    ///
    /// This function does not fallback to default if no match is found.
    fn find_matchs<T: AsRef<locid::LanguageIdentifier>>(langid: T) -> Vec<Self> {
        let matches: Vec<L> =
            filter_matches(std::slice::from_ref(langid.as_ref()), Self::get_all());
        matches.into_iter().map(Self::from_base_locale).collect()
    }

    /// Return the keys based on self
    #[inline]
    fn get_keys(self) -> Self::Keys {
        LocaleKeys::from_locale(self.to_base_locale())
    }

    /// Convert this type to the base locale, this is used for wrappers around a locale such as scopes.
    fn to_base_locale(self) -> L;

    /// Create this type from a base locale, this is used for wrappers around a locale such as scopes.
    fn from_base_locale(locale: L) -> Self;

    /// Map the locale with another value, this is useful to change the locale of a scope.
    fn map_locale(self, locale: L) -> Self {
        Self::from_base_locale(locale)
    }

    /// Make the routes
    fn make_routes<View, Chil, R>(
        base_route: crate::routing::BaseRoute<View, Chil, R>,
        base_path: &'static str,
    ) -> Self::Routes<View, Chil, R>
    where
        R: Renderer,
        View: ChooseView<R>;

    /// Associated `#[server]` function to request the translations
    #[cfg(feature = "dynamic_load")]
    fn request_translations(
        self,
        translations_id: Self::TranslationUnitId,
    ) -> impl std::future::Future<
        Output = Result<
            crate::fetch_translations::LocaleServerFnOutput,
            leptos::prelude::ServerFnError,
        >,
    > + Send
           + Sync
           + 'static;

    /// Init the translation unit of the given ID with the given values
    #[cfg(all(feature = "dynamic_load", feature = "hydrate"))]
    fn init_translations(self, translations_id: Self::TranslationUnitId, values: Vec<String>);
}

/// Trait implemented the struct representing the translation keys
///
/// You will probably never need to use it has it only serves the internals of the library.
pub trait LocaleKeys: 'static + Clone + Copy + Send + Sync {
    /// The associated enum representing the supported locales
    type Locale: Locale;

    /// Return a static ref to Self containing the translations for the given locale
    fn from_locale(locale: Self::Locale) -> Self;
}

/// Trait for the type giving an ID to each section of the translations
pub trait TranslationUnitId:
    serde::Serialize + serde::de::DeserializeOwned + Copy + Debug + Send + Sync + Eq + Hash + 'static
{
    /// Return the string representation of that ID
    fn to_str(self) -> Option<&'static str>;
}

impl TranslationUnitId for () {
    fn to_str(self) -> Option<&'static str> {
        None
    }
}

#[cfg(test)]
mod test {
    leptos_i18n_macro::declare_locales! {
        path: crate,
        default: "en",
        locales: ["en", "fr"],
        en: {
            sk: {
                ssk: "test en",
            },
        },
        fr: {
            sk: {
                ssk: "test fr",
            },
        },
    }

    use crate::Locale as _;
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

    #[test]
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn test_scope() {
        use crate::{self as leptos_i18n, __private::LitWrapper, scope_locale};
        let en_sk = scope_locale!(Locale::en, sk);
        assert_eq!(en_sk.get_keys().ssk(), LitWrapper::new("test en"));
        let fr_sk = en_sk.map_locale(Locale::fr);
        assert_eq!(fr_sk.get_keys().ssk(), LitWrapper::new("test fr"));
    }
}
