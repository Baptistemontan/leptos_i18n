use std::fmt;

use icu::list::{ListFormatter, ListLength};
use leptos::IntoView;
use writeable::Writeable;

use crate::Locale;

pub trait WriteableList: Iterator<Item = Self::WItem> + Clone {
    type WItem: Writeable;
}

pub trait FormattedList: 'static {
    type Item: Writeable;
    type List: Iterator<Item = Self::Item> + Clone;

    fn to_list(&self) -> Self::List;
}

impl<W: Writeable, T: IntoIterator<Item = W> + Clone, F: Fn() -> T + Clone + 'static> FormattedList
    for F
where
    T::IntoIter: Clone,
{
    type Item = W;
    type List = T::IntoIter;

    fn to_list(&self) -> Self::List {
        self().into_iter()
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
        let list = list.to_list();
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
        let list = list.to_list();
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
        let list = list.to_list();
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
    let formatted_date = list_formatter.format(list);
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
    let formatted_date = list_formatter.format(list);
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
    let formatted_date = list_formatter.format(list);
    std::fmt::Display::fmt(&formatted_date, f)
}
