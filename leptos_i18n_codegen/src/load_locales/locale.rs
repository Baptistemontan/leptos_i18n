use quote::{quote, ToTokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralType {
    String,
    Bool,
    Signed,
    Unsigned,
    Float,
}

impl From<leptos_i18n_parser::parse_locales::locale::LiteralType> for LiteralType {
    fn from(value: leptos_i18n_parser::parse_locales::locale::LiteralType) -> Self {
        match value {
            leptos_i18n_parser::parse_locales::locale::LiteralType::String => Self::String,
            leptos_i18n_parser::parse_locales::locale::LiteralType::Bool => Self::Bool,
            leptos_i18n_parser::parse_locales::locale::LiteralType::Signed => Self::Signed,
            leptos_i18n_parser::parse_locales::locale::LiteralType::Unsigned => Self::Unsigned,
            leptos_i18n_parser::parse_locales::locale::LiteralType::Float => Self::Float,
        }
    }
}

impl ToTokens for LiteralType {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            LiteralType::String => quote!(&'static str),
            LiteralType::Bool => quote!(bool),
            LiteralType::Signed => quote!(i64),
            LiteralType::Unsigned => quote!(u64),
            LiteralType::Float => quote!(f64),
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.to_token_stream());
    }
}
