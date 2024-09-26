use std::fmt::{self, Display};

use icu::list::{ListFormatter, ListLength};
use leptos::IntoView;
use writeable::Writeable;

use crate::Locale;

/// Marker trait for types that can be turned into an iterator where
/// `Iterator::Item: writeable::Writeable`.
pub trait WriteableList:
    IntoIterator<Item = Self::WItem, IntoIter = Self::WIterator> + Clone
{
    /// The iterator produced.
    type WIterator: Iterator<Item = Self::WItem> + Clone;
    /// The item the iterator returns.
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

/// Marker trait for types that produce a `T: WriteableList`.
pub trait ListFormatterInputFn: 'static + Send + Sync + Clone {
    /// The returned `T: WriteableList`.
    type List: WriteableList;

    /// Produce a `Self::List`.
    fn to_list(&self) -> Self::List;
}

impl<T: WriteableList, F: Fn() -> T + Clone + Send + Sync + 'static> ListFormatterInputFn for F {
    type List = T;

    fn to_list(&self) -> Self::List {
        self()
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ListType {
    And,
    Or,
    Unit,
}

impl ListType {
    pub fn new_formatter(self, locale: &icu::locid::Locale, length: ListLength) -> ListFormatter {
        match self {
            ListType::And => ListFormatter::try_new_and_with_length(&locale.into(), length)
                .expect("A list formatter"),
            ListType::Or => ListFormatter::try_new_or_with_length(&locale.into(), length)
                .expect("A list formatter"),
            ListType::Unit => ListFormatter::try_new_unit_with_length(&locale.into(), length)
                .expect("A list formatter"),
        }
    }
}

#[doc(hidden)]
pub fn format_list_to_view<L: Locale>(
    locale: L,
    list: impl ListFormatterInputFn,
    list_type: ListType,
    length: ListLength,
) -> impl IntoView + Clone {
    let list_formatter = super::get_list_formatter(locale, list_type, length);

    move || {
        let list = list.to_list().into_iter();
        list_formatter.format_to_string(list)
    }
}

#[doc(hidden)]
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

#[doc(hidden)]
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
