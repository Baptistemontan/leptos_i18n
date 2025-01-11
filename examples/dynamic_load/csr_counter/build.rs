use leptos_i18n_build::TranslationsInfos;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let translations_infos = TranslationsInfos::parse().unwrap();

    translations_infos.rerun_if_locales_changed();

    translations_infos
        .get_translations()
        .write_to_dir("./target/i18n")
        .unwrap();
}
