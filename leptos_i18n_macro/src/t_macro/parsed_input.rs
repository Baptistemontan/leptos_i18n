use syn::{token::Comma, Expr};

use crate::utils::Keys;

use super::interpolate::InterpolatedValue;

pub struct ParsedInput {
    pub context: Expr,
    pub keys: Keys,
    pub interpolations: Option<Vec<InterpolatedValue>>,
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<Comma>()?;
        let keys = input.parse()?;
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
            keys,
            interpolations,
        })
    }
}
