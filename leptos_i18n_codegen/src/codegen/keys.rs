use leptos_i18n_parser::{
    extraction::{Builders, Keys, Locales, ValuesOrSubkeys},
    utils::Key,
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    CodegenOptions,
    codegen::{
        docs::gen_keys_doc, locales::strings_accessor_method_name,
        values::gen_values_modules_and_accessors,
    },
};

pub fn gen_subkeys_impls(
    keys: &Keys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &Builders,
    options: &CodegenOptions,
) -> impl Iterator<Item = TokenStream> {
    keys.values.iter().map(|(key, value)| match value {
        ValuesOrSubkeys::Subkeys(keys) => gen_subkeys_module_and_accessor(
            key, keys, keys_ident, enum_ident, locales, builders, options,
        ),
        ValuesOrSubkeys::Values(values) => gen_values_modules_and_accessors(
            key, values, keys_ident, enum_ident, locales, builders, options,
        ),
    })
}

fn gen_subkeys_module_and_accessor(
    key: &Key,
    keys: &Keys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &Builders,
    options: &CodegenOptions,
) -> TokenStream {
    let module_impl = gen_subkeys_module(keys, keys_ident, enum_ident, locales, builders, options);
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
                #key::#keys_ident::__new_internal(self.0)
            }
        }
    }
}

fn gen_subkeys_module(
    keys: &Keys,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &Builders,
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

    let keys_impls = gen_subkeys_impls(keys, keys_ident, enum_ident, locales, builders, options);

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
        pub struct #keys_ident(#enum_ident);

        impl __l_i18n_crate::LocaleKeys for #keys_ident {
            type Locale = #enum_ident;
            fn from_locale(locale: #enum_ident) -> Self {
                Self::__new_internal(locale)
            }
        }

        impl #keys_ident {
            pub const fn __new_internal(locale: #enum_ident) -> Self {
                #keys_ident(locale)
            }

            #(
                #[allow(non_snake_case)]
                #string_accessors
            )*
        }

        #(#keys_impls)*
    }
}
