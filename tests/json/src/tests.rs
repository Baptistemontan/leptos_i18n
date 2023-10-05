use crate::i18n::*;
use common::*;

#[test]
fn click_to_change_lang() {
    let en = td!(Locale::en, click_to_change_lang);
    assert_eq!(en, "Click to change language");
    let fr = td!(Locale::fr, click_to_change_lang);
    assert_eq!(fr, "Cliquez pour changez de langue");
}

#[test]
fn click_count() {
    for count in -5..5 {
        let en = td!(Locale::en, click_count, count);
        assert_eq_rendered!(en, format!("You clicked {} times", count));
        let fr = td!(Locale::fr, click_count, count);
        assert_eq_rendered!(fr, format!("Vous avez cliqué {} fois", count));
    }

    let count = "whatever impl into view";
    let en = td!(Locale::en, click_count, count);
    assert_eq_rendered!(en, format!("You clicked {} times", count));
    let fr = td!(Locale::fr, click_count, count);
    assert_eq_rendered!(fr, format!("Vous avez cliqué {} fois", count));

    let count = view! { <p>"even a view!"</p> };
    let en = td!(Locale::en, click_count, count = count.clone());
    assert_eq_rendered!(en, "You clicked <p>even a view!</p> times");
    let fr = td!(Locale::fr, click_count, count);
    assert_eq_rendered!(fr, "Vous avez cliqué <p>even a view!</p> fois");
}

#[test]
fn click_count_string() {
    for count in -5..5 {
        let en = td_string!(Locale::en, click_count, count);
        assert_eq!(en, format!("You clicked {} times", count));
        let fr = td_string!(Locale::fr, click_count, count);
        assert_eq!(fr, format!("Vous avez cliqué {} fois", count));
    }

    let en = td_string!(Locale::en, click_count, count = "a lot of");
    assert_eq!(en, "You clicked a lot of times");
    let fr = td_string!(Locale::fr, click_count, count = "beaucoups de");
    assert_eq!(fr, "Vous avez cliqué beaucoups de fois");
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
