#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
#![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic, track_path))]
//! # About Leptos i18n codegen
//!
//! This crate expose the codegen functions for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

#[cfg(feature = "proc_macro")]
extern crate proc_macro;

use leptos_i18n_parser::parse_locales::{error::Result, ParsedLocales};

pub mod load_locales;
pub mod utils;

pub fn gen_code(
    parsed_locales: &ParsedLocales,
    crate_path: &syn::Path,
    interpolate_display: bool,
) -> Result<proc_macro2::TokenStream> {
    load_locales::load_locales(&parsed_locales, crate_path, interpolate_display)
}
