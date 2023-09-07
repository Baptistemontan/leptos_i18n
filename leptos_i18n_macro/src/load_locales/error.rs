use std::{collections::HashSet, fmt::Display};

use super::{cfg_file::ConfigFile, key::KeyPath, plural::PluralType};
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
    MissingKeyInLocale {
        locale: String,
        namespace: Option<String>,
        key_path: KeyPath,
    },
    SubKeyMissmatch {
        locale1: String,
        locale2: String,
        namespace: Option<String>,
        key_path: KeyPath,
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
        locale1: String,
        locale2: String,
        namespace: Option<String>,
        key_path: KeyPath,
        type1: PluralType,
        type2: PluralType,
    },
    InvalidKey(String),
    EmptyPlural,
    InvalidPluralType(String),
    NestedPlurals,
    InvalidFallback,
    MultipleFallbacks,
    MissingFallback(PluralType),
    PluralSubkeys,
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
            Error::MissingKeyInLocale { key_path, namespace: None, locale } => write!(f,
                "Some keys are different beetween locale files, \"{}.json\" is missing key: {}",
                locale, key_path
            ),
            Error::MissingKeyInLocale { key_path, namespace: Some(namespace), locale } => write!(f,
                "Some keys are different beetween namespaces files, \"{}/{}.json\" is missing key: {}",
                locale, namespace, key_path
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
            Error::PluralTypeMissmatch { locale1, locale2, namespace: Some(namespace), key_path, type1, type2 } => write!(f, "missmatch value type beetween locale {:?} and locale {:?} in namespace {:?} at key {}: plurals type don't match (found {} and {})", locale1, locale2, namespace, key_path, type1, type2),
            Error::PluralTypeMissmatch { locale1, locale2, namespace: None, key_path, type1, type2 } => write!(f, "missmatch value type beetween locale {:?} and locale {:?} at key {}: plurals type don't match (found {} and {})", locale1, locale2, key_path, type1, type2),
            Error::InvalidKey(key) => write!(f, "invalid key {:?}, it can't be used as a rust identifier, try removing whitespaces and special characters", key),
            Error::EmptyPlural => write!(f, "empty plurals are not allowed"),
            Error::InvalidPluralType(t) => write!(f, "invalid plural type {:?}", t),
            Error::NestedPlurals => write!(f, "nested plurals are not allowed"),
            Error::InvalidFallback => write!(f, "fallbacks are only allowed in last position"),
            Error::MultipleFallbacks => write!(f, "only one fallback is allowed"),
            Error::MissingFallback(t) => write!(f, "plural type {} require a fallback (or a fullrange \"..\")", t),
            Error::PluralSubkeys => write!(f, "subkeys for plurals are not allowed"),
            Error::SubKeyMissmatch { locale1, locale2, namespace: None, key_path } => {
                write!(f, "missmatch value type beetween locale {:?} and locale {:?} at key {}: one has subkeys and the other has direct value.", locale1, locale2, key_path)
            },
            Error::SubKeyMissmatch { locale1, locale2, namespace: Some(namespace), key_path } => {
                write!(f, "missmatch value type beetween locale {:?} and locale {:?} in namespace {:?} at key {}: one has subkeys and the other has direct value.", locale1, locale2, namespace, key_path)
            },
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
