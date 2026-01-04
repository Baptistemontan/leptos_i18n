use crate::i18n::*;
use tests_common::*;

#[test]
fn click_to_change_lang() {
    let en = td!(Locale::en, click_to_change_lang);
    assert_eq_rendered!(en, "Click to change language");
    let fr = td!(Locale::fr, click_to_change_lang);
    assert_eq_rendered!(fr, "Cliquez pour changez de langue");
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
        assert_eq!(en, format!("You clicked {count} times"));
        let fr = td_string!(Locale::fr, click_count, count);
        assert_eq!(fr, format!("Vous avez cliqué {count} fois"));
    }

    let en = td_string!(Locale::en, click_count, count = "a lot of");
    assert_eq!(en, "You clicked a lot of times");
    let fr = td_string!(Locale::fr, click_count, count = "beaucoups de");
    assert_eq!(fr, "Vous avez cliqué beaucoups de fois");
}

#[test]
fn same_lit_type() {
    let en = td!(Locale::en, same_lit_type);
    assert_eq_rendered!(en, "true");
    let fr = td!(Locale::fr, same_lit_type);
    assert_eq_rendered!(fr, "false");
}

#[test]
fn mixed_lit_type() {
    let en = td!(Locale::en, mixed_lit_type);
    assert_eq_rendered!(en, "59.89");
    let fr = td!(Locale::fr, mixed_lit_type);
    assert_eq_rendered!(fr, "true");
}
