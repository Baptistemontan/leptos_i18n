use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use serde::de::DeserializeSeed;

use crate::{
    error::{Error, Result},
    key::{Key, KeySeed},
    parsed_value::{InterpolateKey, ParsedValue, ParsedValueSeed},
};

pub struct Locale {
    pub name: Key,
    pub keys: HashMap<Key, ParsedValue>,
}

impl Locale {
    pub fn new<T: AsRef<Path>>(path: T, locale: &Key) -> Result<Self> {
        let locale_file =
            File::open(path).map_err(|err| Error::LocaleFileNotFound(locale.name.clone(), err))?;

        let mut deserializer = serde_json::Deserializer::from_reader(locale_file);

        let seed = LocaleSeed {
            locale_name: locale,
        };

        seed.deserialize(&mut deserializer)
            .map_err(|err| Error::LocaleFileDeser(locale.name.clone(), err))
    }

    pub fn get_keys(&self) -> HashSet<&Key> {
        self.keys.keys().collect()
    }

    fn key_missmatch(
        locale1: &Self,
        keys1: &HashSet<&Key>,
        locale2: &Self,
        keys2: &HashSet<&Key>,
    ) -> Error {
        let mut locale = locale2;

        let mut diff = keys1
            .difference(keys2)
            .map(|key| key.name.clone())
            .collect::<Vec<_>>();

        if diff.is_empty() {
            locale = locale1;
            diff = keys2
                .difference(keys1)
                .map(|key| key.name.clone())
                .collect();
        }

        Error::MissingKeysInLocale {
            keys: diff,
            locale: locale.name.name.clone(),
        }
    }

    pub fn check_locales<'a, I>(locales: I) -> Result<HashMap<&'a Key, LocaleValue<'a>>>
    where
        I: IntoIterator<Item = &'a Self>,
    {
        let mut locales = locales.into_iter();
        let first_locale = locales.next().unwrap();

        let first_locale_keys = first_locale.get_keys();

        let mut mapped_keys: HashMap<_, _> = first_locale
            .keys
            .iter()
            .map(|(key, value)| (key, value.get_keys()))
            .collect();

        for locale in locales {
            let keys = locale.get_keys();
            if first_locale_keys != keys {
                return Err(Self::key_missmatch(
                    first_locale,
                    &first_locale_keys,
                    locale,
                    &keys,
                ));
            }

            for (key, key_kind) in &mut mapped_keys {
                if let Some(value) = locale.keys.get(key) {
                    value.get_keys_inner(key_kind)
                }
            }
        }

        let iter = mapped_keys.values_mut().filter_map(Option::as_mut);

        for keys in iter {
            if keys.contains(&InterpolateKey::Count) {
                // if the set contains InterpolateKey::Count, remove variable keys with name "count"
                // ("var_count" with the rename)
                keys.retain(
                    |key| !matches!(key, InterpolateKey::Variable(key) if key.name == "var_count"),
                );
            }
        }

        Ok(mapped_keys
            .into_iter()
            .map(|(key, value)| (key, LocaleValue::new(value)))
            .collect())
    }
}

#[derive(PartialEq, Eq)]
pub enum LocaleValue<'a> {
    String,
    Builder(HashSet<InterpolateKey<'a>>),
}

impl<'a> LocaleValue<'a> {
    fn new(value: Option<HashSet<InterpolateKey<'a>>>) -> Self {
        match value {
            Some(keys) => Self::Builder(keys),
            None => Self::String,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LocaleSeed<'a> {
    locale_name: &'a Key,
}

impl<'a: 'de, 'de> serde::de::Visitor<'de> for LocaleSeed<'a> {
    type Value = HashMap<Key, ParsedValue>;

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keys = HashMap::new();
        let locale_name = self.locale_name.name.as_str();

        while let Some(key) = map.next_key_seed(KeySeed::LocaleKey(locale_name))? {
            let parsed_value_seed = ParsedValueSeed {
                in_plural: false,
                locale_name,
                locale_key: &key.name,
            };
            let value = map.next_value_seed(parsed_value_seed)?;
            keys.insert(key, value);
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

impl<'a: 'de, 'de> serde::de::DeserializeSeed<'de> for LocaleSeed<'a> {
    type Value = Locale;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keys = deserializer.deserialize_map(self)?;
        let name = self.locale_name.clone();
        Ok(Locale { name, keys })
    }
}
