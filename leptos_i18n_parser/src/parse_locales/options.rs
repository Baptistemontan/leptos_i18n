use crate::{
    formatters::{Formatter, Formatters},
    parse_locales::{
        cfg_file::DEFAULT_LOCALES_PATH,
        error::Result,
        locale::{Locale, LocaleSeed, SerdeError},
    },
    utils::Key,
};
use parser::Parser;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::Debug,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    pub default_locale: Key,
    pub locales: Vec<Key>,
    pub locales_path: Cow<'static, Path>,
    pub namespaces: Vec<Key>,
    pub translations_uri: Option<Cow<'static, str>>,
    pub extensions: BTreeMap<Key, Key>,
    pub options: ParseOptions,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ParseOptions {
    pub file_format: FileFormat,
    pub suppress_key_warnings: bool,
    pub interpolate_display: bool,
    pub show_keys_only: bool,
    pub formatters: Formatters,
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub enum FileFormat {
    #[default]
    Json,
    Json5,
    Yaml,
    Toml,
    Custom(Arc<dyn Parser>),
}

impl Debug for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::Json => f.write_str("Json"),
            FileFormat::Json5 => f.write_str("Json5"),
            FileFormat::Yaml => f.write_str("Yaml"),
            FileFormat::Toml => f.write_str("Toml"),
            FileFormat::Custom(..) => f.debug_tuple("Custom").finish(),
        }
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseOptions {
    pub fn new() -> Self {
        ParseOptions {
            file_format: FileFormat::Json,
            suppress_key_warnings: false,
            interpolate_display: false,
            show_keys_only: false,
            formatters: Formatters::new(),
        }
    }

    pub fn file_format(self, file_format: FileFormat) -> Self {
        Self {
            file_format,
            ..self
        }
    }

    pub fn suppress_key_warnings(self, suppress_key_warnings: bool) -> Self {
        Self {
            suppress_key_warnings,
            ..self
        }
    }

    pub fn interpolate_display(self, interpolate_display: bool) -> Self {
        Self {
            interpolate_display,
            ..self
        }
    }

    pub fn show_keys_only(self, show_keys_only: bool) -> Self {
        Self {
            show_keys_only,
            ..self
        }
    }

    pub fn with_custom_parser<P: Parser>(self, parser: P) -> Self {
        Self {
            file_format: FileFormat::Custom(Arc::new(parser)),
            ..self
        }
    }

    pub fn add_formatter<F: Formatter>(mut self, formatter: F) -> Self {
        self.formatters
            .insert_formatter(formatter)
            .expect("tried to overwrite an existing formatter.");
        self
    }
}

impl Config {
    pub fn new(default_locale: &str) -> Result<Self> {
        let default_locale = Key::try_new(default_locale)?;
        Ok(Config {
            locales: vec![default_locale.clone()],
            default_locale,
            locales_path: Cow::Borrowed(DEFAULT_LOCALES_PATH.as_ref()),
            namespaces: vec![],
            translations_uri: None,
            extensions: BTreeMap::new(),
            options: ParseOptions::default(),
        })
    }

    fn add_locale_inner(&mut self, loc: Key) {
        // TODO: check if already present and warn ?
        self.locales.push(loc);
    }

    pub fn add_locale(mut self, locale: &str) -> Result<Self> {
        let loc = Key::try_new(locale)?;
        self.add_locale_inner(loc);
        Ok(self)
    }

    pub fn add_locales<T: AsRef<str>>(
        mut self,
        locales: impl IntoIterator<Item = T>,
    ) -> Result<Self> {
        for locale in locales {
            self = self.add_locale(locale.as_ref())?;
        }
        Ok(self)
    }

    pub fn locales_path(self, path: impl ToPathCow<'static>) -> Self {
        Self {
            locales_path: path.into_cow(),
            ..self
        }
    }

    pub fn add_namespace(mut self, namespace: &str) -> Result<Self> {
        self.namespaces.push(Key::try_new(namespace)?);
        Ok(self)
    }

    pub fn add_namespaces<T: AsRef<str>>(
        mut self,
        locales: impl IntoIterator<Item = T>,
    ) -> Result<Self> {
        for locale in locales {
            self = self.add_namespace(locale.as_ref())?;
        }
        Ok(self)
    }

    pub fn translations_uri(self, uri: impl ToStrCow<'static>) -> Self {
        Self {
            translations_uri: Some(uri.into_cow()),
            ..self
        }
    }

    pub fn extend_locale(mut self, locale_to_extend: &str, inherit_from: &str) -> Result<Self> {
        let loc_to_ext = Key::try_new(locale_to_extend)?;
        let inh_from = Key::try_new(inherit_from)?;

        // TODO: check if `locale_to_extend` is the default
        // TODO: check if both locales are known

        self.extensions.insert(loc_to_ext, inh_from);
        Ok(self)
    }

    pub fn parse_options(self, options: ParseOptions) -> Self {
        Self { options, ..self }
    }
}

impl FileFormat {
    pub fn get_files_exts(&self) -> &'static [&'static str] {
        match self {
            FileFormat::Json => &["json"],
            FileFormat::Json5 => &["json5"],
            FileFormat::Yaml => &["yaml", "yml"],
            FileFormat::Toml => &["toml"],
            FileFormat::Custom(parser) => parser.file_extensions(),
        }
    }

    pub fn deserialize<R: Read>(
        &self,
        locale_file: R,
        path: &Path,
        seed: LocaleSeed,
    ) -> Result<Locale, SerdeError> {
        match self {
            FileFormat::Json => de_json(locale_file, seed),
            FileFormat::Json5 => de_json5(locale_file, seed),
            FileFormat::Yaml => de_yaml(locale_file, seed),
            FileFormat::Toml => de_toml(locale_file, seed),
            FileFormat::Custom(parser) => parser::de_custom(&**parser, locale_file, path, seed),
        }
    }
}

fn de_json<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
    let mut deserializer = serde_json::Deserializer::from_reader(locale_file);
    serde::de::DeserializeSeed::deserialize(seed, &mut deserializer).map_err(SerdeError::Json)
}

fn de_json5<R: Read>(mut locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
    let mut buff = String::new();
    Read::read_to_string(&mut locale_file, &mut buff).map_err(SerdeError::Io)?;
    let mut deserializer = json5::Deserializer::from_str(&buff);
    serde::de::DeserializeSeed::deserialize(seed, &mut deserializer).map_err(SerdeError::Json5)
}

fn de_yaml<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
    let deserializer = serde_yaml::Deserializer::from_reader(locale_file);
    serde::de::DeserializeSeed::deserialize(seed, deserializer).map_err(SerdeError::Yaml)
}

fn de_toml<R: Read>(mut locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
    let mut buf = String::new();
    locale_file
        .read_to_string(&mut buf)
        .map_err(SerdeError::Io)?;
    let deserializer = toml::Deserializer::parse(&buf).map_err(SerdeError::Toml)?;
    serde::de::DeserializeSeed::deserialize(seed, deserializer).map_err(SerdeError::Toml)
}

pub mod parser {
    use crate::parse_locales::locale::{Locale, LocaleSeed};
    use std::{io::Read, path::Path};

    pub use crate::parse_locales::locale::SerdeError;

    pub struct Seed<'a>(LocaleSeed<'a>);

    impl<'de> serde::de::DeserializeSeed<'de> for Seed<'_> {
        type Value = Value;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            serde::de::DeserializeSeed::deserialize(self.0, deserializer).map(Value)
        }
    }

    pub struct Value(Locale);

    pub trait Parser: 'static {
        fn deserialize(
            &self,
            reader: &mut dyn Read,
            path: &Path,
            seed: Seed,
        ) -> Result<Value, SerdeError>;

        fn file_extensions(&self) -> &'static [&'static str];
    }

    pub(crate) fn de_custom<R: Read>(
        parser: &dyn Parser,
        mut reader: R,
        path: &Path,
        seed: LocaleSeed,
    ) -> Result<Locale, SerdeError> {
        let seed = Seed(seed);
        let value = parser.deserialize(&mut reader, path, seed)?;
        Ok(value.0)
    }
}

pub trait ToPathCow<'a> {
    fn into_cow(self) -> Cow<'a, Path>;
}

impl<'a> ToPathCow<'a> for &'a str {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'a> ToPathCow<'a> for &'a Path {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Borrowed(self)
    }
}

impl<'a> ToPathCow<'a> for String {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Owned(self.into())
    }
}

impl<'a> ToPathCow<'a> for PathBuf {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Owned(self)
    }
}

pub trait ToStrCow<'a> {
    fn into_cow(self) -> Cow<'a, str>;
}

impl<'a> ToStrCow<'a> for &'a str {
    fn into_cow(self) -> Cow<'a, str> {
        Cow::Borrowed(self)
    }
}

impl<'a> ToStrCow<'a> for String {
    fn into_cow(self) -> Cow<'a, str> {
        Cow::Owned(self)
    }
}
