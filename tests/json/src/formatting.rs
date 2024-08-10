use crate::i18n::*;
use common::*;

#[test]
fn list_formatting() {
    let list = move || ["A", "B", "C"];

    let en = tdbg!(Locale::en, list_formatting, list);
    assert_eq_rendered!(en, "A, B, and C");
    let fr = tdbg!(Locale::fr, list_formatting, list);
    assert_eq_rendered!(fr, "A, B ou C");
}
