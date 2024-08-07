use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{buffer::Cursor, parse::ParseBuffer, Expr, Ident, Token};

use super::OutputType;

pub enum InterpolatedValue {
    // form t!(i18n, key, count)
    Var(Ident),
    // form t!(i18n, key, count = ..)
    AssignedVar {
        key: Ident,
        value: Expr,
    },
    // form t!(i18n, key, <count>)
    Comp(Ident),
    // form t!(i18n, key, <count> = ..)
    AssignedComp {
        key: Ident,
        value: Expr,
    },
    // form t!(i18n, key, <count> = <count attrs...>)
    DirectComp {
        key: Ident,
        comp_name: Ident,
        attrs: TokenStream,
    },
    Count(Expr),
}

fn check_component_end(input: Cursor) -> bool {
    // check for "/>" with either a ',' or end of stream after.
    let cursor = match input.punct() {
        Some((punct, cursor)) if punct.as_char() == '/' => cursor,
        _ => return false,
    };
    let cursor = match cursor.punct() {
        Some((punct, cursor)) if punct.as_char() == '>' => cursor,
        _ => return false,
    };
    match cursor.token_tree() {
        Some((TokenTree::Punct(punct), _)) if punct.as_char() == ',' => true,
        None => true,
        _ => false,
    }
}

fn parse_view_component(input: &ParseBuffer) -> syn::Result<(Ident, TokenStream)> {
    input.parse::<Token![<]>()?;
    let comp_name = input.parse()?;
    let mut attrs = TokenStream::new();
    while !check_component_end(input.cursor()) {
        let token = input.parse::<TokenTree>()?;
        attrs.append(token)
    }
    input.parse::<Token![/]>()?;
    input.parse::<Token![>]>()?;
    Ok((comp_name, attrs))
}

impl syn::parse::Parse for InterpolatedValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_comp = input.peek(Token![<]);
        if is_comp {
            input.parse::<Token![<]>()?;
        }
        let is_count = !is_comp && input.peek(Token![$]);
        if is_count {
            input.parse::<Token![$]>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse::<Expr>()?;
            return Ok(InterpolatedValue::Count(value));
        }
        let key = input.parse::<Ident>()?;
        if is_comp {
            input.parse::<Token![>]>()?;
        }
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            if input.peek(Token![<]) {
                let (comp_name, attrs) = input.call(parse_view_component)?;
                InterpolatedValue::DirectComp {
                    key,
                    comp_name,
                    attrs,
                }
            } else {
                let value = input.parse()?;
                if is_comp {
                    InterpolatedValue::AssignedComp { key, value }
                } else {
                    InterpolatedValue::AssignedVar { key, value }
                }
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
    fn format_ident(ident: &Ident, variable: bool) -> Ident {
        if variable {
            format_ident!("var_{}", ident)
        } else {
            format_ident!("comp_{}", ident)
        }
    }

    pub fn param(&mut self, output_type: OutputType) -> (Ident, TokenStream) {
        match self {
            InterpolatedValue::Var(ident) => {
                let var_check_fn = output_type.var_check_fn();
                (ident.clone(), quote!(#var_check_fn(#ident)))
            }
            InterpolatedValue::Comp(ident) => {
                let comp_check_fn = output_type.comp_check_fn();
                (ident.clone(), quote!(#comp_check_fn(#ident)))
            }
            InterpolatedValue::AssignedVar { key, value } => {
                let var_check_fn = output_type.var_check_fn();
                let ts = (key.clone(), quote!(#var_check_fn(#value)));
                *self = InterpolatedValue::Var(key.clone());
                ts
            }
            InterpolatedValue::AssignedComp { key, value } => {
                let comp_check_fn = output_type.comp_check_fn();
                let ts = (key.clone(), quote!(#comp_check_fn(#value)));
                *self = InterpolatedValue::Comp(key.clone());
                ts
            }
            InterpolatedValue::DirectComp {
                key,
                comp_name,
                attrs,
            } => {
                let comp_check_fn = output_type.comp_check_fn();
                let ts = quote! {
                    #comp_check_fn(move |__children:leptos::ChildrenFn| { leptos::view! { <#comp_name #attrs>{move || __children()}</#comp_name> } })
                };
                let ts = (key.clone(), ts);
                *self = InterpolatedValue::Comp(key.clone());
                ts
            }
            InterpolatedValue::Count(expr) => {
                let count_check_fn = output_type.count_check_fn();
                (
                    format_ident!("__plural_count__"),
                    quote!(#count_check_fn(#expr)),
                )
            }
        }
    }
}

impl ToTokens for InterpolatedValue {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            InterpolatedValue::Var(ident) => {
                let var_ident = Self::format_ident(ident, true);
                quote!(#var_ident(Clone::clone(&#ident)))
            }
            InterpolatedValue::Comp(ident) => {
                let comp_ident = Self::format_ident(ident, false);
                quote!(#comp_ident(Clone::clone(&#ident)))
            }
            InterpolatedValue::Count(_) => {
                quote!(plural_count(Clone::clone(&__plural_count__)))
            }
            InterpolatedValue::AssignedVar { .. }
            | InterpolatedValue::AssignedComp { .. }
            | InterpolatedValue::DirectComp { .. } => {
                unreachable!(
                    "Assigned values should have been transformed to normal var in the param step"
                )
            }
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_token_stream().to_tokens(tokens)
    }
}
