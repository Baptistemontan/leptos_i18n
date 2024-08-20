use std::fmt::{self, Display};

use icu::{
    calendar::AnyCalendar,
    datetime::{input::DateTimeInput, options::length},
};
use leptos::IntoView;

use crate::Locale;

use super::{AsIcuDate, AsIcuTime, IntoIcuDate, IntoIcuTime};

/// Marker trait for types that can lend a reference to
/// `T: icu::datetime::input::DateTimeInput<Calendar = icu::calendar::AnyCalendar>`.
pub trait AsIcuDateTime {
    /// The returned `T: DateTimeInput<Calendar = AnyCalendar>`.
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    /// Lend a reference to `Self::DateTime`.
    fn as_icu_datetime(&self) -> &Self::DateTime;
}

impl<DT: DateTimeInput<Calendar = AnyCalendar>, T: AsIcuDate<Date = DT> + AsIcuTime<Time = DT>>
    AsIcuDateTime for T
{
    type DateTime = DT;

    fn as_icu_datetime(&self) -> &Self::DateTime {
        self.as_icu_date()
    }
}

/// Marker trait for types that can be turned into a type
/// `T: icu::datetime::input::DateTimeInput<Calendar = icu::calendar::AnyCalendar>`.
pub trait IntoIcuDateTime {
    /// The returned `T: DateTimeInput<Calendar = AnyCalendar>`.
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    /// Consume self and return a `T: DateTimeInput<Calendar = AnyCalendar>`
    fn into_icu_datetime(self) -> Self::DateTime;
}

impl<
        DT: DateTimeInput<Calendar = AnyCalendar>,
        T: IntoIcuDate<Date = DT> + IntoIcuTime<Time = DT>,
    > IntoIcuDateTime for T
{
    type DateTime = DT;

    fn into_icu_datetime(self) -> Self::DateTime {
        self.into_icu_date()
    }
}

/// Marker trait for types that produce a `T: DateTimeInput<Calendar = AnyCalendar>`.
pub trait DateTimeFormatterInputFn: 'static + Clone {
    /// The returned `T: DateTimeInput<Calendar = AnyCalendar>`.
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    /// Produce a `Self::DateTime`.
    fn to_icu_datetime(&self) -> Self::DateTime;
}

impl<T: IntoIcuDateTime, F: Fn() -> T + Clone + 'static> DateTimeFormatterInputFn for F {
    type DateTime = T::DateTime;

    fn to_icu_datetime(&self) -> Self::DateTime {
        IntoIcuDateTime::into_icu_datetime(self())
    }
}

#[doc(hidden)]
pub fn format_datetime_to_view<L: Locale>(
    locale: L,
    datetime: impl DateTimeFormatterInputFn,
    date_length: length::Date,
    time_length: length::Time,
) -> impl IntoView {
    let datetime_formatter = super::get_datetime_formatter(locale, date_length, time_length);

    move || {
        let datetime = datetime.to_icu_datetime();
        datetime_formatter
            .format_to_string(&datetime)
            .expect("The datetime formatter to return a formatted datetime.")
    }
}

#[doc(hidden)]
pub fn format_datetime_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    datetime: &impl AsIcuDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> fmt::Result {
    let formatted_date = format_datetime_to_display(locale, datetime, date_length, time_length);
    Display::fmt(&formatted_date, f)
}

#[doc(hidden)]
pub fn format_datetime_to_display<L: Locale>(
    locale: L,
    datetime: &impl AsIcuDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> impl Display {
    let datetime_formatter = super::get_datetime_formatter(locale, date_length, time_length);
    let date = datetime.as_icu_datetime();
    datetime_formatter
        .format(date)
        .expect("The datetime formatter to return a formatted datetime.")
}
