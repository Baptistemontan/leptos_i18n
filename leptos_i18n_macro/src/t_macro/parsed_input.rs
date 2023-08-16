use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{token::Comma, Expr, Token};

pub struct ParsedInput {
    pub context: Ident,
    pub key: Ident,
    pub interpolations: Option<Vec<InterpolatedValue>>,
}

pub enum InterpolatedValue {
    // form t!(i18n, key, count)
    Ident(Ident),
    // form t!(i18n, key, count = ..)
    Assignement { key: Ident, value: Expr },
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<Comma>()?;
        let key = input.parse()?;
        let comma = input.parse::<Comma>();
        let interpolations = match comma {
            Ok(_) => {
                let interpolations = input
                    .parse_terminated(InterpolatedValue::parse, Comma)?
                    .into_iter()
                    .collect();
                Some(interpolations)
            }
            Err(_) if input.is_empty() => None,
            Err(err) => return Err(err),
        };
        Ok(ParsedInput {
            context,
            key,
            interpolations,
        })
    }
}

impl syn::parse::Parse for InterpolatedValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            InterpolatedValue::Assignement { key, value }
        } else {
            InterpolatedValue::Ident(key)
        };
        Ok(value)
    }
}

impl ToTokens for InterpolatedValue {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            InterpolatedValue::Ident(ident) => quote!(#ident(#ident)),
            InterpolatedValue::Assignement { key, value } => quote!(#key(#value)),
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_token_stream().to_tokens(tokens)
    }
}
