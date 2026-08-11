use crate::i18n::*;
use tests_common::*;

#[test]
fn try_use_i18n_without_provider() {
    assert!(try_use_i18n().is_none());
}

#[test]
fn try_use_i18n_scoped_without_provider() {
    type SubkeysScope = define_scope!(crate::i18n, subkeys);

    assert!(try_use_i18n_scoped::<SubkeysScope>().is_none());
}

#[test]
fn use_i18n_or_without_provider() {
    let i18n = use_i18n_or(Locale::fr);
    assert_eq!(i18n.get_locale_untracked(), Locale::fr);
    assert_eq_rendered!(t!(i18n, click_to_change_lang), "Cliquez pour changez de langue");
}

#[test]
fn use_i18n_or_else_without_provider() {
    let i18n = use_i18n_or_else(|| Locale::fr);
    assert_eq!(i18n.get_locale_untracked(), Locale::fr);
}
