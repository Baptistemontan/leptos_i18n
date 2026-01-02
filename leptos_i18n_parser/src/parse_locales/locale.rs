use serde::de::MapAccess;

use crate::{
    parse_locales::options::{FileFormat, ParseOptions},
    utils::{Key, KeyPath, UnwrapAt, formatter::Formatter},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet, btree_map::Entry},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::{
    ForeignKeysPaths, StringIndexer,
    cfg_file::ConfigFile,
    error::{Diagnostics, Error, Result, Warning},
    parsed_value::{ParsedValue, ParsedValueSeed},
    plurals::{PluralForm, PluralRuleType, Plurals},
    ranges::RangeType,
};
// use super::warning::{Warning, Warnings};

#[derive(Debug)]
#[non_exhaustive]
pub enum SerdeError {
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Toml(toml::de::Error),
    Json5(json5::Error),
    Custom(String),
    Io(std::io::Error),
}

impl std::fmt::Display for SerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerdeError::Json(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Yaml(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Toml(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Json5(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Io(error) => std::fmt::Display::fmt(error, f),
            SerdeError::Custom(err) => std::fmt::Display::fmt(err, f),
        }
    }
}

impl SerdeError {
    pub fn custom<T: ToString>(err: T) -> Self {
        SerdeError::Custom(err.to_string())
    }
}

impl From<std::io::Error> for SerdeError {
    fn from(value: std::io::Error) -> Self {
        SerdeError::Io(value)
    }
}

impl std::error::Error for SerdeError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub top_locale_name: Key,
    pub name: Key,
    pub keys: BTreeMap<Key, ParsedValue>,
    pub strings: Vec<Rc<str>>,
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
    components_self_closed: BTreeSet<Key>,
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
pub struct DefaultedLocales {
    default_locale: Key,
    mapping: BTreeMap<Key, Key>,
}

impl DefaultedLocales {
    pub fn new(default_locale: Key) -> Self {
        DefaultedLocales {
            default_locale,
            mapping: Default::default(),
        }
    }

    pub fn push(&mut self, key: Key, default_to: Key) {
        self.mapping.insert(key, default_to);
    }

    pub fn default_of<'a>(&'a self, key: &'a Key) -> &'a Key {
        let mut visited = HashSet::new();
        self.default_of_inner(key, &mut visited)
    }

    fn default_of_inner<'a>(&'a self, key: &'a Key, visited: &mut HashSet<&'a Key>) -> &'a Key {
        let mut current_key = key;
        while let Some(key) = self.mapping.get(current_key) {
            visited.insert(current_key);
            if visited.contains(key) {
                return &self.default_locale;
            }
            current_key = key;
        }
        current_key
    }

    pub fn compute(&self) -> BTreeMap<Key, BTreeSet<Key>> {
        let mut defaults: BTreeMap<Key, BTreeSet<Key>> = BTreeMap::new();
        let mut visited = HashSet::new();
        for key in self.mapping.keys() {
            visited.clear();
            let default_to = self.default_of_inner(key, &mut visited);
            defaults
                .entry(default_to.clone())
                .or_default()
                .insert(key.clone());
        }
        defaults
    }
}

#[derive(Debug)]
pub enum LocaleValue {
    Value {
        value: InterpolOrLit,
        defaults: DefaultedLocales,
    },
    Subkeys {
        locales: Vec<Locale>,
        keys: BuildersKeysInner,
    },
}

#[derive(Default, Debug)]
pub struct BuildersKeysInner(pub BTreeMap<Key, LocaleValue>);

#[derive(Debug)]
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

#[derive(Clone)]
pub struct LocaleSeed<'a> {
    pub name: Key,
    pub top_locale_name: Key,
    pub key_path: KeyPath,
    pub foreign_keys_paths: &'a ForeignKeysPaths,
    pub diag: &'a Diagnostics,
}

#[derive(Debug, Clone)]
pub enum DefaultTo {
    Explicit(Key),
    Implicit(Key),
}

impl DefaultTo {
    pub fn get_key(&self) -> &Key {
        match self {
            DefaultTo::Explicit(key) | DefaultTo::Implicit(key) => key,
        }
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

    pub fn push_comp_self_closed(&mut self, key: Key) {
        self.components_self_closed.insert(key);
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
                    key_path: key_path.clone(),
                }
                .into())
            }
            (Some(RangeOrPlural::Range(old)), RangeOrPlural::Range(new)) => {
                Err(Error::RangeTypeMissmatch {
                    key_path: key_path.clone(),
                    type1: old,
                    type2: new,
                }
                .into())
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

    pub fn iter_comps_self_closed(&self) -> impl Iterator<Item = Key> + '_ {
        self.components_self_closed.iter().cloned()
    }
}

impl BuildersKeysInner {
    pub fn propagate_string_count(&mut self, top_locales: &[Locale]) {
        for value in self.0.values_mut() {
            if let LocaleValue::Subkeys { locales, keys, .. } = value {
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
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        options: &ParseOptions,
    ) -> Result<Self> {
        let mut locales = Vec::with_capacity(locale_keys.len());
        for locale in locale_keys.iter().cloned() {
            let file_path: &Path = key.name.as_ref().as_ref();
            locales_dir_path.push(&*locale.name);
            locales_dir_path.push(file_path);

            let locale_file = find_file(locales_dir_path, &options.file_format)?;

            let locale = Locale::new(
                locale_file,
                locales_dir_path,
                locale,
                Some(key.clone()),
                foreign_keys_paths,
                diag,
                tracked_files,
                options,
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
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        options: &ParseOptions,
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
                    diag,
                    tracked_files,
                    options,
                )?);
            }
            Ok(LocalesOrNamespaces::NameSpaces(namespaces))
        } else {
            let mut locales = Vec::with_capacity(locale_keys.len());
            for locale in locale_keys.iter().cloned() {
                manifest_dir_path.push(&*locale.name);
                let locale_file = find_file(manifest_dir_path, &options.file_format)?;
                let locale = Locale::new(
                    locale_file,
                    manifest_dir_path,
                    locale,
                    None,
                    foreign_keys_paths,
                    diag,
                    tracked_files,
                    options,
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
        diag: &Diagnostics,
    ) -> Result<()> {
        let mut key_path = KeyPath::new(namespace);

        for locale in locales {
            let top_locale = locale.name.clone();
            locale.merge_plurals(top_locale.clone(), &mut key_path, diag)?;
        }

        Ok(())
    }

    // this step would be more optimized to be done during `check_locales` but plurals merging need to be done before foreign key resolution,
    // which also need to be done before `check_locales`.
    pub fn merge_plurals(&mut self, diag: &Diagnostics) -> Result<()> {
        match self {
            LocalesOrNamespaces::NameSpaces(namespaces) => {
                for namespace in namespaces {
                    Self::merge_plurals_inner(
                        &mut namespace.locales,
                        Some(namespace.key.clone()),
                        diag,
                    )?;
                }
                Ok(())
            }
            LocalesOrNamespaces::Locales(locales) => {
                Self::merge_plurals_inner(&mut *locales, None, diag)
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
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        options: &ParseOptions,
    ) -> Result<Self> {
        track_file(tracked_files, &locale, namespace.as_ref(), path, diag);

        let seed = LocaleSeed {
            name: locale.clone(),
            top_locale_name: locale,
            key_path: KeyPath::new(namespace),
            foreign_keys_paths,
            diag,
        };

        Self::de(locale_file, path, seed, &options.file_format)
    }

    fn de(
        locale_file: File,
        path: &mut PathBuf,
        seed: LocaleSeed,
        file_format: &FileFormat,
    ) -> Result<Self> {
        let reader = BufReader::new(locale_file);
        let locale =
            file_format
                .deserialize(reader, path, seed)
                .map_err(|err| Error::LocaleFileDeser {
                    path: std::mem::take(path),
                    err,
                })?;
        Ok(locale)
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
                    None => unreachable!(
                        "called get_value_at on empty subkeys. If you got this error please open an issue on github."
                    ),
                    Some(subkeys) => subkeys.get_value_at(path),
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
        diag: &Diagnostics,
    ) -> Result<()> {
        let keys = std::mem::take(&mut self.keys);
        #[allow(clippy::type_complexity)]
        let mut possible_plurals: BTreeMap<
            String,
            BTreeMap<PluralForm, (Key, PluralRuleType, ParsedValue)>,
        > = BTreeMap::new();
        for (key, mut value) in keys {
            if let ParsedValue::Subkeys(Some(subkeys)) = &mut value {
                let mut pushed_key = key_path.push_key(key.clone());
                subkeys.merge_plurals(locale.clone(), &mut pushed_key, diag)?;
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
            let key = Key::new(&base_key).unwrap_at("merge_plurals_1");
            let pushed_key = key_path.push_key(key);
            if !cfg!(feature = "plurals") {
                return Err(Error::DisabledPlurals {
                    locale: locale.clone(),
                    key_path: pushed_key.clone(),
                }
                .into());
            }

            let forms = plurals
                .into_iter()
                .map(|(form, (_, rule, value))| {
                    if rule == rule_type {
                        Ok((form, value))
                    } else {
                        Err(Error::ConflictingPluralRuleType {
                            locale: locale.clone(),
                            key_path: pushed_key.clone(),
                        }
                        .into())
                    }
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            let plural = Plurals {
                rule_type,
                forms,
                count_key: Key::count(),
                other: Box::new(other),
            };
            plural.check_forms(&locale, &pushed_key, diag)?;
            let value = ParsedValue::Plurals(plural);
            let key = pushed_key.pop().unwrap_at("merge_plurals_3");
            if self.keys.insert(key.clone(), value).is_some() {
                let pushed_key = key_path.push_key(key);
                return Err(Error::PluralsAtNormalKey {
                    locale,
                    key_path: pushed_key.clone(),
                }
                .into());
            }
        }

        Ok(())
    }

    pub fn merge(
        &mut self,
        keys: &mut BuildersKeysInner,
        top_locale: Key,
        default_to: &DefaultTo,
        key_path: &mut KeyPath,
        strings: &mut StringIndexer,
        diag: &Diagnostics,
        options: &ParseOptions,
    ) -> Result<()> {
        for (key, keys) in &mut keys.0 {
            let mut pushed_key = key_path.push_key(key.clone());
            let entry = self.keys.entry(key.clone());
            let value = match entry {
                Entry::Vacant(entry) => {
                    if matches!(default_to, DefaultTo::Implicit(_)) {
                        diag.emit_warning(Warning::MissingKey {
                            locale: top_locale.clone(),
                            key_path: pushed_key.clone(),
                        });
                    }
                    entry.insert(ParsedValue::Default)
                }
                Entry::Occupied(entry) => entry.into_mut(),
            };
            value.merge(
                keys,
                top_locale.clone(),
                default_to,
                &mut pushed_key,
                strings,
                diag,
                options,
            )?;
        }

        if !options.suppress_key_warnings {
            // reverse key comparaison
            for key in self.keys.keys() {
                if !keys.0.contains_key(key) {
                    let pushed_key = key_path.push_key(key.clone());
                    diag.emit_warning(Warning::SurplusKey {
                        locale: top_locale.clone(),
                        key_path: pushed_key.clone(),
                    });
                }
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
            let mut pushed_key = key_path.push_key(key.clone());
            let locale_value =
                value.make_locale_value(&self.top_locale_name, &mut pushed_key, strings)?;
            let key = pushed_key.pop().unwrap_at("make_builder_keys_1");
            keys.0.insert(key, locale_value);
        }
        Ok(keys)
    }

    pub fn update_top_locale_name(&mut self, top_locale_name: &Key) {
        self.top_locale_name = top_locale_name.clone();
        for value in self.keys.values_mut() {
            value.update_top_locale_name(top_locale_name);
        }
    }

    pub fn clone_with_top_locale_name(&self, top_locale_name: &Key) -> Self {
        let mut this = self.clone();
        this.update_top_locale_name(top_locale_name);
        this
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for LocaleSeed<'_> {
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

impl<'de> serde::de::Visitor<'de> for LocaleSeed<'_> {
    type Value = BTreeMap<Key, ParsedValue>;

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeMap::new();

        while let Some(locale_key) = map.next_key::<Key>()? {
            let pushed_key = self.key_path.push_key(locale_key.clone());
            let value = map.next_value_seed(ParsedValueSeed {
                top_locale_name: &self.top_locale_name,
                key: &locale_key,
                key_path: &pushed_key,
                in_range: false,
                foreign_keys_paths: self.foreign_keys_paths,
                diag: self.diag,
            })?;
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
