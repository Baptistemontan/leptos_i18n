#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
#![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic, track_path))]
//! # About Leptos i18n macro
//!
//! This crate expose the utility macro for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

mod data_provider;
pub(crate) mod load_locales;
pub(crate) mod t_format;
pub(crate) mod t_macro;
pub(crate) mod t_plural;
pub(crate) mod utils;

use load_locales::plurals::PluralRuleType;
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
        Err(err) => {
            let err = err.to_string();
            quote::quote!(compile_error!(#err);).into()
        }
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
/// If your variable/component value is the same as the key, you remove the assignment, such that this:
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
/// This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
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
/// This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
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
/// let i18n = use_i18n();
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
/// let i18n = scope_i18n!(i18n, namespace.subkeys);
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
/// let i18n = use_i18n();
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

/// Format a given value with a given formatter and return:
///
/// ```rust, ignore
/// let i18n =  use_i18n();
/// let num = 100_000usize;
///
/// t_format!(i18n, num, formatter: number);
///
/// let list = || ["A", "B", "C"];
///
/// t_format!(i18n, list, formatter: list(list_type: and; list_style: wide));
/// ```
/// This function does exactly the same as if you had "{{ var, formatter_name(formatter_arg: value; ...) }}"
/// for a translation and do
///
/// ```rust,ignore
/// t!(i18n, key, var = ...)
/// ```
#[proc_macro]
pub fn t_format(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Context,
        t_format::OutputType::View,
    )
}

/// Same as the `t_format!` macro but untracked.
#[proc_macro]
pub fn tu_format(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Untracked,
        t_format::OutputType::View,
    )
}

/// Same as the `t_format!` macro but takes the desired `Locale` as the first argument.
#[proc_macro]
pub fn td_format(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Locale,
        t_format::OutputType::View,
    )
}

/// Format a given value with a given formatter and return a `String`:
///
/// ```rust, ignore
/// let i18n =  use_i18n();
/// let num = 100_000usize;
///
/// t_format_string!(i18n, num, formatter: number);
///
/// let list = || ["A", "B", "C"];
///
/// t_format_string!(i18n, list, formatter: list(list_type: and; list_style: wide));
/// ```
/// This function does exactly the same as if you had "{{ var, formatter_name(formatter_arg: value; ...) }}"
/// for a translation and do
///
/// ```rust,ignore
/// t_string!(i18n, key, var = ...)
/// ```
#[proc_macro]
pub fn t_format_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Context,
        t_format::OutputType::String,
    )
}

/// Same as the `t_format_string!` macro but untracked.
#[proc_macro]
pub fn tu_format_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Untracked,
        t_format::OutputType::String,
    )
}

/// Same as the `t_format_string!` macro but takes the desired `Locale` as the first argument.
#[proc_macro]
pub fn td_format_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Locale,
        t_format::OutputType::String,
    )
}

/// Format a given value with a given formatter and return a `impl Display`:
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let num = 100_000usize;
///
/// t_format_display!(i18n, num, formatter: number);
///
/// let list = || ["A", "B", "C"];
///
/// t_format_display!(i18n, list, formatter: list(list_type: and; list_style: wide));
/// ```
/// This function does exactly the same as if you had "{{ var, formatter_name(formatter_arg: value; ...) }}"
/// for a translation and do
///
/// ```rust,ignore
/// t_display!(i18n, key, var = ...)
/// ```
#[proc_macro]
pub fn t_format_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Context,
        t_format::OutputType::Display,
    )
}

/// Same as the `t_format_display!` macro but untracked.
#[proc_macro]
pub fn tu_format_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Untracked,
        t_format::OutputType::Display,
    )
}

/// Same as the `t_format_display!` macro but takes the desired `Locale` as the first argument.
#[proc_macro]
pub fn td_format_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Locale,
        t_format::OutputType::Display,
    )
}

/// Match against the plural form of a given count:
///
/// ```rust, ignore
/// let i18n = use_i18n();
///
/// let form = t_plural! {
///     i18n,
///     count = || 0,
///     one => "one",
///     _ => "other"
/// };
///
/// Effect::new(|| {
///     let s = form();
///     log!("{}", s);
/// })
/// ```
///
/// This will print "one" with locale "fr" but "other" with locale "en".
///
/// Accepted forms are: `zero`, `one`, `two`, `few`, `many`, `other` and `_`.
///
/// This is for the cardinal form of plurals, for ordinal form see `t_plural_ordinal!`.
#[proc_macro]
pub fn t_plural(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Context,
        PluralRuleType::Cardinal,
    )
}

/// Same as the `t_plural!` macro but untracked.
/// Directly return the value instead of wrapping it in a closure.
#[proc_macro]
pub fn tu_plural(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Untracked,
        PluralRuleType::Cardinal,
    )
}

/// Same as the `t_plural!` macro but takes the desired `Locale` as the first argument.
/// Directly return the value instead of wrapping it in a closure.
#[proc_macro]
pub fn td_plural(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Locale,
        PluralRuleType::Cardinal,
    )
}

/// Match against the plural form of a given count:
///
/// ```rust, ignore
/// let i18n = use_i18n();
///
/// let form = t_plural! {
///     i18n,
///     count = || 2,
///     two => "two",
///     _ => "other"
/// };
///
/// Effect::new(|| {
///     let s = form();
///     log!("{}", s);
/// })
/// ```
///
/// This will print "other" with locale "fr" but "two" with locale "en".
///
/// Accepted forms are: `zero`, `one`, `two`, `few`, `many`, `other` and `_`.
///
/// This is for the ordinal form of plurals, for cardinal form see `t_plural!`.
#[proc_macro]
pub fn t_plural_ordinal(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Context,
        PluralRuleType::Ordinal,
    )
}

/// Same as the `t_plural_ordinal!` macro but untracked.
/// Directly return the value instead of wrapping it in a closure.
#[proc_macro]
pub fn tu_plural_ordinal(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Untracked,
        PluralRuleType::Ordinal,
    )
}

/// Same as the `t_plural_ordinal!` macro but takes the desired `Locale` as the first argument.
/// Directly return the value instead of wrapping it in a closure.
#[proc_macro]
pub fn td_plural_ordinal(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(tokens, t_plural::InputType::Locale, PluralRuleType::Ordinal)
}

/// Derive the `IcuDataProvider` trait
#[proc_macro_derive(IcuDataProvider)]
pub fn derive_icu_data_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    data_provider::derive_icu_data_provider(input)
}
