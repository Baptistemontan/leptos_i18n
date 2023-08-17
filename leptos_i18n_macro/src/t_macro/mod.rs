use quote::quote;
use syn::parse_macro_input;

use self::parsed_input::ParsedInput;

// pub mod error;
pub mod interpolate;
pub mod parsed_input;

pub fn t_macro(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    t_macro_inner(input).into()
}

pub fn t_macro_inner(input: ParsedInput) -> proc_macro2::TokenStream {
    let ParsedInput {
        context,
        key,
        interpolations,
    } = input;
    let get_key = match key {
        parsed_input::Key::Key(key) => quote!(::leptos_i18n::I18nContext::get_keys(#context).#key),
        parsed_input::Key::Namespace { namespace, key } => {
            quote!(::leptos_i18n::I18nContext::get_keys(#context).#namespace.#key)
        }
    };
    if let Some(interpolations) = interpolations {
        quote! {
            move || {
                let _key = #get_key;
                #(
                    let _key = _key.#interpolations;
                )*
                _key
            }
        }
    } else {
        quote!(move || #get_key)
    }
}
