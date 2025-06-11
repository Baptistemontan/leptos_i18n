use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    path::PathBuf,
    rc::Rc,
};

use cfg_file::ConfigFile;
use locale::{BuildersKeys, BuildersKeysInner, DefaultTo, Locale, LocalesOrNamespaces};

pub mod cfg_file;
pub mod error;
pub mod locale;
pub mod options;
pub mod parsed_value;
pub mod plurals;
pub mod ranges;
pub mod warning;

use error::{Error, Errors, Result};
use warning::Warnings;

use crate::{
    parse_locales::options::Options,
    utils::{Key, KeyPath, UnwrapAt},
};

pub const VAR_COUNT_KEY: &str = "var_count";

fn get_manifest_dir() -> Result<PathBuf> {
    let path = std::env::var("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .map_err(Error::CargoDirEnvNotPresent)?;

    Ok(path)
}

fn unwrap_manifest_dir(cargo_manifest_dir: Option<PathBuf>) -> Result<PathBuf> {
    match cargo_manifest_dir {
        Some(path) => Ok(path),
        None => get_manifest_dir(),
    }
}

pub struct RawParsedLocales {
    pub locales: LocalesOrNamespaces,
    pub cfg_file: ConfigFile,
    pub foreign_keys_paths: ForeignKeysPaths,
    pub warnings: Warnings,
    pub errors: Errors,
    pub tracked_files: Vec<String>,
}

pub fn parse_locales_raw(
    cargo_manifest_dir: Option<PathBuf>,
    options: &Options,
) -> Result<RawParsedLocales> {
    let mut cargo_manifest_dir = unwrap_manifest_dir(cargo_manifest_dir)?;

    let foreign_keys_paths = ForeignKeysPaths::new();

    let errors = Errors::new();

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
        &errors,
        &mut tracked_files,
        options,
    )?;

    let raw_parsed_locales = RawParsedLocales {
        locales,
        cfg_file,
        foreign_keys_paths,
        warnings,
        errors,
        tracked_files,
    };

    Ok(raw_parsed_locales)
}

pub fn make_builder_keys(
    mut locales: LocalesOrNamespaces,
    cfg_file: &ConfigFile,
    foreign_keys_paths: ForeignKeysPaths,
    warnings: &Warnings,
    options: &Options,
) -> Result<BuildersKeys> {
    locales.merge_plurals(warnings)?;

    resolve_foreign_keys(&locales, &cfg_file.default, foreign_keys_paths.into_inner())?;

    check_locales(locales, &cfg_file.extensions, warnings, options)
}

pub struct ParsedLocales {
    pub cfg_file: ConfigFile,
    pub builder_keys: BuildersKeys,
    pub warnings: Warnings,
    pub errors: Errors,
    pub tracked_files: Option<Vec<String>>,
    pub options: Options,
}

pub fn parse_locales(
    cargo_manifest_dir: Option<PathBuf>,
    options: Options,
) -> Result<ParsedLocales> {
    let RawParsedLocales {
        locales,
        cfg_file,
        foreign_keys_paths,
        warnings,
        tracked_files,
        errors,
    } = parse_locales_raw(cargo_manifest_dir, &options)?;

    let builder_keys =
        make_builder_keys(locales, &cfg_file, foreign_keys_paths, &warnings, &options)?;

    Ok(ParsedLocales {
        cfg_file,
        builder_keys,
        warnings,
        errors,
        tracked_files: Some(tracked_files),
        options,
    })
}

fn resolve_foreign_keys(
    values: &LocalesOrNamespaces,
    default_locale: &Key,
    foreign_keys_paths: BTreeSet<(Key, KeyPath)>,
) -> Result<()> {
    for (locale, value_path) in foreign_keys_paths {
        let value = values
            .get_value_at(&locale, &value_path)
            .unwrap_at("resolve_foreign_keys_1");
        value.resolve_foreign_key(values, &locale, default_locale, &value_path)?;
    }
    Ok(())
}

fn check_locales(
    locales: LocalesOrNamespaces,
    extensions: &BTreeMap<Key, Key>,
    warnings: &Warnings,
    options: &Options,
) -> Result<BuildersKeys> {
    match locales {
        LocalesOrNamespaces::NameSpaces(mut namespaces) => {
            let mut keys = BTreeMap::new();
            for namespace in &mut namespaces {
                let k = check_locales_inner(
                    &mut namespace.locales,
                    Some(namespace.key.clone()),
                    extensions,
                    warnings,
                    options,
                )?;
                keys.insert(namespace.key.clone(), k);
            }
            Ok(BuildersKeys::NameSpaces { namespaces, keys })
        }
        LocalesOrNamespaces::Locales(mut locales) => {
            let keys = check_locales_inner(&mut locales, None, extensions, warnings, options)?;
            Ok(BuildersKeys::Locales { locales, keys })
        }
    }
}

fn find_base_default(name: &Key, default_locale: &Locale, locales: &[Locale]) -> Option<Key> {
    // the "robust way" would be to parse into ICU4X LanguageIdentifier and check the language field, but
    // valid Unicode language identifier starts with the language and the only valid sepertors are "-" and "_", so just check that:

    // if None, already base language
    let (language, _) = name.name.split_once(&['-', '_'])?;

    // check if any locale is just the base language

    // technically we check on itself if it matches but it is'nt a problem,
    // if it could match it would have returned previous step
    // and it's more work to filter it out than just try the impossible
    locales
        .iter()
        .chain(Some(default_locale))
        .map(|locale| &locale.top_locale_name)
        .find(|locale| locale.name.as_ref() == language)
        .cloned()
}

fn get_locale_with_default<'a>(
    extensions: &BTreeMap<Key, Key>,
    default_locale: &Locale,
    locales: &'a mut [Locale],
    locale_idx: usize,
    suppress_key_warnings: bool,
) -> (&'a mut Locale, DefaultTo) {
    let locale_name = locales[locale_idx].name.clone();

    let default = extensions
        .get(&locale_name)
        .cloned()
        .or_else(|| find_base_default(&locale_name, default_locale, locales))
        .map(DefaultTo::Explicit)
        .unwrap_or_else(|| {
            if suppress_key_warnings {
                DefaultTo::Explicit(default_locale.top_locale_name.clone())
            } else {
                DefaultTo::Implicit(default_locale.top_locale_name.clone())
            }
        });

    (&mut locales[locale_idx], default)
}

fn check_locales_inner(
    locales: &mut [Locale],
    namespace: Option<Key>,
    extensions: &BTreeMap<Key, Key>,
    warnings: &Warnings,
    options: &Options,
) -> Result<BuildersKeysInner> {
    let (default_locale, other_locales) =
        locales.split_first_mut().unwrap_at("check_locales_inner_1");
    let mut key_path = KeyPath::new(namespace);

    let mut string_indexer = StringIndexer::default();
    let mut default_keys = default_locale.make_builder_keys(&mut key_path, &mut string_indexer)?;
    default_locale.strings = string_indexer.get_strings();
    default_locale.top_locale_string_count = default_locale.strings.len();

    for locale_idx in 0..other_locales.len() {
        let (locale, default_to) = get_locale_with_default(
            extensions,
            default_locale,
            other_locales,
            locale_idx,
            options.suppress_key_warnings,
        );

        let top_locale = locale.name.clone();
        let mut string_indexer = StringIndexer::default();

        locale.merge(
            &mut default_keys,
            top_locale,
            &default_to,
            &mut key_path,
            &mut string_indexer,
            warnings,
            options,
        )?;
        locale.strings = string_indexer.get_strings();
        locale.top_locale_string_count = locale.strings.len();
    }

    default_keys.propagate_string_count(locales);

    Ok(default_keys)
}

#[derive(Default)]
pub struct StringIndexer {
    current: HashMap<Rc<str>, usize>,
    acc: Vec<Rc<str>>,
}

impl StringIndexer {
    pub fn push_str(&mut self, s: &str) -> usize {
        if let Some(index) = self.current.get(s) {
            *index
        } else {
            let i = self.acc.len();
            let s: Rc<str> = Rc::from(s);
            self.acc.push(s.clone());
            self.current.insert(s, i);
            i
        }
    }

    pub fn get_strings(self) -> Vec<Rc<str>> {
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
