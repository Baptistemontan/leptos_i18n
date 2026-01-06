use leptos_i18n_build::{Config, TranslationsInfos};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let cfg = Config::new("en")?
        .add_locale("fr")?
        .add_namespaces(["change_locale", "counter"])?;

    let translations_infos = TranslationsInfos::parse(cfg)?;

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_i18n_module(i18n_mod_directory)?;

    translations_infos
        .get_translations()
        .write_to_dir("./target/i18n")?;

    Ok(())
}
