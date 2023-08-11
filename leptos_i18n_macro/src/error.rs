use std::collections::HashSet;

use crate::cfg_file::RawConfigFile;
use quote::quote;

#[derive(Debug)]
pub struct InterpolateVariableNotMatching {
    pub key: String,
    pub locale1: String,
    pub locale2: String,
    pub keys1: HashSet<String>,
    pub keys2: HashSet<String>,
}

#[derive(Debug)]
pub enum Error {
    ConfigFileNotFound(std::io::Error),
    ConfigFileDeser(serde_json::Error),
    ConfigFileDefaultMissing(RawConfigFile),
    LocaleFileNotFound(String, std::io::Error),
    LocaleFileDeser(String, serde_json::Error),
    MissingKeysInLocale {
        keys: Vec<String>,
        locale: String,
    },
    InterpolateVariableNotMatching(Box<InterpolateVariableNotMatching>),
    MismatchLocaleKeyKind {
        key: String,
        locale_str: String,
        locale_inter: String,
    },
    InvalidKey(String),
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
            Error::InvalidKey(key) => {
                format!("key {:?} is invalid to be used as field name.", key)
            }
            Error::InterpolateVariableNotMatching(err) => todo!(),
            Error::MismatchLocaleKeyKind {
                key,
                locale_str,
                locale_inter,
            } => todo!(),
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
