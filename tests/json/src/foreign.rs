use crate::i18n::*;
use common::*;

#[test]
fn foreign_key_to_string() {
    let en = td!(Locale::en, foreign_key_to_string);
    assert_eq!(en, "before Click to increment the counter after");
    let fr = td!(Locale::fr, foreign_key_to_string);
    assert_eq!(fr, "before Cliquez pour incrémenter le compteur after");
}

#[test]
fn foreign_key_to_interpolation() {
    for count in -5..5 {
        let en = td!(Locale::en, foreign_key_to_interpolation, count);
        assert_eq_rendered!(en, format!("before You clicked {} times after", count));
        let fr = td!(Locale::fr, foreign_key_to_interpolation, count);
        assert_eq_rendered!(fr, format!("before Vous avez cliqué {} fois after", count));
    }

    let count = "whatever impl into view";
    let en = td!(Locale::en, foreign_key_to_interpolation, count);
    assert_eq_rendered!(en, format!("before You clicked {} times after", count));
    let fr = td!(Locale::fr, foreign_key_to_interpolation, count);
    assert_eq_rendered!(fr, format!("before Vous avez cliqué {} fois after", count));

    let count = view! { <p>"even a view!"</p> };
    let en = td!(
        Locale::en,
        foreign_key_to_interpolation,
        count = count.clone()
    );
    assert_eq_rendered!(en, "before You clicked <p>even a view!</p> times after");
    let fr = td!(Locale::fr, foreign_key_to_interpolation, count);
    assert_eq_rendered!(fr, "before Vous avez cliqué <p>even a view!</p> fois after");
}

#[test]
fn foreign_key_to_subkey() {
    let en = td!(Locale::en, foreign_key_to_subkey);
    assert_eq!(en, "before subkey_1 after");
    let fr = td!(Locale::fr, foreign_key_to_subkey);
    assert_eq!(fr, "before subkey_1 after");
}

#[test]
fn foreign_key_to_explicit_default() {
    let en = td!(Locale::en, foreign_key_to_explicit_default);
    assert_eq!(en, "no explicit default in default locale");
    let fr = td!(Locale::fr, foreign_key_to_explicit_default);
    assert_eq!(fr, "before this string is declared in locale en after");
}

#[test]
fn populated_foreign_key() {
    let en = td!(Locale::en, populated_foreign_key);
    assert_eq!(en, "before You clicked 45 times after");
    let fr = td!(Locale::fr, populated_foreign_key);
    assert_eq!(fr, "before Vous avez cliqué 32 fois after");
}
