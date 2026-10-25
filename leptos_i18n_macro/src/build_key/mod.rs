use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, token::Comma};

mod args;

use args::Arg;

pub struct ParsedInput {
    pub args: Vec<Arg>,
}

pub fn build_key_macro(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    build_key_macro_inner(input).into()
}

fn build_key_macro_inner(input: ParsedInput) -> TokenStream {
    let ParsedInput { mut args } = input;

    let (keys, values): (Vec<_>, Vec<_>) = args.iter_mut().map(|arg| arg.param()).unzip();
    let params = quote! {
        let (#(#keys,)*) = (#(#values,)*);
    };

    quote! {
        {
            #params
            move |__builder| {
                #(
                    let __builder = __builder.#args;
                )*
                #[deny(deprecated)]
                __builder.build()
            }
        }
    }
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = input
            .parse_terminated(Arg::parse, Comma)?
            .into_iter()
            .collect();
        Ok(ParsedInput { args })
    }
}
