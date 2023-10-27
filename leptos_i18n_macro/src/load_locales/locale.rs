use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::{
    cfg_file::ConfigFile,
    error::{Error, Result},
    key::{Key, KeyPath},
    parsed_value::{InterpolateKey, ParsedValue, ParsedValueSeed},
    warning::{emit_warning, Warning},
};

#[cfg(feature = "yaml_files")]
const FILE_EXTS: &[&str] = &["yaml", "yml"];
#[cfg(feature = "json_files")]
const FILE_EXTS: &[&str] = &["json"];
#[cfg(not(any(feature = "json_files", feature = "yaml_files")))]
const FILE_EXTS: &[&str] = &[];

#[derive(Debug)]
pub struct Namespace {
    pub key: Rc<Key>,
    pub locales: Vec<Locale>,
}

#[derive(Debug)]
pub enum LocalesOrNamespaces {
    NameSpaces(Vec<Namespace>),
    Locales(Vec<Locale>),
}

#[derive(Default, Debug)]
pub struct BuildersKeysInner(pub HashMap<Rc<Key>, LocaleValue>);

pub enum BuildersKeys<'a> {
    NameSpaces {
        namespaces: &'a [Namespace],
        keys: HashMap<Rc<Key>, BuildersKeysInner>,
    },
    Locales {
        locales: &'a [Locale],
        keys: BuildersKeysInner,
    },
}

fn find_file(path: &mut PathBuf) -> Result<File> {
    let mut errs = vec![];

    for ext in FILE_EXTS {
        path.set_extension(ext);
        match File::open(&path) {
            Ok(file) => return Ok(file),
            Err(err) => {
                errs.push((path.to_owned(), err));
            }
        };
    }

    Err(Error::LocaleFileNotFound(errs))
}

impl Namespace {
    pub fn new(
        locales_dir_path: &mut PathBuf,
        key: Rc<Key>,
        locale_keys: &[Rc<Key>],
    ) -> Result<Self> {
        let mut locales = Vec::with_capacity(locale_keys.len());
        for locale in locale_keys.iter().cloned() {
            let file_path: &Path = key.name.as_ref();
            locales_dir_path.push(&locale.name);
            locales_dir_path.push(file_path);

            let locale_file = find_file(locales_dir_path)?;

            let locale = Locale::new(locale_file, locales_dir_path, locale, Some(Rc::clone(&key)))?;

            locales.push(locale);
            locales_dir_path.pop();
            locales_dir_path.pop();
        }
        Ok(Namespace { key, locales })
    }
}

impl LocalesOrNamespaces {
    pub fn get_value_at(&self, top_locale: &Rc<Key>, path: &KeyPath) -> Option<&'_ ParsedValue> {
        let locale = match (&path.namespace, self) {
            (None, LocalesOrNamespaces::NameSpaces(_))
            | (Some(_), LocalesOrNamespaces::Locales(_)) => None,
            (None, LocalesOrNamespaces::Locales(locales)) => {
                locales.iter().find(|locale| &locale.name == top_locale)
            }
            (Some(target_namespace), LocalesOrNamespaces::NameSpaces(namespaces)) => {
                let namespace = namespaces.iter().find(|ns| &ns.key == target_namespace)?;

                namespace
                    .locales
                    .iter()
                    .find(|locale| &locale.name == top_locale)
            }
        }?;

        locale.get_value_at(&path.path)
    }

    pub fn new(manifest_dir_path: &mut PathBuf, cfg_file: &ConfigFile) -> Result<Self> {
        let locale_keys = &cfg_file.locales;
        manifest_dir_path.push(&*cfg_file.locales_dir);
        if let Some(namespace_keys) = &cfg_file.name_spaces {
            let mut namespaces = Vec::with_capacity(namespace_keys.len());
            for namespace in namespace_keys {
                namespaces.push(Namespace::new(
                    manifest_dir_path,
                    Rc::clone(namespace),
                    locale_keys,
                )?);
            }
            Ok(LocalesOrNamespaces::NameSpaces(namespaces))
        } else {
            let mut locales = Vec::with_capacity(locale_keys.len());
            for locale in locale_keys.iter().cloned() {
                manifest_dir_path.push(&locale.name);
                let locale_file = find_file(manifest_dir_path)?;
                let locale = Locale::new(locale_file, manifest_dir_path, locale, None)?;
                locales.push(locale);
                manifest_dir_path.pop();
            }
            Ok(LocalesOrNamespaces::Locales(locales))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub top_locale_name: Rc<Key>,
    pub name: Rc<Key>,
    pub keys: HashMap<Rc<Key>, ParsedValue>,
}

impl Locale {
    pub fn get_value_at(&self, path: &[Rc<Key>]) -> Option<&'_ ParsedValue> {
        match path {
            [] => None,
            [key] => self.keys.get(key),
            [key, path @ ..] => {
                let value = self.keys.get(key)?;
                let ParsedValue::Subkeys(subkeys) = value else {
                    return None;
                };
                match subkeys {
                    None => unreachable!("called get_value_at on empty subkeys. If you got this error please open an issue on github."),
                    Some(subkeys) => subkeys.get_value_at(path)
                }
            }
        }
    }

    #[cfg(feature = "yaml_files")]
    fn de_inner(locale_file: File, seed: LocaleSeed) -> Result<Self, super::error::SerdeError> {
        let deserializer = serde_yaml::Deserializer::from_reader(locale_file);
        serde::de::DeserializeSeed::deserialize(seed, deserializer)
    }

    #[cfg(feature = "json_files")]
    fn de_inner(locale_file: File, seed: LocaleSeed) -> Result<Self, super::error::SerdeError> {
        let mut deserializer = serde_json::Deserializer::from_reader(locale_file);
        serde::de::DeserializeSeed::deserialize(seed, &mut deserializer)
    }

    #[cfg(not(any(feature = "json_files", feature = "yaml_files")))]
    fn de_inner(locale_file: File, seed: LocaleSeed) -> Result<Self, super::error::SerdeError> {
        let _ = (locale_file, seed);
        compile_error!("No file format has been provided, supported formats are: json and yaml")
    }

    fn de(locale_file: File, path: &mut PathBuf, seed: LocaleSeed) -> Result<Self> {
        Self::de_inner(locale_file, seed).map_err(|err| Error::LocaleFileDeser {
            path: std::mem::take(path),
            err,
        })
    }

    pub fn new(
        locale_file: File,
        path: &mut PathBuf,
        locale: Rc<Key>,
        namespace: Option<Rc<Key>>,
    ) -> Result<Self> {
        let seed = LocaleSeed {
            name: Rc::clone(&locale),
            top_locale_name: locale,
            key_path: KeyPath::new(namespace),
        };

        Self::de(locale_file, path, seed)
    }

    pub fn make_builder_keys(&mut self, key_path: &mut KeyPath) -> Result<BuildersKeysInner> {
        let mut keys = BuildersKeysInner::default();
        for (key, value) in &mut self.keys {
            value.reduce();
            key_path.push_key(Rc::clone(key));
            let locale_value = value.make_locale_value(key_path)?;
            let key = key_path
                .pop_key()
                .expect("Unexpected empty KeyPath in make_builder_keys. If you got this error please open an issue on github.");
            keys.0.insert(key, locale_value);
        }
        Ok(keys)
    }

    pub fn merge(
        &mut self,
        keys: &mut BuildersKeysInner,
        default_locale: &str,
        top_locale: Rc<Key>,
        key_path: &mut KeyPath,
    ) -> Result<()> {
        for (key, keys) in &mut keys.0 {
            key_path.push_key(Rc::clone(key));
            if let Some(value) = self.keys.get_mut(key) {
                value.merge(keys, default_locale, Rc::clone(&self.name), key_path)?;
            } else {
                emit_warning(Warning::MissingKey {
                    locale: top_locale.clone(),
                    key_path: key_path.clone(),
                });
            }
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
        locales: &mut [Locale],
        namespace: Option<Rc<Key>>,
    ) -> Result<BuildersKeysInner> {
        let mut locales = locales.iter_mut();
        let default_locale = locales.next().unwrap();
        let mut key_path = KeyPath::new(namespace);

        let mut default_keys = default_locale.make_builder_keys(&mut key_path)?;

        let default_locale_name = &default_locale.name.name;

        for locale in locales {
            let top_locale = locale.name.clone();
            locale.merge(
                &mut default_keys,
                default_locale_name,
                top_locale,
                &mut key_path,
            )?;
        }

        default_keys.check_conflicts(&mut key_path)?;

        Ok(default_keys)
    }

    pub fn check_locales(locales: &mut LocalesOrNamespaces) -> Result<BuildersKeys> {
        match locales {
            LocalesOrNamespaces::NameSpaces(namespaces) => {
                let mut keys = HashMap::with_capacity(namespaces.len());
                for namespace in &mut *namespaces {
                    let k = Self::check_locales_inner(
                        &mut namespace.locales,
                        Some(Rc::clone(&namespace.key)),
                    )?;
                    keys.insert(Rc::clone(&namespace.key), k);
                }
                Ok(BuildersKeys::NameSpaces { namespaces, keys })
            }
            LocalesOrNamespaces::Locales(locales) => {
                let keys = Self::check_locales_inner(locales, None)?;
                Ok(BuildersKeys::Locales { locales, keys })
            }
        }
    }
}

impl BuildersKeysInner {
    fn check_conflicts(&mut self, key_path: &mut KeyPath) -> Result<()> {
        for (key, values) in &mut self.0 {
            key_path.push_key(Rc::clone(key));
            values.check_conflicts(key_path)?;
            key_path.pop_key();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum LocaleValue {
    Value(Option<HashSet<InterpolateKey>>),
    Subkeys {
        locales: Vec<Locale>,
        keys: BuildersKeysInner,
    },
}

impl LocaleValue {
    fn check_conflicts(&mut self, key_path: &mut KeyPath) -> Result<()> {
        match self {
            LocaleValue::Value(Some(keys)) => {
                let mut iter = keys.iter();
                let Some(count_type) = iter.find_map(|key| match key {
                    InterpolateKey::Count(plural_type) => Some(*plural_type),
                    _ => None,
                }) else {
                    return Ok(());
                };

                let other_type = iter.find_map(|key| match key {
                    InterpolateKey::Count(plural_type) => Some(*plural_type),
                    _ => None,
                });

                if let Some(other_type) = other_type {
                    return Err(Error::PluralTypeMissmatch {
                        key_path: std::mem::take(key_path),
                        type1: count_type,
                        type2: other_type,
                    });
                }

                // if the set contains InterpolateKey::Count, remove variable keys with name "count"
                // ("var_count" with the rename)
                keys.retain(
                    |key| !matches!(key, InterpolateKey::Variable(key) if key.name == "var_count"),
                );

                Ok(())
            }
            LocaleValue::Value(None) => Ok(()),
            LocaleValue::Subkeys { keys, .. } => keys.check_conflicts(key_path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocaleSeed {
    pub name: Rc<Key>,
    pub top_locale_name: Rc<Key>,
    pub key_path: KeyPath,
}

impl<'de> serde::de::Visitor<'de> for LocaleSeed {
    type Value = HashMap<Rc<Key>, ParsedValue>;

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keys = HashMap::new();

        while let Some(locale_key) = map.next_key()? {
            self.key_path.push_key(Rc::clone(&locale_key));
            let value = map.next_value_seed(ParsedValueSeed {
                top_locale_name: &self.top_locale_name,
                key: &locale_key,
                key_path: &self.key_path,
                in_plural: false,
            })?;
            self.key_path.pop_key();
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

impl<'de> serde::de::DeserializeSeed<'de> for LocaleSeed {
    type Value = Locale;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keys = deserializer.deserialize_map(self.clone())?;
        let Self {
            name,
            top_locale_name,
            ..
        } = self;
        Ok(Locale {
            name,
            keys,
            top_locale_name,
        })
    }
}
