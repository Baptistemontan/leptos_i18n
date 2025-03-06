use crate::utils::key::{Key, KeyPath};
use std::{cell::RefCell, fmt::Display};

use super::plurals::{PluralForm, PluralRuleType};

#[derive(Debug)]
pub enum Warning {
    MissingKey {
        locale: Key,
        key_path: KeyPath,
    },
    SurplusKey {
        locale: Key,
        key_path: KeyPath,
    },
    UnusedForm {
        locale: Key,
        key_path: KeyPath,
        form: PluralForm,
        rule_type: PluralRuleType,
    },
    NonUnicodePath {
        locale: Key,
        namespace: Option<Key>,
        path: std::path::PathBuf,
    },
}

#[derive(Default)]
pub struct Warnings(RefCell<Vec<Warning>>);

impl Warnings {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn emit_warning(&self, warning: Warning) {
        self.0.borrow_mut().push(warning);
    }

    pub fn into_inner(self) -> Vec<Warning> {
        self.0.into_inner()
    }
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::MissingKey { locale, key_path } => {
                write!(f, "Missing key \"{}\" in locale {:?}", key_path, locale)
            }
            Warning::SurplusKey { locale, key_path } => write!(
                f,
                "Key \"{}\" is present in locale {:?} but not in default locale, it is ignored",
                key_path, locale
            ),
            Warning::UnusedForm { locale, key_path, form, rule_type } => {
                write!(f, "At key \"{}\", locale {:?} does not use {} plural form \"{}\", it is still kept but is useless.", key_path, locale, rule_type, form)
            },
            Warning::NonUnicodePath { locale, namespace: None, path } => write!(f, "File path for locale {:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {:?}", locale, path),
            Warning::NonUnicodePath { locale, namespace: Some(ns), path } => write!(f, "File path for locale {:?} in namespace {:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {:?}", locale, ns, path),
        }
    }
}
