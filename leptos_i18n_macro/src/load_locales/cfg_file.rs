use super::{
    error::{Error, Result},
    key::{Key, KeySeed, KeyVecSeed},
};
use std::{collections::HashSet, fs::File, path::Path};

#[derive(Debug)]
pub struct ConfigFile {
    pub default: Key,
    pub locales: Vec<Key>,
    pub name_spaces: Option<Vec<Key>>,
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
    Namespaces,
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
            "namespaces" => Ok(Field::Namespaces),
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
        let mut default = None;
        let mut locales = None;
        let mut name_spaces = None;
        while let Some(field) = map.next_key::<Field>()? {
            match field {
                Field::Default => {
                    if default
                        .replace(map.next_value_seed(KeySeed::LocaleName)?)
                        .is_some()
                    {
                        return Err(serde::de::Error::duplicate_field("default"));
                    }
                }
                Field::Locales => {
                    if locales
                        .replace(map.next_value_seed(KeyVecSeed(KeySeed::LocaleName))?)
                        .is_some()
                    {
                        return Err(serde::de::Error::duplicate_field("locales"));
                    }
                }
                Field::Namespaces => {
                    if name_spaces
                        .replace(map.next_value_seed(KeyVecSeed(KeySeed::Namespace))?)
                        .is_some()
                    {
                        return Err(serde::de::Error::duplicate_field("locales"));
                    }
                }
            }
        }
        let Some(default) = default else {
            return Err(serde::de::Error::missing_field("default"));
        };

        let Some(locales) = locales else {
            return Err(serde::de::Error::missing_field("locales"));
        };

        Ok(ConfigFile {
            default,
            locales,
            name_spaces,
        })
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a struct with fields \"default\" and \"locales\""
        )
    }
}
