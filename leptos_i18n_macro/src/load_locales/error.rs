use std::{fmt::Display, collections::HashSet};

use super::cfg_file::ConfigFile;
use quote::quote;

#[derive(Debug)]
pub enum Error {
    ConfigFileNotFound(std::io::Error),
    ConfigFileDeser(serde_json::Error),
    ConfigFileDefaultMissing(ConfigFile),
    LocaleFileNotFound(String, std::io::Error),
    LocaleFileDeser(String, serde_json::Error),
    DuplicateLocalesInConfig(HashSet<String>),
    MissingKeysInLocale {
        keys: Vec<String>,
        locale: String,
    },
    InvalidLocaleName(String),
    InvalidLocaleKey {
        key: String,
        locale: String,
    },
    InvalidPlural {
        locale_name: String,
        locale_key: String,
        plural: String,
    },
    KeyKindMissmatch {
        locale_key: String,
        key: String
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigFileNotFound(err) => {
                write!(f, "Could not found configuration file (i18n.json) : {}", err)
            }
            Error::ConfigFileDeser(err) => {
                write!(f, "Parsing of configuration file (i18n.json) failed: {}", err)
            }
            Error::ConfigFileDefaultMissing(cfg_file) => write!(f, 
                "{:?} is set as default locale but is not in the locales list: {:?}",
                cfg_file.default, cfg_file.locales
            ),
            Error::LocaleFileNotFound(locale_name, err) => {
                write!(f, 
                    "Could not found locale file \"{}.json\" : {}",
                    locale_name, err
                )
            }
            Error::LocaleFileDeser(locale_name, err) => write!(f, 
                "Parsing of locale file \"{}.json\" failed: {}",
                locale_name, err
            ),
            Error::MissingKeysInLocale { keys, locale } => write!(f, 
                "Some keys are different beetween locale files, \"{}.json\" is missing keys: {:?}",
                locale, keys
            ),
            Error::InvalidLocaleName(name) => {
                write!(f, 
                    "locale name {:?} could not be turned into an identifier",
                    name
                )
            }
            Error::InvalidLocaleKey { key, locale } => {
                write!(f, 
                    "In locale {:?} the key {:?} cannot be used as an identifier",
                    locale, key
                )
            }
            Error::InvalidPlural { locale_name, locale_key, plural } => write!(f, "In locale {:?} at key {:?} found invalid plural {:?}", locale_name, locale_key, plural),
            Error::DuplicateLocalesInConfig(duplicates) => write!(f, "Found duplicates locales in configuration file (i18n.json): {:?}", duplicates),
            Error::KeyKindMissmatch { locale_key, key } => write!(f, "At locale key {:?}, use of the key {:?} for a variable and a component is not allowed", locale_key, key),
        }
    }
}

impl From<Error> for proc_macro::TokenStream {
    fn from(value: Error) -> Self {
        let error = value.to_string();
        quote!(compile_error!(#error)).into()
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
