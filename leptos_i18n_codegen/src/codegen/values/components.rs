use leptos_i18n_parser::extraction::{Attribute, AttributeValue, Attributes, Value};
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::values::gen_render_value;

type Component = leptos_i18n_parser::parsing::Component<Value, Attributes>;

pub fn render_component(
    component: &Component,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> TokenStream {
    let attributes = render_attributes(&component.attributes, translations_ident, strings_count);

    match &component.inner {
        None => render_self_closed_comp(&component.key.ident, &attributes),
        Some(inner) => render_comp_with_children(
            &component.key.ident,
            inner,
            &attributes,
            translations_ident,
            strings_count,
            locale_field,
        ),
    }
}

fn render_comp_with_children(
    key: &syn::Ident,
    inner: &Value,
    attributes: &TokenStream,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> TokenStream {
    let captured_keys = super::captured_keys(inner);

    let inner = gen_render_value(inner, translations_ident, strings_count, locale_field);
    let children_fn = quote!(
        {
            #(
                let #captured_keys = core::clone::Clone::clone(&#captured_keys);
            )*
            move || #inner
        }
    );

    quote!({
        let __boxed_children_fn = __l_i18n_crate::reexports::leptos::children::ToChildren::to_children(#children_fn);
        let __attrs = { #attributes };
        let #key = core::clone::Clone::clone(&#key);
        move || {
            __l_i18n_crate::__private::InterpolateComp::to_view(&#key, core::clone::Clone::clone(&__boxed_children_fn), &__attrs)
        }
    })
}

fn render_self_closed_comp(key: &syn::Ident, attributes: &TokenStream) -> TokenStream {
    quote! ({
        let __attrs = { #attributes };
        let #key = core::clone::Clone::clone(&#key);
        move || __l_i18n_crate::__private::InterpolateCompSelfClosed::to_view(&#key, &__attrs)
    })
}

fn render_attributes(
    attributes: &Attributes,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    let attrs = attributes
        .attrs
        .iter()
        .map(|attr| render_attribute(attr, translations_ident, strings_count));
    quote!(vec![#(#attrs,)*])
}

fn render_attribute(
    attr: &Attribute,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    let key_index = &attr.key_index;
    let key = super::gen_string_access(*key_index, translations_ident, strings_count);
    let ts = match &attr.value {
        Some(value) => render_attr_value(value, translations_ident, strings_count),
        None => quote!(true),
    };

    quote! {
        __l_i18n_crate::reexports::leptos::prelude::IntoAnyAttribute::into_any_attr(
            __l_i18n_crate::reexports::leptos::attr::custom::custom_attribute(#key, #ts)
        )
    }
}

fn render_attr_value(
    value: &AttributeValue,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    match value {
        AttributeValue::Literal(lit) => {
            super::gen_render_lit(lit, translations_ident, strings_count)
        }
        AttributeValue::Variable(key) => {
            quote!(core::clone::Clone::clone(&#key))
        }
    }
}
