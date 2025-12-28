use leptos_i18n_parser::utils::Key;
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use tinystr::TinyAsciiStr;

macro_rules! impl_from {
    ($t: ident, $($variant:ident $(( $($inner:ident),+ ))? ),*) => {
        impl From<leptos_i18n_parser::utils::formatter::$t> for $t {
            fn from(value: leptos_i18n_parser::utils::formatter::$t) -> Self {
                match value {
                    $(
                        leptos_i18n_parser::utils::formatter::$t::$variant $(( $($inner),+ ))? => Self::$variant $(( $(From::from($inner)),+ ))?,
                    )*
                }
            }
        }
    };
}

macro_rules! impl_to_tokens {
    (
        $type_name:ident,
        $path_prefix:expr,
        { $($variant:ident $(( $($inner:ident),+ ))?),+ $(,)? }
    ) => {
        impl ToTokens for $type_name {
            fn to_token_stream(&self) -> TokenStream {
                match self {
                    $(
                        $type_name::$variant $(( $($inner),+ ))? => {
                            quote!($path_prefix::$variant $(( $(#$inner),+ ))?)
                        },
                    )+
                }
            }

            fn to_tokens(&self, tokens: &mut TokenStream) {
                let ts = Self::to_token_stream(self);
                tokens.extend(ts);
            }
        }
    };
}

macro_rules! impl_to_tokens_and_from {
    (
        $type_name:ident,
        $path_prefix:expr,
        { $($variant:ident $(( $($inner:ident),+ ))?),+ $(,)? }
    ) => {
        impl_to_tokens!($type_name, $path_prefix, { $($variant $(( $($inner),+ ))?),+ });
        impl_from!($type_name, $($variant $(( $($inner),+ ))?),+);
    };
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupingStrategy {
    Auto,
    Never,
    Always,
    Min2,
}

impl_to_tokens_and_from!(
    GroupingStrategy,
    l_i18n_crate::reexports::icu::decimal::options::GroupingStrategy,
    {
        Auto,
        Never,
        Always,
        Min2
    }
);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CurrencyWidth {
    Short,
    Narrow,
}

impl_to_tokens_and_from!(
    CurrencyWidth,
    l_i18n_crate::reexports::icu::currency::options::Width,
    {
        Short,
        Narrow
    }
);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CurrencyCode(pub TinyAsciiStr<3>);

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

impl From<leptos_i18n_parser::utils::formatter::CurrencyCode> for CurrencyCode {
    fn from(value: leptos_i18n_parser::utils::formatter::CurrencyCode) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeLength {
    Long,
    Medium,
    Short,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeAlignment {
    Auto,
    Column,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeYearStyle {
    Auto,
    Full,
    WithEra,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeTimePrecision {
    Hour,
    Minute,
    Second,
    Subsecond(DateTimeSubsecondDigits),
    MinuteOptional,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeSubsecondDigits {
    S1 = 1,
    S2 = 2,
    S3 = 3,
    S4 = 4,
    S5 = 5,
    S6 = 6,
    S7 = 7,
    S8 = 8,
    S9 = 9,
}

impl_to_tokens_and_from!(
    DateTimeLength,
    l_i18n_crate::reexports::icu::datetime::options::Length,
    {
        Long,
        Medium,
        Short
    }
);

impl_to_tokens_and_from!(
    DateTimeAlignment,
    l_i18n_crate::reexports::icu::datetime::options::Alignment,
    {
        Auto,
        Column
    }
);

impl_to_tokens_and_from!(
    DateTimeYearStyle,
    l_i18n_crate::reexports::icu::datetime::options::YearStyle,
    {
        Auto,
        Full,
        WithEra
    }
);

impl_to_tokens_and_from!(
    DateTimeSubsecondDigits,
    l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits,
    {
        S1, S2, S3, S4, S5, S6, S7, S8, S9,
    }
);

impl_to_tokens_and_from!(
    DateTimeTimePrecision,
    l_i18n_crate::reexports::icu::datetime::options::TimePrecision,
    {
        Hour,
        Minute,
        Second,
        Subsecond(subsecond),
        MinuteOptional
    }
);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    And,
    Or,
    Unit,
}

impl_to_tokens_and_from!(
    ListType,
    l_i18n_crate::__private::ListType,
    {
        And,
        Or,
        Unit
    }
);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListStyle {
    Wide,
    Short,
    Narrow,
}

impl_to_tokens_and_from!(
    ListStyle,
    l_i18n_crate::reexports::icu::list::options::ListLength,
    {
        Wide,
        Short,
        Narrow
    }
);

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formatter {
    /// NOT A FORMATTER (see leptos_i18n_parser::utils::formatter::Formatter::Dummy)
    Dummy,
    #[default]
    None,
    Currency(CurrencyWidth, CurrencyCode),
    Number(GroupingStrategy),
    Date(DateTimeLength, DateTimeAlignment, DateTimeYearStyle),
    Time(DateTimeLength, DateTimeAlignment, DateTimeTimePrecision),
    DateTime(
        DateTimeLength,
        DateTimeAlignment,
        DateTimeTimePrecision,
        DateTimeYearStyle,
    ),
    List(ListType, ListStyle),
}

impl_from!(
    Formatter,
    Dummy,
    None,
    Currency(width, code),
    Number(strat),
    Date(len, align, style),
    Time(len, align, precision),
    DateTime(len, align, precision, style),
    List(list_type, style)
);

impl Formatter {
    pub fn var_to_view(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Formatter::None => {
                quote!(#key)
            }
            Formatter::Dummy => unreachable!(
                "var_to_view function should not have been called on a dummy formatter"
            ),
            Formatter::Currency(width, code) => {
                quote!(l_i18n_crate::__private::format_currency_to_view(#locale_field, #key, #width, #code))
            }
            Formatter::Number(grouping_strategy) => {
                quote!(l_i18n_crate::__private::format_number_to_view(#locale_field, #key, #grouping_strategy))
            }
            Formatter::Date(length, alignment, year_style) => {
                quote!(l_i18n_crate::__private::format_date_to_view(#locale_field, #key, #length, #alignment, #year_style))
            }
            Formatter::Time(length, alignment, time_precision) => {
                quote!(l_i18n_crate::__private::format_time_to_view(#locale_field, #key, #length, #alignment, #time_precision))
            }
            Formatter::DateTime(length, alignment, time_precision, year_style) => {
                quote!(l_i18n_crate::__private::format_datetime_to_view(#locale_field, #key, #length, #alignment, #time_precision, #year_style))
            }
            Formatter::List(list_type, list_style) => {
                quote!(l_i18n_crate::__private::format_list_to_view(#locale_field, #key, #list_type, #list_style))
            }
        }
    }

    pub fn var_to_display(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Formatter::None => unreachable!(
                "var_to_display function should not have been called on a variable with no formatter."
            ),
            Formatter::Dummy => {
                unreachable!(
                    "var_to_display function should not have been called on a dummy formatter"
                )
            }
            Formatter::Currency(width, code) => {
                quote!(l_i18n_crate::__private::format_currency_to_display(#locale_field, #key, #width, #code))
            }
            Formatter::Number(grouping_strategy) => {
                quote!(l_i18n_crate::__private::format_number_to_display(#locale_field, #key, #grouping_strategy))
            }
            Formatter::Date(length, alignment, year_style) => {
                quote!(l_i18n_crate::__private::format_date_to_display(#locale_field, #key, #length, #alignment, #year_style))
            }
            Formatter::Time(length, alignment, time_precision) => {
                quote!(l_i18n_crate::__private::format_time_to_display(#locale_field, #key, #length, #alignment, #time_precision))
            }
            Formatter::DateTime(length, alignment, time_precision, year_style) => {
                quote!(l_i18n_crate::__private::format_datetime_to_display(#locale_field, #key, #length, #alignment, #time_precision, #year_style))
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
            Formatter::Dummy => {
                unreachable!("var_fmt function should not have been called on a dummy formatter")
            }
            Formatter::Currency(width, code) => {
                quote!(l_i18n_crate::__private::format_currency_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #width, #code))
            }
            Formatter::Number(grouping_strategy) => {
                quote!(l_i18n_crate::__private::format_number_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #grouping_strategy))
            }
            Formatter::Date(length, alignment, year_style) => {
                quote!(l_i18n_crate::__private::format_date_to_formatter(__formatter, *#locale_field, #key, #length, #alignment, #year_style))
            }
            Formatter::Time(length, alignment, time_precision) => {
                quote!(l_i18n_crate::__private::format_time_to_formatter(__formatter, *#locale_field, #key, #length, #alignment, #time_precision))
            }
            Formatter::DateTime(length, alignment, time_precision, year_style) => {
                quote!(l_i18n_crate::__private::format_datetime_to_formatter(__formatter, *#locale_field, #key, #length, #alignment, #time_precision, #year_style))
            }
            Formatter::List(list_type, list_style) => {
                quote!(l_i18n_crate::__private::format_list_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key), #list_type, #list_style))
            }
        }
    }

    pub fn to_bound(self) -> TokenStream {
        match self {
            Formatter::None => quote!(l_i18n_crate::__private::InterpolateVar),
            Formatter::Dummy => quote!(l_i18n_crate::__private::AnyBound),
            Formatter::Currency(_, _) => quote!(l_i18n_crate::__private::NumberFormatterInputFn),
            Formatter::Number(_) => quote!(l_i18n_crate::__private::NumberFormatterInputFn),
            Formatter::Date(_, _, _) => quote!(l_i18n_crate::__private::DateFormatterInputFn),
            Formatter::Time(_, _, _) => quote!(l_i18n_crate::__private::TimeFormatterInputFn),
            Formatter::DateTime(_, _, _, _) => {
                quote!(l_i18n_crate::__private::DateTimeFormatterInputFn)
            }
            Formatter::List(_, _) => quote!(l_i18n_crate::__private::ListFormatterInputFn),
        }
    }

    pub fn to_string_bound(self) -> TokenStream {
        match self {
            Formatter::None => quote!(::std::fmt::Display),
            Formatter::Dummy => quote!(l_i18n_crate::__private::AnyBound),
            Formatter::Currency(_, _) => quote!(l_i18n_crate::__private::IntoFixedDecimal),
            Formatter::Number(_) => quote!(l_i18n_crate::__private::IntoFixedDecimal),
            Formatter::Date(_, _, _) => quote!(l_i18n_crate::__private::AsIcuDate),
            Formatter::Time(_, _, _) => quote!(l_i18n_crate::__private::AsIcuTime),
            Formatter::DateTime(_, _, _, _) => quote!(l_i18n_crate::__private::AsIcuDateTime),
            Formatter::List(_, _) => quote!(l_i18n_crate::__private::WriteableList),
        }
    }
}
