use leptos_i18n_parser::formatters::ValueFormatter;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::parse_macro_input;

use parsed_input::ParsedInput;

pub mod parsed_input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    View,
    String,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Locale,
    Context,
    Untracked,
}

pub fn t_format(
    tokens: proc_macro::TokenStream,
    input_type: InputType,
    output_type: OutputType,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ParsedInput);
    t_format_inner(input, input_type, output_type).into()
}

pub fn t_format_inner(
    input: ParsedInput,
    input_type: InputType,
    output_type: OutputType,
) -> TokenStream {
    let ParsedInput {
        context,
        value,
        formatter,
    } = input;

    let locale_ident = syn::Ident::new("_locale", Span::call_site());
    let value_ident = syn::Ident::new("_value", Span::call_site());
    let ctx = syn::Ident::new("_ctx", Span::call_site());

    let get_locale = input_type.get_locale(&ctx);
    let to_output = output_type.to_output(formatter, &value_ident, &locale_ident);

    let ts = output_type.wrapp(get_locale, to_output, &locale_ident);

    quote! {{
        use leptos_i18n as l_i18n_crate;
        let #value_ident = #value;
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
}

impl OutputType {
    fn to_output(
        self,
        formatter: ValueFormatter,
        value_ident: &syn::Ident,
        locale_ident: &syn::Ident,
    ) -> TokenStream {
        match self {
            OutputType::View => formatter.var_to_view(value_ident, locale_ident),
            OutputType::Display => formatter.var_to_display(value_ident, locale_ident),
            OutputType::String => {
                let ts = formatter.var_to_display(value_ident, locale_ident);
                quote! { std::string::ToString::to_string(&#ts) }
            }
        }
    }

    fn wrapp(
        self,
        get_locale: TokenStream,
        to_output: TokenStream,
        locale_ident: &syn::Ident,
    ) -> TokenStream {
        match self {
            OutputType::View => quote! {
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
