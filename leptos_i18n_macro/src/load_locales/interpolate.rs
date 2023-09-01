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
        let ident = syn::Ident::new(&format!("{}_builder", key.name), Span::call_site());

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
            .map(|_| quote!(builders::EmptyInterpolateValue));
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

    fn split_at<T>(slice: &[T], i: usize) -> (&[T], &T, &[T]) {
        let (left, rest) = slice.split_at(i);
        let (mid, right) = rest.split_first().unwrap();
        (left, mid, right)
    }
    fn genenerate_set_fns(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        (0..fields.len())
            .map(|i| Self::split_at(fields, i))
            .map(|(left_fields, field, right_fields)| {
                Self::create_field_set_fn(ident, locale_field, left_fields, right_fields, field)
            })
            .collect()
    }

    fn builder_impl(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        let set_fns = Self::genenerate_set_fns(ident, locale_field, fields);

        quote! {
            #set_fns
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

    fn generate_generics<'a, F: 'a, T: Clone + 'a>(
        left_fields: &'a [Field],
        field_generic: Option<T>,
        right_fields: &'a [Field],
        other_field_map_fn: F,
    ) -> impl Iterator<Item = T> + 'a + Clone
    where
        F: FnMut(&'a Field) -> T + Copy,
    {
        left_fields
            .iter()
            .map(other_field_map_fn)
            .chain(field_generic)
            .chain(right_fields.iter().map(other_field_map_fn))
    }

    fn create_field_set_fn(
        ident: &syn::Ident,
        locale_field: &Key,
        left_fields: &[Field],
        right_fields: &[Field],
        field: &Field,
    ) -> TokenStream {
        let quoted_gen = |field: &Field| {
            let generic = &field.generic;
            quote!(#generic)
        };
        let output_field_generic = match field.kind {
            InterpolateKey::Variable(_) => {
                quote!(leptos::IntoView + core::clone::Clone + 'static)
            }
            InterpolateKey::Count(plural_type) => {
                quote!(Fn() -> #plural_type + core::clone::Clone + 'static)
            }
            InterpolateKey::Component(_) => quote!(
                Fn(leptos::Scope, leptos::ChildrenFn) -> leptos::View
                    + core::clone::Clone
                    + 'static
            ),
        };
        let output_generics = Self::generate_generics(
            left_fields,
            Some(quote!(impl #output_field_generic)),
            right_fields,
            quoted_gen,
        );
        let other_fields = Self::generate_generics(left_fields, None, right_fields, |field| {
            if let Some(key) = field.kind.as_key() {
                quote!(#key)
            } else {
                quote!(count)
            }
        })
        .chain(Some(quote!(#locale_field)));

        let kind = field.kind;

        let destructure = {
            let other_fields = other_fields.clone();
            quote!(let Self { #(#other_fields,)* .. } = self;)
        };
        let restructure = quote!(#ident { #(#other_fields,)* #kind });

        let set_function = match kind {
            InterpolateKey::Variable(key) => {
                quote! {
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
                quote! {
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
            InterpolateKey::Count(plural_type) => {
                quote! {
                    #[inline]
                    pub fn var_count<__T, __N>(self, var_count: __T) -> #ident<#(#output_generics,)*>
                        where __T: Fn() -> __N + core::clone::Clone + 'static,
                              __N: core::convert::Into<#plural_type>
                    {
                        #destructure
                        let var_count = move || core::convert::Into::into(var_count());
                        #restructure
                    }
                }
            }
        };

        let left_generics_empty =
            Self::generate_generics(left_fields, None, right_fields, |field| &field.generic);
        let left_generics_already_set = Self::generate_generics(
            left_fields,
            Some({
                let field_gen = &field.generic;
                quote!(#field_gen: #output_field_generic)
            }),
            right_fields,
            quoted_gen,
        );
        let right_generics_empty = Self::generate_generics(
            left_fields,
            Some(quote!(EmptyInterpolateValue)),
            right_fields,
            quoted_gen,
        );
        let right_generics_already_set =
            Self::generate_generics(left_fields, Some(&field.generic), right_fields, |field| {
                &field.generic
            });

        let compile_warning = match field.kind {
            InterpolateKey::Count(_) => "variable `count` is already set".to_string(),
            InterpolateKey::Variable(_) => format!("variable `{}` is already set", field.name),
            InterpolateKey::Component(_) => format!("component `{}` is already set", field.name),
        };

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics_empty,)*> #ident<#(#right_generics_empty,)*> {
                #set_function
            }
            #[allow(non_camel_case_types)]
            impl<#(#left_generics_already_set,)*> #ident<#(#right_generics_already_set,)*> {
                #[deprecated(note = #compile_warning)]
                #set_function
            }
        }
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
                InterpolateKey::Count(plural_type) => {
                    quote!(#ident: Fn() -> #plural_type + core::clone::Clone + 'static)
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
