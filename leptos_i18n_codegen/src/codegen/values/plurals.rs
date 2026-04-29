use std::collections::BTreeSet;

use leptos_i18n_parser::extraction::{PluralForm, PluralRuleType, Plurals, Value};
use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::EitherOfWrapper;

pub fn gen_render_plurals(
    plurals: &Plurals,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> TokenStream {
    let mut captured_keys = BTreeSet::new();
    for (_, value) in plurals.forms.iter_forms() {
        super::into_view::captured_keys_inner(value, &mut captured_keys);
    }

    let rule_type_ts = render_rule_type(plurals.rule_type);
    let count_key = &*plurals.count_key.ident;
    let forms_count = plurals.forms.iter_forms().count();
    let either_of = EitherOfWrapper::new(forms_count);

    let match_arms = plurals
        .forms
        .iter_forms()
        .enumerate()
        .map(|(idx, (plural_form, value))| {
            render_plural_form_match_arm(
                plural_form,
                value,
                &either_of,
                idx,
                translations_ident,
                strings_count,
                locale_field,
            )
        });

    quote! {{
        #(
            let #captured_keys = core::clone::Clone::clone(&#captured_keys);
        )*
        let _plural_rules = __l_i18n_crate::__private::get_plural_rules(#locale_field, #rule_type_ts);
        move || {
            match _plural_rules.category_for(#count_key()) {
                #(
                    #match_arms,
                )*
            }
        }
    }}
}

fn render_rule_type(rule_type: PluralRuleType) -> TokenStream {
    match rule_type {
        PluralRuleType::Cardinal => {
            quote!(__l_i18n_crate::reexports::icu::plurals::PluralRuleType::Cardinal)
        }
        PluralRuleType::Ordinal => {
            quote!(__l_i18n_crate::reexports::icu::plurals::PluralRuleType::Ordinal)
        }
    }
}

fn render_plural_form_match_arm(
    plural_form: PluralForm,
    value: &Value,
    either_of: &EitherOfWrapper,
    idx: usize,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> TokenStream {
    let render_value_ts =
        super::into_view::gen_render_value(value, translations_ident, strings_count, locale_field);
    let render_value_ts = either_of.wrap(idx, render_value_ts);
    match plural_form {
        PluralForm::Zero => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Zero => #render_value_ts
        },
        PluralForm::One => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::One => #render_value_ts
        },
        PluralForm::Two => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Two => #render_value_ts
        },
        PluralForm::Few => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Few => #render_value_ts
        },
        PluralForm::Many => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Many => #render_value_ts
        },
        PluralForm::Other => quote! {
            _ => #render_value_ts
        },
    }
}

pub fn gen_fmt_plurals(
    plurals: &Plurals,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
    formatter_ident: &syn::Ident,
) -> TokenStream {
    let rule_type_ts = render_rule_type(plurals.rule_type);
    let count_key = &*plurals.count_key.ident;

    let match_arms = plurals.forms.iter_forms().map(|(plural_form, value)| {
        fmt_plural_form_match_arm(
            plural_form,
            value,
            translations_ident,
            strings_count,
            locale_field,
            formatter_ident,
        )
    });

    quote! {{
        let _plural_rules = __l_i18n_crate::__private::get_plural_rules(#locale_field, #rule_type_ts);
        match _plural_rules.category_for(core::clone::Clone::clone(#count_key)) {
            #(
                #match_arms,
            )*
        }
    }}
}

fn fmt_plural_form_match_arm(
    plural_form: PluralForm,
    value: &Value,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
    formatter_ident: &syn::Ident,
) -> TokenStream {
    let render_value_ts = super::fmt::gen_fmt_value(
        value,
        translations_ident,
        strings_count,
        locale_field,
        formatter_ident,
    );
    match plural_form {
        PluralForm::Zero => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Zero => #render_value_ts
        },
        PluralForm::One => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::One => #render_value_ts
        },
        PluralForm::Two => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Two => #render_value_ts
        },
        PluralForm::Few => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Few => #render_value_ts
        },
        PluralForm::Many => quote! {
            __l_i18n_crate::reexports::icu::plurals::PluralCategory::Many => #render_value_ts
        },
        PluralForm::Other => quote! {
            _ => #render_value_ts
        },
    }
}
