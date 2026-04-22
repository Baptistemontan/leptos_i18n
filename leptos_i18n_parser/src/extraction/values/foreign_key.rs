use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    str::FromStr,
};

use fixed_decimal::{FloatPrecision, LimitError};
use icu_plurals::PluralOperands;

use crate::{
    error::Diagnostics,
    extraction::values::plurals::{
        MergedPlurals, PluralForm, PluralForms, PluralRuleType, Plurals,
    },
    parser::{
        dummy::Dummy,
        locale::{RawLocale, RawLocalesOrNamespaces, RawNamespace, RawValueOrSubkeys, RawValues},
        raw_value::{
            RawLiteral, RawValue,
            component::{Component, RawAttribute, RawAttributeValue, RawAttributes},
            variable::Variable,
        },
    },
    utils::{Key, KeyPath, Loc, Location},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedLocalesOrNamespaces {
    Locales(Vec<ResolvedLocale>),
    Namespaces(Vec<ResolvedNamespace>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedNamespace {
    pub name: Key,
    pub locales: Vec<ResolvedLocale>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLocale {
    pub name: Key,
    pub values: ResolvedValues,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedValue {
    Literal(RawLiteral),
    Variable(Variable),
    Component(Component<Self>),
    Bloc(Vec<Self>),
    Plurals(Plurals<Self>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedValueOrSubkeys<S = ResolvedValues> {
    Value(ResolvedValue),
    Subkeys(S),
    Defaulted,
    Dummy(Dummy),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedValues {
    pub values: BTreeMap<Key, ResolvedValueOrSubkeys>,
}

pub fn resolve_foreign_key(
    values: RawLocalesOrNamespaces<MergedPlurals>,
    diag: &Diagnostics,
) -> ResolvedLocalesOrNamespaces {
    let mut fks = BTreeMap::<Location, MergedPlurals>::new();
    let mut resolved = remove_fks(values, &mut fks);
    let mut set_fks = BTreeMap::<Location, ResolvedValue>::new();

    for (path, value) in core::mem::take(&mut fks) {
        // TODO: cycle detection
        let mut waiting_on = BTreeSet::new();
        let maybe_resolved = resolve_fk(value, &resolved, &set_fks, &path, diag, &mut waiting_on);
        match maybe_resolved {
            Ok(value) => {
                set_fks.insert(path, value);
            }
            Err(value) => {
                fks.insert(path, value);
            }
        }
    }

    for (loc, value) in set_fks {
        let loc = Loc {
            key_path: &loc.key_path,
            locale: &loc.locale,
        };
        resolved.insert_at(loc, ResolvedValueOrSubkeys::Value(value));
    }

    resolved
}

fn resolve_fk(
    value: MergedPlurals,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    loc: &Location,
    diag: &Diagnostics,
    waiting_on: &mut BTreeSet<Location>,
) -> Result<ResolvedValue, MergedPlurals> {
    if !has_deps(&value, resolved, set_fks, &loc.locale, waiting_on) {
        return Err(value);
    }

    let value = match value {
        MergedPlurals::RawValue(raw_value) => {
            resolve_raw_value_fk(raw_value, resolved, set_fks, loc, diag)
        }
        MergedPlurals::Plurals(plurals) => {
            let forms = plurals
                .forms
                .map(|value| resolve_raw_value_fk(value, resolved, set_fks, loc, diag));

            ResolvedValue::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: plurals.count_key,
                forms,
            })
        }
    };

    Ok(value)
}

fn resolve_raw_value_fk(
    value: RawValue,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    loc: &Location,
    diag: &Diagnostics,
) -> ResolvedValue {
    match value {
        RawValue::ForeignKey(foreign_key) => {
            let path = &foreign_key.target_key_path;
            let taget_loc = Loc {
                key_path: path,
                locale: &loc.locale,
            };
            let target_value = match resolved.get_value_at(taget_loc) {
                Some(ResolvedValueOrSubkeys::Value(target_value)) => target_value,
                None => {
                    let loc: Location = taget_loc.into();
                    set_fks
                        .get(&loc)
                        .expect("should have already been checked by has_deps")
                }
                Some(ResolvedValueOrSubkeys::Defaulted)
                | Some(ResolvedValueOrSubkeys::Dummy(_))
                | Some(ResolvedValueOrSubkeys::Subkeys(_)) => {
                    unreachable!("should have already been checked by has_deps")
                }
            };

            populate_value(
                target_value,
                &foreign_key.args,
                resolved,
                set_fks,
                loc,
                diag,
            )
        }
        RawValue::Literal(raw_literal) => ResolvedValue::Literal(raw_literal),
        RawValue::Variable(variable) => ResolvedValue::Variable(variable),
        RawValue::Component(component) => {
            let inner = component
                .inner
                .map(|inner| resolve_raw_value_fk(*inner, resolved, set_fks, loc, diag))
                .map(Box::new);
            ResolvedValue::Component(Component {
                inner,
                key: component.key,
                attributes: component.attributes,
            })
        }
        RawValue::Bloc(raw_values) => ResolvedValue::Bloc(
            raw_values
                .into_iter()
                .map(|v| resolve_raw_value_fk(v, resolved, set_fks, loc, diag))
                .collect(),
        ),
    }
}

fn populate_value(
    value: &ResolvedValue,
    args: &BTreeMap<String, RawValue>,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    loc: &Location,
    diag: &Diagnostics,
) -> ResolvedValue {
    match value {
        ResolvedValue::Literal(raw_literal) => ResolvedValue::Literal(raw_literal.clone()),
        ResolvedValue::Variable(variable) => {
            populate_variable(variable, args, resolved, set_fks, loc, diag)
        }
        ResolvedValue::Component(component) => {
            let inner = component
                .inner
                .as_deref()
                .map(|inner| populate_value(inner, args, resolved, set_fks, loc, diag))
                .map(Box::new);
            let attributes =
                populate_attributes(&component.attributes, args, resolved, set_fks, loc, diag);
            ResolvedValue::Component(Component {
                key: component.key.clone(),
                inner,
                attributes,
            })
        }
        ResolvedValue::Bloc(resolved_values) => {
            let bloc = resolved_values
                .iter()
                .map(|value| populate_value(value, args, resolved, set_fks, loc, diag))
                .collect();
            ResolvedValue::Bloc(bloc)
        }
        ResolvedValue::Plurals(plurals) => {
            populate_plurals(plurals, args, resolved, set_fks, loc, diag)
        }
    }
}

fn populate_variable(
    variable: &Variable,
    args: &BTreeMap<String, RawValue>,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    loc: &Location,
    diag: &Diagnostics,
) -> ResolvedValue {
    let Some(value) = args.get(variable.actual_name()) else {
        return ResolvedValue::Variable(variable.clone());
    };
    resolve_raw_value_fk(value.clone(), resolved, set_fks, loc, diag)
}

fn populate_plurals(
    plurals: &Plurals<ResolvedValue>,
    args: &BTreeMap<String, RawValue>,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    loc: &Location,
    diag: &Diagnostics,
) -> ResolvedValue {
    let mapped_count_key = map_plural_key(&plurals.count_key, plurals.rule_type, args, loc, diag);

    match mapped_count_key {
        Ok(new_count_key) => {
            let forms = plurals
                .forms
                .map_ref(|value| populate_value(value, args, resolved, set_fks, loc, diag));
            ResolvedValue::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: new_count_key,
                forms,
            })
        }
        Err(form) => plurals.forms.get_form_or_other(form).clone(),
    }
}

fn extract_single_value_from_bloc(bloc: &[RawValue]) -> Result<Option<&RawValue>, ()> {
    let mut iter = bloc.iter().peekable();
    while let Some(v) = iter.peek() {
        if v.is_empty() {
            iter.next();
        } else {
            break;
        }
    }

    let value = match iter.next() {
        Some(RawValue::Bloc(bloc)) => extract_single_value_from_bloc(bloc)?,
        Some(value) => Some(value),
        None => None,
    };

    let Some(value) = value else {
        return Ok(None);
    };

    if !iter.all(|v| v.is_empty()) {
        Err(())
    } else {
        Ok(Some(value))
    }
}

fn map_plural_key(
    count_key: &Key,
    rule_type: PluralRuleType,
    args: &BTreeMap<String, RawValue>,
    loc: &Location,
    diag: &Diagnostics,
) -> Result<Key, PluralForm> {
    let var_name = count_key
        .name
        .strip_prefix("var_")
        .expect("the count_key should have started with var_");
    let Some(value) = args.get(var_name) else {
        return Ok(count_key.clone());
    };

    match value {
        RawValue::Literal(lit) => Err(get_form_for_literal(lit, rule_type, &loc.locale, diag)),
        RawValue::Bloc(bloc) => match extract_single_value_from_bloc(bloc) {
            Ok(Some(RawValue::Variable(var))) => Ok(var.key.clone()),
            Ok(None) => todo!(),
            Ok(Some(_)) => todo!(),
            Err(()) => todo!(),
        },
        RawValue::Component(_) => todo!(),
        RawValue::ForeignKey(_) => todo!(),
        RawValue::Variable(var) => Ok(var.key.clone()),
    }
}

fn get_form_for_literal(
    lit: &RawLiteral,
    rule_type: PluralRuleType,
    locale: &Key,
    diag: &Diagnostics,
) -> PluralForm {
    match lit {
        RawLiteral::Signed(value) => get_plural_form_for(rule_type, locale, *value, diag),
        RawLiteral::Unsigned(value) => get_plural_form_for(rule_type, locale, *value, diag),
        RawLiteral::String(value) => {
            get_plural_form_for(rule_type, locale, StrToPluralOperands(value), diag)
        }
        RawLiteral::Float(value) => {
            get_plural_form_for(rule_type, locale, F64ToPluralOperands(*value), diag)
        }
        RawLiteral::Bool(_) => todo!(),
    }
}

#[derive(Debug, Clone, Copy)]
struct F64ToPluralOperands(f64);

impl Display for F64ToPluralOperands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl TryFrom<F64ToPluralOperands> for PluralOperands {
    type Error = LimitError;
    fn try_from(value: F64ToPluralOperands) -> Result<Self, Self::Error> {
        let fd = fixed_decimal::Decimal::try_from_f64(value.0, FloatPrecision::RoundTrip)?;
        Ok((&fd).into())
    }
}

#[derive(Debug, Clone, Copy)]
struct StrToPluralOperands<'a>(&'a str);

impl Display for StrToPluralOperands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<'a> TryFrom<StrToPluralOperands<'a>> for PluralOperands {
    type Error = <PluralOperands as FromStr>::Err;
    fn try_from(value: StrToPluralOperands) -> Result<Self, Self::Error> {
        value.0.parse()
    }
}

fn get_plural_form_for<V, E>(
    rule_type: PluralRuleType,
    locale: &Key,
    value: V,
    diag: &Diagnostics,
) -> PluralForm
where
    V: TryInto<PluralOperands, Error = E> + Display + Copy,
    E: Display,
{
    let plural_op = match value.try_into() {
        Ok(op) => op,
        Err(err) => {
            let _ = (err, diag);
            todo!()
        }
    };
    let rules =
        Plurals::<RawValue>::get_plural_rules(rule_type, locale).expect("some plural rules");
    rules.category_for(plural_op).into()
}

fn populate_attributes(
    attrs: &RawAttributes,
    args: &BTreeMap<String, RawValue>,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    loc: &Location,
    diag: &Diagnostics,
) -> RawAttributes {
    // TODO: diag and fk resolve further
    let _ = (resolved, set_fks, loc, diag);

    let mut new_attrs = Vec::with_capacity(attrs.attrs.len());

    for attr in &attrs.attrs {
        let var = match &attr.value {
            Some(RawAttributeValue::Variable(var)) => var,
            other => {
                new_attrs.push(RawAttribute {
                    key: attr.key.clone(),
                    value: other.clone(),
                });
                continue;
            }
        };
        let arg_name = var
            .name
            .strip_prefix("var_")
            .expect("variable keys must start with var_");
        let Some(arg) = args.get(arg_name) else {
            new_attrs.push(RawAttribute {
                key: attr.key.clone(),
                value: Some(RawAttributeValue::Variable(var.clone())),
            });
            continue;
        };

        let attr_value = match arg {
            RawValue::Variable(variable) => RawAttributeValue::Variable(variable.key.clone()),
            RawValue::Literal(raw_literal) => RawAttributeValue::Literal(raw_literal.clone()),
            RawValue::ForeignKey(_) => todo!(),
            RawValue::Component(_) => todo!(),
            RawValue::Bloc(bloc) => match extract_single_value_from_bloc(bloc) {
                Ok(Some(RawValue::Variable(var))) => RawAttributeValue::Variable(var.key.clone()),
                Ok(Some(RawValue::Literal(lit))) => RawAttributeValue::Literal(lit.clone()),
                Ok(None) => RawAttributeValue::Literal(RawLiteral::String(String::new())),
                Ok(Some(_)) => todo!(),
                Err(()) => todo!(),
            },
        };

        new_attrs.push(RawAttribute {
            key: attr.key.clone(),
            value: Some(attr_value),
        });
    }

    RawAttributes { attrs: new_attrs }
}

fn has_deps(
    value: &MergedPlurals,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    locale: &Key,
    waiting_on: &mut BTreeSet<Location>,
) -> bool {
    match value {
        MergedPlurals::RawValue(raw_value) => {
            raw_value_has_deps(raw_value, resolved, set_fks, locale, waiting_on)
        }
        MergedPlurals::Plurals(plurals) => {
            let mut has_deps = true;
            for (_, value) in plurals.forms.iter_forms() {
                if !raw_value_has_deps(value, resolved, set_fks, locale, waiting_on) {
                    has_deps = false;
                    break;
                }
            }
            has_deps
        }
    }
}

fn raw_value_has_deps(
    value: &RawValue,
    resolved: &ResolvedLocalesOrNamespaces,
    set_fks: &BTreeMap<Location, ResolvedValue>,
    locale: &Key,
    waiting_on: &mut BTreeSet<Location>,
) -> bool {
    match value {
        RawValue::ForeignKey(foreign_key) => {
            let path = &foreign_key.target_key_path;
            let loc = Loc {
                key_path: path,
                locale,
            };
            // TODO: check the kind of value we get back => if default, check default locale
            // => if subkeys, error
            match resolved.get_value_at(loc) {
                Some(ResolvedValueOrSubkeys::Value(_)) => {}
                // TODO: invalid values
                Some(ResolvedValueOrSubkeys::Defaulted) => todo!(),
                Some(ResolvedValueOrSubkeys::Dummy(_)) => todo!(),
                Some(ResolvedValueOrSubkeys::Subkeys(_)) => todo!(),
                None => {
                    let loc: Location = loc.into();
                    if set_fks.get(&loc).is_none() {
                        waiting_on.insert(loc);
                        return false;
                    }
                }
            }

            foreign_key
                .args
                .values()
                .all(|arg| raw_value_has_deps(arg, resolved, set_fks, locale, waiting_on))
        }
        RawValue::Literal(_) | RawValue::Variable(_) => true,
        RawValue::Component(component) => match &component.inner {
            Some(inner) => raw_value_has_deps(inner, resolved, set_fks, locale, waiting_on),
            None => true,
        },
        RawValue::Bloc(raw_values) => raw_values
            .iter()
            .all(|v| raw_value_has_deps(v, resolved, set_fks, locale, waiting_on)),
    }
}

impl ResolvedLocalesOrNamespaces {
    pub fn get_value_at(&self, loc: Loc) -> Option<&ResolvedValueOrSubkeys> {
        match self {
            ResolvedLocalesOrNamespaces::Locales(resolved_locales) => {
                let locale = resolved_locales.iter().find(|l| l.name == *loc.locale)?;
                locale.values.get_value_at(&loc.key_path.path)
            }
            ResolvedLocalesOrNamespaces::Namespaces(resolved_namespaces) => {
                let ns = resolved_namespaces
                    .iter()
                    .find(|ns| ns.name == *loc.key_path.namespace.as_ref().unwrap())?;
                let locale = ns.locales.iter().find(|l| l.name == *loc.locale)?;
                locale.values.get_value_at(&loc.key_path.path)
            }
        }
    }

    pub fn insert_at(
        &mut self,
        loc: Loc,
        value: ResolvedValueOrSubkeys,
    ) -> Option<ResolvedValueOrSubkeys> {
        match self {
            ResolvedLocalesOrNamespaces::Locales(resolved_locales) => {
                let locale = resolved_locales
                    .iter_mut()
                    .find(|l| l.name == *loc.locale)?;
                locale.values.insert_at(&loc.key_path.path, value)
            }
            ResolvedLocalesOrNamespaces::Namespaces(resolved_namespaces) => {
                let ns = resolved_namespaces
                    .iter_mut()
                    .find(|ns| ns.name == *loc.key_path.namespace.as_ref().unwrap())?;
                let locale = ns.locales.iter_mut().find(|l| l.name == *loc.locale)?;
                locale.values.insert_at(&loc.key_path.path, value)
            }
        }
    }
}

impl ResolvedValues {
    pub fn get_value_at(&self, mut path: &[Key]) -> Option<&ResolvedValueOrSubkeys> {
        let mut this = self;
        loop {
            match path {
                [] => break None,
                [key] => break this.values.get(key),
                [key, rest @ ..] => {
                    let value = this.values.get(key)?;
                    match value {
                        ResolvedValueOrSubkeys::Subkeys(subkeys) => {
                            this = subkeys;
                            path = rest;
                        }
                        _ => break None,
                    }
                }
            }
        }
    }

    pub fn insert_at(
        &mut self,
        mut path: &[Key],
        value: ResolvedValueOrSubkeys,
    ) -> Option<ResolvedValueOrSubkeys> {
        let mut this = self;
        loop {
            match path {
                [] => break None,
                [key] => break this.values.insert(key.clone(), value),
                [key, rest @ ..] => {
                    let value = this.values.get_mut(key)?;
                    match value {
                        ResolvedValueOrSubkeys::Subkeys(subkeys) => {
                            this = subkeys;
                            path = rest;
                        }
                        _ => break None,
                    }
                }
            }
        }
    }
}

fn remove_fks(
    values: RawLocalesOrNamespaces<MergedPlurals>,
    fks: &mut BTreeMap<Location, MergedPlurals>,
) -> ResolvedLocalesOrNamespaces {
    match values {
        RawLocalesOrNamespaces::Locales(locales) => {
            ResolvedLocalesOrNamespaces::Locales(remove_locales_fks(locales, fks, None))
        }
        RawLocalesOrNamespaces::Namespaces(namespaces) => {
            ResolvedLocalesOrNamespaces::Namespaces(remove_namespaces_fks(namespaces, fks))
        }
    }
}

fn remove_namespaces_fks(
    namespaces: Vec<RawNamespace<MergedPlurals>>,
    fks: &mut BTreeMap<Location, MergedPlurals>,
) -> Vec<ResolvedNamespace> {
    namespaces
        .into_iter()
        .map(|ns| remove_namespace_fks(ns, fks))
        .collect()
}

fn remove_namespace_fks(
    namespace: RawNamespace<MergedPlurals>,
    fks: &mut BTreeMap<Location, MergedPlurals>,
) -> ResolvedNamespace {
    let locales = remove_locales_fks(namespace.locales, fks, Some(namespace.name.clone()));
    ResolvedNamespace {
        name: namespace.name,
        locales,
    }
}

fn remove_locales_fks(
    locales: Vec<RawLocale<MergedPlurals>>,
    fks: &mut BTreeMap<Location, MergedPlurals>,
    ns: Option<Key>,
) -> Vec<ResolvedLocale> {
    locales
        .into_iter()
        .map(|loc| remove_locale_fks(loc, fks, ns.clone()))
        .collect()
}

fn remove_locale_fks(
    locale: RawLocale<MergedPlurals>,
    fks: &mut BTreeMap<Location, MergedPlurals>,
    ns: Option<Key>,
) -> ResolvedLocale {
    let mut loc = Location::new(locale.name.clone(), KeyPath::new(ns));
    let values = remove_values_fk(locale.values, fks, &mut loc);
    ResolvedLocale {
        name: locale.name,
        values,
    }
}

fn remove_merged_fks(value: MergedPlurals) -> Result<ResolvedValue, MergedPlurals> {
    match value {
        MergedPlurals::RawValue(value) => match map_value(value) {
            Ok(resolved) => Ok(resolved),
            Err(fk) => Err(MergedPlurals::RawValue(fk)),
        },
        MergedPlurals::Plurals(plurals) => match map_plurals(plurals) {
            Ok(resolved) => Ok(ResolvedValue::Plurals(resolved)),
            Err(fk) => Err(MergedPlurals::Plurals(fk)),
        },
    }
}

fn remove_value_or_subkey_fks<S, M, F, V, Fv>(
    value: RawValueOrSubkeys<V, S>,
    mut map_subkeys: F,
    mut map_values: Fv,
    fks: &mut BTreeMap<Location, MergedPlurals>,
    loc: &mut Location,
) -> Result<ResolvedValueOrSubkeys<M>, V>
where
    F: FnMut(S, &mut BTreeMap<Location, MergedPlurals>, &mut Location) -> M,
    Fv: FnMut(V) -> Result<ResolvedValue, V>,
{
    match value {
        RawValueOrSubkeys::Value(value) => match map_values(value) {
            Ok(v) => Ok(ResolvedValueOrSubkeys::Value(v)),
            Err(fk) => Err(fk),
        },
        RawValueOrSubkeys::Subkeys(values) => Ok(ResolvedValueOrSubkeys::Subkeys(map_subkeys(
            values, fks, loc,
        ))),
        RawValueOrSubkeys::Defaulted => Ok(ResolvedValueOrSubkeys::Defaulted),
        RawValueOrSubkeys::Dummy(dummy) => Ok(ResolvedValueOrSubkeys::Dummy(dummy)),
    }
}

fn remove_values_fk(
    values: RawValues<MergedPlurals>,
    fks: &mut BTreeMap<Location, MergedPlurals>,
    loc: &mut Location,
) -> ResolvedValues {
    let mut no_fks_values = BTreeMap::new();
    for (key, value) in values.values {
        let mut loc = loc.push_key(key.clone());
        match remove_value_or_subkey_fks(value, remove_values_fk, remove_merged_fks, fks, &mut loc)
        {
            Ok(resolved) => {
                no_fks_values.insert(key, resolved);
            }
            Err(fk) => {
                fks.insert(loc.clone(), fk);
            }
        }
    }

    ResolvedValues {
        values: no_fks_values,
    }
}

fn map_plurals(plurals: Plurals<RawValue>) -> Result<Plurals<ResolvedValue>, Plurals<RawValue>> {
    match map_forms(plurals.forms) {
        Ok(resolved) => Ok(Plurals {
            rule_type: plurals.rule_type,
            count_key: plurals.count_key,
            forms: resolved,
        }),
        Err(fk) => Err(Plurals {
            rule_type: plurals.rule_type,
            count_key: plurals.count_key,
            forms: fk,
        }),
    }
}

fn map_forms(
    forms: PluralForms<RawValue>,
) -> Result<PluralForms<ResolvedValue>, PluralForms<RawValue>> {
    let mut resolved_forms = BTreeMap::new();
    let mut fk_forms = BTreeMap::new();
    for (form, value) in forms.into_form_iter() {
        if fk_forms.is_empty() {
            match map_value(value) {
                Ok(resolved) => {
                    resolved_forms.insert(form, resolved);
                }
                Err(fk) => {
                    fk_forms.insert(form, fk);
                }
            }
        } else {
            fk_forms.insert(form, value);
        }
    }

    if fk_forms.is_empty() {
        Ok(PluralForms::try_from_iter(resolved_forms).unwrap())
    } else {
        for (form, value) in resolved_forms {
            fk_forms.insert(form, unmap_value(value));
        }
        Err(PluralForms::try_from_iter(fk_forms).unwrap())
    }
}

fn map_value(value: RawValue) -> Result<ResolvedValue, RawValue> {
    match value {
        RawValue::ForeignKey(fk) => Err(RawValue::ForeignKey(fk)),
        RawValue::Literal(raw_literal) => Ok(ResolvedValue::Literal(raw_literal)),
        RawValue::Variable(variable) => Ok(ResolvedValue::Variable(variable)),
        RawValue::Component(component) => {
            if let Some(inner) = component.inner {
                match map_value(*inner) {
                    Ok(no_fk) => Ok(ResolvedValue::Component(Component {
                        key: component.key,
                        inner: Some(Box::new(no_fk)),
                        attributes: component.attributes,
                    })),
                    Err(with_fks) => Err(RawValue::Component(Component {
                        key: component.key,
                        inner: Some(Box::new(with_fks)),
                        attributes: component.attributes,
                    })),
                }
            } else {
                Ok(ResolvedValue::Component(Component {
                    key: component.key,
                    inner: None,
                    attributes: component.attributes,
                }))
            }
        }
        RawValue::Bloc(bloc) => {
            let mut no_fks = Vec::new();
            let mut with_fks = Vec::new();

            for value in bloc {
                if with_fks.is_empty() {
                    match map_value(value) {
                        Ok(v) => no_fks.push(v),
                        Err(v) => with_fks.push(v),
                    }
                } else {
                    with_fks.push(value);
                }
            }

            if with_fks.is_empty() {
                Ok(ResolvedValue::Bloc(no_fks))
            } else {
                for value in no_fks {
                    with_fks.push(unmap_value(value));
                }
                Err(RawValue::Bloc(with_fks))
            }
        }
    }
}

fn unmap_value(value: ResolvedValue) -> RawValue {
    match value {
        ResolvedValue::Literal(raw_literal) => RawValue::Literal(raw_literal),
        ResolvedValue::Variable(variable) => RawValue::Variable(variable),
        ResolvedValue::Component(component) => RawValue::Component(Component {
            key: component.key,
            inner: component.inner.map(|v| Box::new(unmap_value(*v))),
            attributes: component.attributes,
        }),
        ResolvedValue::Bloc(bloc) => {
            let bloc = bloc.into_iter().map(unmap_value).collect();
            RawValue::Bloc(bloc)
        }
        ResolvedValue::Plurals(_) => unreachable!(),
    }
}
