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

fn from_args_helper<T: Default>(
    args: Option<&[(&str, &str)]>,
    name: &str,
    f: impl Fn(&str) -> Option<T>,
) -> T {
    let Some(args) = args else {
        return Default::default();
    };
    for (arg_name, value) in args {
        if *arg_name != name {
            continue;
        }
        if let Some(v) = f(value) {
            return v;
        }
    }
    Default::default()
}

macro_rules! impl_length {
    ($t:ty, $arg_name:pat, $name:ident) => {
        impl $t {
            pub fn from_args(args: Option<&[(&str, &str)]>) -> Self {
                let Some(args) = args else {
                    return Default::default();
                };
                for arg in args {
                    match *arg {
                        ($arg_name, "full") => return Self::Full,
                        ($arg_name, "long") => return Self::Long,
                        ($arg_name, "medium") => return Self::Medium,
                        ($arg_name, "short") => return Self::Short,
                        _ => {}
                    }
                }
                Default::default()
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

impl_length!(DateLength, "date_length" | "length", Date);
impl_length!(TimeLength, "time_length" | "length", Time);

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    #[default]
    And,
    Or,
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
    pub fn from_args(args: Option<&[(&str, &str)]>) -> Self {
        from_args_helper(args, "type", |arg| match arg {
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "unit" => Some(Self::Unit),
            _ => None,
        })
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
    pub fn from_args(args: Option<&[(&str, &str)]>) -> Self {
        from_args_helper(args, "style", |arg| match arg {
            "wide" => Some(Self::Wide),
            "short" => Some(Self::Short),
            "narrow" => Some(Self::Narrow),
            _ => None,
        })
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
    pub fn var_into_view(self, key: &Key, locale_field: &Key) -> TokenStream {
        match self {
            Formatter::None => {
                quote!(leptos::IntoView::into_view(core::clone::Clone::clone(&#key)))
            }
            Formatter::Number => {
                quote!(leptos::IntoView::into_view(l_i18n_crate::__private::format_number_to_string(#locale_field, core::clone::Clone::clone(&#key))))
            }
            Formatter::Date(length) => {
                quote!(leptos::IntoView::into_view(l_i18n_crate::__private::format_date_to_string(#locale_field, core::clone::Clone::clone(&#key), #length)))
            }
            Formatter::Time(length) => {
                quote!(leptos::IntoView::into_view(l_i18n_crate::__private::format_time_to_string(#locale_field, core::clone::Clone::clone(&#key), #length)))
            }
            Formatter::DateTime(date_length, time_length) => {
                quote!(leptos::IntoView::into_view(l_i18n_crate::__private::format_datetime_to_string(#locale_field, core::clone::Clone::clone(&#key), #date_length, #time_length)))
            }
            Formatter::List(list_type, list_style) => {
                quote!(leptos::IntoView::into_view(l_i18n_crate::__private::format_list_to_string(#locale_field, core::clone::Clone::clone(&#key), #list_type, #list_style)))
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
