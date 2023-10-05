use quote::quote;
use syn::parse_macro_input;

use crate::t_macro::interpolate::InterpolatedValueTokenizer;

use self::parsed_input::{Keys, ParsedInput};

pub mod interpolate;
pub mod parsed_input;

pub fn t_macro(
    tokens: proc_macro::TokenStream,
    direct: bool,
    display: bool,
    to_string: bool,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    t_macro_inner(input, direct, display, to_string).into()
}

pub fn t_macro_inner(
    input: ParsedInput,
    direct: bool,
    display: bool,
    to_string: bool,
) -> proc_macro2::TokenStream {
    let ParsedInput {
        context,
        keys,
        interpolations,
    } = input;
    let get_keys = if direct {
        quote!(leptos_i18n::Locale::get_keys(#context))
    } else {
        quote!(leptos_i18n::I18nContext::get_keys(#context))
    };

    let get_key = match keys {
        Keys::SingleKey(key) => quote!(#get_keys.#key),
        Keys::Subkeys(keys) => quote!(#get_keys #(.#keys)*),
        Keys::Namespace(namespace, keys) => {
            quote!(#get_keys.#namespace #(.#keys)*)
        }
    };

    let build = if display {
        if to_string {
            quote!(build_string)
        } else {
            quote!(build_display)
        }
    } else {
        quote!(build)
    };

    let inner = if let Some(interpolations) = interpolations {
        let interpolations = interpolations
            .iter()
            .map(|inter| InterpolatedValueTokenizer::new(inter, display));

        quote! {
            {
                let _key = #get_key;
                #(
                    let _key = _key.#interpolations;
                )*
                #[deny(deprecated)]
                _key.#build()
            }
        }
    } else {
        quote! {
            {
                #[allow(unused)]
                use leptos_i18n::__private::BuildStr;
                let _key = #get_key;
                _key.#build()
            }
        }
    };

    if direct {
        inner
    } else {
        quote!(move || #inner)
    }
}
