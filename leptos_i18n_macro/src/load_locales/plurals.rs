use std::{collections::HashMap, rc::Rc};

use fixed_decimal::{FixedDecimal, FloatPrecision};
use icu::plurals::{PluralCategory, PluralOperands, PluralRuleType as IcuRuleType, PluralRules};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    load_locales::{
        error::{Error, Result},
        interpolate::CACHED_LOCALE_FIELD_KEY,
        locale::LiteralType,
        parsed_value::{InterpolOrLit, Literal},
    },
    utils::key::{Key, KeyPath},
};

use super::parsed_value::ParsedValue;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PluralRuleType {
    Cardinal,
    Ordinal,
}

impl From<PluralRuleType> for IcuRuleType {
    fn from(value: PluralRuleType) -> Self {
        match value {
            PluralRuleType::Cardinal => IcuRuleType::Cardinal,
            PluralRuleType::Ordinal => IcuRuleType::Ordinal,
        }
    }
}

impl ToTokens for PluralRuleType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }

    fn to_token_stream(&self) -> TokenStream {
        match self {
            PluralRuleType::Cardinal => {
                quote!(l_i18n_crate::reexports::icu::plurals::PluralRuleType::Cardinal)
            }
            PluralRuleType::Ordinal => {
                quote!(l_i18n_crate::reexports::icu::plurals::PluralRuleType::Ordinal)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

impl ToTokens for PluralForm {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }

    fn to_token_stream(&self) -> TokenStream {
        match self {
            PluralForm::Zero => quote!(l_i18n_crate::reexports::icu::plurals::PluralCategory::Zero),
            PluralForm::One => quote!(l_i18n_crate::reexports::icu::plurals::PluralCategory::One),
            PluralForm::Two => quote!(l_i18n_crate::reexports::icu::plurals::PluralCategory::Two),
            PluralForm::Few => quote!(l_i18n_crate::reexports::icu::plurals::PluralCategory::Few),
            PluralForm::Many => quote!(l_i18n_crate::reexports::icu::plurals::PluralCategory::Many),
            PluralForm::Other => {
                quote!(l_i18n_crate::reexports::icu::plurals::PluralCategory::Other)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plurals {
    pub rule_type: PluralRuleType,
    // Box to be used inside the `ParsedValue::Plurals` variant without size recursion,
    // we could have `ParsedValue::Plurals(Box<Plurals>)`
    // but that makes `ParsedValue::Plurals(Plurals { .. })` impossible in match patterns.
    pub other: Box<ParsedValue>,
    pub forms: HashMap<PluralForm, ParsedValue>,
}

impl Plurals {
    pub fn as_string_impl(&self) -> TokenStream {
        let match_arms = self.forms.iter().map(|(form, value)| {
            let ts = value.as_string_impl();
            quote!(#form => #ts)
        });

        let locale_field = CACHED_LOCALE_FIELD_KEY.with(Clone::clone);

        let other = self.other.as_string_impl();

        let rule_type = self.rule_type;

        quote! {{
            let _plural_rules = l_i18n_crate::__private::get_plural_rules(*#locale_field, #rule_type);
            match _plural_rules.category_for(core::clone::Clone::clone(var_count)) {
                #(#match_arms,)*
                _ => #other,
            }
        }}
    }

    fn populate_with_count_arg(
        &self,
        count_arg: &ParsedValue,
        args: &HashMap<String, ParsedValue>,
        foreign_key: &KeyPath,
        locale: &Rc<Key>,
        key_path: &KeyPath,
    ) -> Result<ParsedValue> {
        fn get_category<I: Into<PluralOperands>>(
            plurals: &Plurals,
            locale: &Rc<Key>,
            input: I,
        ) -> PluralCategory {
            let locale = locale.name.parse::<icu::locid::Locale>().unwrap();
            let plural_rules =
                PluralRules::try_new(&locale.into(), plurals.rule_type.into()).unwrap();
            plural_rules.category_for(input)
        }

        let category = match count_arg {
            ParsedValue::Literal(Literal::Float(count)) => {
                let count = FixedDecimal::try_from_f64(*count, FloatPrecision::Floating).unwrap();
                get_category(self, locale, &count)
            }
            ParsedValue::Literal(Literal::Unsigned(count)) => get_category(self, locale, *count),
            ParsedValue::Literal(Literal::Signed(count)) => get_category(self, locale, *count),
            _ => {
                return Err(Error::InvalidCountArg {
                    locale: locale.clone(),
                    key_path: key_path.to_owned(),
                    foreign_key: foreign_key.to_owned(),
                })
            }
        };

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
        args: &HashMap<String, ParsedValue>,
        foreign_key: &KeyPath,
        locale: &Rc<Key>,
        key_path: &KeyPath,
    ) -> Result<ParsedValue> {
        if let Some(count_arg) = args.get("var_count") {
            return self.populate_with_count_arg(count_arg, args, foreign_key, locale, key_path);
        }

        let other = self.other.populate(args, foreign_key, locale, key_path)?;
        let mut forms = HashMap::new();
        for (form, value) in &self.forms {
            let value = value.populate(args, foreign_key, locale, key_path)?;
            forms.insert(*form, value);
        }

        Ok(ParsedValue::Plurals(Plurals {
            rule_type: self.rule_type,
            other: Box::new(other),
            forms,
        }))
    }
}

impl ToTokens for Plurals {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.to_token_stream())
    }

    fn to_token_stream(&self) -> TokenStream {
        let match_arms = self
            .forms
            .iter()
            .map(|(form, value)| quote!(#form => #value));

        let locale_field = CACHED_LOCALE_FIELD_KEY.with(Clone::clone);
        let other = &*self.other;

        let mut captured_values = InterpolOrLit::Lit(LiteralType::String);
        let mut key_path = KeyPath::new(None);

        for value in self.forms.values().chain(Some(other)) {
            value
                .get_keys_inner(&mut key_path, &mut captured_values, false)
                .unwrap();
        }

        let captured_values = captured_values.is_interpol().map(|keys| {
            let keys = keys
                .iter_keys()
                .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
            quote!(#(#keys)*)
        });

        let rule_type = self.rule_type;

        quote! {
            leptos::IntoView::into_view(
                {
                    #captured_values
                    let _plural_rules = l_i18n_crate::__private::get_plural_rules(#locale_field, #rule_type);
                    move || {
                        match _plural_rules.category_for(var_count()) {
                            #(#match_arms,)*
                            _ => #other,
                        }
                    }
                },
            )
        }
    }
}
