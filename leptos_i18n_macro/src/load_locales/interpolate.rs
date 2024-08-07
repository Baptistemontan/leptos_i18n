use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use super::parsed_value::BoundOrType;
use super::{
    key::{Key, KeyPath},
    locale::Locale,
    parsed_value::{InterpolateKey, ParsedValue},
};

pub struct Interpolation {
    pub ident: syn::Ident,
    pub imp: TokenStream,
}

enum GenericOrType {
    Generic(syn::Ident),
    Type(TokenStream),
}

impl GenericOrType {
    pub fn as_generic(&self) -> Option<&syn::Ident> {
        match self {
            GenericOrType::Generic(ident) => Some(ident),
            GenericOrType::Type(_) => None,
        }
    }
}

impl ToTokens for GenericOrType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            GenericOrType::Generic(ident) => ident.to_tokens(tokens),
            GenericOrType::Type(ts) => ts.to_tokens(tokens),
        }
    }
}

struct Field<'a> {
    generic: GenericOrType,
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

        let typed_builder_name = format_ident!("{}Builder", ident);

        let fields = keys_set
            .iter()
            .map(|kind| {
                let key = kind
                    .as_key()
                    .map(|key| key.name.as_str())
                    .unwrap_or("plural_count");
                let name = format!("__{}__", key);
                let generic = match kind.get_generic() {
                    BoundOrType::Bound(_) => {
                        GenericOrType::Generic(syn::Ident::new(&name, Span::call_site()))
                    }
                    BoundOrType::Type(t) => GenericOrType::Type(t),
                };
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
            &typed_builder_name,
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

        let (display_impl, builder_display) = if cfg!(feature = "interpolate_display") {
            let display_impl = Self::display_impl(
                key,
                &ident,
                enum_ident,
                &locale_field,
                &fields,
                locales,
                default_match,
                &into_views,
            );
            let builder_display = Self::builder_string_build_fns(
                enum_ident,
                &typed_builder_name,
                &fields,
                &into_views,
            );
            (display_impl, builder_display)
        } else {
            (quote!(), quote!())
        };

        let imp = quote! {
            #type_def

            #into_view_impl

            #debug_impl

            #display_impl

            #builder_display

            #dummy_impl
        };

        Self {
            imp,
            ident: dummy_ident,
        }
    }

    fn get_string_bounded_left_generics<'a>(
        fields: &'a [Field],
    ) -> impl Iterator<Item = TokenStream> + Clone + 'a {
        fields.iter().filter_map(|field| {
            let ident = &field.generic.as_generic()?;
            let generic = field.kind.get_string_generic().into_bound()?;
            Some(quote!(#ident: #generic))
        })
    }
    fn get_string_bounded_right_generics<'a>(
        fields: &'a [Field],
        into_views: &'a [&syn::Ident],
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let into_views = into_views.iter().map(|_| quote!(()));

        fields
            .iter()
            .map(|field| match &field.generic {
                GenericOrType::Type(t) => t.clone(),
                GenericOrType::Generic(ident) => quote!(#ident),
            })
            .chain(into_views)
    }

    fn builder_string_build_fns(
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        fields: &[Field],
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let left_generics = Self::get_string_bounded_left_generics(fields);
        let right_generics = Self::get_string_bounded_right_generics(fields, into_views);

        let into_views = into_views.iter().map(|_| quote!(()));
        let marker = fields.iter().map(|field| match &field.generic {
            GenericOrType::Type(t) => t.clone(),
            GenericOrType::Generic(ident) => quote!(#ident),
        });

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #((#marker,),)*)> {
                #[inline]
                pub fn build_display(self) -> impl std::fmt::Display {
                    self.build()
                }

                #[inline]
                pub fn build_string(self) -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Owned(self.build().to_string())
                }
            }
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
            .filter_map(|field| {
                let ident = field.generic.as_generic()?;
                let generic_bound = field.kind.get_generic().into_bound()?;
                let ts = if let Some(into_view) = field.into_view.as_ref() {
                    quote!(#ident: #generic_bound<#into_view>)
                } else {
                    quote!(#ident: #generic_bound)
                };
                Some(ts)
            })
            .chain(into_view_generics)
    }

    fn display_builder_fn(
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let left_generics = Self::get_string_bounded_left_generics(fields);
        let right_generics = Self::get_string_bounded_right_generics(fields, into_views);
        let builder_marker = fields.iter().map(|_| quote!(()));
        let into_views = into_views.iter().map(|_| quote!(()));

        quote! {
            pub fn display_builder<#(#left_generics,)*>(self) -> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #(#builder_marker,)*)> {
                #ident::builder().#locale_field(self.#locale_field).#into_view_field(core::marker::PhantomData)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dummy_impl(
        ident: &syn::Ident,
        dummy_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let left_generics = Self::bounded_generics(fields, into_views);

        let right_generics = fields
            .iter()
            .map(|field| {
                let generic_or_type = &field.generic;
                quote!(#generic_or_type)
            })
            .chain(into_views.iter().map(|i| quote!(#i)));

        let builder_marker = fields.iter().map(|_| quote!(()));

        let display_builder_fn = if cfg!(feature = "interpolate_display") {
            Self::display_builder_fn(
                ident,
                enum_ident,
                typed_builder_name,
                locale_field,
                into_view_field,
                fields,
                into_views,
            )
        } else {
            quote!()
        };

        quote! {
            impl #dummy_ident {
                pub const fn new(#locale_field: #enum_ident) -> Self {
                    Self {
                        #locale_field
                    }
                }

                pub fn builder<#(#left_generics,)*>(self) -> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #(#builder_marker,)*)> {
                    #ident::builder().#locale_field(self.#locale_field).#into_view_field(core::marker::PhantomData)
                }

                #display_builder_fn
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
            .filter_map(|field| field.generic.as_generic())
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
            .filter_map(|field| field.generic.as_generic())
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

    #[allow(clippy::too_many_arguments)]
    fn display_impl(
        key: &Key,
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
        into_views: &[&syn::Ident],
    ) -> TokenStream {
        let left_generics = Self::get_string_bounded_left_generics(fields);
        let right_generics = Self::get_string_bounded_right_generics(fields, into_views);

        let fields_key = fields.iter().map(|f| f.kind);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field, .. } = self;);

        let plural = fields
            .iter()
            .any(|field| matches!(field.kind, InterpolateKey::Count(_)));

        let var_count =
            plural.then(|| quote!(let var_count = core::clone::Clone::clone(&plural_count);));

        let locales_impls =
            Self::create_locale_string_impl(key, enum_ident, locales, default_match);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> ::core::fmt::Display for #ident<#(#right_generics,)*> {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                    #destructure
                    #var_count
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
            .filter_map(|field| {
                let ident = &field.generic;
                let generic = field.kind.get_generic().into_bound()?;
                let ts = if let Some(into_view) = field.into_view.as_ref() {
                    quote!(#ident: #generic<#into_view>)
                } else {
                    quote!(#ident: #generic)
                };
                Some(ts)
            })
            .chain(into_view_generics);

        let right_generics = fields
            .iter()
            .filter_map(|field| field.generic.as_generic())
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

        let plural = fields
            .iter()
            .any(|field| matches!(field.kind, InterpolateKey::Count(_)));

        let var_count =
            plural.then(|| quote!(let var_count = core::clone::Clone::clone(&plural_count);));

        let locales_impls = Self::create_locale_impl(key, enum_ident, locales, default_match);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> leptos::IntoView for #ident<#(#right_generics,)*> {
                fn into_view(self) -> leptos::View {
                    #destructure
                    #var_count
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
