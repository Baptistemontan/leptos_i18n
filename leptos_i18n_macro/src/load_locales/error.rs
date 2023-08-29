use std::{collections::HashSet, fmt::Display};

use super::{cfg_file::ConfigFile, plural::PluralType};
use quote::quote;

#[derive(Debug)]
pub enum Error {
    ConfigFileNotFound(std::io::Error),
    ConfigFileDeser(serde_json::Error),
    ConfigFileDefaultMissing(ConfigFile),
    LocaleFileNotFound {
        locale: String,
        namespace: Option<String>, 
        err: std::io::Error
    },
    LocaleFileDeser {
        locale: String, 
        namespace: Option<String>,   
        err: serde_json::Error
    },
    DuplicateLocalesInConfig(HashSet<String>),
    DuplicateNamespacesInConfig(HashSet<String>),
    MissingKeysInLocale {
        locale: String,
        namespace: Option<String>,
        keys: Vec<String>,
    },
    InvalidLocaleName(String),
    InvalidNameSpaceName(String),
    InvalidLocaleKey {
        key: String,
        locale: String,
        namespace: Option<String>,
    },
    InvalidPlural {
        locale_name: String,
        locale_key: String,
        namespace: Option<String>,
        plural: String,
        plural_type: PluralType
    },
    InvalidBoundEnd {
        locale_name: String,
        locale_key: String,
        namespace: Option<String>,
        range: String,
        plural_type: PluralType
    },
    ImpossibleRange {
        locale_name: String,
        locale_key: String,
        namespace: Option<String>,
        range: String,
    },
    PluralTypeMissmatch {
        locale_key: String,
        namespace: Option<String>,
    },
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
            Error::LocaleFileNotFound {locale, namespace: None, err} => {
                write!(f,
                    "Could not found locale file \"{}.json\" : {}",
                    locale, err
                )
            }
            Error::LocaleFileNotFound {locale, namespace: Some(namespace), err} => {
                write!(f,
                    "Could not found namespace file \"{}/{}.json\" : {}",
                    locale, namespace, err
                )
            }
            Error::LocaleFileDeser {locale, namespace: None, err} => write!(f,
                "Parsing of locale file \"{}.json\" failed: {}",
                locale, err
            ),
            Error::LocaleFileDeser {locale, namespace: Some(namespace), err} => write!(f,
                "Parsing of namespace file \"{}/{}.json\" failed: {}",
                locale, namespace, err
            ),
            Error::MissingKeysInLocale { keys, namespace: None, locale } => write!(f,
                "Some keys are different beetween locale files, \"{}.json\" is missing keys: {:?}",
                locale, keys
            ),
            Error::MissingKeysInLocale { keys, namespace: Some(namespace), locale } => write!(f,
                "Some keys are different beetween namespaces files, \"{}/{}.json\" is missing keys: {:?}",
                locale, namespace, keys
            ),
            Error::InvalidLocaleName(name) => {
                write!(f,
                    "locale name {:?} could not be turned into an identifier",
                    name
                )
            }
            Error::InvalidLocaleKey { key, locale, namespace } => {
                match namespace {
                    Some(namespace) => write!(f,
                        "In locale {:?} namespace {:?} the key {:?} cannot be used as an identifier",
                        locale, namespace, key
                    ),
                    None => write!(f,
                        "In locale {:?} the key {:?} cannot be used as an identifier",
                        locale, key
                    ),
                }
                
            }
            Error::InvalidPlural {
                locale_name,
                locale_key,
                namespace: None,
                plural,
                plural_type
            } => write!(f,
                "In locale {:?} at key {:?} found invalid plural {:?}, expected {:?}", 
                locale_name, locale_key, plural, plural_type
            ),
            Error::InvalidPlural {
                locale_name,
                locale_key,
                namespace: Some(namespace),
                plural,
                plural_type
            } => write!(f,
                "In locale {:?} at namespace {:?} at key {:?} found invalid plural {:?}, expected {:?}", 
                locale_name, namespace, locale_key, plural, plural_type
            ),
            Error::DuplicateLocalesInConfig(duplicates) => write!(f,
                "Found duplicates locales in configuration file (i18n.json): {:?}", 
                duplicates
            ),
            Error::InvalidBoundEnd {
                locale_name,
                locale_key,
                namespace: None,
                range,
                plural_type: plural_type @ (PluralType::F32 | PluralType::F64)
            } => write!(f,
                "In locale {:?} at key {:?} the range {:?} end bound is invalid, you can't use exclusif range with {:?}", 
                locale_name, locale_key, range, plural_type
            ),
            Error::InvalidBoundEnd {
                locale_name,
                locale_key,
                namespace: None,
                range,
                plural_type
            } => write!(f,
                "In locale {:?} at key {:?} the range {:?} end bound is invalid, you can't end before {:?} MIN", 
                locale_name, locale_key, range, plural_type
            ),
            Error::InvalidBoundEnd {
                locale_name,
                locale_key,
                namespace: Some(namespace),
                range,
                plural_type: plural_type @ (PluralType::F32 | PluralType::F64)
            } => write!(f,
                "In locale {:?} at namespace {:?} at key {:?} the range {:?} end bound is invalid, you can't use exclusif range with {:?}", 
                locale_name, namespace, locale_key, range, plural_type
            ),
            Error::InvalidBoundEnd {
                locale_name,
                locale_key,
                namespace: Some(namespace),
                range,
                plural_type
            } => write!(f,
                "In locale {:?} at namespace {:?} at key {:?} the range {:?} end bound is invalid, you can't end before {:?} MIN", 
                locale_name, namespace, locale_key, range, plural_type
            ),
            Error::ImpossibleRange {
                locale_name,
                locale_key,
                namespace: None,
                range
            } => write!(f, "In locale {:?} at key {:?} the range {:?} is impossible, it end before it starts",
                locale_name, locale_key, range
            ),
            Error::ImpossibleRange {
                locale_name,
                locale_key,
                namespace: Some(namespace),
                range
            } => write!(f, "In locale {:?} at namespace {:?} at key {:?} the range {:?} is impossible, it end before it starts",
                locale_name, namespace, locale_key, range
            ),
            Error::InvalidNameSpaceName(name) => write!(f,
                "namespace {:?} could not be turned into an identifier",
                name
            ),
            Error::DuplicateNamespacesInConfig(duplicates) => write!(f,
                "Found duplicates namespaces in configuration file (i18n.json): {:?}", 
                duplicates
            ),
            Error::PluralTypeMissmatch { locale_key, namespace: Some(namespace) } => write!(f, "In namespace {:?} at key {:?} the plurals types don't match across locales", namespace, locale_key),
            Error::PluralTypeMissmatch { locale_key, namespace: None } => write!(f, "At key {:?} the plurals types don't match across locales", locale_key),
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
