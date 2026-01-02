use std::fmt::{self, Display};

use icu_datetime::{
    fieldsets,
    input::Time,
    options::{Alignment, Length, TimePrecision},
    scaffold::{AllInputMarkers, ConvertCalendar, InFixedCalendar},
};
use leptos::IntoView;

use crate::Locale;

/// Provides a reference conversion of a type into an ICU4X Time, allowing non-destructive access to calendar-specific date operations.
pub trait AsIcuTime {
    /// The associated Time type that represents a calendar-specific date value in ICU4X format.
    type Time<'a>: ConvertCalendar<Converted<'a> = Time>
        + InFixedCalendar<()>
        + AllInputMarkers<fieldsets::T>;
    /// Returns a reference to the calendar-specific Time representation of this value.
    fn as_icu_time<'a>(&self) -> &Self::Time<'a>;
}

impl<T> AsIcuTime for T
where
    T: for<'a> ConvertCalendar<Converted<'a> = Time>
        + InFixedCalendar<()>
        + AllInputMarkers<fieldsets::T>,
{
    type Time<'a> = Self;
    fn as_icu_time<'a>(&self) -> &Self::Time<'a> {
        self
    }
}

/// Enables consuming conversion of a type into an ICU4X Time, transforming the original value into a calendar-specific date representation.
pub trait IntoIcuTime {
    /// The associated Time type that represents a calendar-specific date value in ICU4X format.
    type Time<'a>: ConvertCalendar<Converted<'a> = Time>
        + InFixedCalendar<()>
        + AllInputMarkers<fieldsets::T>;
    /// Consumes self to produce its calendar-specific Time representation.
    fn into_icu_time<'a>(self) -> Self::Time<'a>;
}

impl<T> IntoIcuTime for T
where
    T: for<'a> ConvertCalendar<Converted<'a> = Time>
        + InFixedCalendar<()>
        + AllInputMarkers<fieldsets::T>,
{
    type Time<'a> = Self;

    fn into_icu_time<'a>(self) -> Self::Time<'a> {
        self
    }
}

/// Defines a stateless, thread-safe function type that can repeatedly generate ICU4X Time instances for formatting operations.
pub trait TimeFormatterInputFn: 'static + Clone + Send + Sync {
    /// The associated Time type that represents a calendar-specific date value in ICU4X format.
    type Time<'a>: ConvertCalendar<Converted<'a> = Time>
        + InFixedCalendar<()>
        + AllInputMarkers<fieldsets::T>;

    /// Generates a new calendar-specific Time instance for formatting operations.
    fn to_icu_time<'a>(&self) -> Self::Time<'a>;
}

impl<T, F> TimeFormatterInputFn for F
where
    T: IntoIcuTime,
    F: Fn() -> T + Clone + Send + Sync + 'static,
{
    type Time<'a> = T::Time<'a>;

    fn to_icu_time<'a>(&self) -> Self::Time<'a> {
        IntoIcuTime::into_icu_time(self())
    }
}

#[doc(hidden)]
pub fn format_time_to_view<L, I>(
    locale: L,
    time: I,
    length: Length,
    alignment: Alignment,
    time_precision: TimePrecision,
) -> impl IntoView + Clone
where
    L: Locale,
    I: TimeFormatterInputFn,
{
    let time_formatter = super::get_time_formatter(locale, length, alignment, time_precision);

    move || {
        let time = time.to_icu_time();

        time_formatter.format(&time).to_string()
    }
}

#[doc(hidden)]
pub fn format_time_to_formatter<L, I>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    time: &I,
    length: Length,
    alignment: Alignment,
    time_precision: TimePrecision,
) -> fmt::Result
where
    L: Locale,
    I: AsIcuTime,
{
    let formatted_time = format_time_to_display(locale, time, length, alignment, time_precision);
    Display::fmt(&formatted_time, f)
}

#[doc(hidden)]
pub fn format_time_to_display<L, I>(
    locale: L,
    time: &I,
    length: Length,
    alignment: Alignment,
    time_precision: TimePrecision,
) -> impl Display
where
    L: Locale,
    I: AsIcuTime,
{
    let time_formatter = super::get_time_formatter(locale, length, alignment, time_precision);
    let time = time.as_icu_time();
    time_formatter.format(time)
}
