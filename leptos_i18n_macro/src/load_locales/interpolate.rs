use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use super::{key::Key, locale::Locale, parsed_value::InterpolateKey};

pub struct Interpolation {
    pub ident: syn::Ident,
    pub default_generic_ident: TokenStream,
    pub imp: TokenStream,
}

struct Field<'a> {
    generic: syn::Ident,
    name: String,
    kind: &'a InterpolateKey<'a>,
}

impl Interpolation {
    pub fn new(key: &Key, keys_set: &HashSet<InterpolateKey>, locales: &[Locale]) -> Self {
        let ident = syn::Ident::new(&format!("__{}_builder", key.name), Span::call_site());

        let locale_field = Key::new("__locale", super::key::KeyKind::LocaleName).unwrap();

        let fields = keys_set
            .iter()
            .map(|kind| {
                let key = kind.as_key();
                let name = format!("__{}", key.map(|key| key.name.as_str()).unwrap_or("count"));
                let generic = syn::Ident::new(&name, Span::call_site());
                Field {
                    generic,
                    name,
                    kind,
                }
            })
            .collect::<Vec<_>>();

        let type_def = Self::create_type(&ident, &fields);
        let builder_impl = Self::builder_impl(&ident, &locale_field, &fields);
        let into_view_impl = Self::into_view_impl(key, &ident, &locale_field, &fields, locales);
        let new_impl = Self::new_impl(&ident, &locale_field, &fields);
        let default_generics = fields
            .iter()
            .map(|_| quote!(_builders::EmptyInterpolateValue));
        let default_generic_ident = quote!(#ident<#(#default_generics,)*>);

        let imp = quote! {
            #type_def

            #new_impl

            #into_view_impl

            #builder_impl
        };

        Self {
            imp,
            ident,
            default_generic_ident,
        }
    }

    fn new_impl(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        let generics = fields.iter().map(|_| quote!(EmptyInterpolateValue));

        let fields = fields.iter().map(|field| {
            let field_key = field.kind;
            quote!(#field_key: EmptyInterpolateValue)
        });

        quote! {
            impl #ident<#(#generics,)*> {
                pub const fn new(#locale_field: LocaleEnum) -> Self {
                    Self {
                        #(#fields,)*
                        #locale_field
                    }
                }
            }
        }
    }

    fn builder_impl(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        let impls = Self::create_key_impl(ident, locale_field, fields);

        let generics = fields
            .iter()
            .map(|field| &field.generic)
            .collect::<Vec<_>>();

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#generics,)*> #ident<#(#generics,)*> {
                #(
                    #impls
                )*
            }
        }
    }

    fn create_type(ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let generics = fields.iter().map(|field| &field.generic);
        let fields = fields.iter().map(|field| {
            let key = field.kind;
            let generic = &field.generic;
            quote!(#key: #generic)
        });

        quote! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub struct #ident<#(#generics,)*> {
                __locale: LocaleEnum,
                #(#fields,)*
            }
        }
    }

    fn create_key_impl<'a>(
        ident: &'a syn::Ident,
        locale_field: &'a Key,
        fields: &'a [Field],
    ) -> impl Iterator<Item = TokenStream> + 'a {
        fields.iter().map(move |field| {
            let output_generics = fields.iter().map(|other_field| {
                if other_field.name == field.name {
                    match field.kind {
                        InterpolateKey::Variable(_) | InterpolateKey::Count => quote!(__T),
                        InterpolateKey::Component(_) => quote!(impl Fn(leptos::Scope, leptos::ChildrenFn) -> leptos::View + core::clone::Clone + 'static),
                    }
                } else {
                    let generic = &other_field.generic;
                    quote!(#generic)
                }
            });
            let other_fields = fields.iter().filter_map(|other_field| {
                if other_field.name == field.name {
                    None
                } else if let Some(key) = other_field.kind.as_key() {
                    Some(quote!(#key))
                } else {
                    Some(quote!(count))
                }
            }).chain(Some(quote!(#locale_field))).collect::<Vec<_>>();

            let kind = field.kind;

            let destructure = quote!(let Self { #(#other_fields,)* .. } = self;);
            let restructure = quote!(#ident { #(#other_fields,)* #kind });

            match kind {
                InterpolateKey::Variable(key) => {
                    quote!{
                        #[inline]
                        pub fn #key<__T>(self, #key: __T) -> #ident<#(#output_generics,)*>
                            where __T: leptos::IntoView + core::clone::Clone + 'static
                        {
                            #destructure
                            #restructure
                        }
                    }
                }
                InterpolateKey::Component(key) => {
                    quote!{
                        #[inline]
                        pub fn #key<__O, __T>(self, #key: __T) -> #ident<#(#output_generics,)*>
                        where
                            __O: leptos::IntoView,
                            __T: Fn(leptos::Scope, leptos::ChildrenFn) -> __O + core::clone::Clone + 'static
                        {
                            #destructure
                            let #key = move |cx, children| leptos::IntoView::into_view(#key(cx, children), cx);
                            #restructure
                        }
                    }
                }
                InterpolateKey::Count => {
                    quote! {
                        #[inline]
                        pub fn var_count<__T>(self, var_count: __T) -> #ident<#(#output_generics,)*>
                            where __T: Fn() -> i64 + core::clone::Clone + 'static
                        {
                            #destructure
                            #restructure
                        }
                    }
                }
            }
        })
    }

    fn into_view_impl(
        key: &Key,
        ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
    ) -> TokenStream {
        let left_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            match field.kind {
                InterpolateKey::Variable(_) => {
                    quote!(#ident: leptos::IntoView + core::clone::Clone + 'static)
                }
                InterpolateKey::Component(_) => {
                    quote!(#ident: Fn(leptos::Scope, leptos::ChildrenFn) -> leptos::View + core::clone::Clone + 'static)
                }
                InterpolateKey::Count => {
                    quote!(#ident: Fn() -> i64 + core::clone::Clone + 'static)
                }
            }
        });

        let right_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            quote!(#ident)
        });

        let fields_key = fields.iter().map(|f| f.kind);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field } = self;);

        let locales_impls = Self::create_locale_impl(key, locales);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> leptos::IntoView for #ident<#(#right_generics,)*> {
                fn into_view(self, cx: leptos::Scope) -> leptos::View {
                    #destructure
                    match #locale_field {
                        #(
                            #locales_impls,
                        )*
                    }
                }
            }
        }
    }

    fn create_locale_impl<'a>(
        key: &'a Key,
        locales: &'a [Locale],
    ) -> impl Iterator<Item = TokenStream> + 'a {
        locales.iter().filter_map(|locale| {
            let locale_key = &locale.name;
            let value = locale.keys.get(key)?;

            Some(quote! {
                LocaleEnum::#locale_key => {
                    #value
                }

            })
        })
    }
}

pub fn create_empty_type() -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub struct EmptyInterpolateValue;
    }
}
