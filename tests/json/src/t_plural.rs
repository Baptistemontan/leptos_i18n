use crate::i18n::*;
use leptos_i18n::plurals::{td_plural, td_plural_ordinal};

#[test]
fn cardinal_plural() {
    // count = 0
    let count = move || 0;
    let en = td_plural!(Locale::en, count = count, one => "one", _ => "other");
    assert_eq!(en, "other");
    let fr = td_plural!(Locale::fr, count = count, one => "one", _ => "other");
    assert_eq!(fr, "one");

    // count = 1
    let count = move || 1;
    let en = td_plural!(Locale::en, count = count, one => "one", _ => "other");
    assert_eq!(en, "one");
    let fr = td_plural!(Locale::fr, count = count, one => "one", _ => "other");
    assert_eq!(fr, "one");

    // count = 2
    let count = move || 2;
    let en = td_plural!(Locale::en, count = count, one => "one", _ => "other");
    assert_eq!(en, "other");
    let fr = td_plural!(Locale::fr, count = count, one => "one", _ => "other");
    assert_eq!(fr, "other");
}

#[test]
fn ordinal_plural() {
    // count = 1
    let count = move || 1;
    let en = td_plural_ordinal!(Locale::en, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(en, "one");
    let fr = td_plural_ordinal!(Locale::fr, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(fr, "one");

    // count = 2
    let count = move || 2;
    let en = td_plural_ordinal!(Locale::en, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(en, "two");
    let fr = td_plural_ordinal!(Locale::fr, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(fr, "other");

    // count = 3
    let count = move || 3;
    let en = td_plural_ordinal!(Locale::en, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(en, "few");
    let fr = td_plural_ordinal!(Locale::fr, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(fr, "other");

    // count = 4
    let count = move || 4;
    let en = td_plural_ordinal!(Locale::en, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(en, "other");
    let fr = td_plural_ordinal!(Locale::fr, count = count, one => "one", two => "two", few => "few", _ => "other");
    assert_eq!(fr, "other");
}
