use super::{Formatter, FormatterToTokens};
use super::{impl_formatter, impl_from_arg, impl_to_tokens};
use crate::utils::Key;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub struct DateTimeFormatterParser;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTimeFormatter(
    DateTimeLength,
    DateTimeAlignment,
    DateTimeTimePrecision,
    DateTimeYearStyle,
);

impl_formatter!(
    DateTimeFormatterParser,
    "datetime",
    DateTimeFormatterBuilder,
    DateTimeFormatter(
        length => DateTimeLength,
        alignment => DateTimeAlignment,
        time_precision => DateTimeTimePrecision,
        year_style => DateTimeYearStyle
    ),
    "format_datetime",
    "Formatting datetime is not enabled, enable the \"format_datetime\" feature to do so"
);

impl FormatterToTokens for DateTimeFormatter {
    fn view_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::DateTimeFormatterInputFn)
    }
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(length, alignment, time_precision, year_style) = self;
        quote!(l_i18n_crate::__private::format_datetime_to_view(#locale_field, #key, #length, #alignment, #time_precision, #year_style))
    }

    fn fmt_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::AsIcuDateTime)
    }

    fn to_impl_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(length, alignment, time_precision, year_style) = self;
        quote!(l_i18n_crate::__private::format_datetime_to_display(#locale_field, #key, #length, #alignment, #time_precision, #year_style))
    }

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        let Self(length, alignment, time_precision, year_style) = self;
        quote!(l_i18n_crate::__private::format_datetime_to_formatter(__formatter, *#locale_field, #key, #length, #alignment, #time_precision, #year_style))
    }
}

pub struct DateFormatterParser;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateFormatter(DateTimeLength, DateTimeAlignment, DateTimeYearStyle);

impl_formatter!(
    DateFormatterParser,
    "date",
    DateFormatterBuilder,
    DateFormatter(length => DateTimeLength, alignment => DateTimeAlignment, year_style => DateTimeYearStyle),
    "format_datetime",
    "Formatting date is not enabled, enable the \"format_datetime\" feature to do so"
);

impl FormatterToTokens for DateFormatter {
    fn view_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::DateFormatterInputFn)
    }
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(length, alignment, year_style) = self;
        quote!(l_i18n_crate::__private::format_date_to_view(#locale_field, #key, #length, #alignment, #year_style))
    }

    fn fmt_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::AsIcuDate)
    }

    fn to_impl_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(length, alignment, year_style) = self;
        quote!(l_i18n_crate::__private::format_date_to_display(#locale_field, #key, #length, #alignment, #year_style))
    }

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        let Self(length, alignment, year_style) = self;
        quote!(l_i18n_crate::__private::format_date_to_formatter(__formatter, *#locale_field, #key, #length, #alignment, #year_style))
    }
}

pub struct TimeFormatterParser;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeFormatter(DateTimeLength, DateTimeAlignment, DateTimeTimePrecision);

impl_formatter!(
    TimeFormatterParser,
    "time",
    TimeFormatterBuilder,
    TimeFormatter(length => DateTimeLength, alignment => DateTimeAlignment, time_precision => DateTimeTimePrecision),
    "format_datetime",
    "Formatting time is not enabled, enable the \"format_datetime\" feature to do so"
);

impl FormatterToTokens for TimeFormatter {
    fn view_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::TimeFormatterInputFn)
    }
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(length, alignment, precision) = self;
        quote!(l_i18n_crate::__private::format_time_to_view(#locale_field, #key, #length, #alignment, #precision))
    }

    fn fmt_bounds(&self) -> TokenStream {
        quote!(l_i18n_crate::__private::AsIcuTime)
    }

    fn to_impl_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        let Self(length, alignment, precision) = self;
        quote!(l_i18n_crate::__private::format_time_to_display(#locale_field, #key, #length, #alignment, #precision))
    }

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        let Self(length, alignment, precision) = self;
        quote!(l_i18n_crate::__private::format_time_to_formatter(__formatter, *#locale_field, #key, #length, #alignment, #precision))
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DateTimeLength {
    Long,
    #[default]
    Medium,
    Short,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DateTimeAlignment {
    #[default]
    Auto,
    Column,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DateTimeTimePrecision {
    Hour,
    Minute,
    #[default]
    Second,
    Subsecond(DateTimeSubsecondDigits),
    MinuteOptional,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DateTimeSubsecondDigits {
    S1 = 1,
    S2 = 2,
    #[default]
    S3 = 3,
    S4 = 4,
    S5 = 5,
    S6 = 6,
    S7 = 7,
    S8 = 8,
    S9 = 9,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DateTimeYearStyle {
    #[default]
    Auto,
    Full,
    WithEra,
}

impl DateTimeLength {
    impl_from_arg! {
        "long" => Self::Long,
        "medium" => Self::Medium,
        "short" => Self::Short,
    }
}

impl DateTimeAlignment {
    impl_from_arg! {
        "auto" => Self::Auto,
        "column" => Self::Column,
    }
}

impl DateTimeYearStyle {
    impl_from_arg! {
        "auto" => Self::Auto,
        "full" => Self::Full,
        "with_era" => Self::WithEra,
    }
}

impl DateTimeTimePrecision {
    impl_from_arg! {
        "hour" => Self::Hour,
        "minute" => Self::Minute,
        "second" => Self::Second,
        "subsecond_s1" => Self::Subsecond(DateTimeSubsecondDigits::S1),
        "subsecond_s2" => Self::Subsecond(DateTimeSubsecondDigits::S2),
        "subsecond_s3" => Self::Subsecond(DateTimeSubsecondDigits::S3),
        "subsecond_s4" => Self::Subsecond(DateTimeSubsecondDigits::S4),
        "subsecond_s5" => Self::Subsecond(DateTimeSubsecondDigits::S5),
        "subsecond_s6" => Self::Subsecond(DateTimeSubsecondDigits::S6),
        "subsecond_s7" => Self::Subsecond(DateTimeSubsecondDigits::S7),
        "subsecond_s8" => Self::Subsecond(DateTimeSubsecondDigits::S8),
        "subsecond_s9" => Self::Subsecond(DateTimeSubsecondDigits::S9),
        "minute_optional" => Self::MinuteOptional,
    }
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

impl_to_tokens!(
    DateTimeAlignment,
    l_i18n_crate::reexports::icu::datetime::options::Alignment,
    {
        Auto,
        Column
    }
);

impl_to_tokens!(
    DateTimeYearStyle,
    l_i18n_crate::reexports::icu::datetime::options::YearStyle,
    {
        Auto,
        Full,
        WithEra
    }
);

impl_to_tokens!(
    DateTimeSubsecondDigits,
    l_i18n_crate::reexports::icu::datetime::options::SubsecondDigits,
    {
        S1, S2, S3, S4, S5, S6, S7, S8, S9,
    }
);

impl_to_tokens!(
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
