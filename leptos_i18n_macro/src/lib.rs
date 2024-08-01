// #![deny(missing_docs)]
#![forbid(unsafe_code)]
// #![deny(warnings)]
#![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic, track_path))]
//! # About Leptos i18n macro
//!
//! This crate expose the utility macro for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

pub(crate) mod load_locales;
pub(crate) mod t_macro;
pub(crate) mod utils;

use t_macro::{InputType, OutputType};

// for deserializing the files custom deserialization is done,
// this is to use `serde::de::DeserializeSeed` to pass information on what locale or key we are currently at
// and give better information on what went wrong when an error is emitted.

/// Look for the configuration in the cargo manifest `Cargo.toml` at the root of the project and load the given locales.
///
/// It creates multiple types allowing to easily incorporate translations in you application such as:
///
/// - `Locale`: an enum representing the available locales of the application.
/// - `I18nKeys`: a struct representing the translation keys.
#[proc_macro]
pub fn load_locales(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match load_locales::load_locales() {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}

/// This is for a private use writing tests.
#[doc(hidden)]
#[proc_macro]
pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    load_locales::declare_locales::declare_locales(tokens)
}

/// Utility macro to easily put translation in your application.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// let i18n = use_i18n();
///
/// view! {
///     <p>{t!(i18n, $key)}</p>
///     <p>{t!(i18n, $key, $variable = $value, <$component> = |child| ... )}</p>
/// }
/// ```
///
/// # Notes
///
/// If your variable/component value is the same as the key, you remove the assignement, such that this:
///
/// ```rust, ignore
/// t!(i18n, $key, variable = variable, <component> = component, $other_key = $other_value, ..)
/// ```
///
/// can be shortened to:
///
/// ```rust, ignore
/// t!(i18n, $key, variable, <component>, $other_key = $other_value, ..)
/// ```
#[proc_macro]
pub fn t(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Context, OutputType::View)
}

/// Same as the `t!` macro but untracked.
#[proc_macro]
pub fn tu(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Untracked, OutputType::View)
}

/// Just like the `t!` macro but instead of taking `I18nContext` as the first argument it takes the desired locale.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// view! {
///     <p>{td!(Locale::en, $key)}</p>
///     <p>{td!(Locale::fr, $key, $variable = $value, <$component> = |child| ... )}</p>
/// }
/// ```
///
/// This let you use a specific locale regardless of the current one.
#[proc_macro]
pub fn td(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Locale, OutputType::View)
}

/// Just like the `t!` macro but return a `Cow<'static, str>` instead of a view.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// let i18n = use_i18n(); // locale = "en"
///
/// // click_count = "You clicked {{ count }} times"
///
/// assert_eq!(
///     t_string!(i18n, click_count, count = 10),
///     "You clicked 10 times"
/// )
///
/// assert_eq!(
///     t_string!(i18n, click_count, count = "a lot of"),
///     "You clicked a lot of times"
/// )
/// ```
#[proc_macro]
pub fn t_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Context, OutputType::String)
}

/// Same as the `t_string!` macro but untracked.
#[proc_macro]
pub fn tu_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Untracked, OutputType::String)
}

/// Just like the `t_string!` macro but return either a struct implementing `Display` or a `&'static str` instead of a `Cow<'static, str>`.
///
/// This is usefull if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// let i18n = use_i18n(); // locale = "en"
///
/// // click_count = "You clicked {{ count }} times"
/// let t = t_display!(i18n, click_count, count = 10); // this only return the builder, no work has been done.
///
/// assert_eq!(format!("before {t} after"), "before You clicked 10 times after");
///
/// let t_str = t.to_string(); // can call `to_string` as the value impl `Display`
///
/// assert_eq!(t_str, "You clicked 10 times");
/// ```
#[proc_macro]
pub fn t_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Context, OutputType::Display)
}

/// Same as the `t_display!` macro but untracked.
#[proc_macro]
pub fn tu_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Untracked, OutputType::Display)
}

/// Just like the `t_string!` macro but takes the `Locale` as an argument instead of the context.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// // click_count = "You clicked {{ count }} times"
/// assert_eq!(
///     td_string!(Locale::en, click_count, count = 10),
///     "You clicked 10 times"
/// )
///
/// assert_eq!(
///     td_string!(Locale::en, click_count, count = "a lot of"),
///     "You clicked a lot of times"
/// )
/// ```
#[proc_macro]
pub fn td_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Locale, OutputType::String)
}

/// Just like the `t_display!` macro but takes the `Locale` as an argument instead of the context.
///
/// This is usefull if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// // click_count = "You clicked {{ count }} times"
/// let t = td_display!(Locale::en, click_count, count = 10); // this only return the builder, no work has been done.
///
/// assert_eq!(format!("before {t} after"), "before You clicked 10 times after");
///
/// let t_str = t.to_string(); // can call `to_string` as the value impl `Display`
///
/// assert_eq!(t_str, "You clicked 10 times");
/// ```
#[proc_macro]
pub fn td_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Locale, OutputType::Display)
}

/// Like `use_i18n` but enable to scope the context:
///
/// Instead of
///
/// ```rust, ignore
/// let i18n = use_i18n;
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, ignore
/// let i18n = use_i18n_scoped!(namespace);
/// t!(i18n, subkeys.value);
/// ```
///
/// Or
///
/// ```rust, ignore
/// let i18n = use_i18n_scoped!(namespace.subkeys);
/// t!(i18n, value);
/// ```
///
/// This macro is the equivalent to do
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let i18n = scope_i18n!(i18n, namespace.sukeys);
/// ```
#[proc_macro]
pub fn use_i18n_scoped(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::scoped::use_i18n_scoped(tokens)
}

/// Scope a context to the given keys
///
/// Instead of
///
/// ```rust, ignore
/// let i18n = use_i18n;
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let namespace_i18n = scope_i18n!(i18n, namespace);
///
/// t!(namespace_i18n, subkeys.value);
///
/// let subkeys_i18n = scope_i18n!(namespace_i18n, subkeys);
/// //  subkeys_i18n = scope_i18n!(i18n, namespace.subkeys);

/// t!(subkeys_i18n, value);
/// ```
#[proc_macro]
pub fn scope_i18n(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::scoped::scope_i18n(tokens)
}

/// Scope a locale to the given keys
///
/// Instead of
///
/// ```rust, ignore
/// let i18n = use_i18n;
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let namespace_i18n = scope_i18n!(i18n, namespace);
///
/// t!(namespace_i18n, subkeys.value);
///
/// let subkeys_i18n = scope_i18n!(namespace_i18n, subkeys);
/// //  subkeys_i18n = scope_i18n!(i18n, namespace.subkeys);

/// t!(subkeys_i18n, value);
/// ```
#[proc_macro]
pub fn scope_locale(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::scoped::scope_locale(tokens)
}
