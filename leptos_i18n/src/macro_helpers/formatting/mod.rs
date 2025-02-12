//! This module contain traits and helper functions for formatting
//! different kind of value based on a locale.

#[cfg(feature = "format_currency")]
mod currency;
#[cfg(feature = "format_datetime")]
mod date;
#[cfg(feature = "format_datetime")]
mod datetime;
#[cfg(feature = "format_list")]
mod list;
#[cfg(feature = "format_nums")]
mod nums;
#[cfg(feature = "format_datetime")]
mod time;

#[cfg(feature = "format_currency")]
pub use currency::*;
#[cfg(feature = "format_datetime")]
pub use date::*;
#[cfg(feature = "format_datetime")]
pub use datetime::*;
#[cfg(feature = "format_datetime")]
use icu_datetime::{options::length, DateFormatter, DateTimeFormatter, TimeFormatter};
#[cfg(feature = "format_list")]
use icu_list::{ListFormatter, ListLength};
#[cfg(feature = "plurals")]
use icu_plurals::{PluralRuleType, PluralRules};
#[cfg(feature = "format_list")]
pub use list::*;
#[cfg(feature = "format_nums")]
pub use nums::*;
#[cfg(feature = "format_datetime")]
pub use time::*;

#[cfg(any(
    feature = "format_nums",
    feature = "format_datetime",
    feature = "format_list",
    feature = "plurals",
    feature = "format_currency",
))]
use crate::Locale;
#[cfg(feature = "format_nums")]
use icu_decimal::options::FixedDecimalFormatterOptions;
#[cfg(feature = "format_nums")]
use icu_decimal::options::GroupingStrategy;
#[cfg(feature = "format_nums")]
use icu_decimal::FixedDecimalFormatter;
#[cfg(feature = "format_currency")]
use icu_experimental::dimension::currency::formatter::CurrencyFormatter;
#[cfg(feature = "format_currency")]
use icu_experimental::dimension::currency::options::CurrencyFormatterOptions;
#[cfg(feature = "format_currency")]
use icu_experimental::dimension::currency::options::Width as CurrencyWidth;

pub use leptos_i18n_macro::{
    t_format, t_format_display, t_format_string, td_format, td_format_display, td_format_string,
    tu_format, tu_format_display, tu_format_string,
};

#[cfg(feature = "format_currency")]
fn get_currency_formatter<L: Locale>(
    locale: L,
    width: CurrencyWidth,
) -> &'static CurrencyFormatter {
    use data_provider::IcuDataProvider;

    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let currency_formatters = formatters.currency.entry(locale).or_default();
        let currency_formatter = currency_formatters.entry(width.into()).or_insert_with(|| {
            let formatter = formatters
                .provider
                .try_new_currency_formatter(&locale.into(), CurrencyFormatterOptions::from(width))
                .expect("A CurrencyFormatter");
            Box::leak(Box::new(formatter))
        });
        *currency_formatter
    })
}

#[cfg(feature = "format_nums")]
fn get_num_formatter<L: Locale>(
    locale: L,
    grouping_strategy: GroupingStrategy,
) -> &'static FixedDecimalFormatter {
    use data_provider::IcuDataProvider;

    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let num_formatters = formatters.num.entry(locale).or_default();
        let num_formatter = num_formatters.entry(grouping_strategy).or_insert_with(|| {
            let formatter = formatters
                .provider
                .try_new_num_formatter(
                    &locale.into(),
                    FixedDecimalFormatterOptions::from(grouping_strategy),
                )
                .expect("A FixedDecimalFormatter");
            Box::leak(Box::new(formatter))
        });
        *num_formatter
    })
}

#[cfg(feature = "format_datetime")]
fn get_date_formatter<L: Locale>(locale: L, length: length::Date) -> &'static DateFormatter {
    use data_provider::IcuDataProvider;

    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let date_formatters = formatters.date.entry(locale).or_default();
        let date_formatter = date_formatters.entry(length).or_insert_with(|| {
            let formatter = formatters
                .provider
                .try_new_date_formatter(&locale.into(), length)
                .expect("A DateFormatter");
            Box::leak(Box::new(formatter))
        });
        *date_formatter
    })
}

#[cfg(feature = "format_datetime")]
fn get_time_formatter<L: Locale>(locale: L, length: length::Time) -> &'static TimeFormatter {
    use data_provider::IcuDataProvider;

    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let time_formatters = formatters.time.entry(locale).or_default();
        let time_formatter = time_formatters.entry(length).or_insert_with(|| {
            let formatter = formatters
                .provider
                .try_new_time_formatter(&locale.into(), length)
                .expect("A TimeFormatter");
            Box::leak(Box::new(formatter))
        });
        *time_formatter
    })
}

#[cfg(feature = "format_datetime")]
fn get_datetime_formatter<L: Locale>(
    locale: L,
    date_length: length::Date,
    time_length: length::Time,
) -> &'static DateTimeFormatter {
    use data_provider::IcuDataProvider;

    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let datetime_formatters = formatters.datetime.entry(locale).or_default();
        let datetime_formatter = datetime_formatters
            .entry((date_length, time_length))
            .or_insert_with(|| {
                let options = length::Bag::from_date_time_style(date_length, time_length);
                let formatter = formatters
                    .provider
                    .try_new_datetime_formatter(&locale.into(), options.into())
                    .expect("A DateTimeFormatter");
                Box::leak(Box::new(formatter))
            });
        *datetime_formatter
    })
}

#[cfg(feature = "format_list")]
fn get_list_formatter<L: Locale>(
    locale: L,
    list_type: list::ListType,
    length: ListLength,
) -> &'static ListFormatter {
    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let list_formatters = formatters.list.entry(locale).or_default();
        let list_formatter = list_formatters
            .entry((list_type, length))
            .or_insert_with(|| {
                let formatter = list_type.new_formatter(&formatters.provider, locale, length);
                Box::leak(Box::new(formatter))
            });
        *list_formatter
    })
}

#[cfg(feature = "plurals")]
#[doc(hidden)]
pub fn get_plural_rules<L: Locale>(
    locale: L,
    plural_rule_type: PluralRuleType,
) -> &'static PluralRules {
    use data_provider::IcuDataProvider;

    inner::FORMATTERS.with_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let plural_rules = formatters.plural_rule.entry(locale).or_default();
        let plural_rules = plural_rules.entry(plural_rule_type).or_insert_with(|| {
            let plural_rules = formatters
                .provider
                .try_new_plural_rules(&locale.into(), plural_rule_type)
                .expect("A PluralRules");
            Box::leak(Box::new(plural_rules))
        });
        *plural_rules
    })
}

#[cfg(any(
    feature = "format_nums",
    feature = "format_datetime",
    feature = "format_list",
    feature = "plurals",
    feature = "format_currency",
))]
pub(crate) mod inner {
    use super::*;
    use icu_locid::Locale as IcuLocale;
    use std::{
        collections::HashMap,
        sync::{OnceLock, RwLock},
    };

    // Formatters cache
    //
    // The reason we leak the formatter is so that we can get a static ref,
    // making possible to return values borrowing from the formatter,
    // such as all *Formatter::format(..) returned values.
    pub static FORMATTERS: StaticLock<Formatters> = StaticLock::new();

    #[derive(Debug, Default)]
    #[repr(transparent)]
    pub struct StaticLock<T>(OnceLock<RwLock<T>>);

    impl<T> StaticLock<T> {
        pub const fn new() -> Self {
            StaticLock(OnceLock::new())
        }

        pub fn with_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> U
        where
            T: Default,
        {
            let mutex = self.0.get_or_init(Default::default);
            let mut guard = mutex.write().unwrap();
            f(&mut guard)
        }
    }
    #[derive(Default)]
    pub struct Formatters {
        #[cfg(feature = "format_currency")]
        pub currency: HashMap<
            &'static IcuLocale,
            HashMap<super::currency::Width, &'static CurrencyFormatter>,
        >,
        #[cfg(feature = "format_nums")]
        pub num:
            HashMap<&'static IcuLocale, HashMap<GroupingStrategy, &'static FixedDecimalFormatter>>,
        #[cfg(feature = "format_datetime")]
        pub date: HashMap<&'static IcuLocale, HashMap<length::Date, &'static DateFormatter>>,
        #[cfg(feature = "format_datetime")]
        pub time: HashMap<&'static IcuLocale, HashMap<length::Time, &'static TimeFormatter>>,
        #[cfg(feature = "format_datetime")]
        pub datetime: HashMap<
            &'static IcuLocale,
            HashMap<(length::Date, length::Time), &'static DateTimeFormatter>,
        >,
        #[cfg(feature = "format_list")]
        pub list: HashMap<
            &'static IcuLocale,
            HashMap<(list::ListType, ListLength), &'static ListFormatter>,
        >,
        #[cfg(feature = "plurals")]
        pub plural_rule: HashMap<&'static IcuLocale, HashMap<PluralRuleType, &'static PluralRules>>,
        pub provider: data_provider::BakedDataProvider,
    }

    /// Supply a custom ICU data provider
    /// Does nothing if the "icu_compiled_data" feature is enabled.
    pub fn set_icu_data_provider(data_provider: impl super::data_provider::IcuDataProvider) {
        #[cfg(feature = "icu_compiled_data")]
        let _ = data_provider;
        #[cfg(not(feature = "icu_compiled_data"))]
        inner::FORMATTERS.with_mut(|formatters| {
            formatters.provider =
                super::data_provider::BakedDataProvider(Some(Box::new(data_provider)));
        });
    }
}

#[cfg(not(any(
    feature = "format_nums",
    feature = "format_datetime",
    feature = "format_list",
    feature = "plurals",
    feature = "format_currency",
)))]
pub(crate) mod inner {
    /// Supply a custom ICU data provider
    /// Does nothing if the "icu_compiled_data" feature is enabled.
    pub fn set_icu_data_provider(data_provider: impl super::data_provider::IcuDataProvider) {
        let _ = data_provider;
    }
}

pub(crate) mod data_provider {
    #[cfg(any(
        feature = "format_nums",
        feature = "format_datetime",
        feature = "format_list",
        feature = "plurals",
        feature = "format_currency",
    ))]
    use super::*;

    #[cfg(any(
        feature = "format_nums",
        feature = "format_datetime",
        feature = "format_list",
        feature = "plurals",
        feature = "format_currency",
    ))]
    use icu_provider::DataLocale;

    /// Trait for custom ICU data providers.
    pub trait IcuDataProvider: Send + Sync + 'static {
        /// Tries to create a new `FixedDecimalFormatter` with the given options
        #[cfg(feature = "format_nums")]
        fn try_new_num_formatter(
            &self,
            locale: &DataLocale,
            options: icu_decimal::options::FixedDecimalFormatterOptions,
        ) -> Result<FixedDecimalFormatter, icu_decimal::DecimalError>;

        /// Tries to create a new `DateFormatter` with the given options
        #[cfg(feature = "format_datetime")]
        fn try_new_date_formatter(
            &self,
            locale: &DataLocale,
            length: length::Date,
        ) -> Result<DateFormatter, icu_datetime::DateTimeError>;

        /// Tries to create a new `TimeFormatter` with the given options
        #[cfg(feature = "format_datetime")]
        fn try_new_time_formatter(
            &self,
            locale: &DataLocale,
            length: length::Time,
        ) -> Result<TimeFormatter, icu_datetime::DateTimeError>;

        /// Tries to create a new `DateTimeFormatter` with the given options
        #[cfg(feature = "format_datetime")]
        fn try_new_datetime_formatter(
            &self,
            locale: &DataLocale,
            options: icu_datetime::options::DateTimeFormatterOptions,
        ) -> Result<DateTimeFormatter, icu_datetime::DateTimeError>;

        /// Tries to create a and `ListFormatter` with the given options
        #[cfg(feature = "format_list")]
        fn try_new_and_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError>;

        /// Tries to create a new or `ListFormatter` with the given options
        #[cfg(feature = "format_list")]
        fn try_new_or_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError>;

        /// Tries to create a new unit `ListFormatter` with the given options
        #[cfg(feature = "format_list")]
        fn try_new_unit_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError>;

        /// Tries to create a new `PluralRules` with the given options
        #[cfg(feature = "plurals")]
        fn try_new_plural_rules(
            &self,
            locale: &DataLocale,
            rule_type: PluralRuleType,
        ) -> Result<PluralRules, icu_plurals::PluralsError>;
        ///
        /// Tries to create a new `CurrencyFormatter` with the given options
        #[cfg(feature = "format_currency")]
        fn try_new_currency_formatter(
            &self,
            locale: &DataLocale,
            options: CurrencyFormatterOptions,
        ) -> Result<CurrencyFormatter, icu_provider::DataError>;
    }

    #[cfg(feature = "icu_compiled_data")]
    #[derive(Default)]
    pub struct BakedDataProvider;

    #[cfg(feature = "icu_compiled_data")]
    impl IcuDataProvider for BakedDataProvider {
        #[cfg(feature = "format_nums")]
        fn try_new_num_formatter(
            &self,
            locale: &DataLocale,
            options: icu_decimal::options::FixedDecimalFormatterOptions,
        ) -> Result<FixedDecimalFormatter, icu_decimal::DecimalError> {
            FixedDecimalFormatter::try_new(locale, options)
        }

        #[cfg(feature = "format_datetime")]
        fn try_new_date_formatter(
            &self,
            locale: &DataLocale,
            length: length::Date,
        ) -> Result<DateFormatter, icu_datetime::DateTimeError> {
            DateFormatter::try_new_with_length(locale, length)
        }

        #[cfg(feature = "format_datetime")]
        fn try_new_time_formatter(
            &self,
            locale: &DataLocale,
            length: length::Time,
        ) -> Result<TimeFormatter, icu_datetime::DateTimeError> {
            TimeFormatter::try_new_with_length(locale, length)
        }

        #[cfg(feature = "format_datetime")]
        fn try_new_datetime_formatter(
            &self,
            locale: &DataLocale,
            options: icu_datetime::options::DateTimeFormatterOptions,
        ) -> Result<DateTimeFormatter, icu_datetime::DateTimeError> {
            DateTimeFormatter::try_new(locale, options)
        }

        #[cfg(feature = "format_list")]
        fn try_new_and_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError> {
            ListFormatter::try_new_and_with_length(locale, style)
        }

        #[cfg(feature = "format_list")]
        fn try_new_or_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError> {
            ListFormatter::try_new_or_with_length(locale, style)
        }

        #[cfg(feature = "format_list")]
        fn try_new_unit_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError> {
            ListFormatter::try_new_unit_with_length(locale, style)
        }

        #[cfg(feature = "plurals")]
        fn try_new_plural_rules(
            &self,
            locale: &DataLocale,
            rule_type: PluralRuleType,
        ) -> Result<PluralRules, icu_plurals::PluralsError> {
            PluralRules::try_new(locale, rule_type)
        }

        #[cfg(feature = "format_currency")]
        fn try_new_currency_formatter(
            &self,
            locale: &DataLocale,
            options: CurrencyFormatterOptions,
        ) -> Result<CurrencyFormatter, icu_provider::DataError> {
            CurrencyFormatter::try_new(locale, options)
        }
    }

    #[cfg(not(feature = "icu_compiled_data"))]
    #[derive(Default)]
    pub struct BakedDataProvider(pub Option<Box<dyn IcuDataProvider>>);

    #[cfg(not(feature = "icu_compiled_data"))]
    impl BakedDataProvider {
        #[allow(dead_code)]
        fn get_provider(&self) -> &dyn IcuDataProvider {
            self.0.as_deref().expect("No DataProvider provided.")
        }
    }

    #[cfg(not(feature = "icu_compiled_data"))]
    impl IcuDataProvider for BakedDataProvider {
        #[cfg(feature = "format_nums")]
        fn try_new_num_formatter(
            &self,
            locale: &DataLocale,
            options: icu_decimal::options::FixedDecimalFormatterOptions,
        ) -> Result<FixedDecimalFormatter, icu_decimal::DecimalError> {
            self.get_provider().try_new_num_formatter(locale, options)
        }

        #[cfg(feature = "format_datetime")]
        fn try_new_date_formatter(
            &self,
            locale: &DataLocale,
            length: length::Date,
        ) -> Result<DateFormatter, icu_datetime::DateTimeError> {
            self.get_provider().try_new_date_formatter(locale, length)
        }

        #[cfg(feature = "format_datetime")]
        fn try_new_time_formatter(
            &self,
            locale: &DataLocale,
            length: length::Time,
        ) -> Result<TimeFormatter, icu_datetime::DateTimeError> {
            self.get_provider().try_new_time_formatter(locale, length)
        }

        #[cfg(feature = "format_datetime")]
        fn try_new_datetime_formatter(
            &self,
            locale: &DataLocale,
            options: icu_datetime::options::DateTimeFormatterOptions,
        ) -> Result<DateTimeFormatter, icu_datetime::DateTimeError> {
            self.get_provider()
                .try_new_datetime_formatter(locale, options)
        }

        #[cfg(feature = "format_list")]
        fn try_new_and_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError> {
            self.get_provider()
                .try_new_and_list_formatter(locale, style)
        }

        #[cfg(feature = "format_list")]
        fn try_new_or_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError> {
            self.get_provider().try_new_or_list_formatter(locale, style)
        }

        #[cfg(feature = "format_list")]
        fn try_new_unit_list_formatter(
            &self,
            locale: &DataLocale,
            style: ListLength,
        ) -> Result<ListFormatter, icu_list::ListError> {
            self.get_provider()
                .try_new_unit_list_formatter(locale, style)
        }

        #[cfg(feature = "plurals")]
        fn try_new_plural_rules(
            &self,
            locale: &DataLocale,
            rule_type: PluralRuleType,
        ) -> Result<PluralRules, icu_plurals::PluralsError> {
            self.get_provider().try_new_plural_rules(locale, rule_type)
        }

        #[cfg(feature = "format_currency")]
        fn try_new_currency_formatter(
            &self,
            locale: &DataLocale,
            options: CurrencyFormatterOptions,
        ) -> Result<CurrencyFormatter, icu_provider::DataError> {
            self.get_provider()
                .try_new_currency_formatter(locale, options)
        }
    }
}
