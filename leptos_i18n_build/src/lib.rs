#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! This crate provide `build.rs` utilities for the `leptos_i18n` crate.

use std::collections::HashSet;
use std::fmt::{Display, Write};
use std::fs::{create_dir_all, File};
use std::io::BufWriter;
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
    fn parse_inner(dir_path: Option<PathBuf>) -> Result<Self> {
        // We don't really care for warnings, they will already be displayed by the macro
        let (locales, _, paths) = parse_locales::parse_locales(true, dir_path)?;

        Ok(TranslationsInfos { locales, paths })
    }

    /// Parse the translations and obtain informations about them.
    pub fn parse() -> Result<Self> {
        Self::parse_inner(None)
    }

    /// Parse the translations at the given directory and obtain informations about them.
    pub fn parse_at_dir<P: Into<PathBuf>>(dir_path: P) -> Result<Self> {
        Self::parse_inner(Some(dir_path.into()))
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
                    .take(1)
                    .map(|ns| &ns.locales)
                    .flat_map(|locales| map_locales(locales));
                EitherIter::Iter1(iter)
            }
            BuildersKeys::Locales { locales, .. } => EitherIter::Iter2(map_locales(locales)),
        }
    }

    /// Return the parsed and sliced translations
    pub fn get_translations(
        &self,
    ) -> TranslationsType<
        impl Iterator<Item = NamespaceTranslations<'_, impl Iterator<Item = LocaleTranslations<'_>>>>,
        impl Iterator<Item = LocaleTranslations<'_>>,
    > {
        fn map_locales(locales: &[Locale]) -> impl Iterator<Item = LocaleTranslations<'_>> + '_ {
            locales.iter().map(|locale| LocaleTranslations {
                name: &locale.name.name,
                strings: &locale.strings,
            })
        }
        match &self.locales {
            BuildersKeys::NameSpaces { namespaces, .. } => {
                let iter = namespaces.iter().map(|ns| {
                    let locales = map_locales(&ns.locales);
                    NamespaceTranslations {
                        name: &ns.key.name,
                        locales,
                    }
                });
                TranslationsType::Namespace(iter)
            }
            BuildersKeys::Locales { locales, .. } => TranslationsType::Locale(map_locales(locales)),
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

/// Describe if the translations have been declared in namespaces or as is.
pub enum TranslationsType<N, L> {
    /// Cases where the translations are declared in namespaces.
    Namespace(N),
    /// Cases where translations are declared as is.
    Locale(L),
}

/// Translations of a namespace
#[derive(Debug, Clone, Copy)]
pub struct NamespaceTranslations<'a, L> {
    name: &'a str,
    locales: L,
}

/// Translations of a locale
#[derive(Debug, Clone, Copy)]
pub struct LocaleTranslations<'a> {
    name: &'a str,
    strings: &'a [String],
}

/// Formatter for the translations parsed strings
#[derive(Debug, Clone, Copy)]
pub struct TranslationsFormatter<'a> {
    #[allow(unused)]
    strings: &'a [String],
}

impl<N, L> TranslationsType<N, L> {
    /// Return `Some` if the translations have been declared in namespaces.
    pub fn into_namespaces(self) -> Option<N> {
        match self {
            TranslationsType::Namespace(namespaces) => Some(namespaces),
            TranslationsType::Locale(_) => None,
        }
    }

    /// Return `Some` if the translations have been declared without namespacing.
    pub fn into_locales(self) -> Option<L> {
        match self {
            TranslationsType::Namespace(_) => None,
            TranslationsType::Locale(locales) => Some(locales),
        }
    }
}

fn write_locales_to_dir<'a>(
    locales: impl Iterator<Item = LocaleTranslations<'a>>,
    path: &mut PathBuf,
) -> std::io::Result<()> {
    create_dir_all(&*path)?;
    for locale in locales {
        locale.write_to_dir(path)?;
    }
    Ok(())
}

impl<
        'a,
        N: Iterator<Item = NamespaceTranslations<'a, NL>>,
        NL: Iterator<Item = LocaleTranslations<'a>>,
        L: Iterator<Item = LocaleTranslations<'a>>,
    > TranslationsType<N, L>
{
    /// Write the translations in the given directory
    pub fn write_to_dir<P: Into<PathBuf>>(self, path: P) -> std::io::Result<()> {
        let mut path: PathBuf = path.into();
        match self {
            TranslationsType::Namespace(namespaces) => {
                for namespace in namespaces {
                    namespace.write_to_dir(&mut path)?;
                }
                Ok(())
            }
            TranslationsType::Locale(locales) => write_locales_to_dir(locales, &mut path),
        }
    }
}

impl<'a, L> NamespaceTranslations<'a, L> {
    /// Return the name of the namespace.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Return the locales translations for that namespace.
    pub fn into_locales(self) -> L {
        self.locales
    }
}

impl<'a, L: Iterator<Item = LocaleTranslations<'a>>> NamespaceTranslations<'a, L> {
    fn write_to_dir(self, path: &mut PathBuf) -> std::io::Result<()> {
        path.push(self.name);
        write_locales_to_dir(self.locales, path)?;
        path.pop();
        Ok(())
    }
}

impl<'a> LocaleTranslations<'a> {
    /// Return the name of that locale.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Return the formatter for the parsed string of the that locale.
    pub fn translations_formatter(&self) -> TranslationsFormatter<'a> {
        TranslationsFormatter {
            strings: self.strings,
        }
    }

    fn write_to_dir(self, path: &mut PathBuf) -> std::io::Result<()> {
        use std::io::Write;
        path.push(self.name);
        path.set_extension("json");
        let mut file = File::create(&*path)?;
        path.pop();
        let mut f = BufWriter::new(&mut file);
        write!(f, "{}", self.translations_formatter())
    }
}

impl Display for TranslationsFormatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut iter = self.strings.iter();
        if let Some(first) = iter.next() {
            write!(f, "{:?}", first)?;
        }
        for s in iter {
            write!(f, ",{:?}", s)?;
        }
        f.write_char(']')
    }
}
