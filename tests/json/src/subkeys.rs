use crate::i18n::*;
use common::*;

#[test]
fn subkey_1() {
    let en = td!(Locale::en, subkeys.subkey_1);
    assert_eq!(en, "subkey_1");
    let fr = td!(Locale::fr, subkeys.subkey_1);
    assert_eq!(fr, "subkey_1");
}

#[test]
fn subkey_2() {
    let b = |children: ChildrenFn| view! { <b>{children}</b> };
    let en = td!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<b>subkey_2</b>");
    let fr = td!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<b>subkey_2</b>");

    let b = |children: ChildrenFn| view! { <div>"before "{children}" after"</div> };
    let en = td!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_rendered!(en, "<div>before subkey_2 after</div>");
    let fr = td!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_rendered!(fr, "<div>before subkey_2 after</div>");
}

#[test]
fn subkey_2_string() {
    let b = |f: &mut core::fmt::Formatter,
             children: &dyn Fn(&mut core::fmt::Formatter) -> core::fmt::Result|
     -> core::fmt::Result {
        write!(f, "<b>before ")?;
        children(f)?;
        write!(f, " after</b>")
    };
    let en = td_string!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_string!(en, "<b>before subkey_2 after</b>");
    let fr = td_string!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_string!(fr, "<b>before subkey_2 after</b>");

    let b = leptos_i18n::display::DisplayComp("div");
    let en = td_string!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_string!(en, "<div>subkey_2</div>");
    let fr = td_string!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_string!(fr, "<div>subkey_2</div>");
}

#[test]
fn subkey_3() {
    let count = || 0;
    let en = td!(Locale::en, subkeys.subkey_3, count);
    assert_eq_rendered!(en, "zero");
    let fr = td!(Locale::fr, subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "0");
    let count = || 1;
    let en = td!(Locale::en, subkeys.subkey_3, count);
    assert_eq_rendered!(en, "one");
    let fr = td!(Locale::fr, subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "1");
    let count = || 3;
    let en = td!(Locale::en, subkeys.subkey_3, count);
    assert_eq_rendered!(en, "3");
    let fr = td!(Locale::fr, subkeys.subkey_3, count);
    assert_eq_rendered!(fr, "3");
}
