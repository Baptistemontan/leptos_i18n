use leptos_i18n_parser::{
    extraction::{Builders, Locales, Values},
    utils::Key,
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::CodegenOptions;

pub fn gen_values_modules_and_accessors(
    key: &Key,
    values: &Values,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &Builders,
    options: &CodegenOptions,
) -> TokenStream {
    let docs = if options.gen_docs {
        let mut docs = String::new();
        // gen_keys_doc(&mut docs, keys).unwrap();
        quote! {
            #[doc = #docs]
        }
    } else {
        quote! {}
    };

    let builder = builders
        .builders
        .get(&values.builder_id)
        .expect("invalid builder id");
    let builder_name = &builder.name;

    quote! {
        #docs
        pub mod #key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate, __builders};
            pub type Builder = __builders::#builder_name;

        }

        impl #keys_ident {
            #docs
            pub const fn #key(self) {
            }
        }
    }
}
