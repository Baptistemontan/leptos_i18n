//! This module contain traits and helper functions for formatting
//! different kind of value based on a locale.

mod date;
mod datetime;
mod list;
mod nums;
mod time;

pub use date::*;
pub use datetime::*;
pub use leptos_i18n_macro::{
    t_format, t_format_display, t_format_string, td_format, td_format_display, td_format_string,
    tu_format, tu_format_display, tu_format_string,
};
pub use list::*;
pub use nums::*;
pub use time::*;

use crate::Locale;
use icu::datetime::options::length;
use icu::datetime::{DateFormatter, DateTimeFormatter, TimeFormatter};
use icu::decimal::FixedDecimalFormatter;
use icu::list::{ListFormatter, ListLength};
use icu::locid;
use std::collections::HashMap;

#[derive(Default)]
struct Formatters {
    num: HashMap<&'static locid::Locale, &'static FixedDecimalFormatter>,
    date: HashMap<&'static locid::Locale, HashMap<length::Date, &'static DateFormatter>>,
    time: HashMap<&'static locid::Locale, HashMap<length::Time, &'static TimeFormatter>>,
    datetime: HashMap<
        &'static locid::Locale,
        HashMap<(length::Date, length::Time), &'static DateTimeFormatter>,
    >,
    list: HashMap<
        &'static locid::Locale,
        HashMap<(list::ListType, ListLength), &'static ListFormatter>,
    >,
}

// Formatters cache
//
// The reason we leak the formatter is so that we can get a static ref,
// making possible to return values borrowing from the formatter,
// such as all *Formatter::format(..) returned values.

#[cfg(not(feature = "sync"))]
thread_local! {
  static FORMATTERS: std::cell::RefCell<Formatters> = Default::default();
}

#[cfg(not(feature = "sync"))]
fn with_formatters_mut<T>(f: impl FnOnce(&mut Formatters) -> T) -> T {
    FORMATTERS.with_borrow_mut(f)
}

#[cfg(feature = "sync")]
static FORMATTERS: std::sync::OnceLock<std::sync::Mutex<Formatters>> = std::sync::OnceLock::new();

#[cfg(feature = "sync")]
fn with_formatters_mut<T>(f: impl FnOnce(&mut Formatters) -> T) -> T {
    let mutex = FORMATTERS.get_or_init(Default::default);
    let mut guard = mutex.lock().unwrap();
    f(&mut *guard)
}

fn get_num_formatter<L: Locale>(locale: L) -> &'static FixedDecimalFormatter {
    let locale = locale.as_icu_locale();
    with_formatters_mut(|formatters| {
        let num_formatter = formatters.num.entry(locale).or_insert_with(|| {
            let formatter =
                FixedDecimalFormatter::try_new(&locale.into(), Default::default()).unwrap();
            Box::leak(Box::new(formatter))
        });
        *num_formatter
    })
}

fn get_date_formatter<L: Locale>(locale: L, length: length::Date) -> &'static DateFormatter {
    with_formatters_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let date_formatters = formatters.date.entry(locale).or_default();
        let date_formatter = date_formatters.entry(length).or_insert_with(|| {
            let formatter = DateFormatter::try_new_with_length(&locale.into(), length).unwrap();
            Box::leak(Box::new(formatter))
        });
        *date_formatter
    })
}

fn get_time_formatter<L: Locale>(locale: L, length: length::Time) -> &'static TimeFormatter {
    with_formatters_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let time_formatters = formatters.time.entry(locale).or_default();
        let time_formatter = time_formatters.entry(length).or_insert_with(|| {
            let formatter = TimeFormatter::try_new_with_length(&locale.into(), length).unwrap();
            Box::leak(Box::new(formatter))
        });
        *time_formatter
    })
}

fn get_datetime_formatter<L: Locale>(
    locale: L,
    date_length: length::Date,
    time_length: length::Time,
) -> &'static DateTimeFormatter {
    with_formatters_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let datetime_formatters = formatters.datetime.entry(locale).or_default();
        let datetime_formatter = datetime_formatters
            .entry((date_length, time_length))
            .or_insert_with(|| {
                let options = length::Bag::from_date_time_style(date_length, time_length);
                let formatter = DateTimeFormatter::try_new(&locale.into(), options.into()).unwrap();
                Box::leak(Box::new(formatter))
            });
        *datetime_formatter
    })
}

fn get_list_formatter<L: Locale>(
    locale: L,
    list_type: list::ListType,
    length: ListLength,
) -> &'static ListFormatter {
    with_formatters_mut(|formatters| {
        let locale = locale.as_icu_locale();
        let list_formatters = formatters.list.entry(locale).or_default();
        let list_formatter = list_formatters
            .entry((list_type, length))
            .or_insert_with(|| {
                let formatter = list_type.new_formatter(locale, length);
                Box::leak(Box::new(formatter))
            });
        *list_formatter
    })
}
