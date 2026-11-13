//! Regression tests for the code generation.
//!
//! Each test parses some translations, generate the code, and inspects it.

use std::{borrow::Cow, mem};

use leptos_i18n_codegen::CodegenOptions;
use leptos_i18n_parser::{
    error::{Diagnostics, Error, Result, Warning},
    extraction::extract_locales,
    options::{Config, FileFormat, ParseOptions},
    parsing::{LocaleFiles, RawParsedLocales, TranslationFiles, parse_locales_raw_with_files},
};

/// `(locale, file content)` pairs.
type Files<'a> = [(&'a str, &'a str)];

fn locale_files<'a>(files: &Files<'a>) -> LocaleFiles<'a> {
    LocaleFiles {
        files: files
            .iter()
            .map(|(locale, file)| (*locale, (None, Cow::Borrowed(*file))))
            .collect(),
    }
}

fn new_files<'a>(namespaces: &[(Option<&'a str>, &Files<'a>)]) -> TranslationFiles<'a> {
    let locales = namespaces[0].1;
    match namespaces[0].0 {
        None => TranslationFiles::Locales {
            locales: locale_files(locales),
        },
        Some(_) => TranslationFiles::NameSpaces {
            namespaces: namespaces
                .iter()
                .map(|(ns, files)| (ns.unwrap(), locale_files(files)))
                .collect(),
        },
    }
}

fn gen_code_with(
    namespaces: &[(Option<&str>, &Files)],
    extensions: &[(&str, &str)],
    file_format: FileFormat,
    crate_path: Option<&str>,
) -> Result<(String, Diagnostics)> {
    let locales = namespaces[0].1;
    let t_files = new_files(namespaces);
    let mut cfg = Config::new(locales[0].0)?
        .add_locales(locales.iter().skip(1).map(|l| l.0))?
        .parse_options(ParseOptions::default().file_format(file_format));
    if namespaces[0].0.is_some() {
        cfg = cfg.add_namespaces(namespaces.iter().map(|(ns, _)| ns.unwrap()))?;
    }

    for (l1, l2) in extensions {
        cfg = cfg.extend_locale(l1, l2)?;
    }
    let RawParsedLocales {
        values,
        diag,
        tracked_files: _,
    } = parse_locales_raw_with_files(&t_files, &cfg)?;
    let mut parsed_locales = extract_locales(values, &cfg, Some(diag))?;
    let diag = mem::take(&mut parsed_locales.diag);
    let crate_path = crate_path.map(|p| syn::parse_str::<syn::Path>(p).unwrap());
    let options = CodegenOptions::default()
        .translations_uri("test")
        .crate_path(crate_path.as_ref());
    let code = leptos_i18n_codegen::gen_code(&parsed_locales, options, &cfg)?;
    Ok((code.to_string(), diag))
}

fn gen_code(files: &Files, extensions: &[(&str, &str)]) -> Result<(String, Diagnostics)> {
    gen_code_with(&[(None, files)], extensions, FileFormat::Json, None)
}

/// Same as `gen_code`, but expects an error.
fn gen_err(files: &Files, extensions: &[(&str, &str)]) -> Error {
    match gen_code(files, extensions) {
        Ok(_) => panic!("expected an error for {files:?}"),
        Err(err) => err.into_inner(),
    }
}

// *********************************************
// * WARNINGS
// *********************************************

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

    assert!(has_surplus, "{:?}", &*diag.warnings());
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

    assert!(!has_surplus, "{:?}", &*diag.warnings());
}

// *********************************************
// * ERRORS
// *********************************************

#[test]
fn invalid_json() {
    let err = gen_err(&[("en", r#"{ "my_key": "#)], &[]);
    assert!(matches!(err, Error::LocaleFileDeser { .. }), "{err:?}");
}

#[test]
fn root_must_be_an_object() {
    let err = gen_err(&[("en", r#"["my_key"]"#)], &[]);
    assert!(matches!(err, Error::LocaleFileDeser { .. }), "{err:?}");
}

#[test]
fn keys_must_be_rust_identifiers() {
    for key in ["1key", "type", "self", "", "my key"] {
        let file = format!(r#"{{ "{key}": "value" }}"#);
        let err = gen_err(&[("en", &file)], &[]);
        assert!(
            matches!(err, Error::LocaleFileDeser { .. }),
            "key {key:?}: {err:?}"
        );
        assert!(
            err.to_string()
                .contains("can't be used as a rust identifier"),
            "key {key:?}: {err}"
        );
    }
}

#[test]
fn null_value_in_default_locale() {
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": null }"#)], &[]).unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::ExplicitDefaultInDefault(_))),
        "{:?}",
        &*diag.errors()
    )
}

#[test]
fn subkey_and_value_mismatch() {
    let (_code, diag) = gen_code(
        &[
            ("en", r#"{ "my_key": "value" }"#),
            ("fr", r#"{ "my_key": { "sub": "value" } }"#),
        ],
        &[],
    )
    .unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::SubKeyMissmatch { .. })),
        "{:?}",
        &*diag.errors()
    );
}

#[test]
fn unknown_formatter() {
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": "{{ var, nope }}" }"#)], &[]).unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::UnknownFormatter { .. })),
        "{:?}",
        &*diag.errors()
    );
}

#[test]
fn invalid_foreign_key() {
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": "$t(nope)" }"#)], &[]).unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::InvalidForeignKey { .. })),
        "{:?}",
        &*diag.errors()
    );
}

#[test]
fn recursive_foreign_keys() {
    let (_code, diag) = gen_code(&[("en", r#"{ "a": "$t(b)", "b": "$t(a)" }"#)], &[]).unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::RecursiveForeignKey { .. })),
        "{:?}",
        &*diag.errors()
    );

    let (_code, diag) = gen_code(&[("en", r#"{ "a": "$t(a)" }"#)], &[]).unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::RecursiveForeignKey { .. })),
        "{:?}",
        &*diag.errors()
    );
}

#[test]
fn invalid_locale_name() {
    let err = gen_err(&[("en", "{}"), ("not a locale", "{}")], &[]);
    assert!(matches!(err, Error::InvalidLocale { .. }), "{err:?}");
}

#[test]
fn inherit_from_unknown_locale() {
    let err = gen_err(&[("en", "{}"), ("fr", "{}")], &[("fr", "de")]);
    assert!(
        matches!(err, Error::UnknownLocaleInInherit { .. }),
        "{err:?}"
    );
}

#[test]
fn default_locale_cant_inherit() {
    let err = gen_err(&[("en", "{}"), ("fr", "{}")], &[("en", "fr")]);
    assert!(
        matches!(err, Error::DefaultLocaleCantInherit { .. }),
        "{err:?}"
    );
}

// *********************************************
// * KNOWN BUGS
// * see https://github.com/Baptistemontan/leptos_i18n/pull/327
// *********************************************

#[test]
#[ignore = "known bug: the builder ident uses the raw key name, see PR #327"]
fn kebab_case_key_with_interpolation() {
    // panics with `"my-key_builder" is not a valid Ident`
    gen_code(&[("en", r#"{ "my-key": "hello {{ count }}" }"#)], &[]).unwrap();
}

#[test]
#[ignore = "known bug: colliding keys emit duplicate definitions, see PR #327"]
fn keys_colliding_on_the_same_ident() {
    for file in [
        r#"{ "a-b": "dash", "a_b": "underscore" }"#,
        r#"{ "sub": { "a-b": "dash", "a_b": "underscore" } }"#,
    ] {
        let (code, _diag) = gen_code(&[("en", file)], &[]).unwrap();
        assert_eq!(code.matches("fn a_b (").count(), 1, "{file}");
        // Expected behavior: diag should contain some kind of "Duplicate key" error
    }
}

#[test]
#[ignore = "known bug: colliding variables and components emit duplicate fields"]
fn vars_and_comps_colliding_on_the_same_ident() {
    for files in [
        &[("en", r#"{ "k": "{{ a-b }} {{ a_b }}" }"#)][..],
        &[
            ("en", r#"{ "k": "{{ a-b }}" }"#),
            ("fr", r#"{ "k": "{{ a_b }}" }"#),
        ],
    ] {
        let (code, _diag) = gen_code(files, &[]).unwrap();
        assert_eq!(code.matches("var_a_b :").count(), 1, "{files:?}");
        // Expected behavior: a single variable, and diag should contain some kind of warning
        // that equivalent variables have different raw identifiers
    }

    for files in [
        &[("en", r#"{ "k": "<a-b>x</a-b> <a_b>y</a_b>" }"#)][..],
        &[("en", r#"{ "k": "<a-b/> <a_b/>" }"#)],
        &[
            ("en", r#"{ "k": "<a-b>x</a-b>" }"#),
            ("fr", r#"{ "k": "<a_b>y</a_b>" }"#),
        ],
    ] {
        let (code, _diag) = gen_code(files, &[]).unwrap();
        assert_eq!(code.matches("comp_a_b :").count(), 1, "{files:?}");
        // Expected behavior: a single component, and diag should contain some kind of warning
        // that equivalent components have different raw identifiers
    }
}

#[test]
#[ignore = "known bug: integer ranges with holes emit a non-exhaustive match, see PR #327"]
fn integer_ranges_with_holes() {
    let (code, _diag) = gen_code(
        &[("en", r#"{ "my_key": [["zero", 0], ["some", "1..=3"]] }"#)],
        &[],
    )
    .unwrap();
    assert!(code.contains("compile_error"));
}

#[test]
#[ignore = "known bug: non-finite floats panic in proc-macro2, see PR #327"]
fn non_finite_float_values() {
    for value in [".nan", ".inf", "-.inf"] {
        let file = format!("my_key: {value}");
        // panics with `assertion failed: f.is_finite()`
        gen_code_with(&[(None, &[("en", &file)])], &[], FileFormat::Yaml, None).unwrap();
    }
}

#[test]
#[ignore = "known bug: the namespaces `Deserialize` impl hardcodes `leptos_i18n`, see PR #327"]
fn namespaces_with_custom_crate_path() {
    let (code, _) = gen_code_with(
        &[
            (Some("first"), &[("en", r#"{ "a": "<b>{{ var }}</b>" }"#)]),
            (Some("second"), &[("en", r#"{ "b": "B" }"#)]),
        ],
        &[],
        FileFormat::Json,
        Some("my_crate"),
    )
    .unwrap();
    assert!(
        !code.contains("leptos_i18n ::"),
        "found a hardcoded `leptos_i18n` path"
    );
}
