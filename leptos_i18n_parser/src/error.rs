use core::panic;
use icu_locale::ParseError as LocidError;
use icu_provider::DataError as IcuDataError;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::{
    cell::{Ref, RefCell},
    collections::BTreeSet,
    fmt::{Debug, Display},
    io,
    path::PathBuf,
    rc::Rc,
};

use crate::extractor::values::plurals::{PluralForm, PluralRuleType};
use crate::utils::{
    Location,
    key::{Key, KeyPath},
};

#[derive(Debug)]
#[non_exhaustive]
pub enum SerdeError {
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Toml(toml::de::Error),
    Json5(json5::Error),
    Custom(String),
    Io(std::io::Error),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    IoError(io::Error),
    InvalidLocale {
        locale: String,
        err: LocidError,
    },
    PluralRulesError(IcuDataError),
    CargoDirEnvNotPresent(std::env::VarError),
    LocaleFileNotFound(Vec<(PathBuf, std::io::Error)>),
    PluralsMergingOverlap {
        loc: Location,
        is_rule_type_overlap: bool,
        is_other_subkeys: bool,
    },
    LocaleFileDeser {
        path: Option<PathBuf>,
        err: SerdeError,
    },
    DuplicateLocalesInConfig(BTreeSet<Key>),
    DuplicateNamespacesInConfig(BTreeSet<Key>),
    MissingLocaleFile {
        locale: Key,
        namespace: Option<Key>,
    },
    MissingNamespaceFiles(Key),
    TranslationFilesMismatch {
        /// `true` if the configuration has namespaces but the files are given per locale.
        expected_namespaces: bool,
    },
    SubKeyMissmatch {
        loc: Location,
    },
    InvalidKey(String),
    CollidingKeys {
        loc1: Location,
        loc2: Location,
    },
    InvalidKeyAt {
        key: String,
        loc: Location,
    },
    InvalidFallback,
    MultipleFallbacks,
    ExplicitDefaultInDefault(KeyPath),
    ExplicitDefaultInPlurals {
        loc: Location,
        form: PluralForm,
    },
    RecursiveForeignKey {
        loc: Location,
    },
    InvalidForeignKey {
        foreign_key: KeyPath,
        loc: Location,
    },
    ForeignKeyToSubkey {
        foreign_key: KeyPath,
        loc: Location,
    },
    UnknownFormatter {
        name: String,
        loc: Location,
    },
    ConflictingPluralRuleType {
        loc: Location,
    },
    InvalidForeignKeyArgs {
        loc: Location,
        err: serde_json::Error,
    },
    InvalidPluralOperandForeignKeyArg {
        loc: Location,
        arg_name: String,
        value: String,
        err: String,
    },
    InvalidFkCountArg {
        key: Key,
        loc: Location,
        foreign_key: KeyPath,
    },
    InvalidFkAttrArg {
        key: Key,
        loc: Location,
        foreign_key: KeyPath,
    },
    UnexpectedToken {
        loc: Location,
        message: String,
    },
    PluralsAtNormalKey {
        loc: Location,
    },
    DisabledFormatter {
        loc: Location,
        formatter_err: &'static str,
    },
    DisabledPlurals {
        loc: Location,
    },
    NoFileFormats,
    MultipleFilesFormats,
    MissingTranslationsURI,
    InvalidFormatterArgName {
        loc: Location,
        name: String,
        err: String,
    },
    InvalidFormatterArg {
        loc: Location,
        arg_name: String,
        arg: Option<String>,
        err: String,
    },
    InvalidFormatter {
        loc: Location,
        err: String,
    },
    InvalidAttributeName {
        loc: Location,
        value: String,
    },
    InvalidAttribute {
        loc: Location,
        attr_name: String,
        attr_value: String,
        err: String,
    },
    InvalidForeignKeyArgForAttribute {
        loc: Location,
        arg_name: Key,
        foreign_key: KeyPath,
    },
    UnknownLocaleInInherit {
        loc: &'static panic::Location<'static>,
        locale: String,
    },
    DefaultLocaleCantInherit {
        loc: &'static panic::Location<'static>,
    },
    ComponentSelfClosedMissmatch {
        path: KeyPath,
        component_key: String,
    },

    Custom(String),
}

/// Variables and components keys are stored with these prefixes (`var_count`, `comp_b`), not shown to the user.
const VAR_PREFIX: &str = "var_";
const COMP_PREFIX: &str = "comp_";

fn strip_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    s.strip_prefix(prefix).unwrap_or(s)
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
            Error::LocaleFileNotFound(errs) => {
                for (path, err) in errs {
                    writeln!(f, "Could not found file {path:?} : {err}")?;
                }
                Ok(())
            }
            Error::LocaleFileDeser { path, err } => {
                write!(f, "Parsing of file {path:?} failed: {err}")
            }
            Error::DuplicateLocalesInConfig(duplicates) => write!(
                f,
                "Found duplicates locales in configuration (Cargo.toml): {duplicates:?}"
            ),
            Error::DuplicateNamespacesInConfig(duplicates) => write!(
                f,
                "Found duplicates namespaces in configuration (Cargo.toml): {duplicates:?}"
            ),
            Error::MissingLocaleFile {
                locale,
                namespace: None,
            } => write!(
                f,
                "Missing translation file for locale {:?}: the locale is in the configuration but no file was given for it.",
                locale.name
            ),
            Error::MissingLocaleFile {
                locale,
                namespace: Some(namespace),
            } => write!(
                f,
                "Missing translation file for locale {:?} in namespace {:?}: the locale is in the configuration but no file was given for it.",
                locale.name, namespace.name
            ),
            Error::MissingNamespaceFiles(namespace) => write!(
                f,
                "Missing translation files for namespace {:?}: the namespace is in the configuration but no files were given for it.",
                namespace.name
            ),
            Error::TranslationFilesMismatch {
                expected_namespaces: true,
            } => f.write_str(
                "The configuration has namespaces, but the translation files are given per locale (`TranslationFiles::Locales`) instead of per namespace (`TranslationFiles::NameSpaces`).",
            ),
            Error::TranslationFilesMismatch {
                expected_namespaces: false,
            } => f.write_str(
                "The configuration has no namespaces, but the translation files are given per namespace (`TranslationFiles::NameSpaces`) instead of per locale (`TranslationFiles::Locales`).",
            ),
            Error::InvalidKey(key) => write!(
                f,
                "invalid key {key:?}, it can't be used as a rust identifier, try removing whitespaces and special characters."
            ),
            Error::InvalidKeyAt { key, loc } => write!(
                f,
                "invalid key {key:?} at {loc}, it can't be used as a rust identifier, try removing whitespaces and special characters."
            ),
            Error::InvalidFallback => write!(f, "fallbacks are only allowed in last position"),
            Error::MultipleFallbacks => write!(f, "only one fallback is allowed"),
            Error::SubKeyMissmatch { loc } => {
                write!(
                    f,
                    "Missmatch value type beetween locale {:?} and default at key \"{}\": one has subkeys and the other has direct value.",
                    loc.locale.key.name, loc.key_path
                )
            }
            Error::ExplicitDefaultInDefault(key_path) => write!(
                f,
                "Explicit defaults (null) are not allowed in default locale, at key \"{key_path}\""
            ),
            Error::RecursiveForeignKey { loc } => write!(
                f,
                "Borrow Error while linking foreign key at {loc}, check for recursive foreign key."
            ),
            Error::InvalidForeignKey { foreign_key, loc } => write!(
                f,
                "Invalid foreign key \"{foreign_key}\" at {loc}, key don't exist."
            ),
            Error::ForeignKeyToSubkey { foreign_key, loc } => write!(
                f,
                "Invalid foreign key \"{foreign_key}\" at {loc}, foreign key to subkeys are not allowed."
            ),
            Error::UnknownFormatter { name, loc } => {
                write!(f, "Unknown formatter {name:?} at {loc}.")
            }
            Error::ConflictingPluralRuleType { loc } => {
                write!(f, "Found both ordinal and cardinal plurals at {loc}.")
            }
            Error::InvalidForeignKeyArgs { loc, err } => {
                write!(f, "Malformed foreign key args at {loc}: {err}.")
            }
            Error::InvalidFkCountArg {
                key,
                loc,
                foreign_key,
            } => write!(
                f,
                "Invalid arg \"{}\" at {loc} to foreign key \"{foreign_key}\": arguments for plurals count can only be a literal number or a single variable.",
                key.name
            ),
            Error::InvalidFkAttrArg {
                key,
                loc,
                foreign_key,
            } => write!(
                f,
                "Invalid arg \"{}\" at {loc} to foreign key \"{foreign_key}\": argument attributes can only be a literal or a single variable.",
                key.name
            ),
            Error::UnexpectedToken { loc, message } => write!(
                f,
                "Unexpected error occured while parsing at {loc}: {message}"
            ),
            Error::PluralsAtNormalKey { loc } => write!(
                f,
                "At {loc}, Found plurals but a key of that name is already present."
            ),
            Error::DisabledFormatter { loc, formatter_err } => {
                write!(f, "{}, at {loc}", formatter_err)
            }
            Error::DisabledPlurals { loc } => write!(
                f,
                "Plurals are not enabled, enable the \"plurals\" feature to use them, at {loc}"
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
                    "`translations_uri` config option is missing. You are using dynamic loading in CSR, that value is required."
                )
            }
            Error::Custom(err) => {
                write!(f, "{err}")
            }
            Error::InvalidFormatterArgName { loc, name, err } => write!(
                f,
                "Formatter argument name {name:?} is invalid at {loc}: {err}"
            ),
            Error::InvalidFormatterArg {
                loc,
                arg_name,
                arg,
                err,
            } => write!(
                f,
                "Formatter argument value {arg:?} for argument name {arg_name:?} is invalid at {loc}: {err}"
            ),
            Error::InvalidFormatter { loc, err } => {
                write!(f, "Formatter is invalid at {loc}: {err}")
            }
            Error::InvalidAttribute {
                loc,
                attr_name,
                attr_value,
                err,
            } => write!(
                f,
                "Invalid component attribute value {attr_value:?} for attribute {attr_name:?} at {loc}: {err}"
            ),
            Error::InvalidForeignKeyArgForAttribute {
                loc,
                arg_name,
                foreign_key,
            } => write!(
                f,
                "Invalid foreign key argument {arg_name:?} to key \"{foreign_key}\" at {loc}: argument to attributes must be either a variable or a literal (boolean, string, numbers)"
            ),
            Error::InvalidAttributeName { loc, value } => {
                write!(f, "Invalid attribute name {value:?} at {loc}")
            }
            Error::UnknownLocaleInInherit { loc, locale } => {
                write!(
                    f,
                    "Tried to declare inheritance for an unknown locale \"{locale}\" at {loc}, make sure to add it before declaring the inheritance."
                )
            }
            Error::DefaultLocaleCantInherit { loc } => {
                write!(
                    f,
                    "Tried to declare inheritance for the default locale at {loc}"
                )
            }
            Error::CollidingKeys { loc1, loc2 } => {
                // both locations are always in the same locale
                write!(
                    f,
                    "Colliding keys \"{}\" and \"{}\" in locale {:?}: they resolve to the same Rust identifier.",
                    loc1.key_path, loc2.key_path, loc1.locale.key.name
                )
            }
            Error::ComponentSelfClosedMissmatch {
                path,
                component_key,
            } => {
                write!(
                    f,
                    "At {path} the component \"{component_key}\" is both used as a self-closed component and a normal one, this is not allowed."
                )
            }
            Error::ExplicitDefaultInPlurals { loc, form } => {
                write!(
                    f,
                    "Explicit default in plurals is not allowed, at {loc}{form}"
                )
            }
            Error::PluralsMergingOverlap {
                loc,
                is_rule_type_overlap: false,
                is_other_subkeys: false,
            } => write!(
                f,
                "once merged, plurals for key {} overlap with a normal value",
                loc
            ),
            Error::PluralsMergingOverlap {
                loc,
                is_rule_type_overlap: true,
                is_other_subkeys: _,
            } => write!(
                f,
                "key {} is both ordinal and cardinal plurals, this is not allowed.",
                loc
            ),
            Error::PluralsMergingOverlap {
                loc,
                is_rule_type_overlap: _,
                is_other_subkeys: true,
            } => write!(
                f,
                "once merged, plurals for key {} overlap with subkeys",
                loc
            ),
            Error::InvalidPluralOperandForeignKeyArg {
                loc,
                arg_name,
                value,
                err,
            } => write!(
                f,
                "Invalid value for foreign key arg \"{arg_name}\" at {loc}. Value \"{value}\" can't be used as a plural operand: {err}"
            ),
        }
    }
}

impl Error {
    pub fn custom(err: impl ToString) -> Self {
        Self::Custom(err.to_string())
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

impl From<BoxedError> for Box<dyn core::error::Error> {
    fn from(value: BoxedError) -> Self {
        value.0
    }
}

impl From<Box<Error>> for BoxedError {
    fn from(value: Box<Error>) -> Self {
        BoxedError(value)
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

impl std::fmt::Display for SerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerdeError::Json(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Yaml(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Toml(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Json5(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Io(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Custom(err) => std::fmt::Display::fmt(err, f),
        }
    }
}

impl SerdeError {
    pub fn custom<T: ToString>(err: T) -> Self {
        SerdeError::Custom(err.to_string())
    }
}

impl From<std::io::Error> for SerdeError {
    fn from(value: std::io::Error) -> Self {
        SerdeError::Io(value)
    }
}

impl std::error::Error for SerdeError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum Warning {
    MissingKey {
        loc: Location,
    },
    SurplusKey {
        loc: Location,
    },
    UnusedForm {
        loc: Location,
        form: PluralForm,
        rule_type: PluralRuleType,
    },
    KeySpellingMismatch {
        key_path: KeyPath,
        provided_key: Location,
    },
    VarKeySpellingMismatch {
        loc: Location,
        key1: Rc<str>,
        key2: Rc<str>,
        resolved_ident: Rc<str>,
    },
    CompKeySpellingMismatch {
        loc: Location,
        key1: Rc<str>,
        key2: Rc<str>,
        resolved_ident: Rc<str>,
        self_closed: bool,
    },
    PluralKeySpellingMismatch {
        key1: Location,
        base1: Rc<str>,
        key2: KeyPath,
        base2: Rc<str>,
        resolved_ident: Rc<str>,
    },
    DuplicateFkArgs {
        loc: Location,
        fk_target: KeyPath,
        arg_name: String,
    },
    CollidingFkArgs {
        loc: Location,
        fk_target: KeyPath,
        arg1: String,
        arg2: String,
        ident_repr: String,
    },
    NonUnicodePath {
        locale: Key,
        namespace: Option<Key>,
        path: std::path::PathBuf,
    },
    UnexpectedCharsAfterFormatter {
        loc: Location,
        formatter_name: String,
        chars: String,
    },
    PluralLookingKeyInvalidBase {
        loc: Location,
        base: Rc<str>,
    },
    PluralLookingKeysWithNoOther {
        base_key_location: Location,
        keys: Vec<Key>,
        rule_type: PluralRuleType,
    },
    Custom(String),
}

impl Warning {
    pub fn custom(err: impl ToString) -> Self {
        Warning::Custom(err.to_string())
    }
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::MissingKey { loc } => {
                write!(
                    f,
                    "Missing key \"{}\" in locale {:?}",
                    loc.key_path, loc.locale.key.name
                )
            }
            Warning::SurplusKey { loc } => write!(
                f,
                "Key \"{}\" is present in locale {:?} but not in default locale, it is ignored",
                loc.key_path, loc.locale.key.name
            ),
            Warning::UnusedForm {
                loc,
                form,
                rule_type,
            } => {
                write!(
                    f,
                    "At key \"{}\", locale {:?} does not use {rule_type} plural form \"{form}\", it is still kept but is useless.",
                    loc.key_path, loc.locale.key.name
                )
            }
            Warning::NonUnicodePath {
                locale,
                namespace: None,
                path,
            } => write!(
                f,
                "File path for locale {locale:?} is not valid UTF8, can't add it to build script depedencies. Path: {path:?}"
            ),
            Warning::NonUnicodePath {
                locale,
                namespace: Some(ns),
                path,
            } => write!(
                f,
                "File path for locale {locale:?} in namespace {ns:?} is not valid UTF8, can't add it to build script depedencies. Path: {path:?}"
            ),
            Warning::Custom(warn) => write!(f, "{warn}"),
            Warning::UnexpectedCharsAfterFormatter {
                loc,
                chars,
                formatter_name,
            } => write!(
                f,
                "Unexpected characters {chars:?} after formatter {formatter_name:?} at {loc}"
            ),
            Warning::PluralLookingKeyInvalidBase { loc, base } => {
                write!(
                    f,
                    "Key \"{}\" in locale {:?} looks like a plural form, but ",
                    loc.key_path, loc.locale.key.name
                )?;
                if base.is_empty() {
                    f.write_str("its base is empty")?;
                } else {
                    write!(f, "its base \"{base}\" is not a valid Rust identifier")?;
                }
                f.write_str(", so it can't be a plural and is kept as a normal key.")
            }
            Warning::PluralLookingKeysWithNoOther {
                base_key_location,
                keys,
                rule_type,
            } => {
                f.write_str("Keys ")?;
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "\"{key}\"")?;
                }
                let other_suffix = match rule_type {
                    PluralRuleType::Cardinal => "_other",
                    PluralRuleType::Ordinal => "_ordinal_other",
                };
                let base = base_key_location
                    .key_path
                    .path
                    .last()
                    .map(|key| &*key.name)
                    .unwrap_or_default();
                write!(
                    f,
                    " in locale {:?} look like {rule_type} plural forms of \"{}\", but there is no \"{base}{other_suffix}\" form, which every plural needs, so they are kept as normal keys.",
                    base_key_location.locale.key.name, base_key_location.key_path
                )
            }
            Warning::KeySpellingMismatch {
                key_path,
                provided_key,
            } => write!(
                f,
                "Key \"{}\" in locale {:?} is spelled \"{}\" in the default locale, both resolve to the same Rust identifier and are treated as the same key.",
                provided_key.key_path, provided_key.locale.key.name, key_path
            ),
            Warning::VarKeySpellingMismatch {
                loc,
                key1,
                key2,
                resolved_ident,
            } => {
                let (key1, key2, resolved_ident) = (
                    strip_prefix(key1, VAR_PREFIX),
                    strip_prefix(key2, VAR_PREFIX),
                    strip_prefix(resolved_ident, VAR_PREFIX),
                );
                write!(
                    f,
                    "Variable \"{key2}\" at {loc} is spelled \"{key1}\" elsewhere in this key, both resolve to the same Rust identifier \"{resolved_ident}\" and are treated as the same variable."
                )
            }
            Warning::CompKeySpellingMismatch {
                loc,
                key1,
                key2,
                resolved_ident,
                self_closed,
            } => {
                let (key1, key2, resolved_ident) = (
                    strip_prefix(key1, COMP_PREFIX),
                    strip_prefix(key2, COMP_PREFIX),
                    strip_prefix(resolved_ident, COMP_PREFIX),
                );
                let (open, close) = if *self_closed {
                    ("<", " />")
                } else {
                    ("<", ">")
                };
                write!(
                    f,
                    "Component {open}{key2}{close} at {loc} is spelled {open}{key1}{close} elsewhere in this key, both resolve to the same Rust identifier \"{resolved_ident}\" and are treated as the same component."
                )
            }
            Warning::PluralKeySpellingMismatch {
                key1,
                base1,
                key2,
                base2,
                resolved_ident,
            } => write!(
                f,
                "Plural keys \"{}\" and \"{}\" in locale {:?} spell their base differently (\"{}\" / \"{}\"), both resolve to the same Rust identifier \"{}\" and are treated as the same plural.",
                key1.key_path, key2, key1.locale.key.name, base1, base2, resolved_ident
            ),
            Warning::DuplicateFkArgs {
                loc,
                fk_target,
                arg_name,
            } => write!(
                f,
                "Argument {arg_name:?} is given more than once to foreign key \"{fk_target}\" at {loc}, only the first value is kept."
            ),
            Warning::CollidingFkArgs {
                loc,
                fk_target,
                arg1,
                arg2,
                ident_repr,
            } => write!(
                f,
                "Arguments {arg1:?} and {arg2:?} to foreign key \"{fk_target}\" at {loc} both resolve to the same Rust identifier {ident_repr:?}, only the first one ({arg1:?}) is kept."
            ),
        }
    }
}

#[derive(Default)]
pub struct Diagnostics {
    errors: RefCell<Vec<Error>>,
    warnings: RefCell<Vec<Warning>>,
}

pub trait IntoError {
    fn into_err(self) -> Error;
}

impl IntoError for Error {
    fn into_err(self) -> Error {
        self
    }
}

impl IntoError for BoxedError {
    fn into_err(self) -> Error {
        self.into_inner()
    }
}

impl Diagnostics {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn emit_error(&self, error: impl IntoError) {
        self.errors.borrow_mut().push(error.into_err());
    }

    pub fn emit_custom_error(&self, err: impl ToString) {
        self.emit_error(Error::custom(err));
    }

    pub fn emit_custom_warning(&self, err: impl ToString) {
        self.emit_warning(Warning::custom(err));
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

    pub fn extend(&self, other: Self) {
        let mut errors = self.errors.borrow_mut();
        let mut other_err = other.errors.borrow_mut();
        errors.append(&mut other_err);

        let mut warnings = self.warnings.borrow_mut();
        let mut other_warn = other.warnings.borrow_mut();
        warnings.append(&mut other_warn);
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

impl Debug for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let warnings = self.warnings.borrow();
        let errors = self.errors.borrow();
        f.debug_struct("Diagnostics")
            .field("warnings", &*warnings)
            .field("errors", &*errors)
            .finish()
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
