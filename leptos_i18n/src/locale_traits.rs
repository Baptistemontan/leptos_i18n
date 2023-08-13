use serde::{de::DeserializeOwned, Serialize};

pub trait LocaleVariant: 'static + Default + Serialize + DeserializeOwned + Clone + Copy {
    fn from_str(s: &str) -> Option<Self>;
    fn as_str(&self) -> &'static str;

    fn find_locale<T: AsRef<str>>(accepted_langs: &[T]) -> Self {
        accepted_langs
            .iter()
            .find_map(|l| Self::from_str(l.as_ref()))
            .unwrap_or_default()
    }
}

pub trait LocaleKeys: 'static + Clone + Copy {
    type Locales: Locales;

    fn from_variant(variant: <Self::Locales as Locales>::Variants) -> Self;
}

pub trait Locales: 'static + Clone + Copy {
    type Variants: LocaleVariant;
    type LocaleKeys: LocaleKeys<Locales = Self>;

    #[inline]
    fn get_keys(locale: Self::Variants) -> Self::LocaleKeys {
        <Self::LocaleKeys as LocaleKeys>::from_variant(locale)
    }
}
