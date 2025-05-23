use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use fixed_decimal::{FixedDecimal, FloatPrecision};
use icu_plurals::{PluralCategory, PluralOperands, PluralRuleType as IcuRuleType, PluralRules};

use super::{
    error::{Error, Result},
    parsed_value::Literal,
    warning::{Warning, Warnings},
    StringIndexer,
};
use crate::utils::{Key, KeyPath, UnwrapAt};

use super::parsed_value::ParsedValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Plurals {
    pub rule_type: PluralRuleType,
    pub count_key: Key,
    // Box to be used inside the `ParsedValue::Plurals` variant without size recursion,
    // we could have `ParsedValue::Plurals(Box<Plurals>)`
    // but that makes `ParsedValue::Plurals(Plurals { .. })` impossible in match patterns.
    pub other: Box<ParsedValue>,
    pub forms: BTreeMap<PluralForm, ParsedValue>,
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

impl From<PluralRuleType> for IcuRuleType {
    fn from(value: PluralRuleType) -> Self {
        match value {
            PluralRuleType::Cardinal => IcuRuleType::Cardinal,
            PluralRuleType::Ordinal => IcuRuleType::Ordinal,
        }
    }
}

impl Plurals {
    fn get_plural_rules(&self, locale: &Key) -> Result<PluralRules> {
        let locale =
            locale
                .name
                .parse::<icu_locid::Locale>()
                .map_err(|err| Error::InvalidLocale {
                    locale: locale.name.clone(),
                    err,
                })?;
        PluralRules::try_new(&locale.into(), self.rule_type.into())
            .map_err(Error::PluralRulesError)
            .map_err(Box::new)
    }

    pub fn check_forms(&self, locale: &Key, key_path: &KeyPath, warnings: &Warnings) -> Result<()> {
        let plural_rules = self.get_plural_rules(locale)?;
        let forms = self.forms.keys().copied().collect::<BTreeSet<_>>();
        let used_forms = plural_rules
            .categories()
            .map(PluralForm::from_icu_category)
            .collect::<BTreeSet<_>>();
        for form in forms.difference(&used_forms).copied() {
            warnings.emit_warning(Warning::UnusedForm {
                locale: locale.clone(),
                key_path: key_path.to_owned(),
                form,
                rule_type: self.rule_type,
            });
        }
        Ok(())
    }

    fn populate_with_new_key(
        &self,
        new_key: Key,
        args: &BTreeMap<String, ParsedValue>,
        foreign_key: &KeyPath,
        locale: &Key,
        key_path: &KeyPath,
    ) -> Result<ParsedValue> {
        let other = self.other.populate(args, foreign_key, locale, key_path)?;
        let mut forms = BTreeMap::new();
        for (form, value) in &self.forms {
            let value = value.populate(args, foreign_key, locale, key_path)?;
            forms.insert(*form, value);
        }

        Ok(ParsedValue::Plurals(Plurals {
            rule_type: self.rule_type,
            count_key: new_key,
            other: Box::new(other),
            forms,
        }))
    }

    pub fn find_variable(
        values: &[ParsedValue],
        locale: &Key,
        key_path: &KeyPath,
        foreign_key: &KeyPath,
    ) -> Result<Key> {
        let mut iter = values.iter().peekable();
        while let Some(next) = iter.peek() {
            match next {
                ParsedValue::Literal(Literal::String(s, _)) if s.trim().is_empty() => {
                    iter.next();
                }
                ParsedValue::Variable { .. } => break,
                _ => {
                    return Err(Error::InvalidCountArg {
                        locale: locale.clone(),
                        key_path: key_path.to_owned(),
                        foreign_key: foreign_key.to_owned(),
                    }
                    .into())
                }
            }
        }
        let Some(ParsedValue::Variable { key, .. }) = iter.next() else {
            return Err(Error::InvalidCountArg {
                locale: locale.clone(),
                key_path: key_path.to_owned(),
                foreign_key: foreign_key.to_owned(),
            }
            .into());
        };

        for next in iter {
            match next {
                ParsedValue::Literal(Literal::String(s, _)) if s.trim().is_empty() => continue,
                _ => {
                    return Err(Error::InvalidCountArg {
                        locale: locale.clone(),
                        key_path: key_path.to_owned(),
                        foreign_key: foreign_key.to_owned(),
                    }
                    .into())
                }
            }
        }

        Ok(key.clone())
    }

    fn populate_with_count_arg(
        &self,
        count_arg: &ParsedValue,
        args: &BTreeMap<String, ParsedValue>,
        foreign_key: &KeyPath,
        locale: &Key,
        key_path: &KeyPath,
    ) -> Result<ParsedValue> {
        fn get_category<I: Into<PluralOperands>>(
            plurals: &Plurals,
            locale: &Key,
            input: I,
        ) -> Result<PluralCategory> {
            let plural_rules = plurals.get_plural_rules(locale)?;
            let cat = plural_rules.category_for(input);
            Ok(cat)
        }

        let category = match count_arg {
            ParsedValue::Literal(Literal::Float(count)) => {
                let count = FixedDecimal::try_from_f64(*count, FloatPrecision::Floating)
                    .unwrap_at("populate_with_count_arg_1");
                get_category(self, locale, &count)
            }
            ParsedValue::Literal(Literal::Unsigned(count)) => get_category(self, locale, *count),
            ParsedValue::Literal(Literal::Signed(count)) => get_category(self, locale, *count),
            ParsedValue::Bloc(values) => {
                let new_key = Self::find_variable(values, locale, key_path, foreign_key)?;
                return self.populate_with_new_key(new_key, args, foreign_key, locale, key_path);
            }
            ParsedValue::Variable { key, .. } => {
                return self.populate_with_new_key(key.clone(), args, foreign_key, locale, key_path)
            }
            _ => {
                return Err(Error::InvalidCountArg {
                    locale: locale.clone(),
                    key_path: key_path.to_owned(),
                    foreign_key: foreign_key.to_owned(),
                }
                .into())
            }
        };

        let category = category?;

        match PluralForm::from_icu_category(category) {
            PluralForm::Other => self.other.populate(args, foreign_key, locale, key_path),
            other_cat => self.forms.get(&other_cat).unwrap_or(&self.other).populate(
                args,
                foreign_key,
                locale,
                key_path,
            ),
        }
    }

    pub fn populate(
        &self,
        args: &BTreeMap<String, ParsedValue>,
        foreign_key: &KeyPath,
        locale: &Key,
        key_path: &KeyPath,
    ) -> Result<ParsedValue> {
        if let Some(count_arg) = args.get("var_count") {
            return self.populate_with_count_arg(count_arg, args, foreign_key, locale, key_path);
        }

        self.populate_with_new_key(self.count_key.clone(), args, foreign_key, locale, key_path)
    }

    pub fn index_strings(&mut self, strings: &mut StringIndexer) {
        for form in self.forms.values_mut() {
            form.index_strings(strings);
        }
        self.other.index_strings(strings);
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
