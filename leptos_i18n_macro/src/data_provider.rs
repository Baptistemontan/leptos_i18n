use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_icu_data_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if cfg!(feature = "icu_compiled_data") {
        let ts = quote! {
            std::compile_error!("Implementing this trait is useless with the \"icu_compiled_data\" feature enabled.");
        };

        return proc_macro::TokenStream::from(ts);
    }

    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let new_num_formatter = if cfg!(feature = "format_nums") {
        quote! {
            fn try_new_num_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::decimal::options::FixedDecimalFormatterOptions
            ) -> Result<leptos_i18n::reexports::icu::decimal::FixedDecimalFormatter, leptos_i18n::reexports::icu::decimal::DecimalError> {
                leptos_i18n::reexports::icu::decimal::FixedDecimalFormatter::try_new_unstable(self, locale, options)
            }
        }
    } else {
        quote!()
    };

    let new_datetime_formatter = if cfg!(feature = "format_datetime") {
        quote! {
            fn try_new_date_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::datetime::options::length::Date
            ) -> Result<leptos_i18n::reexports::icu::datetime::DateFormatter, leptos_i18n::reexports::icu::datetime::DateTimeError> {
                leptos_i18n::reexports::icu::datetime::DateFormatter::try_new_with_length_unstable(self, locale, options)
            }

            fn try_new_time_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::datetime::options::length::Time
            ) -> Result<leptos_i18n::reexports::icu::datetime::TimeFormatter, leptos_i18n::reexports::icu::datetime::DateTimeError> {
                leptos_i18n::reexports::icu::datetime::TimeFormatter::try_new_with_length_unstable(self, locale, options)
            }

            fn try_new_datetime_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::datetime::options::DateTimeFormatterOptions
            ) -> Result<leptos_i18n::reexports::icu::datetime::DateTimeFormatter, leptos_i18n::reexports::icu::datetime::DateTimeError> {
                leptos_i18n::reexports::icu::datetime::DateTimeFormatter::try_new_unstable(self, locale, options)
            }
        }
    } else {
        quote!()
    };

    let new_list_formatter = if cfg!(feature = "format_list") {
        quote! {
            fn try_new_and_list_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::list::ListLength
            ) -> Result<leptos_i18n::reexports::icu::list::ListFormatter, leptos_i18n::reexports::icu::list::ListError> {
                leptos_i18n::reexports::icu::list::ListFormatter::try_new_and_with_length_unstable(self, locale, options)
            }

            fn try_new_or_list_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::list::ListLength
            ) -> Result<leptos_i18n::reexports::icu::list::ListFormatter, leptos_i18n::reexports::icu::list::ListError> {
                leptos_i18n::reexports::icu::list::ListFormatter::try_new_or_with_length_unstable(self, locale, options)
            }

            fn try_new_unit_list_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::list::ListLength
            ) -> Result<leptos_i18n::reexports::icu::list::ListFormatter, leptos_i18n::reexports::icu::list::ListError> {
                leptos_i18n::reexports::icu::list::ListFormatter::try_new_unit_with_length_unstable(self, locale, options)
            }
        }
    } else {
        quote!()
    };

    let new_plural_rules = if cfg!(feature = "plurals") {
        quote! {
            fn try_new_plural_rules(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::plurals::PluralRuleType
            ) -> Result<leptos_i18n::reexports::icu::plurals::PluralRules, leptos_i18n::reexports::icu::plurals::PluralsError> {
                leptos_i18n::reexports::icu::plurals::PluralRules::try_new_unstable(self, locale, options)
            }
        }
    } else {
        quote!()
    };

    let new_currency_formatter = if cfg!(feature = "format_currency") {
        quote! {
                fn try_new_currency_formatter(
                &self,
                locale: &leptos_i18n::reexports::icu::provider::DataLocale,
                options: leptos_i18n::reexports::icu::currency::options::CurrencyFormatterOptions
            ) -> Result<leptos_i18n::reexports::icu::currency::formatter::CurrencyFormatter, leptos_i18n::reexports::icu::provider::DataError> {
                leptos_i18n::reexports::icu::currency::formatter::CurrencyFormatter::try_new_unstable(self, locale, options)
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
