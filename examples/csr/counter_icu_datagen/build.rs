use leptos_i18n_build::TranslationsInfos;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");

    let translations_infos = TranslationsInfos::parse().unwrap();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_data(mod_directory).unwrap();
}
