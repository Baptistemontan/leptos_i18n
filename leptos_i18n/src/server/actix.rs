use crate::locale_traits::*;
use actix_web::http::header;
use leptos::*;

pub fn fetch_locale_server<T: Locales>(cx: Scope) -> T::Variants {
    // when leptos_router inspect the routes it execute the code once but don't set an HttpRequest in the context,
    // so we can't expect it to be present.
    use_context::<actix_web::HttpRequest>(cx)
        .map(|req| from_req(&req))
        .unwrap_or_default()
}

fn from_req<T: LocaleVariant>(req: &actix_web::HttpRequest) -> T {
    #[cfg(feature = "cookie")]
    if let Some(pref) = req
        .cookie(crate::COOKIE_PREFERED_LANG)
        .and_then(|ck| T::from_str(ck.value()))
    {
        return pref;
    }

    let Some(header) = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|header| header.to_str().ok())
    else {
        return Default::default();
    };

    let langs = super::parse_header(header);

    LocaleVariant::find_locale(&langs)
}
