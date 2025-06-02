#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
#![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic, track_path))]
//! # About Leptos i18n codegen
//!
//! This crate expose the codegen functions for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.

use leptos_i18n_parser::parse_locales::{
    cfg_file::ConfigFile,
    error::{Errors, Result},
    locale::LocalesOrNamespaces,
    warning::Warnings,
    ForeignKeysPaths,
};

pub mod load_locales;
pub mod utils;

pub fn load_locales(
    crate_path: &syn::Path,
    cfg_file: &ConfigFile,
    locales: LocalesOrNamespaces,
    foreign_keys_paths: ForeignKeysPaths,
    warnings: Warnings,
    errors: Errors,
    tracked_files: Option<Vec<String>>,
    interpolate_display: bool,
) -> Result<proc_macro2::TokenStream> {
    load_locales::load_locales(
        crate_path,
        cfg_file,
        locales,
        foreign_keys_paths,
        warnings,
        errors,
        tracked_files,
        interpolate_display,
    )
}
