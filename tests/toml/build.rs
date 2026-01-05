use leptos_i18n_build::{Config, FileFormat, ParseOptions, TranslationsInfos};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let options = ParseOptions::default()
        .interpolate_display(true)
        .file_format(FileFormat::Toml);

    let cfg = Config::new("en")?.add_locale("fr")?.parse_options(options);

    let translations_infos = TranslationsInfos::parse(cfg).unwrap();

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_i18n_module(i18n_mod_directory)?;

    Ok(())
}
