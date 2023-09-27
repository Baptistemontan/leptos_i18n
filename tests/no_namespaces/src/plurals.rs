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
