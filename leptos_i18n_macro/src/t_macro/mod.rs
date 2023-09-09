use quote::quote;
use syn::parse_macro_input;

use self::parsed_input::{Keys, ParsedInput};

pub mod interpolate;
pub mod parsed_input;

pub fn t_macro(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    t_macro_inner(input).into()
}

pub fn t_macro_inner(input: ParsedInput) -> proc_macro2::TokenStream {
    let ParsedInput {
        context,
        keys,
        interpolations,
    } = input;
    let get_key = match keys {
        Keys::SingleKey(key) => quote!(leptos_i18n::I18nContext::get_keys(#context).#key),
        Keys::Subkeys(keys) => quote!(leptos_i18n::I18nContext::get_keys(#context)#(.#keys)*),
        Keys::Namespace(namespace, keys) => {
            quote!(leptos_i18n::I18nContext::get_keys(#context).#namespace #(.#keys)*)
        }
    };
    if let Some(interpolations) = interpolations {
        if cfg!(feature = "debug_interpolations") {
            quote! {
                move || {
                    let _key = #get_key;
                    #(
                        let _key = _key.#interpolations;
                    )*
                    #[deny(deprecated)]
                    _key.build()
                }
            }
        } else {
            quote! {
                move || {
                    let _key = #get_key;
                    #(
                        let _key = _key.#interpolations;
                    )*
                    _key
                }
            }
        }
    } else if cfg!(feature = "debug_interpolations") {
        quote! {
            move || {
                #[allow(unused)]
                use leptos_i18n::__private::BuildStr;
                let _key = #get_key;
                _key.build()
            }
        }
    } else {
        quote!(move || #get_key)
    }
}
