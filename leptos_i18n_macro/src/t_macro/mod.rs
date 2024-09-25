use interpolate::InterpolatedValue;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use self::parsed_input::ParsedInput;
use crate::utils::Keys;

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
        mut interpolations,
    } = input;

    let get_key = input_type.get_key(context, keys);
    let (builder_fn, build_fn) = output_type.build_fns();

    let (inner, params) = if let Some(interpolations) = interpolations.as_mut() {
        let (keys, values): (Vec<_>, Vec<_>) =
            interpolations.iter_mut().map(|inter| inter.param()).unzip();
        let params = quote! {
            let (#(#keys,)*) = (#(#values,)*);
        };

        let inner = quote! {
            {
                let _builder = #get_key.#builder_fn();
                #(
                    let _builder = _builder.#interpolations;
                )*
                #[deny(deprecated)]
                _builder.#build_fn()
            }
        };

        (inner, Some(params))
    } else {
        let inner = quote! {
            {
                let _builder = #get_key.#builder_fn();
                #[deny(deprecated)]
                _builder.#build_fn()
            }
        };
        (inner, None)
    };

    output_type.wrapp(inner, params, interpolations.as_deref())
}

impl OutputType {
    pub fn build_fns(self) -> (TokenStream, TokenStream) {
        match self {
            OutputType::View => (quote!(builder), quote!(build().into_view)),
            OutputType::String => (quote!(display_builder), quote!(build_string)),
            OutputType::Display => (quote!(display_builder), quote!(build_display)),
        }
    }

    pub fn clone_values(interpolations: &[InterpolatedValue]) -> TokenStream {
        let keys = interpolations
            .iter()
            .filter_map(InterpolatedValue::get_ident);

        let keys_to_clone = keys.clone();

        quote!(let (#(#keys,)*) = (#(core::clone::Clone::clone(&#keys_to_clone),)*);)
    }

    pub fn wrapp(
        self,
        ts: TokenStream,
        params: Option<TokenStream>,
        interpolations: Option<&[InterpolatedValue]>,
    ) -> TokenStream {
        match self {
            OutputType::View => {
                let clone_values = interpolations.map(Self::clone_values);
                quote! {
                    {
                        #params
                        move || {
                            #clone_values
                            #ts
                        }
                    }
                }
            }
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
            InputType::Context => quote!(leptos_i18n::I18nContext::get_keys(#input).#keys()),
            InputType::Untracked => {
                quote!(leptos_i18n::I18nContext::get_keys_untracked(#input).#keys())
            }
            InputType::Locale => quote!(leptos_i18n::Locale::get_keys(#input).#keys()),
        }
    }
}
