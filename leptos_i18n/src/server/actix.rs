pub const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";
use crate::locale_traits::*;
use actix_web::{http::header, FromRequest, ResponseError};
use leptos::*;
use std::{
    fmt::Display,
    future::{ready, Ready},
};

#[derive(serde::Deserialize)]
pub struct SetLocaleCookieParams {
    lang: String,
    origin: String,
}

pub async fn set_locale_cookie(
    params: actix_web::web::Query<SetLocaleCookieParams>,
) -> impl actix_web::Responder {
    use actix_web::cookie::*;

    let params = params.into_inner();
    let cookie = CookieBuilder::new(COOKIE_PREFERED_LANG, params.lang)
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::MAX)
        .path("/")
        .finish()
        .encoded()
        .to_string();

    let mut res = actix_web::HttpResponse::Found();
    res.append_header((header::SET_COOKIE, cookie));
    res.append_header((header::LOCATION, params.origin));

    res.finish()
}

pub async fn fetch_locale<T: Locales>(cx: Scope) -> Result<T::Variants, ServerFnError> {
    fn inner<T>() -> impl FnOnce(AcceptedLang<T::Variants>) -> T::Variants + Clone + 'static
    where
        T: Locales,
    {
        |accepted_lang| accepted_lang.0
    }

    let f = inner::<T>();
    let f = |selected_lang| async { f(selected_lang) };

    leptos_actix::extract(cx, f).await
}

pub struct AcceptedLang<T: LocaleVariant>(pub T);

impl<T: LocaleVariant> Default for AcceptedLang<T> {
    fn default() -> Self {
        AcceptedLang(Default::default())
    }
}

impl<T: LocaleVariant> AcceptedLang<T> {}

impl<T: LocaleVariant> FromRequest for AcceptedLang<T> {
    type Error = Impossible;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let prefered_lang = req
            .cookie(COOKIE_PREFERED_LANG)
            .and_then(|ck| T::from_str(ck.value()));

        if let Some(pref) = prefered_lang {
            return ready(Ok(AcceptedLang(pref)));
        }

        let Some(header) = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|header| header.to_str().ok())
        else {
            return ready(Ok(AcceptedLang::default()));
        };

        let langs = crate::accepted_lang::parse_header(header);

        ready(Ok(AcceptedLang(LocaleVariant::find_locale(&langs))))
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
