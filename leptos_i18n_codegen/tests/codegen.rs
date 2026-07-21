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

/// `"a-b"` and `"a_b"` are two distinct keys resolving to the same ident, generating an accessor
/// for both would emit duplicate methods (`E0592`).
#[test]
fn colliding_keys_are_reported() {
    let dir = fixture(
        "colliding_keys_are_reported",
        BASE_CARGO,
        &[("locales/en.json", r#"{ "a-b": "x", "a_b": "y" }"#)],
    );

    let code = gen_for(dir);

    assert!(code.contains("conflicting keys"), "{code}");
    assert!(code.contains("\\\"a-b\\\", \\\"a_b\\\""), "{code}");
    // the duplicated accessor is skipped so the collision is the only reported error.
    assert_eq!(code.matches("pub const fn a_b (self)").count(), 1, "{code}");
}

#[test]
fn colliding_subkeys_are_reported() {
    let dir = fixture(
        "colliding_subkeys_are_reported",
        BASE_CARGO,
        &[(
            "locales/en.json",
            r#"{ "sub": { "a-b": "x", "a_b": "y" } }"#,
        )],
    );

    let code = gen_for(dir);

    assert!(code.contains("conflicting keys at \\\"sub\\\""), "{code}");
}

#[test]
fn colliding_namespaces_are_reported() {
    let cargo = r#"
[package]
name = "test"

[package.metadata.leptos-i18n]
default = "en"
locales = ["en"]
namespaces = ["a-b", "a_b"]
"#;
    let dir = fixture(
        "colliding_namespaces_are_reported",
        cargo,
        &[
            ("locales/en/a-b.json", r#"{ "hello": "world" }"#),
            ("locales/en/a_b.json", r#"{ "hello": "world" }"#),
        ],
    );

    let code = gen_for(dir);

    assert!(code.contains("conflicting keys"), "{code}");
    assert_eq!(code.matches("pub fn a_b (self)").count(), 1, "{code}");
}

#[test]
fn non_colliding_keys_are_not_reported() {
    let dir = fixture(
        "non_colliding_keys_are_not_reported",
        BASE_CARGO,
        &[("locales/en.json", r#"{ "a-b": "x", "c_d": "y" }"#)],
    );

    let code = gen_for(dir);

    assert!(!code.contains("compile_error"), "{code}");
}

/// Extract the brace balanced block following `start`.
fn block_after(code: &str, start: &str) -> String {
    let start = code
        .find(start)
        .expect("pattern not found in generated code");
    let block_start = code[start..].find('{').expect("no block found") + start;
    let mut depth = 0usize;
    for (i, c) in code[block_start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return code[block_start..=block_start + i].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced block in generated code");
}

/// Only floats are required to have a fallback, so integer ranges can leave holes in the domain of
/// the count, which would generate a non exhaustive `match` (`E0004`).
#[test]
fn non_exhaustive_integer_ranges_are_reported() {
    let dir = fixture(
        "non_exhaustive_integer_ranges_are_reported",
        BASE_CARGO,
        &[(
            "locales/en.json",
            r#"{ "r": [["zero", 0], ["some", "1..=3"]] }"#,
        )],
    );

    let code = gen_for(dir);
    let block = block_after(&code, "match var_count ()");

    assert!(block.contains("_ => core :: compile_error !"), "{block}");
    assert!(
        block.contains("missing: -2147483648..=-1, 4..=2147483647"),
        "{block}"
    );
}

#[test]
fn exhaustive_integer_ranges_have_no_wildcard_arm() {
    let dir = fixture(
        "exhaustive_integer_ranges_have_no_wildcard_arm",
        BASE_CARGO,
        &[(
            "locales/en.json",
            r#"{ "r": [["neg", "..0"], ["zero", 0], ["pos", "1.."]] }"#,
        )],
    );

    let code = gen_for(dir);
    let block = block_after(&code, "match var_count ()");

    // an unneeded wildcard arm would trigger `unreachable_patterns` in the user crate.
    assert!(!block.contains("_ =>"), "{block}");
}

#[test]
fn integer_ranges_with_fallback_have_no_error_arm() {
    let dir = fixture(
        "integer_ranges_with_fallback_have_no_error_arm",
        BASE_CARGO,
        &[(
            "locales/en.json",
            r#"{ "r": [["zero", 0], ["some", "1..=3"], ["other", "_"]] }"#,
        )],
    );

    let code = gen_for(dir);

    assert!(!code.contains("compile_error"), "{code}");
}
