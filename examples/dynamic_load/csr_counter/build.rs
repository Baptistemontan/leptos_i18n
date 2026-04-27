use leptos_i18n_build::{optins::CodegenOptions, Config, TranslationsInfos};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let cfg = Config::new("en")?.add_locale("fr")?;
    let codegen_options = CodegenOptions::default().translations_uri("i18n/{locale}.json");

    let translations_infos = TranslationsInfos::parse(cfg)?;

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_i18n_module_with_options(i18n_mod_directory, codegen_options)?;

    translations_infos
        .get_translations()
        .write_to_dir("./target/i18n")?;

    Ok(())
}
