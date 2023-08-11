use crate::cfg_file::RawConfigFile;
use quote::quote;

#[derive(Debug)]
pub struct InterpolateKeysNotMatching {
    pub key: String,
    pub locale1: String,
    pub locale2: String,
    pub comp_keys1: Vec<String>,
    pub var_keys1: Vec<String>,
    pub comp_keys2: Vec<String>,
    pub var_keys2: Vec<String>,
}

impl ToString for InterpolateKeysNotMatching {
    fn to_string(&self) -> String {
        format!("for key {:?} locales {:?} and {:?} don't have the same keys, locale {:?} has {:?} for variable keys and {:?} for component keys, but locale {:?} has {:?} for variable keys and {:?} for component keys",
        self.key,
        self.locale1,
        self.locale2,
        self.locale1,
        self.var_keys1,
        self.comp_keys1,
        self.locale2,
        self.var_keys2,
        self.var_keys2
    )
    }
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
    InterpolateVariableNotMatching(Box<InterpolateKeysNotMatching>),
    MismatchLocaleKeyKind {
        key: String,
        locale_str: String,
        locale_inter: String,
    },
    InvalidLocaleName(String),
    InvalidLocaleKey {
        key: String,
        locale: String,
    },
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
            Error::InterpolateVariableNotMatching(err) => err.to_string(),
            Error::MismatchLocaleKeyKind {
                key,
                locale_str,
                locale_inter,
            } => {
                format!(
                    "for key {:?} locale {:?} is a plain string but locale {:?} need interpolation",
                    key, locale_str, locale_inter
                )
            }
            Error::InvalidLocaleName(name) => {
                format!(
                    "locale name {:?} could not be turned into an identifier",
                    name
                )
            }
            Error::InvalidLocaleKey { key, locale } => {
                format!(
                    "In locale {:?} the key {:?} cannot be used as an identifier",
                    locale, key
                )
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

pub type Result<T, E = Error> = core::result::Result<T, E>;
