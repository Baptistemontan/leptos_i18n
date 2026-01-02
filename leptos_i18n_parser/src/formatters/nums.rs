use super::{Formatter, FormatterToTokens};
use super::{impl_formatter, impl_from_arg, impl_to_tokens};
use crate::utils::Key;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub struct NumberFormatterParser;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NumberFormatter(GroupingStrategy);

impl_formatter!(
    NumberFormatterParser,
    "number",
    NumberFormatterBuilder,
    NumberFormatter(grouping_strategy => GroupingStrategy),
    "format_nums",
    "Formatting numbers is not enabled, enable the \"format_nums\" feature to do so"
);

impl FormatterToTokens for NumberFormatter {
    fn view_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::NumberFormatterInputFn)
    }
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(strat) = self;
        quote!(l_i18n_crate::__private::format_number_to_view(#locale_field, #key, #strat))
    }

    fn display_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::IntoFixedDecimal)
    }

    fn to_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(strat) = self;
        quote!(l_i18n_crate::__private::format_number_to_display(#locale_field, #key, #strat))
    }

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        let Self(strat) = self;
        quote!(l_i18n_crate::__private::format_number_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #strat))
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum GroupingStrategy {
    #[default]
    Auto,
    Never,
    Always,
    Min2,
}

impl GroupingStrategy {
    impl_from_arg! {
        "auto" => Self::Auto,
        "never" => Self::Never,
        "always" => Self::Always,
        "min2" => Self::Min2,
    }
}

impl_to_tokens!(
    GroupingStrategy,
    l_i18n_crate::reexports::icu::decimal::options::GroupingStrategy,
    {
        Auto,
        Never,
        Always,
        Min2
    }
);
