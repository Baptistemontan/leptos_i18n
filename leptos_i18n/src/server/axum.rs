use crate::locale_traits::*;
use axum::http::header;
use http::request::Parts;

pub fn fetch_locale_server<T: Locale>() -> T {
    // when leptos_router inspect the routes it execute the code once but don't set a RequestParts in the context,
    // so we can't expect it to be present.
    leptos::use_context::<Parts>()
        .map(|req| from_req(&req))
        .unwrap_or_default()
}

fn from_req<T: Locale>(req: &Parts) -> T {
    if cfg!(feature = "cookie") {
        if let Some(pref_lang_cookie) = get_prefered_lang_cookie::<T>(req) {
            return pref_lang_cookie;
        }
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

fn get_prefered_lang_cookie<T: Locale>(req: &Parts) -> Option<T> {
    req.headers
        .get_all(header::COOKIE)
        .into_iter()
        .filter_map(parse_cookie)
        .filter_map(T::from_str)
        .next()
}

fn parse_cookie(cookie: &axum::http::HeaderValue) -> Option<&str> {
    std::str::from_utf8(cookie.as_bytes())
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .filter_map(|s| s.split_once('='))
        .find(|(name, _)| name == &crate::COOKIE_PREFERED_LANG)
        .map(|(_, value)| value)
}
