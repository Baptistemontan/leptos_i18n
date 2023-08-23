#![deny(missing_docs)]
#![deny(unsafe_code)]
//! # About Leptos i18n macro
//!
//! This crate expose the utility macro for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

pub(crate) mod load_locales;
pub(crate) mod t_macro;

// for deserializing the files custom deserialization is done,
// this is to use serde::de::DeserializeSeed to pass information on what locale or key we are currently at
// and give better information on what went wrong when an error is emitted.

/// Look at the `i18n.json` configuration file at the root of the project and load the given locales.
///
/// It creates multiple types allowing to easily incorporate translations in you application such as:
///
/// - `LocaleEnum`: an enum representing the available locales of the application.
/// - `I18nKeys`: a struct representing the translation keys.
/// - `Locales`: an empty type that serves as a bridge beetween the two types.
#[proc_macro]
pub fn load_locales(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match load_locales::load_locales(None::<String>) {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}

/// Utility macro to easily put translation in your application.
///
/// Usage:
///
/// ```rust
/// let i18n = get_i18n_context(cx);
///
/// view! { cx,
///     <p>{t!(i18n, $key)}</p>
///     <p>{t!(i18n, $key, $variable = $value, <$component> = |cx, children| view! { cx, <b>{childent(cx)}</b> })}</p>
/// }
///```
///
/// # Notes
///
/// If your variable/component value is the same as the key, you remove the assignement, such that this:
///
/// ```rust
/// t!(i18n, $key, variable = variable, <component> = component, $other_key = $other_value, ..)
/// ```
///
/// can be shortened to:
///
/// ```rust
/// t!(i18n, $key, variable, <component>, $other_key = $other_value, ..)
/// ```
#[proc_macro]
pub fn t(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens)
}
