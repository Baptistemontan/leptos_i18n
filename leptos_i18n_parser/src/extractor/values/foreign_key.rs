use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Display,
    str::FromStr,
};

use fixed_decimal::{FloatPrecision, LimitError};
use icu_plurals::PluralOperands;

use crate::{
    error::{Diagnostics, Error},
    extractor::values::plurals::{MergedPlurals, PluralForm, PluralForms, PluralRuleType, Plurals},
    options::Config,
    parser::{
        dummy::{Dummy, DummyArg},
        locale::{RawLocale, RawLocalesOrNamespaces, RawNamespace, RawValueOrSubkeys, RawValues},
        options::LocaleName,
        raw_value::{
            RawLiteral, RawValue,
            component::{Component, RawAttribute, RawAttributeValue, RawAttributes},
            foreign_key::ForeignKey,
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
    pub name: LocaleName,
    pub values: ResolvedValues,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedValue {
    Literal(RawLiteral),
    Variable(Variable),
    Component(Component<Self>),
    Bloc(Vec<Self>),
    Plurals(Plurals<Self>),
    Dummy(Dummy),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedValueOrSubkeys<S = ResolvedValues> {
    Value(ResolvedValue),
    Subkeys(S),
    Defaulted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedValues {
    pub values: BTreeMap<Key, ResolvedValueOrSubkeys>,
}

type Waiters = BTreeMap<Location, BTreeSet<Location>>;

impl ResolvedValue {
    pub fn is_empty(&self) -> bool {
        match self {
            ResolvedValue::Literal(RawLiteral::String(s)) => s.is_empty(),
            ResolvedValue::Bloc(b) => b.iter().all(ResolvedValue::is_empty),
            _ => false,
        }
    }
}

pub fn resolve_foreign_keys(
    values: RawLocalesOrNamespaces<MergedPlurals>,
    diag: &Diagnostics,
    cfg: &Config,
) -> ResolvedLocalesOrNamespaces {
    let mut fks = BTreeMap::<Location, MergedPlurals>::new();
    let mut resolved = remove_fks(values, &mut fks);

    let mut waiters: Waiters = BTreeMap::new();
    let mut queue: VecDeque<Location> = fks.keys().cloned().collect();

    while let Some(loc) = queue.pop_front() {
        let Some(value) = fks.remove(&loc) else {
            continue;
        };

        let maybe_resolved =
            resolve_fk(value, &loc, &resolved, &fks, &loc, diag, &mut waiters, cfg);

        match maybe_resolved {
            Ok(value) => {
                if let Some(waiters) = waiters.remove(&loc) {
                    queue.extend(waiters);
                }
                let insert_loc = Loc {
                    key_path: &loc.key_path,
                    locale: &loc.locale,
                };
                resolved.insert_at(insert_loc, ResolvedValueOrSubkeys::Value(value));
            }
            Err(value) => {
                fks.insert(loc, value);
            }
        }
    }

    // leftovers are cycles, impossible to resolve
    // emit error + create placeholder to reduce errors
    for (loc, value) in fks {
        let insert_loc = Loc {
            key_path: &loc.key_path,
            locale: &loc.locale,
        };
        let value = map_recursive_fk(value, &resolved, cfg, &loc, diag);
        resolved.insert_at(insert_loc, ResolvedValueOrSubkeys::Value(value));
        diag.emit_error(Error::RecursiveForeignKey { loc });
    }

    resolved
}

fn resolve_fk(
    mut value: MergedPlurals,
    current_loc: &Location,
    resolved: &ResolvedLocalesOrNamespaces,
    unset_fks: &BTreeMap<Location, MergedPlurals>,
    loc: &Location,
    diag: &Diagnostics,
    waiters: &mut Waiters,
    cfg: &Config,
) -> Result<ResolvedValue, MergedPlurals> {
    if !has_deps(
        &mut value,
        current_loc,
        resolved,
        unset_fks,
        waiters,
        cfg,
        diag,
    ) {
        return Err(value);
    }

    let value = resolve_value_fk(value, resolved, loc, diag, cfg);

    Ok(value)
}

fn resolve_value_fk(
    value: MergedPlurals,
    resolved: &ResolvedLocalesOrNamespaces,
    loc: &Location,
    diag: &Diagnostics,
    cfg: &Config,
) -> ResolvedValue {
    match value {
        MergedPlurals::RawValue(raw_value) => {
            resolve_raw_value_fk(raw_value, resolved, loc, diag, cfg)
        }
        MergedPlurals::Plurals(plurals) => {
            let forms = plurals
                .forms
                .map(|value| resolve_raw_value_fk(value, resolved, loc, diag, cfg));

            ResolvedValue::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: plurals.count_key,
                forms,
            })
        }
    }
}

fn resolve_foreign_key(
    foreign_key: ForeignKey,
    resolved: &ResolvedLocalesOrNamespaces,
    loc: &Location,
    diag: &Diagnostics,
    cfg: &Config,
) -> Option<ResolvedValue> {
    let target_loc = Loc {
        key_path: &foreign_key.target_location.key_path,
        locale: &foreign_key.target_location.locale,
    };
    let Ok(ResolvedValueOrSubkeys::Value(target_value)) =
        get_value_at_with_defaulting(target_loc, resolved, cfg)
    else {
        return None;
    };
    let args = foreign_key
        .args
        .into_iter()
        .map(|(key, value)| {
            let value = resolve_raw_value_fk(value, resolved, loc, diag, cfg);
            (key, value)
        })
        .collect::<BTreeMap<String, ResolvedValue>>();

    let value = populate_value(
        target_value,
        &args,
        loc,
        &foreign_key.target_location.key_path,
        diag,
        cfg,
    );

    Some(value)
}

fn resolve_raw_value_fk(
    value: RawValue,
    resolved: &ResolvedLocalesOrNamespaces,
    loc: &Location,
    diag: &Diagnostics,
    cfg: &Config,
) -> ResolvedValue {
    match value {
        RawValue::ForeignKey(foreign_key) => {
            resolve_foreign_key(foreign_key, resolved, loc, diag, cfg)
                .expect("should have already been checked by has_deps")
        }
        RawValue::Literal(raw_literal) => ResolvedValue::Literal(raw_literal),
        RawValue::Variable(variable) => ResolvedValue::Variable(variable),
        RawValue::Component(component) => {
            let inner = component
                .inner
                .map(|inner| resolve_raw_value_fk(*inner, resolved, loc, diag, cfg))
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
                .map(|v| resolve_raw_value_fk(v, resolved, loc, diag, cfg))
                .collect(),
        ),
        RawValue::Dummy(dummies) => ResolvedValue::Dummy(dummies),
    }
}

fn populate_value(
    value: &ResolvedValue,
    args: &BTreeMap<String, ResolvedValue>,
    loc: &Location,
    fk_path: &KeyPath,
    diag: &Diagnostics,
    cfg: &Config,
) -> ResolvedValue {
    match value {
        ResolvedValue::Literal(raw_literal) => ResolvedValue::Literal(raw_literal.clone()),
        ResolvedValue::Variable(variable) => populate_variable(variable, args),
        ResolvedValue::Component(component) => {
            let inner = component
                .inner
                .as_deref()
                .map(|inner| populate_value(inner, args, loc, fk_path, diag, cfg))
                .map(Box::new);
            let attributes = populate_attributes(&component.attributes, args, loc, fk_path, diag);
            ResolvedValue::Component(Component {
                key: component.key.clone(),
                inner,
                attributes,
            })
        }
        ResolvedValue::Bloc(resolved_values) => {
            let bloc = resolved_values
                .iter()
                .map(|value| populate_value(value, args, loc, fk_path, diag, cfg))
                .collect();
            ResolvedValue::Bloc(bloc)
        }
        ResolvedValue::Plurals(plurals) => populate_plurals(plurals, args, loc, fk_path, diag, cfg),
        ResolvedValue::Dummy(dumdum) => {
            let mut bloc = Vec::new();
            let mut dummies = Vec::new();
            for dummy in &dumdum.dummies {
                match dummy {
                    DummyArg::Variable(key) => {
                        let Some(arg) = args.get(&*key.name) else {
                            dummies.push(DummyArg::Variable(key.clone()));
                            continue;
                        };
                        bloc.push(ResolvedValue::Dummy(Dummy {
                            dummies: core::mem::take(&mut dummies),
                        }));
                        bloc.push(arg.clone());
                    }
                    DummyArg::Component(key) => dummies.push(DummyArg::Component(key.clone())),
                }
            }
            if bloc.is_empty() {
                ResolvedValue::Dummy(Dummy { dummies })
            } else if dummies.is_empty() {
                ResolvedValue::Bloc(bloc)
            } else {
                bloc.push(ResolvedValue::Dummy(Dummy { dummies }));
                ResolvedValue::Bloc(bloc)
            }
        }
    }
}

fn populate_variable(variable: &Variable, args: &BTreeMap<String, ResolvedValue>) -> ResolvedValue {
    match args.get(&*variable.key.name) {
        Some(value) => value.clone(),
        None => ResolvedValue::Variable(variable.clone()),
    }
}

fn populate_plurals(
    plurals: &Plurals<ResolvedValue>,
    args: &BTreeMap<String, ResolvedValue>,
    loc: &Location,
    fk_path: &KeyPath,
    diag: &Diagnostics,
    cfg: &Config,
) -> ResolvedValue {
    let mapped_count_key = map_plural_key(
        &plurals.count_key,
        plurals.rule_type,
        args,
        loc,
        fk_path,
        diag,
    );

    match mapped_count_key {
        Ok(new_count_key) => {
            let forms = plurals
                .forms
                .map_ref(|value| populate_value(value, args, loc, fk_path, diag, cfg));
            ResolvedValue::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: new_count_key,
                forms,
            })
        }
        Err(form) => {
            let value = plurals.forms.get_form_or_other(form);
            populate_value(value, args, loc, fk_path, diag, cfg)
        }
    }
}

fn extract_single_value_from_bloc(bloc: &[ResolvedValue]) -> Result<Option<&ResolvedValue>, ()> {
    let mut iter = bloc.iter().skip_while(|v| v.is_empty());

    let value = match iter.next() {
        Some(ResolvedValue::Bloc(bloc)) => extract_single_value_from_bloc(bloc)?,
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
    args: &BTreeMap<String, ResolvedValue>,
    loc: &Location,
    fk_path: &KeyPath,
    diag: &Diagnostics,
) -> Result<Key, PluralForm> {
    let arg_name = &*count_key.name;
    let Some(value) = args.get(arg_name) else {
        return Ok(count_key.clone());
    };

    match value {
        ResolvedValue::Literal(lit) => Err(get_form_for_literal(
            lit, rule_type, count_key, loc, fk_path, arg_name, diag,
        )),
        ResolvedValue::Variable(var) => Ok(var.key.clone()),
        ResolvedValue::Bloc(bloc) => match extract_single_value_from_bloc(bloc) {
            Ok(Some(ResolvedValue::Variable(var))) => Ok(var.key.clone()),
            Ok(Some(ResolvedValue::Literal(lit))) => Err(get_form_for_literal(
                lit, rule_type, count_key, loc, fk_path, arg_name, diag,
            )),
            Ok(None) | Ok(Some(_)) | Err(()) => {
                diag.emit_error(Error::InvalidFkCountArg {
                    key: count_key.clone(),
                    loc: loc.clone(),
                    foreign_key: fk_path.clone(),
                });
                Ok(count_key.clone())
            }
        },
        ResolvedValue::Component(_) | ResolvedValue::Plurals(_) | ResolvedValue::Dummy(_) => {
            diag.emit_error(Error::InvalidFkCountArg {
                key: count_key.clone(),
                loc: loc.clone(),
                foreign_key: fk_path.clone(),
            });
            Ok(count_key.clone())
        }
    }
}

fn get_form_for_literal(
    lit: &RawLiteral,
    rule_type: PluralRuleType,
    count_key: &Key,
    loc: &Location,
    fk_path: &KeyPath,
    arg_name: &str,
    diag: &Diagnostics,
) -> PluralForm {
    match lit {
        RawLiteral::Signed(value) => get_plural_form_for(rule_type, loc, *value, arg_name, diag),
        RawLiteral::Unsigned(value) => get_plural_form_for(rule_type, loc, *value, arg_name, diag),
        RawLiteral::String(value) => {
            get_plural_form_for(rule_type, loc, StrToPluralOperands(value), arg_name, diag)
        }
        RawLiteral::Float(value) => {
            get_plural_form_for(rule_type, loc, F64ToPluralOperands(*value), arg_name, diag)
        }
        RawLiteral::Bool(_) => {
            diag.emit_error(Error::InvalidFkCountArg {
                key: count_key.clone(),
                loc: loc.clone(),
                foreign_key: fk_path.clone(),
            });
            PluralForm::Other
        }
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
    loc: &Location,
    value: V,
    arg_name: &str,
    diag: &Diagnostics,
) -> PluralForm
where
    V: TryInto<PluralOperands, Error = E> + Display + Copy,
    E: Display,
{
    let str_value = value.to_string();
    let plural_op = match value.try_into() {
        Ok(op) => op,
        Err(err) => {
            diag.emit_error(Error::InvalidPluralOperandForeignKeyArg {
                loc: loc.clone(),
                arg_name: arg_name.to_string(),
                value: str_value,
                err: err.to_string(),
            });
            return PluralForm::Other;
        }
    };
    let rules =
        Plurals::<RawValue>::get_plural_rules(rule_type, &loc.locale).expect("some plural rules");
    rules.category_for(plural_op).into()
}

fn populate_attributes(
    attrs: &RawAttributes,
    args: &BTreeMap<String, ResolvedValue>,
    loc: &Location,
    fk_path: &KeyPath,
    diag: &Diagnostics,
) -> RawAttributes {
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
        let Some(arg) = args.get(&*var.name) else {
            new_attrs.push(RawAttribute {
                key: attr.key.clone(),
                value: Some(RawAttributeValue::Variable(var.clone())),
            });
            continue;
        };

        let value = populate_attribute(arg, var, loc, fk_path, diag);

        new_attrs.push(RawAttribute {
            key: attr.key.clone(),
            value: Some(value),
        });
    }

    RawAttributes { attrs: new_attrs }
}

fn populate_attribute(
    arg: &ResolvedValue,
    var_key: &Key,
    loc: &Location,
    fk_path: &KeyPath,
    diag: &Diagnostics,
) -> RawAttributeValue {
    match arg {
        ResolvedValue::Variable(variable) => RawAttributeValue::Variable(variable.key.clone()),
        ResolvedValue::Literal(raw_literal) => RawAttributeValue::Literal(raw_literal.clone()),
        ResolvedValue::Bloc(bloc) => match extract_single_value_from_bloc(bloc) {
            Ok(Some(ResolvedValue::Variable(var))) => RawAttributeValue::Variable(var.key.clone()),
            Ok(Some(ResolvedValue::Literal(lit))) => RawAttributeValue::Literal(lit.clone()),
            Ok(None) | Ok(Some(_)) | Err(()) => {
                diag.emit_error(Error::InvalidFkAttrArg {
                    key: var_key.clone(),
                    loc: loc.clone(),
                    foreign_key: fk_path.clone(),
                });
                RawAttributeValue::Literal(RawLiteral::String(String::new()))
            }
        },
        ResolvedValue::Component(_) | ResolvedValue::Plurals(_) | ResolvedValue::Dummy(_) => {
            diag.emit_error(Error::InvalidFkAttrArg {
                key: var_key.clone(),
                loc: loc.clone(),
                foreign_key: fk_path.clone(),
            });
            RawAttributeValue::Literal(RawLiteral::String(String::new()))
        }
    }
}

fn has_deps(
    value: &mut MergedPlurals,
    current_loc: &Location,
    resolved: &ResolvedLocalesOrNamespaces,
    unset_fks: &BTreeMap<Location, MergedPlurals>,
    waiters: &mut Waiters,
    cfg: &Config,
    diag: &Diagnostics,
) -> bool {
    match value {
        MergedPlurals::RawValue(raw_value) => raw_value_has_deps(
            raw_value,
            current_loc,
            resolved,
            unset_fks,
            waiters,
            cfg,
            diag,
        ),
        MergedPlurals::Plurals(plurals) => {
            let mut has_deps = true;
            for (_, value) in plurals.forms.iter_forms_mut() {
                if !raw_value_has_deps(value, current_loc, resolved, unset_fks, waiters, cfg, diag)
                {
                    has_deps = false;
                    break;
                }
            }
            has_deps
        }
    }
}

fn get_value_at_with_defaulting<'a, 'b>(
    mut loc: Loc<'b>,
    values: &'a ResolvedLocalesOrNamespaces,
    cfg: &'b Config,
) -> Result<&'a ResolvedValueOrSubkeys, Location> {
    loop {
        match values.get_value_at(loc) {
            Some(ResolvedValueOrSubkeys::Defaulted) => {}
            Some(value) => break Ok(value),
            None => break Err(loc.into()),
        }

        if loc.locale.key == cfg.default_locale {
            // This is a complicated situation here,
            // we need this check to stop an infinite loop
            // and we technically would want to emit an error here,
            // but it will duplicate the error in later step
            // the later step take precedence because it is more general
            // so here we just return a static ref to an empty string value
            // and let the next step deal with emitting the error and replacing the value.
            const PLACEHOLDER: &ResolvedValueOrSubkeys = &ResolvedValueOrSubkeys::Value(
                ResolvedValue::Literal(RawLiteral::String(String::new())),
            );
            break Ok(PLACEHOLDER);
        }

        let default_to_key = cfg
            .extensions
            .get(&loc.locale.key)
            .unwrap_or(&cfg.default_locale);
        let default_to = cfg
            .locales
            .iter()
            .find(|l| l.key == *default_to_key)
            .expect("this locale should exist");
        loc.locale = default_to;
    }
}

fn raw_value_has_deps(
    value: &mut RawValue,
    current_loc: &Location,
    resolved: &ResolvedLocalesOrNamespaces,
    unset_fks: &BTreeMap<Location, MergedPlurals>,
    waiters: &mut Waiters,
    cfg: &Config,
    diag: &Diagnostics,
) -> bool {
    match value {
        RawValue::ForeignKey(foreign_key) => {
            let loc = Loc {
                key_path: &foreign_key.target_location.key_path,
                locale: &foreign_key.target_location.locale,
            };
            let pointed_value = get_value_at_with_defaulting(loc, resolved, cfg);
            match pointed_value {
                Ok(ResolvedValueOrSubkeys::Value(_)) => {}
                Ok(ResolvedValueOrSubkeys::Defaulted) => unreachable!(),
                Ok(ResolvedValueOrSubkeys::Subkeys(_)) => {
                    diag.emit_error(Error::ForeignKeyToSubkey {
                        foreign_key: loc.key_path.clone(),
                        loc: current_loc.clone(),
                    });
                    // remove this invalid fk by setting it to an empty string
                    *value = RawValue::Literal(RawLiteral::String(String::new()));
                    return true;
                }
                Err(loc) => {
                    foreign_key.target_location = loc;
                    let loc = &foreign_key.target_location;
                    if !unset_fks.contains_key(loc) {
                        diag.emit_error(Error::InvalidForeignKey {
                            foreign_key: loc.key_path.clone(),
                            loc: current_loc.clone(),
                        });
                        // remove this invalid fk by setting it to an empty string
                        *value = RawValue::Literal(RawLiteral::String(String::new()));
                        return true;
                    }
                    let w = waiters.entry(loc.clone()).or_default();
                    w.insert(current_loc.clone());
                    return false;
                }
            }

            foreign_key.args.values_mut().all(|arg| {
                raw_value_has_deps(arg, current_loc, resolved, unset_fks, waiters, cfg, diag)
            })
        }
        RawValue::Literal(_) | RawValue::Variable(_) => true,
        RawValue::Component(component) => match component.inner.as_deref_mut() {
            Some(inner) => {
                raw_value_has_deps(inner, current_loc, resolved, unset_fks, waiters, cfg, diag)
            }
            None => true,
        },
        RawValue::Bloc(raw_values) => raw_values
            .iter_mut()
            .all(|v| raw_value_has_deps(v, current_loc, resolved, unset_fks, waiters, cfg, diag)),
        RawValue::Dummy(_) => true,
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
                let bloc = no_fks
                    .into_iter()
                    .map(unmap_value)
                    .chain(with_fks)
                    .collect();
                Err(RawValue::Bloc(bloc))
            }
        }
        RawValue::Dummy(dummy) => Ok(ResolvedValue::Dummy(dummy)),
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
        ResolvedValue::Dummy(dummy) => RawValue::Dummy(dummy),
    }
}

fn map_recursive_fk(
    value: MergedPlurals,
    resolved: &ResolvedLocalesOrNamespaces,
    cfg: &Config,
    loc: &Location,
    diag: &Diagnostics,
) -> ResolvedValue {
    match value {
        MergedPlurals::RawValue(value) => map_recursive_fk_value(value, resolved, loc, diag, cfg),
        MergedPlurals::Plurals(plurals) => {
            let forms = plurals
                .forms
                .map(|v| map_recursive_fk_value(v, resolved, loc, diag, cfg));
            ResolvedValue::Plurals(Plurals {
                rule_type: plurals.rule_type,
                count_key: plurals.count_key,
                forms,
            })
        }
    }
}

fn map_recursive_fk_value(
    value: RawValue,
    resolved: &ResolvedLocalesOrNamespaces,
    loc: &Location,
    diag: &Diagnostics,
    cfg: &Config,
) -> ResolvedValue {
    match value {
        RawValue::ForeignKey(foreign_key) => {
            match resolve_foreign_key(foreign_key, resolved, loc, diag, cfg) {
                Some(rv) => rv,
                None => {
                    // if not found, either the fk point to inexistant key, or is a cycle,
                    // both already been checked before and warning emitted
                    // so for both case just put a dummy value like an empty string.
                    ResolvedValue::Literal(RawLiteral::String("".to_string()))
                }
            }
        }
        RawValue::Literal(raw_literal) => ResolvedValue::Literal(raw_literal),
        RawValue::Variable(variable) => ResolvedValue::Variable(variable),
        RawValue::Component(component) => {
            let inner = component
                .inner
                .map(|inner| map_recursive_fk_value(*inner, resolved, loc, diag, cfg))
                .map(Box::new);
            ResolvedValue::Component(Component {
                key: component.key,
                inner,
                attributes: component.attributes,
            })
        }
        RawValue::Bloc(bloc) => {
            let bloc = bloc
                .into_iter()
                .map(|v| map_recursive_fk_value(v, resolved, loc, diag, cfg))
                .collect();
            ResolvedValue::Bloc(bloc)
        }
        RawValue::Dummy(dummy) => ResolvedValue::Dummy(dummy),
    }
}
