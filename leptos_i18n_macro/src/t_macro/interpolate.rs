use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{buffer::Cursor, parse::ParseBuffer, Expr, Ident, Token};

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
    pub fn get_ident(&self) -> Option<&Ident> {
        match self {
            InterpolatedValue::Var(ident) | InterpolatedValue::Comp(ident) => Some(ident),
            InterpolatedValue::AssignedVar { .. }
            | InterpolatedValue::AssignedComp { .. }
            | InterpolatedValue::DirectComp { .. } => None,
        }
    }

    fn format_ident(ident: &Ident, variable: bool) -> Ident {
        if variable {
            format_ident!("var_{}", ident)
        } else {
            format_ident!("comp_{}", ident)
        }
    }

    pub fn param(&mut self) -> (Ident, TokenStream) {
        match self {
            InterpolatedValue::Var(ident) => (ident.clone(), quote!(#ident)),
            InterpolatedValue::Comp(ident) => (ident.clone(), quote!(#ident)),
            InterpolatedValue::AssignedVar { key, value } => {
                let key = key.clone();
                let ts = quote!(#value);
                *self = InterpolatedValue::Var(key.clone());
                (key.clone(), ts)
            }
            InterpolatedValue::AssignedComp { key, value } => {
                let key = key.clone();
                let ts = quote!(#value);
                *self = InterpolatedValue::Comp(key.clone());
                (key.clone(), ts)
            }
            InterpolatedValue::DirectComp {
                key,
                comp_name,
                attrs,
            } => {
                let ts = quote! {
                    move |__children: leptos::ChildrenFn| { leptos::view! { <#comp_name #attrs>{move || __children()}</#comp_name> } }
                };
                let key = key.clone();
                *self = InterpolatedValue::Comp(key.clone());
                (key.clone(), ts)
            }
        }
    }
}

impl ToTokens for InterpolatedValue {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            InterpolatedValue::Var(ident) => {
                let var_ident = Self::format_ident(ident, true);
                quote!(#var_ident(#ident))
            }
            InterpolatedValue::Comp(ident) => {
                let comp_ident = Self::format_ident(ident, false);
                quote!(#comp_ident(#ident))
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
