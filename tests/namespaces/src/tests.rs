use crate::i18n::*;
use crate::{assert_eq_rendered, render_to_string};
use leptos::*;

mod first_namespace {
    use super::*;

    #[test]
    fn click_to_change_lang() {
        let en = td!(LocaleEnum::en, first_namespace::click_to_change_lang);
        assert_eq!(en, "Click to change language");
        let fr = td!(LocaleEnum::fr, first_namespace::click_to_change_lang);
        assert_eq!(fr, "Cliquez pour changez de langue");
    }

    #[test]
    fn common_key() {
        let en = td!(LocaleEnum::en, first_namespace::common_key);
        assert_eq!(en, "first namespace");
        let fr = td!(LocaleEnum::fr, first_namespace::common_key);
        assert_eq!(fr, "premier namespace");
    }

    #[test]
    fn plural_only_en() {
        let count = move || 0;
        let en = td!(LocaleEnum::en, first_namespace::plural_only_en, count);
        assert_eq_rendered!(en, "exact");
        for i in -3..0 {
            let count = move || i;
            let en = td!(LocaleEnum::en, first_namespace::plural_only_en, count);
            assert_eq_rendered!(en, "unbounded start");
        }
        for i in 99..103 {
            let count = move || i;
            let en = td!(LocaleEnum::en, first_namespace::plural_only_en, count);
            assert_eq_rendered!(en, "unbounded end");
        }
        for i in 1..3 {
            let count = move || i;
            let en = td!(LocaleEnum::en, first_namespace::plural_only_en, count);
            assert_eq_rendered!(en, "excluded end");
        }
        for i in 3..=8 {
            let count = move || i;
            let en = td!(LocaleEnum::en, first_namespace::plural_only_en, count);
            assert_eq_rendered!(en, "included end");
        }
        for i in [30, 40, 55, 73] {
            let count = move || i;
            let en = td!(LocaleEnum::en, first_namespace::plural_only_en, count);
            assert_eq_rendered!(en, "fallback");
        }
        let fr = td!(LocaleEnum::fr, first_namespace::plural_only_en, count);
        assert_eq_rendered!(fr, "pas de plurals en français");
    }
}

mod second_namespace {
    use super::*;

    #[test]
    fn common_key() {
        let en = td!(LocaleEnum::en, second_namespace::common_key);
        assert_eq!(en, "second namespace");
        let fr = td!(LocaleEnum::fr, second_namespace::common_key);
        assert_eq!(fr, "deuxième namespace");
    }

    #[test]
    fn click_count() {
        for i in -5..=5 {
            let count = move || i;
            let en = td!(LocaleEnum::en, second_namespace::click_count, count);
            assert_eq_rendered!(en, format!("You clicked {} times", i));
            let fr = td!(LocaleEnum::fr, second_namespace::click_count, count);
            assert_eq_rendered!(fr, format!("Vous avez cliqué {} fois", i));
        }
    }

    #[test]
    fn click_to_inc() {
        let en = td!(LocaleEnum::en, second_namespace::click_to_inc);
        assert_eq!(en, "Click to increment the counter");
        let fr = td!(LocaleEnum::fr, second_namespace::click_to_inc);
        assert_eq!(fr, "Cliquez pour incrémenter le compteur");
    }

    #[test]
    fn subkey_1() {
        let en = td!(LocaleEnum::en, second_namespace::subkeys.subkey_1);
        assert_eq!(en, "subkey_1");
        let fr = td!(LocaleEnum::fr, second_namespace::subkeys.subkey_1);
        assert_eq!(fr, "subkey_1");
    }

    #[test]
    fn subkey_2() {
        let b = |children: ChildrenFn| view! { <b>{children}</b> };
        let en = td!(LocaleEnum::en, second_namespace::subkeys.subkey_2, <b>);
        assert_eq_rendered!(en, "<b>subkey_2</b>");
        let fr = td!(LocaleEnum::fr, second_namespace::subkeys.subkey_2, <b>);
        assert_eq_rendered!(fr, "<b>subkey_2</b>");

        let b = |children: ChildrenFn| view! { <div>"before "{children}" after"</div> };
        let en = td!(LocaleEnum::en, second_namespace::subkeys.subkey_2, <b>);
        assert_eq_rendered!(en, "<div>before subkey_2 after</div>");
        let fr = td!(LocaleEnum::fr, second_namespace::subkeys.subkey_2, <b>);
        assert_eq_rendered!(fr, "<div>before subkey_2 after</div>");
    }

    #[test]
    fn subkey_3() {
        let count = || 0;
        let en = td!(LocaleEnum::en, second_namespace::subkeys.subkey_3, count);
        assert_eq_rendered!(en, "zero");
        let fr = td!(LocaleEnum::fr, second_namespace::subkeys.subkey_3, count);
        assert_eq_rendered!(fr, "zero");
        let count = || 1;
        let en = td!(LocaleEnum::en, second_namespace::subkeys.subkey_3, count);
        assert_eq_rendered!(en, "one");
        let fr = td!(LocaleEnum::fr, second_namespace::subkeys.subkey_3, count);
        assert_eq_rendered!(fr, "1");
        let count = || 3;
        let en = td!(LocaleEnum::en, second_namespace::subkeys.subkey_3, count);
        assert_eq_rendered!(en, "3");
        let fr = td!(LocaleEnum::fr, second_namespace::subkeys.subkey_3, count);
        assert_eq_rendered!(fr, "3");
    }
}
