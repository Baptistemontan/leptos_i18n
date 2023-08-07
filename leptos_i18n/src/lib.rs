mod fetch_locale;
mod locale_traits;
#[cfg(feature = "ssr")]
pub mod server;
mod t_macro;
mod view;

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub(crate) const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

pub use locale_traits::*;

pub use view::{get_context, get_locale, get_variant, provide_i18n_context, set_locale};

pub use leptos_i18n_macro::*;
