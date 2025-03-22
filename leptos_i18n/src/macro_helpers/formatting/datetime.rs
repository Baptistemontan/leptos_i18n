use std::fmt::{self, Display};

use icu_calendar::{AnyCalendar, Ref};
use icu_datetime::{
    input::DateTime,
    options::{Alignment, Length, TimePrecision, YearStyle},
    scaffold::ConvertCalendar,
};
use leptos::IntoView;

use crate::Locale;

/// Provides a reference conversion of a type into an ICU4X DateTime, allowing non-destructive access to calendar-specific datetime operations.
pub trait AsIcuDateTime {
    /// The associated DateTime type that represents a calendar-specific datetime value in ICU4X format.
    type DateTime<'a>: ConvertCalendar<Converted<'a> = DateTime<Ref<'a, AnyCalendar>>>;

    /// Returns a reference to the calendar-specific DateTime representation of this value.
    fn as_icu_datetime<'a>(&self) -> &Self::DateTime<'a>;
}

impl<T> AsIcuDateTime for T
where
    T: for<'a> ConvertCalendar<Converted<'a> = DateTime<Ref<'a, AnyCalendar>>>,
{
    type DateTime<'a> = Self;
    fn as_icu_datetime<'a>(&self) -> &Self::DateTime<'a> {
        self
    }
}

/// Enables consuming conversion of a type into an ICU4X DateTime, transforming the original value into a calendar-specific datetime representation.
pub trait IntoIcuDateTime {
    /// The associated DateTime type that represents a calendar-specific datetime value in ICU4X format.
    type DateTime<'a>: ConvertCalendar<Converted<'a> = DateTime<Ref<'a, AnyCalendar>>>;

    /// Consumes self to produce its calendar-specific DateTime representation.
    fn into_icu_datetime<'a>(self) -> Self::DateTime<'a>;
}

impl<T> IntoIcuDateTime for T
where
    T: for<'a> ConvertCalendar<Converted<'a> = DateTime<Ref<'a, AnyCalendar>>>,
{
    type DateTime<'a> = Self;

    fn into_icu_datetime<'a>(self) -> Self::DateTime<'a> {
        self
    }
}

/// Defines a stateless, thread-safe function type that can repeatedly generate ICU4X DateTime instances for formatting operations.
pub trait DateTimeFormatterInputFn: 'static + Clone + Send + Sync {
    /// The associated DateTime type that represents a calendar-specific datetime value in ICU4X format.
    type DateTime<'a>: ConvertCalendar<Converted<'a> = DateTime<Ref<'a, AnyCalendar>>>;

    /// Generates a new calendar-specific DateTime instance for formatting operations.
    fn to_icu_datetime<'a>(&self) -> Self::DateTime<'a>;
}

impl<T, F> DateTimeFormatterInputFn for F
where
    T: IntoIcuDateTime,
    F: Fn() -> T + Clone + Send + Sync + 'static,
{
    type DateTime<'a> = T::DateTime<'a>;

    fn to_icu_datetime<'a>(&self) -> Self::DateTime<'a> {
        IntoIcuDateTime::into_icu_datetime(self())
    }
}

#[doc(hidden)]
pub fn format_datetime_to_view<L, I>(
    locale: L,
    datetime: I,
    length: Length,
    alignment: Alignment,
    time_precision: TimePrecision,
    year_style: YearStyle,
) -> impl IntoView + Clone
where
    L: Locale,
    I: DateTimeFormatterInputFn,
{
    let datetime_formatter =
        super::get_datetime_formatter(locale, length, alignment, time_precision, year_style);

    move || {
        let datetime = datetime.to_icu_datetime();

        datetime_formatter.format(&datetime).to_string()
    }
}

#[doc(hidden)]
pub fn format_datetime_to_formatter<L, I>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    datetime: &I,
    length: Length,
    alignment: Alignment,
    time_precision: TimePrecision,
    year_style: YearStyle,
) -> fmt::Result
where
    L: Locale,
    I: AsIcuDateTime,
{
    let formatted_datetime = format_datetime_to_display(
        locale,
        datetime,
        length,
        alignment,
        time_precision,
        year_style,
    );
    Display::fmt(&formatted_datetime, f)
}

#[doc(hidden)]
pub fn format_datetime_to_display<L, I>(
    locale: L,
    datetime: &I,
    length: Length,
    alignment: Alignment,
    time_precision: TimePrecision,
    year_style: YearStyle,
) -> impl Display
where
    L: Locale,
    I: AsIcuDateTime,
{
    let datetime_formatter =
        super::get_datetime_formatter(locale, length, alignment, time_precision, year_style);
    let datetime = datetime.as_icu_datetime();
    datetime_formatter.format(datetime).to_string()
}
