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

    // pub fn new(locale: &'a RawLocale) -> Result<Self> {
    //     let RawLocale {
    //         name: locale_name,
    //         keys,
    //     } = locale;
    //     let name = Key::new(locale.name, crate::key::KeyKind::LocaleName)?;
    //     let keys = keys
    //         .iter()
    //         .map(|(key, value)| -> Result<(Key, ParsedValue)> {
    //             let key = Key::new(key, crate::key::KeyKind::LocaleKey { locale_name })?;
    //             let value = ParsedValue::new(value);
    //             Ok((key, value))
    //         })
    //         .collect::<Result<_>>()?;

    //     Ok(Locale { name, keys })
    // }

    // pub fn get_keys(&self) -> HashMap<&Key, LocaleValue> {
    //     self.keys
    //         .iter()
    //         .map(|(key, value)| (key, value.get_keys().into()))
    //         .collect()
    // }

    // fn key_diff(
    //     locale1: &Locale,
    //     keys1: &HashMap<&Key, LocaleValue>,
    //     locale2: &Locale,
    //     keys2: &HashMap<&Key, LocaleValue>,
    // ) -> Error {
    //     // check key mismatch

    //     let keys_not_in_2 = keys1
    //         .iter()
    //         .map(|(key, _)| key)
    //         .filter(|key| keys2.get(*key).is_none())
    //         .map(|key| key.name.to_string())
    //         .collect::<Vec<_>>();
    //     if !keys_not_in_2.is_empty() {
    //         return Error::MissingKeysInLocale {
    //             keys: keys_not_in_2,
    //             locale: locale2.name.name.to_string(),
    //         };
    //     }

    //     let keys_not_in_1 = keys2
    //         .iter()
    //         .map(|(key, _)| key)
    //         .filter(|key| keys1.get(*key).is_none())
    //         .map(|key| key.name.to_string())
    //         .collect::<Vec<_>>();
    //     if !keys_not_in_1.is_empty() {
    //         return Error::MissingKeysInLocale {
    //             keys: keys_not_in_1,
    //             locale: locale1.name.name.to_string(),
    //         };
    //     }

    //     // check key kind mismatch
    //     let (key, value, other_value) = keys1
    //         .iter()
    //         .filter_map(|(key, value)| {
    //             let other_value = keys2.get(*key)?;
    //             (value != other_value).then_some((key, value, other_value))
    //         })
    //         .next()
    //         .expect("error was reported but everyhting seems fine...");

    //     match (value, other_value) {
    //         (LocaleValue::String, LocaleValue::Interpolate(_)) => Error::MismatchLocaleKeyKind {
    //             key: key.name.to_string(),
    //             locale_str: locale1.name.name.to_string(),
    //             locale_inter: locale2.name.name.to_string(),
    //         },
    //         (LocaleValue::Interpolate(_), LocaleValue::String) => Error::MismatchLocaleKeyKind {
    //             key: key.name.to_string(),
    //             locale_str: locale2.name.name.to_string(),
    //             locale_inter: locale1.name.name.to_string(),
    //         },
    //         (LocaleValue::Interpolate(keys1), LocaleValue::Interpolate(keys2)) => {
    //             let comp_keys1 = keys1
    //                 .iter()
    //                 .filter_map(|key| match key {
    //                     InterpolateKey::Variable(_) => None,
    //                     InterpolateKey::Component(key) => Some(key.name.to_string()),
    //                 })
    //                 .collect();

    //             let comp_keys2 = keys2
    //                 .iter()
    //                 .filter_map(|key| match key {
    //                     InterpolateKey::Variable(_) => None,
    //                     InterpolateKey::Component(key) => Some(key.name.to_string()),
    //                 })
    //                 .collect();

    //             let var_keys1 = keys1
    //                 .iter()
    //                 .filter_map(|key| match key {
    //                     InterpolateKey::Variable(key) => Some(key.name.to_string()),
    //                     InterpolateKey::Component(_) => None,
    //                 })
    //                 .collect();

    //             let var_keys2 = keys2
    //                 .iter()
    //                 .filter_map(|key| match key {
    //                     InterpolateKey::Variable(key) => Some(key.name.to_string()),
    //                     InterpolateKey::Component(_) => None,
    //                 })
    //                 .collect();

    //             Error::InterpolateVariableNotMatching(
    //                 InterpolateKeysNotMatching {
    //                     key: key.name.to_string(),
    //                     locale1: locale1.name.name.to_string(),
    //                     locale2: locale2.name.name.to_string(),
    //                     comp_keys1,
    //                     comp_keys2,
    //                     var_keys1,
    //                     var_keys2,
    //                 }
    //                 .into(),
    //             )
    //         }
    //         (LocaleValue::String, LocaleValue::String) => unreachable!(),
    //     }
    // }

    // pub fn check_locales<I>(locales: I) -> Result<HashMap<&'a Key<'a>, LocaleValue<'a, 'a>>>
    // where
    //     I: IntoIterator<Item = &'a Locale<'a>>,
    // {
    //     let mut iter = locales.into_iter();
    //     let first_locale = iter.next().unwrap();

    //     let first_locale_keys = first_locale.get_keys();

    //     for locale in iter {
    //         let keys = locale.get_keys();
    //         if first_locale_keys != keys {
    //             return Err(Self::key_diff(
    //                 first_locale,
    //                 &first_locale_keys,
    //                 locale,
    //                 &keys,
    //             ));
    //         }
    //     }

    //     Ok(first_locale_keys)
    // }

    pub fn get_keys(&self) -> HashSet<&Key> {
        self.keys.keys().collect()
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
                todo!("key mismatch beetween locales")
            }

            for (key, key_kind) in &mut mapped_keys {
                if let Some(value) = locale.keys.get(key) {
                    value.get_keys_inner(key_kind)
                }
            }
        }

        let iter = mapped_keys
            .iter_mut()
            .filter_map(|(key, value)| value.as_mut().map(|value| (key, value)));

        for (locale_key, keys) in iter {
            // if the set contains InterpolateKey::Count, remove variable keys with name "count"
            if keys.contains(&InterpolateKey::Count) {
                keys.retain(
                    |key| !matches!(key, InterpolateKey::Variable(key) if key.name == "count"),
                );

                for key in keys.iter() {
                    if matches!(key, InterpolateKey::Component(key) if key.name == "count") {
                        todo!("found component with name \"count\" but key is used with plurals, count is a reserved name.")
                        // error
                    }
                }
            }
            let var_keys = keys
                .iter()
                .filter_map(|key| match key {
                    InterpolateKey::Variable(key) => Some(key),
                    _ => None,
                })
                .collect::<HashSet<_>>();
            let comp_keys = keys
                .iter()
                .filter_map(|key| match key {
                    InterpolateKey::Component(key) => Some(key),
                    _ => None,
                })
                .collect::<HashSet<_>>();

            let common_key = var_keys.intersection(&comp_keys).next();

            if let Some(common_key) = common_key {
                todo!(
                    "found key {:?} used for both variables and components for locale key {:?}.",
                    common_key.name,
                    locale_key
                ) // error
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
