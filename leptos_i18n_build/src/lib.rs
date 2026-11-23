#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! This crate provide `build.rs` utilities for the `leptos_i18n` crate.

pub use datamarker::FormatterOptions;

use icu_locale::LocaleFallbacker;
use icu_provider::{DataError, DataMarkerInfo};
use icu_provider_export::{
    DataLocaleFamily, DeduplicationStrategy, ExportDriver, ExportMetadata,
    baked_exporter::{self, BakedExporter},
};
use icu_provider_source::SourceDataProvider;
use leptos_i18n_parser::extraction::{
    Locales, LocalesOrNamespaces, ParsedLocales, extract_locales,
};
use leptos_i18n_parser::parsing::parse_locales_raw;
use leptos_i18n_parser::{error::Result, parsing::RawParsedLocales};
use std::collections::HashMap;
use std::{
    collections::HashSet,
    fmt::{Display, Write},
    fs::{File, create_dir_all},
    io::BufWriter,
    path::PathBuf,
    rc::Rc,
};

mod datamarker;

use leptos_i18n_codegen::CodegenOptions;
pub use leptos_i18n_parser::options::Config;
pub use leptos_i18n_parser::parsing::{LocaleFiles, TranslationFiles};

/// Parsing and codegen options
pub mod options {
    pub use leptos_i18n_codegen::CodegenOptions;
    pub use leptos_i18n_parser::options::{FileFormat, ParseOptions, parser};
}

/// Module for custom formatters.
pub mod formatter {
    pub use leptos_i18n_parser::formatters::{Formatter, FormatterToTokens};
    pub use leptos_i18n_parser::utils::Key;
}

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
    tracked_files: Vec<String>,
    cfg: Config,
}

impl TranslationsInfos {
    fn parse_inner(dir_path: Option<PathBuf>, cfg: Config) -> Result<Self> {
        let RawParsedLocales {
            values,
            diag,
            tracked_files,
        } = parse_locales_raw(dir_path, &cfg)?;

        let parsed_locales = extract_locales(values, &cfg, Some(diag))?;

        Ok(TranslationsInfos {
            parsed_locales,
            tracked_files,
            cfg,
        })
    }

    /// Parse the translations and obtain informations about them.
    pub fn parse(cfg: Config) -> Result<Self> {
        let this = Self::parse_inner(None, cfg)?;
        Ok(this)
    }

    /// Parse the translations at the given directory and obtain informations about them.
    pub fn parse_at_dir<P: Into<PathBuf>>(dir_path: P, cfg: Config) -> Result<Self> {
        Self::parse_inner(Some(dir_path.into()), cfg)
    }

    /// Parse the translations directly as strings instead of loading files
    pub fn parse_with_files_as_string(cfg: Config, files: &TranslationFiles) -> Result<Self> {
        let RawParsedLocales {
            values,
            diag,
            tracked_files,
        } = leptos_i18n_parser::parsing::parse_locales_raw_with_files(files, &cfg)?;

        let parsed_locales = extract_locales(values, &cfg, Some(diag))?;

        Ok(TranslationsInfos {
            parsed_locales,
            tracked_files,
            cfg,
        })
    }

    /// Paths to all files containing translations.
    pub fn files_paths(&self) -> &[String] {
        &self.tracked_files
    }

    /// Output "cargo::rerun-if-changed" for all locales files.
    pub fn rerun_if_locales_changed(&self) {
        for path in &self.tracked_files {
            println!("cargo::rerun-if-changed={path}");
        }
    }

    /// Return an iterator containing the name of each locales.
    pub fn get_locales(&self) -> impl Iterator<Item = Rc<str>> + '_ {
        fn map_locales(locales: &Locales) -> impl Iterator<Item = Rc<str>> + '_ {
            locales
                .locales
                .iter()
                .map(|locale| locale.name.key.name.clone())
        }
        match &self.parsed_locales.values {
            LocalesOrNamespaces::Namespaces(namespaces) => {
                let iter = namespaces
                    .iter()
                    .take(1)
                    .flat_map(|ns| map_locales(&ns.locales));
                EitherIter::Iter1(iter)
            }
            LocalesOrNamespaces::Locales(locales) => EitherIter::Iter2(map_locales(locales)),
        }
    }

    /// Return the parsed and sliced translations
    pub fn get_translations(
        &self,
    ) -> TranslationsType<
        impl Iterator<Item = NamespaceTranslations<'_, impl Iterator<Item = LocaleTranslations<'_>>>>,
        impl Iterator<Item = LocaleTranslations<'_>>,
    > {
        fn map_locales(locales: &Locales) -> impl Iterator<Item = LocaleTranslations<'_>> + '_ {
            locales.locales.iter().map(|locale| LocaleTranslations {
                name: &locale.name.key.name,
                strings: &locale.strings,
            })
        }
        match &self.parsed_locales.values {
            LocalesOrNamespaces::Namespaces(namespaces) => {
                let iter = namespaces.iter().map(|ns| {
                    let locales = map_locales(&ns.locales);
                    NamespaceTranslations {
                        name: &ns.name.name,
                        locales,
                    }
                });
                TranslationsType::Namespace(iter)
            }
            LocalesOrNamespaces::Locales(locales) => TranslationsType::Locale(map_locales(locales)),
        }
    }

    /// Return an iterator containing the name of each namespaces, if any.
    pub fn get_namespaces(&self) -> Option<impl Iterator<Item = Rc<str>> + '_> {
        match &self.parsed_locales.values {
            LocalesOrNamespaces::Namespaces(namespaces) => {
                let namespaces = namespaces.iter().map(|ns| ns.name.name.clone());
                Some(namespaces)
            }
            LocalesOrNamespaces::Locales(_) => None,
        }
    }

    /// Return an iterator containing each locales in the form of `LanguageIdentifier`.
    pub fn get_locales_langids(&self) -> impl Iterator<Item = DataLocaleFamily> + '_ {
        self.get_locales()
            .map(|locale| locale.parse::<DataLocaleFamily>().unwrap())
    }

    fn get_icu_keys_inner(&self, used_icu_keys: &mut HashSet<FormatterOptions>) {
        for builder in self.parsed_locales.builders.builders.values() {
            datamarker::find_used_datamarker(&builder.keys, used_icu_keys);
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
    ///
    /// # Warning
    ///
    /// `mod_directory` is recursively removed before the data is generated,
    /// see [`generate_data`](Self::generate_data).
    pub fn generate_data_with_data_markers(
        &self,
        mod_directory: PathBuf,
        keys: impl IntoIterator<Item = DataMarkerInfo>,
    ) -> Result<ExportMetadata, DataError> {
        // `overwrite` makes `BakedExporter::new` recursively remove `mod_directory`,
        // which ICU4X wants empty and Rust Analyzer can leave behind by triggering
        // the build.rs without cleaning the out directory.
        let exporter = BakedExporter::new(mod_directory, {
            let mut options = baked_exporter::Options::default();
            options.overwrite = true;
            options.use_internal_fallback = false;
            options
        })?;

        self.build_datagen_driver_with_data_keys(keys)
            .export(&SourceDataProvider::new(), exporter)
    }

    /// Same as `generate_data` but can be supplied additionnal options.
    ///
    /// # Warning
    ///
    /// `mod_directory` is recursively removed before the data is generated,
    /// see [`generate_data`](Self::generate_data).
    pub fn generate_data_with_options(
        &self,
        mod_directory: PathBuf,
        keys: impl IntoIterator<Item = FormatterOptions>,
    ) -> Result<ExportMetadata, DataError> {
        self.generate_data_with_data_markers(mod_directory, datamarker::get_markers(keys))
    }

    /// Generate an ICU datagen at the given mod_directory using the infos from the translations.
    ///
    /// # Warning
    ///
    /// If `mod_directory` already exists it is **recursively removed** before the
    /// data is generated, so it must be a directory dedicated to that data,
    /// typically one under `OUT_DIR`. Anything else stored there is lost.
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
        let module_file_name = options.module_file_name;

        let ts = leptos_i18n_codegen::gen_code(&self.parsed_locales, options, &self.cfg)?;

        #[cfg(feature = "pretty_print")]
        let ts = {
            let as_file = syn::parse_quote!(#ts);
            prettyplease::unparse(&as_file)
        };

        create_dir_all(&mod_directory)?;

        mod_directory.push(module_file_name);

        let mut file = File::create(&mod_directory)?;

        use std::io::Write;
        write!(&mut file, "{ts}")?;

        Ok(())
    }

    /// Emit the warnings generated when parsing the translations
    pub fn emit_warnings(&self) {
        let warnings = self.parsed_locales.diag.warnings();

        for warning in warnings.iter() {
            println!("cargo::warning={warning}");
        }
    }

    /// emit the errors generated when parsing the translations
    pub fn emit_errors(&self) {
        let errors = self.parsed_locales.diag.errors();

        for error in errors.iter() {
            println!("cargo::error={error}");
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

type JSONTranslationsAsString<'a> =
    TranslationsType<HashMap<&'a str, HashMap<&'a str, String>>, HashMap<&'a str, String>>;

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

    /// Write the translations as JSON in maps
    pub fn to_strings(self) -> JSONTranslationsAsString<'a> {
        fn insert_map<'a, N: Iterator<Item = LocaleTranslations<'a>>>(
            locales: N,
            map: &mut HashMap<&'a str, String>,
        ) {
            for locale in locales {
                let l = locale.name;
                let json = locale.translations_formatter().to_string();
                map.insert(l, json);
            }
        }
        fn locale_map<'a, N: Iterator<Item = LocaleTranslations<'a>>>(
            locales: N,
        ) -> HashMap<&'a str, String> {
            let mut l_map = HashMap::new();
            insert_map(locales, &mut l_map);
            l_map
        }
        match self {
            TranslationsType::Namespace(namespaces) => {
                let mut map = HashMap::new();
                for namespace in namespaces {
                    let n = namespace.name;
                    let l_map = locale_map(namespace.locales);
                    map.insert(n, l_map);
                }
                TranslationsType::Namespace(map)
            }
            TranslationsType::Locale(locales) => {
                let map = locale_map(locales);
                TranslationsType::Locale(map)
            }
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

/// Write `s` as a JSON string, surrounding quotes included.
///
/// `str`'s `Debug` impl looks close enough but is not a JSON serializer:
/// it escapes control characters as `\u{7f}` and nul as `\0`, neither of which
/// JSON accepts, and it escapes a leading combining mark the same way.
fn write_json_str(f: &mut std::fmt::Formatter<'_>, s: &str) -> std::fmt::Result {
    f.write_char('"')?;
    // Index of the first byte not yet written, everything that needs no escaping
    // is copied in one go when an escape is met or at the end of the string.
    let mut copied = 0;
    for (index, c) in s.char_indices() {
        let escape = match c {
            '"' => "\\\"",
            '\\' => "\\\\",
            '\u{8}' => "\\b",
            '\u{c}' => "\\f",
            '\n' => "\\n",
            '\r' => "\\r",
            '\t' => "\\t",
            // Anything above U+001F is allowed as is, JSON only requires the
            // control characters to be escaped.
            _ if c >= '\u{20}' => continue,
            _ => {
                f.write_str(&s[copied..index])?;
                write!(f, "\\u{:04x}", c as u32)?;
                copied = index + c.len_utf8();
                continue;
            }
        };
        f.write_str(&s[copied..index])?;
        f.write_str(escape)?;
        copied = index + c.len_utf8();
    }
    f.write_str(&s[copied..])?;
    f.write_char('"')
}

impl Display for TranslationsFormatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut iter = self.strings.iter();
        if let Some(first) = iter.next() {
            write_json_str(f, first)?;
        }
        for s in iter {
            f.write_char(',')?;
            write_json_str(f, s)?;
        }
        f.write_char(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(strings: &[&str]) -> String {
        let strings: Vec<Rc<str>> = strings.iter().copied().map(Rc::from).collect();
        TranslationsFormatter { strings: &strings }.to_string()
    }

    #[test]
    fn control_characters_use_json_escapes() {
        assert_eq!(format(&["a\0b"]), r#"["a\u0000b"]"#);
        assert_eq!(format(&["\u{7}"]), r#"["\u0007"]"#);
        assert_eq!(format(&["\u{1f}"]), r#"["\u001f"]"#);
        assert_eq!(format(&["\u{8}\u{c}\n\r\t"]), r#"["\b\f\n\r\t"]"#);
        assert_eq!(format(&[r#"say "hi"\"#]), r#"["say \"hi\"\\"]"#);
    }

    #[test]
    fn printable_characters_are_left_alone() {
        // U+007F and combining marks are valid JSON string content, but `Debug`
        // escapes them with the Rust-only `\u{..}` syntax.
        assert_eq!(format(&["x\u{7f}y"]), "[\"x\u{7f}y\"]");
        assert_eq!(format(&["\u{301}a"]), "[\"\u{301}a\"]");
        assert_eq!(format(&["héllo 🎉"]), "[\"héllo 🎉\"]");
    }

    #[test]
    fn array_layout() {
        assert_eq!(format(&[]), "[]");
        assert_eq!(format(&["a"]), r#"["a"]"#);
        assert_eq!(format(&["a", "b", "c"]), r#"["a","b","c"]"#);
    }

    /// The emitted files are fetched and parsed as JSON by the `dynamic_load`
    /// client, so they have to match what a JSON serializer would produce.
    #[test]
    fn matches_serde_json() {
        let strings = [
            "",
            "a\0b",
            "\u{7}\u{8}\u{c}\n\r\t\u{1f}",
            "x\u{7f}y",
            "\u{301}a",
            r#"quotes " and backslashes \"#,
            "héllo 🎉",
        ];
        let rendered = format(&strings);
        assert_eq!(rendered, serde_json::to_string(&strings).unwrap());
        assert_eq!(
            serde_json::from_str::<Vec<String>>(&rendered).unwrap(),
            strings
        );
    }
}
