use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{load_locales::interpolate::CACHED_LOCALE_FIELD_KEY, utils::key::KeyPath};

use super::parsed_value::ParsedValue;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PluralRuleType {
    Cardinal,
    #[allow(unused)]
    Ordinal,
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

        let rule_type = PluralRuleType::Cardinal;

        quote! {{
            let _plural_rules = l_i18n_crate::__private::get_plural_rules(*#locale_field, #rule_type);
            match _plural_rules.category_for(core::clone::Clone::clone(plural_count)) {
                #(#match_arms,)*
                _ => #other,
            }
        }}
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

        let mut captured_values = None;
        let mut key_path = KeyPath::new(None);

        for value in self.forms.values().chain(Some(other)) {
            value
                .get_keys_inner(&mut key_path, &mut captured_values)
                .unwrap();
        }

        let captured_values = captured_values.map(|keys| {
            let keys = keys
                .iter_keys()
                .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
            quote!(#(#keys)*)
        });

        let rule_type = PluralRuleType::Cardinal;

        quote! {
            leptos::IntoView::into_view(
                {
                    #captured_values
                    let _plural_rules = l_i18n_crate::__private::get_plural_rules(#locale_field, #rule_type);
                    move || {
                        match _plural_rules.category_for(plural_count()) {
                            #(#match_arms,)*
                            _ => #other,
                        }
                    }
                },
            )
        }
    }
}
