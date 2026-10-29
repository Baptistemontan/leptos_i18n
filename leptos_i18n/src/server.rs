//! Access to the current request and response on the server.
//!
//! The server integrations expose both through the reactive context; the
//! `axum` and `actix` features select which one is read.

#[cfg(all(feature = "axum", feature = "actix"))]
compile_error!("only one of the features \"axum\" and \"actix\" can be enabled at a time");

/// The value of the request header `name`, when the enabled integration
/// provides the current request in the reactive context and the value is ASCII.
pub(crate) fn request_header(name: &str) -> Option<String> {
    #[cfg(feature = "axum")]
    {
        leptos::prelude::use_context::<http::request::Parts>()
            .and_then(|parts| parts.headers.get(name)?.to_str().ok().map(str::to_owned))
    }
    #[cfg(feature = "actix")]
    {
        leptos::prelude::use_context::<leptos_actix::Request>().and_then(|request| {
            request
                .headers()
                .get(name)?
                .to_str()
                .ok()
                .map(str::to_owned)
        })
    }
    #[cfg(not(any(feature = "axum", feature = "actix")))]
    {
        leptos::logging::warn!(
            "leptos_i18n cannot read the `{name}` request header: enable the `axum` or `actix` \
             feature, or provide the value through the context options"
        );
        None
    }
}

/// Appends a `Set-Cookie` header with the given value to the current response,
/// when the enabled integration provides the response options in the reactive context.
pub(crate) fn append_set_cookie(header: &str) {
    #[cfg(feature = "axum")]
    {
        if let (Some(response), Ok(value)) = (
            leptos::prelude::use_context::<leptos_axum::ResponseOptions>(),
            http::HeaderValue::from_str(header),
        ) {
            response.append_header(http::header::SET_COOKIE, value);
        }
    }
    #[cfg(feature = "actix")]
    {
        use actix_web::http::header::{HeaderValue, SET_COOKIE};
        if let (Some(response), Ok(value)) = (
            leptos::prelude::use_context::<leptos_actix::ResponseOptions>(),
            HeaderValue::from_str(header),
        ) {
            response.append_header(SET_COOKIE, value);
        }
    }
    #[cfg(not(any(feature = "axum", feature = "actix")))]
    {
        leptos::logging::warn!(
            "leptos_i18n cannot set the locale cookie `{header}`: enable the `axum` or `actix` \
             feature, or provide `ssr_set_cookie` through the cookie options"
        );
    }
}
