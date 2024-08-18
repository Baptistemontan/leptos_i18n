use super::key::Key;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateLength {
    Full,
    Long,
    #[default]
    Medium,
    Short,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeLength {
    Full,
    Long,
    Medium,
    #[default]
    Short,
}

fn from_args_helper<'a, T: Default, S: PartialEq + PartialEq<&'a str>>(
    args: Option<&[(S, S)]>,
    name: &'a str,
    f: impl Fn(&S) -> Option<T>,
) -> T {
    let Some(args) = args else {
        return Default::default();
    };
    for (arg_name, value) in args {
        if arg_name != &name {
            continue;
        }
        if let Some(v) = f(value) {
            return v;
        }
    }
    Default::default()
}

macro_rules! impl_from_args {
    ($name:literal, $($arg_name:literal => $value:expr,)*) => {
        pub fn from_args<'a, S: PartialEq + PartialEq<&'a str>>(args: Option<&[(S, S)]>) -> Self {
        from_args_helper(args, $name, |arg| {
            $(
                if arg == &$arg_name {
                    Some($value)
                } else
            )*
            {
                None
            }
        })
    }
    }
}

macro_rules! impl_length {
    ($t:ty, $arg_name:literal, $name:ident) => {
        impl $t {
            impl_from_args! {
                $arg_name,
                "full" => Self::Full,
                "long" => Self::Long,
                "medium" => Self::Medium,
                "short" => Self::Short,
            }
        }

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
    };
}

impl_length!(DateLength, "date_length", Date);
impl_length!(TimeLength, "time_length", Time);

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    And,
    Or,
    #[default]
    Unit,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListStyle {
    #[default]
    Wide,
    Short,
    Narrow,
}

impl ListType {
    impl_from_args! {
        "list_type",
        "and" => Self::And,
        "or" => Self::Or,
        "unit" => Self::Unit,
    }
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

impl ListStyle {
    impl_from_args! {
        "list_style",
        "wide" => Self::Wide,
        "short" => Self::Short,
        "narrow" => Self::Narrow,
    }
}

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

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formatter {
    #[default]
    None,
    Number,
    Date(DateLength),
    Time(TimeLength),
    DateTime(DateLength, TimeLength),
    List(ListType, ListStyle),
}

impl Formatter {
    pub fn from_name_and_args<'a, S: PartialEq + PartialEq<&'a str>>(
        name: S,
        args: Option<&[(S, S)]>,
    ) -> Option<Formatter> {
        if name == "number" {
            Some(Formatter::Number)
        } else if name == "datetime" {
            Some(Formatter::DateTime(
                DateLength::from_args(args),
                TimeLength::from_args(args),
            ))
        } else if name == "date" {
            Some(Formatter::Date(DateLength::from_args(args)))
        } else if name == "time" {
            Some(Formatter::Time(TimeLength::from_args(args)))
        } else if name == "list" {
            Some(Formatter::List(
                ListType::from_args(args),
                ListStyle::from_args(args),
            ))
        } else {
            None
        }
    }

    pub fn var_to_view(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Formatter::None => {
                quote!(#key)
            }
            Formatter::Number => {
                quote!(l_i18n_crate::__private::format_number_to_view(#locale_field, #key))
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
            Formatter::None => unreachable!(),
            Formatter::Number => {
                quote!(l_i18n_crate::__private::format_number_to_display(#locale_field, #key))
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
            Formatter::Number => {
                quote!(l_i18n_crate::__private::format_number_to_formatter(__formatter, *#locale_field, core::clone::Clone::clone(#key)))
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
            Formatter::Number => quote!(l_i18n_crate::__private::NumberFormatterInputFn),
            Formatter::Date(_) => quote!(l_i18n_crate::__private::DateFormatterInputFn),
            Formatter::Time(_) => quote!(l_i18n_crate::__private::TimeFormatterInputFn),
            Formatter::DateTime(_, _) => quote!(l_i18n_crate::__private::DateTimeFormatterInputFn),
            Formatter::List(_, _) => quote!(l_i18n_crate::__private::ListFormatterInputFn),
        }
    }

    pub fn to_string_bound(self) -> TokenStream {
        match self {
            Formatter::None => quote!(::std::fmt::Display),
            Formatter::Number => quote!(l_i18n_crate::__private::IntoFixedDecimal),
            Formatter::Date(_) => quote!(l_i18n_crate::__private::AsIcuDate),
            Formatter::Time(_) => quote!(l_i18n_crate::__private::AsIcuTime),
            Formatter::DateTime(_, _) => quote!(l_i18n_crate::__private::AsIcuDateTime),
            Formatter::List(_, _) => quote!(l_i18n_crate::__private::WriteableList),
        }
    }
}
