use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
    rc::Rc,
};

use serde::de::DeserializeSeed;

use super::{
    cfg_file::ConfigFile,
    error::{Error, Result},
    key::{Key, KeyPath},
    parsed_value::{InterpolateKey, ParsedValue, ParsedValueSeed},
    warning::{emit_warning, Warning},
};

pub struct Namespace {
    pub key: Rc<Key>,
    pub locales: Vec<Rc<RefCell<Locale>>>,
}

pub enum LocalesOrNamespaces {
    NameSpaces(Vec<Namespace>),
    Locales(Vec<Rc<RefCell<Locale>>>),
}

#[derive(Default)]
pub struct BuildersKeysInner(pub HashMap<Rc<Key>, LocaleValue>);

pub enum BuildersKeys {
    NameSpaces {
        namespaces: Vec<Namespace>,
        keys: HashMap<Rc<Key>, BuildersKeysInner>,
    },
    Locales {
        locales: Vec<Rc<RefCell<Locale>>>,
        keys: BuildersKeysInner,
    },
}

impl Namespace {
    pub fn new<P: AsRef<Path>>(
        manifest_dir_path: P,
        locales_dir: &Path,
        key: Rc<Key>,
        locale_keys: &[Rc<Key>],
    ) -> Result<Self> {
        let mut locales = Vec::with_capacity(locale_keys.len());
        let path = manifest_dir_path.as_ref().join(locales_dir);
        for locale in locale_keys.iter().cloned() {
            let path = path
                .join(&locale.name)
                .join(&key.name)
                .with_extension("json");
            locales.push(Rc::new(RefCell::new(Locale::new(path, locale)?)));
        }
        Ok(Namespace { key, locales })
    }
}

impl LocalesOrNamespaces {
    pub fn new<P: AsRef<Path>>(manifest_dir_path: P, cfg_file: &ConfigFile) -> Result<Self> {
        let locale_keys = &cfg_file.locales;
        let locales_dir: &str = cfg_file.locales_dir.as_ref();
        let locales_dir = locales_dir.as_ref();
        let manifest_dir_path = manifest_dir_path.as_ref();
        if let Some(namespace_keys) = &cfg_file.name_spaces {
            let mut namespaces = Vec::with_capacity(namespace_keys.len());
            for namespace in namespace_keys {
                namespaces.push(Namespace::new(
                    manifest_dir_path,
                    locales_dir,
                    Rc::clone(namespace),
                    locale_keys,
                )?);
            }
            Ok(LocalesOrNamespaces::NameSpaces(namespaces))
        } else {
            let mut locales = Vec::with_capacity(locale_keys.len());
            for locale in locale_keys.iter().cloned() {
                let path = manifest_dir_path
                    .join(locales_dir)
                    .join(&locale.name)
                    .with_extension("json");
                locales.push(Rc::new(RefCell::new(Locale::new(path, locale)?)));
            }
            Ok(LocalesOrNamespaces::Locales(locales))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub name: Rc<Key>,
    pub keys: HashMap<Rc<Key>, Rc<ParsedValue>>,
}

impl Locale {
    pub fn new<P: AsRef<Path>>(path: P, locale: Rc<Key>) -> Result<Self> {
        let path = path.as_ref();
        let locale_file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                return Err(Error::LocaleFileNotFound {
                    path: path.to_owned(),
                    err,
                })
            }
        };

        let mut deserializer = serde_json::Deserializer::from_reader(locale_file);

        LocaleSeed(locale)
            .deserialize(&mut deserializer)
            .map_err(|err| Error::LocaleFileDeser {
                path: path.to_owned(),
                err,
            })
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
        &mut self,
        keys: &mut BuildersKeysInner,
        default_locale: &str,
        default_values: &Self,
        top_locale: Rc<Key>,
        key_path: &mut KeyPath,
    ) -> Result<()> {
        for (key, keys) in &mut keys.0 {
            let default_value = default_values.keys.get(key).unwrap();
            key_path.push_key(Rc::clone(key));
            let locale = self.name.clone();
            let value_entry = self.keys.entry(Rc::clone(key));
            let value = value_entry.or_insert_with(|| {
                emit_warning(Warning::MissingKey {
                    locale: top_locale.clone(),
                    key_path: key_path.clone(),
                });
                Rc::clone(default_value)
            });
            value.merge(keys, default_locale, default_value, locale, key_path)?;
            key_path.pop_key();
        }

        // reverse key comparaison
        for key in self.keys.keys() {
            if keys.0.get(key).is_none() {
                key_path.push_key(Rc::clone(key));
                emit_warning(Warning::SurplusKey {
                    locale: top_locale.clone(),
                    key_path: key_path.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn check_locales_inner(
        locales: &[Rc<RefCell<Locale>>],
        namespace: Option<Rc<Key>>,
    ) -> Result<BuildersKeysInner> {
        let mut locales = locales.iter();
        let default_locale = locales.next().unwrap();
        let default_locale_ref = default_locale.borrow();

        let mut default_keys = default_locale_ref.to_builder_keys();

        let default_locale_name = &default_locale_ref.name.name;

        let mut key_path = KeyPath::new(namespace);

        for locale in locales {
            let top_locale = locale.borrow().name.clone();
            locale.borrow_mut().merge(
                &mut default_keys,
                default_locale_name,
                &default_locale.borrow(),
                top_locale,
                &mut key_path,
            )?;
        }

        Ok(default_keys)
    }

    pub fn check_locales(locales: LocalesOrNamespaces) -> Result<BuildersKeys> {
        match locales {
            LocalesOrNamespaces::NameSpaces(namespaces) => {
                let mut keys = HashMap::with_capacity(namespaces.len());
                for namespace in &namespaces {
                    let k = Self::check_locales_inner(
                        &namespace.locales,
                        Some(Rc::clone(&namespace.key)),
                    )?;
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
        locales: Vec<Rc<RefCell<Locale>>>,
        keys: BuildersKeysInner,
    },
}

#[derive(Debug, Clone)]
pub struct LocaleSeed(pub Rc<Key>);

impl<'de> serde::de::Visitor<'de> for LocaleSeed {
    type Value = HashMap<Rc<Key>, Rc<ParsedValue>>;

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
            keys.insert(locale_key, Rc::new(value));
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
