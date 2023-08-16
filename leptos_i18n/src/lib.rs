mod context;
mod fetch_locale;
mod locale_traits;
#[cfg(feature = "ssr")]
pub mod server;
// mod t_macro;

#[cfg(all(any(feature = "ssr", feature = "hydrate"), feature = "cookie"))]
pub(crate) const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

pub use locale_traits::*;

pub use context::{get_context, provide_i18n_context, I18nContext};

pub use leptos_i18n_macro::{load_locales, t};
