//! This module contain some helpers for building the locales.

use std::path::Path;
use std::{fs, io};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Does the same as `build` but can receive a custom path to where to find the generated translations.
pub fn build_with_custom_path(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}/*", src.display());

    copy_dir_all(src, dest)
}

/// Utility function to copy the generated translations files to the given destination
///
/// By default when requesting the translations the client expect to find them at "/i18n_translations/{namespace?}/{locale}",
/// so the destination should be such that requesting at those URI send back the files.
///
/// For example if you use `cargo_leptos` the destination should be "./target/site/i18n_translations".
///
/// By default the translations are generated in the "./target/i18n_translations" folder,
/// If you specified another directory you can use `build_with_custom_path` to give the custom path.
pub fn build(dest: impl AsRef<Path>) -> io::Result<()> {
    // keep in sync with the one in "leptos_i18n_macro/src/load_locales/cfg_file.rs"
    const DEFAULT_OUT_DIR: &str = "./target/i18n_translations";
    build_with_custom_path(DEFAULT_OUT_DIR, dest)
}
