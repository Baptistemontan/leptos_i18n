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

use leptos::*;

use crate::Locales;

pub fn fetch_locale_server_side<T: Locales>(cx: Scope) -> T::Variants {
    fetch_locale_server::<T>(cx)
}
