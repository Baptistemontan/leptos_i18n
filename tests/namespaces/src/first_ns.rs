use crate::i18n::*;
use tests_common::*;

#[test]
fn click_to_change_lang() {
    let en = td!(Locale::en, first_namespace.click_to_change_lang);
    assert_eq_rendered!(en, "Click to change language");
    let fr = td!(Locale::fr, first_namespace.click_to_change_lang);
    assert_eq_rendered!(fr, "Cliquez pour changez de langue");
}

#[test]
fn common_key() {
    let en = td!(Locale::en, first_namespace.common_key);
    assert_eq_rendered!(en, "first namespace");
    let fr = td!(Locale::fr, first_namespace.common_key);
    assert_eq_rendered!(fr, "premier namespace");
}
