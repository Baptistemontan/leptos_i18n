use leptos_i18n_parser::utils::Key;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

macro_rules! impl_from {
    ($t: ident, $($variant:ident),*) => {
        impl From<leptos_i18n_parser::utils::formatter::$t> for $t {
            fn from(value: leptos_i18n_parser::utils::formatter::$t) -> Self {
                match value {
                    $(
                        leptos_i18n_parser::utils::formatter::$t::$variant => Self::$variant,
                    )*
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupingStrategy {
    Auto,
    Never,
    Always,
    Min2,
}

impl ToTokens for GroupingStrategy {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            GroupingStrategy::Auto => {
                quote!(l_i18n_crate::reexports::icu::decimal::options::GroupingStrategy::Auto)
            }
            GroupingStrategy::Never => {
                quote!(l_i18n_crate::reexports::icu::decimal::options::GroupingStrategy::Never)
            }
            GroupingStrategy::Always => {
                quote!(l_i18n_crate::reexports::icu::decimal::options::GroupingStrategy::Always)
            }
            GroupingStrategy::Min2 => {
                quote!(l_i18n_crate::reexports::icu::decimal::options::GroupingStrategy::Min2)
            }
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = Self::to_token_stream(self);
        tokens.extend(ts);
    }
}

impl_from!(GroupingStrategy, Auto, Never, Always, Min2);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateLength {
    Full,
    Long,
    Medium,
    Short,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeLength {
    Full,
    Long,
    Medium,
    Short,
}

macro_rules! impl_length {
    ($t:ident, $name:ident) => {
        impl ToTokens for $t {
            fn to_token_stream(&self) -> TokenStream {
                match self {
                    Self::Full => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::length::$name::Full)
                    }
                    Self::Long => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::length::$name::Long)
                    }
                    Self::Medium => {
                        quote!(
                            l_i18n_crate::reexports::icu::datetime::options::length::$name::Medium
                        )
                    }
                    Self::Short => {
                        quote!(
                            l_i18n_crate::reexports::icu::datetime::options::length::$name::Short
                        )
                    }
                }
            }

            fn to_tokens(&self, tokens: &mut TokenStream) {
                let ts = self.to_token_stream();
                tokens.extend(ts);
            }
        }

        impl_from!($t, Full, Long, Medium, Short);
    };
}

impl_length!(DateLength, Date);
impl_length!(TimeLength, Time);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    And,
    Or,
    Unit,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListStyle {
    Wide,
    Short,
    Narrow,
}
impl ToTokens for ListType {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            ListType::And => quote!(l_i18n_crate::__private::ListType::And),
            ListType::Or => quote!(l_i18n_crate::__private::ListType::Or),
            ListType::Unit => quote!(l_i18n_crate::__private::ListType::Unit),
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = Self::to_token_stream(self);
        tokens.extend(ts);
    }
}

impl_from!(ListType, And, Or, Unit);

impl ToTokens for ListStyle {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            ListStyle::Wide => quote!(l_i18n_crate::reexports::icu::list::ListLength::Wide),
            ListStyle::Short => quote!(l_i18n_crate::reexports::icu::list::ListLength::Short),
            ListStyle::Narrow => quote!(l_i18n_crate::reexports::icu::list::ListLength::Narrow),
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = Self::to_token_stream(self);
        tokens.extend(ts);
    }
}

impl_from!(ListStyle, Wide, Short, Narrow);

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formatter {
    #[default]
    None,
    Number(GroupingStrategy),
    Date(DateLength),
    Time(TimeLength),
    DateTime(DateLength, TimeLength),
    List(ListType, ListStyle),
}

impl From<leptos_i18n_parser::utils::formatter::Formatter> for Formatter {
    fn from(value: leptos_i18n_parser::utils::formatter::Formatter) -> Self {
        match value {
            leptos_i18n_parser::utils::formatter::Formatter::None => Self::None,
            leptos_i18n_parser::utils::formatter::Formatter::Number(grouping_strategy) => {
                Self::Number(grouping_strategy.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::Date(date_length) => {
                Self::Date(date_length.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::Time(time_length) => {
                Self::Time(time_length.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::DateTime(date_length, time_length) => {
                Self::DateTime(date_length.into(), time_length.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::List(list_type, list_style) => {
                Self::List(list_type.into(), list_style.into())
            }
        }
    }
}

impl Formatter {
    pub fn var_to_view(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Formatter::None => {
                quote!(#key)
            }
            Formatter::Number(grouping_strategy) => {
                quote!(l_i18n_crate::__private::format_number_to_view(#locale_field, #key, #grouping_strategy))
            }
            Formatter::Date(length) => {
                quote!(l_i18n_crate::__private::format_date_to_view(#locale_field, #key, #length))
            }
            Formatter::Time(length) => {
                quote!(l_i18n_crate::__private::format_time_to_view(#locale_field, #key, #length))
            }
            Formatter::DateTime(date_length, time_length) => {
                quote!(l_i18n_crate::__private::format_datetime_to_view(#locale_field, #key, #date_length, #time_length))
            }
            Formatter::List(list_type, list_style) => {
                quote!(l_i18n_crate::__private::format_list_to_view(#locale_field, #key, #list_type, #list_style))
            }
        }
    }

    pub fn var_to_display(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Formatter::None => unreachable!(
                "This function should not have been called on a variable with no formatter."
            ),
            Formatter::Number(grouping_strategy) => {
                quote!(l_i18n_crate::__private::format_number_to_display(#locale_field, #key, #grouping_strategy))
            }
            Formatter::Date(length) => {
                quote!(l_i18n_crate::__private::format_date_to_display(#locale_field, #key, #length))
            }
            Formatter::Time(length) => {
                quote!(l_i18n_crate::__private::format_time_to_display(#locale_field, #key, #length))
            }
            Formatter::DateTime(date_length, time_length) => {
                quote!(l_i18n_crate::__private::format_datetime_to_display(#locale_field, #key, #date_length, #time_length))
            }
            Formatter::List(list_type, list_style) => {
                quote!(l_i18n_crate::__private::format_list_to_display(#locale_field, #key, #list_type, #list_style))
            }
        }
    }

    pub fn var_fmt(self, key: &Key, locale_field: &Key) -> TokenStream {
        match self {
            Formatter::None => {
                quote!(core::fmt::Display::fmt(#key, __formatter))
            }
            Formatter::Number(grouping_strategy) => {
                quote!(l_i18n_crate::__private::format_number_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #grouping_strategy))
            }
            Formatter::Date(length) => {
                quote!(l_i18n_crate::__private::format_date_to_formatter(__formatter, *#locale_field, #key, #length))
            }
            Formatter::Time(length) => {
                quote!(l_i18n_crate::__private::format_time_to_formatter(__formatter, *#locale_field, #key, #length))
            }
            Formatter::DateTime(date_length, time_length) => {
                quote!(l_i18n_crate::__private::format_datetime_to_formatter(__formatter, *#locale_field, #key, #date_length, #time_length))
            }
            Formatter::List(list_type, list_style) => {
                quote!(l_i18n_crate::__private::format_list_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #list_type, #list_style))
            }
        }
    }

    pub fn to_bound(self) -> TokenStream {
        match self {
            Formatter::None => quote!(l_i18n_crate::__private::InterpolateVar),
            Formatter::Number(_) => quote!(l_i18n_crate::__private::NumberFormatterInputFn),
            Formatter::Date(_) => quote!(l_i18n_crate::__private::DateFormatterInputFn),
            Formatter::Time(_) => quote!(l_i18n_crate::__private::TimeFormatterInputFn),
            Formatter::DateTime(_, _) => quote!(l_i18n_crate::__private::DateTimeFormatterInputFn),
            Formatter::List(_, _) => quote!(l_i18n_crate::__private::ListFormatterInputFn),
        }
    }

    pub fn to_string_bound(self) -> TokenStream {
        match self {
            Formatter::None => quote!(::std::fmt::Display),
            Formatter::Number(_) => quote!(l_i18n_crate::__private::IntoFixedDecimal),
            Formatter::Date(_) => quote!(l_i18n_crate::__private::AsIcuDate),
            Formatter::Time(_) => quote!(l_i18n_crate::__private::AsIcuTime),
            Formatter::DateTime(_, _) => quote!(l_i18n_crate::__private::AsIcuDateTime),
            Formatter::List(_, _) => quote!(l_i18n_crate::__private::WriteableList),
        }
    }
}
