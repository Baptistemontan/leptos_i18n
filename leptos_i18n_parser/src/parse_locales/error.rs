use icu_locale::ParseError as LocidError;
use icu_provider::DataError as IcuDataError;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::{
    cell::{Cell, Ref, RefCell},
    collections::BTreeSet,
    fmt::{Debug, Display},
    io,
    num::TryFromIntError,
    path::PathBuf,
    rc::Rc,
};

use super::{locale::SerdeError, ranges::RangeType};
use crate::{
    parse_locales::cfg_file,
    utils::key::{Key, KeyPath},
};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    IoError(io::Error),
    InvalidLocale {
        locale: Rc<str>,
        err: LocidError,
    },
    PluralRulesError(IcuDataError),
    CargoDirEnvNotPresent(std::env::VarError),
    ManifestNotFound(std::io::Error),
    ConfigNotPresent,
    ConfigFileDeser(toml::de::Error),
    LocaleFileNotFound(Vec<(PathBuf, std::io::Error)>),
    LocaleFileDeser {
        path: PathBuf,
        err: SerdeError,
    },
    DuplicateLocalesInConfig(BTreeSet<Key>),
    DuplicateNamespacesInConfig(BTreeSet<Key>),
    SubKeyMissmatch {
        locale: Key,
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
        locale: Key,
        key_path: KeyPath,
    },
    MissingForeignKey {
        foreign_key: KeyPath,
        locale: Key,
        key_path: KeyPath,
    },
    InvalidForeignKey {
        foreign_key: KeyPath,
        locale: Key,
        key_path: KeyPath,
    },
    UnknownFormatter {
        name: String,
        locale: Key,
        key_path: KeyPath,
    },
    ConflictingPluralRuleType {
        locale: Key,
        key_path: KeyPath,
    },
    InvalidForeignKeyArgs {
        locale: Key,
        key_path: KeyPath,
        err: serde_json::Error,
    },
    InvalidCountArg {
        locale: Key,
        key_path: KeyPath,
        foreign_key: KeyPath,
    },
    InvalidCountArgType {
        locale: Key,
        key_path: KeyPath,
        foreign_key: KeyPath,
        input_type: RangeType,
        range_type: RangeType,
    },
    CountArgOutsideRange {
        locale: Key,
        key_path: KeyPath,
        foreign_key: KeyPath,
        err: TryFromIntError,
    },
    UnexpectedToken {
        locale: Key,
        key_path: KeyPath,
        message: String,
    },
    RangeAndPluralsMix {
        key_path: KeyPath,
    },
    PluralsAtNormalKey {
        locale: Key,
        key_path: KeyPath,
    },
    DisabledFormatter {
        locale: Key,
        key_path: KeyPath,
        formatter_err: &'static str,
    },
    DisabledPlurals {
        locale: Key,
        key_path: KeyPath,
    },
    NoFileFormats,
    MultipleFilesFormats,
    MissingTranslationsURI,
    Custom {
        locale: Key,
        key_path: KeyPath,
        err: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => <io::Error as Display>::fmt(err, f),
            Error::CargoDirEnvNotPresent(err) => {
                write!(
                    f,
                    "Error, can't access env variable \"CARGO_MANIFEST_DIR\": {err}"
                )
            }
            Error::ManifestNotFound(err) => {
                write!(f, "Error accessing cargo manifest (Cargo.toml) : {err}")
            }
            Error::ConfigNotPresent => {
                write!(
                    f,
                    "Could not found \"[package.metadata.leptos-i18n]\" in cargo manifest (Cargo.toml)"
                )
            }
            Error::ConfigFileDeser(err) => {
                write!(f, "Parsing of cargo manifest (Cargo.toml) failed: {err}")
            }
            Error::LocaleFileNotFound(errs) => {
                for (path, err) in errs {
                    writeln!(f, "Could not found file {path:?} : {err}")?;
                }
                Ok(())
            }
            Error::LocaleFileDeser { path, err } => {
                write!(f, "Parsing of file {path:?} failed: {err}")
            }
            Error::RangeParse { range, range_type } => {
                write!(f, "error parsing {range:?} as {range_type}")
            }
            Error::DuplicateLocalesInConfig(duplicates) => write!(
                f,
                "Found duplicates locales in configuration (Cargo.toml): {duplicates:?}"
            ),
            Error::InvalidBoundEnd {
                range,
                range_type: range_type @ (RangeType::F32 | RangeType::F64),
            } => write!(
                f,
                "the range {range:?} end bound is invalid, you can't use exclusif range with {range_type}"
            ),
            Error::InvalidBoundEnd { range, range_type } => write!(
                f,
                "the range {range:?} end bound is invalid, you can't end before {range_type}::MIN"
            ),
            Error::ImpossibleRange(range) => write!(
                f,
                "the range {range:?} is impossible, it end before it starts"
            ),
            Error::DuplicateNamespacesInConfig(duplicates) => write!(
                f,
                "Found duplicates namespaces in configuration (Cargo.toml): {duplicates:?}"
            ),
            Error::RangeTypeMissmatch {
                key_path,
                type1,
                type2,
            } => write!(
                f,
                "Conflicting range value type at key \"{key_path}\", found type {type1} but also type {type2}."
            ),
            Error::InvalidKey(key) => write!(
                f,
                "invalid key {key:?}, it can't be used as a rust identifier, try removing whitespaces and special characters."
            ),
            Error::EmptyRange => write!(f, "empty ranges are not allowed"),
            Error::InvalidRangeType(t) => write!(f, "invalid range type {t:?}"),
            Error::NestedRanges => write!(f, "nested ranges are not allowed"),
            Error::InvalidFallback => write!(f, "fallbacks are only allowed in last position"),
            Error::MultipleFallbacks => write!(f, "only one fallback is allowed"),
            Error::MissingFallback(t) => write!(
                f,
                "range type {t} require a fallback (or a fullrange \"..\")"
            ),
            Error::RangeSubkeys => write!(f, "subkeys for ranges are not allowed"),
            Error::SubKeyMissmatch { locale, key_path } => {
                write!(
                    f,
                    "Missmatch value type beetween locale {locale:?} and default at key \"{key_path}\": one has subkeys and the other has direct value."
                )
            }
            Error::RangeNumberType { found, expected } => write!(
                f,
                "number type {found} can't be used for range type {expected}"
            ),
            Error::ExplicitDefaultInDefault(key_path) => write!(
                f,
                "Explicit defaults (null) are not allowed in default locale, at key \"{key_path}\""
            ),
            Error::RecursiveForeignKey { locale, key_path } => write!(
                f,
                "Borrow Error while linking foreign key at key \"{key_path}\" in locale {locale:?}, check for recursive foreign key."
            ),
            Error::MissingForeignKey {
                foreign_key,
                locale,
                key_path,
            } => write!(
                f,
                "Invalid foreign key \"{foreign_key}\" at key \"{key_path}\" in locale {locale:?}, key don't exist."
            ),
            Error::InvalidForeignKey {
                foreign_key,
                locale,
                key_path,
            } => write!(
                f,
                "Invalid foreign key \"{foreign_key}\" at key \"{key_path}\" in locale {locale:?}, foreign key to subkeys are not allowed."
            ),
            Error::UnknownFormatter {
                name,
                locale,
                key_path,
            } => write!(
                f,
                "Unknown formatter {name:?} at key \"{key_path}\" in locale {locale:?}."
            ),
            Error::ConflictingPluralRuleType { locale, key_path } => write!(
                f,
                "Found both ordinal and cardinal plurals for key \"{key_path}\" in locale {locale:?}."
            ),
            Error::InvalidForeignKeyArgs {
                locale,
                key_path,
                err,
            } => write!(
                f,
                "Malformed foreign key args in locale {locale:?} at key \"{key_path}\": {err}."
            ),
            Error::InvalidCountArg {
                locale,
                key_path,
                foreign_key,
            } => write!(
                f,
                "Invalid arg \"count\" in locale {locale:?} at key \"{key_path}\" to foreign key \"{foreign_key}\": argument \"count\" for plurals or ranges can only be a literal number or a single variable."
            ),
            Error::InvalidCountArgType {
                locale,
                key_path,
                foreign_key,
                input_type,
                range_type,
            } => write!(
                f,
                "Invalid arg \"count\" in locale {locale:?} at key \"{key_path}\" to foreign key \"{foreign_key}\": argument \"count\" of type {input_type} for range of type {range_type} is not allowed."
            ),
            Error::CountArgOutsideRange {
                locale,
                key_path,
                foreign_key,
                err,
            } => write!(
                f,
                "Invalid arg \"count\" in locale {locale:?} at key \"{key_path}\" to foreign key \"{foreign_key}\": argument \"count\" is outside range: {err}"
            ),
            Error::UnexpectedToken {
                locale,
                key_path,
                message,
            } => write!(
                f,
                "Unexpected error occured while parsing key \"{key_path}\" in locale {locale:?}: {message}"
            ),
            Error::RangeAndPluralsMix { key_path } => write!(
                f,
                "mixing plurals and ranges are not supported yet, for key \"{key_path}\""
            ),
            Error::PluralsAtNormalKey { key_path, locale } => write!(
                f,
                "In locale {locale:?} at key \"{key_path}\", Found plurals but a key of that name is already present."
            ),
            Error::DisabledFormatter {
                locale,
                key_path,
                formatter_err,
            } => write!(
                f,
                "{}, at key \"{}\" in locale {:?}",
                formatter_err, key_path, locale
            ),
            Error::DisabledPlurals { locale, key_path } => write!(
                f,
                "Plurals are not enabled, enable the \"plurals\" feature to use them, at key \"{key_path}\" in locale {locale:?}"
            ),
            Error::NoFileFormats => write!(
                f,
                "No file formats has been provided for leptos_i18n. Supported formats are: json, json5, yaml and toml."
            ),
            Error::MultipleFilesFormats => write!(
                f,
                "Multiple file formats have been provided for leptos_i18n, choose only one. Supported formats are: json, json5, yaml and toml."
            ),
            Error::InvalidLocale { locale, err } => {
                write!(f, "Found invalid locale {locale:?}: {err}")
            }
            Error::PluralRulesError(plurals_error) => write!(
                f,
                "Error while computing plurals categories: {plurals_error}"
            ),
            Error::MissingTranslationsURI => {
                write!(
                    f,
                    "{:?} config option is missing. You are using dynamic loading in CSR, that value is required.",
                    cfg_file::Field::TRANSLATIONS_URI
                )
            }
            Error::Custom {
                locale,
                key_path,
                err,
            } => write!(
                f,
                "Error in locale {locale:?} at key \"{key_path}\": {err:?}"
            ),
        }
    }
}

impl Error {
    pub fn custom(locale: Key, key_path: KeyPath, err: impl ToString) -> Self {
        Self::Custom {
            locale,
            key_path,
            err: err.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

pub struct BoxedError(Box<Error>);

impl<T: Into<Error>> From<T> for BoxedError {
    fn from(value: T) -> Self {
        BoxedError(Box::new(value.into()))
    }
}

impl BoxedError {
    pub fn into_inner(self) -> Error {
        *self.0
    }

    pub fn into_boxed(self) -> Box<Error> {
        self.0
    }
}

impl Debug for BoxedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Error as Debug>::fmt(&self.0, f)
    }
}

impl Display for BoxedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Error as Display>::fmt(&self.0, f)
    }
}

pub type Result<T, E = BoxedError> = core::result::Result<T, E>;

impl std::error::Error for Error {}

use super::plurals::{PluralForm, PluralRuleType};

#[derive(Debug)]
pub enum Warning {
    MissingKey {
        locale: Key,
        key_path: KeyPath,
    },
    SurplusKey {
        locale: Key,
        key_path: KeyPath,
    },
    UnusedForm {
        locale: Key,
        key_path: KeyPath,
        form: PluralForm,
        rule_type: PluralRuleType,
    },
    NonUnicodePath {
        locale: Key,
        namespace: Option<Key>,
        path: std::path::PathBuf,
    },
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::MissingKey { locale, key_path } => {
                write!(f, "Missing key \"{key_path}\" in locale {locale:?}")
            }
            Warning::SurplusKey { locale, key_path } => write!(
                f,
                "Key \"{key_path}\" is present in locale {locale:?} but not in default locale, it is ignored"
            ),
            Warning::UnusedForm {
                locale,
                key_path,
                form,
                rule_type,
            } => {
                write!(
                    f,
                    "At key \"{key_path}\", locale {locale:?} does not use {rule_type} plural form \"{form}\", it is still kept but is useless."
                )
            }
            Warning::NonUnicodePath {
                locale,
                namespace: None,
                path,
            } => write!(
                f,
                "File path for locale {locale:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {path:?}"
            ),
            Warning::NonUnicodePath {
                locale,
                namespace: Some(ns),
                path,
            } => write!(
                f,
                "File path for locale {locale:?} in namespace {ns:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {path:?}"
            ),
        }
    }
}

#[derive(Default)]
pub struct Diagnostics {
    errors: RefCell<Vec<Error>>,
    warnings: RefCell<Vec<Warning>>,
    has_ranges: Cell<bool>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn emit_error(&self, error: Error) {
        self.errors.borrow_mut().push(error);
    }

    pub fn emit_warning(&self, warning: Warning) {
        self.warnings.borrow_mut().push(warning);
    }

    pub fn errors(&self) -> Ref<'_, [Error]> {
        let errors = self.errors.borrow();
        Ref::map(errors, Vec::as_slice)
    }

    pub fn warnings(&self) -> Ref<'_, [Warning]> {
        let warnings = self.warnings.borrow();
        Ref::map(warnings, Vec::as_slice)
    }

    pub fn borrow(&self) -> (Ref<'_, [Error]>, Ref<'_, [Warning]>) {
        (self.errors(), self.warnings())
    }

    pub fn has_ranges(&self) -> bool {
        self.has_ranges.get()
    }

    pub fn set_has_ranges(&self) {
        self.has_ranges.set(true);
    }
}

impl ToTokens for Diagnostics {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let (errors, warnings) = self.borrow();
        let iter = errors.iter().map(ToString::to_string);
        let warnings = generate_warnings(&warnings);

        quote! {
            #(
                compile_error!(#iter);
            )*

            #warnings
        }
    }
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ts = Self::to_token_stream(self);
        tokens.extend(ts);
    }
}

fn warning_fn((index, warning): (usize, &Warning)) -> TokenStream {
    let msg = warning.to_string();
    let fn_name = format_ident!("w{}", index);
    quote! {
        #[deprecated(note = #msg)]
        fn #fn_name() {
            unimplemented!()
        }
    }
}

fn generate_warnings_inner(warnings: &[Warning]) -> TokenStream {
    let warning_fns = warnings.iter().enumerate().map(warning_fn);

    let fn_calls = (0..warnings.len()).map(|i| {
        let fn_name = format_ident!("w{}", i);
        quote!(#fn_name();)
    });

    quote! {
        #[allow(unused)]
        fn warnings() {
            #(
                #warning_fns
            )*

            #(
                #fn_calls
            )*
        }
    }
}

pub fn generate_warnings(warnings: &[Warning]) -> Option<TokenStream> {
    if warnings.is_empty() {
        None
    } else {
        Some(generate_warnings_inner(warnings))
    }
}
