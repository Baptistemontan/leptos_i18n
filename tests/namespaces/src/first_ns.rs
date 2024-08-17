use crate::i18n::*;
use common::*;

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

#[test]
fn plural_only_en() {
    let count = move || 0;
    let en = td!(Locale::en, first_namespace.plural_only_en, $ = count);
    assert_eq_rendered!(en, "exact");
    for i in -3..0 {
        let count = move || i;
        let en = td!(Locale::en, first_namespace.plural_only_en, $ = count);
        assert_eq_rendered!(en, "unbounded start");
    }
    for i in 99..103 {
        let count = move || i;
        let en = td!(Locale::en, first_namespace.plural_only_en, $ = count);
        assert_eq_rendered!(en, "unbounded end");
    }
    for i in 1..3 {
        let count = move || i;
        let en = td!(Locale::en, first_namespace.plural_only_en, $ = count);
        assert_eq_rendered!(en, "excluded end");
    }
    for i in 3..=8 {
        let count = move || i;
        let en = td!(Locale::en, first_namespace.plural_only_en, $ = count);
        assert_eq_rendered!(en, "included end");
    }
    for i in [30, 40, 55, 73] {
        let count = move || i;
        let en = td!(Locale::en, first_namespace.plural_only_en, $ = count);
        assert_eq_rendered!(en, "fallback");
    }
    let fr = td!(Locale::fr, first_namespace.plural_only_en, $ = count);
    assert_eq_rendered!(fr, "pas de plurals en français");
}
