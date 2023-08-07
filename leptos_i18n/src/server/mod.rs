pub const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

#[cfg(all(feature = "actix", not(feature = "axum")))]
mod actix;
#[cfg(all(feature = "actix", not(feature = "axum")))]
use actix::*;

#[cfg(all(feature = "axum", not(feature = "actix")))]
mod axum;
#[cfg(all(feature = "axum", not(feature = "actix")))]
use axum::*;

#[cfg(all(feature = "actix", feature = "axum"))]
compile_error!("Can't enable \"actix\" and \"axum\" features together.");
#[cfg(not(any(feature = "actix", feature = "axum")))]
compile_error!("Need either \"actix\" or \"axum\" feature to be enabled.");
#[cfg(any(
    all(feature = "actix", feature = "axum"),
    not(any(feature = "actix", feature = "axum"))
))]
fn fetch_locale_server<T>(_: T) -> ! {
    unimplemented!()
}

use leptos::*;

use crate::Locales;

pub fn fetch_locale_server_side<T: Locales>(cx: Scope) -> T::Variants {
    fetch_locale_server::<T>(cx)
}
