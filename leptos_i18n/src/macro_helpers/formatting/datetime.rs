use std::fmt::{self, Display};

use icu::{
    calendar::AnyCalendar,
    datetime::{input::DateTimeInput, options::length},
};
use leptos::IntoView;

use crate::Locale;

use super::{IntoDate, AsDate, IntoTime, AsTime};

pub trait AsDateTime: AsDate<Date = Self::DateTime> + AsTime<Time = Self::DateTime> {
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    fn as_datetime(&self) -> &Self::DateTime;
}

impl<DT: DateTimeInput<Calendar = AnyCalendar>, T: AsDate<Date = DT> + AsTime<Time = DT>>
    AsDateTime for T
{
    type DateTime = DT;

    fn as_datetime(&self) -> &Self::DateTime {
        self.as_date()
    }
}

pub trait IntoDateTime: IntoDate<Date = Self::DateTime> + IntoTime<Time = Self::DateTime> {
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    fn into_datetime(self) -> Self::DateTime;
}

impl<DT: DateTimeInput<Calendar = AnyCalendar>, T: IntoDate<Date = DT> + IntoTime<Time = DT>>
    IntoDateTime for T
{
    type DateTime = DT;

    fn into_datetime(self) -> Self::DateTime {
        self.into_date()
    }
}

pub trait FormattedDateTime: 'static + Clone {
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    fn to_datetime(&self) -> Self::DateTime;
}

impl<T: IntoDateTime, F: Fn() -> T + Clone + 'static> FormattedDateTime for F {
    type DateTime = T::DateTime;

    fn to_datetime(&self) -> Self::DateTime {
        IntoDateTime::into_datetime(self())
    }
}

pub fn format_datetime_to_string<L: Locale>(
    locale: L,
    datetime: impl FormattedDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> impl IntoView {
    let datetime_formatter = super::get_datetime_formatter(locale, date_length, time_length);

    move || {
        let datetime = datetime.to_datetime();
        datetime_formatter.format_to_string(&datetime).unwrap()
    }
}

pub fn format_datetime_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    datetime: &impl AsDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> fmt::Result {
    let formatted_date = format_datetime_to_display(locale, datetime, date_length, time_length);
    Display::fmt(&formatted_date, f)
}

pub fn format_datetime_to_display<L: Locale>(
    locale: L,
    datetime: &impl AsDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> impl Display {
    let datetime_formatter = super::get_datetime_formatter(locale, date_length, time_length);
    let date = datetime.as_datetime();
    datetime_formatter.format(date).unwrap()
}
