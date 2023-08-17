use proc_macro2::Ident;
use syn::token::Comma;
use syn::Token;

use super::interpolate::InterpolatedValue;

pub enum Key {
    Key(Ident),
    Namespace { namespace: Ident, key: Ident },
}

pub struct ParsedInput {
    pub context: Ident,
    pub key: Key,
    pub interpolations: Option<Vec<InterpolatedValue>>,
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

impl syn::parse::Parse for Key {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let namespace = key;
            let key = input.parse()?;
            Ok(Key::Namespace { namespace, key })
        } else {
            Ok(Key::Key(key))
        }
    }
}
