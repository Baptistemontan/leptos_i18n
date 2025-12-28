use proc_macro2::Span;
use quote::ToTokens;
use std::{collections::BTreeMap, fmt::Display};
use syn::{Expr, Ident, Token, parse::ParseBuffer, spanned::Spanned, token::Comma};

use leptos_i18n_codegen::load_locales::plurals::PluralForm;

pub struct ParsedInput {
    pub context: Expr,
    pub count: Expr,
    pub forms: BTreeMap<PluralForm, Expr>,
    pub fallback: Option<(Expr, Span)>,
}

fn emit_err<A, T: ToTokens, U: Display>(tokens: T, message: U) -> syn::Result<A> {
    Err(syn::Error::new_spanned(tokens, message))
}

fn parse_plural_form(input: &ParseBuffer) -> syn::Result<(Option<PluralForm>, Expr, Span)> {
    let is_fallback = input.peek(Token![_]);
    if is_fallback {
        let token = input.parse::<Token![_]>()?;
        input.parse::<Token![=>]>()?;
        let block = input.parse::<Expr>()?;
        let span = token.span();
        return Ok((None, block, span));
    }
    let ident = input.parse::<Ident>()?;
    input.parse::<Token![=>]>()?;
    let block = input.parse::<Expr>()?;
    let form = if ident == "zero" {
        PluralForm::Zero
    } else if ident == "one" {
        PluralForm::One
    } else if ident == "two" {
        PluralForm::Two
    } else if ident == "few" {
        PluralForm::Few
    } else if ident == "many" {
        PluralForm::Many
    } else if ident == "other" {
        PluralForm::Other
    } else {
        return emit_err(
            ident,
            "Unknown form. Allowed forms are \"zero\", \"one\", \"two\", \"few\", \"many\", \"other\" and \"_\" fallback.",
        );
    };
    Ok((Some(form), block, ident.span()))
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<Comma>()?;
        let count_ident = input.parse::<Ident>()?;
        if count_ident != "count" {
            return emit_err(count_ident, "expected \"count\" ident.");
        }
        input.parse::<Token![=]>()?;
        let count = input.parse()?;
        input.parse::<Comma>()?;
        let parsed_forms = input.parse_terminated(parse_plural_form, Token![,])?;
        let mut forms = BTreeMap::new();
        let mut fallback = None;

        for (form, block, span) in parsed_forms {
            let already_exist = match form {
                Some(form) => forms.insert(form, block),
                None => fallback.replace((block, span)).map(|x| x.0),
            };

            if already_exist.is_some() {
                let form = match form {
                    None => "_",
                    Some(PluralForm::Zero) => "zero",
                    Some(PluralForm::One) => "one",
                    Some(PluralForm::Two) => "two",
                    Some(PluralForm::Few) => "few",
                    Some(PluralForm::Many) => "many",
                    Some(PluralForm::Other) => "other",
                };
                let msg = format!("Duplicate form {form}.");
                return Err(syn::Error::new(span, msg));
            }
        }

        Ok(ParsedInput {
            context,
            count,
            forms,
            fallback,
        })
    }
}
