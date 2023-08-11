use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use crate::{
    error::{Error, InterpolateKeysNotMatching, Result},
    key::Key,
    value_kind::{InterpolateKeyKind, ValueKind},
};

pub struct RawLocale<'a> {
    pub name: &'a str,
    pub keys: HashMap<String, String>,
}

pub struct Locale<'a> {
    pub name: Key<'a>,
    pub keys: HashMap<Key<'a>, ValueKind<'a>>,
}

// impl<'a> PartialEq for Locale<'a> {
//     fn eq(&self, other: &Self) -> bool {
//         self.keys.g == other.keys
//     }
// }

// impl<'a> Eq for Locale<'a> {}

impl<'a> Locale<'a> {
    pub fn new(locale: &'a RawLocale) -> Result<Self> {
        let RawLocale {
            name: locale_name,
            keys,
        } = locale;
        let name = Key::new(locale.name, crate::key::KeyKind::LocaleName)?;
        let keys = keys
            .iter()
            .map(|(key, value)| -> Result<(Key, ValueKind)> {
                let key = Key::new(key, crate::key::KeyKind::LocaleKey { locale_name })?;
                let value = ValueKind::new(value);
                Ok((key, value))
            })
            .collect::<Result<_>>()?;

        Ok(Locale { name, keys })
    }

    pub fn get_keys<'b>(&'b self) -> HashMap<&'b Key<'a>, LocaleValue<'a, 'b>> {
        self.keys
            .iter()
            .map(|(key, value)| (key, value.get_keys().into()))
            .collect()
    }

    fn key_diff(
        locale1: &Locale,
        keys1: &HashMap<&Key, LocaleValue>,
        locale2: &Locale,
        keys2: &HashMap<&Key, LocaleValue>,
    ) -> Error {
        // check key mismatch

        let keys_not_in_2 = keys1
            .iter()
            .map(|(key, _)| key)
            .filter(|key| keys2.get(*key).is_none())
            .map(|key| key.name.to_string())
            .collect::<Vec<_>>();
        if !keys_not_in_2.is_empty() {
            return Error::MissingKeysInLocale {
                keys: keys_not_in_2,
                locale: locale2.name.name.to_string(),
            };
        }

        let keys_not_in_1 = keys2
            .iter()
            .map(|(key, _)| key)
            .filter(|key| keys1.get(*key).is_none())
            .map(|key| key.name.to_string())
            .collect::<Vec<_>>();
        if !keys_not_in_1.is_empty() {
            return Error::MissingKeysInLocale {
                keys: keys_not_in_1,
                locale: locale1.name.name.to_string(),
            };
        }

        // check key kind mismatch
        let (key, value, other_value) = keys1
            .iter()
            .filter_map(|(key, value)| {
                let other_value = keys2.get(*key)?;
                (value != other_value).then_some((key, value, other_value))
            })
            .next()
            .expect("error was reported but everyhting seems fine...");

        match (value, other_value) {
            (LocaleValue::String, LocaleValue::Interpolate(_)) => Error::MismatchLocaleKeyKind {
                key: key.name.to_string(),
                locale_str: locale1.name.name.to_string(),
                locale_inter: locale2.name.name.to_string(),
            },
            (LocaleValue::Interpolate(_), LocaleValue::String) => Error::MismatchLocaleKeyKind {
                key: key.name.to_string(),
                locale_str: locale2.name.name.to_string(),
                locale_inter: locale1.name.name.to_string(),
            },
            (LocaleValue::Interpolate(keys1), LocaleValue::Interpolate(keys2)) => {
                let comp_keys1 = keys1
                    .iter()
                    .filter_map(|key| match key {
                        InterpolateKeyKind::Variable(_) => None,
                        InterpolateKeyKind::Component(key) => Some(key.name.to_string()),
                    })
                    .collect();

                let comp_keys2 = keys2
                    .iter()
                    .filter_map(|key| match key {
                        InterpolateKeyKind::Variable(_) => None,
                        InterpolateKeyKind::Component(key) => Some(key.name.to_string()),
                    })
                    .collect();

                let var_keys1 = keys1
                    .iter()
                    .filter_map(|key| match key {
                        InterpolateKeyKind::Variable(key) => Some(key.name.to_string()),
                        InterpolateKeyKind::Component(_) => None,
                    })
                    .collect();

                let var_keys2 = keys2
                    .iter()
                    .filter_map(|key| match key {
                        InterpolateKeyKind::Variable(key) => Some(key.name.to_string()),
                        InterpolateKeyKind::Component(_) => None,
                    })
                    .collect();

                Error::InterpolateVariableNotMatching(
                    InterpolateKeysNotMatching {
                        key: key.name.to_string(),
                        locale1: locale1.name.name.to_string(),
                        locale2: locale2.name.name.to_string(),
                        comp_keys1,
                        comp_keys2,
                        var_keys1,
                        var_keys2,
                    }
                    .into(),
                )
            }
            (LocaleValue::String, LocaleValue::String) => unreachable!(),
        }
    }

    pub fn check_locales<I>(locales: I) -> Result<HashMap<&'a Key<'a>, LocaleValue<'a, 'a>>>
    where
        I: IntoIterator<Item = &'a Locale<'a>>,
    {
        let mut iter = locales.into_iter();
        let first_locale = iter.next().unwrap();

        let first_locale_keys = first_locale.get_keys();

        for locale in iter {
            let keys = locale.get_keys();
            if first_locale_keys != keys {
                return Err(Self::key_diff(
                    first_locale,
                    &first_locale_keys,
                    locale,
                    &keys,
                ));
            }
        }

        Ok(first_locale_keys)
    }
}

impl<'a> RawLocale<'a> {
    pub fn new<T: AsRef<Path>>(path: T, locale: &'a str) -> Result<Self> {
        let locale_file =
            File::open(path).map_err(|err| Error::LocaleFileNotFound(locale.to_string(), err))?;

        let keys = serde_json::from_reader(locale_file)
            .map_err(|err| Error::LocaleFileDeser(locale.to_string(), err))?;

        let name = locale.trim();
        Ok(RawLocale { name, keys })
    }
}

#[derive(PartialEq, Eq)]
pub enum LocaleValue<'a, 'b> {
    String,
    Interpolate(HashSet<InterpolateKeyKind<'a, 'b>>),
}

impl<'a, 'b> From<Option<HashSet<InterpolateKeyKind<'a, 'b>>>> for LocaleValue<'a, 'b> {
    fn from(value: Option<HashSet<InterpolateKeyKind<'a, 'b>>>) -> Self {
        match value {
            Some(keys) => Self::Interpolate(keys),
            None => Self::String,
        }
    }
}
