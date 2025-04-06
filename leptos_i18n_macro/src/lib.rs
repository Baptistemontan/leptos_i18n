#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
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

#[proc_macro]
pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    load_locales::declare_locales::declare_locales(tokens)
}

#[proc_macro]
pub fn t(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Context, OutputType::View)
}

#[proc_macro]
pub fn tu(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Untracked, OutputType::View)
}

#[proc_macro]
pub fn td(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Locale, OutputType::View)
}

#[proc_macro]
pub fn t_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Context, OutputType::String)
}

#[proc_macro]
pub fn tu_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Untracked, OutputType::String)
}

#[proc_macro]
pub fn t_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Context, OutputType::Display)
}

#[proc_macro]
pub fn tu_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Untracked, OutputType::Display)
}

#[proc_macro]
pub fn td_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Locale, OutputType::String)
}

#[proc_macro]
pub fn td_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens, InputType::Locale, OutputType::Display)
}

#[proc_macro]
pub fn use_i18n_scoped(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::scoped::use_i18n_scoped(tokens)
}

#[proc_macro]
pub fn scope_i18n(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::scoped::scope_i18n(tokens)
}

#[proc_macro]
pub fn scope_locale(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::scoped::scope_locale(tokens)
}

#[proc_macro]
pub fn t_format(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Context,
        t_format::OutputType::View,
    )
}

#[proc_macro]
pub fn tu_format(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Untracked,
        t_format::OutputType::View,
    )
}

#[proc_macro]
pub fn td_format(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Locale,
        t_format::OutputType::View,
    )
}

#[proc_macro]
pub fn t_format_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Context,
        t_format::OutputType::String,
    )
}

#[proc_macro]
pub fn tu_format_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Untracked,
        t_format::OutputType::String,
    )
}

#[proc_macro]
pub fn td_format_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Locale,
        t_format::OutputType::String,
    )
}

#[proc_macro]
pub fn t_format_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Context,
        t_format::OutputType::Display,
    )
}

#[proc_macro]
pub fn tu_format_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Untracked,
        t_format::OutputType::Display,
    )
}

#[proc_macro]
pub fn td_format_display(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_format::t_format(
        tokens,
        t_format::InputType::Locale,
        t_format::OutputType::Display,
    )
}

#[proc_macro]
pub fn t_plural(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Context,
        PluralRuleType::Cardinal,
    )
}

#[proc_macro]
pub fn tu_plural(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Untracked,
        PluralRuleType::Cardinal,
    )
}

#[proc_macro]
pub fn td_plural(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Locale,
        PluralRuleType::Cardinal,
    )
}

#[proc_macro]
pub fn t_plural_ordinal(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Context,
        PluralRuleType::Ordinal,
    )
}

#[proc_macro]
pub fn tu_plural_ordinal(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(
        tokens,
        t_plural::InputType::Untracked,
        PluralRuleType::Ordinal,
    )
}

#[proc_macro]
pub fn td_plural_ordinal(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_plural::t_plural(tokens, t_plural::InputType::Locale, PluralRuleType::Ordinal)
}

/// Derive the `IcuDataProvider` trait
#[proc_macro_derive(IcuDataProvider)]
pub fn derive_icu_data_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    data_provider::derive_icu_data_provider(input)
}
