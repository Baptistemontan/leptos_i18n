use std::fmt::Display;

use leptos_i18n_codegen::utils::formatter::Formatter;
use quote::ToTokens;
use syn::{
    parse::{ParseBuffer, ParseStream},
    token::Comma,
    Expr, Ident, Token,
};

pub struct ParsedInput {
    pub context: Expr,
    pub value: Expr,
    pub formatter: Formatter,
}

fn emit_err<A, T: ToTokens, U: Display>(tokens: T, message: U) -> syn::Result<A> {
    Err(syn::Error::new_spanned(tokens, message))
}

fn parse_arg(input: ParseStream) -> syn::Result<(Ident, Ident)> {
    let arg_name = input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let arg_value = input.parse::<Ident>()?;
    Ok((arg_name, arg_value))
}

fn is_parenthesized(input: ParseStream) -> syn::Result<ParseBuffer> {
    let content;
    syn::parenthesized!(content in input);
    Ok(content)
}

fn convert_formatter_result(
    res: Result<
        Option<leptos_i18n_parser::utils::formatter::Formatter>,
        leptos_i18n_parser::utils::formatter::Formatter,
    >,
    span: proc_macro2::Span,
    err: syn::Error,
) -> syn::Result<Formatter> {
    match res {
        Ok(Some(formatter)) => Ok(formatter.into()),
        Ok(None) => Err(err),
        Err(formatter) => Err(syn::Error::new(span, formatter.err_message())),
    }
}

fn parse_formatter(
    input: ParseBuffer,
    formatter_name: Ident,
    err: syn::Error,
) -> syn::Result<Formatter> {
    let args = input.parse_terminated(parse_arg, Token![;])?;
    let args: Vec<_> = args.into_iter().collect();

    let span = formatter_name.span();
    let res = leptos_i18n_parser::utils::formatter::Formatter::from_name_and_args(
        formatter_name,
        Some(&args),
    );
    convert_formatter_result(res, span, err)
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
        let formatter_name_err = syn::Error::new(formatter_name.span(), "unknown formatter name.");
        let formatter = if let Ok(args) = is_parenthesized(input) {
            parse_formatter(args, formatter_name, formatter_name_err)?
        } else {
            let span = formatter_name.span();
            let res = leptos_i18n_parser::utils::formatter::Formatter::from_name_and_args(
                formatter_name,
                None,
            );
            convert_formatter_result(res, span, formatter_name_err)?
        };

        Ok(ParsedInput {
            context,
            value,
            formatter,
        })
    }
}
