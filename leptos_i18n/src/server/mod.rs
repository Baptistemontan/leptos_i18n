#[cfg(all(feature = "actix", not(feature = "axum")))]
mod actix;
#[cfg(all(feature = "actix", not(feature = "axum")))]
pub use actix::*;

#[cfg(all(feature = "axum", not(feature = "actix")))]
mod axum;
#[cfg(all(feature = "axum", not(feature = "actix")))]
pub use axum::*;

#[cfg(all(feature = "actix", feature = "axum"))]
compile_error!("Can't enable \"actix\" and \"axum\" feature.");
