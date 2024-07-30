use crate::locale_traits::*;
use actix_web::http::header;

pub fn fetch_locale_server<T: Locale>(current_cookie: Option<T>) -> T {
    if let Some(lang) = current_cookie {
        return lang;
    }

    // when leptos_router inspect the routes it execute the code once but don't set an HttpRequest in the context,
    // so we can't expect it to be present.
    leptos::use_context::<actix_web::HttpRequest>()
        .map(|req| from_req(&req))
        .unwrap_or_default()
}

fn from_req<T: Locale>(req: &actix_web::HttpRequest) -> T {
    let Some(header) = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|header| header.to_str().ok())
    else {
        return Default::default();
    };

    let langs = super::parse_header(header);

    T::find_locale(&langs)
}
