use std::borrow::Cow;

use super::{Formatter, FormatterToTokens};
use super::{impl_formatter, impl_from_arg, impl_to_tokens};
use crate::utils::Key;
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use tinystr::{TinyAsciiStr, tinystr};

pub struct CurrencyFormatterParser;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CurrencyFormatter(CurrencyWidth, CurrencyCode);

impl_formatter!(
    CurrencyFormatterParser,
    "currency",
    CurrencyFormatterBuilder,
    CurrencyFormatter(width => CurrencyWidth, currency_code => CurrencyCode),
    "format_currency",
    "Formatting currencies is not enabled, enable the \"format_currency\" feature to do so"
);

impl FormatterToTokens for CurrencyFormatter {
    fn view_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::NumberFormatterInputFn)
    }
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(width, code) = self;
        quote!(l_i18n_crate::__private::format_currency_to_view(#locale_field, #key, #width, #code))
    }

    fn display_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::IntoFixedDecimal)
    }

    fn to_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(width, code) = self;
        quote!(l_i18n_crate::__private::format_currency_to_display(#locale_field, #key, #width, #code))
    }

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        let Self(width, code) = self;
        quote!(l_i18n_crate::__private::format_currency_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #width, #code))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct CurrencyCode(pub TinyAsciiStr<3>);

impl Default for CurrencyCode {
    fn default() -> Self {
        Self(tinystr!(3, "USD"))
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum CurrencyWidth {
    #[default]
    Short,
    Narrow,
}

impl CurrencyCode {
    pub fn from_arg(arg: Option<&str>) -> Result<Self, Cow<'static, str>> {
        match arg {
            Some(v) => match TinyAsciiStr::try_from_str(v) {
                Ok(code) => Ok(Self(code)),
                Err(err) => {
                    let err = format!("Invalid code: {err}");
                    Err(Cow::Owned(err))
                }
            },
            None => Err(Cow::Borrowed("missing currency code")),
        }
    }
}

impl ToTokens for CurrencyCode {
    fn to_token_stream(&self) -> TokenStream {
        let code = Literal::string(self.0.as_str());
        quote!(l_i18n_crate::reexports::icu::currency::CurrencyCode(
            l_i18n_crate::reexports::tinystr!(3, #code)
        ))
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = Self::to_token_stream(self);
        tokens.extend(ts);
    }
}

impl CurrencyWidth {
    impl_from_arg! {
        "short" => Self::Short,
        "narrow" => Self::Narrow,
    }
}

impl_to_tokens!(
    CurrencyWidth,
    l_i18n_crate::reexports::icu::currency::options::Width,
    {
        Short,
        Narrow
    }
);
