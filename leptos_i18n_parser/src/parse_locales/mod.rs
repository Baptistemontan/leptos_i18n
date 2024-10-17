use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    path::PathBuf,
};

use cfg_file::ConfigFile;
use locale::{BuildersKeys, BuildersKeysInner, Locale, LocalesOrNamespaces};

pub mod cfg_file;
pub mod error;
pub mod locale;
pub mod parsed_value;
pub mod plurals;
pub mod ranges;
pub mod warning;

use error::{Error, Result};
use warning::Warnings;

use crate::utils::{Key, KeyPath};

pub const VAR_COUNT_KEY: &str = "var_count";

pub fn parse_locales_raw() -> Result<(
    LocalesOrNamespaces,
    ConfigFile,
    ForeignKeysPaths,
    Warnings,
    Vec<String>,
)> {
    let mut cargo_manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(Error::CargoDirEnvNotPresent)?
        .into();

    let foreign_keys_paths = ForeignKeysPaths::new();

    let cfg_file = ConfigFile::new(&mut cargo_manifest_dir)?;

    let warnings = Warnings::new();

    let mut tracked_files = Vec::with_capacity(
        cfg_file.locales.len() * cfg_file.name_spaces.as_ref().map(Vec::len).unwrap_or(1),
    );

    let locales = LocalesOrNamespaces::new(
        &mut cargo_manifest_dir,
        &cfg_file,
        &foreign_keys_paths,
        &warnings,
        &mut tracked_files,
    )?;

    Ok((
        locales,
        cfg_file,
        foreign_keys_paths,
        warnings,
        tracked_files,
    ))
}

pub fn make_builder_keys(
    mut locales: LocalesOrNamespaces,
    cfg_file: &ConfigFile,
    foreign_keys_paths: ForeignKeysPaths,
    warnings: &Warnings,
) -> Result<BuildersKeys> {
    locales.merge_plurals(warnings)?;

    resolve_foreign_keys(&locales, &cfg_file.default, foreign_keys_paths.into_inner())?;

    check_locales(locales, warnings)
}

pub fn parse_locales() -> Result<(BuildersKeys, Warnings, Vec<String>)> {
    let (locales, cfg_file, foreign_keys_paths, warnings, tracked_files) = parse_locales_raw()?;

    let builder_keys = make_builder_keys(locales, &cfg_file, foreign_keys_paths, &warnings)?;

    Ok((builder_keys, warnings, tracked_files))
}

fn resolve_foreign_keys(
    values: &LocalesOrNamespaces,
    default_locale: &Key,
    foreign_keys_paths: BTreeSet<(Key, KeyPath)>,
) -> Result<()> {
    for (locale, value_path) in foreign_keys_paths {
        let value = values
            .get_value_at(&locale, &value_path)
            .expect("The foreign key to be present a that path.");
        value.resolve_foreign_key(values, &locale, default_locale, &value_path)?;
    }
    Ok(())
}

fn check_locales(locales: LocalesOrNamespaces, warnings: &Warnings) -> Result<BuildersKeys> {
    match locales {
        LocalesOrNamespaces::NameSpaces(mut namespaces) => {
            let mut keys = BTreeMap::new();
            for namespace in &mut namespaces {
                let k = check_locales_inner(
                    &mut namespace.locales,
                    Some(namespace.key.clone()),
                    warnings,
                )?;
                keys.insert(namespace.key.clone(), k);
            }
            Ok(BuildersKeys::NameSpaces { namespaces, keys })
        }
        LocalesOrNamespaces::Locales(mut locales) => {
            let keys = check_locales_inner(&mut locales, None, warnings)?;
            Ok(BuildersKeys::Locales { locales, keys })
        }
    }
}

fn check_locales_inner(
    locales: &mut [Locale],
    namespace: Option<Key>,
    warnings: &Warnings,
) -> Result<BuildersKeysInner> {
    let mut locales_iter = locales.iter_mut();
    let default_locale = locales_iter
        .next()
        .expect("There should be at least one Locale");
    let mut key_path = KeyPath::new(namespace);

    let mut string_indexer = StringIndexer::default();
    let mut default_keys = default_locale.make_builder_keys(&mut key_path, &mut string_indexer)?;
    default_locale.strings = string_indexer.get_strings();
    default_locale.top_locale_string_count = default_locale.strings.len();

    for locale in locales_iter {
        let top_locale = locale.name.clone();
        let mut string_indexer = StringIndexer::default();
        locale.merge(
            &mut default_keys,
            default_locale,
            top_locale,
            &mut key_path,
            &mut string_indexer,
            warnings,
        )?;
        locale.strings = string_indexer.get_strings();
        locale.top_locale_string_count = locale.strings.len()
    }

    default_keys.propagate_string_count(locales);

    Ok(default_keys)
}

#[derive(Default)]
pub struct StringIndexer {
    current: HashSet<String>,
    acc: Vec<String>,
}

impl StringIndexer {
    pub fn push_str(&mut self, s: String) -> usize {
        if self.current.contains(&s) {
            self.acc.iter().position(|i| i == &s).unwrap_or(usize::MAX)
        } else {
            let i = self.acc.len();
            self.acc.push(s.clone());
            self.current.insert(s);
            i
        }
    }

    pub fn get_strings(self) -> Vec<String> {
        self.acc
    }
}

#[derive(Default, Debug)]
pub struct ForeignKeysPaths(RefCell<BTreeSet<(Key, KeyPath)>>);

impl ForeignKeysPaths {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_path(&self, locale: Key, path: KeyPath) {
        self.0.borrow_mut().insert((locale, path));
    }

    pub fn into_inner(self) -> BTreeSet<(Key, KeyPath)> {
        self.0.into_inner()
    }
}
