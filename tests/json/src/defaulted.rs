use crate::i18n::*;
use tests_common::*;

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
fn defaulted_foreign_key() {
    let en = td!(Locale::en, defaulted_foreign_key);
    assert_eq_rendered!(en, "before Click to increment the counter after");
    let fr = td!(Locale::fr, defaulted_foreign_key);
    assert_eq_rendered!(fr, "before Click to increment the counter after");
}

#[test]
fn defaulted_subkeys() {
    let en = td!(Locale::en, defaulted_subkeys.subkey);
    assert_eq_rendered!(en, "some string");
    let fr = td!(Locale::fr, defaulted_subkeys.subkey);
    assert_eq_rendered!(fr, "some string");
}
