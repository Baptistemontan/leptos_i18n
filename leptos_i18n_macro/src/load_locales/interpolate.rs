use leptos_i18n_parser::parse_locales::locale::InterpolationKeys;
use leptos_i18n_parser::parse_locales::locale::Locale;
use leptos_i18n_parser::utils::Key;
use leptos_i18n_parser::utils::KeyPath;
use leptos_i18n_parser::utils::UnwrapAt;
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use super::parsed_value;
// use super::parsed_value::InterpolationKeys;
// use super::parsed_value::RangeOrPlural;
use super::parsed_value::TRANSLATIONS_KEY;
use super::ranges::RangeType;
use super::strings_accessor_method_name;
use crate::utils::formatter::Formatter;
use crate::utils::EitherOfWrapper;

pub const LOCALE_FIELD_KEY: &str = "_locale";

#[derive(Clone)]
enum EitherIter<A, B> {
    Iter1(A),
    Iter2(B),
}

impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for EitherIter<A, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Iter1(iter) => iter.next(),
            EitherIter::Iter2(iter) => iter.next(),
        }
    }
}

pub struct Interpolation {
    pub ident: syn::Ident,
    pub imp: TokenStream,
}

#[derive(Debug, Copy, Clone)]
enum RangeOrPlural {
    Range(RangeType),
    Plural,
}

impl From<leptos_i18n_parser::parse_locales::locale::RangeOrPlural> for RangeOrPlural {
    fn from(value: leptos_i18n_parser::parse_locales::locale::RangeOrPlural) -> Self {
        match value {
            leptos_i18n_parser::parse_locales::locale::RangeOrPlural::Range(range_type) => {
                RangeOrPlural::Range(range_type.into())
            }
            leptos_i18n_parser::parse_locales::locale::RangeOrPlural::Plural => {
                RangeOrPlural::Plural
            }
        }
    }
}

impl RangeOrPlural {
    pub fn to_bound(self) -> TokenStream {
        match self {
            RangeOrPlural::Range(range_type) => {
                quote!(l_i18n_crate::__private::InterpolateRangeCount<#range_type>)
            }
            RangeOrPlural::Plural => {
                quote!(l_i18n_crate::__private::InterpolatePluralCount)
            }
        }
    }
}

enum VarOrComp {
    Var {
        formatters: Vec<Formatter>,
        plural: Option<RangeOrPlural>,
    },
    Comp {
        into_view: syn::Ident,
    },
}

struct Field {
    key: Key,
    generic: syn::Ident,
    var_or_comp: VarOrComp,
}

impl Field {
    fn get_var_generics(
        generic: &syn::Ident,
        formatters: &[Formatter],
        plural: Option<RangeOrPlural>,
    ) -> TokenStream {
        let bounds = formatters.iter().copied().map(Formatter::to_bound);
        let plural_bound = plural.map(RangeOrPlural::to_bound);
        let bounds = bounds.chain(plural_bound);

        quote!(#generic: 'static + ::core::clone::Clone #(+ #bounds)*)
    }

    fn get_string_var_generics(
        generic: &syn::Ident,
        formatters: &[Formatter],
        range: Option<RangeOrPlural>,
    ) -> Option<TokenStream> {
        match range {
            None => {
                let bounds = formatters.iter().copied().map(Formatter::to_string_bound);
                Some(quote!(#generic: #(#bounds +)*))
            }
            Some(RangeOrPlural::Range(_)) => None,
            Some(RangeOrPlural::Plural) => {
                let bounds = formatters.iter().copied().map(Formatter::to_string_bound);
                Some(
                    quote!(#generic: #(#bounds +)* Clone + Into<l_i18n_crate::reexports::icu::plurals::PluralOperands>),
                )
            }
        }
    }

    pub fn as_bounded_generic(&self) -> impl Iterator<Item = TokenStream> {
        let generic = &self.generic;
        match &self.var_or_comp {
            VarOrComp::Var {
                formatters,
                plural: range,
            } => {
                let ts = Self::get_var_generics(generic, formatters, *range);
                EitherIter::Iter1(std::iter::once(ts))
            }
            VarOrComp::Comp { into_view } => {
                let ts = [
                    quote!(#generic: l_i18n_crate::__private::InterpolateComp<#into_view>),
                    quote!(#into_view: l_i18n_crate::reexports::leptos::IntoView + Clone),
                ];
                EitherIter::Iter2(ts.into_iter())
            }
        }
    }

    pub fn as_string_bounded_generic(&self) -> Option<TokenStream> {
        let generic = &self.generic;
        match &self.var_or_comp {
            VarOrComp::Var {
                formatters,
                plural: range,
            } => Self::get_string_var_generics(generic, formatters, *range),
            VarOrComp::Comp { .. } => {
                Some(quote!(#generic: l_i18n_crate::display::DisplayComponent))
            }
        }
    }

    pub fn as_right_generics(&self) -> impl Iterator<Item = &syn::Ident> {
        let generic = std::iter::once(&self.generic);
        match &self.var_or_comp {
            VarOrComp::Var { .. } => EitherIter::Iter1(generic),
            VarOrComp::Comp { into_view } => EitherIter::Iter2(generic.chain(Some(into_view))),
        }
    }

    pub fn as_string_right_generics(&self) -> impl Iterator<Item = TokenStream> {
        let generic = std::iter::once(self.generic.to_token_stream());
        match &self.var_or_comp {
            VarOrComp::Var {
                plural: Some(RangeOrPlural::Range(range_type)),
                ..
            } => EitherIter::Iter1(std::iter::once(quote!(#range_type))),
            VarOrComp::Var { .. } => EitherIter::Iter1(generic),
            VarOrComp::Comp { .. } => EitherIter::Iter2(generic.chain(Some(quote!(())))),
        }
    }

    pub fn as_string_builder_marker(&self) -> TokenStream {
        match &self.var_or_comp {
            VarOrComp::Var {
                plural: Some(RangeOrPlural::Range(range_type)),
                ..
            } => quote!(#range_type),
            _ => self.generic.to_token_stream(),
        }
    }

    pub fn as_struct_field(&self) -> TokenStream {
        let Field { key, generic, .. } = self;
        quote!(#key: #generic)
    }

    pub fn as_into_view_generic(&self) -> Option<&syn::Ident> {
        match &self.var_or_comp {
            VarOrComp::Var { .. } => None,
            VarOrComp::Comp { into_view } => Some(into_view),
        }
    }
}

impl Interpolation {
    fn make_fields(keys: &InterpolationKeys) -> Vec<Field> {
        let vars = keys.iter_vars().map(|(key, infos)| {
            let mut formatters = infos
                .formatters
                .iter()
                .copied()
                .map(Into::into)
                .collect::<Vec<_>>();
            formatters.sort_unstable();
            let var_or_comp = VarOrComp::Var {
                formatters,
                plural: infos.range_count.map(Into::into),
            };
            let generic = format_ident!("__{}__", key);
            Field {
                key,
                var_or_comp,
                generic,
            }
        });

        let comps = keys.iter_comps().map(|key| {
            let into_view = format_ident!("__into_view_{}__", key);
            let var_or_comp = VarOrComp::Comp { into_view };
            let generic = format_ident!("__{}__", key);
            Field {
                key,
                var_or_comp,
                generic,
            }
        });

        let mut fields: Vec<_> = vars.chain(comps).collect();

        fields.sort_by(|a, b| a.key.cmp(&b.key));

        fields
    }

    pub fn new(
        key: &Key,
        enum_ident: &syn::Ident,
        keys: &InterpolationKeys,
        locales: &[Locale],
        key_path: &KeyPath,
        locale_type_ident: &syn::Ident,
        interpolate_display: bool,
    ) -> Self {
        let builder_name = format!("{}_builder", key);

        let ident = syn::Ident::new(&builder_name, Span::call_site());

        let dummy_ident = format_ident!("{}_dummy", ident);

        let locale_field = Key::new(LOCALE_FIELD_KEY).unwrap_at("LOCALE_FIELD_KEY");
        let into_view_field = Key::new("_into_views_marker").unwrap_at("Interpolation::new_1");

        let typed_builder_name = format_ident!("{}Builder", ident);
        let display_struct_ident = format_ident!("{}Display", ident);

        let fields = Self::make_fields(keys);

        let type_def = Self::create_types(
            &ident,
            &dummy_ident,
            enum_ident,
            &locale_field,
            &into_view_field,
            &fields,
        );

        let dummy_impl = Self::dummy_impl(
            &ident,
            &dummy_ident,
            enum_ident,
            &typed_builder_name,
            &locale_field,
            &into_view_field,
            &fields,
            interpolate_display,
        );

        let into_view_impl = Self::into_view_impl(
            key,
            &ident,
            enum_ident,
            &locale_field,
            &fields,
            locales,
            key_path,
            locale_type_ident,
        );

        let debug_impl = Self::debug_impl(&builder_name, &ident, &fields);

        let (display_impl, builder_display) = if interpolate_display {
            let display_impl = Self::display_impl(
                key,
                &ident,
                &display_struct_ident,
                enum_ident,
                &locale_field,
                &fields,
                locales,
                locale_type_ident,
            );
            let builder_display = Self::builder_string_build_fns(
                enum_ident,
                &typed_builder_name,
                &display_struct_ident,
                &fields,
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

    fn builder_string_build_fns(
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        display_struct_ident: &syn::Ident,
        fields: &[Field],
    ) -> TokenStream {
        let left_generics = fields.iter().filter_map(Field::as_string_bounded_generic);

        let right_generics = fields.iter().flat_map(Field::as_string_right_generics);
        let marker = fields.iter().map(Field::as_string_builder_marker);

        let into_views = fields
            .iter()
            .filter_map(Field::as_into_view_generic)
            .map(|_| quote!(()));

        let fns = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            quote! {
                #[inline]
                pub async fn build_display(self) -> impl std::fmt::Display {
                    let inner = self.build();
                    #display_struct_ident::new(inner).await
                }

                #[inline]
                pub async fn build_string(self) -> String {
                    self.build_display().await.to_string()
                }
            }
        } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
            quote! {
                #[inline]
                pub async fn build_display(self) -> impl std::fmt::Display {
                    let inner = self.build();
                    #display_struct_ident::new(inner)
                }

                #[inline]
                pub async fn build_string(self) -> String {
                    self.build_display().await.to_string()
                }
            }
        } else {
            quote! {
                #[inline]
                pub fn build_display(self) -> impl std::fmt::Display {
                    let inner = self.build();
                    #display_struct_ident::new(inner)
                }

                #[inline]
                pub fn build_string(self) -> String {
                    self.build_display().to_string()
                }
            }
        };

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #((#marker,),)*)> {
                #fns
            }
        }
    }

    fn display_builder_fn(
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
    ) -> TokenStream {
        let left_generics = fields.iter().filter_map(Field::as_string_bounded_generic);
        let right_generics = fields.iter().flat_map(Field::as_string_right_generics);
        let builder_marker = fields.iter().map(|_| quote!(()));
        let into_views = fields
            .iter()
            .filter_map(Field::as_into_view_generic)
            .map(|_| quote!(()));

        quote! {
            #[allow(non_camel_case_types)]
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
        interpolate_display: bool,
    ) -> TokenStream {
        let left_generics = fields.iter().flat_map(Field::as_bounded_generic);

        let right_generics = fields.iter().flat_map(Field::as_right_generics);

        let empty_builder_marker = fields.iter().map(|_| quote!(()));

        let display_builder_fn = if interpolate_display {
            Self::display_builder_fn(
                ident,
                enum_ident,
                typed_builder_name,
                locale_field,
                into_view_field,
                fields,
            )
        } else {
            quote!()
        };

        let into_views = fields.iter().filter_map(Field::as_into_view_generic);

        let string_builder_trait_impl = if interpolate_display {
            quote! {
                impl l_i18n_crate::__private::InterpolationStringBuilder for #dummy_ident {}
            }
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

                #[allow(non_camel_case_types)]
                pub fn builder<#(#left_generics,)*>(self) -> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #(#empty_builder_marker,)*)> {
                    #ident::builder().#locale_field(self.#locale_field).#into_view_field(core::marker::PhantomData)
                }

                #display_builder_fn
            }

            #string_builder_trait_impl
        }
    }

    fn create_types(
        ident: &syn::Ident,
        dummy_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
    ) -> TokenStream {
        let generics = fields.iter().flat_map(Field::as_right_generics);

        let into_views = fields.iter().filter_map(Field::as_into_view_generic);

        let fields = fields.iter().map(Field::as_struct_field);

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
            #[derive(l_i18n_crate::reexports::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = l_i18n_crate::reexports::typed_builder)]
            pub struct #ident<#(
                #[allow(non_camel_case_types)]
                #generics,
            )*> {
                #locale_field: #enum_ident,
                #into_views_marker,
                #(#fields,)*
            }
        }
    }

    fn debug_impl(builder_name: &str, ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let left_generics = fields.iter().flat_map(Field::as_right_generics);

        let right_generics = fields.iter().flat_map(Field::as_right_generics);

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
        display_struct_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        locale_type_ident: &syn::Ident,
    ) -> TokenStream {
        let left_generics = fields.iter().filter_map(Field::as_string_bounded_generic);

        let right_generics = fields.iter().flat_map(Field::as_string_right_generics);

        let raw_generics = fields
            .iter()
            .flat_map(Field::as_right_generics)
            .collect::<Vec<_>>();

        let fields_key = fields.iter().map(|f| &f.key);

        let destructure = quote!(let #ident { #(#fields_key,)* #locale_field, .. } = &self.1;);

        let translations_holder_enum_ident = format_ident!("{}Enum", display_struct_ident);
        let locales_impls = Self::create_locale_string_impl(
            key,
            &translations_holder_enum_ident,
            locales,
            locale_type_ident,
        );

        let str_name = display_struct_ident.to_string();

        let translations_holder_enum = if cfg!(all(feature = "dynamic_load", not(feature = "ssr")))
        {
            let translations_holder_enum_ident_variants = locales.iter().map(|locale| {
                let top_locale = &locale.top_locale_name.ident;
                quote! {
                    #top_locale(&'static str)
                }
            });

            quote! {
                #[derive(Clone, Copy)]
                #[allow(non_camel_case_types, non_snake_case)]
                enum #translations_holder_enum_ident {
                    #(
                        #translations_holder_enum_ident_variants,
                    )*
                }
            }
        } else {
            quote! {
                #[allow(non_camel_case_types, non_snake_case)]
                type #translations_holder_enum_ident = #enum_ident;
            }
        };

        let new_fn = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            let match_arms = locales.iter().map(|locale| {
                let top_locale = &locale.top_locale_name.ident;
                let string_accessor = strings_accessor_method_name(locale);
                quote! {
                    #enum_ident::#top_locale => {
                        let translations: &'static str = super::#locale_type_ident::#string_accessor().await;
                        #translations_holder_enum_ident::#top_locale(translations)
                    }
                }
            });
            quote! {
                pub async fn new(builder: #ident<#(#raw_generics,)*>) -> Self {
                    let translations = match builder.#locale_field {
                        #(
                            #match_arms,
                        )*
                    };
                    #display_struct_ident(translations, builder)
                }
            }
        } else {
            quote! {
                pub fn new(builder: #ident<#(#raw_generics,)*>) -> Self {
                    #display_struct_ident(builder.#locale_field, builder)
                }
            }
        };

        quote! {

            #translations_holder_enum

            #[allow(non_camel_case_types, non_snake_case)]
            struct #display_struct_ident<#(#raw_generics,)*>(#translations_holder_enum_ident, #ident<#(#raw_generics,)*>);

            #[allow(non_camel_case_types)]
            impl<#(#raw_generics,)*> core::fmt::Debug for #display_struct_ident<#(#raw_generics,)*> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(#str_name).finish()
                }
            }

            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> ::core::fmt::Display for #display_struct_ident<#(#right_generics,)*> {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                    #destructure
                    match self.0 {
                        #(
                            #locales_impls,
                        )*
                    }
                }
            }

            #[allow(non_camel_case_types)]
            impl<#(#raw_generics,)*> #display_struct_ident<#(#raw_generics,)*> {
                #new_fn
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
        key_path: &KeyPath,
        locale_type_ident: &syn::Ident,
    ) -> TokenStream {
        let left_generics = fields.iter().flat_map(Field::as_bounded_generic);

        let right_generics = fields.iter().flat_map(Field::as_right_generics);

        if cfg!(feature = "show_keys_only") {
            let key = key_path.to_string_with_key(key);
            return quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics,)*> #ident<#(#right_generics,)*> {
                    pub fn into_view(self) -> impl l_i18n_crate::reexports::leptos::IntoView + Clone {
                        let _ = self;
                        #key
                    }
                }
            };
        }

        let fields_key = fields.iter().map(|f| &f.key);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field, .. } = self;);

        let locales_impls = Self::create_locale_impl(key, enum_ident, locales, locale_type_ident);
        if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics,)*> #ident<#(#right_generics,)*> {
                    pub async fn into_view(self) -> impl l_i18n_crate::reexports::leptos::IntoView + Clone {
                        #destructure
                        match #locale_field {
                            #(
                                #locales_impls,
                            )*
                        }
                    }
                }
            }
        } else {
            quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics,)*> #ident<#(#right_generics,)*> {
                    pub fn into_view(self) -> impl l_i18n_crate::reexports::leptos::IntoView + Clone {
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
    }

    fn create_locale_impl<'a>(
        key: &'a Key,
        enum_ident: &'a syn::Ident,
        locales: &'a [Locale],
        locale_type_ident: &'a syn::Ident,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let either_wrapper = EitherOfWrapper::new(locales.len());
        locales
            .iter()
            .enumerate()
            .rev()
            .map(move |(i, locale)| {
                let locale_key = &locale.top_locale_name;

                let value = locale
                    .keys
                    .get(key)
                    .unwrap_at("create_locale_impl_1");

                let value = parsed_value::to_token_stream(value);

                let wrapped_value = either_wrapper.wrap(i, value);

                let translations_key = Key::new(TRANSLATIONS_KEY).unwrap_at("TRANSLATIONS_KEY");

                let string_accessor = strings_accessor_method_name(locale);
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote!{
                        #enum_ident::#locale_key => {
                            let #translations_key: &'static str = super::#locale_type_ident::#string_accessor().await;
                            #wrapped_value
                        }
                    }
                } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                    quote!{
                        #enum_ident::#locale_key => {
                            let #translations_key: &'static str = super::#locale_type_ident::#string_accessor();
                            #wrapped_value
                        }
                    }
                } else {
                    quote!{
                        #enum_ident::#locale_key => {
                            const #translations_key: &str = super::#locale_type_ident::#string_accessor();
                            #wrapped_value
                        }
                    }
                }
            })
    }

    fn create_locale_string_impl<'a>(
        key: &'a Key,
        enum_ident: &'a syn::Ident,
        locales: &'a [Locale],
        locale_type_ident: &'a syn::Ident,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        locales.iter().rev().map(move |locale| {
            let locale_key = &locale.top_locale_name;
            let value = locale
                .keys
                .get(key)
                .unwrap_at("create_locale_string_impl_1");

            let value = parsed_value::as_string_impl(value);

            let translations_key = Key::new(TRANSLATIONS_KEY).unwrap_at("TRANSLATIONS_KEY");

            let string_accessor = strings_accessor_method_name(locale);

            if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                quote!{
                    #enum_ident::#locale_key(#translations_key) => {
                        #value
                    }
                }
            } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                quote!{
                    #enum_ident::#locale_key => {
                        let #translations_key: &'static str = super::#locale_type_ident::#string_accessor();
                        #value
                    }
                }
            }else {
                quote!{
                    #enum_ident::#locale_key => {
                        const #translations_key: &str = super::#locale_type_ident::#string_accessor();
                        #value
                    }
                }
            }
        })
    }
}
