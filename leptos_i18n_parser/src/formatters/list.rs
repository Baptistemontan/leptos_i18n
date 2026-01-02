use super::{Formatter, FormatterToTokens};
use super::{impl_formatter, impl_from_arg, impl_to_tokens};
use crate::utils::Key;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub struct ListFormatterParser;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ListFormatter(ListType, ListStyle);

impl_formatter!(
    ListFormatterParser,
    "list",
    ListFormatterBuilder,
    ListFormatter(list_type => ListType, list_style => ListStyle),
    "format_list",
    "Formatting lists is not enabled, enable the \"format_list\" feature to do so"
);

impl FormatterToTokens for ListFormatter {
    fn view_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::ListFormatterInputFn)
    }
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(list_type, list_style) = self;
        quote!(l_i18n_crate::__private::format_list_to_view(#locale_field, #key, #list_type, #list_style))
    }

    fn fmt_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::WriteableList)
    }

    fn to_impl_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(list_type, list_style) = self;
        quote!(l_i18n_crate::__private::format_list_to_display(#locale_field, #key, #list_type, #list_style))
    }

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        let Self(list_type, list_style) = self;
        quote!(l_i18n_crate::__private::format_list_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #list_type, #list_style))
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ListType {
    And,
    Or,
    #[default]
    Unit,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ListStyle {
    #[default]
    Wide,
    Short,
    Narrow,
}
impl ListType {
    impl_from_arg! {
        "and" => Self::And,
        "or" => Self::Or,
        "unit" => Self::Unit,
    }
}

impl ListStyle {
    impl_from_arg! {
        "wide" => Self::Wide,
        "short" => Self::Short,
        "narrow" => Self::Narrow,
    }
}

impl_to_tokens!(
    ListType,
    l_i18n_crate::__private::ListType,
    {
        And,
        Or,
        Unit
    }
);
impl_to_tokens!(
    ListStyle,
    l_i18n_crate::reexports::icu::list::options::ListLength,
    {
        Wide,
        Short,
        Narrow
    }
);
