use leptos_i18n_parser::extraction::{Attribute, AttributeValue, Attributes, Literal, Value};
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::values::DummyFound;

type Component = leptos_i18n_parser::parsing::Component<Value, Attributes>;

pub fn render_component(
    component: &Component,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> Result<TokenStream, DummyFound> {
    let attributes = render_attributes(&component.attributes, translations_ident, strings_count);

    match component.inner.as_deref() {
        None => {
            let ts = render_self_closed_comp(&component.key.ident, &attributes);
            Ok(ts)
        }
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
) -> Result<TokenStream, DummyFound> {
    let captured_keys = super::into_view::captured_keys(inner);

    let inner =
        super::into_view::gen_render_value(inner, translations_ident, strings_count, locale_field)?;
    let children_fn = quote!(
        {
            #(
                let #captured_keys = core::clone::Clone::clone(&#captured_keys);
            )*
            move || #inner
        }
    );

    let ts = quote!({
        let __boxed_children_fn = __l_i18n_crate::reexports::leptos::children::ToChildren::to_children(#children_fn);
        let __attrs = { #attributes };
        let #key = core::clone::Clone::clone(&#key);
        move || {
            __l_i18n_crate::__private::InterpolateComp::to_view(&#key, core::clone::Clone::clone(&__boxed_children_fn), &__attrs)
        }
    });

    Ok(ts)
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
    let key = super::into_view::gen_string_access(*key_index, translations_ident, strings_count);
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
            super::into_view::gen_render_lit(lit, translations_ident, strings_count)
        }
        AttributeValue::Variable(key) => {
            quote!(core::clone::Clone::clone(&#key))
        }
    }
}

pub fn gen_fmt_component(
    component: &Component,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
    formatter_ident: &syn::Ident,
) -> Result<TokenStream, DummyFound> {
    let key = &*component.key.ident;
    let attrs_ts = attributes_as_string_impl(
        &component.attributes,
        translations_ident,
        strings_count,
        formatter_ident,
    );
    let attrs_ts = quote! {
        let __attrs: &[&dyn Fn(&mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result] = { #attrs_ts };
    };
    let ts = match component.inner.as_deref() {
        Some(inner_value) => {
            let inner_ts = super::fmt::gen_fmt_value(
                inner_value,
                translations_ident,
                strings_count,
                locale_field,
                formatter_ident,
            )?;
            quote!({
                #attrs_ts
                __l_i18n_crate::display::DisplayComponent::fmt(#key, #formatter_ident, { |#formatter_ident| #inner_ts }, __l_i18n_crate::display::Attributes(__attrs))
            })
        }
        None => quote!({
            #attrs_ts
            __l_i18n_crate::display::DisplayComponent::fmt_self_closing(#key, #formatter_ident, __l_i18n_crate::display::Attributes(__attrs))
        }),
    };

    Ok(ts)
}

pub fn attributes_as_string_impl(
    attributes: &Attributes,
    translations_ident: &syn::Ident,
    strings_count: usize,
    formatter_ident: &syn::Ident,
) -> TokenStream {
    let attrs = attributes.attrs.iter().filter_map(|attr| {
        attribute_as_string_impl(attr, translations_ident, strings_count, formatter_ident)
    });

    quote!(&[#(#attrs),*])
}

pub fn attribute_as_string_impl(
    attribute: &Attribute,
    translations_ident: &syn::Ident,
    strings_count: usize,
    formatter_ident: &syn::Ident,
) -> Option<TokenStream> {
    let key =
        super::into_view::gen_string_access(attribute.key_index, translations_ident, strings_count);
    let ts = match &attribute.value {
        None | Some(AttributeValue::Literal(Literal::Bool(true))) => {
            // collapse `attr = true` to just `attr`
            quote! {{
                core::write!(#formatter_ident, " ")?;
                core::fmt::Display::fmt(#key, #formatter_ident)
            }}
        }
        Some(AttributeValue::Literal(Literal::Bool(false))) => {
            // Don't render `false` attributes
            return None;
        }
        Some(AttributeValue::Literal(lit)) => {
            let ts =
                super::fmt::gen_fmt_lit(lit, translations_ident, strings_count, formatter_ident);
            // key="attr"
            quote! {{
                core::write!(#formatter_ident, " ")?;
                core::fmt::Display::fmt(#key, #formatter_ident)?;
                core::write!(#formatter_ident, "=\"")?;
                #ts?;
                core::write!(#formatter_ident, "\"")
            }}
        }
        Some(AttributeValue::Variable(var_key)) => {
            quote! {
                { __l_i18n_crate::display::AttributeValue::fmt_with_name(#var_key, #formatter_ident, #key) }
            }
        }
    };

    Some(quote! {
        &{
            |#formatter_ident: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                #ts
            }
        }
    })
}
