use leptos_i18n_parser::parse_locales::warning::{Warning, Warnings};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn warning_fn((index, warning): (usize, &Warning)) -> TokenStream {
    let msg = warning.to_string();
    let fn_name = format_ident!("w{}", index);
    quote! {
        #[deprecated(note = #msg)]
        fn #fn_name() {
            unimplemented!()
        }
    }
}

fn generate_warnings_inner(warnings: &[Warning]) -> TokenStream {
    let warning_fns = warnings.iter().enumerate().map(warning_fn);

    let fn_calls = (0..warnings.len()).map(|i| {
        let fn_name = format_ident!("w{}", i);
        quote!(#fn_name();)
    });

    quote! {
        #[allow(unused)]
        fn warnings() {
            #(
                #warning_fns
            )*

            #(
                #fn_calls
            )*
        }
    }
}

pub fn generate_warnings(warnings: &Warnings) -> Option<TokenStream> {
    let ws = warnings.take_inner();

    if ws.is_empty() {
        None
    } else {
        Some(generate_warnings_inner(&ws))
    }
}
