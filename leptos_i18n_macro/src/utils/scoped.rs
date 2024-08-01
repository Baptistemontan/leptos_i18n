use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use super::Keys;

struct ScopeParsedInput {
    pub context: syn::Expr,
    pub keys: Keys,
}

impl syn::parse::Parse for ScopeParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<syn::token::Comma>()?;
        let keys = input.parse()?;
        Ok(ScopeParsedInput { context, keys })
    }
}

pub fn scope_i18n(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ScopeParsedInput);
    scope_i18n_inner(input).into()
}

fn scope_i18n_inner(input: ScopeParsedInput) -> TokenStream {
    let ScopeParsedInput { context, keys } = input;
    quote! {{
        leptos_i18n::__private::scope_ctx_util(#context, |_k| &_k.#keys)
    }}
}

pub fn use_i18n_scoped(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as Keys);
    use_i18n_scoped_inner(input).into()
}

fn use_i18n_scoped_inner(keys: Keys) -> TokenStream {
    quote! {{
        leptos_i18n::__private::scope_ctx_util(use_i18n(), |_k| &_k.#keys)
    }}
}

pub fn scope_locale(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ScopeParsedInput);
    scope_locale_inner(input).into()
}

fn scope_locale_inner(input: ScopeParsedInput) -> TokenStream {
    let ScopeParsedInput {
        context: locale,
        keys,
    } = input;
    quote! {{ leptos_i18n::__private::scope_locale_util(#locale, |_k| &_k.#keys) }}
}
