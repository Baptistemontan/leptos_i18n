use std::fmt;

use icu::{
    calendar::{AnyCalendar, Date, DateTime},
    datetime::{input::DateInput, options::length, DateFormatter},
};
use leptos::IntoView;

use crate::Locale;

pub trait IntoDate {
    type Date: DateInput<Calendar = AnyCalendar>;

    fn into_date(self) -> Self::Date;
}

impl IntoDate for Date<AnyCalendar> {
    type Date = Self;

    fn into_date(self) -> Self::Date {
        self
    }
}

impl IntoDate for DateTime<AnyCalendar> {
    type Date = Self;

    fn into_date(self) -> Self::Date {
        self
    }
}

pub trait FormattedDate: 'static {
    type Date: DateInput<Calendar = AnyCalendar>;

    fn to_date(&self) -> Self::Date;
}

impl<T: IntoDate, F: Fn() -> T + Clone + 'static> FormattedDate for F {
    type Date = T::Date;

    fn to_date(&self) -> Self::Date {
        IntoDate::into_date(self())
    }
}

pub fn format_date_to_string<L: Locale>(
    locale: L,
    date: impl FormattedDate,
    length: length::Date,
) -> impl IntoView {
    let formatter = DateFormatter::try_new_with_length(&locale.as_langid().into(), length).unwrap();

    move || {
        let date = date.to_date();
        formatter.format_to_string(&date).unwrap()
    }
}

pub fn format_date_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    date: impl IntoDate,
    length: length::Date,
) -> fmt::Result {
    let date_formatter =
        DateFormatter::try_new_with_length(&locale.as_langid().into(), length).unwrap();
    let date = date.into_date();
    let formatted_date = date_formatter.format(&date).unwrap();
    std::fmt::Display::fmt(&formatted_date, f)
}
