use crate::i18n::*;
use tests_common::*;

#[test]
fn scoped() {
    let en_scope = scope_locale!(Locale::en, first_namespace);
    let fr_scope = scope_locale!(Locale::fr, first_namespace);

    let en = td!(en_scope, click_to_change_lang);
    assert_eq_rendered!(en, "Click to change language");
    let fr = td!(fr_scope, click_to_change_lang);
    assert_eq_rendered!(fr, "Cliquez pour changez de langue");

    let en = td!(en_scope, common_key);
    assert_eq_rendered!(en, "first namespace");
    let fr = td!(fr_scope, common_key);
    assert_eq_rendered!(fr, "premier namespace");
}

#[test]
fn scoped_sub_subkeys() {
    let en_scope = scope_locale!(Locale::en, second_namespace.subkeys);
    let fr_scope = scope_locale!(Locale::fr, second_namespace.subkeys);

    let en = td!(en_scope, subkey_3, count = 0);
    assert_eq_rendered!(en, "0");
    let fr = td!(fr_scope, subkey_3, count = 4);
    assert_eq_rendered!(fr, "4");
}

#[test]
fn defined_scope() {
    type FirstNamespaceScope = define_scope!(crate::i18n, first_namespace);

    let locale = Locale::en.scope::<FirstNamespaceScope>();

    let en = td!(locale, click_to_change_lang);
    assert_eq_rendered!(en, "Click to change language");

    type SubkeysScope = define_scope!(crate::i18n, second_namespace.subkeys);

    let en_scope = Locale::en.scope::<SubkeysScope>();
    let en = td!(en_scope, subkey_3, count = 0);
    assert_eq_rendered!(en, "0");
}
