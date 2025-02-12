//! Contain utilities for locales

use codee::string::FromToStringCodec;
use leptos::prelude::*;

use crate::{
    context::{I18nContextOptions, ENABLE_COOKIE},
    fetch_locale, Locale,
};

/// Same as `resolve_locale` but with some cookies options.
pub fn resolve_locale_with_options<L: Locale>(options: I18nContextOptions<L>) -> L {
    let I18nContextOptions {
        enable_cookie,
        cookie_name,
        cookie_options,
        ssr_lang_header_getter,
    } = options;
    let (lang_cookie, _) = if ENABLE_COOKIE && enable_cookie {
        leptos_use::use_cookie_with_options::<L, FromToStringCodec>(&cookie_name, cookie_options)
    } else {
        let (lang_cookie, set_lang_cookie) = signal(None);
        (lang_cookie.into(), set_lang_cookie)
    };
    fetch_locale::resolve_locale(lang_cookie.get_untracked(), ssr_lang_header_getter)
}

/// Resolve the locale.
///
/// This as the same behavior as calling `init_i18n_context().get_locale_untracked()`.
///
/// This function primary usage is to access a user locale in a server function, but is not constrained to it.
///
/// Here is the list of detection methods, sorted in priorities:
/// 1. The "lang" attribute is set on the `<html>` element in hydrate
/// 1. A cookie is present that contains a previously detected locale
/// 1. A locale can be matched based on the [`Accept-Language` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language) in SSR
/// 1. A locale can be matched based on the [`navigator.languages` API](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/languages) in CSR
/// 1. As a last resort, the default locale is used.
///
/// *note*: this function does not take into account URL locale prefix when using `I18nRoute` (e.g. `/en/about`)
pub fn resolve_locale<L: Locale>() -> L {
    resolve_locale_with_options(Default::default())
}
