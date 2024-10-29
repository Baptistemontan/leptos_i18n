use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse_macro_input;

use parsed_input::ParsedInput;

use crate::load_locales::plurals::PluralRuleType;

pub mod parsed_input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Locale,
    Context,
    Untracked,
}

pub fn t_plural(
    tokens: proc_macro::TokenStream,
    input_type: InputType,
    plural_type: PluralRuleType,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    t_plural_inner(input, input_type, plural_type).into()
}

pub fn t_plural_inner(
    input: ParsedInput,
    input_type: InputType,
    plural_type: PluralRuleType,
) -> TokenStream {
    if cfg!(not(feature = "plurals")) {
        return quote! {
            compile_error!("Use of the \"t_plural!\" macro is not possible without the \"plurals\" feature enabled.");
        };
    }

    let ParsedInput {
        context,
        count,
        forms,
        fallback,
    } = input;

    let locale_ident = syn::Ident::new("_locale", Span::call_site());
    let count_ident = syn::Ident::new("_value", Span::call_site());
    let ctx = syn::Ident::new("_ctx", Span::call_site());

    let get_locale = input_type.get_locale(&ctx);

    let match_arms = forms.iter().map(|(form, block)| quote!(#form => #block));
    let fallback = fallback.map(|(expr, span)| {
        let fb = quote_spanned! { span => _ };
        quote!(#fb => #expr)
    });

    let ts = quote! {
        match leptos_i18n::__private::get_plural_category_for(#locale_ident, &#count_ident, #plural_type) {
            #(
                #match_arms,
            )*
            #fallback,
        }
    };

    let ts = input_type.wrapp(get_locale, ts, &locale_ident);

    quote! {{
        use leptos_i18n as l_i18n_crate;
        let #count_ident = #count;
        let #ctx = #context;
        #ts
    }}
}

impl InputType {
    fn get_locale(self, context: &syn::Ident) -> TokenStream {
        match self {
            InputType::Locale => ToTokens::to_token_stream(context),
            InputType::Context => quote! {
                leptos_i18n::I18nContext::get_locale(#context)
            },
            InputType::Untracked => quote! {
                leptos_i18n::I18nContext::get_locale_untracked(#context)
            },
        }
    }

    fn wrapp(
        self,
        get_locale: TokenStream,
        to_output: TokenStream,
        locale_ident: &syn::Ident,
    ) -> TokenStream {
        match self {
            InputType::Context => quote! {
                move || {
                    let #locale_ident = #get_locale;
                    #to_output
                }
            },
            _ => quote! {{
                let #locale_ident = #get_locale;
                #to_output
            }},
        }
    }
}
