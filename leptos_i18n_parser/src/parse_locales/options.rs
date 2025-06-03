use std::io::Read;

use crate::parse_locales::locale::{Locale, LocaleSeed, SerdeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Options {
    pub file_format: FileFormat,
    pub suppress_key_warnings: bool,
    pub interpolate_display: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FileFormat {
    #[default]
    Json,
    Json5,
    Yaml,
}

impl Default for Options {
    fn default() -> Self {
        Self::default()
    }
}

impl Options {
    /// const-friendly version of default.
    pub const fn default() -> Self {
        Options {
            file_format: FileFormat::Json,
            suppress_key_warnings: false,
            interpolate_display: false,
        }
    }

    pub const fn set_format(self, file_format: FileFormat) -> Self {
        Self {
            file_format,
            ..self
        }
    }

    pub const fn set_suppress_key_warnings(self, suppress_key_warnings: bool) -> Self {
        Self {
            suppress_key_warnings,
            ..self
        }
    }

    pub const fn set_interpolate_display(self, interpolate_display: bool) -> Self {
        Self {
            interpolate_display,
            ..self
        }
    }
}

impl FileFormat {
    pub const fn get_files_exts(self) -> &'static [&'static str] {
        match self {
            FileFormat::Json => &["json"],
            FileFormat::Json5 => &["json5"],
            FileFormat::Yaml => &["yaml", "yml"],
        }
    }

    pub fn deserialize<R: Read>(
        self,
        locale_file: R,
        seed: LocaleSeed,
    ) -> Result<Locale, SerdeError> {
        match self {
            FileFormat::Json => de_json(locale_file, seed),
            FileFormat::Json5 => de_json5(locale_file, seed),
            FileFormat::Yaml => de_yaml(locale_file, seed),
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
    let mut deserializer = json5::Deserializer::from_str(&buff).map_err(SerdeError::Json5)?;
    serde::de::DeserializeSeed::deserialize(seed, &mut deserializer).map_err(SerdeError::Json5)
}

fn de_yaml<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
    let deserializer = serde_yaml::Deserializer::from_reader(locale_file);
    serde::de::DeserializeSeed::deserialize(seed, deserializer).map_err(SerdeError::Yaml)
}
