//! Contain utilities for locales

use crate::{
    context::{use_cookie, I18nContextOptions},
    fetch_locale, Locale,
};

/// Same as `resolve_locale` but with some cookies options.
pub fn resolve_locale_with_options<L: Locale>(options: I18nContextOptions<L>) -> L {
    let I18nContextOptions {
        enable_cookie,
        cookie_name,
        cookie_options,
        ssr_lang_header_getter,
        skip_locale_resolution,
    } = options;

    if skip_locale_resolution {
        return L::default();
    }

    let cookie_name = enable_cookie.then_some(cookie_name);
    let (lang_cookie, _) = use_cookie(cookie_name.as_deref(), cookie_options);
    fetch_locale::resolve_locale(lang_cookie, ssr_lang_header_getter)
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
