use crate::locale_traits::*;
use axum::http::header;
use leptos::*;

pub fn fetch_locale_server<T: Locales>() -> T::Variants {
    // when leptos_router inspect the routes it execute the code once but don't set a RequestParts in the context,
    // so we can't expect it to be present.
    use_context::<leptos_axum::RequestParts>()
        .map(|req| from_req(&req))
        .unwrap_or_default()
}

fn from_req<T: LocaleVariant>(req: &leptos_axum::RequestParts) -> T {
    #[cfg(feature = "cookie")]
    if let Some(pref_lang_cookie) = get_prefered_lang_cookie::<T>(req) {
        return pref_lang_cookie;
    }

    let Some(header) = req
        .headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|header| header.to_str().ok())
    else {
        return Default::default();
    };

    let langs = super::parse_header(header);

    T::find_locale(&langs)
}

#[cfg(feature = "cookie")]
fn get_prefered_lang_cookie<T: LocaleVariant>(req: &leptos_axum::RequestParts) -> Option<T> {
    req.headers
        .get_all(header::COOKIE)
        .into_iter()
        .filter_map(parse_cookie)
        .filter_map(T::from_str)
        .next()
}

#[cfg(feature = "cookie")]
fn parse_cookie(cookie: &axum::http::HeaderValue) -> Option<&str> {
    std::str::from_utf8(cookie.as_bytes())
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .filter_map(|s| s.split_once('='))
        .find(|(name, _)| name == &crate::COOKIE_PREFERED_LANG)
        .map(|(_, value)| value)
}
