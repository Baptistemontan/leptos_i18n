use crate::locale_traits::*;
use axum::http::header;
use leptos::*;

pub fn fetch_locale_server<T: Locales>(cx: Scope) -> T::Variants {
    // when leptos_router inspect the routes it execute the code once but don't set a RequestParts in the context,
    // so we can't expect it to be present.
    use_context::<leptos_axum::RequestParts>(cx)
        .map(|req| from_req::<T>(&req))
        .unwrap_or_default()
}

fn from_req<T: Locales>(req: &leptos_axum::RequestParts) -> T::Variants {
    // TODO: read cookie like in the actix version

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
