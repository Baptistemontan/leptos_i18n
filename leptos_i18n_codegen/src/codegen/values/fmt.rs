use std::collections::{BTreeMap, BTreeSet};

use leptos_i18n_parser::{
    extraction::{Literal, Locale, Value},
    parsing::Variable,
    utils::Key,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::codegen::locales::strings_accessor_method_name;

pub fn gen_fmt_body(
    non_defaulted_locales: &[(&Locale, &Value)],
    defaults: &BTreeMap<Key, BTreeSet<Key>>,
    enum_ident: &syn::Ident,
    keys_ident: &syn::Ident,
    locale_field: &syn::Ident,
    formatter_ident: &syn::Ident,
    data_ident: &syn::Ident,
) -> TokenStream {
    let translations_ident = if cfg!(feature = "dynamic_load") {
        format_ident!("__i18n_translations__")
    } else {
        format_ident!("__I18N_TRANSLATIONS__")
    };
    let render_match_arms = non_defaulted_locales.iter().map(|(l, value)| {
        let loc = &*l.name.key.ident;
        let accessor_name = strings_accessor_method_name(&l.name);
        let strings_count = l.strings.len();
        let defaults = defaults.get(&l.name.key).map(|defaulted_locales| {
            defaulted_locales.iter().map(|key| {
                quote!(| #enum_ident::#key)
            }).collect::<TokenStream>()
        });
        let fmt_value = gen_fmt_value(value, &translations_ident, strings_count, locale_field, formatter_ident);

        if cfg!(feature = "dynamic_load") {
            quote! {
                #enum_ident::#loc #defaults => {
                    let #translations_ident = __l_i18n_crate::__private::cast_unsized_strings::<_, #strings_count>(#data_ident);
                    #fmt_value
                }
            }
        } else {
            quote! {
                #enum_ident::#loc #defaults => {
                    const #translations_ident: &[&str; #strings_count] = super::#keys_ident::#accessor_name();
                    #fmt_value
                }
            }
        }
    });

    quote! {
        match #locale_field {
            #(
                #render_match_arms,
            )*
        }
    }
}

pub fn gen_fmt_value(
    value: &Value,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
    formatter_ident: &syn::Ident,
) -> TokenStream {
    match value {
        Value::Literal(lit) => gen_fmt_lit(lit, translations_ident, strings_count, formatter_ident),
        Value::Variable(Variable { key, bound }) => {
            bound.var_fmt(key, locale_field, formatter_ident)
        }
        Value::Component(component) => super::components::gen_fmt_component(
            component,
            translations_ident,
            strings_count,
            locale_field,
            formatter_ident,
        ),
        Value::Bloc(values) => match &**values {
            [] => quote!(core::fmt::Result::Ok(())),
            [values @ .., last_value] => {
                let iter = values.iter().map(|value| {
                    gen_fmt_value(
                        value,
                        translations_ident,
                        strings_count,
                        locale_field,
                        formatter_ident,
                    )
                });
                let last_value = gen_fmt_value(
                    last_value,
                    translations_ident,
                    strings_count,
                    locale_field,
                    formatter_ident,
                );
                quote! {
                    {
                        #(
                            #iter?;
                        )*
                        #last_value
                    }
                }
            }
        },
        Value::Plurals(plurals) => super::plurals::gen_fmt_plurals(
            plurals,
            translations_ident,
            strings_count,
            locale_field,
            formatter_ident,
        ),
    }
}

pub fn gen_fmt_lit(
    lit: &Literal,
    translations_ident: &syn::Ident,
    strings_count: usize,
    formatter_ident: &syn::Ident,
) -> TokenStream {
    match lit {
        Literal::String(index) => {
            let str_access =
                super::into_view::gen_string_access(*index, translations_ident, strings_count);
            quote! {
                {
                    let __s = #str_access;
                    core::fmt::Display::fmt(__s, #formatter_ident)
                }
            }
        }
        Literal::Signed(n) => quote! {
            core::fmt::Display::fmt(&#n, #formatter_ident)
        },
        Literal::Unsigned(n) => quote! {
            core::fmt::Display::fmt(&#n, #formatter_ident)
        },
        Literal::Float(n) => quote! {
            core::fmt::Display::fmt(&#n, #formatter_ident)
        },
        Literal::Bool(n) => quote! {
            core::fmt::Display::fmt(&#n, #formatter_ident)
        },
    }
}
