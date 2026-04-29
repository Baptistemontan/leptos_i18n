use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

use crate::{
    error::{Diagnostics, Result},
    extractor::values::{
        Value, Values,
        foreign_key::{ResolvedLocale, ResolvedLocalesOrNamespaces, ResolvedNamespace},
    },
    parser::options::LocaleName,
    utils::{KeyPath, Location},
};
use crate::{
    extractor::values::plurals,
    parser::{locale::RawLocalesOrNamespaces, options::Config},
};
use crate::{formatters::VarBound, utils::Key};

pub mod defaults;
pub mod values;

use values::Keys;
pub const VAR_COUNT_KEY: &str = "var_count";

#[derive(Debug)]
pub struct ParsedLocales {
    pub values: LocalesOrNamespaces,
    pub builders: Builders,
    pub cfg: Config,
    pub diag: Diagnostics,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalesOrNamespaces {
    Namespaces(Vec<Namespace>),
    Locales(Locales),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locales {
    pub locales: Vec<Locale>,
    pub keys: Keys,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub name: LocaleName,
    pub strings: Vec<Rc<str>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Namespace {
    pub name: Key,
    pub locales: Locales,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Builder {
    pub name: Key,
    pub keys: InterpolationKeys,
    pub used_by: BTreeSet<KeyPath>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Builders {
    pub builders: BTreeMap<BuilderId, Builder>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuilderId(usize);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompInfos {
    pub self_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct InterpolationKeys {
    pub components: BTreeMap<Key, CompInfos>,
    pub vars: BTreeMap<Key, VarInfos>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInfos {
    pub bounds: BTreeSet<VarBound>,
    pub plural: bool,
}

#[derive(Default)]
pub struct StringIndexer {
    current: HashMap<Rc<str>, usize>,
    acc: Vec<Rc<str>>,
}

pub fn extract_locales(
    values: RawLocalesOrNamespaces,
    cfg: Config,
    diag: Option<Diagnostics>,
) -> Result<ParsedLocales> {
    let diag = diag.unwrap_or_default();
    let merged_plurals = plurals::merge_plurals(values, &diag);
    let resolved_fk = values::foreign_key::resolve_foreign_key(merged_plurals, &diag);
    let mut merged = merge_values(resolved_fk, &cfg, &diag);

    let builders = get_builders(&mut merged);

    Ok(ParsedLocales {
        values: merged,
        builders,
        cfg,
        diag,
    })
}

fn merge_values(
    values: ResolvedLocalesOrNamespaces,
    cfg: &Config,
    diag: &Diagnostics,
) -> LocalesOrNamespaces {
    match values {
        ResolvedLocalesOrNamespaces::Locales(locales) => {
            LocalesOrNamespaces::Locales(merge_locales(locales, cfg, diag, None))
        }
        ResolvedLocalesOrNamespaces::Namespaces(namespaces) => {
            LocalesOrNamespaces::Namespaces(merge_namespaces(namespaces, cfg, diag))
        }
    }
}

fn merge_namespaces(
    namespaces: Vec<ResolvedNamespace>,
    cfg: &Config,
    diag: &Diagnostics,
) -> Vec<Namespace> {
    namespaces
        .into_iter()
        .map(|ns| merge_namespace(ns, cfg, diag))
        .collect()
}

fn merge_namespace(namespace: ResolvedNamespace, cfg: &Config, diag: &Diagnostics) -> Namespace {
    let locales = merge_locales(namespace.locales, cfg, diag, Some(namespace.name.clone()));
    Namespace {
        name: namespace.name,
        locales,
    }
}

fn merge_locales(
    mut locales: Vec<ResolvedLocale>,
    cfg: &Config,
    diag: &Diagnostics,
    ns: Option<Key>,
) -> Locales {
    let default_idx = locales
        .iter()
        .position(|l| l.name.key == cfg.default_locale)
        .expect("default locale not present");
    locales.swap(default_idx, 0);

    let default_locale = cfg.default_locale.clone();

    let mut merged_locales = Locales {
        locales: Vec::with_capacity(locales.len()),
        keys: Keys::default(),
    };

    for locale in locales {
        let mut loc = Location {
            key_path: KeyPath::new(ns.clone()),
            locale: locale.name.clone(),
        };
        let mut str_indexer = StringIndexer::default();
        values::merge_and_index_keys(
            locale,
            &default_locale,
            &mut merged_locales.keys,
            &mut loc,
            cfg,
            &mut str_indexer,
            diag,
        );

        let strings = str_indexer.get_strings();

        merged_locales.locales.push(Locale {
            name: loc.locale,
            strings,
        });
    }

    merged_locales
}

#[derive(Default)]
struct BuilderIndexer {
    ids: BTreeMap<InterpolationKeys, (usize, BTreeSet<KeyPath>)>,
    reverse_keys: Vec<Key>,
    keys: BTreeSet<String>,
}

fn get_builders(locales: &mut LocalesOrNamespaces) -> Builders {
    let mut ids = BuilderIndexer::default();

    match locales {
        LocalesOrNamespaces::Namespaces(namespaces) => {
            for ns in namespaces {
                let mut path = KeyPath::new(Some(ns.name.clone()));
                get_keys_builders(&mut ns.locales.keys, &mut ids, &mut path);
            }
        }
        LocalesOrNamespaces::Locales(locales) => {
            let mut path = KeyPath::new(None);
            get_keys_builders(&mut locales.keys, &mut ids, &mut path);
        }
    }

    ids.into_builders()
}

fn get_keys_builders(keys: &mut Keys, ids: &mut BuilderIndexer, path: &mut KeyPath) {
    for (key, value) in keys.values.iter_mut() {
        let mut path = path.push_key(key.clone());
        match value {
            values::ValuesOrSubkeys::Values(values) => {
                get_values_builders(values, ids, path.clone())
            }
            values::ValuesOrSubkeys::Subkeys(keys) => get_keys_builders(keys, ids, &mut path),
        }
    }
}

fn get_values_builders(values: &mut Values, ids: &mut BuilderIndexer, path: KeyPath) {
    let interpolation_keys = make_keys(values);
    let id = ids.push_keys(interpolation_keys, path);
    values.builder_id = id;
}

fn make_keys(values: &Values) -> InterpolationKeys {
    let mut interpolation_keys = InterpolationKeys::default();

    for value in values.values.values() {
        extract_value_keys(value, &mut interpolation_keys);
    }

    interpolation_keys
}

fn extract_value_keys(value: &Value, keys: &mut InterpolationKeys) {
    match value {
        Value::Literal(_) => {}
        Value::Variable(variable) => {
            let info = keys.vars.entry(variable.key.clone()).or_default();
            info.bounds.insert(variable.bound.clone());
        }
        Value::Component(component) => {
            if let Some(inner) = &component.inner {
                extract_value_keys(inner, keys);
            }
            let is_self_closed = component.inner.is_none();
            let info = keys
                .components
                .entry(component.key.clone())
                .or_insert(CompInfos {
                    self_closed: is_self_closed,
                });
            if info.self_closed != is_self_closed {
                todo!("can't have self closed and normal component sharing the same key")
            }
        }
        Value::Bloc(values) => {
            for value in values {
                extract_value_keys(value, keys);
            }
        }
        Value::Plurals(plurals) => {
            for (_, value) in plurals.forms.iter_forms() {
                extract_value_keys(value, keys);
            }
            let var_info = keys.vars.entry(plurals.count_key.clone()).or_default();
            var_info.plural = true;
        }
    }
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

impl Default for BuilderId {
    fn default() -> Self {
        BuilderId(usize::MAX)
    }
}

impl BuilderIndexer {
    pub fn push_keys(&mut self, keys: InterpolationKeys, path: KeyPath) -> BuilderId {
        if let Some((id, paths)) = self.ids.get_mut(&keys) {
            paths.insert(path);
            return BuilderId(*id);
        }

        let mut name = keys.generate_builder_key();
        loop {
            if self.keys.contains(&name) {
                name.push('_');
            } else {
                let key = Key::new(&name).expect("the builder name should be a valid key");
                let id = self.reverse_keys.len();
                self.reverse_keys.push(key);
                self.ids.insert(keys, (id, BTreeSet::from([path])));
                self.keys.insert(name);
                break BuilderId(id);
            }
        }
    }

    pub fn into_builders(self) -> Builders {
        let builders = self
            .ids
            .into_iter()
            .map(|(keys, (id, used_by))| {
                let name = self.reverse_keys[id].clone();
                let id = BuilderId(id);
                let builder = Builder {
                    name,
                    keys,
                    used_by,
                };
                (id, builder)
            })
            .collect();
        Builders { builders }
    }
}

impl InterpolationKeys {
    pub fn generate_builder_key(&self) -> String {
        let mut s = String::from("i18n_builder_");
        for (key, info) in &self.vars {
            s.push_str(&key.name);
            if info.plural {
                s.push_str("pl");
            }
            for bound in info.bounds.iter() {
                match bound {
                    VarBound::Dummy => {
                        s.push_str("dy");
                    }
                    VarBound::None => {}
                    VarBound::AttributeValue => {
                        s.push_str("ar");
                    }
                    VarBound::Formatted {
                        formatter_name,
                        to_tokens: _,
                    } => {
                        Self::write_formatter_name(&mut s, formatter_name);
                    }
                }
            }
            s.push('_');
        }
        for (key, info) in &self.components {
            s.push_str(&key.name);
            if info.self_closed {
                s.push_str("sf");
            }
            s.push('_');
        }

        s
    }

    fn write_formatter_name(s: &mut String, f_name: &str) {
        s.reserve(f_name.len());
        for c in f_name.chars() {
            if c.is_ascii_alphabetic() {
                s.push(c);
            } else {
                s.push('_');
            }
        }
    }
}
