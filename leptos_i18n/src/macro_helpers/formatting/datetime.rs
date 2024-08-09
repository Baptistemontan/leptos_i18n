use std::fmt;

use icu::{
    calendar::AnyCalendar,
    datetime::{input::DateTimeInput, options::length, DateTimeFormatter},
};
use leptos::IntoView;

use crate::Locale;

use super::{IntoDate, IntoTime};

pub trait IntoDateTime: IntoDate + IntoTime {
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    fn into_date_time(self) -> Self::DateTime;
}

impl<DT: DateTimeInput<Calendar = AnyCalendar>, T: IntoDate<Date = DT> + IntoTime<Time = DT>>
    IntoDateTime for T
{
    type DateTime = DT;

    fn into_date_time(self) -> Self::DateTime {
        self.into_date()
    }
}

pub trait FormattedDateTime: 'static {
    type DateTime: DateTimeInput<Calendar = AnyCalendar>;

    fn to_date_time(&self) -> Self::DateTime;
}

impl<T: IntoDateTime, F: Fn() -> T + Clone + 'static> FormattedDateTime for F {
    type DateTime = T::DateTime;

    fn to_date_time(&self) -> Self::DateTime {
        IntoDateTime::into_date_time(self())
    }
}

pub fn format_date_time_to_string<L: Locale>(
    locale: L,
    date_time: impl FormattedDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> impl IntoView {
    let options = length::Bag::from_date_time_style(date_length, time_length);

    let formatter = DateTimeFormatter::try_new(&locale.as_langid().into(), options.into()).unwrap();

    move || {
        let date_time = date_time.to_date_time();
        formatter.format_to_string(&date_time).unwrap()
    }
}

pub fn format_date_time_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    date_time: impl IntoDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> fmt::Result {
    let options = length::Bag::from_date_time_style(date_length, time_length);
    let date_time_formatter =
        DateTimeFormatter::try_new(&locale.as_langid().into(), options.into()).unwrap();

    let date = date_time.into_date_time();
    let formatted_date = date_time_formatter.format(&date).unwrap();
    std::fmt::Display::fmt(&formatted_date, f)
}
