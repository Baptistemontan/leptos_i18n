use crate::i18n::*;
use common::*;

#[test]
fn f32_range() {
    // count = 0
    let count = move || 0.0;
    let en = td!(Locale::en, f32_range, count);
    assert_eq_rendered!(en, "You are broke");
    let fr = td!(Locale::fr, f32_range, count);
    assert_eq_rendered!(fr, "Vous êtes pauvre");

    // count = ..0
    for i in [-100.34, -57.69, 0.0 - 0.00001] {
        let count = move || i;
        let en = td!(Locale::en, f32_range, count);
        assert_eq_rendered!(en, "You owe money");
        let fr = td!(Locale::fr, f32_range, count);
        assert_eq_rendered!(fr, "Vous devez de l'argent");
    }

    // count = _
    for i in [100.34, 57.69, 0.0 + 0.00001] {
        let count = move || i;
        let en = td!(Locale::en, f32_range, count);
        assert_eq_rendered!(en, format!("You have {}€", i));
        let fr = td!(Locale::fr, f32_range, count);
        assert_eq_rendered!(fr, format!("Vous avez {}€", i));
    }
}

#[test]
fn u32_range() {
    // count = 0
    let count = move || 0;
    let en = td!(Locale::en, u32_range, count);
    assert_eq_rendered!(en, "0");
    let fr = td!(Locale::fr, u32_range, count);
    assert_eq_rendered!(fr, "0");

    // count = 1..
    for i in [1, 45, 72] {
        let count = move || i;
        let en = td!(Locale::en, u32_range, count);
        assert_eq_rendered!(en, "1..");
        let fr = td!(Locale::fr, u32_range, count);
        assert_eq_rendered!(fr, "1..");
    }
}

#[test]
fn u32_range_string() {
    // count = 0
    let count = 0;
    let en = td_string!(Locale::en, u32_range, count);
    assert_eq!(en.to_string(), "0");
    let fr = td_string!(Locale::fr, u32_range, count);
    assert_eq!(fr.to_string(), "0");

    // count = 1..
    for count in [1, 45, 72] {
        let en = td_string!(Locale::en, u32_range, count);
        assert_eq!(en.to_string(), "1..");
        let fr = td_string!(Locale::fr, u32_range, count);
        assert_eq!(fr.to_string(), "1..");
    }
}

#[test]
fn or_range() {
    // count = 0 | 5
    for i in [0, 5] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count);
        assert_eq_rendered!(en, "0 or 5");
        let fr = td!(Locale::fr, OR_range, count);
        assert_eq_rendered!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for i in [1, 4, 6, 9] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count);
        assert_eq_rendered!(en, "1..5 | 6..10");
        let fr = td!(Locale::fr, OR_range, count);
        assert_eq_rendered!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for i in [10, 12, 14, 20] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count);
        assert_eq_rendered!(en, "10..15 | 20");
        let fr = td!(Locale::fr, OR_range, count);
        assert_eq_rendered!(fr, "10..15 | 20");
    }

    // count = _
    for i in [15, 17, 21, 56] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count);
        assert_eq_rendered!(en, "fallback with no count");
        let fr = td!(Locale::fr, OR_range, count);
        assert_eq_rendered!(fr, "fallback sans count");
    }
}

#[test]
fn f32_or_range() {
    // count = 0 | 5
    for i in [0.0, 5.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count);
        assert_eq_rendered!(en, "0 or 5");
        let fr = td!(Locale::fr, f32_OR_range, count);
        assert_eq_rendered!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for i in [1.0, 4.0, 6.0, 9.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count);
        assert_eq_rendered!(en, "1..5 | 6..10");
        let fr = td!(Locale::fr, f32_OR_range, count);
        assert_eq_rendered!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for i in [10.0, 12.0, 14.0, 20.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count);
        assert_eq_rendered!(en, "10..15 | 20");
        let fr = td!(Locale::fr, f32_OR_range, count);
        assert_eq_rendered!(fr, "10..15 | 20");
    }

    // count = _
    for i in [15.0, 17.0, 21.0, 56.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count);
        assert_eq_rendered!(en, "fallback with no count");
        let fr = td!(Locale::fr, f32_OR_range, count);
        assert_eq_rendered!(fr, "fallback avec tuple vide");
    }
}

#[test]
fn f32_or_range_string() {
    // count = 0 | 5
    for count in [0.0, 5.0] {
        let en = td_string!(Locale::en, f32_OR_range, count);
        assert_eq!(en, "0 or 5");
        let fr = td_string!(Locale::fr, f32_OR_range, count);
        assert_eq!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for count in [1.0, 4.0, 6.0, 9.0] {
        let en = td_string!(Locale::en, f32_OR_range, count);
        assert_eq!(en, "1..5 | 6..10");
        let fr = td_string!(Locale::fr, f32_OR_range, count);
        assert_eq!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for count in [10.0, 12.0, 14.0, 20.0] {
        let en = td_string!(Locale::en, f32_OR_range, count);
        assert_eq!(en, "10..15 | 20");
        let fr = td_string!(Locale::fr, f32_OR_range, count);
        assert_eq!(fr, "10..15 | 20");
    }

    // count = _
    for count in [15.0, 17.0, 21.0, 56.0] {
        let en = td_string!(Locale::en, f32_OR_range, count);
        assert_eq!(en, "fallback with no count");
        let fr = td_string!(Locale::fr, f32_OR_range, count);
        assert_eq!(fr, "fallback avec tuple vide");
    }
}

#[test]
fn args_to_range() {
    let count = move || 1;
    let en = td!(Locale::en, args_to_range, count);
    assert_eq_rendered!(en, "en 1");
    let fr = td!(Locale::fr, args_to_range, count);
    assert_eq_rendered!(fr, "fr 1");
}

#[test]
fn count_arg_to_range() {
    let en = td!(Locale::en, count_arg_to_range, arg = "en");
    assert_eq_rendered!(en, "en zero");
    let fr = td!(Locale::fr, count_arg_to_range, arg = "fr");
    assert_eq_rendered!(fr, "fr zero");
}

#[test]
fn renamed_ranges_count() {
    let first_count = move || 0.0;
    let second_count = move || 1;
    let en = td!(Locale::en, renamed_ranges_count, first_count, second_count);
    assert_eq_rendered!(en, "You are broke 1..5 | 6..10");
    let fr = td!(Locale::fr, renamed_ranges_count, first_count, second_count);
    assert_eq_rendered!(fr, "Vous êtes pauvre 1..5 | 6..10");
}
