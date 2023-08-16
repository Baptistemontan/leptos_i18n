use quote::{format_ident, quote, ToTokens};
use syn::{Expr, Ident, Token};

pub enum InterpolatedValue {
    // form t!(i18n, key, count)
    Var(Ident),
    // form t!(i18n, key, count = ..)
    AssignedVar { key: Ident, value: Expr },
    // form t!(i18n, key, <count>)
    Comp(Ident),
    // form t!(i18n, key, <count> = ..)
    AssignedComp { key: Ident, value: Expr },
}

impl syn::parse::Parse for InterpolatedValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_comp = input.peek(Token![<]);
        if is_comp {
            input.parse::<Token![<]>()?;
        }
        let key = input.parse()?;
        if is_comp {
            input.parse::<Token![>]>()?;
        }
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            if is_comp {
                InterpolatedValue::AssignedComp { key, value }
            } else {
                InterpolatedValue::AssignedVar { key, value }
            }
        } else if is_comp {
            InterpolatedValue::Comp(key)
        } else {
            InterpolatedValue::Var(key)
        };
        Ok(value)
    }
}

impl ToTokens for InterpolatedValue {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            InterpolatedValue::Var(ident) => {
                let var_ident = format_ident!("var_{}", ident);
                quote!(#var_ident(#ident))
            }
            InterpolatedValue::Comp(ident) => {
                let comp_ident = format_ident!("comp_{}", ident);
                quote!(#comp_ident(#ident))
            }
            InterpolatedValue::AssignedVar { key, value } => {
                let var_ident = format_ident!("var_{}", key);
                quote!(#var_ident(#value))
            }
            InterpolatedValue::AssignedComp { key, value } => {
                let comp_ident = format_ident!("comp_{}", key);
                quote!(#comp_ident(#value))
            }
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_token_stream().to_tokens(tokens)
    }
}
