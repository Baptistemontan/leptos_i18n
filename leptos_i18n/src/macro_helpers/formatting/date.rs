use std::fmt::{self, Display};

use icu::{
    calendar::AnyCalendar,
    datetime::{input::DateInput, options::length},
};
use leptos::IntoView;

use crate::Locale;

/// Marker trait for types that can lend a reference to
/// `T: icu::datetime::input::DateInput<Calendar = icu::calendar::AnyCalendar>`.
pub trait AsIcuDate {
    /// The returned `T: DateInput<Calendar = AnyCalendar>`.
    type Date: DateInput<Calendar = AnyCalendar>;

    /// Lend a reference to `Self::Date`.
    fn as_icu_date(&self) -> &Self::Date;
}

impl<T: DateInput<Calendar = AnyCalendar>> AsIcuDate for T {
    type Date = Self;

    fn as_icu_date(&self) -> &Self::Date {
        self
    }
}

/// Marker trait for types that can be turned into a type
/// `T: icu::datetime::input::DateInput<Calendar = icu::calendar::AnyCalendar>`.
pub trait IntoIcuDate {
    /// The returned `T: DateInput<Calendar = AnyCalendar>`.
    type Date: DateInput<Calendar = AnyCalendar>;

    /// Consume self and return a `T: DateInput<Calendar = AnyCalendar>`.
    fn into_icu_date(self) -> Self::Date;
}

impl<T: DateInput<Calendar = AnyCalendar>> IntoIcuDate for T {
    type Date = Self;

    fn into_icu_date(self) -> Self::Date {
        self
    }
}

/// Marker trait for types that produce a `T: DateInput<Calendar = AnyCalendar>`.
pub trait DateFormatterInputFn: 'static + Clone {
    /// The returned `T: DateInput<Calendar = AnyCalendar>`.
    type Date: DateInput<Calendar = AnyCalendar>;

    /// Produce a `Self::Date`.
    fn to_icu_date(&self) -> Self::Date;
}

impl<T: IntoIcuDate, F: Fn() -> T + Clone + 'static> DateFormatterInputFn for F {
    type Date = T::Date;

    fn to_icu_date(&self) -> Self::Date {
        IntoIcuDate::into_icu_date(self())
    }
}

#[doc(hidden)]
pub fn format_date_to_view<L: Locale>(
    locale: L,
    date: impl DateFormatterInputFn,
    length: length::Date,
) -> impl IntoView {
    let date_formatter = super::get_date_formatter(locale, length);

    move || {
        let date = date.to_icu_date();
        date_formatter
            .format_to_string(&date)
            .expect("The date formatter to return a formatted date.")
    }
}

#[doc(hidden)]
pub fn format_date_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    date: &impl AsIcuDate,
    length: length::Date,
) -> fmt::Result {
    let formatted_date = format_date_to_display(locale, date, length);
    Display::fmt(&formatted_date, f)
}

#[doc(hidden)]
pub fn format_date_to_display<L: Locale>(
    locale: L,
    date: &impl AsIcuDate,
    length: length::Date,
) -> impl Display {
    let date_formatter = super::get_date_formatter(locale, length);
    let date = date.as_icu_date();
    date_formatter
        .format(date)
        .expect("The date formatter to return a formatted date.")
}
