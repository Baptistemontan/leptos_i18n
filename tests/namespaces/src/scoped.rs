use crate::i18n::*;
use common::*;

#[test]
fn scoped() {
    let en_scope = scope_locale!(Locale::en, first_namespace);
    let fr_scope = scope_locale!(Locale::fr, first_namespace);

    let en = tdbg!(en_scope, click_to_change_lang);
    assert_eq!(en, "Click to change language");
    let fr = tdbg!(fr_scope, click_to_change_lang);
    assert_eq!(fr, "Cliquez pour changez de langue");

    let en = tdbg!(en_scope, common_key);
    assert_eq!(en, "first namespace");
    let fr = tdbg!(fr_scope, common_key);
    assert_eq!(fr, "premier namespace");
}

#[test]
fn scoped_plurals() {
    let en_scope = scope_locale!(Locale::en, first_namespace);
    let fr_scope = scope_locale!(Locale::fr, first_namespace);

    let count = move || 0;
    let en = tdbg!(en_scope, plural_only_en, count);
    assert_eq_rendered!(en, "exact");
    for i in -3..0 {
        let count = move || i;
        let en = tdbg!(en_scope, plural_only_en, count);
        assert_eq_rendered!(en, "unbounded start");
    }
    for i in 99..103 {
        let count = move || i;
        let en = tdbg!(en_scope, plural_only_en, count);
        assert_eq_rendered!(en, "unbounded end");
    }
    for i in 1..3 {
        let count = move || i;
        let en = tdbg!(en_scope, plural_only_en, count);
        assert_eq_rendered!(en, "excluded end");
    }
    for i in 3..=8 {
        let count = move || i;
        let en = tdbg!(en_scope, plural_only_en, count);
        assert_eq_rendered!(en, "included end");
    }
    for i in [30, 40, 55, 73] {
        let count = move || i;
        let en = tdbg!(en_scope, plural_only_en, count);
        assert_eq_rendered!(en, "fallback");
    }
    let fr = tdbg!(Locale::fr, first_namespace.plural_only_en, count);
    assert_eq_rendered!(fr, "pas de plurals en français");
}

#[test]
fn scoped_sub_subkeys() {
    let en_scope = scope_locale!(Locale::en, second_namespace.subkeys);
    let fr_scope = scope_locale!(Locale::fr, second_namespace.subkeys);

    let count = || 0;
    let en = tdbg!(en_scope, subkey_3, count);
    assert_eq_rendered!(en, "zero");
    let fr = tdbg!(fr_scope, subkey_3, count);
    assert_eq_rendered!(fr, "zero");
    let count = || 1;
    let en = tdbg!(en_scope, subkey_3, count);
    assert_eq_rendered!(en, "one");
    let fr = tdbg!(fr_scope, subkey_3, count);
    assert_eq_rendered!(fr, "1");
    let count = || 3;
    let en = tdbg!(en_scope, subkey_3, count);
    assert_eq_rendered!(en, "3");
    let fr = tdbg!(en_scope, subkey_3, count);
    assert_eq_rendered!(fr, "3");
}
