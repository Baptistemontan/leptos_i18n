use leptos_i18n_build::TranslationsInfos;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let translations_infos = TranslationsInfos::parse(Default::default()).unwrap();

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos
        .generate_i18n_module(i18n_mod_directory)
        .unwrap();

    translations_infos
        .get_translations()
        .write_to_dir("./target/i18n")
        .unwrap();
}
