use leptos_i18n_parser::{
    extraction::{Keys, Locales, ValuesOrSubkeys},
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    CodegenOptions,
    codegen::{
        builders::infos::BuildersInfos, docs::gen_keys_doc, locales::strings_accessor_method_name,
        values::gen_values_modules_and_accessors,
    },
};

pub fn gen_subkeys_impls(
    keys: &Keys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &BuildersInfos,
    path: &mut KeyPath,
    options: &CodegenOptions,
) -> impl Iterator<Item = TokenStream> {
    keys.values.iter().map(|(key, value)| match value {
        ValuesOrSubkeys::Subkeys(keys) => gen_subkeys_module_and_accessor(
            key, keys, keys_ident, enum_ident, locales, builders, path, options,
        ),
        ValuesOrSubkeys::Values(values) => {
            let path = path.push_key(key.clone());
            gen_values_modules_and_accessors(
                key, values, keys_ident, enum_ident, locales, builders, &path, options,
            )
        }
    })
}

fn gen_subkeys_module_and_accessor(
    key: &Key,
    keys: &Keys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &BuildersInfos,
    path: &mut KeyPath,
    options: &CodegenOptions,
) -> TokenStream {
    let mut path = path.push_key(key.clone());
    let module_impl = gen_subkeys_module(
        keys, keys_ident, enum_ident, locales, builders, &mut path, options,
    );
    let docs = if options.gen_docs {
        let mut docs = String::new();
        gen_keys_doc(&mut docs, keys).unwrap();
        quote! {
            #[doc = #docs]
        }
    } else {
        quote! {}
    };

    quote! {

        #docs
        pub mod #key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate, __builders};

            #module_impl
        }

        impl #keys_ident {
            #docs
            pub const fn #key(self) -> #key::#keys_ident {
                #key::#keys_ident
            }
        }
    }
}

fn gen_subkeys_module(
    keys: &Keys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &BuildersInfos,
    path: &mut KeyPath,
    options: &CodegenOptions,
) -> TokenStream {
    let docs = if options.gen_docs {
        let mut docs = String::new();
        gen_keys_doc(&mut docs, keys).unwrap();
        quote! {
            #[doc = #docs]
        }
    } else {
        quote! {}
    };

    let keys_impls = gen_subkeys_impls(
        keys, keys_ident, enum_ident, locales, builders, path, options,
    );

    let string_accessors = locales.locales.iter().map(|locale| {
        let accessor_ident = strings_accessor_method_name(&locale.name);
        let strings_count = locale.strings.len();
        if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            quote! {
                pub async fn #accessor_ident() -> &'static [Box<str>; #strings_count] {
                    super::#keys_ident::#accessor_ident().await
                }
            }
        } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
            quote! {
                pub fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                    super::#keys_ident::#accessor_ident()
                }
            }
        } else {
            quote! {
                pub const fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                    super::#keys_ident::#accessor_ident()
                }
            }
        }
    });

    quote! {
        #docs
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #keys_ident;

        impl __l_i18n_crate::scopes::Keys for #keys_ident {
            const THIS: Self = #keys_ident;
        }

        impl __l_i18n_crate::scopes::Scope for #keys_ident {
            type BaseLocale = #enum_ident;
            type Keys = Self;
        }


        impl #keys_ident {
            #(
                #[allow(non_snake_case)]
                #string_accessors
            )*
        }

        #(#keys_impls)*
    }
}
