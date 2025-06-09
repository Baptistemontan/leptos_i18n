use leptos_i18n_parser::parse_locales::error::Result;
use proc_macro2::TokenStream;

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
    let parsed_locales =
        leptos_i18n_parser::parse_locales::parse_locales(None, Default::default())?;

    leptos_i18n_codegen::gen_code(&parsed_locales, None)
}
