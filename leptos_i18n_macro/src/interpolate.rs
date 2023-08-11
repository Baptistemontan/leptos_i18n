use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{key::Key, locale::Locale, value_kind::InterpolateKeyKind};

pub struct Interpolation {
    pub ident: syn::Ident,
    pub default_generic_ident: TokenStream,
    pub imp: TokenStream,
}

struct Field<'a> {
    generic: syn::Ident,
    name: String,
    kind: &'a InterpolateKeyKind<'a, 'a>,
}

impl Interpolation {
    pub fn new(key: &Key, keys_set: &HashSet<InterpolateKeyKind>, locales: &[Locale]) -> Self {
        let ident = syn::Ident::new(&format!("__{}_builder", key.name), Span::call_site());

        let locale_field = Key::new("__locale", crate::key::KeyKind::LocaleName).unwrap();

        let fields = keys_set
            .iter()
            .map(|kind| {
                let key = kind.as_key();
                let name = format!("__{}", key.name);
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
        let (new_impl, default_generic_ident) = Self::new_impl(&ident, &locale_field, &fields);

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

    fn new_impl(
        ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
    ) -> (TokenStream, TokenStream) {
        let generics = fields.iter().map(|_| quote!(EmptyInterpolateValue));

        let fields = fields.iter().map(|field| {
            let field_key = field.kind;
            quote!(#field_key: EmptyInterpolateValue)
        });

        let default_generic_ident = quote!(#ident<#(#generics,)*>);

        let imp = quote! {
            impl #default_generic_ident {
                pub fn new(#locale_field: LocaleEnum) -> Self {
                    Self {
                        #(#fields,)*
                        #locale_field
                    }
                }
            }
        };

        (imp, default_generic_ident)
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
                        InterpolateKeyKind::Variable(_) => quote!(__T),
                        InterpolateKeyKind::Component(_) => quote!(impl Fn(__leptos__::Scope, __leptos__::ChildrenFn) -> __leptos__::View + core::clone::Clone + 'static),
                    }
                } else {
                    let generic = &other_field.generic;
                    quote!(#generic)
                }
            });
            let other_fields = fields.iter().filter_map(|other_field| {
                if other_field.name == field.name {
                    None
                } else {
                    Some(other_field.kind.as_key())
                }
            }).chain(Some(locale_field)).collect::<Vec<_>>();

            let field_key = field.kind;

            let destructure = quote!(let Self { #(#other_fields,)* .. } = self;);
            let restructure = quote!(#ident { #(#other_fields,)* #field_key });

            match field.kind {
                InterpolateKeyKind::Variable(key) => {
                    quote!(pub fn #key<__T: __leptos__::IntoView + core::clone::Clone + 'static>(self, #field_key: __T) -> #ident<#(#output_generics,)*> {
                        #destructure
                        #restructure
                    })
                }
                InterpolateKeyKind::Component(key) => {
                    quote!(pub fn #key<__O: __leptos__::IntoView, __T: Fn(__leptos__::Scope, __leptos__::ChildrenFn) -> __O + core::clone::Clone + 'static>(self, _value_fn: __T) -> #ident<#(#output_generics,)*> {
                        #destructure
                        let #field_key = move |cx, children| __leptos__::IntoView::into_view(_value_fn(cx, children), cx);
                        #restructure
                    })
                }
            }
        })
    }

    fn into_view_impl<'a>(
        key: &Key,
        ident: &'a syn::Ident,
        locale_field: &'a Key,
        fields: &'a [Field],
        locales: &[Locale],
    ) -> TokenStream {
        let left_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            match field.kind {
                InterpolateKeyKind::Variable(_) => {
                    quote!(#ident: __leptos__::IntoView + core::clone::Clone + 'static)
                }
                InterpolateKeyKind::Component(_) => {
                    quote!(#ident: Fn(__leptos__::Scope, __leptos__::ChildrenFn) -> __leptos__::View + core::clone::Clone + 'static)
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
            impl<#(#left_generics,)*> __leptos__::IntoView for #ident<#(#right_generics,)*> {
                fn into_view(self, cx: __leptos__::Scope) -> __leptos__::View {
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
                    __leptos__::CollectView::collect_view([#value], cx)
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
