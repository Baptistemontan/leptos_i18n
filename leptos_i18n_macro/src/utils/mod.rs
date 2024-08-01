use quote::{quote, TokenStreamExt};
use syn::Token;

pub mod scoped;

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
            Keys::Subkeys(keys) => tokens.append_separated(keys, quote!(.)),
        }
    }
}
