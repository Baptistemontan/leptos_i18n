// #![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

use datakey::DataK;
use icu::locid::LanguageIdentifier;
use icu_datagen::baked_exporter::BakedExporter;
use icu_datagen::prelude::DataKey;
use icu_datagen::{DatagenDriver, DatagenProvider};
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

pub struct TranslationsInfos {
    locales: BuildersKeys,
    paths: Vec<String>,
}

impl TranslationsInfos {
    pub fn parse() -> Result<Self> {
        // We don't really care for warnings, they will already be displayed by the macro
        let (locales, _, paths) = parse_locales::parse_locales()?;

        Ok(TranslationsInfos { locales, paths })
    }

    pub fn files_paths(&self) -> &[String] {
        &self.paths
    }

    pub fn rerun_if_locales_changed(&self) {
        for path in &self.paths {
            println!("cargo:rerun-if-changed={}", path);
        }
    }

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

    pub fn get_namespaces(&self) -> Option<impl Iterator<Item = Rc<str>> + '_> {
        match &self.locales {
            BuildersKeys::NameSpaces { namespaces, .. } => {
                let namespaces = namespaces.iter().map(|ns| ns.key.name.clone());
                Some(namespaces)
            }
            BuildersKeys::Locales { .. } => None,
        }
    }

    pub fn get_locales_langids(&self) -> impl Iterator<Item = LanguageIdentifier> + '_ {
        self.get_locales()
            .map(|locale| locale.parse::<LanguageIdentifier>().unwrap())
    }

    fn get_icu_keys_inner(&self, used_icu_keys: &mut HashSet<DataK>) {
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

    pub fn get_icu_keys(&self) -> HashSet<DataKey> {
        let mut used_icu_keys = HashSet::new();
        self.get_icu_keys_inner(&mut used_icu_keys);
        datakey::get_keys(used_icu_keys)
    }

    pub fn build_datagen_driver_with_data_keys(&self, keys: HashSet<DataKey>) -> DatagenDriver {
        let mut icu_keys = self.get_icu_keys();
        icu_keys.extend(keys);

        let locales = self.get_locales_langids();
        DatagenDriver::new()
            .with_keys(icu_keys)
            .with_locales_no_fallback(locales, Default::default())
    }

    pub fn build_datagen_driver_with_options(&self, keys: HashSet<DataK>) -> DatagenDriver {
        self.build_datagen_driver_with_data_keys(datakey::get_keys(keys))
    }

    pub fn build_datagen_driver(&self) -> DatagenDriver {
        self.build_datagen_driver_with_options(Default::default())
    }

    pub fn generate_data_with_data_keys(
        &self,
        mod_directory: PathBuf,
        keys: HashSet<DataKey>,
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

    pub fn generate_data_with_options(
        &self,
        mod_directory: PathBuf,
        keys: HashSet<DataK>,
    ) -> Result<(), DataError> {
        self.generate_data_with_data_keys(mod_directory, datakey::get_keys(keys))
    }

    pub fn generate_data(&self, mod_directory: PathBuf) -> Result<(), DataError> {
        self.generate_data_with_options(mod_directory, Default::default())
    }
}
