use std::{path::PathBuf, rc::Rc};

use super::key::Key;

#[cfg(all(feature = "track_locale_files", not(feature = "nightly")))]
thread_local! {
    pub static LOCALES_PATH: std::cell::RefCell<Vec<String>>  = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(all(feature = "track_locale_files", not(feature = "nightly")))]
pub fn generate_file_tracking() -> proc_macro2::TokenStream {
    LOCALES_PATH
        .with_borrow(|paths| quote::quote!(#(const _: &'static [u8] = include_bytes!(#paths);)*))
}

#[cfg(any(not(feature = "track_locale_files"), feature = "nightly"))]
pub fn generate_file_tracking() -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::new()
}

#[cfg(any(feature = "nightly", feature = "track_locale_files"))]
pub fn track_file(locale: &Rc<Key>, namespace: Option<&Rc<Key>>, path: &PathBuf) {
    use super::warning::Warning;
    if let Some(path) = path.as_os_str().to_str() {
        #[cfg(all(feature = "track_locale_files", not(feature = "nightly")))]
        LOCALES_PATH.with_borrow_mut(|paths| paths.push(path.to_owned()));
        #[cfg(feature = "nightly")]
        proc_macro::tracked_path::path(path);
    } else {
        super::warning::emit_warning(Warning::NonUnicodePath {
            locale: locale.clone(),
            namespace: namespace.cloned(),
            path: path.clone(),
        });
    }
}

#[cfg(not(any(feature = "nightly", feature = "track_locale_files")))]
pub fn track_file(locale: &Rc<Key>, namespace: Option<&Rc<Key>>, path: &PathBuf) {
    let _ = (locale, namespace, path);
}
