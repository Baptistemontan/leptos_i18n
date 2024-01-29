use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use crate::t_macro::interpolate::InterpolatedValueTokenizer;

use self::parsed_input::{Keys, ParsedInput};

pub mod interpolate;
pub mod parsed_input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    View,
    #[cfg(feature = "interpolate_display")]
    String,
    #[cfg(feature = "interpolate_display")]
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Locale,
    Context,
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

    let inner = if let Some(interpolations) = interpolations {
        let interpolations = interpolations
            .iter()
            .map(|inter| InterpolatedValueTokenizer::new(inter, output_type.is_string()));

        quote! {
            {
                let _key = #get_key;
                #(
                    let _key = _key.#interpolations;
                )*
                #[deny(deprecated)]
                _key.#build_fn()
            }
        }
    } else {
        quote! {
            {
                #[allow(unused)]
                use leptos_i18n::__private::BuildStr;
                let _key = #get_key;
                _key.#build_fn()
            }
        }
    };

    output_type.wrapp(inner)
}

impl OutputType {
    pub fn build_fn(self) -> TokenStream {
        match self {
            OutputType::View => quote!(build),
            #[cfg(feature = "interpolate_display")]
            OutputType::String => quote!(build_string),
            #[cfg(feature = "interpolate_display")]
            OutputType::Display => quote!(build_display),
        }
    }

    pub fn is_string(self) -> bool {
        match self {
            OutputType::View => false,
            #[cfg(feature = "interpolate_display")]
            OutputType::String | OutputType::Display => true,
        }
    }

    pub fn wrapp(self, ts: TokenStream) -> TokenStream {
        match self {
            OutputType::View => quote!(move || #ts),
            #[cfg(feature = "interpolate_display")]
            OutputType::String | OutputType::Display => ts,
        }
    }
}

impl InputType {
    pub fn get_key<T: ToTokens>(self, input: T, keys: Keys) -> TokenStream {
        match self {
            InputType::Context => quote!(leptos_i18n::I18nContext::get_keys(#input).#keys),
            InputType::Locale => quote!(leptos_i18n::Locale::get_keys(#input).#keys),
        }
    }
}
