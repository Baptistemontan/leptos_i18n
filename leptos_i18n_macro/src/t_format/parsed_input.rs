use leptos_i18n_parser::formatters::{Formatters, ValueFormatter};
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
    pub formatter: ValueFormatter,
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

fn convert_formatter_result(
    res: Option<Result<ValueFormatter, &'static str>>,
    span: proc_macro2::Span,
    err: syn::Error,
) -> syn::Result<ValueFormatter> {
    match res {
        Some(Ok(formatter)) => Ok(formatter),
        None => Err(err),
        Some(Err(err)) => Err(syn::Error::new(span, err)),
    }
}

fn parse_formatter(
    input: ParseBuffer,
    formatter_name: Ident,
    formatters: &Formatters,
    err: syn::Error,
) -> syn::Result<ValueFormatter> {
    let args = input.parse_terminated(parse_arg, Token![;])?;
    let args: Vec<_> = args.into_iter().collect();
    let n = formatter_name.to_string();

    let span = formatter_name.span();
    let res = formatters.parse_from_tt(&n, &args);
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
        let formatters = Formatters::new();
        let formatter = if let Ok(args) = is_parenthesized(input) {
            parse_formatter(args, formatter_name, &formatters, formatter_name_err)?
        } else {
            let span = formatter_name.span();
            let n = formatter_name.to_string();
            let res = formatters.parse_from_tt(&n, &[]);
            convert_formatter_result(res, span, formatter_name_err)?
        };

        Ok(ParsedInput {
            context,
            value,
            formatter,
        })
    }
}
