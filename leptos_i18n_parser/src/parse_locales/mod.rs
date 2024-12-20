use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    rc::Rc,
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

use crate::utils::{formatter::SkipIcuCfgGuard, Key, KeyPath, UnwrapAt};

pub const VAR_COUNT_KEY: &str = "var_count";

pub fn parse_locales_raw(
    skip_icu_cfg: bool,
) -> Result<(
    LocalesOrNamespaces,
    ConfigFile,
    ForeignKeysPaths,
    Warnings,
    Vec<String>,
)> {
    let _guard = SkipIcuCfgGuard::new(skip_icu_cfg);

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
    skip_icu_cfg: bool,
) -> Result<BuildersKeys> {
    let _guard = SkipIcuCfgGuard::new(skip_icu_cfg);

    locales.merge_plurals(warnings)?;

    resolve_foreign_keys(&locales, &cfg_file.default, foreign_keys_paths.into_inner())?;

    check_locales(locales, warnings)
}

pub fn parse_locales(skip_icu_cfg: bool) -> Result<(BuildersKeys, Warnings, Vec<String>)> {
    let (locales, cfg_file, foreign_keys_paths, warnings, tracked_files) =
        parse_locales_raw(skip_icu_cfg)?;

    let builder_keys = make_builder_keys(
        locales,
        &cfg_file,
        foreign_keys_paths,
        &warnings,
        skip_icu_cfg,
    )?;

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
            .unwrap_at("resolve_foreign_keys_1");
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
    let default_locale = locales_iter.next().unwrap_at("check_locales_inner_1");
    let mut key_path = KeyPath::new(namespace);

    let mut string_indexer = StringIndexer::default();
    let mut default_keys = default_locale.make_builder_keys(&mut key_path, &mut string_indexer)?;
    default_locale.string = string_indexer.get_string();

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
        locale.string = string_indexer.get_string();
    }

    Ok(default_keys)
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct StringIndex(Rc<Cell<Option<(usize, usize)>>>);

impl StringIndex {
    pub fn get(&self) -> (usize, usize) {
        self.0.get().unwrap()
    }
}

#[derive(Default)]
pub struct StringIndexer(VecDeque<(String, StringIndex)>);

fn make_overlap<'a>(s1: &str, s2: &'a str) -> (&'a str, usize) {
    for i in (1..s1.len().min(s2.len())).rev() {
        let start = s1.len() - i;
        if let Some(prefix) = s1.get(start..) {
            if let Some(s) = s2.strip_prefix(prefix) {
                return (s, start);
            }
        }
    }
    (s2, s1.len())
}

impl StringIndexer {
    pub fn push_str(&mut self, s: String, index: StringIndex) {
        self.0.push_back((s, index));
    }

    pub fn get_string(mut self) -> String {
        let Some((mut buff, first_index)) = self.0.pop_front() else {
            return String::new();
        };
        first_index.0.set(Some((0, buff.len())));
        for (s, index) in self.0 {
            let indices = if let Some(start) = buff.find(&s) {
                (start, start + s.len())
            } else {
                let (to_push, start) = make_overlap(&buff, &s);
                buff.push_str(to_push);
                (start, buff.len())
            };

            index.0.set(Some(indices));
        }

        buff
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
