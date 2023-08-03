use std::{collections::HashMap, ops::Deref, sync::Arc};

#[cfg(feature = "ssr")]
mod accepted_lang;
#[cfg(feature = "ssr")]
mod server;
mod t_macro;
mod view;

#[cfg(feature = "ssr")]
pub use server::config;

pub use view::{get_locale, set_locale, translate, I18nContextProvider};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LocaleInner {
    lang: String,
    translations: HashMap<String, String>,
}

impl LocaleInner {
    fn get_by_key(&self, key: &str) -> Option<&'_ str> {
        self.translations.get(key).map(String::as_str)
    }

    fn get_by_key_with_default<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get_by_key(key).unwrap_or(default)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Locale(Arc<LocaleInner>);

impl Deref for Locale {
    type Target = LocaleInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        &self.0.lang
    }
}
