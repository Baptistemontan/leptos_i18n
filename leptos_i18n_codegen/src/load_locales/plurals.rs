use leptos_i18n_parser::{
    parse_locales::{
        locale::{InterpolOrLit, LiteralType},
        plurals::Plurals,
    },
    utils::{Key, KeyPath, UnwrapAt},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    load_locales::{interpolate::LOCALE_FIELD_KEY, parsed_value},
    utils::EitherOfWrapper,
};

pub enum PluralRuleType {
    Cardinal,
    Ordinal,
}

impl From<leptos_i18n_parser::parse_locales::plurals::PluralRuleType> for PluralRuleType {
    fn from(value: leptos_i18n_parser::parse_locales::plurals::PluralRuleType) -> Self {
        match value {
            leptos_i18n_parser::parse_locales::plurals::PluralRuleType::Cardinal => {
                PluralRuleType::Cardinal
            }
            leptos_i18n_parser::parse_locales::plurals::PluralRuleType::Ordinal => {
                PluralRuleType::Ordinal
            }
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

impl From<leptos_i18n_parser::parse_locales::plurals::PluralForm> for PluralForm {
    fn from(value: leptos_i18n_parser::parse_locales::plurals::PluralForm) -> Self {
        match value {
            leptos_i18n_parser::parse_locales::plurals::PluralForm::Zero => PluralForm::Zero,
            leptos_i18n_parser::parse_locales::plurals::PluralForm::One => PluralForm::One,
            leptos_i18n_parser::parse_locales::plurals::PluralForm::Two => PluralForm::Two,
            leptos_i18n_parser::parse_locales::plurals::PluralForm::Few => PluralForm::Few,
            leptos_i18n_parser::parse_locales::plurals::PluralForm::Many => PluralForm::Many,
            leptos_i18n_parser::parse_locales::plurals::PluralForm::Other => PluralForm::Other,
        }
    }
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

pub fn as_string_impl(this: &Plurals, count_key: &Key, strings_count: usize) -> TokenStream {
    let match_arms = this.forms.iter().map(|(form, value)| {
        let form = PluralForm::from(*form);
        let ts = parsed_value::as_string_impl(value, strings_count);
        quote!(#form => { #ts })
    });

    let locale_field = Key::new(LOCALE_FIELD_KEY).unwrap_at("LOCALE_FIELD_KEY");

    let other = parsed_value::as_string_impl(&this.other, strings_count);

    let rule_type = PluralRuleType::from(this.rule_type);

    quote! {{
        let _plural_rules = l_i18n_crate::__private::get_plural_rules(*#locale_field, #rule_type);
        match _plural_rules.category_for(core::clone::Clone::clone(#count_key)) {
            #(#match_arms,)*
            _ => #other,
        }
    }}
}

pub fn to_token_stream(this: &Plurals, strings_count: usize) -> TokenStream {
    let either_of = EitherOfWrapper::new(this.forms.len() + 1);
    let match_arms = this.forms.iter().enumerate().map(|(i, (form, value))| {
        let form = PluralForm::from(*form);
        let ts = parsed_value::to_token_stream(value, strings_count);
        let ts = either_of.wrap(i, ts);
        quote!(#form => { #ts })
    });

    let locale_field = Key::new(LOCALE_FIELD_KEY).unwrap_at("LOCALE_FIELD_KEY");
    let other = &*this.other;

    let mut captured_values = InterpolOrLit::Lit(LiteralType::String);
    let mut key_path = KeyPath::new(None);

    for value in this.forms.values().chain(Some(other)) {
        value
            .get_keys_inner(&mut key_path, &mut captured_values, false)
            .unwrap_at("plurals::to_token_stream_1");
    }

    let captured_values = captured_values.is_interpol().map(|keys| {
        let keys = keys
            .iter_keys()
            .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
        quote!(#(#keys)*)
    });

    let rule_type = PluralRuleType::from(this.rule_type);

    let count_key = &this.count_key;

    let other_ts = parsed_value::to_token_stream(&this.other, strings_count);
    let other = either_of.wrap(this.forms.len(), other_ts);

    quote! {
        {
            #captured_values
            let _plural_rules = l_i18n_crate::__private::get_plural_rules(#locale_field, #rule_type);
            move || {
                match _plural_rules.category_for(#count_key()) {
                    #(#match_arms,)*
                    _ => #other,
                }
            }
        }
    }
}
