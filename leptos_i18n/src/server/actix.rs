use crate::locale_traits::*;
use actix_web::{http::header, FromRequest, ResponseError};
use leptos::*;
use std::{
    fmt::Display,
    future::{ready, Ready},
};

use crate::COOKIE_PREFERED_LANG;

pub fn fetch_locale_server<T: Locales>(cx: Scope) -> T::Variants {
    // when leptos_router inspect the routes it execute the code once but don't set an HttpRequest in the context,
    // so we can't expect it to be present.
    use_context::<actix_web::HttpRequest>(cx)
        .map(|req| AcceptedLang::from_req(&req).0)
        .unwrap_or_default()
}

pub struct AcceptedLang<T: LocaleVariant>(pub T);

impl<T: LocaleVariant> Default for AcceptedLang<T> {
    fn default() -> Self {
        AcceptedLang(Default::default())
    }
}

impl<T: LocaleVariant> AcceptedLang<T> {
    fn from_req(req: &actix_web::HttpRequest) -> Self {
        let prefered_lang = req
            .cookie(COOKIE_PREFERED_LANG)
            .and_then(|ck| T::from_str(ck.value()));

        if let Some(pref) = prefered_lang {
            return AcceptedLang(pref);
        }

        let Some(header) = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|header| header.to_str().ok())
        else {
            return AcceptedLang::default();
        };

        let langs = crate::accepted_lang::parse_header(header);

        AcceptedLang(LocaleVariant::find_locale(&langs))
    }
}

impl<T: LocaleVariant> FromRequest for AcceptedLang<T> {
    type Error = Impossible;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(Self::from_req(req)))
    }
}

#[derive(Debug)]
pub enum Impossible {}

impl Display for Impossible {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl ResponseError for Impossible {}
