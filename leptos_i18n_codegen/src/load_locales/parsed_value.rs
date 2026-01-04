use std::fmt::Display;

use crate::utils::fit_in_leptos_tuple;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use leptos_i18n_parser::{
    parse_locales::parsed_value::{Attribute, AttributeValue, Attributes, ParsedValue},
    utils::{Key, KeyPath, UnwrapAt},
};

use super::{interpolate::LOCALE_FIELD_KEY, plurals, ranges};

pub const TRANSLATIONS_KEY: &str = if cfg!(feature = "dynamic_load") {
    "__i18n_translations__"
} else {
    "I18N_TRANSLATIONS"
};

impl<'a> From<&'a leptos_i18n_parser::parse_locales::parsed_value::Literal> for Literal<'a> {
    fn from(value: &'a leptos_i18n_parser::parse_locales::parsed_value::Literal) -> Self {
        match value {
            leptos_i18n_parser::parse_locales::parsed_value::Literal::String(s, index) => {
                Self::String(s, *index)
            }
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Signed(lit) => {
                Self::Signed(*lit)
            }
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Unsigned(lit) => {
                Self::Unsigned(*lit)
            }
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Float(lit) => {
                Self::Float(*lit)
            }
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Bool(lit) => Self::Bool(*lit),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'a> {
    String(&'a str, usize),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

impl Literal<'_> {
    fn to_token_stream(&self, strings_count: usize) -> TokenStream {
        match self {
            Literal::String(_, index) => {
                let translations_key = Key::new(TRANSLATIONS_KEY).unwrap_at("TRANSLATIONS_KEY");
                if cfg!(feature = "dynamic_load") {
                    quote!(l_i18n_crate::__private::index_translations::<#strings_count, #index>(#translations_key))
                } else {
                    quote! {
                        {
                            const S: &str = l_i18n_crate::__private::index_translations::<#strings_count, #index>(#translations_key);
                            S
                        }
                    }
                }
            }
            Literal::Signed(v) => ToTokens::to_token_stream(v),
            Literal::Unsigned(v) => ToTokens::to_token_stream(v),
            Literal::Float(v) => ToTokens::to_token_stream(v),
            Literal::Bool(v) => ToTokens::to_token_stream(v),
        }
    }
}

fn flatten(
    this: &ParsedValue,
    tokens: &mut Vec<TokenStream>,
    locale_field: &Key,
    strings_count: usize,
) {
    match this {
        ParsedValue::Default => unreachable!("defaulted value should never have been rendered"),
        ParsedValue::Subkeys(_) => unreachable!("subkeys should never have been rendered"),
        ParsedValue::Literal(lit) => tokens.push(Literal::from(lit).to_token_stream(strings_count)),
        ParsedValue::Ranges(ranges) => tokens.push(ranges::to_token_stream(ranges, strings_count)),
        ParsedValue::Variable { key, bounds } => {
            let ts = bounds.var_to_view(&key.ident, &locale_field.ident);
            tokens.push(quote! {{
                    let #key = core::clone::Clone::clone(&#key);
                    #ts
            }});
        }
        ParsedValue::Component {
            key,
            inner,
            attributes,
        } => {
            let attrs = attributes_to_token_stream(attributes, strings_count);
            if let Some(inner) = inner {
                let mut key_path = KeyPath::new(None);
                let captured_keys = inner
                    .get_keys(&mut key_path)
                    .unwrap_at("parsed_value::flatten_1")
                    .is_interpol()
                    .map(|keys| {
                        let keys = keys
                            .iter_keys()
                            .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
                        quote!(#(#keys)*)
                    });

                let inner = to_token_stream(inner, strings_count);
                let f = quote!(
                    {
                        #captured_keys
                        move || #inner
                    }
                );
                tokens.push(quote!({
                    let __boxed_children_fn = l_i18n_crate::reexports::leptos::children::ToChildren::to_children(#f);
                    let __attrs = { #attrs };
                    let #key = core::clone::Clone::clone(&#key);
                    move || {
                        l_i18n_crate::__private::InterpolateComp::to_view(&#key, core::clone::Clone::clone(&__boxed_children_fn), &__attrs)
                    }
                }));
            } else {
                tokens.push(quote!({
                    let __attrs = { #attrs };
                    let #key = core::clone::Clone::clone(&#key);
                    move || l_i18n_crate::__private::InterpolateCompSelfClosed::to_view(&#key, &__attrs)
                }));
            }
        }

        ParsedValue::Bloc(values) => {
            for value in values {
                flatten(value, tokens, locale_field, strings_count);
            }
        }
        ParsedValue::ForeignKey(foreign_key) => {
            let f_value = foreign_key.borrow();
            let value = f_value.as_inner("flatten");
            flatten(value, tokens, locale_field, strings_count);
        }
        ParsedValue::Plurals(plurals) => {
            tokens.push(plurals::to_token_stream(plurals, strings_count))
        }
        // don't emit any code for dummies, it will default to "" just for compiling
        ParsedValue::Dummy(_) => {}
    }
}

fn flatten_string(
    this: &ParsedValue,
    tokens: &mut Vec<TokenStream>,
    locale_field: &Key,
    strings_count: usize,
) {
    match this {
        ParsedValue::Default => unreachable!("defaulted value should never have been rendered"),
        ParsedValue::Subkeys(_) => unreachable!("subkeys should never have been rendered"),
        ParsedValue::Literal(lit) => {
            let ts = Literal::from(lit).to_token_stream(strings_count);
            tokens.push(quote!(core::fmt::Display::fmt(&#ts, __formatter)))
        }
        ParsedValue::Ranges(ranges) => tokens.push(ranges::as_string_impl(ranges, strings_count)),
        ParsedValue::Variable { key, bounds } => {
            let ts = bounds.var_fmt(key, locale_field);
            tokens.push(ts);
        }
        ParsedValue::Component {
            key,
            inner,
            attributes,
        } => {
            let attrs_ts = attributes_as_string_impl(attributes, strings_count);
            let attrs_ts = quote! {
                let __attrs: &[&dyn Fn(&mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result] = { #attrs_ts };
            };
            match inner {
                Some(inner_value) => {
                    let inner_ts = as_string_impl(inner_value, strings_count);
                    tokens.push(quote!({
                        #attrs_ts
                        l_i18n_crate::display::DisplayComponent::fmt(#key, __formatter, { |__formatter| #inner_ts }, l_i18n_crate::display::Attributes(__attrs))
                    }))
                }
                None => tokens.push(quote!({
                    #attrs_ts
                    l_i18n_crate::display::DisplayComponent::fmt_self_closing(#key, __formatter, l_i18n_crate::display::Attributes(__attrs))
                })),
            }
        }

        ParsedValue::Bloc(values) => {
            for value in values {
                flatten_string(value, tokens, locale_field, strings_count);
            }
        }
        ParsedValue::ForeignKey(foreign_key) => {
            let f_value = foreign_key.borrow();
            let value = f_value.as_inner("flatten_string");
            flatten_string(value, tokens, locale_field, strings_count);
        }
        ParsedValue::Plurals(plurals) => tokens.push(plurals::as_string_impl(
            plurals,
            &plurals.count_key,
            strings_count,
        )),
        // Same as for view
        ParsedValue::Dummy(_) => {}
    }
}

pub fn to_token_stream(this: &ParsedValue, strings_count: usize) -> TokenStream {
    let mut tokens = Vec::new();
    let locale_field = Key::new(LOCALE_FIELD_KEY).unwrap_at("LOCALE_FIELD_KEY");
    flatten(this, &mut tokens, &locale_field, strings_count);

    match &mut tokens[..] {
        [] => quote!(""),
        [value] => std::mem::take(value),
        values => fit_in_leptos_tuple(values),
    }
}

pub fn as_string_impl(this: &ParsedValue, strings_count: usize) -> TokenStream {
    let mut tokens = Vec::new();
    let locale_field = Key::new(LOCALE_FIELD_KEY).unwrap_at("LOCALE_FIELD_KEY");
    flatten_string(this, &mut tokens, &locale_field, strings_count);

    match &mut tokens[..] {
        [] => quote!(Ok(())),
        [value] => std::mem::take(value),
        values => quote!({ #(#values?;)* Ok(()) }),
    }
}

pub fn attributes_to_token_stream(this: &Attributes, strings_count: usize) -> TokenStream {
    let attrs = this
        .0
        .iter()
        .filter_map(|attr| attribute_to_token_stream(attr, strings_count));

    quote!(vec![#(#attrs),*])
}

pub fn attributes_as_string_impl(this: &Attributes, strings_count: usize) -> TokenStream {
    let attrs = this
        .0
        .iter()
        .filter_map(|attr| attribute_as_string_impl(attr, strings_count));

    quote!(&[#(#attrs),*])
}

pub fn attribute_value_to_token_stream(
    this: &AttributeValue,
    strings_count: usize,
) -> Option<TokenStream> {
    match this {
        // TODO: should we really skip `false` attributes ? already skipped for string rendering, but it might have an impact with DOM rendering ?
        AttributeValue::Literal(
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Bool(false),
        ) => None,
        AttributeValue::Literal(lit) => {
            let ts = Literal::from(lit).to_token_stream(strings_count);
            Some(ts)
        }
        AttributeValue::Variable(key) => Some(quote!( core::clone::Clone::clone(&#key) )),
    }
}

fn attribute_num_string_impl(key: &str, value: &impl Display) -> TokenStream {
    let s = format!(" {}=\"{}\"", key, value);
    quote!(::core::write!(__formatter, #s))
}

pub fn attribute_as_string_impl(this: &Attribute, strings_count: usize) -> Option<TokenStream> {
    let key = &this.key;
    let ts = match &this.value {
        None
        | Some(AttributeValue::Literal(
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Bool(true), // collapse `attre = true` to just `attr`
        )) => {
            let s = format!(" {}", key);
            quote!(::core::write!(__formatter, #s))
        }
        Some(AttributeValue::Literal(
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Bool(false),
        )) => {
            // Don't render `false` attributes
            return None;
        }
        Some(AttributeValue::Literal(
            lit_s @ leptos_i18n_parser::parse_locales::parsed_value::Literal::String(_, _),
        )) => {
            let ts = Literal::from(lit_s).to_token_stream(strings_count);
            let fstr = format!(" {}={{:?}}", key);
            quote!({
                let __v = #ts;
                ::core::write!(__formatter, #fstr, __v)
            })
        }
        Some(AttributeValue::Literal(
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Signed(value),
        )) => attribute_num_string_impl(key, value),
        Some(AttributeValue::Literal(
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Unsigned(value),
        )) => attribute_num_string_impl(key, value),
        Some(AttributeValue::Literal(
            leptos_i18n_parser::parse_locales::parsed_value::Literal::Float(value),
        )) => attribute_num_string_impl(key, value),
        Some(AttributeValue::Variable(var_key)) => {
            quote!({ l_i18n_crate::display::AttributeValue::fmt_with_name(&#var_key, __formatter, #key) })
        }
    };

    Some(quote! {
        &{
            |__formatter: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                #ts
            }
        }
    })
}

pub fn attribute_to_token_stream(this: &Attribute, strings_count: usize) -> Option<TokenStream> {
    let key = &this.key;
    let ts = match &this.value {
        Some(v) => attribute_value_to_token_stream(v, strings_count)?,
        None => quote!(true),
    };

    let ts = quote! {
        l_i18n_crate::reexports::leptos::prelude::IntoAnyAttribute::into_any_attr(
            l_i18n_crate::reexports::leptos::attr::custom::custom_attribute(#key, #ts)
        )
    };

    Some(ts)
}
