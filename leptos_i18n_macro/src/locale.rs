use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use crate::{
    error::{Error, Result},
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

    pub fn check_locales<I>(locales: I) -> Result<HashMap<&'a Key<'a>, LocaleValue<'a, 'a>>>
    where
        I: IntoIterator<Item = &'a Locale<'a>>,
    {
        let mut iter = locales.into_iter();
        let first_locale = iter.next().unwrap();

        let first_locale_keys = first_locale.get_keys();

        for locale in iter {
            if first_locale_keys != locale.get_keys() {
                todo!();
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
