use std::borrow::Cow;

use leptos::prelude::*;
use leptos_router::location::{BrowserUrl, LocationProvider, RequestUrl};
use leptos_use::UseLocalesOptions;

use crate::Locale;

pub fn fetch_locale<L: Locale>(
    current_cookie: Option<L>,
    options: UseLocalesOptions,
    parse_locale_from_path: Option<Cow<'static, str>>,
) -> Memo<L> {
    let accepted_locales = leptos_use::use_locales_with_options(options);
    let accepted_locale =
        Memo::new(move |_| accepted_locales.with(|accepted| L::find_locale(accepted)));

    let url_locale = get_locale_from_path::<L>(parse_locale_from_path);

    if cfg!(feature = "ssr") {
        fetch_locale_ssr(current_cookie, accepted_locale, url_locale)
    } else if cfg!(feature = "hydrate") {
        fetch_locale_hydrate(current_cookie, accepted_locale)
    } else {
        fetch_locale_csr(current_cookie, accepted_locale, url_locale)
    }
}

fn get_locale_from_path<L: Locale>(parse_locale_from_path: Option<Cow<'static, str>>) -> Option<L> {
    let base_path = parse_locale_from_path?;
    let url = if cfg!(feature = "ssr") {
        let req = use_context::<RequestUrl>().expect("no RequestUrl provided");
        req.parse().expect("could not parse RequestUrl")
    } else {
        let location = BrowserUrl::new().expect("could not access browser navigation");
        location.as_url().get_untracked()
    };

    crate::routing::get_locale_from_path(url.path(), &base_path)
}

pub fn signal_once_then<T: Clone + PartialEq + Send + Sync + 'static>(
    start: T,
    then: Memo<T>,
) -> Memo<T> {
    Memo::new(move |init| {
        let then = then.get();
        if init.is_none() {
            start.clone()
        } else {
            then
        }
    })
}

pub fn signal_maybe_once_then<T: Clone + PartialEq + Send + Sync + 'static>(
    start: Option<T>,
    then: Memo<T>,
) -> Memo<T> {
    match start {
        Some(start) => signal_once_then(start, then),
        None => then,
    }
}

// ssr fetch
fn fetch_locale_ssr<L: Locale>(
    current_cookie: Option<L>,
    accepted_locale: Memo<L>,
    url_locale: Option<L>,
) -> Memo<L> {
    signal_maybe_once_then(url_locale.or(current_cookie), accepted_locale)
}

// hydrate fetch
fn fetch_locale_hydrate<L: Locale>(current_cookie: Option<L>, accepted_locale: Memo<L>) -> Memo<L> {
    let base_locale = leptos::prelude::document()
        .document_element()
        .and_then(|el| match el.get_attribute("lang") {
            None => {
                leptos::logging::debug_warn!("No \"lang\" attribute found on <html> element. With hydrate this attribute must be set to the used locale as it is used to determine what locale the server has choosen. Either use the `set_lang_attr_on_html` props on <I18nContextProvider> or manually set it.");
                None
            }
            Some(lang) => Some(lang),
        })
        .and_then(|lang| L::from_str(&lang).ok())
        .or(current_cookie);

    signal_maybe_once_then(base_locale, accepted_locale)
}

// csr fetch
fn fetch_locale_csr<L: Locale>(
    current_cookie: Option<L>,
    accepted_locale: Memo<L>,
    url_locale: Option<L>,
) -> Memo<L> {
    signal_maybe_once_then(url_locale.or(current_cookie), accepted_locale)
}
