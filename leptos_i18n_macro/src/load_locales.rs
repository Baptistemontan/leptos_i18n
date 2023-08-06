use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    ConfigFileNotFound(std::io::Error),
    ConfigFileDeser(serde_json::Error),
    ConfigFileDefaultMissing(ConfigFile),
    LocaleFileNotFound(String, std::io::Error),
    LocaleFileDeser(String, serde_json::Error),
    MissingKeysInLocale { keys: Vec<String>, locale: String },
    InvalidKeys(Vec<String>),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::ConfigFileNotFound(err) => {
                format!("Could not found configuration file (i18n.json) : {}", err)
            }
            Error::ConfigFileDeser(err) => {
                format!("Parsing of configuration file (i18n.json) failed: {}", err)
            }
            Error::ConfigFileDefaultMissing(cfg_file) => format!(
                "{:?} is set as default locale but is not in the locales list: {:?}",
                cfg_file.default, cfg_file.locales
            ),
            Error::LocaleFileNotFound(locale_name, err) => {
                format!(
                    "Could not found locale file \"{}.json\" : {}",
                    locale_name, err
                )
            }
            Error::LocaleFileDeser(locale_name, err) => format!(
                "Parsing of locale file \"{}.json\" failed: {}",
                locale_name, err
            ),
            Error::MissingKeysInLocale { keys, locale } => format!(
                "Some keys are different beetween locale files, \"{}.json\" is missing keys: {:?}",
                locale, keys
            ),
            Error::InvalidKeys(keys) => {
                format!("Some keys are invalid to be used as field name: {:?}", keys)
            }
        }
    }
}

impl From<Error> for proc_macro::TokenStream {
    fn from(value: Error) -> Self {
        let error = value.to_string();
        quote!(compile_error!(#error)).into()
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    default: String,
    locales: Vec<String>,
}

pub fn load_locales_inner(cfg_file_path: Option<impl AsRef<Path>>) -> Result<TokenStream> {
    let cfg_file = get_cfg_file(cfg_file_path)?;

    let mut raw_locales = Vec::with_capacity(cfg_file.locales.len());

    for locale in &cfg_file.locales {
        raw_locales.push(get_locale(format!("./locales/{}.json", locale), locale)?);
    }

    let keys = check_keys(&cfg_file, &raw_locales)?;

    let locale_type = create_locale_type(&cfg_file, &keys, &raw_locales);
    let locale_variants = create_locales_enum(&cfg_file);
    let locales = create_locales_type(&cfg_file);

    Ok(quote! {
        #locale_type

        #locale_variants

        #locales
    })
}

fn create_locales_enum(cfg_file: &ConfigFile) -> TokenStream {
    let variants = cfg_file
        .locales
        .iter()
        .map(|locale| syn::Ident::new(locale, Span::call_site()))
        .collect::<Vec<_>>();
    let default = syn::Ident::new(&cfg_file.default, Span::call_site());

    let as_str_match_arms = variants
        .iter()
        .zip(&cfg_file.locales)
        .map(|(variant, locale)| quote!(LocaleEnum::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = variants
        .iter()
        .zip(&cfg_file.locales)
        .map(|(variant, locale)| quote!(#locale => Some(LocaleEnum::#variant)))
        .collect::<Vec<_>>();

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        #[allow(non_camel_case_types)]
        pub enum LocaleEnum {
            #(#variants,)*
        }

        impl Default for LocaleEnum {
            fn default() -> Self {
                LocaleEnum::#default
            }
        }

        impl ::leptos_i18n::LocaleVariant for LocaleEnum {
            fn as_str(&self) -> &'static str {
                match *self {
                    #(#as_str_match_arms,)*
                }
            }
            fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_match_arms,)*
                    _ => None
                }
            }
        }
    }
}

fn create_locales_type(_cfg_file: &ConfigFile) -> TokenStream {
    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub struct Locales;

        impl ::leptos_i18n::Locales for Locales {
            type Variants = LocaleEnum;
            type LocaleKeys = I18nKeys;
        }
    }
}

fn create_locale_type(
    cfg_file: &ConfigFile,
    keys: &HashSet<&str>,
    raw_locales: &[HashMap<String, String>],
) -> TokenStream {
    let fields = keys
        .iter()
        .map(|key| {
            let key = syn::Ident::new(key, Span::call_site());
            quote!(pub #key: &'static str)
        })
        .collect::<Vec<_>>();

    let filled_locale_fields: Vec<Vec<TokenStream>> = raw_locales
        .iter()
        .map(|entries| {
            entries
                .iter()
                .map(|(key, value)| {
                    let key = syn::Ident::new(key, Span::call_site());
                    quote!(#key: #value)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let from_variant_match_arms =
        cfg_file
            .locales
            .iter()
            .zip(filled_locale_fields)
            .map(|(locale_name, filled_fields)| {
                let locale_name = syn::Ident::new(locale_name, Span::call_site());
                quote! {
                    LocaleEnum::#locale_name => I18nKeys {
                        #(#filled_fields,)*
                    }
                }
            });

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_snake_case)]
        pub struct I18nKeys {
            #(#fields,)*
        }

        impl ::leptos_i18n::LocaleKeys for I18nKeys {
            type Locales = Locales;
            fn from_variant(variant: LocaleEnum) -> Self {
                match variant {
                    #(
                        #from_variant_match_arms,
                    )*
                }
            }

        }
    }
}

fn check_keys<'a>(
    cfg_file: &ConfigFile,
    raw_locales: &'a [HashMap<String, String>],
) -> Result<HashSet<&'a str>> {
    // locales is non empty, as default need to exist and locales to contain default, so len >= 1
    let first_locale = raw_locales.first().unwrap();
    let first_locale_name = cfg_file.locales.first().unwrap();

    let keys: HashSet<&str> = first_locale.keys().map(String::as_str).collect();

    // this check if all locales have the same keys, no more no less.
    for (raw_locale, locale_name) in raw_locales.iter().zip(&cfg_file.locales).skip(1) {
        let mut count = 0;
        let mut missing_keys = Vec::new();
        // check if all keys in first locale are in this locale
        for key in raw_locale.keys() {
            count += 1;
            if !keys.contains(key.as_str()) {
                missing_keys.push(key.to_owned());
            }
        }
        if !missing_keys.is_empty() {
            // missing key in first locale
            return Err(Error::MissingKeysInLocale {
                keys: missing_keys,
                locale: first_locale_name.to_owned(),
            });
        }
        // missing key in current locale
        if count < keys.len() {
            missing_keys.reserve(keys.len() - count);
            for key in keys {
                if !raw_locale.contains_key(key) {
                    missing_keys.push(key.to_owned());
                }
            }
            return Err(Error::MissingKeysInLocale {
                keys: missing_keys,
                locale: locale_name.to_owned(),
            });
        }
    }

    // check if keys are valid to be a field name
    let invalid_keys: Vec<String> = keys
        .iter()
        .filter(|key| !check_if_key_valid_for_field_name(key))
        .copied()
        .map(String::from)
        .collect();

    if !invalid_keys.is_empty() {
        return Err(Error::InvalidKeys(invalid_keys));
    }

    Ok(keys)
}

fn check_if_key_valid_for_field_name(key: &str) -> bool {
    syn::parse_str::<syn::Ident>(key).is_ok()
}

fn get_cfg_file<T: AsRef<Path>>(path: Option<T>) -> Result<ConfigFile> {
    let path = path
        .as_ref()
        .map(|path| path.as_ref())
        .unwrap_or("./i18n.json".as_ref());
    let cfg_file = File::open(path).map_err(Error::ConfigFileNotFound)?;

    let cfg: ConfigFile = serde_json::from_reader(cfg_file).map_err(Error::ConfigFileDeser)?;

    if cfg.locales.contains(&cfg.default) {
        Ok(cfg)
    } else {
        Err(Error::ConfigFileDefaultMissing(cfg))
    }
}

fn get_locale<T: AsRef<Path>>(path: T, locale: &str) -> Result<HashMap<String, String>> {
    let locale_file =
        File::open(path).map_err(|err| Error::LocaleFileNotFound(locale.to_string(), err))?;

    serde_json::from_reader(locale_file)
        .map_err(|err| Error::LocaleFileDeser(locale.to_string(), err))
}
