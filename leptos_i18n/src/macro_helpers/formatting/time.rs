use std::fmt;

use icu::{
    calendar::{AnyCalendar, DateTime, Time},
    datetime::{input::IsoTimeInput, options::length, TimeFormatter},
};
use leptos::IntoView;

use crate::Locale;

pub trait AsTime {
    type Time: IsoTimeInput;

    fn as_time(&self) -> &Self::Time;
}

impl<T: IsoTimeInput> AsTime for T {
    type Time = Self;

    fn as_time(&self) -> &Self::Time {
        self
    }
}

pub trait IntoTime {
    type Time: IsoTimeInput;

    fn into_time(self) -> Self::Time;
}

impl IntoTime for Time {
    type Time = Self;

    fn into_time(self) -> Self::Time {
        self
    }
}

impl IntoTime for DateTime<AnyCalendar> {
    type Time = Self;

    fn into_time(self) -> Self::Time {
        self
    }
}

pub trait FormattedTime: 'static + Clone {
    type Time: IsoTimeInput;

    fn to_time(&self) -> Self::Time;
}

impl<T: IntoTime, F: Fn() -> T + Clone + 'static> FormattedTime for F {
    type Time = T::Time;

    fn to_time(&self) -> Self::Time {
        IntoTime::into_time(self())
    }
}

pub fn format_time_to_string<L: Locale>(
    locale: L,
    time: impl FormattedTime,
    length: length::Time,
) -> impl IntoView {
    let formatter = TimeFormatter::try_new_with_length(&locale.as_langid().into(), length).unwrap();

    move || {
        let time = time.to_time();
        formatter.format_to_string(&time)
    }
}

pub fn format_time_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    time: &impl AsTime,
    length: length::Time,
) -> fmt::Result {
    let time_formatter =
        TimeFormatter::try_new_with_length(&locale.as_langid().into(), length).unwrap();
    let time = time.as_time();
    let formatted_time = time_formatter.format(time);
    std::fmt::Display::fmt(&formatted_time, f)
}
