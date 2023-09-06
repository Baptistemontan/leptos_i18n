use std::{collections::HashSet, fmt::Display};

use super::{cfg_file::ConfigFile, plural::PluralType};
use quote::quote;

#[derive(Debug)]
pub enum Error {
    ManifestNotFound(std::io::Error),
    ConfigNotPresent,
    ConfigFileDeser(toml::de::Error),
    ConfigFileDefaultMissing(Box<ConfigFile>),
    LocaleFileNotFound {
        path: String,
        err: std::io::Error,
    },
    LocaleFileDeser {
        path: String,
        err: serde_json::Error,
    },
    DuplicateLocalesInConfig(HashSet<String>),
    DuplicateNamespacesInConfig(HashSet<String>),
    MissingKeysInLocale {
        locale: String,
        namespace: Option<String>,
        keys: Vec<String>,
    },
    PluralParse {
        plural: String,
        plural_type: PluralType,
    },
    InvalidBoundEnd {
        range: String,
        plural_type: PluralType,
    },
    ImpossibleRange(String),
    PluralTypeMissmatch {
        locale_key: String,
        namespace: Option<String>,
    },
    InvalidKey(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ManifestNotFound(err) => {
                write!(f, "Error accessing cargo manifest (Cargo.toml) : {}", err)
            },
            Error::ConfigNotPresent => {
                write!(f, "Could not found \"[package.metadata.leptos-i18n]\" in cargo manifest (Cargo.toml)")
            }
            Error::ConfigFileDeser(err) => {
                write!(f, "Parsing of cargo manifest (Cargo.toml) failed: {}", err)
            }
            Error::ConfigFileDefaultMissing(cfg_file) => write!(f,
                "{:?} is set as default locale but is not in the locales list: {:?}",
                cfg_file.default, cfg_file.locales
            ),
            Error::LocaleFileNotFound { path, err} => {
                write!(f,
                    "Could not found file {:?} : {}",
                    path, err
                )
            }
            Error::LocaleFileDeser { path, err} => write!(f,
                "Parsing of file {:?} failed: {}",
                path, err
            ),
            Error::MissingKeysInLocale { keys, namespace: None, locale } => write!(f,
                "Some keys are different beetween locale files, \"{}.json\" is missing keys: {:?}",
                locale, keys
            ),
            Error::MissingKeysInLocale { keys, namespace: Some(namespace), locale } => write!(f,
                "Some keys are different beetween namespaces files, \"{}/{}.json\" is missing keys: {:?}",
                locale, namespace, keys
            ),
            Error::PluralParse {
                plural,
                plural_type
            } => write!(f,
                "error parsing {:?} as {}", 
                plural, plural_type
            ),
            Error::DuplicateLocalesInConfig(duplicates) => write!(f,
                "Found duplicates locales in configuration (Cargo.toml): {:?}", 
                duplicates
            ),
            Error::InvalidBoundEnd {
                range,
                plural_type: plural_type @ (PluralType::F32 | PluralType::F64)
            } => write!(f,
                "the range {:?} end bound is invalid, you can't use exclusif range with {}", 
                range, plural_type
            ),
            Error::InvalidBoundEnd {
                range,
                plural_type
            } => write!(f,
                "the range {:?} end bound is invalid, you can't end before {}::MIN", 
                range, plural_type
            ),
            Error::ImpossibleRange(range) => write!(f, "the range {:?} is impossible, it end before it starts",
                range
            ),
            Error::DuplicateNamespacesInConfig(duplicates) => write!(f,
                "Found duplicates namespaces in configuration (Cargo.toml): {:?}", 
                duplicates
            ),
            Error::PluralTypeMissmatch { locale_key, namespace: Some(namespace) } => write!(f, "In namespace {:?} at key {:?} the plurals types don't match across locales", namespace, locale_key),
            Error::PluralTypeMissmatch { locale_key, namespace: None } => write!(f, "At key {:?} the plurals types don't match across locales", locale_key),
            Error::InvalidKey(key) => write!(f, "invalid key {:?}, it can't be used as a rust identifier, try removing whitespaces and special characters", key),
        }
    }
}

impl From<Error> for proc_macro::TokenStream {
    fn from(value: Error) -> Self {
        let error = value.to_string();
        quote!(compile_error!(#error);).into()
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
