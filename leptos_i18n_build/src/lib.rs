#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! This crate provide `build.rs` utilities for the `leptos_i18n` crate.

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

pub use datakey::Options;
use icu_datagen::baked_exporter::BakedExporter;
use icu_datagen::prelude::DataKey;
use icu_datagen::{DatagenDriver, DatagenProvider};
use icu_locid::LanguageIdentifier;
use icu_provider::DataError;
use leptos_i18n_parser::parse_locales;
use leptos_i18n_parser::parse_locales::error::Result;
use leptos_i18n_parser::parse_locales::locale::{BuildersKeys, Locale};

mod datakey;

#[derive(Clone)]
enum EitherIter<A, B> {
    Iter1(A),
    Iter2(B),
}

impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for EitherIter<A, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Iter1(iter) => iter.next(),
            EitherIter::Iter2(iter) => iter.next(),
        }
    }
}

/// Contains informations about the translations.
pub struct TranslationsInfos {
    locales: BuildersKeys,
    paths: Vec<String>,
}

impl TranslationsInfos {
    /// Parse the translations and obtain informations about them.
    pub fn parse() -> Result<Self> {
        // We don't really care for warnings, they will already be displayed by the macro
        let (locales, _, paths) = parse_locales::parse_locales(true)?;

        Ok(TranslationsInfos { locales, paths })
    }

    /// Paths to all files containing translations.
    pub fn files_paths(&self) -> &[String] {
        &self.paths
    }

    /// Output "cargo:rerun-if-changed" for all locales files.
    pub fn rerun_if_locales_changed(&self) {
        for path in &self.paths {
            println!("cargo:rerun-if-changed={}", path);
        }
    }

    /// Return an iterator containing the name of each locales.
    pub fn get_locales(&self) -> impl Iterator<Item = Rc<str>> + '_ {
        fn map_locales(locales: &[Locale]) -> impl Iterator<Item = Rc<str>> + '_ {
            locales.iter().map(|locale| locale.name.name.clone())
        }
        match &self.locales {
            BuildersKeys::NameSpaces { namespaces, .. } => {
                let iter = namespaces
                    .iter()
                    .map(|ns| &ns.locales)
                    .flat_map(|locales| map_locales(locales));
                EitherIter::Iter1(iter)
            }
            BuildersKeys::Locales { locales, .. } => EitherIter::Iter2(map_locales(locales)),
        }
    }

    /// Return an iterator containing the name of each namespaces, if any.
    pub fn get_namespaces(&self) -> Option<impl Iterator<Item = Rc<str>> + '_> {
        match &self.locales {
            BuildersKeys::NameSpaces { namespaces, .. } => {
                let namespaces = namespaces.iter().map(|ns| ns.key.name.clone());
                Some(namespaces)
            }
            BuildersKeys::Locales { .. } => None,
        }
    }

    /// Return an iterator containing each locales in the form of `LanguageIdentifier`.
    pub fn get_locales_langids(&self) -> impl Iterator<Item = LanguageIdentifier> + '_ {
        self.get_locales()
            .map(|locale| locale.parse::<LanguageIdentifier>().unwrap())
    }

    fn get_icu_keys_inner(&self, used_icu_keys: &mut HashSet<Options>) {
        match &self.locales {
            BuildersKeys::NameSpaces { keys, .. } => {
                for builder_keys in keys.values() {
                    datakey::find_used_datakey(builder_keys, used_icu_keys);
                }
            }
            BuildersKeys::Locales { keys, .. } => {
                datakey::find_used_datakey(keys, used_icu_keys);
            }
        }
    }

    /// Return the ICU `DataKey` needed by the translations.
    pub fn get_icu_keys(&self) -> impl Iterator<Item = DataKey> {
        let mut used_icu_keys = HashSet::new();
        self.get_icu_keys_inner(&mut used_icu_keys);
        datakey::get_keys(used_icu_keys)
    }

    /// Same as `build_datagen_driver` but can be supplied with additional ICU `DataKey`.
    pub fn build_datagen_driver_with_data_keys(
        &self,
        keys: impl IntoIterator<Item = DataKey>,
    ) -> DatagenDriver {
        let mut icu_keys: HashSet<DataKey> = self.get_icu_keys().collect();
        icu_keys.extend(keys);

        let locales = self.get_locales_langids();
        DatagenDriver::new()
            .with_keys(icu_keys)
            .with_locales_no_fallback(locales, Default::default())
    }

    /// Same as `build_datagen_driver` but can be supplied with additional options.
    /// This usefull if you use `t*_format!` and use formatters not used in the translations.
    pub fn build_datagen_driver_with_options(
        &self,
        keys: impl IntoIterator<Item = Options>,
    ) -> DatagenDriver {
        self.build_datagen_driver_with_data_keys(datakey::get_keys(keys))
    }

    /// Build a `DatagenDriver` using the locales and keys needed for the translations.
    pub fn build_datagen_driver(&self) -> DatagenDriver {
        self.build_datagen_driver_with_options(std::iter::empty())
    }

    /// Same as `generate_data` but can be supplied additionnal ICU `DataKey`.
    pub fn generate_data_with_data_keys(
        &self,
        mod_directory: PathBuf,
        keys: impl IntoIterator<Item = DataKey>,
    ) -> Result<(), DataError> {
        // This is'nt really needed, but ICU4X wants the directory to be empty
        // and Rust Analyzer can trigger the build.rs without cleaning the out directory.
        if mod_directory.exists() {
            std::fs::remove_dir_all(&mod_directory).unwrap();
        }

        let exporter = BakedExporter::new(mod_directory, Default::default()).unwrap();

        self.build_datagen_driver_with_data_keys(keys)
            .export(&DatagenProvider::new_latest_tested(), exporter)
    }

    /// Same as `generate_data` but can be supplied additionnal options.
    pub fn generate_data_with_options(
        &self,
        mod_directory: PathBuf,
        keys: impl IntoIterator<Item = Options>,
    ) -> Result<(), DataError> {
        self.generate_data_with_data_keys(mod_directory, datakey::get_keys(keys))
    }

    /// Generate an ICU datagen at the given mod_directory using the infos from the translations.
    pub fn generate_data(&self, mod_directory: PathBuf) -> Result<(), DataError> {
        self.generate_data_with_options(mod_directory, std::iter::empty())
    }
}
