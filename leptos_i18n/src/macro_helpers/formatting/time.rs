use std::fmt::{self, Display};

use icu::datetime::{input::IsoTimeInput, options::length};
use leptos::IntoView;

use crate::Locale;

/// Marker trait for types that can lend a reference to
/// `T: icu::datetime::input::IsoTimeInput`.
pub trait AsIcuTime {
    /// The returned `T: IsoTimeInput`.
    type Time: IsoTimeInput;

    /// Lend a reference to `Self::Time`.
    fn as_icu_time(&self) -> &Self::Time;
}

impl<T: IsoTimeInput> AsIcuTime for T {
    type Time = Self;

    fn as_icu_time(&self) -> &Self::Time {
        self
    }
}

/// Marker trait for types that can be turned into a type
/// `T: icu::datetime::input::IsoTimeInput`.
pub trait IntoIcuTime {
    /// The returned `T: IsoTimeInput`.
    type Time: IsoTimeInput;

    /// Consume self and return a `T: IsoTimeInput`.
    fn into_icu_time(self) -> Self::Time;
}

impl<T: IsoTimeInput> IntoIcuTime for T {
    type Time = Self;

    fn into_icu_time(self) -> Self::Time {
        self
    }
}

/// Marker trait for types that produce a `T: IsoTimeInput`.
pub trait TimeFormatterInputFn: 'static + Clone + Send + Sync {
    /// The returned `T: IsoTimeInput`.
    type Time: IsoTimeInput;

    /// Produce a `Self::Time`.
    fn to_icu_time(&self) -> Self::Time;
}

impl<T: IntoIcuTime, F: Fn() -> T + Clone + Send + Sync + 'static> TimeFormatterInputFn for F {
    type Time = T::Time;

    fn to_icu_time(&self) -> Self::Time {
        IntoIcuTime::into_icu_time(self())
    }
}

#[doc(hidden)]
pub fn format_time_to_view<L: Locale>(
    locale: L,
    time: impl TimeFormatterInputFn,
    length: length::Time,
) -> impl IntoView {
    let time_formatter = super::get_time_formatter(locale, length);

    move || {
        let time = time.to_icu_time();
        time_formatter.format_to_string(&time)
    }
}

#[doc(hidden)]
pub fn format_time_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    time: &impl AsIcuTime,
    length: length::Time,
) -> fmt::Result {
    let formatted_time = format_time_to_display(locale, time, length);
    Display::fmt(&formatted_time, f)
}

#[doc(hidden)]
pub fn format_time_to_display<L: Locale>(
    locale: L,
    time: &impl AsIcuTime,
    length: length::Time,
) -> impl Display {
    let time_formatter = super::get_time_formatter(locale, length);
    let time = time.as_icu_time();
    time_formatter.format(time)
}
