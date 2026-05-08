use icu_locale::{LanguageIdentifier, Locale as IcuLocale};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{
    Scope,
    langid::{convert_vec_str_to_langids_lossy, filter_matches, find_match},
    scopes::ScopedLocale,
};

pub trait Locale: LocaleRequirementsMarker {
    /// Convert this type to the base locale, this is used for wrappers around a locale such as scopes.
    fn to_base_locale(self) -> Self::BaseLocale;

    /// Create this type from a base locale, this is used for wrappers around a locale such as scopes.
    fn from_base_locale(locale: Self::BaseLocale) -> Self;

    /// Map the locale with another value, this is useful to change the locale of a scope.
    fn map_locale(self, locale: Self::BaseLocale) -> Self {
        Self::from_base_locale(locale)
    }

    /// Scope the locale to the given scope
    fn scope<S: Scope<BaseLocale = Self::BaseLocale>>(self) -> ScopedLocale<S> {
        ScopedLocale::new(self.to_base_locale())
    }

    /// Return a static str that represent the base locale.
    fn as_str(self) -> &'static str {
        Locale::as_str(self.to_base_locale())
    }

    /// Return a static reference to a icu `Locale`
    fn as_icu_locale(self) -> &'static IcuLocale {
        Locale::as_icu_locale(self.to_base_locale())
    }

    /// Return the direction of the locale.
    fn direction(self) -> Direction {
        Locale::direction(self.to_base_locale())
    }

    /// Return a static reference to a `LanguageIdentifier`
    fn as_langid(self) -> &'static LanguageIdentifier {
        let icu_locale = Locale::as_icu_locale(self);
        &icu_locale.id
    }

    /// Given a slice of accepted languages sorted in preferred order, return the locale that fit the best the request.
    fn find_locale<T: AsRef<[u8]>>(accepted_languages: &[T]) -> Self {
        let langids = convert_vec_str_to_langids_lossy(accepted_languages);
        let availables = Self::iter_variants().collect();
        find_match(&langids, availables)
    }

    /// Given a langid, return a Vec of suitables `Locale` sorted in compatibility (first one being the best match).
    ///
    /// This function does not fallback to default if no match is found.
    fn find_matchs<T: AsRef<LanguageIdentifier>>(langid: T) -> Vec<Self> {
        let availables = Self::iter_variants().collect();
        filter_matches(std::slice::from_ref(langid.as_ref()), availables)
    }

    fn iter_variants() -> impl Iterator<Item = Self> {
        <Self::BaseLocale as BaseLocale>::ALL_VARIANTS
            .iter()
            .copied()
            .map(Locale::from_base_locale)
    }
}

/// Trait implemented the enum representing the supported locales of the application
///
/// Carefull when implementing this trait, methods `as_str`, `as_icu_locale` and `direction` on the `Locale` trait have their default impl based on the base locale impl,
/// so when implementing `BaseLocale` for your type you must also override the impls of those methods on your impl of `Locale` to not have infinite recursion.
pub trait BaseLocale: Locale<BaseLocale = Self> + LocaleRequirementsMarker {
    const ALL_VARIANTS: &'static [Self];

    /// Enum where each variants is an ID of a translation unit
    type TranslationUnitId: TranslationUnitId;

    /// Associated `#[server]` function type to request the translations
    #[cfg(all(feature = "dynamic_load", not(feature = "csr")))]
    type ServerFn: leptos::server_fn::ServerFn;

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
    fn init_translations(self, translations_id: Self::TranslationUnitId, values: Vec<Box<str>>);
}

pub trait LocaleRequirementsMarker:
    'static
    + Default
    + Copy
    + FromStr<Err: Display>
    + AsRef<LanguageIdentifier>
    + AsRef<IcuLocale>
    + AsRef<str>
    + AsRef<<Self as Scope>::BaseLocale>
    + std::fmt::Display
    + std::fmt::Debug
    + Ord
    + Hash
    + Send
    + Sync
    + serde::Serialize
    + serde::de::DeserializeOwned
    + Scope
{
}

impl<L> LocaleRequirementsMarker for L where
    L: 'static
        + Default
        + Copy
        + FromStr<Err: Display>
        + AsRef<LanguageIdentifier>
        + AsRef<IcuLocale>
        + AsRef<str>
        + AsRef<<Self as Scope>::BaseLocale>
        + std::fmt::Display
        + std::fmt::Debug
        + Ord
        + Hash
        + Send
        + Sync
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Scope
{
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

/// Represents the direction of a script.
/// This is computed at compile time with [`icu_locale::LocaleDirectionality`](https://docs.rs/icu_locale/2.0.0/icu_locale/struct.LocaleDirectionality.html)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    /// The script is left-to-right.
    LeftToRight,
    /// The script is right-to-left.
    RightToLeft,
    /// `icu_locale::LocaleDirectionality::get` return an Option, this variant represent the None case, it is unknown.
    Auto,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Direction {
    /// Return the string representation for the the html `dir` attribute: "ltr", "rtl" and "auto".
    pub const fn as_str(self) -> &'static str {
        match self {
            Direction::LeftToRight => "ltr",
            Direction::RightToLeft => "rtl",
            Direction::Auto => "auto",
        }
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
                test: "{{ some_var }}"
            },
        },
        fr: {
            sk: {
                ssk: "test fr",
                test: "<p>{{ some_var }}</p>"
            },
        },
    }

    use super::Locale as _;
    use crate::key;
    use i18n::Locale;

    macro_rules! scope {
        ($scope: expr, $first_key:ident $(.$key:ident)*) => {
            {
                let scope = $scope;
                $crate::__private::check_is_scope(
                    $crate::__private::get_keys_from_ref(&scope)
                    .$first_key()
                    $(.$key())*
                )
            }
        };
    }

    macro_rules! const_value {
        ($key: expr, $locale: expr) => {
            $crate::__private::check_is_literal({
                let (args, id) =
                    $crate::keys::Key::const_into_args_and_id($crate::build_key!($key));
                args.__const_value(id, $locale)
            })
        };
    }

    const _: () = {
        const fn check_str_eq_const(a: &str, b: &str) -> bool {
            if a.len() != b.len() {
                return false;
            }
            let (mut a, mut b) = (a.as_bytes(), b.as_bytes());
            loop {
                match (a.split_first(), b.split_first()) {
                    (Some((first_a, rest_a)), Some((first_b, rest_b))) if *first_a == *first_b => {
                        a = rest_a;
                        b = rest_b;
                    }
                    (None, None) => return true,
                    _ => return false,
                }
            }
        }
        let ssk = key!(scope = Locale, sk.ssk);
        let fr_ssk = const_value!(ssk, Locale::fr);
        assert!(check_str_eq_const(fr_ssk, "test fr"));
        let en_ssk = const_value!(ssk, Locale::en);
        assert!(check_str_eq_const(en_ssk, "test en"));
    };

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
    fn test_scope() {
        let sk = scope!(Locale::en, sk);
        let ssk = key!(sk, ssk);

        assert_eq!(const_value!(ssk, Locale::en), "test en");
        assert_eq!(const_value!(ssk, Locale::fr), "test fr");
    }
}
