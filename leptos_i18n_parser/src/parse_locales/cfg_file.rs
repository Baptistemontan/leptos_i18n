use std::{borrow::Cow, collections::BTreeSet, path::PathBuf};

use super::error::{Error, Result};
use crate::utils::Key;

#[derive(Debug)]
pub struct ConfigFile {
    pub default: Key,
    pub locales: Vec<Key>,
    pub name_spaces: Option<Vec<Key>>,
    pub locales_dir: Cow<'static, str>,
    pub translations_uri: Option<String>,
}

impl ConfigFile {
    pub fn new(manifest_dir_path: &mut PathBuf) -> Result<ConfigFile> {
        manifest_dir_path.push("Cargo.toml");

        let cfg_file_str =
            std::fs::read_to_string(&manifest_dir_path).map_err(Error::ManifestNotFound)?;

        manifest_dir_path.pop();

        let Some((before, i18n_cfg)) = cfg_file_str.split_once("[package.metadata.leptos-i18n]")
        else {
            return Err(Error::ConfigNotPresent);
        };

        // this is to have the correct line number in the reported error.
        let cfg_file_whitespaced = before
            .chars()
            .filter(|c| *c == '\n')
            .chain(i18n_cfg.chars())
            .collect::<String>();

        let mut cfg: ConfigFile =
            toml::de::from_str(&cfg_file_whitespaced).map_err(Error::ConfigFileDeser)?;

        if let Some(i) = cfg.locales.iter().position(|l| l == &cfg.default) {
            // put default as first locale
            cfg.locales.swap(0, i);
        } else {
            let len = cfg.locales.len();
            cfg.locales.push(cfg.default.clone());
            cfg.locales.swap(0, len);
        }

        if let Some(duplicates) = Self::contain_duplicates(&cfg.locales) {
            Err(Error::DuplicateLocalesInConfig(duplicates))
        } else if let Some(duplicates) = cfg
            .name_spaces
            .as_deref()
            .and_then(Self::contain_duplicates)
        {
            Err(Error::DuplicateNamespacesInConfig(duplicates))
        } else {
            Ok(cfg)
        }
    }

    fn contain_duplicates(locales: &[Key]) -> Option<BTreeSet<Key>> {
        // monkey time

        let mut marked = BTreeSet::new();

        let mut duplicates = None;

        for key in locales {
            if !marked.insert(key) {
                duplicates
                    .get_or_insert_with(BTreeSet::new)
                    .insert(key.clone());
            }
        }

        duplicates
    }
}

// -----------------------------------------
// Deserialization
// -----------------------------------------

struct CfgFileVisitor;

impl<'de> serde::Deserialize<'de> for ConfigFile {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("ConfigFile", Field::FIELDS, CfgFileVisitor)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Field {
    Default,
    Locales,
    Namespaces,
    LocalesDir,
    TranslationsUri,
    Unknown,
}

impl Field {
    pub const DEFAULT: &'static str = "default";
    pub const LOCALES: &'static str = "locales";
    pub const NAMESPACES: &'static str = "namespaces";
    pub const LOCALES_DIR: &'static str = "locales-dir";
    pub const TRANSLATIONS_URI: &'static str = "translations-uri";
    pub const FIELDS: &'static [&'static str] = &[
        Self::DEFAULT,
        Self::LOCALES,
        Self::NAMESPACES,
        Self::LOCALES_DIR,
        Self::TRANSLATIONS_URI,
    ];
}

struct FieldVisitor;

impl<'de> serde::Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

impl serde::de::Visitor<'_> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "an identifier for the fields {:?}",
            Field::FIELDS
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            Field::DEFAULT => Ok(Field::Default),
            Field::LOCALES => Ok(Field::Locales),
            Field::NAMESPACES => Ok(Field::Namespaces),
            Field::LOCALES_DIR => Ok(Field::LocalesDir),
            Field::TRANSLATIONS_URI => Ok(Field::TranslationsUri),
            _ => Ok(Field::Unknown), // skip unknown fields
        }
    }
}

impl<'de> serde::de::Visitor<'de> for CfgFileVisitor {
    type Value = ConfigFile;

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        fn deser_field<'de, A, T>(
            option: &mut Option<T>,
            map: &mut A,
            field_name: &'static str,
        ) -> Result<(), A::Error>
        where
            A: serde::de::MapAccess<'de>,
            T: serde::de::DeserializeOwned,
        {
            if option.replace(map.next_value()?).is_some() {
                Err(serde::de::Error::duplicate_field(field_name))
            } else {
                Ok(())
            }
        }
        let mut default = None;
        let mut locales = None;
        let mut name_spaces = None;
        let mut locales_dir = None;
        let mut translations_uri = None;
        while let Some(field) = map.next_key::<Field>()? {
            match field {
                Field::Default => deser_field(&mut default, &mut map, Field::DEFAULT)?,
                Field::Locales => deser_field(&mut locales, &mut map, Field::LOCALES)?,
                Field::Namespaces => deser_field(&mut name_spaces, &mut map, Field::NAMESPACES)?,
                Field::LocalesDir => deser_field(&mut locales_dir, &mut map, Field::LOCALES_DIR)?,
                Field::TranslationsUri => {
                    deser_field(&mut translations_uri, &mut map, Field::TRANSLATIONS_URI)?
                }
                Field::Unknown => continue,
            }
        }
        let Some(default) = default else {
            return Err(serde::de::Error::missing_field("default"));
        };

        let Some(locales) = locales else {
            return Err(serde::de::Error::missing_field("locales"));
        };

        let locales_dir = locales_dir
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed("locales"));

        Ok(ConfigFile {
            default,
            locales,
            name_spaces,
            locales_dir,
            translations_uri,
        })
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a struct with fields \"default\" and \"locales\""
        )
    }
}
