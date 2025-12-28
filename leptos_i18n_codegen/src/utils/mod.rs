use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::Token;

pub mod formatter;

pub enum Keys {
    SingleKey(syn::Ident),
    Subkeys(Vec<syn::Ident>),
}

fn parse_subkeys(input: syn::parse::ParseStream, keys: &mut Vec<syn::Ident>) -> syn::Result<()> {
    keys.push(input.parse()?);
    while input.peek(Token![.]) {
        input.parse::<Token![.]>()?;
        keys.push(input.parse()?);
    }
    Ok(())
}

impl syn::parse::Parse for Keys {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first_key = input.parse()?;
        let dot = input.peek(Token![.]);
        if dot || input.peek(Token![::]) {
            if dot {
                input.parse::<Token![.]>()?;
            } else {
                input.parse::<Token![::]>()?;
            }
            let mut keys = vec![first_key];
            parse_subkeys(input, &mut keys)?;
            Ok(Keys::Subkeys(keys))
        } else {
            Ok(Keys::SingleKey(first_key))
        }
    }
}

impl quote::ToTokens for Keys {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Keys::SingleKey(key) => tokens.append(key.clone()),
            Keys::Subkeys(keys) => tokens.append_separated(keys, quote!(().)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EitherOfWrapper {
    Single,
    Duo,
    Multiple(syn::Ident),
    Nested(Box<Self>),
}

impl EitherOfWrapper {
    pub fn new(size: usize) -> EitherOfWrapper {
        match size {
            0 => {
                unreachable!("0 locales ? how is this possible ? should have been checked by now.")
            }
            1 => EitherOfWrapper::Single,
            2 => EitherOfWrapper::Duo,
            3..=16 => EitherOfWrapper::Multiple(format_ident!("EitherOf{}", size)),
            17.. => EitherOfWrapper::Nested(Box::new(Self::new(size - 15))),
        }
    }

    pub fn wrap<T: ToTokens>(&self, i: usize, ts: T) -> TokenStream {
        const LETTERS: [char; 16] = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
        ];
        match self {
            EitherOfWrapper::Single => ts.into_token_stream(),
            EitherOfWrapper::Duo if i == 0 => {
                quote!(l_i18n_crate::reexports::leptos::either::Either::Left(#ts))
            }
            EitherOfWrapper::Duo => {
                quote!(l_i18n_crate::reexports::leptos::either::Either::Right(#ts))
            }
            EitherOfWrapper::Multiple(ident) => {
                let variant = format_ident!("{}", LETTERS[i]);
                quote!(l_i18n_crate::reexports::leptos::either::#ident::#variant(#ts))
            }
            EitherOfWrapper::Nested(last) => match i {
                0..=14 => {
                    let variant = format_ident!("{}", LETTERS[i]);
                    quote!(l_i18n_crate::reexports::leptos::either::EitherOf16::#variant(#ts))
                }
                15.. => {
                    let variant = format_ident!("{}", LETTERS[15]);
                    let ts = last.wrap(i - 15, ts);
                    quote!(l_i18n_crate::reexports::leptos::either::EitherOf16::#variant(#ts))
                }
            },
        }
    }
}

pub fn fit_in_leptos_tuple(values: &[TokenStream]) -> TokenStream {
    const TUPLE_MAX_SIZE: usize = 26;
    let values_len = values.len();
    if values_len <= TUPLE_MAX_SIZE {
        quote!((#(#values,)*))
    } else {
        let chunk_size = values_len.div_ceil(TUPLE_MAX_SIZE);
        let values = values.chunks(chunk_size).map(fit_in_leptos_tuple);
        quote!((#(#values,)*))
    }
}
