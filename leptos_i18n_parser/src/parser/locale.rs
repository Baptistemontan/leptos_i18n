use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{
    error::{Diagnostics, Error, Result, Warning},
    parser::{
        ValuesSeed,
        dummy::Dummy,
        options::{Config, FileFormat, ParseOptions},
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
    pub name: Key,
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
    Dummy(Dummy),
}

impl RawLocalesOrNamespaces {
    pub fn new(
        manifest_dir_path: &mut PathBuf,
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        cfg: &Config,
    ) -> Result<Self> {
        manifest_dir_path.push(&cfg.locales_path);
        if !cfg.namespaces.is_empty() {
            let mut namespaces = Vec::with_capacity(cfg.namespaces.len());
            for namespace in &cfg.namespaces {
                namespaces.push(RawNamespace::new(
                    manifest_dir_path,
                    namespace.clone(),
                    &cfg.locales,
                    diag,
                    tracked_files,
                    &cfg.options,
                )?);
            }
            Ok(RawLocalesOrNamespaces::Namespaces(namespaces))
        } else {
            let mut locales = Vec::with_capacity(cfg.locales.len());
            for locale in cfg.locales.iter().cloned() {
                manifest_dir_path.push(&*locale.name);
                let locale_file = find_file(manifest_dir_path, &cfg.options.file_format)?;
                let locale = RawLocale::new(
                    locale_file,
                    manifest_dir_path,
                    locale,
                    None,
                    diag,
                    tracked_files,
                    &cfg.options,
                )?;
                locales.push(locale);
                manifest_dir_path.pop();
            }
            Ok(RawLocalesOrNamespaces::Locales(locales))
        }
    }
}

impl RawNamespace {
    pub fn new(
        locales_dir_path: &mut PathBuf,
        name: Key,
        locale_keys: &[Key],
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        options: &ParseOptions,
    ) -> Result<Self> {
        let mut locales = Vec::with_capacity(locale_keys.len());
        for locale in locale_keys.iter().cloned() {
            let file_path: &Path = name.name.as_ref().as_ref();
            locales_dir_path.push(&*locale.name);
            locales_dir_path.push(file_path);

            let locale_file = find_file(locales_dir_path, &options.file_format)?;

            let locale = RawLocale::new(
                locale_file,
                locales_dir_path,
                locale,
                Some(name.clone()),
                diag,
                tracked_files,
                options,
            )?;

            locales.push(locale);
            locales_dir_path.pop();
            locales_dir_path.pop();
        }
        Ok(RawNamespace { name, locales })
    }
}

impl RawLocale {
    pub fn new(
        locale_file: File,
        path: &mut PathBuf,
        name: Key,
        namespace: Option<Key>,
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        options: &ParseOptions,
    ) -> Result<Self> {
        track_file(tracked_files, &name, namespace.as_ref(), path, diag);

        let seed = ValuesSeed {
            name: name.clone(),
            top_locale_name: name.clone(),
            key_path: KeyPath::new(namespace),
            diag,
            formatters: &options.formatters,
        };

        let values = Self::de(locale_file, path, seed, &options.file_format)?;

        Ok(RawLocale { name, values })
    }

    fn de(
        locale_file: File,
        path: &mut PathBuf,
        seed: ValuesSeed,
        file_format: &FileFormat,
    ) -> Result<RawValues> {
        let reader = BufReader::new(locale_file);
        let values =
            file_format
                .deserialize(reader, path, seed)
                .map_err(|err| Error::LocaleFileDeser {
                    path: std::mem::take(path),
                    err,
                })?;
        Ok(values)
    }
}

fn find_file(path: &mut PathBuf, file_format: &FileFormat) -> Result<File> {
    let mut errs = vec![];

    for ext in file_format.get_files_exts() {
        path.set_extension(ext);
        #[allow(clippy::needless_borrows_for_generic_args)]
        // see https://github.com/rust-lang/rust-clippy/issues/12856
        match File::open(&path) {
            Ok(file) => return Ok(file),
            Err(err) => {
                errs.push((path.to_owned(), err));
            }
        };
    }

    Err(Error::LocaleFileNotFound(errs).into())
}

fn track_file(
    tracked_files: &mut Vec<String>,
    locale: &Key,
    namespace: Option<&Key>,
    path: &Path,
    diag: &Diagnostics,
) {
    if let Some(path) = path.as_os_str().to_str().map(ToOwned::to_owned) {
        tracked_files.push(path);
    } else {
        diag.emit_warning(Warning::NonUnicodePath {
            locale: locale.clone(),
            namespace: namespace.cloned(),
            path: path.to_owned(),
        });
    }
}
