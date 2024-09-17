use crate::i18n::*;
use tests_common::*;

#[test]
fn f32_range() {
    // count = 0
    let count = move || 0.0;
    let en = td!(Locale::en, f32_range, count = count);
    assert_eq_rendered!(en, "You are broke");
    let fr = td!(Locale::fr, f32_range, count = count);
    assert_eq_rendered!(fr, "Vous êtes pauvre");

    // count = ..0
    for i in [-100.34, -57.69, 0.0 - 0.00001] {
        let count = move || i;
        let en = td!(Locale::en, f32_range, count = count);
        assert_eq_rendered!(en, "You owe money");
        let fr = td!(Locale::fr, f32_range, count = count);
        assert_eq_rendered!(fr, "Vous devez de l'argent");
    }

    // count = _
    for i in [100.34, 57.69, 0.0 + 0.00001] {
        let count = move || i;
        let en = td!(Locale::en, f32_range, count = count);
        assert_eq_rendered!(en, format!("You have {}€", i));
        let fr = td!(Locale::fr, f32_range, count = count);
        assert_eq_rendered!(fr, format!("Vous avez {}€", i));
    }
}

#[test]
fn u32_range() {
    // count = 0
    let count = move || 0;
    let en = td!(Locale::en, u32_range, count = count);
    assert_eq_rendered!(en, "0");
    let fr = td!(Locale::fr, u32_range, count = count);
    assert_eq_rendered!(fr, "0");

    // count = 1..
    for i in [1, 45, 72] {
        let count = move || i;
        let en = td!(Locale::en, u32_range, count = count);
        assert_eq_rendered!(en, "1..");
        let fr = td!(Locale::fr, u32_range, count = count);
        assert_eq_rendered!(fr, "1..");
    }
}

#[test]
fn or_range() {
    // count = 0 | 5
    for i in [0, 5] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count = count);
        assert_eq_rendered!(en, "0 or 5");
        let fr = td!(Locale::fr, OR_range, count = count);
        assert_eq_rendered!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for i in [1, 4, 6, 9] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count = count);
        assert_eq_rendered!(en, "1..5 | 6..10");
        let fr = td!(Locale::fr, OR_range, count = count);
        assert_eq_rendered!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for i in [10, 12, 14, 20] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count = count);
        assert_eq_rendered!(en, "10..15 | 20");
        let fr = td!(Locale::fr, OR_range, count = count);
        assert_eq_rendered!(fr, "10..15 | 20");
    }

    // count = _
    for i in [15, 17, 21, 56] {
        let count = move || i;
        let en = td!(Locale::en, OR_range, count = count);
        assert_eq_rendered!(en, "fallback with no count");
        let fr = td!(Locale::fr, OR_range, count = count);
        assert_eq_rendered!(fr, "fallback sans count");
    }
}

#[test]
fn f32_or_range() {
    // count = 0 | 5
    for i in [0.0, 5.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count = count);
        assert_eq_rendered!(en, "0 or 5");
        let fr = td!(Locale::fr, f32_OR_range, count = count);
        assert_eq_rendered!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for i in [1.0, 4.0, 6.0, 9.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count = count);
        assert_eq_rendered!(en, "1..5 | 6..10");
        let fr = td!(Locale::fr, f32_OR_range, count = count);
        assert_eq_rendered!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for i in [10.0, 12.0, 14.0, 20.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count = count);
        assert_eq_rendered!(en, "10..15 | 20");
        let fr = td!(Locale::fr, f32_OR_range, count = count);
        assert_eq_rendered!(fr, "10..15 | 20");
    }

    // count = _
    for i in [15.0, 17.0, 21.0, 56.0] {
        let count = move || i;
        let en = td!(Locale::en, f32_OR_range, count = count);
        assert_eq_rendered!(en, "fallback with no count");
        let fr = td!(Locale::fr, f32_OR_range, count = count);
        assert_eq_rendered!(fr, "fallback avec tuple vide");
    }
}
