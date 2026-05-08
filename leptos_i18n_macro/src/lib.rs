#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
// #![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic, track_path))]
//! # About Leptos i18n macro
//!
//! This crate expose the utility macro for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

mod build_key;
mod data_provider;
pub(crate) mod declare_locales;
pub(crate) mod t_format;
pub(crate) mod t_plural;

use t_plural::PluralRuleType;

#[proc_macro]
pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    declare_locales::declare_locales(tokens)
}

#[proc_macro]
pub fn build_key_inner(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    build_key::build_key_macro(tokens)
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
