use std::fmt;

use icu::list::{ListFormatter, ListLength};
use leptos::IntoView;
use writeable::Writeable;

use crate::Locale;

pub trait WriteableList:
    IntoIterator<Item = Self::WItem, IntoIter = Self::WIterator> + Clone
{
    type WIterator: Iterator<Item = Self::WItem> + Clone;
    type WItem: Writeable;
}

impl<T> WriteableList for T
where
    T: IntoIterator + Clone,
    T::Item: Writeable,
    T::IntoIter: Clone,
{
    type WItem = T::Item;
    type WIterator = T::IntoIter;
}

pub trait FormattedList: 'static {
    type List: WriteableList;

    fn to_list(&self) -> Self::List;
}

impl<T: WriteableList, F: Fn() -> T + Clone + 'static> FormattedList for F {
    type List = T;

    fn to_list(&self) -> Self::List {
        self()
    }
}

pub fn format_and_list_to_string<L: Locale>(
    locale: L,
    list: impl FormattedList,
    length: ListLength,
) -> impl IntoView {
    let formatter =
        ListFormatter::try_new_and_with_length(&locale.as_langid().into(), length).unwrap();

    move || {
        let list = list.to_list().into_iter();
        formatter.format_to_string(list)
    }
}

pub fn format_or_list_to_string<L: Locale>(
    locale: L,
    list: impl FormattedList,
    length: ListLength,
) -> impl IntoView {
    let formatter =
        ListFormatter::try_new_or_with_length(&locale.as_langid().into(), length).unwrap();

    move || {
        let list = list.to_list().into_iter();
        formatter.format_to_string(list)
    }
}

pub fn format_unit_list_to_string<L: Locale>(
    locale: L,
    list: impl FormattedList,
    length: ListLength,
) -> impl IntoView {
    let formatter =
        ListFormatter::try_new_unit_with_length(&locale.as_langid().into(), length).unwrap();

    move || {
        let list = list.to_list().into_iter();
        formatter.format_to_string(list)
    }
}

pub fn format_and_list_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    list: impl WriteableList,
    length: ListLength,
) -> fmt::Result {
    let list_formatter =
        ListFormatter::try_new_and_with_length(&locale.as_langid().into(), length).unwrap();
    let formatted_date = list_formatter.format(list.into_iter());
    std::fmt::Display::fmt(&formatted_date, f)
}

pub fn format_or_list_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    list: impl WriteableList,
    length: ListLength,
) -> fmt::Result {
    let list_formatter =
        ListFormatter::try_new_or_with_length(&locale.as_langid().into(), length).unwrap();
    let formatted_date = list_formatter.format(list.into_iter());
    std::fmt::Display::fmt(&formatted_date, f)
}

pub fn format_unit_list_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    list: impl WriteableList,
    length: ListLength,
) -> fmt::Result {
    let list_formatter =
        ListFormatter::try_new_unit_with_length(&locale.as_langid().into(), length).unwrap();
    let formatted_date = list_formatter.format(list.into_iter());
    std::fmt::Display::fmt(&formatted_date, f)
}
