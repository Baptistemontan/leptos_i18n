//! Keys as values: `key_builder!`, `build_key!`, `key!`, `display_key!`, ... and rendering them with `key = ..`.

use crate::i18n::*;
use leptos_i18n::keys::AnyKey;
use leptos_i18n::{build_key, build_key_display, display_key, key, key_builder, td_const};
use tests_common::*;

#[test]
fn build_key_without_args() {
    let en = build_key!(key_builder!(Locale::en, click_to_inc));
    assert_eq_rendered!(td!(Locale::en, key = en), "Click to increment the counter");
    let fr = build_key!(key_builder!(Locale::fr, click_to_inc));
    assert_eq_rendered!(
        td!(Locale::fr, key = fr),
        "Cliquez pour incrémenter le compteur"
    );
}

#[test]
fn build_key_with_args() {
    let builder = key_builder!(Locale::en, click_count);
    let en = build_key!(builder, count = 3);
    assert_eq_rendered!(td!(Locale::en, key = en), "You clicked 3 times");
    let fr = build_key!(builder, count = 3);
    assert_eq_rendered!(td!(Locale::fr, key = fr), "Vous avez cliqué 3 fois");
}

#[test]
fn key_macro() {
    let en = key!(Locale::en, click_count, count = 5);
    assert_eq_rendered!(td!(Locale::en, key = en), "You clicked 5 times");

    let en = key!(Locale::en, subkeys.subkey_2, <b> = <span />);
    assert_eq_rendered!(td!(Locale::en, key = en), "<span>subkey_2</span>");
}

#[test]
fn key_macro_with_scope() {
    type SubkeysScope = define_scope!(Locale, subkeys);

    let en = key!(scope = SubkeysScope, subkey_3, count = 7);
    assert_eq_rendered!(td!(Locale::en, key = en), "7");

    let builder = key_builder!(scope = SubkeysScope, subkey_1);
    assert_eq_rendered!(td!(Locale::en, key = build_key!(builder)), "subkey_1");
}

#[test]
fn display_keys() {
    let en = display_key!(Locale::en, click_count, count = 2);
    assert_eq!(td_string!(Locale::en, key = en), "You clicked 2 times");

    let en = display_key!(Locale::en, subkeys.subkey_2, <b> = "span");
    assert_eq!(td_string!(Locale::en, key = en), "<span>subkey_2</span>");

    let builder = key_builder!(Locale::fr, click_count);
    let fr = build_key_display!(builder, count = 2);
    assert_eq!(td_string!(Locale::fr, key = fr), "Vous avez cliqué 2 fois");
    assert_eq!(
        td_display!(Locale::fr, key = fr).to_string(),
        "Vous avez cliqué 2 fois"
    );
}

#[test]
fn downgrade_keys_with_the_same_args() {
    // same args (none): same type once downgraded
    let keys = [
        key!(Locale::en, click_to_inc).downgrade(),
        key!(Locale::en, click_to_change_lang).downgrade(),
    ];
    let rendered = keys.map(|key| render_to_string(td!(Locale::en, key = key)));
    assert_eq!(
        rendered,
        ["Click to increment the counter", "Click to change language"]
    );
}

#[test]
fn downgrade_builders_with_the_same_args() {
    // `click_count` and `subkeys.subkey_3` both only take `count`
    for first in [true, false] {
        let builder = if first {
            key_builder!(Locale::en, click_count).downgrade()
        } else {
            key_builder!(Locale::en, subkeys.subkey_3).downgrade()
        };
        let key = build_key!(builder, count = 4);
        let expected = if first { "You clicked 4 times" } else { "4" };
        assert_eq_rendered!(td!(Locale::en, key = key), expected);
    }
}

#[test]
fn downgrade_any() {
    let keys: Vec<AnyKey<Locale>> = vec![
        key!(Locale::en, click_to_inc).downgrade_any(),
        key!(Locale::en, click_count, count = 1).downgrade_any(),
        // an already downgraded key goes to `AnyKey` with `downgrade`
        key!(Locale::en, click_count, count = 2)
            .downgrade()
            .downgrade(),
    ];
    let rendered: Vec<_> = keys
        .into_iter()
        .map(|key| render_to_string(td!(Locale::en, key = key)))
        .collect();
    assert_eq!(
        rendered,
        [
            "Click to increment the counter",
            "You clicked 1 times",
            "You clicked 2 times"
        ]
    );
}

#[test]
fn const_access() {
    const EN: &str = td_const!(Locale::en, click_to_inc);
    assert_eq!(EN, "Click to increment the counter");
    const FR: &str = td_const!(Locale::fr, subkeys.subkey_1);
    assert_eq!(FR, "subkey_1");
    const SAME_LIT_TYPE: bool = td_const!(Locale::en, same_lit_type);
    assert!(SAME_LIT_TYPE);
}
