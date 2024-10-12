// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use icu_datagen::baked_exporter::*;
use icu_datagen::keys;
use icu_datagen::prelude::*;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");

    if mod_directory.exists() {
        std::fs::remove_dir_all(&mod_directory).unwrap();
    }

    let exporter = BakedExporter::new(mod_directory, Default::default()).unwrap();

    DatagenDriver::new()
        .with_keys(keys(&["plurals/cardinal@1", "plurals/ordinal@1"]))
        .with_locales_no_fallback([langid!("en"), langid!("fr")], Default::default())
        .export(&DatagenProvider::new_latest_tested(), exporter)
        .unwrap();
}
