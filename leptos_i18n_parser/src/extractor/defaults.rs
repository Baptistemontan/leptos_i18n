use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::{parser::options::Config, utils::Key};

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultedLocales {
    pub default_locale: Key,
    pub mapping: BTreeMap<Key, Key>,
}

impl DefaultedLocales {
    pub fn new(default_locale: Key) -> Self {
        DefaultedLocales {
            default_locale,
            mapping: Default::default(),
        }
    }

    pub fn push(&mut self, key: Key, cfg: &Config) {
        let default_to = cfg
            .extensions
            .get(&key)
            .unwrap_or(&self.default_locale)
            .clone();
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
