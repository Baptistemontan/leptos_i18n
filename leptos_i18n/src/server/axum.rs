use crate::{locale_traits::*, COOKIE_PREFERED_LANG};
use axum::http::{header, HeaderValue};
use leptos::*;

pub fn fetch_locale_server<T: Locales>(cx: Scope) -> T::Variants {
    // when leptos_router inspect the routes it execute the code once but don't set a RequestParts in the context,
    // so we can't expect it to be present.
    use_context::<leptos_axum::RequestParts>(cx)
        .map(|req| from_req::<T>(&req))
        .unwrap_or_default()
}

fn from_req<T: Locales>(req: &leptos_axum::RequestParts) -> T::Variants {
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

    let langs = crate::accepted_lang::parse_header(header);

    LocaleVariant::find_locale(&langs)
}

fn get_prefered_lang_cookie<T: Locales>(req: &leptos_axum::RequestParts) -> Option<T::Variants> {
    req.headers
        .get_all(header::COOKIE)
        .into_iter()
        .filter_map(parse_cookie)
        .filter_map(LocaleVariant::from_str)
        .next()
}

fn parse_cookie(cookie: &HeaderValue) -> Option<&str> {
    std::str::from_utf8(cookie.as_bytes())
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .filter_map(|s| s.split_once('='))
        .find(|(name, _)| name == &COOKIE_PREFERED_LANG)
        .map(|(_, value)| value)
}
