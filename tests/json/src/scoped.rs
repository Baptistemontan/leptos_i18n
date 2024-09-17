use crate::i18n::*;
use tests_common::*;

#[test]
fn subkey_3() {
    let en_scope = scope_locale!(Locale::en, subkeys);
    let fr_scope = scope_locale!(Locale::fr, subkeys);

    let count = || 0;
    let en = td!(en_scope, subkey_3, count);
    assert_eq_rendered!(en, "zero");
    let fr = td!(fr_scope, subkey_3, count);
    assert_eq_rendered!(fr, "0");
    let count = || 1;
    let en = td!(en_scope, subkey_3, count);
    assert_eq_rendered!(en, "one");
    let fr = td!(fr_scope, subkey_3, count);
    assert_eq_rendered!(fr, "1");
    let count = || 3;
    let en = td!(en_scope, subkey_3, count);
    assert_eq_rendered!(en, "3");
    let fr = td!(fr_scope, subkey_3, count);
    assert_eq_rendered!(fr, "3");
}
