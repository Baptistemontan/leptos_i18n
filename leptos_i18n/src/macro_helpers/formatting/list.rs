use std::fmt::{self, Display};

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ListType {
    And,
    Or,
    Unit,
}

impl ListType {
    pub fn new_formatter(self, locale: &icu::locid::Locale, length: ListLength) -> ListFormatter {
        match self {
            ListType::And => {
                ListFormatter::try_new_and_with_length(&locale.into(), length).unwrap()
            }
            ListType::Or => ListFormatter::try_new_or_with_length(&locale.into(), length).unwrap(),
            ListType::Unit => {
                ListFormatter::try_new_unit_with_length(&locale.into(), length).unwrap()
            }
        }
    }
}

pub fn format_list_to_string<L: Locale>(
    locale: L,
    list: impl FormattedList,
    list_type: ListType,
    length: ListLength,
) -> impl IntoView {
    let list_formatter = super::get_list_formatter(locale, list_type, length);

    move || {
        let list = list.to_list().into_iter();
        list_formatter.format_to_string(list)
    }
}

pub fn format_list_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    list: impl WriteableList,
    list_type: ListType,
    length: ListLength,
) -> fmt::Result {
    let formatted_list = format_list_to_display(locale, list, list_type, length);
    Display::fmt(&formatted_list, f)
}

pub fn format_list_to_display<'a, WL: WriteableList, L: Locale>(
    locale: L,
    list: WL,
    list_type: ListType,
    length: ListLength,
) -> impl Display + 'a
where
    WL::WItem: 'a,
    WL::WIterator: 'a,
{
    let list_formatter = super::get_list_formatter(locale, list_type, length);
    list_formatter.format(list.into_iter())
}
