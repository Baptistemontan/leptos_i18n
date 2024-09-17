use crate::i18n::*;
use leptos_i18n::formatting::*;
use leptos_i18n::reexports::fixed_decimal::FixedDecimal;
use leptos_i18n::reexports::icu::calendar::{Date, DateTime, Time};
use tests_common::*;

#[test]
fn list_formatting() {
    let list = move || ["A", "B", "C"];

    let en = td_format!(Locale::en, list, formatter: list);
    assert_eq_rendered!(en, "A, B, C");
    let fr = td_format!(Locale::fr, list, formatter: list(list_type: or));
    assert_eq_rendered!(fr, "A, B ou C");
}

#[test]
fn date_formatting() {
    let date = move || Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();

    let en = td_format!(Locale::en, date, formatter: date);
    assert_eq_rendered!(en, "Jan 2, 1970");
    let fr = td_format!(Locale::fr, date, formatter: date(date_length: full));
    assert_eq_rendered!(fr, "vendredi 2 janvier 1970");
}

#[test]
fn time_formatting() {
    let time = move || Time::try_new(14, 34, 28, 0).unwrap();

    let en = td_format!(Locale::en, time, formatter: time);
    assert_eq_rendered!(en, "2:34\u{202f}PM");
    let fr = td_format!(Locale::fr, time, formatter: time);
    assert_eq_rendered!(fr, "14:34");
}

#[test]
fn datetime_formatting() {
    let date = move || {
        let date = Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();
        let time = Time::try_new(14, 34, 28, 0).unwrap();
        DateTime::new(date, time)
    };

    let en = td_format!(Locale::en, date, formatter: datetime);
    assert_eq_rendered!(en, "Jan 2, 1970, 2:34\u{202f}PM");
    let fr = td_format!(Locale::fr, date, formatter: datetime);
    assert_eq_rendered!(fr, "2 janv. 1970, 14:34");
}

#[test]
fn number_formatting() {
    let num = move || FixedDecimal::from(200050).multiplied_pow10(-2);

    let en = td_format!(Locale::en, num, formatter: number);
    assert_eq_rendered!(en, "2,000.50");
    let fr = td_format!(Locale::fr, num, formatter: number);
    assert_eq_rendered!(fr, "2\u{202f}000,50");

    let num = move || 2000.50f64;

    let en = td_format!(Locale::en, num, formatter: number);
    assert_eq_rendered!(en, "2,000.5");
    let fr = td_format!(Locale::fr, num, formatter: number);
    assert_eq_rendered!(fr, "2\u{202f}000,5");
}
