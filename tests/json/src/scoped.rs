use crate::i18n::*;
use tests_common::*;

#[test]
fn subkey_3() {
    let en_scope = scope_locale!(Locale::en, subkeys);
    let fr_scope = scope_locale!(Locale::fr, subkeys);

    let en = td!(en_scope, subkey_3, count = 0);
    assert_eq_rendered!(en, "0");
    let fr = td!(fr_scope, subkey_3, count = 4);
    assert_eq_rendered!(fr, "4");
}
