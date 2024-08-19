use crate::i18n::*;
use common::*;

#[test]
fn defaulted_string() {
    let en = td!(Locale::en, defaulted_string);
    assert_eq_rendered!(en, "this string is declared in locale en");
    let fr = td!(Locale::fr, defaulted_string);
    assert_eq_rendered!(fr, "this string is declared in locale en");
}

#[test]
fn defaulted_interpolation() {
    let en = td!(Locale::en, defaulted_interpolation, locale = "en");
    assert_eq_rendered!(en, "this interpolation is declared in locale en");
    let fr = td!(Locale::fr, defaulted_interpolation, locale = "en");
    assert_eq_rendered!(fr, "this interpolation is declared in locale en");
}

#[test]
fn defaulted_ranges() {
    let count = || 0;
    let en = td!(Locale::en, defaulted_ranges, locale = "en", count = count);
    assert_eq_rendered!(en, "zero");
    let fr = td!(Locale::fr, defaulted_ranges, locale = "en", count = count);
    assert_eq_rendered!(fr, "zero");

    for i in [-3, 5, 12] {
        let count = move || i;
        let en = td!(Locale::en, defaulted_ranges, locale = "en", count = count);
        assert_eq_rendered!(en, "this range is declared in locale en");
        let fr = td!(Locale::fr, defaulted_ranges, locale = "en", count = count);
        assert_eq_rendered!(fr, "this range is declared in locale en");
    }
}
