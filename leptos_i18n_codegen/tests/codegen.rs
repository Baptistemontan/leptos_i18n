// `dynamic_load` changes the shape of the generated accessors and needs a translations URI to be
// configured, the assertions below are written against the default codegen.
#![cfg(not(feature = "dynamic_load"))]

//! Regression tests for the code generation.
//!
//! Each test writes a locale fixture to disk, parses it exactly like the `load_locales!` macro
//! does and inspects the generated token stream.

use std::path::{Path, PathBuf};

use leptos_i18n_parser::parse_locales::{
    cfg_file::ConfigFile,
    options::{Config, FileFormat, ParseOptions},
    parse_locales,
};

const BASE_CARGO: &str = r#"
[package]
name = "test"

[package.metadata.leptos-i18n]
default = "en"
locales = ["en"]
"#;

fn fixture(name: &str, cargo: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), cargo).unwrap();
    for (path, content) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    dir
}

fn gen_for(dir: PathBuf) -> String {
    gen_with(dir, None, FileFormat::Json)
}

fn gen_with(dir: PathBuf, crate_path: Option<&str>, file_format: FileFormat) -> String {
    let mut manifest_dir = dir;
    let cfg_file = ConfigFile::new(&mut manifest_dir).unwrap();
    let mut cfg: Config = cfg_file.into();
    cfg.options = ParseOptions::default().file_format(file_format);
    let parsed = parse_locales(Some(manifest_dir), cfg).unwrap();
    let crate_path = crate_path.map(|path| syn::parse_str::<syn::Path>(path).unwrap());
    leptos_i18n_codegen::gen_code(&parsed, crate_path.as_ref(), true, None, true)
        .unwrap()
        .to_string()
}

/// `Key` keeps the raw name for display and a sanitized ident for codegen, the builder type must
/// be named after the ident, `-` is not valid in an ident.
#[test]
fn kebab_case_key_with_interpolation() {
    let dir = fixture(
        "kebab_case_key_with_interpolation",
        BASE_CARGO,
        &[("locales/en.json", r#"{ "my-key": "hello {{ count }}" }"#)],
    );

    let code = gen_for(dir);

    assert!(code.contains("struct my_key_builder"));
}

#[test]
fn kebab_case_key_without_interpolation() {
    let dir = fixture(
        "kebab_case_key_without_interpolation",
        BASE_CARGO,
        &[("locales/en.json", r#"{ "my-key": "hello" }"#)],
    );

    let code = gen_for(dir);

    assert!(code.contains("pub const fn my_key (self)"));
}
