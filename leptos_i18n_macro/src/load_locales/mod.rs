use leptos_i18n_parser::parse_locales::error::Result;
use leptos_i18n_parser::parse_locales::RawParsedLocales;
use proc_macro2::{Span, TokenStream};

pub mod declare_locales;

/// Steps:
///
/// 1: Locate and parse the manifest (`ConfigFile::new`)
/// 2: parse each locales/namespaces files (`LocalesOrNamespaces::new`)
/// 3: Resolve foreign keys (`ParsedValue::resolve_foreign_keys`)
/// 4: check the locales: (`Locale::check_locales`)
/// 4.1: get interpolations keys of the default, meaning all variables/components/ranges of the default locale (`Locale::make_builder_keys`)
/// 4.2: in the process reduce all values and check for default in the default locale
/// 4.3: then merge all other locales in the default locale keys, reducing all values in the process (`Locale::merge`)
/// 4.4: discard any surplus key and emit a warning
/// 5: generate code (and warnings)
pub fn load_locales() -> Result<TokenStream> {
    let RawParsedLocales {
        locales,
        cfg_file,
        foreign_keys_paths,
        warnings,
        tracked_files,
        errors,
    } = leptos_i18n_parser::parse_locales::parse_locales_raw(None)?;

    let crate_path = syn::Path::from(syn::Ident::new("leptos_i18n", Span::call_site()));

    let interpolate_display = cfg!(feature = "interpolate_display");

    leptos_i18n_codegen::load_locales(
        &crate_path,
        &cfg_file,
        locales,
        foreign_keys_paths,
        warnings,
        errors,
        Some(tracked_files),
        interpolate_display,
    )
}
