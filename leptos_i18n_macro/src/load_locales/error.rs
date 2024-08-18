use std::{collections::HashSet, fmt::Display, path::PathBuf, rc::Rc};

use super::{locale::SerdeError, ranges::RangeType};
use crate::utils::key::{Key, KeyPath};
use quote::quote;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    CargoDirEnvNotPresent(std::env::VarError),
    ManifestNotFound(std::io::Error),
    ConfigNotPresent,
    ConfigFileDeser(toml::de::Error),
    LocaleFileNotFound(Vec<(PathBuf, std::io::Error)>),
    LocaleFileDeser {
        path: PathBuf,
        err: SerdeError,
    },
    DuplicateLocalesInConfig(HashSet<String>),
    DuplicateNamespacesInConfig(HashSet<String>),
    MissingKeyInLocale {
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    SubKeyMissmatch {
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    RangeParse {
        range: String,
        range_type: RangeType,
    },
    InvalidBoundEnd {
        range: String,
        range_type: RangeType,
    },
    ImpossibleRange(String),
    RangeTypeMissmatch {
        key_path: KeyPath,
        type1: RangeType,
        type2: RangeType,
    },
    InvalidKey(String),
    EmptyRange,
    InvalidRangeType(String),
    NestedRanges,
    InvalidFallback,
    MultipleFallbacks,
    MissingFallback(RangeType),
    RangeSubkeys,
    RangeNumberType {
        found: RangeType,
        expected: RangeType,
    },
    ExplicitDefaultInDefault(KeyPath),
    RecursiveForeignKey {
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    MissingForeignKey {
        foreign_key: KeyPath,
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    InvalidForeignKey {
        foreign_key: KeyPath,
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    UnknownFormatter {
        name: String,
        locale: Rc<Key>,
        key_path: KeyPath,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CargoDirEnvNotPresent(err) => {
                write!(f, "Error, can't access env variable \"CARGO_MANIFEST_DIR\": {}", err)
            }
            Error::ManifestNotFound(err) => {
                write!(f, "Error accessing cargo manifest (Cargo.toml) : {}", err)
            },
            Error::ConfigNotPresent => {
                write!(f, "Could not found \"[package.metadata.leptos-i18n]\" in cargo manifest (Cargo.toml)")
            }
            Error::ConfigFileDeser(err) => {
                write!(f, "Parsing of cargo manifest (Cargo.toml) failed: {}", err)
            }
            Error::LocaleFileNotFound(errs) => {
                for (path, err) in errs {
                    writeln!(f,
                        "Could not found file {:?} : {}",
                        path, err
                    )?;
                }
                Ok(())
            }
            Error::LocaleFileDeser { path, err} => write!(f,
                "Parsing of file {:?} failed: {}",
                path, err
            ),
            Error::MissingKeyInLocale { key_path, locale } => write!(f,
                "Some keys are different beetween locale files, locale {:?} is missing key: \"{}\"",
                locale, key_path
            ),
            Error::RangeParse {
                range,
                range_type
            } => write!(f,
                "error parsing {:?} as {}", 
                range, range_type
            ),
            Error::DuplicateLocalesInConfig(duplicates) => write!(f,
                "Found duplicates locales in configuration (Cargo.toml): {:?}", 
                duplicates
            ),
            Error::InvalidBoundEnd {
                range,
                range_type: range_type @ (RangeType::F32 | RangeType::F64)
            } => write!(f,
                "the range {:?} end bound is invalid, you can't use exclusif range with {}", 
                range, range_type
            ),
            Error::InvalidBoundEnd {
                range,
                range_type
            } => write!(f,
                "the range {:?} end bound is invalid, you can't end before {}::MIN", 
                range, range_type
            ),
            Error::ImpossibleRange(range) => write!(f, "the range {:?} is impossible, it end before it starts",
                range
            ),
            Error::DuplicateNamespacesInConfig(duplicates) => write!(f,
                "Found duplicates namespaces in configuration (Cargo.toml): {:?}", 
                duplicates
            ),
            Error::RangeTypeMissmatch { key_path, type1, type2 } => write!(f, "Conflicting range value type at key \"{}\", found type {} but also type {}.", key_path, type1, type2),
            Error::InvalidKey(key) => write!(f, "invalid key {:?}, it can't be used as a rust identifier, try removing whitespaces and special characters.", key),
            Error::EmptyRange => write!(f, "empty ranges are not allowed"),
            Error::InvalidRangeType(t) => write!(f, "invalid prange type {:?}", t),
            Error::NestedRanges => write!(f, "nested ranges are not allowed"),
            Error::InvalidFallback => write!(f, "fallbacks are only allowed in last position"),
            Error::MultipleFallbacks => write!(f, "only one fallback is allowed"),
            Error::MissingFallback(t) => write!(f, "range type {} require a fallback (or a fullrange \"..\")", t),
            Error::RangeSubkeys => write!(f, "subkeys for ranges are not allowed"),
            Error::SubKeyMissmatch { locale, key_path } => {
                write!(f, "Missmatch value type beetween locale {:?} and default at key \"{}\": one has subkeys and the other has direct value.", locale, key_path)
            },
            Error::RangeNumberType { found, expected } => write!(f, "number type {} can't be used for range type {}", found, expected),
            Error::ExplicitDefaultInDefault(key_path) => write!(f, "Explicit defaults (null) are not allowed in default locale, at key \"{}\"", key_path),
            Error::RecursiveForeignKey { locale, key_path } => write!(f, "Borrow Error while linking foreign key at key \"{}\" in locale {:?}, check for recursive foreign key.", key_path, locale),
            Error::MissingForeignKey { foreign_key, locale, key_path } => write!(f, "Invalid foreign key \"{}\" at key \"{}\" in locale {:?}, key don't exist.", foreign_key, key_path, locale),
            Error::Custom(s) => f.write_str(s),
            Error::InvalidForeignKey { foreign_key, locale, key_path } => write!(f, "Invalid foreign key \"{}\" at key \"{}\" in locale {:?}, foreign key to ranges, plurals or subkeys are not allowed.", foreign_key, key_path, locale),
            Error::UnknownFormatter { name, locale, key_path } => write!(f, "Unknown formatter {:?} at key \"{}\" in locale {:?}.", name, key_path, locale),
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
