use crate::i18n::*;
use common::*;

#[test]
fn f32_plural() {
    // count = 0
    let count = move || 0.0;
    let en = td!(LocaleEnum::en, f32_plural, count);
    assert_eq_rendered!(en, "You are broke");
    let fr = td!(LocaleEnum::fr, f32_plural, count);
    assert_eq_rendered!(fr, "Vous êtes pauvre");

    // count = ..0
    for i in [-100.34, -57.69, 0.0 - 0.00001] {
        let count = move || i;
        let en = td!(LocaleEnum::en, f32_plural, count);
        assert_eq_rendered!(en, "You owe money");
        let fr = td!(LocaleEnum::fr, f32_plural, count);
        assert_eq_rendered!(fr, "Vous devez de l'argent");
    }

    // count = _
    for i in [100.34, 57.69, 0.0 + 0.00001] {
        let count = move || i;
        let en = td!(LocaleEnum::en, f32_plural, count);
        assert_eq_rendered!(en, format!("You have {}€", i));
        let fr = td!(LocaleEnum::fr, f32_plural, count);
        assert_eq_rendered!(fr, format!("Vous avez {}€", i));
    }
}

#[test]
fn u32_plural() {
    // count = 0
    let count = move || 0u32;
    let en = td!(LocaleEnum::en, u32_plural, count);
    assert_eq_rendered!(en, "0");
    let fr = td!(LocaleEnum::fr, u32_plural, count);
    assert_eq_rendered!(fr, "0");

    // count = 1..
    for i in [1u32, 45, 72] {
        let count = move || i;
        let en = td!(LocaleEnum::en, u32_plural, count);
        assert_eq_rendered!(en, "1..");
        let fr = td!(LocaleEnum::fr, u32_plural, count);
        assert_eq_rendered!(fr, "1..");
    }
}

#[test]
fn or_plural() {
    // count = 0 | 5
    for i in [0u8, 5] {
        let count = move || i;
        let en = td!(LocaleEnum::en, OR_plural, count);
        assert_eq_rendered!(en, "0 or 5");
        let fr = td!(LocaleEnum::fr, OR_plural, count);
        assert_eq_rendered!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for i in [1u8, 4, 6, 9] {
        let count = move || i;
        let en = td!(LocaleEnum::en, OR_plural, count);
        assert_eq_rendered!(en, "1..5 | 6..10");
        let fr = td!(LocaleEnum::fr, OR_plural, count);
        assert_eq_rendered!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for i in [10u8, 12, 14, 20] {
        let count = move || i;
        let en = td!(LocaleEnum::en, OR_plural, count);
        assert_eq_rendered!(en, "10..15 | 20");
        let fr = td!(LocaleEnum::fr, OR_plural, count);
        assert_eq_rendered!(fr, "10..15 | 20");
    }

    // count = _
    for i in [15u8, 17, 21, 56] {
        let count = move || i;
        let en = td!(LocaleEnum::en, OR_plural, count);
        assert_eq_rendered!(en, "fallback with no count");
        let fr = td!(LocaleEnum::fr, OR_plural, count);
        assert_eq_rendered!(fr, "fallback sans count");
    }
}

#[test]
fn f32_or_plural() {
    // count = 0 | 5
    for i in [0.0, 5.0] {
        let count = move || i;
        let en = td!(LocaleEnum::en, f32_OR_plural, count);
        assert_eq_rendered!(en, "0 or 5");
        let fr = td!(LocaleEnum::fr, f32_OR_plural, count);
        assert_eq_rendered!(fr, "0 or 5");
    }

    // count = 1..5 | 6..10
    for i in [1.0, 4.0, 6.0, 9.0] {
        let count = move || i;
        let en = td!(LocaleEnum::en, f32_OR_plural, count);
        assert_eq_rendered!(en, "1..5 | 6..10");
        let fr = td!(LocaleEnum::fr, f32_OR_plural, count);
        assert_eq_rendered!(fr, "1..5 | 6..10");
    }

    // count = 10..15 | 20
    for i in [10.0, 12.0, 14.0, 20.0] {
        let count = move || i;
        let en = td!(LocaleEnum::en, f32_OR_plural, count);
        assert_eq_rendered!(en, "10..15 | 20");
        let fr = td!(LocaleEnum::fr, f32_OR_plural, count);
        assert_eq_rendered!(fr, "10..15 | 20");
    }

    // count = _
    for i in [15.0, 17.0, 21.0, 56.0] {
        let count = move || i;
        let en = td!(LocaleEnum::en, f32_OR_plural, count);
        assert_eq_rendered!(en, "fallback with no count");
        let fr = td!(LocaleEnum::fr, f32_OR_plural, count);
        assert_eq_rendered!(fr, "fallback avec tuple vide");
    }
}
