use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_icu_data_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let new_num_formatter = if cfg!(feature = "format_nums") {
        quote! {
            fn try_new_num_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                options: leptos_i18n::reexports::icu::decimal::options::DecimalFormatterOptions
            ) -> Result<leptos_i18n::reexports::icu::decimal::DecimalFormatter, leptos_i18n::reexports::icu::provider::DataError> {
                leptos_i18n::reexports::icu::decimal::DecimalFormatter::try_new_unstable(self, locale.into(), options)
            }
        }
    } else {
        quote!()
    };

    let new_datetime_formatter = if cfg!(feature = "format_datetime") {
        quote! {
            fn try_new_date_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                length: leptos_i18n::reexports::icu::datetime::options::Length,
                alignment: leptos_i18n::reexports::icu::datetime::options::Alignment,
                year_style: leptos_i18n::reexports::icu::datetime::options::YearStyle,
            ) -> Result<leptos_i18n::reexports::icu::datetime::DateTimeFormatter<leptos_i18n::reexports::icu::datetime::fieldsets::YMD>, leptos_i18n::reexports::icu::datetime::DateTimeFormatterLoadError> {

                let fset = leptos_i18n::reexports::icu::datetime::fieldsets::YMD::for_length(length)
                    .with_alignment(alignment).with_year_style(year_style);
                leptos_i18n::reexports::icu::datetime::DateTimeFormatter::try_new_unstable(self, locale.into(), fset)
            }

            fn try_new_time_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                length: leptos_i18n::reexports::icu::datetime::options::Length,
                alignment: leptos_i18n::reexports::icu::datetime::options::Alignment,
                time_precision: leptos_i18n::reexports::icu::datetime::options::TimePrecision,
            ) -> Result<leptos_i18n::reexports::icu::datetime::NoCalendarFormatter<leptos_i18n::reexports::icu::datetime::fieldsets::T>, leptos_i18n::reexports::icu::datetime::DateTimeFormatterLoadError> {
                let fset = leptos_i18n::reexports::icu::datetime::fieldsets::T::for_length(length)
                    .with_alignment(alignment)
                    .with_time_precision(time_precision);
                leptos_i18n::reexports::icu::datetime::NoCalendarFormatter::try_new_unstable(self, locale.into(), fset)
            }

            fn try_new_datetime_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                length: leptos_i18n::reexports::icu::datetime::options::Length,
                alignment: leptos_i18n::reexports::icu::datetime::options::Alignment,
                time_precision: leptos_i18n::reexports::icu::datetime::options::TimePrecision,
                year_style: leptos_i18n::reexports::icu::datetime::options::YearStyle,
            ) -> Result<leptos_i18n::reexports::icu::datetime::DateTimeFormatter<leptos_i18n::reexports::icu::datetime::fieldsets::YMDT>, leptos_i18n::reexports::icu::datetime::DateTimeFormatterLoadError> {
                let fset = leptos_i18n::reexports::icu::datetime::fieldsets::YMDT::for_length(length)
                    .with_alignment(alignment)
                    .with_time_precision(time_precision).with_year_style(year_style);
                leptos_i18n::reexports::icu::datetime::DateTimeFormatter::try_new_unstable(self, locale.into(), fset)
            }
        }
    } else {
        quote!()
    };

    let new_list_formatter = if cfg!(feature = "format_list") {
        quote! {
            fn try_new_and_list_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                length: leptos_i18n::reexports::icu::list::options::ListLength
            ) -> Result<leptos_i18n::reexports::icu::list::ListFormatter, leptos_i18n::reexports::icu::provider::DataError> {
                let options = leptos_i18n::reexports::icu::list::options::ListFormatterOptions::default().with_length(length);
                leptos_i18n::reexports::icu::list::ListFormatter::try_new_and_unstable(self, locale.into(), options)
            }

            fn try_new_or_list_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                length: leptos_i18n::reexports::icu::list::options::ListLength
            ) -> Result<leptos_i18n::reexports::icu::list::ListFormatter, leptos_i18n::reexports::icu::provider::DataError> {
                let options = leptos_i18n::reexports::icu::list::options::ListFormatterOptions::default().with_length(length);
                leptos_i18n::reexports::icu::list::ListFormatter::try_new_or_unstable(self, locale.into(), options)
            }

            fn try_new_unit_list_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                length: leptos_i18n::reexports::icu::list::options::ListLength
            ) -> Result<leptos_i18n::reexports::icu::list::ListFormatter, leptos_i18n::reexports::icu::provider::DataError> {
                let options = leptos_i18n::reexports::icu::list::options::ListFormatterOptions::default().with_length(length);
                leptos_i18n::reexports::icu::list::ListFormatter::try_new_unit_unstable(self, locale.into(), options)
            }
        }
    } else {
        quote!()
    };

    let new_plural_rules = if cfg!(feature = "plurals") {
        quote! {
            fn try_new_plural_rules(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                rule_type: leptos_i18n::reexports::icu::plurals::PluralRuleType
            ) -> Result<leptos_i18n::reexports::icu::plurals::PluralRules, leptos_i18n::reexports::icu::provider::DataError> {
                let options = leptos_i18n::reexports::icu::plurals::PluralRulesOptions::default().with_type(rule_type);
                leptos_i18n::reexports::icu::plurals::PluralRules::try_new_unstable(self, locale.into(), options)
            }
        }
    } else {
        quote!()
    };

    let new_currency_formatter = if cfg!(feature = "format_currency") {
        quote! {
                fn try_new_currency_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::locid::Locale,
                options: leptos_i18n::reexports::icu::currency::options::CurrencyFormatterOptions
            ) -> Result<leptos_i18n::reexports::icu::currency::formatter::CurrencyFormatter, leptos_i18n::reexports::icu::provider::DataError> {
                leptos_i18n::reexports::icu::currency::formatter::CurrencyFormatter::try_new_unstable(self, locale.into(), options)
            }
        }
    } else {
        quote!()
    };

    let expanded = quote! {
        impl #impl_generics leptos_i18n::custom_provider::IcuDataProvider for #name #ty_generics #where_clause {

            #new_num_formatter

            #new_datetime_formatter

            #new_list_formatter

            #new_plural_rules

            #new_currency_formatter
        }
    };

    proc_macro::TokenStream::from(expanded)
}
