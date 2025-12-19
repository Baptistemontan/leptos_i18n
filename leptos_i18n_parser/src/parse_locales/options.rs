use crate::parse_locales::locale::{Locale, LocaleSeed, SerdeError};
use parser::Parser;
use std::{fmt::Debug, io::Read, path::Path, sync::Arc};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParseOptions {
    pub file_format: FileFormat,
    pub suppress_key_warnings: bool,
    pub interpolate_display: bool,
    pub show_keys_only: bool,
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub enum FileFormat {
    #[default]
    Json,
    Json5,
    Yaml,
    Custom(Arc<dyn Parser>),
}

impl Debug for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::Json => f.write_str("Json"),
            FileFormat::Json5 => f.write_str("Json5"),
            FileFormat::Yaml => f.write_str("Yaml"),
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
}

impl FileFormat {
    pub fn get_files_exts(&self) -> &'static [&'static str] {
        match self {
            FileFormat::Json => &["json"],
            FileFormat::Json5 => &["json5"],
            FileFormat::Yaml => &["yaml", "yml"],
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
