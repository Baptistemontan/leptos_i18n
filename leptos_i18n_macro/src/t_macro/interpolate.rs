use proc_macro2::TokenStream;
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

impl InterpolatedValue {
    fn to_token_stream(&self, string: bool) -> TokenStream {
        fn format_ident(ident: &Ident, variable: bool, string: bool) -> Ident {
            match (variable, string) {
                (true, true) => format_ident!("var_{}_string", ident),
                (true, false) => format_ident!("var_{}", ident),
                (false, true) => format_ident!("comp_{}_string", ident),
                (false, false) => format_ident!("comp_{}", ident),
            }
        }

        match self {
            InterpolatedValue::Var(ident) => {
                let var_ident = format_ident(ident, true, string);
                quote!(#var_ident(#ident))
            }
            InterpolatedValue::Comp(ident) => {
                let comp_ident = format_ident(ident, false, string);
                quote!(#comp_ident(#ident))
            }
            InterpolatedValue::AssignedVar { key, value } => {
                let var_ident = format_ident(key, true, string);
                quote!(#var_ident(#value))
            }
            InterpolatedValue::AssignedComp { key, value } => {
                let comp_ident = format_ident(key, false, string);
                quote!(#comp_ident(#value))
            }
        }
    }
}

pub struct InterpolatedValueTokenizer<'a> {
    string: bool,
    value: &'a InterpolatedValue,
}

impl<'a> InterpolatedValueTokenizer<'a> {
    pub fn new(value: &'a InterpolatedValue, string: bool) -> Self {
        Self { string, value }
    }
}

impl ToTokens for InterpolatedValueTokenizer<'_> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        self.value.to_token_stream(self.string)
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_token_stream().to_tokens(tokens)
    }
}
