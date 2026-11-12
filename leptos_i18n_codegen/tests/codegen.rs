//! Regression tests for the code generation.
//!
//! Each test parses some translations, generate the code, and inspects it.

use std::{borrow::Cow, collections::HashMap, mem};

use leptos_i18n_parser::parse_locales::{
    error::{Diagnostics, Warning},
    files::{LocaleFiles, TranslationFiles},
    options::Config,
    parse_locales_with_files,
};

use leptos_i18n_parser::parse_locales::error::Result;

fn gen_code(files: &[(&str, &str)], extensions: &[(&str, &str)]) -> Result<(String, Diagnostics)> {
    let mut locales = HashMap::with_capacity(files.len());
    for (locale, file) in files {
        locales.insert(*locale, (None, Cow::Borrowed(*file)));
    }
    let t_files = TranslationFiles::Locales {
        locales: LocaleFiles { files: locales },
    };
    let mut cfg = Config::new(files[0].0)
        .unwrap()
        .add_locales(files.iter().skip(1).map(|l| l.0))
        .unwrap();

    for (l1, l2) in extensions {
        cfg = cfg.extend_locale(l1, l2).unwrap();
    }

    let mut parsed_locales = parse_locales_with_files(&t_files, cfg)?;
    let diag = mem::take(&mut parsed_locales.diag);
    let code = leptos_i18n_codegen::gen_code(&parsed_locales, None, true, None, false)?.to_string();
    Ok((code, diag))
}

#[test]
fn warn_on_missing_key() {
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": "" }"#), ("fr", r"{}")], &[]).unwrap();

    let has_surplus = diag
        .warnings()
        .iter()
        .any(|w| matches!(w, Warning::MissingKey { .. }));

    assert!(has_surplus);
}

#[test]
fn warn_on_surplus_key() {
    let (_code, diag) = gen_code(
        &[
            ("en", r#"{ "my_key": "" }"#),
            ("fr", r#"{ "my_key": "", "surplus": "" }"#),
        ],
        &[],
    )
    .unwrap();

    let has_surplus = diag
        .warnings()
        .iter()
        .any(|w| matches!(w, Warning::SurplusKey { .. }));

    assert!(has_surplus);
}

#[test]
fn suppress_missing_key_on_inherit() {
    let (_code, diag) = gen_code(
        &[("en", r#"{ "my_key": "" }"#), ("fr", r"{}")],
        &[("fr", "en")],
    )
    .unwrap();

    let has_surplus = diag
        .warnings()
        .iter()
        .any(|w| matches!(w, Warning::MissingKey { .. }));

    assert!(!has_surplus);
}
