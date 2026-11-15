use quote::format_ident;
use syn::{Token, parse_macro_input};

struct ParsedInput {
    key: String,
    path: syn::Path,
}

pub fn scope_marker(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ParsedInput { key, path } = parse_macro_input!(tokens as ParsedInput);

    let ts = leptos_i18n_codegen::utils::scope_marker::key_to_scope_marker(&key, &path);

    ts.into()
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let path = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse()?
        } else {
            syn::Path::from(format_ident!("leptos_i18n"))
        };

        let key = ident.to_string();
        Ok(ParsedInput { key, path })
    }
}
