#![forbid(unsafe_code)]
// #![deny(warnings)]
#![allow(clippy::too_many_arguments)]
//! # About Leptos i18n codegen
//!
//! This crate expose the codegen functions for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

use leptos_i18n_parser::{error::Result, extraction::ParsedLocales};

use proc_macro2::TokenStream;

mod codegen;
// pub mod load_locales;
mod options;
pub mod utils;

pub use options::CodegenOptions;

pub fn gen_code(parsed_locales: &ParsedLocales, options: CodegenOptions) -> Result<TokenStream> {
    codegen::gen_code(parsed_locales, options)
}
