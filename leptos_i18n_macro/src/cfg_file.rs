use crate::{
    error::{Error, Result},
    key::{Key, KeySeed, KeyVecSeed},
};
use std::{collections::HashSet, fs::File, path::Path};

#[derive(Debug)]
pub struct ConfigFile {
    pub default: Key,
    pub locales: Vec<Key>,
}

impl ConfigFile {
    fn contain_duplicates(locales: &[Key]) -> Option<HashSet<String>> {
        // monkey time

        let mut marked = HashSet::with_capacity(locales.len());

        let mut duplicates = None;

        for key in locales {
            if !marked.insert(key) {
                duplicates
                    .get_or_insert_with(HashSet::new)
                    .insert(key.name.clone());
            }
        }

        duplicates
    }

    pub fn new<T: AsRef<Path>>(path: Option<T>) -> Result<ConfigFile> {
        let path = path
            .as_ref()
            .map(|path| path.as_ref())
            .unwrap_or("./i18n.json".as_ref());
        let cfg_file = File::open(path).map_err(Error::ConfigFileNotFound)?;

        let cfg: ConfigFile = serde_json::from_reader(cfg_file).map_err(Error::ConfigFileDeser)?;

        if !cfg.locales.contains(&cfg.default) {
            Err(Error::ConfigFileDefaultMissing(cfg))
        } else if let Some(duplicates) = Self::contain_duplicates(&cfg.locales) {
            Err(Error::DuplicateLocalesInConfig(duplicates))
        } else {
            Ok(cfg)
        }
    }
}

struct CfgFileVisitor;

impl<'de> serde::Deserialize<'de> for ConfigFile {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("ConfigFile", &["default", "locales"], CfgFileVisitor)
    }
}

enum Field {
    Default,
    Locales,
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

impl<'de> serde::de::Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "an identifier for the field \"default\" or the field \"locales\""
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "default" => Ok(Field::Default),
            "locales" => Ok(Field::Locales),
            _ => Err(E::unknown_field(v, &["default", "locales"])),
        }
    }
}

impl<'de> serde::de::Visitor<'de> for CfgFileVisitor {
    type Value = ConfigFile;

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let Some(first_field) = map.next_key::<Field>()? else {
            return Err(serde::de::Error::missing_field("default"));
        };

        match first_field {
            Field::Default => {
                let default = map.next_value_seed(KeySeed::LocaleName)?;
                match map.next_key::<Field>()? {
                    None => return Err(serde::de::Error::missing_field("locales")),
                    Some(Field::Default) => {
                        return Err(serde::de::Error::duplicate_field("default"))
                    }
                    _ => {}
                }

                let locales = map.next_value_seed(KeyVecSeed(KeySeed::LocaleName))?;

                Ok(ConfigFile { default, locales })
            }
            Field::Locales => {
                let locales = map.next_value_seed(KeyVecSeed(KeySeed::LocaleName))?;
                match map.next_key::<Field>()? {
                    None => return Err(serde::de::Error::missing_field("default")),
                    Some(Field::Locales) => {
                        return Err(serde::de::Error::duplicate_field("locales"))
                    }
                    _ => {}
                }

                let default = map.next_value_seed(KeySeed::LocaleName)?;

                Ok(ConfigFile { default, locales })
            }
        }
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a struct with fields \"default\" and \"locales\""
        )
    }
}
