use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    path::PathBuf,
    rc::Rc,
};

use cfg_file::ConfigFile;
use icu_locale::LanguageIdentifier;
use locale::{BuildersKeys, BuildersKeysInner, DefaultTo, Locale, LocalesOrNamespaces};

pub mod cfg_file;
pub mod error;
pub mod locale;
pub mod options;
pub mod parsed_value;
pub mod plurals;
pub mod ranges;
// pub mod warning;

use error::{Diagnostics, Error, Result};
// use warning::Warnings;

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
    pub diag: Diagnostics,
    pub tracked_files: Vec<String>,
}

pub fn parse_locales_raw(
    cargo_manifest_dir: Option<PathBuf>,
    options: &Options,
) -> Result<RawParsedLocales> {
    let mut cargo_manifest_dir = unwrap_manifest_dir(cargo_manifest_dir)?;

    let foreign_keys_paths = ForeignKeysPaths::new();

    let diag = Diagnostics::new();

    let cfg_file = ConfigFile::new(&mut cargo_manifest_dir)?;

    let mut tracked_files = Vec::with_capacity(
        cfg_file.locales.len() * cfg_file.name_spaces.as_ref().map(Vec::len).unwrap_or(1),
    );

    let locales = LocalesOrNamespaces::new(
        &mut cargo_manifest_dir,
        &cfg_file,
        &foreign_keys_paths,
        &diag,
        &mut tracked_files,
        options,
    )?;

    let raw_parsed_locales = RawParsedLocales {
        locales,
        cfg_file,
        foreign_keys_paths,
        diag,
        tracked_files,
    };

    Ok(raw_parsed_locales)
}

pub fn make_builder_keys(
    mut locales: LocalesOrNamespaces,
    cfg_file: &ConfigFile,
    foreign_keys_paths: ForeignKeysPaths,
    diag: &Diagnostics,
    options: &Options,
) -> Result<BuildersKeys> {
    locales.merge_plurals(diag)?;

    resolve_foreign_keys(&locales, &cfg_file.default, foreign_keys_paths.into_inner())?;

    check_locales(locales, &cfg_file.extensions, diag, options)
}

pub struct ParsedLocales {
    pub cfg_file: ConfigFile,
    pub builder_keys: BuildersKeys,
    pub diag: Diagnostics,
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
        tracked_files,
        diag,
    } = parse_locales_raw(cargo_manifest_dir, &options)?;

    let builder_keys = make_builder_keys(locales, &cfg_file, foreign_keys_paths, &diag, &options)?;

    Ok(ParsedLocales {
        cfg_file,
        builder_keys,
        diag,
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
    diag: &Diagnostics,
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
                    diag,
                    options,
                )?;
                keys.insert(namespace.key.clone(), k);
            }
            Ok(BuildersKeys::NameSpaces { namespaces, keys })
        }
        LocalesOrNamespaces::Locales(mut locales) => {
            let keys = check_locales_inner(&mut locales, None, extensions, diag, options)?;
            Ok(BuildersKeys::Locales { locales, keys })
        }
    }
}

fn find_base_default(icu_locales: &BTreeMap<Key, icu_locale::Locale>, locale: &Key) -> Option<Key> {
    let icu_locale = icu_locales.get(locale).unwrap();

    // if just language, default to default.
    if icu_locale.extensions.is_empty()
        && icu_locale.id.variants.is_empty()
        && icu_locale.id.script.is_none()
        && icu_locale.id.region.is_none()
    {
        return None;
    }

    let mut looking_for = icu_locale::Locale {
        extensions: Default::default(),
        id: LanguageIdentifier {
            language: icu_locale.id.language,
            region: icu_locale.id.region,
            script: None,
            variants: Default::default(),
        },
    };

    // if has extensions, variants or script, find locale with same language and region but no script, variants nor extensions
    if !icu_locale.extensions.is_empty()
        || !icu_locale.id.variants.is_empty()
        || icu_locale.id.script.is_some()
    {
        for (key, loc) in icu_locales {
            if loc == &looking_for {
                return Some(key.clone());
            }
        }
    }

    // if not found, relax region bound
    looking_for.id.region = None;

    for (key, loc) in icu_locales {
        if loc == &looking_for {
            return Some(key.clone());
        }
    }

    None
}

fn get_locale_fallback(
    extensions: &BTreeMap<Key, Key>,
    icu_locales: &BTreeMap<Key, icu_locale::Locale>,
    default_locale: &Key,
    locale: &Key,
    suppress_key_warnings: bool,
) -> DefaultTo {
    extensions
        .get(locale)
        .cloned()
        // if some it has an explicit default
        // if none then try to find a base locale
        .or_else(|| find_base_default(icu_locales, locale))
        .map(DefaultTo::Explicit)
        // if neither, fallback to default locale.
        .unwrap_or_else(|| {
            if suppress_key_warnings {
                DefaultTo::Explicit(default_locale.clone())
            } else {
                DefaultTo::Implicit(default_locale.clone())
            }
        })
}

fn locales_to_icu(locales: &[Locale]) -> Result<BTreeMap<Key, icu_locale::Locale>> {
    locales
        .iter()
        .map(|locale| locale.top_locale_name.clone())
        .map(
            |locale| match icu_locale::Locale::try_from_str(&locale.name) {
                Ok(icu_locale) => Ok((locale, icu_locale)),
                Err(err) => Err(Error::InvalidLocale {
                    locale: locale.name,
                    err,
                }
                .into()),
            },
        )
        .collect()
}

fn check_locales_inner(
    locales: &mut [Locale],
    namespace: Option<Key>,
    extensions: &BTreeMap<Key, Key>,
    diag: &Diagnostics,
    options: &Options,
) -> Result<BuildersKeysInner> {
    let icu_locales = locales_to_icu(locales)?;
    let (default_locale, other_locales) =
        locales.split_first_mut().unwrap_at("check_locales_inner_1");
    let mut key_path = KeyPath::new(namespace);

    let mut string_indexer = StringIndexer::default();
    let mut default_keys = default_locale.make_builder_keys(&mut key_path, &mut string_indexer)?;
    default_locale.strings = string_indexer.get_strings();
    default_locale.top_locale_string_count = default_locale.strings.len();

    for locale in other_locales {
        let top_locale = locale.top_locale_name.clone();

        let default_to = get_locale_fallback(
            extensions,
            &icu_locales,
            &default_locale.top_locale_name,
            &top_locale,
            options.suppress_key_warnings,
        );

        let mut string_indexer = StringIndexer::default();

        locale.merge(
            &mut default_keys,
            top_locale,
            &default_to,
            &mut key_path,
            &mut string_indexer,
            diag,
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! make_icu_locale {
        ($val: literal) => {
            (Key::new($val).unwrap(), icu_locale::locale!($val))
        };
    }

    macro_rules! check_fallback {
        ($locales: ident, $val: literal) => {{
            let name = Key::new($val).unwrap();
            let found = find_base_default(&$locales, &name);
            assert!(found.is_none());
        }};
        ($locales: ident, $val: literal, $expected: literal) => {{
            let name = Key::new($val).unwrap();
            let expected = Key::new($expected).unwrap();
            let found = find_base_default(&$locales, &name).unwrap();
            assert_eq!(found, expected);
        }};
    }

    fn get_icu_locales() -> BTreeMap<Key, icu_locale::Locale> {
        BTreeMap::from_iter([
            // default
            make_icu_locale!("en"),
            // base
            make_icu_locale!("en"),
            make_icu_locale!("fr"),
            // base with region
            make_icu_locale!("en-US"),
            make_icu_locale!("fr-FR"),
            // locales with extension/script/variants
            make_icu_locale!("fr-FR-u-ca-buddhist"),
            make_icu_locale!("fr-u-ca-buddhist"),
            make_icu_locale!("en-Latn-US-Valencia"),
            make_icu_locale!("en-Latn-US-Valencia-u-ca-buddhist"),
            make_icu_locale!("en-Latn-US-u-ca-buddhist"),
            make_icu_locale!("en-Valencia"),
            make_icu_locale!("en-Latn"),
        ])
    }

    #[test]
    fn test_locale_defaulting() {
        // fr-FR-u-ca-buddhist => fr-FR => fr => en (default)
        // fr-u-ca-buddhist => fr => en (default)
        // fr-FR => fr => en (default)
        // fr => en (default)
        // en-Latn-US-Valencia => en-US => en
        // en-Latn-US-Valencia-u-ca-buddhist => en-US => en
        // en-Latn-US-u-ca-buddhist => en-US => en
        // en-Valencia => en
        // en-Latn => en
        // en => en
        let icu_locales = get_icu_locales();
        check_fallback!(icu_locales, "fr-FR-u-ca-buddhist", "fr-FR");
        check_fallback!(icu_locales, "fr-u-ca-buddhist", "fr");
        check_fallback!(icu_locales, "fr-FR", "fr");
        check_fallback!(icu_locales, "fr"); // default
        check_fallback!(icu_locales, "en-Latn-US-Valencia", "en-US");
        check_fallback!(icu_locales, "en-Latn-US-Valencia-u-ca-buddhist", "en-US");
        check_fallback!(icu_locales, "en-Latn-US-u-ca-buddhist", "en-US");
        check_fallback!(icu_locales, "en-Valencia", "en");
        check_fallback!(icu_locales, "en-Latn", "en");
        check_fallback!(icu_locales, "en-US", "en");
        check_fallback!(icu_locales, "en"); // default
    }
}
