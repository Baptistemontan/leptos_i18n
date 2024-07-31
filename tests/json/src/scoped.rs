use crate::i18n::*;
use common::*;

#[test]
fn subkey_3() {
    let EN = scope_locale!(Locale::en, subkeys);
    let FR = scope_locale!(Locale::en, subkeys);

    let count = || 0;
    let en = tdbg!(EN, subkey_3, count);
    assert_eq_rendered!(en, "zero");
    let fr = tdbg!(FR, subkey_3, count);
    assert_eq_rendered!(fr, "0");
    let count = || 1;
    let en = tdbg!(EN, subkey_3, count);
    assert_eq_rendered!(en, "one");
    let fr = tdbg!(FR, subkey_3, count);
    assert_eq_rendered!(fr, "1");
    let count = || 3;
    let en = tdbg!(EN, subkey_3, count);
    assert_eq_rendered!(en, "3");
    let fr = tdbg!(FR, subkey_3, count);
    assert_eq_rendered!(fr, "3");
}
