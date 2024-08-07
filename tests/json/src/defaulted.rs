use crate::i18n::*;
use common::*;

#[test]
fn defaulted_string() {
    let en = tdbg!(Locale::en, defaulted_string);
    assert_eq!(en, "this string is declared in locale en");
    let fr = tdbg!(Locale::fr, defaulted_string);
    assert_eq!(fr, "this string is declared in locale en");
}

#[test]
fn defaulted_interpolation() {
    let en = tdbg!(Locale::en, defaulted_interpolation, locale = "en");
    assert_eq_rendered!(en, "this interpolation is declared in locale en");
    let fr = tdbg!(Locale::fr, defaulted_interpolation, locale = "en");
    assert_eq_rendered!(fr, "this interpolation is declared in locale en");
}

#[test]
fn defaulted_plurals() {
    let count = || 0;
    let en = tdbg!(Locale::en, defaulted_plurals, locale = "en", $ = count);
    assert_eq_rendered!(en, "zero");
    let fr = tdbg!(Locale::fr, defaulted_plurals, locale = "en", $ = count);
    assert_eq_rendered!(fr, "zero");

    for i in [-3, 5, 12] {
        let count = move || i;
        let en = tdbg!(Locale::en, defaulted_plurals, locale = "en", $ = count);
        assert_eq_rendered!(en, "this plural is declared in locale en");
        let fr = tdbg!(Locale::fr, defaulted_plurals, locale = "en", $ = count);
        assert_eq_rendered!(fr, "this plural is declared in locale en");
    }
}

#[test]
fn defaulted_foreign_key() {
    let en = tdbg!(Locale::en, defaulted_foreign_key);
    assert_eq_rendered!(en, "before Click to increment the counter after");
    let fr = tdbg!(Locale::fr, defaulted_foreign_key);
    assert_eq_rendered!(fr, "before Click to increment the counter after");
}
