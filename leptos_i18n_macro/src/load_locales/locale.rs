use std::{
    collections::{HashMap, HashSet},
    fs::File,
    rc::Rc,
};

use serde::de::DeserializeSeed;

use super::{
    cfg_file::ConfigFile,
    error::{Error, Result},
    key::{Key, KeyPath},
    parsed_value::{InterpolateKey, ParsedValue, ParsedValueSeed},
};

pub struct Namespace {
    pub key: Rc<Key>,
    pub locales: Vec<Rc<Locale>>,
}

pub enum LocalesOrNamespaces {
    NameSpaces(Vec<Namespace>),
    Locales(Vec<Rc<Locale>>),
}

#[derive(Default)]
pub struct BuildersKeysInner(pub HashMap<Rc<Key>, LocaleValue>);

pub enum BuildersKeys {
    NameSpaces {
        namespaces: Vec<Namespace>,
        keys: HashMap<Rc<Key>, BuildersKeysInner>,
    },
    Locales {
        locales: Vec<Rc<Locale>>,
        keys: BuildersKeysInner,
    },
}

impl Namespace {
    pub fn new(locales_dir: &str, key: Rc<Key>, locale_keys: &[Rc<Key>]) -> Result<Self> {
        let mut locales = Vec::with_capacity(locale_keys.len());
        for locale in locale_keys.iter().cloned() {
            let path = format!("{}/{}/{}.json", locales_dir, locale.name, key.name);
            locales.push(Rc::new(Locale::new(path, locale)?));
        }
        Ok(Namespace { key, locales })
    }
}

impl LocalesOrNamespaces {
    pub fn new(cfg_file: &ConfigFile) -> Result<Self> {
        let locale_keys = &cfg_file.locales;
        let locales_dir = cfg_file.locales_dir.as_ref();
        if let Some(namespace_keys) = &cfg_file.name_spaces {
            let mut namespaces = Vec::with_capacity(namespace_keys.len());
            for namespace in namespace_keys {
                namespaces.push(Namespace::new(
                    locales_dir,
                    Rc::clone(namespace),
                    locale_keys,
                )?);
            }
            Ok(LocalesOrNamespaces::NameSpaces(namespaces))
        } else {
            let mut locales = Vec::with_capacity(locale_keys.len());
            for locale in locale_keys.iter().cloned() {
                let path = format!("{}/{}.json", locales_dir, locale.name);
                locales.push(Rc::new(Locale::new(path, locale)?));
            }
            Ok(LocalesOrNamespaces::Locales(locales))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub name: Rc<Key>,
    pub keys: HashMap<Rc<Key>, ParsedValue>,
}

impl Locale {
    pub fn new(path: String, locale: Rc<Key>) -> Result<Self> {
        let locale_file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => return Err(Error::LocaleFileNotFound { path, err }),
        };

        let mut deserializer = serde_json::Deserializer::from_reader(locale_file);

        LocaleSeed(locale)
            .deserialize(&mut deserializer)
            .map_err(|err| Error::LocaleFileDeser { path, err })
    }

    pub fn to_builder_keys(&self) -> BuildersKeysInner {
        let mut keys = BuildersKeysInner::default();
        for (key, value) in &self.keys {
            let locale_value = value.to_locale_value();
            keys.0.insert(Rc::clone(key), locale_value);
        }
        keys
    }

    pub fn merge(
        &self,
        keys: &mut BuildersKeysInner,
        locale1: &str,
        locale2: &str,
        namespace: Option<&str>,
        key_path: &mut KeyPath,
    ) -> Result<()> {
        for (key, keys) in &mut keys.0 {
            key_path.push_key(Rc::clone(key));
            let Some(value) = self.keys.get(key) else {
                return Err(Error::MissingKeyInLocale {
                    locale: locale2.to_string(),
                    namespace: namespace.map(str::to_string),
                    key_path: std::mem::take(key_path),
                });
            };
            value.merge(keys, locale1, locale2, namespace, key_path)?;
            key_path.pop_key();
        }

        // reverse key comparaison
        for key in self.keys.keys() {
            if keys.0.get(key).is_none() {
                key_path.push_key(Rc::clone(key));
                return Err(Error::MissingKeyInLocale {
                    locale: locale1.to_string(),
                    namespace: namespace.map(str::to_string),
                    key_path: std::mem::take(key_path),
                });
            }
        }

        Ok(())
    }

    pub fn check_locales_inner(
        locales: &[Rc<Locale>],
        namespace: Option<&str>,
    ) -> Result<BuildersKeysInner> {
        let mut locales = locales.iter();
        let first_locale = locales.next().unwrap();

        let mut first_locale_keys = first_locale.to_builder_keys();

        let locale1 = &first_locale.name.name;

        let mut key_path = KeyPath::default();

        for locale in locales {
            let locale2 = &locale.name.name;
            locale.merge(
                &mut first_locale_keys,
                locale1,
                locale2,
                namespace,
                &mut key_path,
            )?;
        }

        Ok(first_locale_keys)
    }

    pub fn check_locales(locales: LocalesOrNamespaces) -> Result<BuildersKeys> {
        match locales {
            LocalesOrNamespaces::NameSpaces(namespaces) => {
                let mut keys = HashMap::with_capacity(namespaces.len());
                for namespace in &namespaces {
                    let k =
                        Self::check_locales_inner(&namespace.locales, Some(&namespace.key.name))?;
                    keys.insert(Rc::clone(&namespace.key), k);
                }
                Ok(BuildersKeys::NameSpaces { namespaces, keys })
            }
            LocalesOrNamespaces::Locales(locales) => {
                let keys = Self::check_locales_inner(&locales, None)?;
                Ok(BuildersKeys::Locales { locales, keys })
            }
        }
    }
}

pub enum LocaleValue {
    Value(Option<HashSet<InterpolateKey>>),
    Subkeys {
        locales: Vec<Rc<Locale>>,
        keys: BuildersKeysInner,
    },
}

#[derive(Debug, Clone)]
pub struct LocaleSeed(pub Rc<Key>);

impl<'de> serde::de::Visitor<'de> for LocaleSeed {
    type Value = HashMap<Rc<Key>, ParsedValue>;

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keys = HashMap::new();

        while let Some(locale_key) = map.next_key()? {
            let value = map.next_value_seed(ParsedValueSeed {
                key: &locale_key,
                in_plural: false,
            })?;
            keys.insert(locale_key, value);
        }

        Ok(keys)
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a map of string keys and value either string or map"
        )
    }
}

impl<'a: 'de, 'de> serde::de::DeserializeSeed<'de> for LocaleSeed {
    type Value = Locale;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keys = deserializer.deserialize_map(self.clone())?;
        Ok(Locale { name: self.0, keys })
    }
}
