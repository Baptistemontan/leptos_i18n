use leptos_i18n_parser::extraction::Builders;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_builder_module(builders: &Builders) -> TokenStream {
    quote! {
        #[doc(hidden)]
        pub mod __builders {

        }
    }
}
