use leptos_i18n_parser::extraction::Namespace;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    CodegenOptions,
    codegen::{builders::infos::BuildersInfos, docs::gen_keys_doc, locales::gen_locales},
};

pub fn gen_namespaces(
    namespaces: &[Namespace],
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    builders: &BuildersInfos,
    options: &CodegenOptions,
) -> TokenStream {
    let init_translations = if cfg!(all(feature = "dynamic_load", feature = "hydrate")) {
        let match_arms = namespaces.iter().map(|ns| {
            let ns_ident = &*ns.name.ident;
            quote! {
                #translation_unit_enum_ident::#ns_ident => #ns_ident::#keys_ident::__init_translations__(locale, (), values)
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

    let get_strings_match_arms = namespaces.iter().map(|ns| {
        let ns_ident = &*ns.name.ident;
        let maybe_await = cfg!(all(feature = "dynamic_load", feature = "csr")).then(|| quote!(.await));
        quote! {
            #translation_unit_enum_ident::#ns_ident => #ns_ident::#keys_ident::__i18n_request_translations__(locale, ()) #maybe_await
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

    let translation_request_fn = if cfg!(all(feature = "dynamic_load", feature = "csr")) {
        quote! {
            #[doc(hidden)]
            #[allow(unused_variables)]
            pub async fn __i18n_request_translations__(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, __l_i18n_crate::reexports::leptos::server_fn::ServerFnError> {
                #get_strings_match_stmt
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            #[allow(unused_variables)]
            pub fn __i18n_request_translations__(locale: #enum_ident, translations_id: #translation_unit_enum_ident) -> &'static [&'static str] {
                #get_strings_match_stmt
            }
        }
    };

    let namespaces_accessors = namespaces.iter().map(|ns| {
        let docs = if options.gen_docs {
            let mut docs = format!("Full path: `{}`\n", ns.name);
            gen_keys_doc(&mut docs, &ns.locales.keys).unwrap();
            quote! {
                #[doc = #docs]
            }
        } else {
            quote! {}
        };
        let key = &*ns.name.ident;
        quote! {
            #docs
            pub fn #key(self) -> #key::#keys_ident {
                #key::#keys_ident
            }
        }
    });

    let deserialize_match_arms = namespaces.iter().map(|ns| {
        let ns_ident = &*ns.name.ident;
        let ns_name = &*ns.name.name;
        quote! {
            #ns_name => Ok(#translation_unit_enum_ident::#ns_ident)
        }
    });

    let as_str_match_arms = namespaces.iter().map(|ns| {
        let ns_ident = &*ns.name.ident;
        let ns_name = &*ns.name.name;
        quote! {
            #translation_unit_enum_ident::#ns_ident => #ns_name
        }
    });

    let translations_unit_variants = namespaces.iter().map(|ns| &*ns.name.ident);

    let locales_impls = namespaces.iter().map(|ns| {
        let ns_key = &ns.name;
        let ts = gen_locales(
            &ns.locales,
            keys_ident,
            enum_ident,
            translation_unit_enum_ident,
            Some(ns_key),
            builders,
            options,
        );
        quote! {
            pub mod #ns_key {
                #[allow(unused)]
                use super::{#enum_ident, __l_i18n_crate, __builders};

                #ts
            }
        }
    });

    quote! {

        #(#locales_impls)*

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_snake_case)]
        #[doc(hidden)]
        pub struct #keys_ident;

        impl #keys_ident {
            #(
                #[allow(non_snake_case)]
                #namespaces_accessors
            )*

            #translation_request_fn

            #init_translations
        }

        impl __l_i18n_crate::scopes::Keys for #keys_ident {
            const THIS: Self = #keys_ident;
        }

        impl __l_i18n_crate::scopes::Scope for #keys_ident {
            type BaseLocale = #enum_ident;
            type Keys = Self;
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
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

        impl __l_i18n_crate::__private::TranslationUnitId for #translation_unit_enum_ident {
            fn to_str(self) -> Option<&'static str> {
                Some(self.as_str())
            }
        }

        impl __l_i18n_crate::reexports::serde::Serialize for #translation_unit_enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __l_i18n_crate::reexports::serde::Serializer,
            {
                __l_i18n_crate::reexports::serde::Serialize::serialize((*self).as_str(), serializer)
            }
        }

        impl<'de> __l_i18n_crate::reexports::serde::Deserialize<'de> for #translation_unit_enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<#translation_unit_enum_ident, D::Error>
            where
                D: __l_i18n_crate::reexports::serde::de::Deserializer<'de>,
            {
                let s = __l_i18n_crate::reexports::serde::de::Deserializer::deserialize_string(deserializer, __l_i18n_crate::__private::StrVisitor)?;
                match s.as_str() {
                    #(
                        #deserialize_match_arms,
                    )*
                    _ => Err(<D::Error as leptos_i18n::reexports::serde::de::Error>::custom(format!("invalid translation unit id: {s}")))
                }
            }
        }
    }
}
