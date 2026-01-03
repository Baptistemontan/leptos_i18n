use leptos_i18n_parser::formatters::{Formatters, VarBounds};
use quote::ToTokens;
use std::fmt::Display;
use syn::{
    Expr, Ident, Token,
    parse::{ParseBuffer, ParseStream},
    token::Comma,
};

pub struct ParsedInput {
    pub context: Expr,
    pub value: Expr,
    pub formatter: VarBounds,
}

fn emit_err<A, T: ToTokens, U: Display>(tokens: T, message: U) -> syn::Result<A> {
    Err(syn::Error::new_spanned(tokens, message))
}

fn parse_arg(input: ParseStream) -> syn::Result<(Ident, Option<Ident>)> {
    let arg_name = input.parse::<Ident>()?;
    let arg_value = if input.peek(Token![:]) {
        input.parse::<Token![:]>()?;
        let arg_value = input.parse::<Ident>()?;
        Some(arg_value)
    } else {
        None
    };
    Ok((arg_name, arg_value))
}

fn is_parenthesized(input: ParseStream) -> syn::Result<ParseBuffer> {
    let content;
    syn::parenthesized!(content in input);
    Ok(content)
}

fn parse_formatter(
    input: ParseBuffer,
    formatter_name: Ident,
    formatters: &Formatters,
) -> syn::Result<VarBounds> {
    let args = input.parse_terminated(parse_arg, Token![;])?;
    formatters.parse_from_tt(formatter_name, Some(args))
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<Comma>()?;
        let value = input.parse()?;
        input.parse::<Comma>()?;
        let formatter_ident = input.parse::<Ident>()?;
        if formatter_ident != "formatter" {
            return emit_err(formatter_ident, "expected \"formatter\" ident.");
        }
        input.parse::<Token![:]>()?;
        let formatter_name = input.parse::<Ident>()?;
        let formatters = Formatters::new();
        let formatter = if let Ok(args) = is_parenthesized(input) {
            parse_formatter(args, formatter_name, &formatters)?
        } else {
            formatters.parse_from_tt(formatter_name, None)?
        };

        Ok(ParsedInput {
            context,
            value,
            formatter,
        })
    }
}
