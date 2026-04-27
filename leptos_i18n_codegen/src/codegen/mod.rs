use leptos_i18n_parser::error::{Error, Result};
use leptos_i18n_parser::extraction::{LocalesOrNamespaces, ParsedLocales};
use leptos_i18n_parser::options::LocaleName;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

mod builders;
mod docs;
mod keys;
mod locales;
mod namespaces;
mod values;

use crate::CodegenOptions;
use crate::codegen::builders::{gen_builder_module, infos::BuildersInfos};
use crate::codegen::docs::gen_keys_doc;
use crate::codegen::locales::gen_locales;
use crate::codegen::namespaces::gen_namespaces;

pub fn gen_code(parsed_values: &ParsedLocales, options: CodegenOptions) -> Result<TokenStream> {
    let default_crate_path = syn::Path::from(syn::Ident::new("leptos_i18n", Span::call_site()));
    let crate_path = options.crate_path.as_ref().unwrap_or(&default_crate_path);

    let ParsedLocales {
        cfg,
        values,
        builders,
        diag: _,
    } = parsed_values;

    if cfg!(all(feature = "csr", feature = "dynamic_load")) && options.translations_uri.is_none() {
        return Err(Error::MissingTranslationsURI.into());
    }

    let enum_ident = syn::Ident::new("Locale", Span::call_site());
    let keys_ident = syn::Ident::new("__I18nKeys", Span::call_site());
    let translation_unit_enum_ident =
        syn::Ident::new("__I18nTranslationUnitsId", Span::call_site());

    let locale_enum = gen_enum(
        values,
        &keys_ident,
        &enum_ident,
        &translation_unit_enum_ident,
        &cfg.locales,
        &options,
    )?;

    let markers_field = format_ident!("_into_view_markers__");

    let (builders_module, builders_infos) =
        gen_builder_module(builders, &enum_ident, markers_field, options.gen_docs);

    let keys_impls = gen_keys_impls(
        values,
        &keys_ident,
        &enum_ident,
        &translation_unit_enum_ident,
        &builders_infos,
        &options,
    );

    let mut macros_reexport = vec![
        quote!(t),
        quote!(td),
        quote!(tu),
        quote!(use_i18n_scoped),
        quote!(scope_i18n),
        quote!(scope_locale),
        quote!(define_scope),
        quote!(t_string),
        quote!(tu_string),
        quote!(t_display),
        quote!(tu_display),
        quote!(td_string),
        quote!(td_display),
    ];

    let providers = if cfg!(feature = "islands") {
        macros_reexport.push(quote!(ti));
        quote! {
            use leptos::children::Children;
            use leptos::prelude::RenderHtml;

            /// Create and provide a i18n context for all children components, directly accessible with `use_i18n`.
            #[__l_i18n_crate::reexports::leptos::island]
            #[allow(non_snake_case)]
            pub fn I18nContextProvider(
                /// If the "lang" attribute should be set on the root `<html>` element. (default to true)
                #[prop(optional)]
                set_lang_attr_on_html: Option<bool>,
                /// If the "dir" attribute should be set on the root `<html>` element. (default to true)
                #[prop(optional)]
                set_dir_attr_on_html: Option<bool>,
                /// Enable the use of a cookie to save the choosen locale (default to true).
                /// Does nothing without the "cookie" feature
                #[prop(optional)]
                enable_cookie: Option<bool>,
                /// Specify a name for the cookie, default to the library default.
                #[prop(optional, into)]
                cookie_name: Option<Cow<'static, str>>,
                children: Children
            ) -> impl IntoView {
                __l_i18n_crate::context::provide_i18n_context_component_island::<#enum_ident>(
                    set_lang_attr_on_html,
                    set_dir_attr_on_html,
                    enable_cookie,
                    cookie_name,
                    children
                )
            }

            /// Create and provide a i18n subcontext for all children components, directly accessible with `use_i18n`.
            #[__l_i18n_crate::reexports::leptos::island]
            #[allow(non_snake_case)]
            pub fn I18nSubContextProvider(
                children: Children,
                /// The initial locale for this subcontext.
                /// Default to the locale set in the cookie if set and some,
                /// if not use the parent context locale.
                /// if no parent context, use the default locale.
                #[prop(optional)]
                initial_locale: Option<#enum_ident>,
                /// If set save the locale in a cookie of the given name (does nothing without the `cookie` feature).
                #[prop(optional, into)]
                cookie_name: Option<Cow<'static, str>>,
            ) -> impl IntoView {
                __l_i18n_crate::context::i18n_sub_context_provider_island::<#enum_ident>(
                    children,
                    initial_locale,
                    cookie_name,
                )
            }
        }
    } else {
        quote! {
            use leptos::prelude::TypedChildren;

            /// Create and provide a i18n context for all children components, directly accessible with `use_i18n`.
            #[__l_i18n_crate::reexports::leptos::component]
            #[allow(non_snake_case)]
            pub fn I18nContextProvider<Chil: IntoView + 'static>(
                /// If the "lang" attribute should be set on the root `<html>` element. (default to true)
                #[prop(optional)]
                set_lang_attr_on_html: Option<bool>,
                /// If the "dir" attribute should be set on the root `<html>` element. (default to true)
                #[prop(optional)]
                set_dir_attr_on_html: Option<bool>,
                /// Enable the use of a cookie to save the choosen locale (default to true).
                /// Does nothing without the "cookie" feature
                #[prop(optional)]
                enable_cookie: Option<bool>,
                /// Specify a name for the cookie, default to the library default.
                #[prop(optional, into)]
                cookie_name: Option<Cow<'static, str>>,
                /// Options for the cookie, see `leptos_use::UseCookieOptions`.
                #[prop(optional)]
                cookie_options: Option<CookieOptions<#enum_ident>>,
                /// Options for getting the Accept-Language header, see `leptos_use::UseLocalesOptions`.
                #[prop(optional)]
                ssr_lang_header_getter: Option<UseLocalesOptions>,
                children: TypedChildren<Chil>
            ) -> impl IntoView {
                __l_i18n_crate::context::provide_i18n_context_component::<#enum_ident, Chil>(
                    set_lang_attr_on_html,
                    set_dir_attr_on_html,
                    enable_cookie,
                    cookie_name,
                    cookie_options,
                    ssr_lang_header_getter,
                    children
                )
            }

            /// Create and provide a subcontext for all children components, directly accessible with `use_i18n`.
            #[__l_i18n_crate::reexports::leptos::component]
            #[allow(non_snake_case)]
            pub fn I18nSubContextProvider<Chil: IntoView + 'static>(
                children: TypedChildren<Chil>,
                /// The initial locale for this subcontext.
                /// Default to the locale set in the cookie if set and some,
                /// if not use the parent context locale.
                /// if no parent context, use the default locale.
                #[prop(optional, into)]
                initial_locale: Option<Signal<#enum_ident>>,
                /// If set save the locale in a cookie of the given name (does nothing without the `cookie` feature).
                #[prop(optional, into)]
                cookie_name: Option<Cow<'static, str>>,
                /// Options for the cookie, see `leptos_use::UseCookieOptions`.
                #[prop(optional)]
                cookie_options: Option<CookieOptions<#enum_ident>>,
                /// Options for getting the Accept-Language header, see `leptos_use::UseLocalesOptions`.
                #[prop(optional)]
                ssr_lang_header_getter: Option<UseLocalesOptions>,
            ) -> impl IntoView {
                __l_i18n_crate::context::i18n_sub_context_provider_inner::<#enum_ident, Chil>(
                    children,
                    initial_locale,
                    cookie_name,
                    cookie_options,
                    ssr_lang_header_getter
                )
            }
        }
    };

    let macros_reexport = quote!(pub use #crate_path::{#(#macros_reexport,)*};);

    let top_level_attributes = options.top_level_attributes.as_ref();

    Ok(quote! {
        pub mod i18n {
            #![allow(unused_braces)]
            #![allow(clippy::type_complexity)]
            #![allow(clippy::let_and_return)]
            #![allow(clippy::unit_arg)]
            #![allow(non_camel_case_types)]
            #![allow(non_snake_case)]
            #top_level_attributes

            use #crate_path as __l_i18n_crate;

            #locale_enum

            #builders_module

            pub mod keys {
                #[allow(unused)]
                use super::{#enum_ident, __l_i18n_crate, __builders};

                #keys_impls
            }

            mod utils {
                use super::{__l_i18n_crate, #enum_ident};
                #[inline]
                #[track_caller]
                pub fn use_i18n() -> __l_i18n_crate::I18nContext<#enum_ident> {
                    use_i18n_scoped()
                }

                #[inline]
                #[track_caller]
                pub fn use_i18n_scoped<S: __l_i18n_crate::Scope<BaseLocale = #enum_ident>>() -> __l_i18n_crate::I18nContext<S> {
                    __l_i18n_crate::use_i18n_context()
                }
            }


            mod providers {
                use super::{__l_i18n_crate, #enum_ident};
                use __l_i18n_crate::reexports::leptos;
                #[allow(unused_imports)]
                use leptos::prelude::{IntoView, Signal};
                use std::borrow::Cow;
                #[allow(unused_imports)]
                use __l_i18n_crate::context::{CookieOptions, UseLocalesOptions};

                #providers
            }

            pub use providers::{I18nContextProvider, I18nSubContextProvider};
            pub use utils::{use_i18n, use_i18n_scoped};
            pub use __l_i18n_crate::Locale as I18nLocaleTrait;

            #macros_reexport
        }
    })
}

pub fn gen_enum(
    values: &LocalesOrNamespaces,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    locales: &[LocaleName],
    codegen_options: &CodegenOptions,
) -> Result<TokenStream> {
    let as_str_match_arms = locales
        .iter()
        .map(|locale| &locale.key)
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#enum_ident::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locales
        .iter()
        .map(|locale| &locale.key)
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#locale => Ok(#enum_ident::#variant)))
        .collect::<Vec<_>>();

    let constant_names_ident = locales
        .iter()
        .map(|locale| &locale.key)
        .map(|key| {
            (
                key,
                format_ident!("{}_LANGID", key.name.to_uppercase().replace('-', "_")),
            )
        })
        .collect::<Vec<_>>();

    let static_icu_locales = constant_names_ident
        .iter()
        .map(|(key, ident)| {
            let locale = &key.name;
            quote!(static #ident: std::sync::LazyLock<__l_i18n_crate::reexports::icu::locid::Locale> = std::sync::LazyLock::new(|| #locale.parse().expect("Valid locale"));)
        })
        .collect::<Vec<_>>();

    let as_icu_locale_match_arms = constant_names_ident
        .iter()
        .map(|(variant, constant)| quote!(#enum_ident::#variant => &#constant))
        .collect::<Vec<_>>();

    let server_fn_mod = if cfg!(all(feature = "dynamic_load", not(feature = "csr"))) {
        quote! {
            mod server_fn {
                #[allow(unused_imports)]
                use super::{__l_i18n_crate, #enum_ident, keys::#keys_ident, keys::#translation_unit_enum_ident};
                use __l_i18n_crate::reexports::leptos::server_fn;

                #[__l_i18n_crate::reexports::leptos::server(I18nRequestTranslationsServerFn)]
                pub async fn i18n_request_translations(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, server_fn::ServerFnError> {
                    let strings = #keys_ident::__i18n_request_translations__(locale, translations_id);
                    let wrapped = __l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput::new(strings);
                    Ok(wrapped)
                }
            }
        }
    } else if cfg!(all(feature = "dynamic_load", feature = "csr")) {
        quote! {
            mod server_fn {
                #[allow(unused_imports)]
                use super::{__l_i18n_crate, #enum_ident, keys::#keys_ident, keys::#translation_unit_enum_ident};
                use __l_i18n_crate::reexports::leptos::server_fn::ServerFnError;

                pub async fn i18n_request_translations(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, ServerFnError> {
                    #keys_ident::__i18n_request_translations__(locale, translations_id).await
                }
            }
        }
    } else {
        quote!()
    };

    let server_fn_type = if cfg!(all(feature = "dynamic_load", not(feature = "csr"))) {
        quote!(
            type ServerFn = server_fn::I18nRequestTranslationsServerFn;
        )
    } else {
        quote!()
    };

    let request_translations = if cfg!(feature = "dynamic_load") {
        quote! {
            fn request_translations(
                self,
                translations_id: Self::TranslationUnitId,
            ) -> impl std::future::Future<Output = Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, __l_i18n_crate::reexports::leptos::server_fn::ServerFnError>> + Send + Sync + 'static {
                server_fn::i18n_request_translations(self, translations_id)
            }
        }
    } else {
        quote!()
    };

    let init_translations = if cfg!(all(feature = "dynamic_load", feature = "hydrate")) {
        quote! {
            fn init_translations(self, translations_id: Self::TranslationUnitId, values: Vec<Box<str>>) {
                keys::#keys_ident::__init_translations__(self, translations_id, values);
            }
        }
    } else {
        quote!()
    };
    let ld = icu_locale::LocaleDirectionality::new_common();

    let direction_match_arms = locales.iter().map(|locale_name| {
        let locale = &locale_name.key;
        let dir = match ld.get(&locale_name.loc_id.id) {
            Some(icu_locale::Direction::LeftToRight) => quote!(LeftToRight),
            Some(icu_locale::Direction::RightToLeft) => quote!(RightToLeft),
            _ => quote!(Auto),
        };

        quote! {
            #enum_ident::#locale => __l_i18n_crate::Direction::#dir
        }
    });

    let docs = if codegen_options.gen_docs {
        let mut docs = String::from("## Supported locales:\n");
        for (i, key) in locales.iter().enumerate() {
            use core::fmt::Write;
            if i == 0 {
                writeln!(&mut docs, "- `{}` (default)", key).unwrap();
            } else {
                writeln!(&mut docs, "- `{}`", key).unwrap();
            }
        }

        match values {
            LocalesOrNamespaces::Namespaces(namespaces) => {
                use core::fmt::Write;
                writeln!(&mut docs, "\n## Namespaces :").unwrap();
                for ns in namespaces {
                    writeln!(&mut docs, "- `{}`", ns.name).unwrap();
                }
            }
            LocalesOrNamespaces::Locales(locales) => {
                gen_keys_doc(&mut docs, &locales.keys).unwrap();
            }
        };

        quote! {
            #[doc = #docs]
        }
    } else {
        quote! {}
    };

    let locales_variants = locales.iter().map(|l| &l.key).collect::<Vec<_>>();

    let ts = quote! {
        #docs
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default, PartialOrd, Ord)]
        #[allow(non_camel_case_types)]
        pub enum #enum_ident {
            #[default]
            #(#locales_variants,)*
        }

        impl __l_i18n_crate::reexports::serde::Serialize for #enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __l_i18n_crate::reexports::serde::Serializer,
            {
                __l_i18n_crate::reexports::serde::Serialize::serialize(__l_i18n_crate::Locale::as_str(*self), serializer)
            }
        }

        impl<'de> __l_i18n_crate::reexports::serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<#enum_ident, D::Error>
            where
                D: __l_i18n_crate::reexports::serde::de::Deserializer<'de>,
            {
                __l_i18n_crate::reexports::serde::de::Deserializer::deserialize_str(deserializer, __l_i18n_crate::__private::LocaleVisitor::<#enum_ident>::new())
            }
        }

        impl #enum_ident {
            pub const fn get_keys_const() -> keys::#keys_ident {
                keys::#keys_ident
            }
        }

        impl __l_i18n_crate::Scope for #enum_ident {
            type BaseLocale = Self;
            type Keys = keys::#keys_ident;

            fn get_keys() -> Self::Keys {
                Self::get_keys_const()
            }
        }

        impl __l_i18n_crate::locale_traits::BaseLocale for #enum_ident {
            const ALL_VARIANTS: &'static [Self] = &[#(#enum_ident::#locales_variants,)*];

            type TranslationUnitId = keys::#translation_unit_enum_ident;

            #server_fn_type

            fn as_str(self) -> &'static str {
                let s = match self {
                    #(
                        #as_str_match_arms,
                    )*
                };
                __l_i18n_crate::__private::intern(s)
            }

            fn as_icu_locale(self) -> &'static __l_i18n_crate::reexports::icu::locid::Locale {
                #(
                    #static_icu_locales
                )*
                match self {
                    #(
                        #as_icu_locale_match_arms,
                    )*
                }
            }

            fn direction(self) -> __l_i18n_crate::Direction {
                match self {
                    #(
                        #direction_match_arms,
                    )*
                }
            }

            #request_translations

            #init_translations
        }

        impl core::str::FromStr for #enum_ident {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim() {
                    #(#from_str_match_arms,)*
                    _ => Err(())
                }
            }
        }

        impl core::convert::AsRef<__l_i18n_crate::reexports::icu::locid::LanguageIdentifier> for #enum_ident {
            fn as_ref(&self) -> &__l_i18n_crate::reexports::icu::locid::LanguageIdentifier {
                __l_i18n_crate::Locale::as_langid(*self)
            }
        }

        impl core::convert::AsRef<__l_i18n_crate::reexports::icu::locid::Locale> for #enum_ident {
            fn as_ref(&self) -> &__l_i18n_crate::reexports::icu::locid::Locale {
                __l_i18n_crate::Locale::as_icu_locale(*self)
            }
        }

        impl core::convert::AsRef<str> for #enum_ident {
            fn as_ref(&self) -> &str {
                __l_i18n_crate::Locale::as_str(*self)
            }
        }

        impl core::convert::AsRef<Self> for #enum_ident {
            fn as_ref(&self) -> &Self {
                self
            }
        }

        impl core::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(__l_i18n_crate::Locale::as_str(*self), f)
            }
        }

        #server_fn_mod
    };

    Ok(ts)
}

fn gen_keys_impls(
    values: &LocalesOrNamespaces,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    builders: &BuildersInfos,
    options: &CodegenOptions,
) -> TokenStream {
    match values {
        LocalesOrNamespaces::Namespaces(namespaces) => gen_namespaces(
            namespaces,
            keys_ident,
            enum_ident,
            translation_unit_enum_ident,
            builders,
            options,
        ),
        LocalesOrNamespaces::Locales(locales) => gen_locales(
            locales,
            keys_ident,
            enum_ident,
            translation_unit_enum_ident,
            None,
            builders,
            options,
        ),
    }
}
