use leptos::prelude::*;
use leptos_use::UseLocalesOptions;

use crate::Locale;

pub fn fetch_locale<L: Locale>(current_cookie: Option<L>, options: UseLocalesOptions) -> Memo<L> {
    let accepted_locales = leptos_use::use_locales_with_options(options);
    let accepted_locale =
        Memo::new(move |_| accepted_locales.with(|accepted| L::find_locale(accepted)));

    if cfg!(feature = "ssr") {
        fetch_locale_ssr(current_cookie, accepted_locale)
    } else if cfg!(feature = "hydrate") {
        fetch_locale_hydrate(current_cookie, accepted_locale)
    } else {
        fetch_locale_csr(current_cookie, accepted_locale)
    }
}

pub fn get_accepted_locale<L: Locale>(options: UseLocalesOptions) -> L {
    leptos_use::use_locales_with_options(options)
        .with_untracked(|accepted| L::find_locale(accepted))
}

pub fn resolve_locale<L: Locale>(current_cookie: Option<L>, options: UseLocalesOptions) -> L {
    cfg!(feature = "hydrate")
        .then(get_locale_from_html)
        .flatten()
        .or(current_cookie)
        .unwrap_or_else(move || get_accepted_locale(options))
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
fn fetch_locale_ssr<L: Locale>(current_cookie: Option<L>, accepted_locale: Memo<L>) -> Memo<L> {
    signal_maybe_once_then(current_cookie, accepted_locale)
}

fn get_locale_from_html<L: Locale>() -> Option<L> {
    leptos::prelude::document()
        .document_element()
        .and_then(|el| match el.get_attribute("lang") {
            None => {
                leptos::logging::debug_warn!("No \"lang\" attribute found on <html> element. With hydrate this attribute must be set to the used locale as it is used to determine what locale the server has choosen. Either use the `set_lang_attr_on_html` props on <I18nContextProvider> or manually set it.");
                None
            }
            Some(lang) => Some(lang),
        })
        .and_then(|lang| L::from_str(&lang).ok())
}

// hydrate fetch
fn fetch_locale_hydrate<L: Locale>(current_cookie: Option<L>, accepted_locale: Memo<L>) -> Memo<L> {
    let base_locale = get_locale_from_html().or(current_cookie);

    signal_maybe_once_then(base_locale, accepted_locale)
}

// csr fetch
fn fetch_locale_csr<L: Locale>(current_cookie: Option<L>, accepted_locale: Memo<L>) -> Memo<L> {
    signal_maybe_once_then(current_cookie, accepted_locale)
}
