use crate::i18n::*;

#[test]
fn try_use_i18n_without_provider() {
    assert!(try_use_i18n().is_none());
}

#[test]
fn try_use_i18n_scoped_without_provider() {
    type SubkeysScope = define_scope!(crate::i18n, subkeys);

    assert!(try_use_i18n_scoped::<SubkeysScope>().is_none());
}
