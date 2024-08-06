use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;

use super::{
    key::{Key, KeyPath},
    locale::Locale,
    parsed_value::{InterpolateKey, ParsedValue},
};

pub struct Interpolation {
    pub ident: syn::Ident,
    pub imp: TokenStream,
}

struct Field<'a> {
    generic: syn::Ident,
    kind: &'a InterpolateKey,
    into_view: Option<syn::Ident>,
}

impl Interpolation {
    pub fn new(
        key: &Key,
        enum_ident: &syn::Ident,
        keys_set: &HashSet<InterpolateKey>,
        locales: &[Locale],
        default_match: &TokenStream,
        key_path: &KeyPath,
    ) -> Self {
        let builder_name = format!("{}_builder", key.name);

        let ident = syn::Ident::new(&builder_name, Span::call_site());

        let dummy_ident = format_ident!("{}_dummy", ident);

        let locale_field = Key::new("_locale").unwrap();
        let into_view_field = Key::new("_into_views_marker").unwrap();

        let fields = keys_set
            .iter()
            .map(|kind| {
                let key = kind
                    .as_key()
                    .map(|key| key.name.as_str())
                    .unwrap_or("var_count");
                let name = format!("__{}__", key);
                let generic = syn::Ident::new(&name, Span::call_site());
                let into_view = kind
                    .as_comp()
                    .map(|key| format_ident!("__{}_into_view__", key.ident));
                Field {
                    generic,
                    kind,
                    into_view,
                }
            })
            .collect::<Vec<_>>();

        let into_views = fields
            .iter()
            .filter_map(|f| f.into_view.as_ref())
            .collect::<Vec<_>>();

        let type_def = Self::create_types(
            &ident,
            &dummy_ident,
            enum_ident,
            &locale_field,
            &into_view_field,
            &fields,
            &into_views,
        );

        let dummy_impl = Self::dummy_impl(
            &ident,
            &dummy_ident,
            enum_ident,
            &locale_field,
            &into_view_field,
            &fields,
            &into_views,
        );

        let into_view_impl = Self::into_view_impl(
            key,
            &ident,
            enum_ident,
            &locale_field,
            &fields,
            locales,
            default_match,
            key_path,
            &into_views,
        );

        let debug_impl = Self::debug_impl(&builder_name, &ident, &fields, &into_views);

        let display_impl = if cfg!(feature = "interpolate_display") {
            Self::display_impl(
                key,
                &ident,
                enum_ident,
                &locale_field,
                &fields,
                locales,
                default_match,
            )
        } else {
            quote!()
        };

        let imp = quote! {
            #type_def

            #into_view_impl

            #debug_impl

            #display_impl

            #dummy_impl
        };

        Self {
            imp,
            ident: dummy_ident,
        }
    }

    fn bounded_generics<'a>(
        fields: &'a [Field],
        into_views: &'a [&syn::Ident],
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let into_view_generics = into_views
            .iter()
            .map(|into_view| quote!(#into_view: l_i18n_crate::__private::leptos::IntoView));

        fields
            .iter()
            .map(|field| {
                let ident = &field.generic;
                let generic_bound = field.kind.get_generic();
                if let Some(into_view) = field.into_view.as_ref() {
                    quote!(#ident: #generic_bound<#into_view>)
                } else {
                    quote!(#ident: #generic_bound)
                }
            })
            .chain(into_view_generics)
    }

    fn dummy_impl(
        ident: &syn::Ident,
        dummy_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let type_builder_name = format_ident!("{}Builder", ident);

        let left_generics = Self::bounded_generics(fields, into_views);

        let right_generics = fields
            .iter()
            .map(|field| &field.generic)
            .chain(into_views.iter().copied());

        let builder_marker = fields.iter().map(|_| quote!(()));

        quote! {
            impl #dummy_ident {
                pub const fn new(#locale_field: #enum_ident) -> Self {
                    Self {
                        #locale_field
                    }
                }

                pub fn view_builder<#(#left_generics,)*>(self) -> #type_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #(#builder_marker,)*)> {
                    #ident::builder().#locale_field(self.#locale_field).#into_view_field(core::marker::PhantomData)
                }
            }
        }
    }

    fn create_types(
        ident: &syn::Ident,
        dummy_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let generics = fields
            .iter()
            .map(|field| &field.generic)
            .chain(into_views.iter().copied());

        let fields = fields.iter().map(|field| {
            let key = field.kind;
            let generic = &field.generic;
            quote!(#key: #generic)
        });

        let into_views_marker = quote! {
            #into_view_field: core::marker::PhantomData<(#(#into_views,)*)>
        };

        quote! {
            #[allow(non_camel_case_types, non_snake_case)]
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub struct #dummy_ident {
                #locale_field: #enum_ident
            }

            #[allow(non_camel_case_types, non_snake_case)]
            #[derive(l_i18n_crate::__private::typed_builder::TypedBuilder)]
            #[builder(crate_module_path=l_i18n_crate::typed_builder)]
            pub struct #ident<#(#generics,)*> {
                #locale_field: #enum_ident,
                #into_views_marker,
                #(#fields,)*
            }
        }
    }

    fn debug_impl(
        builder_name: &str,
        ident: &syn::Ident,
        fields: &[Field],
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let left_generics = fields
            .iter()
            .map(|field| &field.generic)
            .chain(into_views.iter().copied());

        let right_generics = left_generics.clone();

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> core::fmt::Debug for #ident<#(#right_generics,)*> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(#builder_name).finish()
                }
            }
        }
    }

    fn display_impl(
        key: &Key,
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
    ) -> TokenStream {
        let left_generics = fields.iter().filter_map(|field| {
            let ident = &field.generic;
            let generic = field.kind.get_string_generic().ok()?;
            Some(quote!(#ident: #generic))
        });

        let right_generics = fields.iter().map(|field| match field.kind {
            InterpolateKey::Count(t) => quote!(#t),
            _ => {
                let ident = &field.generic;
                quote!(#ident)
            }
        });

        let fields_key = fields.iter().map(|f| f.kind);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field } = self;);

        let locales_impls =
            Self::create_locale_string_impl(key, enum_ident, locales, default_match);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> ::core::fmt::Display for #ident<#(#right_generics,)*> {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
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

    #[allow(clippy::too_many_arguments)]
    fn into_view_impl(
        key: &Key,
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
        key_path: &KeyPath,
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let into_view_generics = into_views
            .iter()
            .map(|into_view| quote!(#into_view: l_i18n_crate::__private::leptos::IntoView));

        let left_generics = fields
            .iter()
            .map(|field| {
                let ident = &field.generic;
                let generic = field.kind.get_generic();
                if let Some(into_view) = field.into_view.as_ref() {
                    quote!(#ident: #generic<#into_view>)
                } else {
                    quote!(#ident: #generic)
                }
            })
            .chain(into_view_generics);

        let right_generics = fields
            .iter()
            .map(|field| &field.generic)
            .chain(into_views.iter().copied());

        if cfg!(feature = "show_keys_only") {
            let key = key_path.to_string_with_key(key);
            return quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics,)*> leptos::IntoView for #ident<#(#right_generics,)*> {
                    fn into_view(self) -> leptos::View {
                        let _ = self;
                        leptos::IntoView::into_view(#key)
                    }
                }
            };
        }

        let fields_key = fields.iter().map(|f| f.kind);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field, .. } = self;);

        let locales_impls = Self::create_locale_impl(key, enum_ident, locales, default_match);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> leptos::IntoView for #ident<#(#right_generics,)*> {
                fn into_view(self) -> leptos::View {
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
        enum_ident: &'a syn::Ident,
        locales: &'a [Locale],
        default_match: &TokenStream,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let mut default_match = default_match.clone();
        locales
            .iter()
            .enumerate()
            .rev()
            .filter_map(move |(i, locale)| {
                let locale_key = &locale.top_locale_name;

                let value = match locale.keys.get(key) {
                    None | Some(ParsedValue::Default) => {
                        default_match.extend(quote!(| #enum_ident::#locale_key));
                        return None;
                    }
                    Some(value) => value,
                };

                let ts = match i == 0 {
                    true => quote!(#default_match => { #value }),
                    false => quote!(#enum_ident::#locale_key => { #value }),
                };
                Some(ts)
            })
    }

    fn create_locale_string_impl<'a>(
        key: &'a Key,
        enum_ident: &'a syn::Ident,
        locales: &'a [Locale],
        default_match: &TokenStream,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let mut default_match = default_match.clone();
        locales
            .iter()
            .enumerate()
            .rev()
            .filter_map(move |(i, locale)| {
                let locale_key = &locale.top_locale_name;

                let value = match locale.keys.get(key) {
                    None | Some(ParsedValue::Default) => {
                        default_match.extend(quote!(| #enum_ident::#locale_key));
                        return None;
                    }
                    Some(value) => value,
                };

                let value = value.as_string_impl();

                let ts = match i == 0 {
                    true => quote!(#default_match => { #value }),
                    false => quote!(#enum_ident::#locale_key => { #value }),
                };
                Some(ts)
            })
    }
}

pub fn create_empty_type() -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub struct EmptyInterpolateValue;
    }
}
