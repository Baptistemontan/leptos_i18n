use crate::i18n::*;
use leptos_i18n::reexports::{
    fixed_decimal::FixedDecimal,
    icu::calendar::{Date, DateTime, Time},
};
use tests_common::*;

#[test]
fn list_formatting() {
    let list = move || ["A", "B", "C"];

    let en = td!(Locale::en, list_formatting, list);
    assert_eq_rendered!(en, "A, B, and C");
    let fr = td!(Locale::fr, list_formatting, list);
    assert_eq_rendered!(fr, "A, B ou C");
}

#[test]
fn date_formatting() {
    let date = move || Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();

    let en = td!(Locale::en, date_formatting, date);
    assert_eq_rendered!(en, "Jan 2, 1970");
    let fr = td!(Locale::fr, date_formatting, date);
    assert_eq_rendered!(fr, "2 janv. 1970");
}

#[test]
fn time_formatting() {
    let time = move || Time::try_new(14, 34, 28, 0).unwrap();

    let en = td!(Locale::en, time_formatting, time);
    assert_eq_rendered!(en, "2:34\u{202f}PM");
    let fr = td!(Locale::fr, time_formatting, time);
    assert_eq_rendered!(fr, "14:34");
}

#[test]
fn datetime_formatting() {
    let date = move || {
        let date = Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();
        let time = Time::try_new(14, 34, 28, 0).unwrap();
        DateTime::new(date, time)
    };

    let en = td!(Locale::en, datetime_formatting, date);
    assert_eq_rendered!(en, "Jan 2, 1970, 2:34\u{202f}PM");
    let fr = td!(Locale::fr, datetime_formatting, date);
    assert_eq_rendered!(fr, "2 janv. 1970, 14:34");
}

#[test]
fn number_formatting() {
    let num = move || FixedDecimal::from(200050).multiplied_pow10(-2);

    let en = td!(Locale::en, number_formatting, num);
    assert_eq_rendered!(en, "2,000.50");
    let fr = td!(Locale::fr, number_formatting, num);
    assert_eq_rendered!(fr, "2\u{202f}000,50");

    let num = move || 2000.50f64;

    let en = td!(Locale::en, number_formatting, num);
    assert_eq_rendered!(en, "2,000.5");
    let fr = td!(Locale::fr, number_formatting, num);
    assert_eq_rendered!(fr, "2\u{202f}000,5");

    let en = td!(Locale::en, number_formatting_grouping, num);
    assert_eq_rendered!(en, "2000.5");
    let fr = td!(Locale::fr, number_formatting_grouping, num);
    assert_eq_rendered!(fr, "2000,5");
}

#[test]
fn currency_formatting() {
    let num = move || FixedDecimal::from(200050).multiplied_pow10(-2);

    let en = td!(Locale::en, currency_formatting, num);
    assert_eq_rendered!(en, "€2000.50");
    let fr = td!(Locale::fr, currency_formatting, num);
    assert_eq_rendered!(fr, "2000.50\u{a0}$US");

    let num = move || 2000.50f64;

    let en = td!(Locale::en, currency_formatting, num);
    assert_eq_rendered!(en, "€2000.5");
    let fr = td!(Locale::fr, currency_formatting, num);
    assert_eq_rendered!(fr, "2000.5\u{a0}$US");

    let en = td!(Locale::en, currency_formatting_width, num);
    assert_eq_rendered!(en, "$2000.5");
    let fr = td!(Locale::fr, currency_formatting_width, num);
    assert_eq_rendered!(fr, "2000.5\u{a0}€");
}
