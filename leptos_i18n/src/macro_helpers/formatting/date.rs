use std::fmt::{self, Display};

use icu_calendar::{AnyCalendar, Date, Ref};
use icu_datetime::{
    options::{Alignment, Length},
    scaffold::ConvertCalendar,
};
use leptos::IntoView;

use crate::Locale;

/// Provides a reference conversion of a type into an ICU4X DateTime, allowing non-destructive access to calendar-specific date operations.
pub trait AsIcuDate {
    /// The associated Date type that represents a calendar-specific date value in ICU4X format.
    type Date<'a>: ConvertCalendar<Converted<'a> = Date<Ref<'a, AnyCalendar>>>;
    /// Returns a reference to the calendar-specific Date representation of this value.
    fn as_icu_date<'a>(&self) -> &Self::Date<'a>;
}

impl<T> AsIcuDate for T
where
    T: for<'a> ConvertCalendar<Converted<'a> = Date<Ref<'a, AnyCalendar>>>,
{
    type Date<'a> = Self;
    fn as_icu_date<'a>(&self) -> &Self::Date<'a> {
        self
    }
}

/// Enables consuming conversion of a type into an ICU4X Date, transforming the original value into a calendar-specific date representation.
pub trait IntoIcuDate {
    /// The associated Date type that represents a calendar-specific date value in ICU4X format.
    type Date<'a>: ConvertCalendar<Converted<'a> = Date<Ref<'a, AnyCalendar>>>;
    /// Consumes self to produce its calendar-specific Date representation.
    fn into_icu_date<'a>(self) -> Self::Date<'a>;
}

impl<T> IntoIcuDate for T
where
    T: for<'a> ConvertCalendar<Converted<'a> = Date<Ref<'a, AnyCalendar>>>,
{
    type Date<'a> = Self;

    fn into_icu_date<'a>(self) -> Self::Date<'a> {
        self
    }
}

/// Defines a stateless, thread-safe function type that can repeatedly generate ICU4X Date instances for formatting operations.
pub trait DateFormatterInputFn: 'static + Clone + Send + Sync {
    /// The associated Date type that represents a calendar-specific date value in ICU4X format.
    type Date<'a>: ConvertCalendar<Converted<'a> = Date<Ref<'a, AnyCalendar>>>;

    /// Generates a new calendar-specific Date instance for formatting operations.
    fn to_icu_date<'a>(&self) -> Self::Date<'a>;
}

impl<T, F> DateFormatterInputFn for F
where
    T: IntoIcuDate,
    F: Fn() -> T + Clone + Send + Sync + 'static,
{
    type Date<'a> = T::Date<'a>;

    fn to_icu_date<'a>(&self) -> Self::Date<'a> {
        IntoIcuDate::into_icu_date(self())
    }
}

#[doc(hidden)]
pub fn format_date_to_view<L, I>(
    locale: L,
    date: I,
    length: Length,
    alignment: Alignment,
) -> impl IntoView + Clone
where
    L: Locale,
    I: DateFormatterInputFn,
{
    let date_formatter = super::get_date_formatter(locale, length, alignment);

    move || {
        let date = date.to_icu_date();

        date_formatter.format(&date).to_string()
    }
}

#[doc(hidden)]
pub fn format_date_to_formatter<L, I>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    date: &I,
    length: Length,
    alignment: Alignment,
) -> fmt::Result
where
    L: Locale,
    I: AsIcuDate,
{
    let formatted_date = format_date_to_display(locale, date, length, alignment);
    Display::fmt(&formatted_date, f)
}

#[doc(hidden)]
pub fn format_date_to_display<L, I>(
    locale: L,
    date: &I,
    length: Length,
    alignment: Alignment,
) -> impl Display
where
    L: Locale,
    I: AsIcuDate,
{
    let date_formatter = super::get_date_formatter(locale, length, alignment);
    let date = date.as_icu_date();
    date_formatter.format(date).to_string()
}
