use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use crate::{
    error::{Diagnostics, Error, Result},
    parser::{
        ValuesSeed,
        files::{LocaleFiles, TranslationFiles},
        options::{Config, FileFormat, LocaleName, ParseOptions},
        raw_value::RawValue,
    },
    utils::{Key, KeyPath},
};

#[derive(Debug, Clone, PartialEq)]
pub enum RawLocalesOrNamespaces<V = RawValue> {
    Locales(Vec<RawLocale<V>>),
    Namespaces(Vec<RawNamespace<V>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawLocale<V = RawValue> {
    pub name: LocaleName,
    pub values: RawValues<V>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawNamespace<V = RawValue> {
    pub name: Key,
    pub locales: Vec<RawLocale<V>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawValues<V = RawValue> {
    pub values: BTreeMap<Key, RawValueOrSubkeys<V>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoSubkey {}

#[derive(Debug, Clone, PartialEq)]
pub enum RawValueOrSubkeys<V = RawValue, S = RawValues<V>> {
    Value(V),
    Subkeys(S),
    Defaulted,
}

impl RawLocalesOrNamespaces {
    fn new_locales(
        files: &LocaleFiles,
        namespace: Option<&Key>,
        diag: &Diagnostics,
        cfg: &Config,
    ) -> Result<Vec<RawLocale>> {
        let mut locales = Vec::with_capacity(cfg.locales.len());
        for key in cfg.locales.iter().cloned() {
            let Some((file_path, file)) = files.files.get(&*key.key.name) else {
                todo!()
            };
            let locale = RawLocale::new(
                file,
                file_path.as_deref(),
                key,
                namespace.cloned(),
                diag,
                &cfg.options,
            )?;
            locales.push(locale);
        }
        Ok(locales)
    }

    fn new_namespaces(
        files: &HashMap<&str, LocaleFiles>,
        diag: &Diagnostics,
        cfg: &Config,
    ) -> Result<Vec<RawNamespace>> {
        let mut namespaces = Vec::with_capacity(cfg.namespaces.len());
        for namespace in cfg.namespaces.iter().cloned() {
            let Some(locales_files) = files.get(&*namespace.name) else {
                todo!()
            };
            let ns = RawNamespace::new(namespace, locales_files, diag, cfg)?;
            namespaces.push(ns);
        }
        Ok(namespaces)
    }

    pub fn new(files: &TranslationFiles, diag: &Diagnostics, cfg: &Config) -> Result<Self> {
        if !cfg.namespaces.is_empty() {
            let TranslationFiles::NameSpaces { namespaces } = files else {
                todo!();
            };
            let namespaces = Self::new_namespaces(namespaces, diag, cfg)?;
            Ok(RawLocalesOrNamespaces::Namespaces(namespaces))
        } else {
            let TranslationFiles::Locales { locales } = files else {
                todo!();
            };
            let locales = Self::new_locales(locales, None, diag, cfg)?;
            Ok(RawLocalesOrNamespaces::Locales(locales))
        }
    }
}

impl RawNamespace {
    pub fn new(
        name: Key,
        locales_files: &LocaleFiles,
        diag: &Diagnostics,
        cfg: &Config,
    ) -> Result<Self> {
        let locales = RawLocalesOrNamespaces::new_locales(locales_files, Some(&name), diag, cfg)?;
        Ok(RawNamespace { name, locales })
    }
}

impl RawLocale {
    pub fn new(
        locale_file: &str,
        path: Option<&Path>,
        name: LocaleName,
        namespace: Option<Key>,
        diag: &Diagnostics,
        options: &ParseOptions,
    ) -> Result<Self> {
        let seed = ValuesSeed {
            name: name.key.clone(),
            top_locale_name: &name,
            key_path: KeyPath::new(namespace),
            diag,
            formatters: &options.formatters,
        };

        let values = Self::de(locale_file, path, seed, &options.file_format)?;

        Ok(RawLocale { name, values })
    }

    fn de(
        locale_file: &str,
        path: Option<&Path>,
        seed: ValuesSeed,
        file_format: &FileFormat,
    ) -> Result<RawValues> {
        let values = file_format
            .deserialize(locale_file, path, seed)
            .map_err(|err| Error::LocaleFileDeser {
                path: path.map(ToOwned::to_owned),
                err,
            })?;
        Ok(values)
    }
}
