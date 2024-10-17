pub fn generate_file_tracking(tracked_files: Option<Vec<String>>) -> proc_macro2::TokenStream {
    if cfg!(all(
        not(feature = "track_locale_files"),
        not(feature = "nightly")
    )) {
        return proc_macro2::TokenStream::new();
    }

    let Some(paths) = tracked_files else {
        return proc_macro2::TokenStream::new();
    };

    if cfg!(all(
        feature = "track_locale_files",
        not(feature = "nightly")
    )) {
        quote::quote!(#(const _: &'static [u8] = include_bytes!(#paths);)*)
    } else {
        #[cfg(feature = "nightly")]
        for path in paths {
            proc_macro::tracked_path::path(path);
        }

        proc_macro2::TokenStream::new()
    }
}
