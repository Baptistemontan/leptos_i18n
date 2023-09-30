use crate::i18n::*;
use common::*;

#[test]
fn common_key() {
    let en = td!(Locale::en, second_namespace::common_key);
    assert_eq!(en, "second namespace");
    let fr = td!(Locale::fr, second_namespace::common_key);
    assert_eq!(fr, "deuxième namespace");
}

#[test]
fn click_count() {
    for i in -5..=5 {
        let count = move || i;
        let en = td!(Locale::en, second_namespace::click_count, count);
        assert_eq_rendered!(en, format!("You clicked {} times", i));
        let fr = td!(Locale::fr, second_namespace::click_count, count);
        assert_eq_rendered!(fr, format!("Vous avez cliqué {} fois", i));
    }
}

#[test]
fn click_to_inc() {
    let en = td!(Locale::en, second_namespace::click_to_inc);
    assert_eq!(en, "Click to increment the counter");
    let fr = td!(Locale::fr, second_namespace::click_to_inc);
    assert_eq!(fr, "Cliquez pour incrémenter le compteur");
}

#[test]
fn subkey_1() {
    let en = td!(Locale::en, second_namespace::subkeys.subkey_1);
    assert_eq!(en, "subkey_1");
    let fr = td!(Locale::fr, second_namespace::subkeys.subkey_1);
    assert_eq!(fr, "subkey_1");
}

#[test]
fn subkey_2() {
    let b = |children: ChildrenFn| view! { <b>{children}</b> };
    let en = td!(Locale::en, second_namespace::subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<b>subkey_2</b>");
    let fr = td!(Locale::fr, second_namespace::subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<b>subkey_2</b>");

    let b = |children: ChildrenFn| view! { <div>"before "{children}" after"</div> };
    let en = td!(Locale::en, second_namespace::subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<div>before subkey_2 after</div>");
    let fr = td!(Locale::fr, second_namespace::subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<div>before subkey_2 after</div>");
}

#[test]
fn subkey_3() {
    let count = || 0;
    let en = td!(Locale::en, second_namespace::subkeys.subkey_3, count);
    assert_eq_rendered!(en, "zero");
    let fr = td!(Locale::fr, second_namespace::subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "zero");
    let count = || 1;
    let en = td!(Locale::en, second_namespace::subkeys.subkey_3, count);
    assert_eq_rendered!(en, "one");
    let fr = td!(Locale::fr, second_namespace::subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "1");
    let count = || 3;
    let en = td!(Locale::en, second_namespace::subkeys.subkey_3, count);
    assert_eq_rendered!(en, "3");
    let fr = td!(Locale::fr, second_namespace::subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "3");
}

#[test]
fn foreign_key_to_another_namespace() {
    let en = td!(
        Locale::en,
        second_namespace::foreign_key_to_another_namespace
    );
    assert_eq!(en, "before first namespace after");
    let fr = td!(
        Locale::fr,
        second_namespace::foreign_key_to_another_namespace
    );
    assert_eq!(fr, "before premier namespace after");
}
