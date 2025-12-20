use crate::i18n::*;
use tests_common::*;

#[test]
fn subkey_1() {
    let en = td!(Locale::en, subkeys.subkey_1);
    assert_eq_rendered!(en, "subkey_1");
    let fr = td!(Locale::fr, subkeys.subkey_1);
    assert_eq_rendered!(fr, "subkey_1");
}

#[test]
fn subkey_2() {
    let b = |children: ChildrenFn| view! { <b>{move || children()}</b> };
    let en = td!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<b>subkey_2</b>");
    let fr = td!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<b>subkey_2</b>");

    let b = |children: ChildrenFn| view! { <div>"before "{move || children()}" after"</div> };
    let en = td!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<div>before subkey_2 after</div>");
    let fr = td!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<div>before subkey_2 after</div>");
}
