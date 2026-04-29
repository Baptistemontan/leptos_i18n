use std::collections::{BTreeMap, BTreeSet};

use crate::{
    codegen::locales::strings_accessor_method_name,
    utils::{EitherOfWrapper, fit_in_leptos_tuple},
};
use leptos_i18n_parser::{
    extraction::{Literal, Locale, Value},
    parsing::Variable,
    utils::Key,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn gen_render_body(
    non_defaulted_locales: &[(&Locale, &Value)],
    defaults: &BTreeMap<Key, BTreeSet<Key>>,
    enum_ident: &syn::Ident,
    keys_ident: &syn::Ident,
    locale_field: &syn::Ident,
) -> TokenStream {
    let translations_ident = if cfg!(feature = "dynamic_load") {
        format_ident!("__i18n_translations__")
    } else {
        format_ident!("__I18N_TRANSLATIONS__")
    };
    let either_of = EitherOfWrapper::new(non_defaulted_locales.len());
    let render_match_arms = non_defaulted_locales.iter().enumerate().map(|(i, (l, value))| {
        let loc = &*l.name.key.ident;
        let accessor_name = strings_accessor_method_name(&l.name);
        let strings_count = l.strings.len();
        let defaults = defaults.get(&l.name.key).map(|defaulted_locales| {
            defaulted_locales.iter().map(|key| {
                quote!(| #enum_ident::#key)
            }).collect::<TokenStream>()
        });
        let render_value = gen_render_value(value, &translations_ident, strings_count, locale_field);
        let render_value = either_of.wrap(i, render_value);
        if cfg!(feature = "dynamic_load") {
            let maybe_await = if cfg!(not(feature = "ssr")) {
                quote!(.await)
            } else {
                quote!()
            };
            quote! {
                #enum_ident::#loc #defaults => {
                    let #translations_ident: &'static [_; #strings_count] = super::#keys_ident::#accessor_name() #maybe_await;
                    #render_value
                }
            }
        } else {
            quote! {
                #enum_ident::#loc #defaults => {
                    const #translations_ident: &[&str; #strings_count] = super::#keys_ident::#accessor_name();
                    #render_value
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

pub fn gen_render_value(
    value: &Value,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> TokenStream {
    let mut tokens = Vec::new();
    flatten_value(
        value,
        &mut tokens,
        translations_ident,
        strings_count,
        locale_field,
    );
    match tokens.as_mut_slice() {
        [] => quote!(""),
        [value] => core::mem::take(value),
        values => fit_in_leptos_tuple(values),
    }
}

fn flatten_value(
    value: &Value,
    tokens: &mut Vec<TokenStream>,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) {
    match value {
        Value::Literal(lit) => {
            let ts = gen_render_lit(lit, translations_ident, strings_count);
            tokens.push(ts);
        }
        Value::Variable(Variable { key, bound }) => {
            let ts = bound.var_to_view(&key.ident, locale_field);
            tokens.push(quote! {{
                let #key = core::clone::Clone::clone(&#key);
                #ts
            }});
        }
        Value::Component(component) => {
            let ts = super::components::render_component(
                component,
                translations_ident,
                strings_count,
                locale_field,
            );
            tokens.push(ts);
        }
        Value::Bloc(values) => {
            for value in values {
                flatten_value(
                    value,
                    tokens,
                    translations_ident,
                    strings_count,
                    locale_field,
                );
            }
        }
        Value::Plurals(plurals) => {
            let ts = super::plurals::gen_render_plurals(
                plurals,
                translations_ident,
                strings_count,
                locale_field,
            );
            tokens.push(ts);
        }
    }
}

pub fn gen_string_access(
    index: usize,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    let str_access = quote! {
        __l_i18n_crate::__private::index_translations::<#strings_count, #index>(#translations_ident)
    };
    if cfg!(feature = "dynamic_load") {
        str_access
    } else {
        quote! {
            const {
                #str_access
            }
        }
    }
}

pub fn gen_render_lit(
    lit: &Literal,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    match lit {
        Literal::String(index) => {
            let str_access = quote! {
                __l_i18n_crate::__private::index_translations::<#strings_count, #index>(#translations_ident)
            };
            if cfg!(feature = "dynamic_load") {
                str_access
            } else {
                quote! {
                    const {
                        #str_access
                    }
                }
            }
        }
        Literal::Signed(v) => quote!(#v),
        Literal::Unsigned(v) => quote!(#v),
        Literal::Float(v) => quote!(#v),
        Literal::Bool(v) => quote!(#v),
    }
}

pub fn gen_const_values_match_arms(
    non_defaulted_locales: &[(&Locale, &Value)],
    defaults: &BTreeMap<Key, BTreeSet<Key>>,
    enum_ident: &syn::Ident,
    keys_ident: &syn::Ident,
) -> impl Iterator<Item = TokenStream> {
    non_defaulted_locales.iter().map(move |(locale, value)| {
        let Value::Literal(lit) = value else { todo!() };
        let loc_ident = &*locale.name.key.ident;
        let defaults = defaults.get(&locale.name.key).map(|defaulted_locales| {
            defaulted_locales.iter().map(|key| {
                quote!(| #enum_ident::#key)
            }).collect::<TokenStream>()
        });
        let value = match lit {
            Literal::String(index) => {
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    let value = &*locale.strings[*index];
                    quote!(__l_i18n_crate::keys::Literal::String(#value))
                } else {
                    let string_count = locale.strings.len();
                    let string_accessor = strings_accessor_method_name(&locale.name);
                    let string_accessor = if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                        format_ident!("{}_no_register", string_accessor)
                    } else {
                        string_accessor
                    };
                    quote!(
                        const {
                            __l_i18n_crate::keys::Literal::String(
                                __l_i18n_crate::__private::index_translations::<#string_count, #index>(
                                    super::#keys_ident::#string_accessor()
                                )
                            )
                        }
                    )
                }
            }
            Literal::Signed(n) => quote!(__l_i18n_crate::keys::Literal::Signed(#n)),
            Literal::Unsigned(n) => quote!(__l_i18n_crate::keys::Literal::Unsigned(#n)),
            Literal::Float(n) => quote!(__l_i18n_crate::keys::Literal::Float(#n)),
            Literal::Bool(n) => quote!(__l_i18n_crate::keys::Literal::Bool(#n)),
        };

        quote! {
            #enum_ident::#loc_ident #defaults => #value
        }
    })
}

pub fn captured_keys(value: &Value) -> BTreeSet<Key> {
    let mut keys = BTreeSet::new();
    captured_keys_inner(value, &mut keys);
    keys
}

pub fn captured_keys_inner(value: &Value, keys: &mut BTreeSet<Key>) {
    match value {
        Value::Literal(_) => {}
        Value::Variable(variable) => {
            keys.insert(variable.key.clone());
        }
        Value::Component(component) => {
            keys.insert(component.key.clone());
            if let Some(inner) = component.inner.as_deref() {
                captured_keys_inner(inner, keys);
            }
        }
        Value::Bloc(values) => {
            for value in values {
                captured_keys_inner(value, keys);
            }
        }
        Value::Plurals(plurals) => {
            for (_, value) in plurals.forms.iter_forms() {
                captured_keys_inner(value, keys);
            }
        }
    }
}
