use std::{collections::BTreeMap, ops::Not};

pub mod declare_locales;
pub mod interpolate;
pub mod locale;
pub mod parsed_value;
pub mod plurals;
pub mod ranges;
pub mod tracking;
pub mod warning;

use icu_locid::LanguageIdentifier;
use interpolate::Interpolation;
use leptos_i18n_parser::{
    parse_locales::{
        cfg_file::ConfigFile, error::{Error, Result}, locale::{
            BuildersKeys, BuildersKeysInner, InterpolOrLit, Locale, LocaleValue,
            LocalesOrNamespaces, Namespace,
        }, parsed_value::ParsedValue, warning::Warnings, ForeignKeysPaths
    },
    utils::{
        key::{Key, KeyPath},
        UnwrapAt,
    },
};
use locale::LiteralType;
use parsed_value::TRANSLATIONS_KEY;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use warning::generate_warnings;

/// Steps:
///
/// 1: Locate and parse the manifest (`ConfigFile::new`)
/// 2: parse each locales/namespaces files (`LocalesOrNamespaces::new`)
/// 3: Resolve foreign keys (`ParsedValue::resolve_foreign_keys`)
/// 4: check the locales: (`Locale::check_locales`)
/// 4.1: get interpolations keys of the default, meaning all variables/components/ranges of the default locale (`Locale::make_builder_keys`)
/// 4.2: in the process reduce all values and check for default in the default locale
/// 4.3: then merge all other locales in the default locale keys, reducing all values in the process (`Locale::merge`)
/// 4.4: discard any surplus key and emit a warning
/// 5: generate code (and warnings)
pub fn load_locales() -> Result<TokenStream> {
    let (locales, cfg_file, foreign_keys, warnings, tracked_files) =
        leptos_i18n_parser::parse_locales::parse_locales_raw(false, None)?;

    let crate_path = syn::Path::from(syn::Ident::new("leptos_i18n", Span::call_site()));

    let interpolate_display = cfg!(feature = "interpolate_display");

    load_locales_inner(
        &crate_path,
        &cfg_file,
        locales,
        foreign_keys,
        warnings,
        Some(tracked_files),
        interpolate_display,
    )
}

fn load_locales_inner(
    crate_path: &syn::Path,
    cfg_file: &ConfigFile,
    locales: LocalesOrNamespaces,
    foreign_keys_paths: ForeignKeysPaths,
    warnings: Warnings,
    tracked_files: Option<Vec<String>>,
    interpolate_display: bool,
) -> Result<TokenStream> {
    if cfg!(all(feature = "csr", feature = "dynamic_load")) && cfg_file.translations_uri.is_none() {
        return Err(Error::MissingTranslationsURI.into());
    }

    let keys = leptos_i18n_parser::parse_locales::make_builder_keys(
        locales,
        cfg_file,
        foreign_keys_paths,
        &warnings,
        false,
    )?;

    let enum_ident = syn::Ident::new("Locale", Span::call_site());
    let keys_ident = syn::Ident::new("I18nKeys", Span::call_site());
    let translation_unit_enum_ident = syn::Ident::new("I18nTranslationUnitsId", Span::call_site());

    let locale_type = create_locale_type(
        &keys,
        &keys_ident,
        &enum_ident,
        &translation_unit_enum_ident,
        interpolate_display,
        cfg_file.translations_uri.as_deref()
    );
    let locale_enum = create_locales_enum(
        &enum_ident,
        &keys_ident,
        &translation_unit_enum_ident,
        &cfg_file.locales,
    )?;

    let warnings = generate_warnings(warnings);

    let file_tracking = tracking::generate_file_tracking(tracked_files);

    let mut macros_reexport = vec![
        quote!(t),
        quote!(td),
        quote!(tu),
        quote!(use_i18n_scoped),
        quote!(scope_i18n),
        quote!(scope_locale),
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
            #[l_i18n_crate::reexports::leptos::island]
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
                l_i18n_crate::context::provide_i18n_context_component_island::<#enum_ident>(
                    set_lang_attr_on_html,
                    set_dir_attr_on_html,
                    enable_cookie,
                    cookie_name,
                    children
                )
            }

            /// Create and provide a i18n subcontext for all children components, directly accessible with `use_i18n`.
            #[l_i18n_crate::reexports::leptos::island]
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
                l_i18n_crate::context::i18n_sub_context_provider_island::<#enum_ident>(
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
            #[l_i18n_crate::reexports::leptos::component]
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
                l_i18n_crate::context::provide_i18n_context_component::<#enum_ident, Chil>(
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
            #[l_i18n_crate::reexports::leptos::component]
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
                l_i18n_crate::context::i18n_sub_context_provider_inner::<#enum_ident, Chil>(
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

    Ok(quote! {
        pub mod i18n {
            use #crate_path as l_i18n_crate;

            #file_tracking

            #locale_enum

            #locale_type

            #[inline]
            #[track_caller]
            pub fn use_i18n() -> l_i18n_crate::I18nContext<#enum_ident> {
                l_i18n_crate::use_i18n_context()
            }

            #[deprecated(
                note = "It is now preferred to use the <I18nContextProvider> component"
            )]
            #[track_caller]
            pub fn provide_i18n_context() -> l_i18n_crate::I18nContext<#enum_ident> {
                l_i18n_crate::context::provide_i18n_context_with_options_inner(Default::default())
            }

            mod providers {
                use super::{l_i18n_crate, #enum_ident};
                use l_i18n_crate::reexports::leptos;
                use leptos::prelude::{IntoView, Signal};
                use std::borrow::Cow;
                use l_i18n_crate::context::{CookieOptions, UseLocalesOptions};

                #providers
            }

            pub use providers::{I18nContextProvider, I18nSubContextProvider};
            pub use l_i18n_crate::Locale as I18nLocaleTrait;

            #macros_reexport

            #warnings
        }
    })
}

fn create_locales_enum(
    enum_ident: &syn::Ident,
    keys_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    locales: &[Key],
) -> Result<TokenStream> {
    let as_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#enum_ident::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#locale => Ok(#enum_ident::#variant)))
        .collect::<Vec<_>>();

    let constant_names_ident = locales
        .iter()
        .map(|key| {
            (
                key,
                format_ident!("{}_LANGID", key.name.to_uppercase().replace('-', "_")),
            )
        })
        .collect::<Vec<_>>();

    let const_icu_locales = constant_names_ident
        .iter()
        .map(|(key, ident)| {
            let locale = &key.name;
            quote!(const #ident: &l_i18n_crate::reexports::icu::locid::Locale = &l_i18n_crate::reexports::icu::locid::locale!(#locale);)
        })
        .collect::<Vec<_>>();

    let as_icu_locale_match_arms = constant_names_ident
        .iter()
        .map(|(variant, constant)| quote!(#enum_ident::#variant => #constant))
        .collect::<Vec<_>>();

    let server_fn_mod = if cfg!(all(feature = "dynamic_load", not(feature = "csr"))) {
        quote! {
            mod server_fn {
                use super::{l_i18n_crate, #enum_ident, #keys_ident, #translation_unit_enum_ident};
                use l_i18n_crate::reexports::leptos::server_fn::ServerFnError;

                #[l_i18n_crate::reexports::leptos::server(I18nRequestTranslationsServerFn)]
                pub async fn i18n_request_translations(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, ServerFnError> {
                    let strings = #keys_ident::__i18n_request_translations__(locale, translations_id);
                    let wrapped = l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput::new(strings);
                    Ok(wrapped)
                }
            }
        }
    } else if cfg!(all(feature = "dynamic_load", feature = "csr")) {
        quote! {
            mod server_fn {
                use super::{l_i18n_crate, #enum_ident, #keys_ident, #translation_unit_enum_ident};
                use l_i18n_crate::reexports::leptos::server_fn::ServerFnError;

                pub async fn i18n_request_translations(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, ServerFnError> {
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
                translations_id: #translation_unit_enum_ident,
            ) -> impl std::future::Future<Output = Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, l_i18n_crate::reexports::leptos::server_fn::ServerFnError>> + Send + Sync + 'static {
                server_fn::i18n_request_translations(self, translations_id)
            }
        }
    } else {
        quote!()
    };

    let init_translations = if cfg!(all(feature = "dynamic_load", feature = "hydrate")) {
        quote! {
            fn init_translations(self, translations_id: Self::TranslationUnitId, values: Vec<Box<str>>) {
                #keys_ident::__init_translations__(self, translations_id, values);
            }
        }
    } else {
        quote!()
    };
    let ld = icu_locid_transform::LocaleDirectionality::new();

    let locids = locales
        .iter()
        .map(|locale| match locale.name.parse::<LanguageIdentifier>() {
            Ok(locid) => Ok((locale, locid)),
            Err(err) => Err(Error::InvalidLocale {
                locale: locale.name.clone(),
                err,
            }.into()),
        })
        .collect::<Result<Vec<_>>>()?;

    let direction_match_arms = locids.iter().map(|(locale, locid)| {
        let dir = match ld.get(locid) {
            Some(icu_locid_transform::Direction::LeftToRight) => quote!(LeftToRight),
            Some(icu_locid_transform::Direction::RightToLeft) => quote!(RightToLeft),
            _ => quote!(Auto),
        };

        quote! {
            #enum_ident::#locale => l_i18n_crate::Direction::#dir
        }
    });

    let ts = quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
        #[allow(non_camel_case_types)]
        pub enum #enum_ident {
            #[default]
            #(#locales,)*
        }

        impl l_i18n_crate::reexports::serde::Serialize for #enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: l_i18n_crate::reexports::serde::Serializer,
            {
                l_i18n_crate::reexports::serde::Serialize::serialize(l_i18n_crate::Locale::as_str(*self), serializer)
            }
        }

        impl<'de> l_i18n_crate::reexports::serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<#enum_ident, D::Error>
            where
                D: l_i18n_crate::reexports::serde::de::Deserializer<'de>,
            {
                l_i18n_crate::reexports::serde::de::Deserializer::deserialize_str(deserializer, l_i18n_crate::__private::LocaleVisitor::<#enum_ident>::new())
            }
        }

        impl #enum_ident {
            pub const fn get_keys_const(self) -> #keys_ident {
                #keys_ident::__new_internal(self)
            }
        }

        impl l_i18n_crate::Locale for #enum_ident {
            type Keys = #keys_ident;
            type TranslationUnitId = #translation_unit_enum_ident;
            #server_fn_type

            fn as_str(self) -> &'static str {
                let s = match self {
                    #(
                        #as_str_match_arms,
                    )*
                };
                l_i18n_crate::__private::intern(s)
            }

            fn as_icu_locale(self) -> &'static l_i18n_crate::reexports::icu::locid::Locale {
                #(
                    #const_icu_locales
                )*
                match self {
                    #(
                        #as_icu_locale_match_arms,
                    )*
                }
            }

            fn direction(self) -> l_i18n_crate::Direction {
                match self {
                    #(
                        #direction_match_arms,
                    )*
                }
            }

            fn get_all() -> &'static [Self] {
                &[#(#enum_ident::#locales,)*]
            }

            fn to_base_locale(self) -> Self {
                self
            }

            fn from_base_locale(locale: Self) -> Self {
                locale
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

        impl core::convert::AsRef<l_i18n_crate::reexports::icu::locid::LanguageIdentifier> for #enum_ident {
            fn as_ref(&self) -> &l_i18n_crate::reexports::icu::locid::LanguageIdentifier {
                l_i18n_crate::Locale::as_langid(*self)
            }
        }

        impl core::convert::AsRef<l_i18n_crate::reexports::icu::locid::Locale> for #enum_ident {
            fn as_ref(&self) -> &l_i18n_crate::reexports::icu::locid::Locale {
                l_i18n_crate::Locale::as_icu_locale(*self)
            }
        }

        impl core::convert::AsRef<str> for #enum_ident {
            fn as_ref(&self) -> &str {
                l_i18n_crate::Locale::as_str(*self)
            }
        }

        impl core::convert::AsRef<Self> for #enum_ident {
            fn as_ref(&self) -> &Self {
                self
            }
        }

        impl core::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(l_i18n_crate::Locale::as_str(*self), f)
            }
        }

        #server_fn_mod
    };
    Ok(ts)
}

struct Subkeys<'a> {
    original_key: Key,
    key: syn::Ident,
    mod_key: syn::Ident,
    locales: &'a [Locale],
    keys: &'a BuildersKeysInner,
}

impl<'a> Subkeys<'a> {
    pub fn new(key: Key, locales: &'a [Locale], keys: &'a BuildersKeysInner) -> Self {
        let mod_key = format_ident!("sk_{}", key);
        let new_key = format_ident!("{}_subkeys", key);
        Subkeys {
            original_key: key,
            key: new_key,
            mod_key,
            locales,
            keys,
        }
    }
}

fn strings_accessor_method_name(locale: &Locale) -> Ident {
    format_ident!("__get_{}_translations__", locale.top_locale_name)
}

#[allow(clippy::too_many_arguments)]
fn create_locale_type_inner<const IS_TOP: bool>(
    type_ident: &syn::Ident,
    parent_ident: Option<&syn::Ident>,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    locales: &[Locale],
    keys: &BTreeMap<Key, LocaleValue>,
    key_path: &mut KeyPath,
    interpolate_display: bool,
    namespace_name: Option<&str>,
    translations_uri: Option<&str>
) -> TokenStream {
    let translations_key = Key::new(TRANSLATIONS_KEY).unwrap_at("TRANSLATIONS_KEY");

    let literal_keys = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Value{
                value: InterpolOrLit::Lit(t),
                defaults,
            } => Some((key, LiteralType::from(*t), defaults)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let literal_accessors = literal_keys
        .iter()
        .map(|(key, literal_type, defaults)| {
            let computed_defaults= defaults.compute();
            if cfg!(feature = "show_keys_only") {
                let key_str = key_path.to_string_with_key(key);
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        pub fn #key(self) -> l_i18n_crate::__private::LitWrapperFut<impl std::future::Future<Output = l_i18n_crate::__private::LitWrapper<&'static str>>> {
                            let fut = async move {
                                l_i18n_crate::__private::LitWrapper::new(#key_str)
                            };
                            l_i18n_crate::__private::LitWrapperFut::new(fut)
                        }
                    }
                } else if cfg!(feature = "dynamic_load") {
                    quote! {
                        pub fn #key(self) -> l_i18n_crate::__private::LitWrapperFut<l_i18n_crate::__private::LitWrapper<&'static str>> {
                            l_i18n_crate::__private::LitWrapperFut::new_not_fut(#key_str)
                        }
                    }
                } else {
                    quote! {
                        pub const fn #key(self) -> l_i18n_crate::__private::LitWrapper<&'static str> {
                            l_i18n_crate::__private::LitWrapper::new(#key_str)
                        }
                    }
                }
            } else {
                let match_arms = locales.iter().filter_map(|locale| {
                    let lit = locale
                        .keys
                        .get(key)
                        .unwrap_at("create_locale_type_inner_1");
                    if matches!(lit, ParsedValue::Default) {
                        return None;
                    }
                    let ident = &locale.top_locale_name;
                    let accessor = strings_accessor_method_name(locale);
                    let defaulted = computed_defaults.get(&locale.top_locale_name).map(|defaulted_locales| {
                        defaulted_locales.iter().map(|key| {
                            quote!(| #enum_ident::#key)
                        }).collect::<TokenStream>()
                    });
                    let lit = parsed_value::to_token_stream(lit, locale.top_locale_string_count);
                    let ts = if *literal_type == LiteralType::String {
                        let strings_count = locale.top_locale_string_count;
                        if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                            quote! {
                                #enum_ident::#ident #defaulted => {
                                    let #translations_key: &'static [Box<str>; #strings_count] = #type_ident::#accessor().await;
                                    l_i18n_crate::__private::LitWrapper::new(#lit)
                                }
                            }
                        } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                            quote! {
                                #enum_ident::#ident #defaulted => {
                                    let #translations_key: &'static [&'static str; #strings_count] = #type_ident::#accessor();
                                    l_i18n_crate::__private::LitWrapperFut::new_not_fut(#lit)
                                }
                            }
                        } else {
                            quote! {
                                #enum_ident::#ident #defaulted => {
                                    const #translations_key: &[&str; #strings_count] = #type_ident::#accessor();
                                    l_i18n_crate::__private::LitWrapper::new(#lit)
                                }
                            }
                        }
                    } else if cfg!(feature = "dynamic_load") {
                        quote! {
                            #enum_ident::#ident #defaulted => {
                                l_i18n_crate::__private::LitWrapperFut::new_not_fut(#lit)
                            }
                        }
                    } else {
                        quote! {
                            #enum_ident::#ident #defaulted => {
                                l_i18n_crate::__private::LitWrapper::new(#lit)
                            }
                        }
                    };
                    Some(ts)
                });
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        pub fn #key(self) -> l_i18n_crate::__private::LitWrapperFut<impl std::future::Future<Output = l_i18n_crate::__private::LitWrapper<#literal_type>>> {
                            let fut = async move {
                                match self.0 {
                                    #(
                                        #match_arms
                                    )*
                                }
                            };
                            l_i18n_crate::__private::LitWrapperFut::new(fut)
                        }
                    }
                } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                    quote! {
                        pub fn #key(self) -> l_i18n_crate::__private::LitWrapperFut<l_i18n_crate::__private::LitWrapper<#literal_type>> {
                            match self.0 {
                                #(
                                    #match_arms
                                )*
                            }
                        }
                    }
                } else {
                    quote! {
                        pub const fn #key(self) -> l_i18n_crate::__private::LitWrapper<#literal_type> {
                            match self.0 {
                                #(
                                    #match_arms
                                )*
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let subkeys = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Subkeys { locales, keys } => {
                Some(Subkeys::new(key.clone(), locales, keys))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let subkeys_ts = subkeys.iter().map(|sk| {
        let subkey_mod_ident = &sk.mod_key;
        key_path.push_key(sk.original_key.clone());
        let subkey_impl = create_locale_type_inner::<false>(
            &sk.key,
            Some(type_ident),
            enum_ident,
            translation_unit_enum_ident,
            sk.locales,
            &sk.keys.0,
            key_path,
            interpolate_display,
            namespace_name,
            translations_uri
        );
        key_path.pop_key();
        quote! {
            pub mod #subkey_mod_ident {
                use super::{#enum_ident, l_i18n_crate};

                #subkey_impl
            }
        }
    });

    let subkeys_accessors = subkeys.iter().map(|sk| {
        let original_key = &sk.original_key;
        let key = &sk.key;
        let mod_ident = &sk.mod_key;
        quote! {
            pub const fn #original_key(self) -> subkeys::#mod_ident::#key {
                subkeys::#mod_ident::#key::__new_internal(self.0)
            }

        }
    });

    let subkeys_module = subkeys.is_empty().not().then(move || {
        quote! {
            #[doc(hidden)]
            pub mod subkeys {
                use super::{#enum_ident, l_i18n_crate};

                #(
                    #subkeys_ts
                )*
            }
        }
    });

    let builders = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Value {
                value: InterpolOrLit::Interpol(keys),
                defaults
             } => Some((
                key,
                Interpolation::new(
                    key,
                    enum_ident,
                    keys,
                    locales,
                    key_path,
                    type_ident,
                    interpolate_display,
                    defaults
                ),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    let builder_accessors = builders.iter().map(|(key, inter)| {
        let inter_ident = &inter.ident;
        quote! {
            pub const fn #key(self) -> builders::#inter_ident {
                builders::#inter_ident::new(self.0)
            }
        }
    });

    let builder_impls = builders.iter().map(|(_, inter)| &inter.imp);

    let builder_module = builders.is_empty().not().then(move || {
        quote! {
            #[doc(hidden)]
            pub mod builders {
                use super::{#enum_ident, l_i18n_crate};

                #(
                    #builder_impls
                )*
            }
        }
    });

    let string_holders = if IS_TOP {
        locales
            .iter()
            .map(|locale| {
                let locale_name = &*locale.top_locale_name.ident;
                let struct_name = format_ident!("{}_{}", type_ident, locale_name);
                let strings_count = locale.top_locale_string_count;
                let strings = &*locale.strings;

                let get_fn = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        pub async fn get_translations() -> &'static [Box<str>; #strings_count] {
                            <Self as l_i18n_crate::__private::fetch_translations::TranslationUnit>::request_strings().await
                        }
                    }
                } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                    quote! {
                        pub fn get_translations() -> &'static [&'static str; #strings_count] {
                            <Self as l_i18n_crate::__private::fetch_translations::TranslationUnit>::register();
                            <Self as l_i18n_crate::__private::fetch_translations::TranslationUnit>::STRINGS
                        }
                    }

                } else {
                    quote! {
                        pub const fn get_translations() -> &'static [&'static str; #strings_count] {
                            <Self as l_i18n_crate::__private::fetch_translations::TranslationUnit>::STRINGS
                        }
                    }
                };

                
                let request_translations = if cfg!(all(feature = "dynamic_load", feature = "csr")) {
                    let uri = translations_uri.expect("Missing URI"); // Already check before
                    // trigger with rustc 1.85, still in nightly tho
                    // #[allow(clippy::literal_string_with_formatting_args)]
                    let endpoint = uri.replace("{locale}", &locale.name.name).replace("{namespace}", namespace_name.unwrap_or(""));
                    quote! {
                        pub async fn __i18n_request_translations__() -> Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, l_i18n_crate::reexports::leptos::server_fn::ServerFnError> {
                            use l_i18n_crate::reexports::leptos::server_fn::ServerFnError;

                            #[l_i18n_crate::reexports::leptos::server(endpoint = #endpoint, prefix = "", input = l_i18n_crate::reexports::leptos::server_fn::codec::GetUrl, output = l_i18n_crate::reexports::leptos::server_fn::codec::Json)]
                            pub async fn i18n_request_translations_inner() -> Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, ServerFnError>;
                            
                            i18n_request_translations_inner().await
                        }
                    }
                } else {
                    quote!()
                };

                let id = if parent_ident.is_some() {
                    quote!(const ID: super::super::#translation_unit_enum_ident = super::super::#translation_unit_enum_ident::#type_ident)
                } else {
                    quote!(const ID: () = ())
                };

                let get_string = if cfg!(not(all(feature = "dynamic_load", not(feature = "ssr")))) {
                    quote!{
                        const STRINGS: &[&str; #strings_count] = &[#(#strings,)*];
                    }
                } else {
                    quote! {
                        fn get_strings_lock() -> &'static l_i18n_crate::__private::fetch_translations::OnceCell<Box<Self::Strings>> {
                            Self::__get_strings_lock()
                        }
                    }
                };

                let string_type = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote!([Box<str>; #strings_count])
                } else {
                    quote!([&'static str; #strings_count])
                };

                let translation_unit_impl = quote! {
                    impl l_i18n_crate::__private::fetch_translations::TranslationUnit for #struct_name {
                        type Locale = #enum_ident;
                        const LOCALE: #enum_ident = #enum_ident::#locale_name;
                        #id;
                        type Strings = #string_type;
                        #get_string
                    }
                };

                let get_strings_lock_fn = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        fn __get_strings_lock() -> &'static l_i18n_crate::__private::fetch_translations::OnceCell<Box<[Box<str>; #strings_count]>> {
                            static STRINGS_LOCK: l_i18n_crate::__private::fetch_translations::OnceCell<Box<[Box<str>; #strings_count]>> = l_i18n_crate::__private::fetch_translations::OnceCell::new();
                            &STRINGS_LOCK
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #[allow(non_camel_case_types)]
                    struct #struct_name;

                    impl #struct_name {
                        #get_fn

                        #request_translations

                        #get_strings_lock_fn
                    }

                    #translation_unit_impl
                }
            })
            .collect()
    } else {
        quote! {}
    };

    let string_accessors = locales.iter().map(|locale| {
        let accessor_ident = strings_accessor_method_name(locale);
        let strings_count = locale.top_locale_string_count;
        match parent_ident {
            Some(parent) if !IS_TOP => {
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        pub async fn #accessor_ident() -> &'static [Box<str>; #strings_count] {
                            super::super::#parent::#accessor_ident().await
                        }
                    }
                } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                    quote! {
                        pub fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                            super::super::#parent::#accessor_ident()
                        }
                    }
                } else {
                    quote! {
                        pub const fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                            super::super::#parent::#accessor_ident()
                        }
                    }
                }
            }
            _ => {
                let string_holder = format_ident!("{}_{}", type_ident, locale.top_locale_name);
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        pub async fn #accessor_ident() -> &'static [Box<str>; #strings_count] {
                            #string_holder::get_translations().await
                        }
                    }
                } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                    quote! {
                        pub fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                            #string_holder::get_translations()
                        }
                    }
                } else {
                    quote! {
                        pub const fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                            #string_holder::get_translations()
                        }
                    }
                }
            }
        }
    });

    let i18n_request_translations_fn = if IS_TOP {
        let match_arms = locales.iter().map(|locale| {
            let string_holder = format_ident!("{}_{}", type_ident, locale.top_locale_name);
            let locale_name = &locale.top_locale_name;
            if cfg!(all(feature = "dynamic_load", feature = "csr")) {
                quote! {
                    #enum_ident::#locale_name => #string_holder::__i18n_request_translations__().await
                }
            } else {
                quote! {
                    #enum_ident::#locale_name => #string_holder::get_translations()
                }
            }
        });
        let match_stmt = if cfg!(all(
            feature = "dynamic_load",
            not(any(feature = "ssr", feature = "csr"))
        )) {
            quote! {
                unreachable!(
                    "This function should not have been called on the client!"
                )
            }
        } else {
            quote! {
                match _locale {
                    #(
                        #match_arms,
                    )*
                }
            }
        };
        if cfg!(all(feature = "dynamic_load", feature = "csr")) {
            quote! {
                #[doc(hidden)]
                pub async fn __i18n_request_translations__(_locale: #enum_ident, _: ()) -> Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, l_i18n_crate::reexports::leptos::server_fn::ServerFnError> {
                    #match_stmt
                }
            }
        } else {
            quote! {
                #[doc(hidden)]
                pub fn __i18n_request_translations__(_locale: #enum_ident, _: ()) -> &'static [&'static str] {
                    #match_stmt
                }
            }
        }
    } else {
        quote!()
    };

    let init_translations = if IS_TOP && cfg!(all(feature = "dynamic_load", feature = "hydrate")) {
        if cfg!(feature = "ssr") {
            quote! {
                #[doc(hidden)]
                pub fn __init_translations__(_locale: #enum_ident, _: (), _values: Vec<Box<str>>) {
                    panic!("Tried to compile with both \"ssr\" and \"hydrate\" features enabled.")
                }
            }
        } else {
            let match_arms = locales.iter().map(|locale| {
                let string_holder = format_ident!("{}_{}", type_ident, locale.top_locale_name);
                let locale_name = &locale.top_locale_name;
                quote! {
                    #enum_ident::#locale_name => <#string_holder as l_i18n_crate::__private::fetch_translations::TranslationUnit>::init_translations(values)
                }
            });
            quote! {
                #[doc(hidden)]
                pub fn __init_translations__(locale: #enum_ident, _: (), values: Vec<Box<str>>) {
                    match locale {
                        #(
                            #match_arms,
                        )*
                    }
                }
            }
        }
    } else {
        quote!()
    };

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #type_ident(#enum_ident);

        type #translation_unit_enum_ident = ();

        impl #type_ident {

            pub const fn __new_internal(locale: #enum_ident) -> Self {
                #type_ident(locale)
            }

            #(
                #[allow(non_snake_case)]
                #literal_accessors
            )*

            #(
                #[allow(non_snake_case)]
                #subkeys_accessors
            )*

            #(
                #[allow(non_snake_case)]
                #builder_accessors
            )*

            #(
                #[allow(non_snake_case)]
                #string_accessors
            )*

            #i18n_request_translations_fn

            #init_translations
        }

        impl l_i18n_crate::LocaleKeys for #type_ident {
            type Locale = #enum_ident;
            fn from_locale(locale: #enum_ident) -> Self {
                Self::__new_internal(locale)
            }
        }

        #string_holders

        #builder_module

        #subkeys_module

    }
}

fn create_namespace_mod_ident(namespace_ident: &syn::Ident) -> syn::Ident {
    format_ident!("ns_{}", namespace_ident)
}

fn create_namespaces_types(
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    namespaces: &[Namespace],
    keys: &BTreeMap<Key, BuildersKeysInner>,
    interpolate_display: bool,
    translations_uri: Option<&str>
) -> TokenStream {
    let namespaces = namespaces
        .iter()
        .map(|ns| {
            let namespace_module_ident = create_namespace_mod_ident(&ns.key.ident);
            (ns, namespace_module_ident)
        })
        .collect::<Vec<_>>();

    let namespaces_ts = namespaces
        .iter()
        .map(|(namespace, namespace_module_ident)| {
            let keys = keys
                .get(&namespace.key)
                .unwrap_at("create_namespaces_types_1");
            let mut key_path = KeyPath::new(Some(namespace.key.clone()));
            let type_impl = create_locale_type_inner::<true>(
                &namespace.key.ident,
                Some(keys_ident),
                enum_ident,
                translation_unit_enum_ident,
                &namespace.locales,
                &keys.0,
                &mut key_path,
                interpolate_display,
                Some(&namespace.key.name),
                translations_uri
            );

            quote! {
                pub mod #namespace_module_ident {
                    use super::{#enum_ident, l_i18n_crate};

                    #type_impl
                }
            }
        });

    let namespaces_accessors = namespaces
        .iter()
        .map(|(namespace, namespace_module_ident)| {
            let key = &namespace.key;
            quote! {
                pub fn #key(self) -> namespaces::#namespace_module_ident::#key {
                    namespaces::#namespace_module_ident::#key::__new_internal(self.0)
                }
            }
        });

    let translations_unit_variants = namespaces.iter().map(|(ns, _)| ns.key.to_token_stream());

    let as_str_match_arms = namespaces.iter().map(|(ns, _)| {
        let ns_ident = &ns.key.ident;
        let ns_name = &ns.key.name;
        quote! {
            #translation_unit_enum_ident::#ns_ident => #ns_name
        }
    });

    let deserialize_match_arms = namespaces.iter().map(|(ns, _)| {
        let ns_ident = &ns.key.ident;
        let ns_name = &ns.key.name;
        quote! {
            #ns_name => Ok(#translation_unit_enum_ident::#ns_ident)
        }
    });

    let get_strings_match_arms = namespaces.iter().map(|(ns, namespace_module_ident)| {
        let ns_ident = &ns.key.ident;
        let maybe_await = cfg!(all(feature = "dynamic_load", feature = "csr")).then(|| quote!(.await));
        quote! {
            #translation_unit_enum_ident::#ns_ident => namespaces::#namespace_module_ident::#ns_ident::__i18n_request_translations__(locale, ()) #maybe_await
        }
    });

    let get_strings_match_stmt = if cfg!(all(
        feature = "dynamic_load",
        not(any(feature = "ssr", feature = "csr"))
    )) {
        quote! {
            unreachable!(
                "This function should not have been called on the client!"
            )
        }
    } else {
        quote! {
            match translations_id {
                #(
                    #get_strings_match_arms,
                )*
            }
        }
    };

    let init_translations = if cfg!(all(feature = "dynamic_load", feature = "hydrate")) {
        let match_arms = namespaces.iter().map(|(ns, namespace_module_ident)| {
            let ns_ident = &ns.key.ident;
            quote! {
                #translation_unit_enum_ident::#ns_ident => namespaces::#namespace_module_ident::#ns_ident::__init_translations__(locale, (), values)
            }
        });
        quote! {
            #[doc(hidden)]
            pub fn __init_translations__(locale: #enum_ident, translations_id: #translation_unit_enum_ident, values: Vec<Box<str>>) {
                match translations_id {
                    #(
                        #match_arms,
                    )*
                }
            }
        }
    } else {
        quote!()
    };

    let translation_request_fn = if cfg!(all(feature = "dynamic_load", feature = "csr")) {
        quote! {
            #[doc(hidden)]
            pub async fn __i18n_request_translations__(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> Result<l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, l_i18n_crate::reexports::leptos::server_fn::ServerFnError> {
                #get_strings_match_stmt
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub fn __i18n_request_translations__(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> &'static [&'static str] {
                #get_strings_match_stmt
            }
        }
    };

    quote! {
        #[doc(hidden)]
        pub mod namespaces {
            use super::{#enum_ident, l_i18n_crate};

            #(
                #namespaces_ts
            )*

        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_snake_case)]
        pub struct #keys_ident(#enum_ident);

        impl #keys_ident {
            pub const fn __new_internal(locale: #enum_ident) -> Self {
                Self(locale)
            }

            #(
                #[allow(non_snake_case)]
                #namespaces_accessors
            )*

            #translation_request_fn

            #init_translations
        }

        impl l_i18n_crate::LocaleKeys for #keys_ident {
            type Locale = #enum_ident;
            fn from_locale(locale: #enum_ident) -> Self {
                Self::__new_internal(locale)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(non_camel_case_types)]
        pub enum #translation_unit_enum_ident {
            #(
                #translations_unit_variants,
            )*
        }

        impl #translation_unit_enum_ident {
            pub fn as_str(self) -> &'static str {
                match self {
                    #(
                        #as_str_match_arms,
                    )*
                }
            }
        }

        impl l_i18n_crate::__private::TranslationUnitId for #translation_unit_enum_ident {
            fn to_str(self) -> Option<&'static str> {
                Some(self.as_str())
            }
        }

        impl l_i18n_crate::reexports::serde::Serialize for #translation_unit_enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: l_i18n_crate::reexports::serde::Serializer,
            {
                l_i18n_crate::reexports::serde::Serialize::serialize((*self).as_str(), serializer)
            }
        }

        impl<'de> l_i18n_crate::reexports::serde::Deserialize<'de> for #translation_unit_enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<#translation_unit_enum_ident, D::Error>
            where
                D: l_i18n_crate::reexports::serde::de::Deserializer<'de>,
            {
                let s = l_i18n_crate::reexports::serde::de::Deserializer::deserialize_string(deserializer, l_i18n_crate::__private::StrVisitor)?;
                match s.as_str() {
                    #(
                        #deserialize_match_arms,
                    )*
                    _ => Err(<D::Error as leptos_i18n::reexports::serde::de::Error>::custom(format!("invalid translation unit id: {}", s)))
                }
            }
        }
    }
}

fn create_locale_type(
    keys: &BuildersKeys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    interpolate_display: bool,
    translations_uri: Option<&str>
) -> TokenStream {
    match keys {
        BuildersKeys::NameSpaces { namespaces, keys } => create_namespaces_types(
            keys_ident,
            enum_ident,
            translation_unit_enum_ident,
            namespaces,
            keys,
            interpolate_display,
            translations_uri
        ),
        BuildersKeys::Locales { locales, keys } => create_locale_type_inner::<true>(
            keys_ident,
            None,
            enum_ident,
            translation_unit_enum_ident,
            locales,
            &keys.0,
            &mut KeyPath::new(None),
            interpolate_display,
            None,
            translations_uri
        ),
    }
}
