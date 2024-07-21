use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use crate::t_macro::interpolate::{InterpolatedValue, InterpolatedValueTokenizer};

use self::parsed_input::{Keys, ParsedInput};

pub mod interpolate;
pub mod parsed_input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    View,
    String,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Locale,
    Context,
    Untracked,
}

pub fn t_macro(
    tokens: proc_macro::TokenStream,
    input_type: InputType,
    output_type: OutputType,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    t_macro_inner(input, input_type, output_type).into()
}

pub fn t_macro_inner(
    input: ParsedInput,
    input_type: InputType,
    output_type: OutputType,
) -> TokenStream {
    let ParsedInput {
        context,
        keys,
        interpolations,
    } = input;

    let get_key = input_type.get_key(context, keys);
    let build_fn = output_type.build_fn();

    let (inner, params) = if let Some(mut interpolations) = interpolations {
        let (keys, values): (Vec<_>, Vec<_>) = interpolations
            .iter_mut()
            .map(InterpolatedValue::param)
            .unzip();
        let params = quote! {
            let (#(#keys,)*) = (#(#values,)*);
        };
        let string = output_type.is_string();
        let interpolations = interpolations
            .iter()
            .map(|inter| InterpolatedValueTokenizer::new(inter, string));

        let inner = quote! {
            {
                let _key = #get_key;
                #(
                    let _key = _key.#interpolations;
                )*
                #[deny(deprecated)]
                _key.#build_fn()
            }
        };

        (inner, Some(params))
    } else {
        let inner = quote! {
            {
                #[allow(unused)]
                use leptos_i18n::__private::BuildStr;
                let _key = #get_key;
                _key.#build_fn()
            }
        };
        (inner, None)
    };

    output_type.wrapp(inner, params)
}

impl OutputType {
    pub fn build_fn(self) -> TokenStream {
        match self {
            OutputType::View => quote!(build),
            OutputType::String => quote!(build_string),
            OutputType::Display => quote!(build_display),
        }
    }

    pub fn is_string(self) -> bool {
        match self {
            OutputType::View => false,
            OutputType::String | OutputType::Display => true,
        }
    }

    pub fn wrapp(self, ts: TokenStream, params: Option<TokenStream>) -> TokenStream {
        match self {
            OutputType::View => quote! {
                {
                    #params
                    move || #ts
                }
            },
            OutputType::String | OutputType::Display => quote! {
                {
                    #params
                    #ts
                }
            },
        }
    }
}

impl InputType {
    pub fn get_key<T: ToTokens>(self, input: T, keys: Keys) -> TokenStream {
        match self {
            InputType::Context => quote!(leptos_i18n::I18nContext::get_keys(#input).#keys),
            InputType::Untracked => {
                quote!(leptos_i18n::I18nContext::get_keys_untracked(#input).#keys)
            }
            InputType::Locale => quote!(leptos_i18n::Locale::get_keys(#input).#keys),
        }
    }
}
