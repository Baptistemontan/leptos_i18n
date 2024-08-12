use std::fmt;

use icu::{
    calendar::AnyCalendar,
    datetime::{input::DateTimeInput, options::length, DateTimeFormatter},
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
    let options = length::Bag::from_date_time_style(date_length, time_length);

    let formatter = DateTimeFormatter::try_new(&locale.as_langid().into(), options.into()).unwrap();

    move || {
        let datetime = datetime.to_datetime();
        formatter.format_to_string(&datetime).unwrap()
    }
}

pub fn format_datetime_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    datetime: &impl AsDateTime,
    date_length: length::Date,
    time_length: length::Time,
) -> fmt::Result {
    let options = length::Bag::from_date_time_style(date_length, time_length);
    let datetime_formatter =
        DateTimeFormatter::try_new(&locale.as_langid().into(), options.into()).unwrap();

    let date = datetime.as_datetime();
    let formatted_date = datetime_formatter.format(date).unwrap();
    std::fmt::Display::fmt(&formatted_date, f)
}
