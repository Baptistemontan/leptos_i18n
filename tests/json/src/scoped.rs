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

#[test]
fn define_scope_subkey_3() {
    type SubkeysScope = define_scope!(crate::i18n, subkeys);

    let en_scope = Locale::en.scope::<SubkeysScope>();
    let fr_scope = Locale::fr.scope::<SubkeysScope>();

    let en = td!(en_scope, subkey_3, count = 0);
    assert_eq_rendered!(en, "0");
    let fr = td!(fr_scope, subkey_3, count = 4);
    assert_eq_rendered!(fr, "4");
}
