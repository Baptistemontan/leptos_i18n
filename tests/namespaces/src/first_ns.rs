use crate::i18n::*;
use common::*;

#[test]
fn click_to_change_lang() {
    let en = tdbg!(Locale::en, first_namespace.click_to_change_lang);
    assert_eq!(en, "Click to change language");
    let fr = tdbg!(Locale::fr, first_namespace.click_to_change_lang);
    assert_eq!(fr, "Cliquez pour changez de langue");
}

#[test]
fn common_key() {
    let en = tdbg!(Locale::en, first_namespace.common_key);
    assert_eq!(en, "first namespace");
    let fr = tdbg!(Locale::fr, first_namespace.common_key);
    assert_eq!(fr, "premier namespace");
}
