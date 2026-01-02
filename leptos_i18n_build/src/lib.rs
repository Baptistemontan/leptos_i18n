#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! This crate provide `build.rs` utilities for the `leptos_i18n` crate.

pub use datamarker::FormatterOptions;
pub use leptos_i18n_parser::parse_locales::options::{FileFormat, ParseOptions, parser};

use icu_locale::LocaleFallbacker;
use icu_provider::{DataError, DataMarkerInfo};
use icu_provider_export::{
    DataLocaleFamily, DeduplicationStrategy, ExportDriver, ExportMetadata,
    baked_exporter::{self, BakedExporter},
};
use icu_provider_source::SourceDataProvider;
use leptos_i18n_parser::parse_locales::{
    ParsedLocales,
    error::Result,
    locale::{BuildersKeys, Locale},
    parse_locales,
};
use std::{
    collections::HashSet,
    fmt::{Display, Write},
    fs::{File, create_dir_all},
    io::BufWriter,
    path::PathBuf,
    rc::Rc,
};

mod datamarker;
pub mod options;

use crate::options::CodegenOptions;

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
    parsed_locales: ParsedLocales,
}

impl TranslationsInfos {
    fn parse_inner(dir_path: Option<PathBuf>, options: ParseOptions) -> Result<Self> {
        // We don't really care for warnings, they will already be displayed by the macro
        let parsed_locales = parse_locales(dir_path, options)?;

        Ok(TranslationsInfos { parsed_locales })
    }

    /// Parse the translations and obtain informations about them.
    pub fn parse(options: ParseOptions) -> Result<Self> {
        let this = Self::parse_inner(None, options)?;
        Ok(this)
    }

    /// Parse the translations at the given directory and obtain informations about them.
    pub fn parse_at_dir<P: Into<PathBuf>>(dir_path: P, options: ParseOptions) -> Result<Self> {
        Self::parse_inner(Some(dir_path.into()), options)
    }

    /// Paths to all files containing translations.
    pub fn files_paths(&self) -> Option<&[String]> {
        self.parsed_locales.tracked_files.as_deref()
    }

    /// Output "cargo::rerun-if-changed" for all locales files.
    pub fn rerun_if_locales_changed(&self) {
        if let Some(paths) = self.files_paths() {
            for path in paths {
                println!("cargo::rerun-if-changed={path}");
            }
        }
    }

    /// Return an iterator containing the name of each locales.
    pub fn get_locales(&self) -> impl Iterator<Item = Rc<str>> + '_ {
        fn map_locales(locales: &[Locale]) -> impl Iterator<Item = Rc<str>> + '_ {
            locales.iter().map(|locale| locale.name.name.clone())
        }
        match &self.parsed_locales.builder_keys {
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
        match &self.parsed_locales.builder_keys {
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
        match &self.parsed_locales.builder_keys {
            BuildersKeys::NameSpaces { namespaces, .. } => {
                let namespaces = namespaces.iter().map(|ns| ns.key.name.clone());
                Some(namespaces)
            }
            BuildersKeys::Locales { .. } => None,
        }
    }

    /// Return an iterator containing each locales in the form of `LanguageIdentifier`.
    pub fn get_locales_langids(&self) -> impl Iterator<Item = DataLocaleFamily> + '_ {
        self.get_locales()
            .map(|locale| locale.parse::<DataLocaleFamily>().unwrap())
    }

    fn get_icu_keys_inner(&self, used_icu_keys: &mut HashSet<FormatterOptions>) {
        match &self.parsed_locales.builder_keys {
            BuildersKeys::NameSpaces { keys, .. } => {
                for builder_keys in keys.values() {
                    datamarker::find_used_datamarker(builder_keys, used_icu_keys);
                }
            }
            BuildersKeys::Locales { keys, .. } => {
                datamarker::find_used_datamarker(keys, used_icu_keys);
            }
        }
    }

    /// Return the ICU `DataMarker` needed by the translations.
    pub fn get_icu_keys(&self) -> impl Iterator<Item = DataMarkerInfo> {
        let mut used_icu_keys = HashSet::new();
        self.get_icu_keys_inner(&mut used_icu_keys);
        datamarker::get_markers(used_icu_keys)
    }

    /// Same as `build_datagen_driver` but can be supplied with additional ICU `DataMarker`.
    pub fn build_datagen_driver_with_data_keys(
        &self,
        keys: impl IntoIterator<Item = DataMarkerInfo>,
    ) -> ExportDriver {
        let mut icu_keys: HashSet<DataMarkerInfo> = self.get_icu_keys().collect();
        icu_keys.extend(keys);

        let locales = self.get_locales_langids();

        ExportDriver::new(
            locales,
            DeduplicationStrategy::None.into(),
            LocaleFallbacker::new_without_data(),
        )
        .with_markers(icu_keys)
    }

    /// Same as `build_datagen_driver` but can be supplied with additional options.
    /// This usefull if you use `t*_format!` and use formatters not used in the translations.
    pub fn build_datagen_driver_with_options(
        &self,
        keys: impl IntoIterator<Item = FormatterOptions>,
    ) -> ExportDriver {
        self.build_datagen_driver_with_data_keys(datamarker::get_markers(keys))
    }

    /// Build a `ExportDriver` using the locales and keys needed for the translations.
    pub fn build_datagen_driver(&self) -> ExportDriver {
        self.build_datagen_driver_with_options(std::iter::empty())
    }

    /// Same as `generate_data` but can be supplied additionnal ICU `DataMarker`.
    pub fn generate_data_with_data_markers(
        &self,
        mod_directory: PathBuf,
        keys: impl IntoIterator<Item = DataMarkerInfo>,
    ) -> Result<ExportMetadata, DataError> {
        // This is'nt really needed, but ICU4X wants the directory to be empty
        // and Rust Analyzer can trigger the build.rs without cleaning the out directory.
        if mod_directory.exists() {
            std::fs::remove_dir_all(&mod_directory).unwrap();
        }

        let exporter = BakedExporter::new(mod_directory, {
            let mut options = baked_exporter::Options::default();
            options.overwrite = true;
            options.use_internal_fallback = false;
            options
        })
        .unwrap();

        self.build_datagen_driver_with_data_keys(keys)
            .export(&SourceDataProvider::new(), exporter)
    }

    /// Same as `generate_data` but can be supplied additionnal options.
    pub fn generate_data_with_options(
        &self,
        mod_directory: PathBuf,
        keys: impl IntoIterator<Item = FormatterOptions>,
    ) -> Result<ExportMetadata, DataError> {
        self.generate_data_with_data_markers(mod_directory, datamarker::get_markers(keys))
    }

    /// Generate an ICU datagen at the given mod_directory using the infos from the translations.
    pub fn generate_data(&self, mod_directory: PathBuf) -> Result<ExportMetadata, DataError> {
        self.generate_data_with_options(mod_directory, std::iter::empty())
    }

    /// Generate the `i18n` module at the given mod directory
    pub fn generate_i18n_module(&self, mod_directory: PathBuf) -> Result<()> {
        self.generate_i18n_module_with_options(mod_directory, CodegenOptions::default())
    }

    /// Generate the `i18n` module at the given mod directory with options
    pub fn generate_i18n_module_with_options(
        &self,
        mut mod_directory: PathBuf,
        options: CodegenOptions,
    ) -> Result<()> {
        let ts = leptos_i18n_codegen::gen_code(
            &self.parsed_locales,
            options.crate_path.as_ref(),
            false,
            options.top_level_attributes.as_ref(),
        )?;

        #[cfg(feature = "pretty_print")]
        let ts = {
            let as_file = syn::parse_quote!(#ts);
            prettyplease::unparse(&as_file)
        };

        create_dir_all(&mod_directory)?;

        mod_directory.push(options.module_file_name);

        let mut file = File::create(&mod_directory)?;

        use std::io::Write;
        write!(&mut file, "{ts}")?;

        Ok(())
    }

    /// Emit the warnings generated when parsing the translations
    pub fn emit_warnings(&self) {
        let warnings = self.parsed_locales.diag.warnings();

        for warning in warnings.iter() {
            println!("cargo:warning={warning}");
        }
    }

    /// emit the errors generated when parsing the translations
    pub fn emit_errors(&self) {
        let errors = self.parsed_locales.diag.errors();

        for error in errors.iter() {
            println!("cargo:error={error}");
        }
    }

    /// Emit the diagnostics generated when parsing the translations
    pub fn emit_diagnostics(&self) {
        self.emit_warnings();
        self.emit_errors();
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
    strings: &'a [Rc<str>],
}

/// Formatter for the translations parsed strings
#[derive(Debug, Clone, Copy)]
pub struct TranslationsFormatter<'a> {
    #[allow(unused)]
    strings: &'a [Rc<str>],
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
            write!(f, "{first:?}")?;
        }
        for s in iter {
            write!(f, ",{s:?}")?;
        }
        f.write_char(']')
    }
}
