use crate::load_locales::warning::{emit_warning, Warning};
use crate::utils::key::Key;
use std::path::Path;
use std::rc::Rc;

thread_local! {
    pub static LOCALES_PATH: std::cell::RefCell<Vec<String>>  = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn generate_file_tracking() -> proc_macro2::TokenStream {
    if cfg!(all(
        feature = "track_locale_files",
        not(feature = "nightly")
    )) {
        LOCALES_PATH.with_borrow(
            |paths| quote::quote!(#(const _: &'static [u8] = include_bytes!(#paths);)*),
        )
    } else {
        proc_macro2::TokenStream::new()
    }
}

pub fn track_file(locale: &Rc<Key>, namespace: Option<&Rc<Key>>, path: &Path) {
    if cfg!(all(
        not(feature = "track_locale_files"),
        not(feature = "nightly")
    )) {
        return;
    }

    if let Some(path) = path.as_os_str().to_str() {
        if cfg!(not(feature = "nightly")) {
            LOCALES_PATH.with_borrow_mut(|paths| paths.push(path.to_owned()));
        }
        #[cfg(feature = "nightly")]
        proc_macro::tracked_path::path(path);
    } else {
        emit_warning(
            Warning::NonUnicodePath {
                locale: locale.clone(),
                namespace: namespace.cloned(),
                path: path.to_owned(),
            },
            None,
        );
    }
}
