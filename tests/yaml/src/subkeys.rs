use crate::i18n::*;
use common::*;

#[test]
fn subkey_1() {
    let en = tdbg!(Locale::en, subkeys.subkey_1);
    assert_eq!(en, "subkey_1");
    let fr = tdbg!(Locale::fr, subkeys.subkey_1);
    assert_eq!(fr, "subkey_1");
}

#[test]
fn subkey_2() {
    let b = |children: ChildrenFn| view! { <b>{children}</b> };
    let en = tdbg!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<b>subkey_2</b>");
    let fr = tdbg!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<b>subkey_2</b>");

    let b = |children: ChildrenFn| view! { <div>"before "{children}" after"</div> };
    let en = tdbg!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<div>before subkey_2 after</div>");
    let fr = tdbg!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<div>before subkey_2 after</div>");
}

#[test]
fn subkey_3() {
    let count = || 0;
    let en = tdbg!(Locale::en, subkeys.subkey_3, count);
    assert_eq_rendered!(en, "zero");
    let fr = tdbg!(Locale::fr, subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "0");
    let count = || 1;
    let en = tdbg!(Locale::en, subkeys.subkey_3, count);
    assert_eq_rendered!(en, "one");
    let fr = tdbg!(Locale::fr, subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "1");
    let count = || 3;
    let en = tdbg!(Locale::en, subkeys.subkey_3, count);
    assert_eq_rendered!(en, "3");
    let fr = tdbg!(Locale::fr, subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "3");
}
