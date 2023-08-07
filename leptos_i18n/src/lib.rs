#[cfg(feature = "ssr")]
mod accepted_lang;
mod fetch_locale;
mod locale_traits;
#[cfg(feature = "ssr")]
pub mod server;
mod t_macro;
mod view;

pub(crate) const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

pub use locale_traits::*;

pub use view::{get_context, get_locale, get_variant, set_locale, I18nContextProvider};

pub use leptos_i18n_macro::*;
