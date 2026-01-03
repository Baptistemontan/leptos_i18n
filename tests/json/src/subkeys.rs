use crate::i18n::*;
use leptos_i18n::display::Attributes;
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

    let en = td!(Locale::en, subkeys.subkey_2, <b> = <span id="test"/>);
    assert_eq_rendered!(en, "<span id=\"test\">subkey_2</span>");
    let fr = td!(Locale::fr, subkeys.subkey_2, <b> = <span/>);
    assert_eq_rendered!(fr, "<span>subkey_2</span>");
}

#[test]
fn subkey_2_string() {
    let b = |f: &mut core::fmt::Formatter,
             attrs: Attributes,
             children: &dyn Fn(&mut core::fmt::Formatter) -> core::fmt::Result|
     -> core::fmt::Result {
        write!(f, "<b{attrs}>before ")?;
        children(f)?;
        write!(f, " after</b>")
    };
    let en = td_display!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq_string!(en, "<b>before subkey_2 after</b>");
    let fr = td_display!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq_string!(fr, "<b>before subkey_2 after</b>");

    let en = td_string!(Locale::en, subkeys.subkey_2, <b> = "div");
    assert_eq!(en, "<div>subkey_2</div>");
    let fr = td_string!(Locale::fr, subkeys.subkey_2, <b> = "div");
    assert_eq!(fr, "<div>subkey_2</div>");

    let attrs = [("id", "my_id")];

    let b = leptos_i18n::display::DisplayComp::new("span", &attrs);

    let en = td_string!(Locale::en, subkeys.subkey_2, <b>);
    assert_eq!(en, "<span id=\"my_id\">subkey_2</span>");
    let fr = td_string!(Locale::fr, subkeys.subkey_2, <b>);
    assert_eq!(fr, "<span id=\"my_id\">subkey_2</span>");
}
