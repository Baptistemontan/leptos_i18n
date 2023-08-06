#[cfg(feature = "ssr")]
mod accepted_lang;
mod locale_traits;
#[cfg(feature = "ssr")]
pub mod server;
mod t_macro;
mod view;

pub use locale_traits::*;

pub use view::{get_context, get_locale, get_variant, set_locale, I18nContextProvider};

pub use leptos_i18n_macro::*;
