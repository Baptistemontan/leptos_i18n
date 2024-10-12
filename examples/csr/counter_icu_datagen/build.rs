use icu_datagen::baked_exporter::*;
use icu_datagen::prelude::*;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");

    // This is'nt really needed, but ICU4X wants the directory to be empty
    // and Rust Analyzer can trigger the build.rs without cleaning the out directory.
    if mod_directory.exists() {
        std::fs::remove_dir_all(&mod_directory).unwrap();
    }

    let exporter = BakedExporter::new(mod_directory, Default::default()).unwrap();

    DatagenDriver::new()
        // Keys needed for plurals
        .with_keys(icu_datagen::keys(&[
            "plurals/cardinal@1",
            "plurals/ordinal@1",
        ]))
        // Used locales, no fallback needed
        .with_locales_no_fallback([langid!("en"), langid!("fr")], Default::default())
        .export(&DatagenProvider::new_latest_tested(), exporter)
        .unwrap();
}
