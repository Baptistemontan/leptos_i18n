mod date;
mod datetime;
mod list;
mod nums;
mod time;

pub use date::*;
pub use datetime::*;
pub use list::*;
pub use nums::*;
pub use time::*;

use icu::locid;
use icu::datetime::{DateFormatter, TimeFormatter, DateTimeFormatter};
use icu::decimal::FixedDecimalFormatter;
use icu::datetime::options::length;
use icu::list::{ListFormatter, ListLength};
use std::collections::HashMap;
use std::cell::RefCell;
use crate::Locale;

#[derive(Default)]
struct Formatters {
  num: HashMap<&'static locid::Locale, &'static FixedDecimalFormatter>,
  date: HashMap<&'static locid::Locale, HashMap<length::Date, &'static DateFormatter>>,
  time: HashMap<&'static locid::Locale, HashMap<length::Time, &'static TimeFormatter>>,
  datetime: HashMap<&'static locid::Locale, HashMap<(length::Date, length::Time), &'static DateTimeFormatter>>,
  list: HashMap<&'static locid::Locale, HashMap<(list::ListType, ListLength), &'static ListFormatter>>
}

// Formatter cache
// The reason we leak the formatter is so that we can get a static ref,
// making possible to return values borrowing from the formatter,
// such as all *Formatter::format(..) returned values.

// FIXME: This is ok on the client, but will probably leak on the server.

thread_local! {
  static FORMATTERS: RefCell<Formatters> = Default::default();
}

fn get_num_formatter<L: Locale>(locale: L) -> &'static FixedDecimalFormatter {
  FORMATTERS.with_borrow_mut(|formatters| {
    let locale = locale.as_icu_locale();
    let num_formatter = formatters.num.entry(locale).or_insert_with(|| {
      let formatter = FixedDecimalFormatter::try_new(&locale.into(), Default::default()).unwrap();
      Box::leak(Box::new(formatter))
    });
    *num_formatter
  })
}

fn get_date_formatter<L: Locale>(locale: L, length: length::Date) -> &'static DateFormatter {
  FORMATTERS.with_borrow_mut(|formatters| {
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
  FORMATTERS.with_borrow_mut(|formatters| {
    let locale = locale.as_icu_locale();
    let time_formatters = formatters.time.entry(locale).or_default();
    let time_formatter = time_formatters.entry(length).or_insert_with(|| {
      let formatter = TimeFormatter::try_new_with_length(&locale.into(), length).unwrap();
      Box::leak(Box::new(formatter))
    });
    *time_formatter
  })
}

fn get_datetime_formatter<L: Locale>(locale: L, date_length: length::Date, time_length: length::Time) -> &'static DateTimeFormatter {
  FORMATTERS.with_borrow_mut(|formatters| {
    let locale = locale.as_icu_locale();
    let datetime_formatters = formatters.datetime.entry(locale).or_default();
    let datetime_formatter = datetime_formatters.entry((date_length, time_length)).or_insert_with(|| {
      let options = length::Bag::from_date_time_style(date_length, time_length);
      let formatter = DateTimeFormatter::try_new(&locale.into(), options.into()).unwrap();
      Box::leak(Box::new(formatter))
    });
    *datetime_formatter
  })
}

fn get_list_formatter<L: Locale>(locale: L, list_type: list::ListType, length: ListLength) -> &'static ListFormatter {
  FORMATTERS.with_borrow_mut(|formatters| {
    let locale = locale.as_icu_locale();
    let list_formatters = formatters.list.entry(locale).or_default();
    let list_formatter = list_formatters.entry((list_type, length)).or_insert_with(|| {
      let formatter = list_type.new_formatter(locale, length);
      Box::leak(Box::new(formatter))
    });
    *list_formatter
  })
}


