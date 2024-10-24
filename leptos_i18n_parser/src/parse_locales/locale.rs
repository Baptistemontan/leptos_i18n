use serde::de::MapAccess;

use crate::utils::formatter::{Formatter, SKIP_ICU_CFG};
use crate::utils::{Key, KeyPath};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use super::cfg_file::ConfigFile;
use super::error::{Error, Result};
use super::parsed_value::{ParsedValue, ParsedValueSeed};
use super::plurals::{PluralForm, PluralRuleType, Plurals};
use super::ranges::RangeType;
use super::warning::{Warning, Warnings};
use super::{ForeignKeysPaths, StringIndexer, VAR_COUNT_KEY};

macro_rules! define_by_format {
    (json => $($tt:tt)*) => {
        #[cfg(all(feature = "json_files", not(any(feature = "yaml_files", feature = "json5_files"))))]
        $($tt)*
    };
    (yaml => $($tt:tt)*) => {
        #[cfg(all(feature = "yaml_files", not(any(feature = "json_files", feature = "json5_files"))))]
        $($tt)*
    };
    (json5 => $($tt:tt)*) => {
        #[cfg(all(feature = "json5_files", not(any(feature = "json_files", feature = "yaml_files"))))]
        $($tt)*
    };
    (none => $($tt:tt)*) => {
        #[cfg(not(any(feature = "json_files", feature = "yaml_files", feature = "json5_files")))]
        $($tt)*
    };
    // This is attrocious, found a better way fgs
    (multiple => $($tt:tt)*) => {
        #[cfg(any(all(feature = "json_files", feature = "yaml_files"), all(feature = "json_files", feature = "json5_files"), all(feature = "yaml_files", feature = "json5_files")))]
        $($tt)*
    }
}

macro_rules! define_error {
    ($ident:ident => $t:ty) => {
        define_by_format!($ident => pub type SerdeError = $t;);
    };
}

macro_rules! define_files_exts {
    ($ident:ident => $($lit:literal),*) => {
        define_by_format!($ident => const FILE_EXTS: &[&str] = &[$($lit,)*];);
    };
    ($ident:ident) => {
        define_by_format!($ident => const FILE_EXTS: &[&str] = &[];);
    };
}

#[cfg(feature = "json5_files")]
#[derive(Debug)]
pub enum Json5Error {
    Serde(json5::Error),
    Io(std::io::Error),
}

#[cfg(feature = "json5_files")]
impl std::fmt::Display for Json5Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Json5Error::Serde(error) => std::fmt::Display::fmt(error, f),
            Json5Error::Io(error) => std::fmt::Display::fmt(error, f),
        }
    }
}

define_error!(json => serde_json::Error);
define_error!(json5 => Json5Error);
define_error!(yaml => serde_yaml::Error);
define_error!(none => &'static str); // whatever impl Display
define_error!(multiple => &'static str); // whatever impl Display

define_files_exts!(json => "json");
define_files_exts!(json5 => "json5");
define_files_exts!(yaml => "yaml", "yml");
define_files_exts!(none);
define_files_exts!(multiple);

define_by_format!(json =>
    fn de_inner<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
        let mut deserializer = serde_json::Deserializer::from_reader(locale_file);
        serde::de::DeserializeSeed::deserialize(seed, &mut deserializer)
    }
);
define_by_format!(json5 =>
    fn de_inner<R: Read>(mut locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
        let mut buff = String::new();
        Read::read_to_string(&mut locale_file, &mut buff).map_err(Json5Error::Io)?;
        let mut deserializer = json5::Deserializer::from_str(&buff).map_err(Json5Error::Serde)?;
        serde::de::DeserializeSeed::deserialize(seed, &mut deserializer).map_err(Json5Error::Serde)
    }
);
define_by_format!(yaml =>
    fn de_inner<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
        let deserializer = serde_yaml::Deserializer::from_reader(locale_file);
        serde::de::DeserializeSeed::deserialize(seed, deserializer)
    }
);
define_by_format!(none =>
    fn de_inner<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
        let _ = (locale_file, seed);
        compile_error!("No file format has been provided for leptos_i18n, supported formats are: json and yaml")
    }
);
define_by_format!(multiple =>
    fn de_inner<R: Read>(locale_file: R, seed: LocaleSeed) -> Result<Locale, SerdeError> {
        let _ = (locale_file, seed);
        compile_error!("Multiple file format have been provided for leptos_i18n, choose only one, supported formats are: json and yaml")
    }
);

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub top_locale_name: Key,
    pub name: Key,
    pub keys: BTreeMap<Key, ParsedValue>,
    pub strings: Vec<String>,
    pub top_locale_string_count: usize,
}

#[derive(Debug)]
pub struct Namespace {
    pub key: Key,
    pub locales: Vec<Locale>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeOrPlural {
    Range(RangeType),
    Plural,
}

#[derive(Debug)]
pub enum LocalesOrNamespaces {
    NameSpaces(Vec<Namespace>),
    Locales(Vec<Locale>),
}

#[derive(Debug, Default)]
pub struct VarInfo {
    pub formatters: BTreeSet<Formatter>,
    pub range_count: Option<RangeOrPlural>,
}

#[derive(Debug, Default)]
pub struct InterpolationKeys {
    components: BTreeSet<Key>,
    variables: BTreeMap<Key, VarInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralType {
    String,
    Bool,
    Signed,
    Unsigned,
    Float,
}

#[derive(Debug)]
pub enum InterpolOrLit {
    Interpol(InterpolationKeys),
    Lit(LiteralType),
}

#[derive(Debug)]
pub enum LocaleValue {
    Value(InterpolOrLit),
    Subkeys {
        locales: Vec<Locale>,
        keys: BuildersKeysInner,
    },
}

#[derive(Default, Debug)]
pub struct BuildersKeysInner(pub BTreeMap<Key, LocaleValue>);

pub enum BuildersKeys {
    NameSpaces {
        namespaces: Vec<Namespace>,
        keys: BTreeMap<Key, BuildersKeysInner>,
    },
    Locales {
        locales: Vec<Locale>,
        keys: BuildersKeysInner,
    },
}

#[derive(Debug, Clone)]
pub struct LocaleSeed<'a> {
    pub name: Key,
    pub top_locale_name: Key,
    pub key_path: KeyPath,
    pub foreign_keys_paths: &'a ForeignKeysPaths,
}

fn find_file(path: &mut PathBuf) -> Result<File> {
    let mut errs = vec![];

    for ext in FILE_EXTS {
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

    Err(Error::LocaleFileNotFound(errs))
}

impl InterpolOrLit {
    pub fn get_interpol_keys_mut(&mut self) -> &mut InterpolationKeys {
        match self {
            InterpolOrLit::Interpol(keys) => keys,
            InterpolOrLit::Lit(_) => {
                *self = InterpolOrLit::Interpol(InterpolationKeys::default());
                self.get_interpol_keys_mut()
            }
        }
    }

    pub fn is_interpol(&self) -> Option<&InterpolationKeys> {
        match self {
            InterpolOrLit::Interpol(keys) => Some(keys),
            InterpolOrLit::Lit(_) => None,
        }
    }
}

impl InterpolationKeys {
    pub fn push_var(&mut self, key: Key, formatter: Formatter) {
        let var_infos = self.variables.entry(key).or_default();
        var_infos.formatters.insert(formatter);
    }

    pub fn push_comp(&mut self, key: Key) {
        self.components.insert(key);
    }

    pub fn push_count(
        &mut self,
        key_path: &mut KeyPath,
        ty: RangeOrPlural,
        count_key: Key,
    ) -> Result<()> {
        let var_infos = self.variables.entry(count_key).or_default();
        match (var_infos.range_count.replace(ty), ty) {
            (None, _) | (Some(RangeOrPlural::Plural), RangeOrPlural::Plural) => Ok(()),
            (Some(RangeOrPlural::Range(old)), RangeOrPlural::Range(new)) if old == new => Ok(()),
            (Some(RangeOrPlural::Plural), RangeOrPlural::Range(_))
            | (Some(RangeOrPlural::Range(_)), RangeOrPlural::Plural) => {
                Err(Error::RangeAndPluralsMix {
                    key_path: std::mem::take(key_path),
                })
            }
            (Some(RangeOrPlural::Range(old)), RangeOrPlural::Range(new)) => {
                Err(Error::RangeTypeMissmatch {
                    key_path: std::mem::take(key_path),
                    type1: old,
                    type2: new,
                })
            }
        }
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Key> {
        self.components.iter().chain(self.variables.keys())
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = (Key, &VarInfo)> {
        self.variables
            .iter()
            .map(|(key, value)| (key.clone(), value))
    }

    pub fn iter_comps(&self) -> impl Iterator<Item = Key> + '_ {
        self.components.iter().cloned()
    }
}

impl BuildersKeysInner {
    pub fn propagate_string_count(&mut self, top_locales: &[Locale]) {
        for value in self.0.values_mut() {
            if let LocaleValue::Subkeys { locales, keys } = value {
                for (locale, top_locale) in locales.iter_mut().zip(top_locales) {
                    locale.top_locale_string_count = top_locale.top_locale_string_count;
                }
                keys.propagate_string_count(top_locales);
            }
        }
    }
}

impl Namespace {
    pub fn new(
        locales_dir_path: &mut PathBuf,
        key: Key,
        locale_keys: &[Key],
        foreign_keys_paths: &ForeignKeysPaths,
        warnings: &Warnings,
        tracked_files: &mut Vec<String>,
    ) -> Result<Self> {
        let mut locales = Vec::with_capacity(locale_keys.len());
        for locale in locale_keys.iter().cloned() {
            let file_path: &Path = key.name.as_ref().as_ref();
            locales_dir_path.push(&*locale.name);
            locales_dir_path.push(file_path);

            let locale_file = find_file(locales_dir_path)?;

            let locale = Locale::new(
                locale_file,
                locales_dir_path,
                locale,
                Some(key.clone()),
                foreign_keys_paths,
                warnings,
                tracked_files,
            )?;

            locales.push(locale);
            locales_dir_path.pop();
            locales_dir_path.pop();
        }
        Ok(Namespace { key, locales })
    }
}

impl LocalesOrNamespaces {
    pub fn new(
        manifest_dir_path: &mut PathBuf,
        cfg_file: &ConfigFile,
        foreign_keys_paths: &ForeignKeysPaths,
        warnings: &Warnings,
        tracked_files: &mut Vec<String>,
    ) -> Result<Self> {
        let locale_keys = &cfg_file.locales;
        manifest_dir_path.push(&*cfg_file.locales_dir);
        if let Some(namespace_keys) = &cfg_file.name_spaces {
            let mut namespaces = Vec::with_capacity(namespace_keys.len());
            for namespace in namespace_keys {
                namespaces.push(Namespace::new(
                    manifest_dir_path,
                    namespace.clone(),
                    locale_keys,
                    foreign_keys_paths,
                    warnings,
                    tracked_files,
                )?);
            }
            Ok(LocalesOrNamespaces::NameSpaces(namespaces))
        } else {
            let mut locales = Vec::with_capacity(locale_keys.len());
            for locale in locale_keys.iter().cloned() {
                manifest_dir_path.push(&*locale.name);
                let locale_file = find_file(manifest_dir_path)?;
                let locale = Locale::new(
                    locale_file,
                    manifest_dir_path,
                    locale,
                    None,
                    foreign_keys_paths,
                    warnings,
                    tracked_files,
                )?;
                locales.push(locale);
                manifest_dir_path.pop();
            }
            Ok(LocalesOrNamespaces::Locales(locales))
        }
    }

    pub fn merge_plurals_inner(
        locales: &mut [Locale],
        namespace: Option<Key>,
        warnings: &Warnings,
    ) -> Result<()> {
        let mut key_path = KeyPath::new(namespace);

        for locale in locales {
            let top_locale = locale.name.clone();
            locale.merge_plurals(top_locale.clone(), &mut key_path, warnings)?;
        }

        Ok(())
    }

    // this step would be more optimized to be done during `check_locales` but plurals merging need to be done before foreign key resolution,
    // which also need to be done before `check_locales`.
    pub fn merge_plurals(&mut self, warnings: &Warnings) -> Result<()> {
        match self {
            LocalesOrNamespaces::NameSpaces(namespaces) => {
                for namespace in namespaces {
                    Self::merge_plurals_inner(
                        &mut namespace.locales,
                        Some(namespace.key.clone()),
                        warnings,
                    )?;
                }
                Ok(())
            }
            LocalesOrNamespaces::Locales(locales) => {
                Self::merge_plurals_inner(&mut *locales, None, warnings)
            }
        }
    }

    pub fn get_value_at(&self, top_locale: &Key, path: &KeyPath) -> Option<&'_ ParsedValue> {
        let locale = match (&path.namespace, self) {
            (None, LocalesOrNamespaces::NameSpaces(_))
            | (Some(_), LocalesOrNamespaces::Locales(_)) => None,
            (None, LocalesOrNamespaces::Locales(locales)) => {
                locales.iter().find(|locale| &locale.name == top_locale)
            }
            (Some(target_namespace), LocalesOrNamespaces::NameSpaces(namespaces)) => {
                let namespace = namespaces.iter().find(|ns| &ns.key == target_namespace)?;

                namespace
                    .locales
                    .iter()
                    .find(|locale| &locale.name == top_locale)
            }
        }?;

        locale.get_value_at(&path.path)
    }
}

impl Locale {
    pub fn new(
        locale_file: File,
        path: &mut PathBuf,
        locale: Key,
        namespace: Option<Key>,
        foreign_keys_paths: &ForeignKeysPaths,
        warnings: &Warnings,
        tracked_files: &mut Vec<String>,
    ) -> Result<Self> {
        track_file(tracked_files, &locale, namespace.as_ref(), path, warnings);

        let seed = LocaleSeed {
            name: locale.clone(),
            top_locale_name: locale,
            key_path: KeyPath::new(namespace),
            foreign_keys_paths,
        };

        Self::de(locale_file, path, seed)
    }

    fn de(locale_file: File, path: &mut PathBuf, seed: LocaleSeed) -> Result<Self> {
        let reader = BufReader::new(locale_file);
        de_inner(reader, seed).map_err(|err| Error::LocaleFileDeser {
            path: std::mem::take(path),
            err,
        })
    }

    pub fn get_value_at(&self, path: &[Key]) -> Option<&'_ ParsedValue> {
        match path {
            [] => None,
            [key] => self.keys.get(key),
            [key, path @ ..] => {
                let value = self.keys.get(key)?;
                let ParsedValue::Subkeys(subkeys) = value else {
                    return None;
                };
                match subkeys {
                    None => unreachable!("called get_value_at on empty subkeys. If you got this error please open an issue on github."),
                    Some(subkeys) => subkeys.get_value_at(path)
                }
            }
        }
    }

    pub fn is_possible_plural<'a>(
        key: &'a Key,
        value: &ParsedValue,
    ) -> Option<(&'a str, PluralRuleType, PluralForm)> {
        if matches!(value, ParsedValue::Ranges(_) | ParsedValue::Subkeys(_)) {
            return None;
        }
        let (base_key, suffix) = key.name.rsplit_once('_')?;
        let (base_key, rule_type) = match base_key.strip_suffix("_ordinal") {
            Some(base_key) => (base_key, PluralRuleType::Ordinal),
            None => (base_key, PluralRuleType::Cardinal),
        };

        PluralForm::try_from_str(suffix).map(|form| (base_key, rule_type, form))
    }

    pub fn merge_plurals(
        &mut self,
        locale: Key,
        key_path: &mut KeyPath,
        warnings: &Warnings,
    ) -> Result<()> {
        let keys = std::mem::take(&mut self.keys);
        #[allow(clippy::type_complexity)]
        let mut possible_plurals: BTreeMap<
            String,
            BTreeMap<PluralForm, (Key, PluralRuleType, ParsedValue)>,
        > = BTreeMap::new();
        for (key, mut value) in keys {
            if let ParsedValue::Subkeys(Some(subkeys)) = &mut value {
                key_path.push_key(key.clone());
                subkeys.merge_plurals(locale.clone(), key_path, warnings)?;
                key_path.pop_key();
            }
            if let Some((base_key, rule_type, plural_form)) = Self::is_possible_plural(&key, &value)
            {
                let map = possible_plurals.entry(base_key.to_owned()).or_default();
                map.insert(plural_form, (key, rule_type, value));
            } else {
                self.keys.insert(key, value);
            }
        }
        for (base_key, mut plurals) in possible_plurals {
            if plurals.len() == 1 {
                for (_, (key, _, value)) in plurals {
                    self.keys.insert(key, value);
                }
                continue;
            }
            let Some((_, rule_type, other)) = plurals.remove(&PluralForm::Other) else {
                for (_, (key, _, value)) in plurals {
                    self.keys.insert(key, value);
                }
                continue;
            };
            let key = Key::new(&base_key).unwrap();
            key_path.push_key(key);
            if !cfg!(feature = "plurals") && !SKIP_ICU_CFG.get() {
                return Err(Error::DisabledPlurals {
                    locale: locale.clone(),
                    key_path: std::mem::take(key_path),
                });
            }

            let forms = plurals
                .into_iter()
                .map(|(form, (_, rule, value))| {
                    if rule == rule_type {
                        Ok((form, value))
                    } else {
                        Err(Error::ConflictingPluralRuleType {
                            locale: locale.clone(),
                            key_path: std::mem::take(key_path),
                        })
                    }
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            let plural = Plurals {
                rule_type,
                forms,
                count_key: Key::new(VAR_COUNT_KEY).unwrap(),
                other: Box::new(other),
            };
            plural.check_categories(&locale, key_path, warnings);
            let value = ParsedValue::Plurals(plural);
            let key = key_path
                .pop_key()
                .expect("The KeyPath should not be empty.");
            if self.keys.insert(key.clone(), value).is_some() {
                key_path.push_key(key);
                return Err(Error::PluralsAtNormalKey {
                    locale,
                    key_path: std::mem::take(key_path),
                });
            }
        }

        Ok(())
    }

    pub fn merge(
        &mut self,
        keys: &mut BuildersKeysInner,
        default_locale: &Self,
        top_locale: Key,
        key_path: &mut KeyPath,
        strings: &mut StringIndexer,
        warnings: &Warnings,
    ) -> Result<()> {
        for (key, keys) in &mut keys.0 {
            key_path.push_key(key.clone());
            if let Some((value, def)) = self.keys.get_mut(key).zip(default_locale.keys.get(key)) {
                value.merge(def, keys, self.name.clone(), key_path, strings, warnings)?;
            } else {
                warnings.emit_warning(Warning::MissingKey {
                    locale: top_locale.clone(),
                    key_path: key_path.clone(),
                });
            }
            key_path.pop_key();
        }

        // reverse key comparaison
        for key in self.keys.keys() {
            if !keys.0.contains_key(key) {
                key_path.push_key(key.clone());
                warnings.emit_warning(Warning::SurplusKey {
                    locale: top_locale.clone(),
                    key_path: key_path.clone(),
                });
                key_path.pop_key();
            }
        }

        Ok(())
    }

    pub fn make_builder_keys(
        &mut self,
        key_path: &mut KeyPath,
        strings: &mut StringIndexer,
    ) -> Result<BuildersKeysInner> {
        let mut keys = BuildersKeysInner::default();
        for (key, value) in &mut self.keys {
            value.reduce();
            key_path.push_key(key.clone());
            let locale_value = value.make_locale_value(key_path, strings)?;
            let key = key_path
                .pop_key()
                .expect("Unexpected empty KeyPath in make_builder_keys. If you got this error please open an issue on github.");
            keys.0.insert(key, locale_value);
        }
        Ok(keys)
    }
}

impl<'de, 'a> serde::de::DeserializeSeed<'de> for LocaleSeed<'a> {
    type Value = Locale;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keys = deserializer.deserialize_map(self.clone())?;
        let Self {
            name,
            top_locale_name,
            ..
        } = self;
        Ok(Locale {
            name,
            keys,
            top_locale_name,
            strings: vec![],
            top_locale_string_count: 0,
        })
    }
}

impl<'de, 'a> serde::de::Visitor<'de> for LocaleSeed<'a> {
    type Value = BTreeMap<Key, ParsedValue>;

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeMap::new();

        while let Some(locale_key) = map.next_key::<Key>()? {
            self.key_path.push_key(locale_key.clone());
            let value = map.next_value_seed(ParsedValueSeed {
                top_locale_name: &self.top_locale_name,
                key: &locale_key,
                key_path: &self.key_path,
                in_range: false,
                foreign_keys_paths: self.foreign_keys_paths,
            })?;
            self.key_path.pop_key();
            keys.insert(locale_key, value);
        }

        Ok(keys)
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a map of string keys and value either string or map"
        )
    }
}

fn track_file(
    tracked_files: &mut Vec<String>,
    locale: &Key,
    namespace: Option<&Key>,
    path: &Path,
    warnings: &Warnings,
) {
    if let Some(path) = path.as_os_str().to_str().map(ToOwned::to_owned) {
        tracked_files.push(path);
    } else {
        warnings.emit_warning(Warning::NonUnicodePath {
            locale: locale.clone(),
            namespace: namespace.cloned(),
            path: path.to_owned(),
        });
    }
}
