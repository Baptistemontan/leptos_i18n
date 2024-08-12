use std::fmt::{self, Display};

use icu::{
    calendar::{AnyCalendar, Date, DateTime},
    datetime::{input::DateInput, options::length},
};
use leptos::IntoView;

use crate::Locale;

pub trait AsDate {
    type Date: DateInput<Calendar = AnyCalendar>;

    fn as_date(&self) -> &Self::Date;
}

impl<T: DateInput<Calendar = AnyCalendar>> AsDate for T {
    type Date = Self;

    fn as_date(&self) -> &Self::Date {
        self
    }
}


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

pub trait FormattedDate: 'static + Clone {
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
    let date_formatter = super::get_date_formatter(locale, length);

    move || {
        let date = date.to_date();
        date_formatter.format_to_string(&date).unwrap()
    }
}

pub fn format_date_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    date: &impl AsDate,
    length: length::Date,
) -> fmt::Result {
    let formatted_date = format_date_to_display(locale, date, length);
    Display::fmt(&formatted_date, f)
}

pub fn format_date_to_display<L: Locale>(
    locale: L,
    date: &impl AsDate,
    length: length::Date,
) -> impl Display {
    let date_formatter = super::get_date_formatter(locale, length);
    let date = date.as_date();
    date_formatter.format(date).unwrap()
}
