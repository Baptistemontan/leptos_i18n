use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    fmt::Display,
};

use icu_plurals::{
    PluralCategory, PluralRuleType as IcuRuleType, PluralRules,
    PluralRulesOptions as IcuPluralRulesOptions,
};

use super::Value;
use crate::{
    error::{Diagnostics, Error, Result, Warning},
    parser::{
        locale::{
            NoSubkey, RawLocale, RawLocalesOrNamespaces, RawNamespace, RawValueOrSubkeys, RawValues,
        },
        raw_value::RawValue,
    },
    utils::{Key, KeyPath, Loc, Location},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Plurals<V = Value> {
    pub rule_type: PluralRuleType,
    pub count_key: Key,
    pub forms: PluralForms<V>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluralForms<V = Value> {
    other: Box<V>,
    zero: Option<Box<V>>,
    one: Option<Box<V>>,
    two: Option<Box<V>>,
    few: Option<Box<V>>,
    many: Option<Box<V>>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PluralRuleType {
    Cardinal,
    Ordinal,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluralForm {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl<V> Plurals<V> {
    pub fn get_plural_rules(rule_type: PluralRuleType, locale: &Key) -> Result<PluralRules> {
        let locale =
            locale
                .name
                .parse::<icu_locale::Locale>()
                .map_err(|err| Error::InvalidLocale {
                    locale: locale.name.clone(),
                    err,
                })?;
        let plural_rules = PluralRules::try_new(locale.into(), rule_type.into())
            .map_err(Error::PluralRulesError)?;

        Ok(plural_rules)
    }

    pub fn check_forms(&self, loc: Loc, diag: &Diagnostics) -> Result<()> {
        let plural_rules = Self::get_plural_rules(self.rule_type, loc.locale)?;
        let forms = self
            .forms
            .iter_forms()
            .map(|(f, _)| f)
            .collect::<BTreeSet<_>>();
        let used_forms = plural_rules
            .categories()
            .map(PluralForm::from_icu_category)
            .collect::<BTreeSet<_>>();
        for form in forms.difference(&used_forms).copied() {
            diag.emit_warning(Warning::UnusedForm {
                loc: loc.into(),
                form,
                rule_type: self.rule_type,
            });
        }
        Ok(())
    }
}

impl PluralForm {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "zero" => Some(PluralForm::Zero),
            "one" => Some(PluralForm::One),
            "two" => Some(PluralForm::Two),
            "few" => Some(PluralForm::Few),
            "many" => Some(PluralForm::Many),
            "other" => Some(PluralForm::Other),
            _ => None,
        }
    }

    pub fn from_icu_category(cat: PluralCategory) -> Self {
        match cat {
            PluralCategory::Zero => PluralForm::Zero,
            PluralCategory::One => PluralForm::One,
            PluralCategory::Two => PluralForm::Two,
            PluralCategory::Few => PluralForm::Few,
            PluralCategory::Many => PluralForm::Many,
            PluralCategory::Other => PluralForm::Other,
        }
    }
}

impl From<PluralCategory> for PluralForm {
    fn from(value: PluralCategory) -> Self {
        Self::from_icu_category(value)
    }
}

impl From<PluralRuleType> for IcuRuleType {
    fn from(value: PluralRuleType) -> Self {
        match value {
            PluralRuleType::Cardinal => IcuRuleType::Cardinal,
            PluralRuleType::Ordinal => IcuRuleType::Ordinal,
        }
    }
}

impl From<PluralRuleType> for IcuPluralRulesOptions {
    fn from(value: PluralRuleType) -> Self {
        Self::default().with_type(value.into())
    }
}

impl Display for PluralRuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluralRuleType::Cardinal => write!(f, "cardinal"),
            PluralRuleType::Ordinal => write!(f, "ordinal"),
        }
    }
}

impl Display for PluralForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluralForm::Zero => write!(f, "_zero"),
            PluralForm::One => write!(f, "_one"),
            PluralForm::Two => write!(f, "_two"),
            PluralForm::Few => write!(f, "_few"),
            PluralForm::Many => write!(f, "_many"),
            PluralForm::Other => write!(f, "_other"),
        }
    }
}

fn replace_box_option<V>(dest: &mut Option<Box<V>>, src: V) -> Option<V> {
    let d = dest.as_deref_mut();
    match d {
        Some(dest) => Some(core::mem::replace(dest, src)),
        None => {
            *dest = Some(Box::new(src));
            None
        }
    }
}

impl<V> PluralForms<V> {
    pub fn new_from_other(other: V) -> Self {
        Self {
            other: Box::new(other),
            zero: None,
            one: None,
            two: None,
            few: None,
            many: None,
        }
    }

    pub fn into_form_iter(self) -> impl Iterator<Item = (PluralForm, V)> {
        [
            self.zero.map(|f| (PluralForm::Zero, *f)),
            self.one.map(|f| (PluralForm::One, *f)),
            self.two.map(|f| (PluralForm::Two, *f)),
            self.few.map(|f| (PluralForm::Few, *f)),
            self.many.map(|f| (PluralForm::Many, *f)),
        ]
        .into_iter()
        .flatten()
        .chain(core::iter::once((PluralForm::Other, *self.other)))
    }

    pub fn iter_forms(&self) -> impl Iterator<Item = (PluralForm, &V)> {
        [
            self.zero.as_deref().map(|f| (PluralForm::Zero, f)),
            self.one.as_deref().map(|f| (PluralForm::One, f)),
            self.two.as_deref().map(|f| (PluralForm::Two, f)),
            self.few.as_deref().map(|f| (PluralForm::Few, f)),
            self.many.as_deref().map(|f| (PluralForm::Many, f)),
        ]
        .into_iter()
        .flatten()
        .chain(core::iter::once((PluralForm::Other, &*self.other)))
    }

    pub fn get_form(&self, form: PluralForm) -> Option<&V> {
        match form {
            PluralForm::Zero => self.zero.as_deref(),
            PluralForm::One => self.one.as_deref(),
            PluralForm::Two => self.two.as_deref(),
            PluralForm::Few => self.few.as_deref(),
            PluralForm::Many => self.many.as_deref(),
            PluralForm::Other => Some(&self.other),
        }
    }

    pub fn get_form_or_other(&self, form: PluralForm) -> &V {
        self.get_form(form).unwrap_or(&self.other)
    }

    pub fn insert_form(&mut self, form: PluralForm, value: V) -> Option<V> {
        match form {
            PluralForm::Zero => replace_box_option(&mut self.zero, value),
            PluralForm::One => replace_box_option(&mut self.one, value),
            PluralForm::Two => replace_box_option(&mut self.two, value),
            PluralForm::Few => replace_box_option(&mut self.few, value),
            PluralForm::Many => replace_box_option(&mut self.many, value),
            PluralForm::Other => Some(core::mem::replace(&mut self.other, value)),
        }
    }

    pub fn try_from_iter(iter: impl IntoIterator<Item = (PluralForm, V)>) -> Option<Self> {
        let mut other = None;
        let mut zero = None;
        let mut one = None;
        let mut two = None;
        let mut few = None;
        let mut many = None;

        for (form, value) in iter {
            let _ = match form {
                PluralForm::Zero => replace_box_option(&mut zero, value),
                PluralForm::One => replace_box_option(&mut one, value),
                PluralForm::Two => replace_box_option(&mut two, value),
                PluralForm::Few => replace_box_option(&mut few, value),
                PluralForm::Many => replace_box_option(&mut many, value),
                PluralForm::Other => replace_box_option(&mut other, value),
            };
        }

        Some(Self {
            other: other?,
            zero,
            one,
            two,
            few,
            many,
        })
    }

    pub fn map<T, F>(self, mut f: F) -> PluralForms<T>
    where
        F: FnMut(V) -> T,
    {
        let f = move |v: V| Ok::<T, Infallible>(f(v));
        let Ok(this) = Self::try_map(self, f);
        this
    }

    pub fn try_map<T, E, F>(self, mut f: F) -> Result<PluralForms<T>, E>
    where
        F: FnMut(V) -> Result<T, E>,
    {
        fn map<V, T, E, F>(v: Option<Box<V>>, f: &mut F) -> Result<Option<Box<T>>, E>
        where
            F: FnMut(V) -> Result<T, E>,
        {
            match v {
                None => Ok(None),
                Some(v) => Some(f(*v).map(Box::new)).transpose(),
            }
        }
        Ok(PluralForms {
            other: Box::new(f(*self.other)?),
            zero: map(self.zero, &mut f)?,
            one: map(self.one, &mut f)?,
            two: map(self.two, &mut f)?,
            few: map(self.few, &mut f)?,
            many: map(self.many, &mut f)?,
        })
    }

    pub fn map_ref<T, F>(&self, mut f: F) -> PluralForms<T>
    where
        F: FnMut(&V) -> T,
    {
        let f = move |v: &V| Ok::<T, Infallible>(f(v));
        let Ok(this) = Self::try_map_ref(self, f);
        this
    }

    pub fn try_map_ref<T, E, F>(&self, mut f: F) -> Result<PluralForms<T>, E>
    where
        F: FnMut(&V) -> Result<T, E>,
    {
        fn map<V, T, E, F>(v: &Option<Box<V>>, f: &mut F) -> Result<Option<Box<T>>, E>
        where
            F: FnMut(&V) -> Result<T, E>,
        {
            match v.as_deref() {
                None => Ok(None),
                Some(v) => Some(f(v).map(Box::new)).transpose(),
            }
        }
        Ok(PluralForms {
            other: Box::new(f(&self.other)?),
            zero: map(&self.zero, &mut f)?,
            one: map(&self.one, &mut f)?,
            two: map(&self.two, &mut f)?,
            few: map(&self.few, &mut f)?,
            many: map(&self.many, &mut f)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergedPlurals {
    RawValue(RawValue),
    Plurals(Plurals<RawValue>),
}

pub fn merge_plurals(
    values: RawLocalesOrNamespaces,
    diag: &Diagnostics,
) -> RawLocalesOrNamespaces<MergedPlurals> {
    match values {
        RawLocalesOrNamespaces::Locales(locales) => {
            RawLocalesOrNamespaces::Locales(merge_locales_plurals(locales, diag, None))
        }
        RawLocalesOrNamespaces::Namespaces(namespaces) => {
            RawLocalesOrNamespaces::Namespaces(merge_namespaces_plurals(namespaces, diag))
        }
    }
}

fn merge_namespaces_plurals(
    namespaces: Vec<RawNamespace>,
    diag: &Diagnostics,
) -> Vec<RawNamespace<MergedPlurals>> {
    namespaces
        .into_iter()
        .map(|ns| merge_namespace_plurals(ns, diag))
        .collect()
}

fn merge_namespace_plurals(
    namespace: RawNamespace,
    diag: &Diagnostics,
) -> RawNamespace<MergedPlurals> {
    let locales = merge_locales_plurals(namespace.locales, diag, Some(namespace.name.clone()));
    RawNamespace {
        name: namespace.name,
        locales,
    }
}

fn merge_locales_plurals(
    locales: Vec<RawLocale>,
    diag: &Diagnostics,
    ns: Option<Key>,
) -> Vec<RawLocale<MergedPlurals>> {
    locales
        .into_iter()
        .map(|loc| merge_locale_plurals(loc, diag, ns.clone()))
        .collect()
}

fn merge_locale_plurals(
    locale: RawLocale,
    diag: &Diagnostics,
    ns: Option<Key>,
) -> RawLocale<MergedPlurals> {
    let key_path = KeyPath::new(ns);
    let mut loc = Location::new(locale.name.clone(), key_path);
    RawLocale {
        name: locale.name,
        values: merge_values_plurals(locale.values, diag, &mut loc),
    }
}

type PossiblePlurals = BTreeMap<String, BTreeMap<PluralForm, (Key, PluralRuleType, RawValue)>>;

fn merge_values_plurals(
    values: RawValues,
    diag: &Diagnostics,
    loc: &mut Location,
) -> RawValues<MergedPlurals> {
    let mut merged_values = BTreeMap::new();
    let mut possible_plurals: PossiblePlurals = BTreeMap::new();
    for (key, value) in values.values {
        let value = match value {
            RawValueOrSubkeys::Subkeys(subkeys) => {
                let mut loc = loc.push_key(key.clone());
                let merged_keys = merge_values_plurals(subkeys, diag, &mut loc);
                // TODO: check key conflict on insert
                merged_values.insert(key, RawValueOrSubkeys::Subkeys(merged_keys));
                continue;
            }
            RawValueOrSubkeys::Value(value) => RawValueOrSubkeys::<_, NoSubkey>::Value(value),
            RawValueOrSubkeys::Defaulted => RawValueOrSubkeys::<_, NoSubkey>::Defaulted,
            RawValueOrSubkeys::Dummy(dummy) => RawValueOrSubkeys::<_, NoSubkey>::Dummy(dummy),
        };
        if let Err(err) = merge_value(value, key, &mut possible_plurals, &mut merged_values) {
            todo!("err: {err}")
        }
    }

    let mut disabled_error_emitted = false;

    for (base_key, mut plurals) in possible_plurals {
        if plurals.len() == 1 {
            for (_, (key, _, value)) in plurals {
                // TODO: check key conflict on insert
                merged_values.insert(
                    key,
                    RawValueOrSubkeys::Value(MergedPlurals::RawValue(value)),
                );
            }
            continue;
        }
        let Some((_, rule_type, other)) = plurals.remove(&PluralForm::Other) else {
            for (_, (key, _, value)) in plurals {
                // TODO: check key conflict on insert
                merged_values.insert(
                    key,
                    RawValueOrSubkeys::Value(MergedPlurals::RawValue(value)),
                );
            }
            continue;
        };
        let key = match Key::try_new_at(&base_key, (&*loc).into()) {
            Ok(key) => key,
            Err(err) => {
                diag.emit_error(err);
                continue;
            }
        };
        let pushed_loc = loc.push_key(key.clone());

        if !cfg!(feature = "plurals") && !core::mem::replace(&mut disabled_error_emitted, true) {
            diag.emit_error(Error::DisabledPlurals {
                loc: pushed_loc.clone(),
            });
        }

        let mut forms = PluralForms::new_from_other(other);
        let mut conflict_error_emitted = false;

        for (form, (_, rule, value)) in plurals {
            if rule == rule_type {
                forms.insert_form(form, value);
            } else if !core::mem::replace(&mut conflict_error_emitted, true) {
                diag.emit_error(Error::ConflictingPluralRuleType {
                    loc: pushed_loc.clone(),
                });
            }
        }
        let count_key = Key::count();

        if conflict_error_emitted {
            merged_values.insert(
                key,
                RawValueOrSubkeys::Value(MergedPlurals::Plurals(Plurals {
                    rule_type,
                    count_key,
                    forms,
                })),
            );
            continue;
        }

        let plural = Plurals {
            rule_type,
            forms,
            count_key,
        };

        let _ = plural.check_forms((&*pushed_loc).into(), diag);
        let value = RawValueOrSubkeys::Value(MergedPlurals::Plurals(plural));
        if merged_values.insert(key, value).is_some() {
            diag.emit_error(Error::PluralsAtNormalKey {
                loc: pushed_loc.clone(),
            });
        }
    }

    RawValues {
        values: merged_values,
    }
}

fn map_value(value: RawValueOrSubkeys<RawValue, NoSubkey>) -> RawValueOrSubkeys<MergedPlurals> {
    match value {
        RawValueOrSubkeys::Defaulted => RawValueOrSubkeys::Defaulted,
        RawValueOrSubkeys::Dummy(dummy) => RawValueOrSubkeys::Dummy(dummy),
        RawValueOrSubkeys::Value(value) => RawValueOrSubkeys::Value(MergedPlurals::RawValue(value)),
    }
}

fn check_plural_value(value: RawValueOrSubkeys<RawValue, NoSubkey>) -> Result<RawValue> {
    match value {
        RawValueOrSubkeys::Defaulted => todo!(),
        RawValueOrSubkeys::Dummy(_) => todo!(),
        RawValueOrSubkeys::Value(v) => Ok(v),
    }
}

fn merge_value(
    value: RawValueOrSubkeys<RawValue, NoSubkey>,
    key: Key,
    possible_plurals: &mut PossiblePlurals,
    merged_values: &mut BTreeMap<Key, RawValueOrSubkeys<MergedPlurals>>,
) -> Result<()> {
    if let Some((base_key, rule_type, plural_form)) = is_possible_plural(&key) {
        let map = possible_plurals.entry(base_key.to_owned()).or_default();
        let value = check_plural_value(value)?;
        map.insert(plural_form, (key, rule_type, value));
    } else {
        let value = map_value(value);
        // TODO: check key conflict on insert
        merged_values.insert(key, value);
    }
    Ok(())
}

fn is_possible_plural(key: &Key) -> Option<(&str, PluralRuleType, PluralForm)> {
    let (base_key, suffix) = key.name.rsplit_once('_')?;
    let (base_key, rule_type) = match base_key.strip_suffix("_ordinal") {
        Some(base_key) => (base_key, PluralRuleType::Ordinal),
        None => (base_key, PluralRuleType::Cardinal),
    };

    PluralForm::try_from_str(suffix).map(|form| (base_key, rule_type, form))
}
