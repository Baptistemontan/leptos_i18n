use crate::i18n::*;
use common::*;

#[test]
fn actual_range() {
    // count = 0
    let count = move || 0;
    let en = td!(Locale::en, actual_plural, $ = count);
    assert_eq_rendered!(en, "0 items");
    let fr = td!(Locale::fr, actual_plural, $ = count);
    assert_eq_rendered!(fr, "0");

    // count = 1
    let count = move || 1;
    let en = td!(Locale::en, actual_plural, $ = count);
    assert_eq_rendered!(en, "one item");
    let fr = td!(Locale::fr, actual_plural, $ = count);
    assert_eq_rendered!(fr, "1");

    // count = 2..
    for i in [2, 5, 10, 1000] {
        let count = move || i;
        let en = td!(Locale::en, actual_plural, $ = count);
        assert_eq_rendered!(en, format!("{} items", i));
        let fr = td!(Locale::fr, actual_plural, $ = count);
        assert_eq_rendered!(fr, i.to_string());
    }
}
