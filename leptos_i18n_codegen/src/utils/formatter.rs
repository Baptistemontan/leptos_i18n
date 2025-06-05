use leptos_i18n_parser::utils::Key;
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use tinystr::TinyAsciiStr;

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

macro_rules! impl_to_tokens {
    (
        $type_name:ident,
        $path_prefix:expr,
        { $($variant:ident),+ $(,)? }
    ) => {
        impl ToTokens for $type_name {
            fn to_token_stream(&self) -> TokenStream {
                match self {
                    $(
                        $type_name::$variant => {
                            quote!($path_prefix::$variant)
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupingStrategy {
    Auto,
    Never,
    Always,
    Min2,
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

impl_from!(GroupingStrategy, Auto, Never, Always, Min2);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CurrencyWidth {
    Short,
    Narrow,
}

impl_to_tokens!(
    CurrencyWidth,
    l_i18n_crate::reexports::icu::currency::options::Width,
    {
        Short,
        Narrow
    }
);

impl_from!(CurrencyWidth, Short, Narrow);

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

impl_to_tokens!(
    DateTimeLength,
    l_i18n_crate::reexports::icu::datetime::options::Length,
    {
        Long,
        Medium,
        Short
    }
);

impl_from!(DateTimeLength, Long, Medium, Short);

impl_to_tokens!(
    DateTimeAlignment,
    l_i18n_crate::reexports::icu::datetime::options::Alignment,
    {
        Auto,
        Column
    }
);

impl_from!(DateTimeAlignment, Auto, Column);

impl_to_tokens!(
    DateTimeYearStyle,
    l_i18n_crate::reexports::icu::datetime::options::YearStyle,
    {
        Auto,
        Full,
        WithEra
    }
);

impl_from!(DateTimeYearStyle, Auto, Full, WithEra);

impl ToTokens for DateTimeTimePrecision {
    fn to_token_stream(&self) -> TokenStream {
        match self {
            DateTimeTimePrecision::Hour => {
                quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Hour)
            }
            DateTimeTimePrecision::Minute => {
                quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Minute)
            }
            DateTimeTimePrecision::Second => {
                quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Second)
            }
            DateTimeTimePrecision::Subsecond(subsecond) => {
                match subsecond {
                    DateTimeSubsecondDigits::S1 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S1))
                    }
                    DateTimeSubsecondDigits::S2 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S2))
                    }
                    DateTimeSubsecondDigits::S3 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S3))
                    }
                    DateTimeSubsecondDigits::S4 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S4))
                    }
                    DateTimeSubsecondDigits::S5 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S5))
                    }
                    DateTimeSubsecondDigits::S6 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S6))
                    }
                    DateTimeSubsecondDigits::S7 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S7))
                    }
                    DateTimeSubsecondDigits::S8 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S8))
                    }
                    DateTimeSubsecondDigits::S9 => {
                        quote!(l_i18n_crate::reexports::icu::datetime::options::TimePrecision::Subsecond(l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits::S9))
                    }
                }
            }
            DateTimeTimePrecision::MinuteOptional => {
                quote!(
                    l_i18n_crate::reexports::icu::datetime::options::TimePrecision::MinuteOptional
                )
            }
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = Self::to_token_stream(self);
        tokens.extend(ts);
    }
}

impl From<leptos_i18n_parser::utils::formatter::DateTimeTimePrecision> for DateTimeTimePrecision {
    fn from(value: leptos_i18n_parser::utils::formatter::DateTimeTimePrecision) -> Self {
        match value {
            leptos_i18n_parser::utils::formatter::DateTimeTimePrecision::Hour => Self::Hour,
            leptos_i18n_parser::utils::formatter::DateTimeTimePrecision::Minute => Self::Minute,
            leptos_i18n_parser::utils::formatter::DateTimeTimePrecision::Second => Self::Second,
            leptos_i18n_parser::utils::formatter::DateTimeTimePrecision::Subsecond(subsecond) => {
                match subsecond {
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S1 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S1)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S2 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S2)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S3 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S3)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S4 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S4)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S5 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S5)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S6 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S6)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S7 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S7)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S8 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S8)
                    }
                    leptos_i18n_parser::utils::formatter::DateTimeSubsecondDigits::S9 => {
                        Self::Subsecond(DateTimeSubsecondDigits::S9)
                    }
                }
            }
            leptos_i18n_parser::utils::formatter::DateTimeTimePrecision::MinuteOptional => {
                Self::MinuteOptional
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    And,
    Or,
    Unit,
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

impl_from!(ListType, And, Or, Unit);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListStyle {
    Wide,
    Short,
    Narrow,
}

impl_to_tokens!(
    ListStyle,
    l_i18n_crate::reexports::icu::list::options::ListLength,
    {
        Wide,
        Short,
        Narrow
    }
);

impl_from!(ListStyle, Wide, Short, Narrow);

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

impl From<leptos_i18n_parser::utils::formatter::Formatter> for Formatter {
    fn from(value: leptos_i18n_parser::utils::formatter::Formatter) -> Self {
        match value {
            leptos_i18n_parser::utils::formatter::Formatter::None => Self::None,
            leptos_i18n_parser::utils::formatter::Formatter::Dummy => Self::Dummy,
            leptos_i18n_parser::utils::formatter::Formatter::Currency(width, code) => {
                Self::Currency(width.into(), code.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::Number(grouping_strategy) => {
                Self::Number(grouping_strategy.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::Date(length, aligment, year_style) => {
                Self::Date(length.into(), aligment.into(), year_style.into())
            }
            leptos_i18n_parser::utils::formatter::Formatter::Time(
                length,
                alignment,
                time_precision,
            ) => Self::Time(length.into(), alignment.into(), time_precision.into()),
            leptos_i18n_parser::utils::formatter::Formatter::DateTime(
                length,
                alignment,
                time_precision,
                year_style,
            ) => Self::DateTime(
                length.into(),
                alignment.into(),
                time_precision.into(),
                year_style.into(),
            ),
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
