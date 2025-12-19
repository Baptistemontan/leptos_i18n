#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
//! # About Leptos i18n codegen
//!
//! This crate expose the codegen functions for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

use leptos_i18n_parser::parse_locales::{error::Result, ParsedLocales};
use proc_macro2::TokenStream;

pub mod load_locales;
pub mod utils;

pub fn gen_code(
    parsed_locales: &ParsedLocales,
    crate_path: Option<&syn::Path>,
    emit_diagnostics: bool,
    top_level_attributes: Option<&TokenStream>,
) -> Result<TokenStream> {
    load_locales::load_locales(
        parsed_locales,
        crate_path,
        emit_diagnostics,
        top_level_attributes,
    )
}
