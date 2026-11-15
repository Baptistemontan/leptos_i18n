//! Regression tests for the code generation.
//!
//! Each test parses some translations, generate the code, and inspects it.

use std::{borrow::Cow, mem};

use leptos_i18n_parser::parse_locales::{
    error::{Diagnostics, Error, Warning},
    files::{LocaleFiles, TranslationFiles},
    options::{Config, FileFormat, ParseOptions},
    parse_locales_with_files,
};

use leptos_i18n_parser::parse_locales::error::Result;

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
        .translations_uri("test")
        .parse_options(ParseOptions::default().file_format(file_format));
    if namespaces[0].0.is_some() {
        cfg = cfg.add_namespaces(namespaces.iter().map(|(ns, _)| ns.unwrap()))?;
    }

    for (l1, l2) in extensions {
        cfg = cfg.extend_locale(l1, l2)?;
    }
    let mut parsed_locales = parse_locales_with_files(&t_files, cfg)?;
    let diag = mem::take(&mut parsed_locales.diag);
    let crate_path = crate_path.map(|p| syn::parse_str::<syn::Path>(p).unwrap());
    let code =
        leptos_i18n_codegen::gen_code(&parsed_locales, crate_path.as_ref(), true, None, false)?;
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

#[test]
fn key_spelled_differently_across_locales() {
    for (en, fr, default_path, fr_path) in [
        (r#"{ "a-b": "en" }"#, r#"{ "a_b": "fr" }"#, "a-b", "a_b"),
        (r#"{ "a_b": "en" }"#, r#"{ "a-b": "fr" }"#, "a_b", "a-b"),
        (
            r#"{ "sub": { "a-b": "en" } }"#,
            r#"{ "sub": { "a_b": "fr" } }"#,
            "sub.a-b",
            "sub.a_b",
        ),
    ] {
        let (_code, diag) = gen_code(&[("en", en), ("fr", fr)], &[]).unwrap();
        assert!(diag.errors().is_empty(), "{:?}", &*diag.errors());
        let warnings = diag.warnings();
        assert!(
            matches!(
                &*warnings,
                [Warning::KeySpellingMismatch { key_path, provided_key }]
                    if key_path.to_string() == default_path
                        && provided_key.key_path.to_string() == fr_path
                        && provided_key.locale.name.as_ref() == "fr"
            ),
            "{en} / {fr}: {:?}",
            &*warnings
        );
    }

    // same spelling: no warning
    let (_code, diag) = gen_code(
        &[("en", r#"{ "a-b": "en" }"#), ("fr", r#"{ "a-b": "fr" }"#)],
        &[],
    )
    .unwrap();
    assert!(diag.warnings().is_empty(), "{:?}", &*diag.warnings());
}

#[test]
fn foreign_key_args_colliding_on_the_same_ident() {
    // the first argument is kept, the second is ignored
    let file = r#"{ "a": "{{ a-b }}", "b": "$t(a, {\"a-b\": \"first\", \"a_b\": \"second\"})" }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert!(diag.errors().is_empty(), "{:?}", &*diag.errors());
    let warnings = diag.warnings();
    assert!(
        matches!(
            &*warnings,
            [Warning::CollidingFkArgs { loc, fk_target, arg1, arg2, ident_repr }]
                if loc.key_path.to_string() == "b"
                    && fk_target.to_string() == "a"
                    && arg1 == "a-b"
                    && arg2 == "a_b"
                    && ident_repr == "a_b"
        ),
        "{:?}",
        &*warnings
    );
    assert!(code.contains("\"first\""));
    assert!(!code.contains("\"second\""));
}

#[test]
fn duplicate_foreign_key_args() {
    // the first argument is kept, the second is ignored
    let file =
        r#"{ "a": "{{ a }}", "sub": { "b": "$t(a, {\"a\": \"first\", \"a\": \"second\"})" } }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert!(diag.errors().is_empty(), "{:?}", &*diag.errors());
    let warnings = diag.warnings();
    assert!(
        matches!(
            &*warnings,
            [Warning::DuplicateFkArgs { loc, fk_target, arg_name }]
                if loc.key_path.to_string() == "sub.b"
                    && fk_target.to_string() == "a"
                    && arg_name == "a"
        ),
        "{:?}",
        &*warnings
    );
    assert!(code.contains("\"first\""));
    assert!(!code.contains("\"second\""));
}

#[test]
fn vars_colliding_on_the_same_ident() {
    // (files, locale where the second spelling is found)
    for (files, locale) in [
        (&[("en", r#"{ "k": "{{ a-b }} {{ a_b }}" }"#)][..], "en"),
        (&[("en", r#"{ "k": "<b>{{ a-b }}</b> {{ a_b }}" }"#)], "en"),
        (
            &[
                ("en", r#"{ "k": "{{ a-b }}" }"#),
                ("fr", r#"{ "k": "{{ a_b }}" }"#),
            ],
            "fr",
        ),
    ] {
        let (code, diag) = gen_code(files, &[]).unwrap();
        assert_eq!(code.matches("var_a_b :").count(), 1, "{files:?}");
        assert!(diag.errors().is_empty(), "{files:?}: {:?}", &*diag.errors());
        let warnings = diag.warnings();
        assert!(
            matches!(
                &*warnings,
                [Warning::VarKeySpellingMismatch { loc, .. }]
                    if loc.key_path.to_string() == "k" && loc.locale.name.as_ref() == locale
            ),
            "{files:?}: {:?}",
            &*warnings
        );
    }

    // same spelling: no warning
    let (_code, diag) = gen_code(&[("en", r#"{ "k": "{{ a-b }} {{ a-b }}" }"#)], &[]).unwrap();
    assert!(diag.warnings().is_empty(), "{:?}", &*diag.warnings());
}

#[test]
fn comps_colliding_on_the_same_ident() {
    // (files, locale where the second spelling is found, self-closed)
    for (files, locale, self_closed) in [
        (
            &[("en", r#"{ "k": "<a-b>x</a-b> <a_b>y</a_b>" }"#)][..],
            "en",
            false,
        ),
        (&[("en", r#"{ "k": "<a-b/> <a_b/>" }"#)], "en", true),
        (
            &[
                ("en", r#"{ "k": "<a-b>x</a-b>" }"#),
                ("fr", r#"{ "k": "<a_b>y</a_b>" }"#),
            ],
            "fr",
            false,
        ),
    ] {
        let (code, diag) = gen_code(files, &[]).unwrap();
        assert_eq!(code.matches("comp_a_b :").count(), 1, "{files:?}");
        assert!(diag.errors().is_empty(), "{files:?}: {:?}", &*diag.errors());
        let warnings = diag.warnings();
        assert!(
            matches!(
                &*warnings,
                [Warning::CompKeySpellingMismatch { loc, self_closed: sc, .. }]
                    if loc.key_path.to_string() == "k"
                        && loc.locale.name.as_ref() == locale
                        && *sc == self_closed
            ),
            "{files:?}: {:?}",
            &*warnings
        );
    }

    // same spelling: no warning
    let (_code, diag) = gen_code(&[("en", r#"{ "k": "<a-b/> <a-b/>" }"#)], &[]).unwrap();
    assert!(diag.warnings().is_empty(), "{:?}", &*diag.warnings());
}

#[test]
fn foreign_key_args_spelled_differently_than_the_variable() {
    for (var, arg) in [
        ("a-b", "a-b"),
        ("a_b", "a-b"),
        ("a-b", "a_b"),
        ("a_b", "a_b"),
    ] {
        let file =
            format!(r#"{{ "a": "{{{{ {var} }}}}", "b": "$t(a, {{\"{arg}\": \"value\"}})" }}"#);
        let (code, diag) = gen_code(&[("en", &file)], &[]).unwrap();
        assert!(diag.errors().is_empty(), "{file}: {:?}", &*diag.errors());
        assert!(
            diag.warnings().is_empty(),
            "{file}: {:?}",
            &*diag.warnings()
        );
        assert!(
            code.contains("\"value\""),
            "{file}: argument not substituted"
        );
    }
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
    let err = gen_err(&[("en", r#"{ "my_key": null }"#)], &[]);
    assert!(matches!(err, Error::ExplicitDefaultInDefault(_)), "{err:?}");
}

#[test]
fn subkey_and_value_mismatch() {
    let err = gen_err(
        &[
            ("en", r#"{ "my_key": "value" }"#),
            ("fr", r#"{ "my_key": { "sub": "value" } }"#),
        ],
        &[],
    );
    assert!(matches!(err, Error::SubKeyMissmatch { .. }), "{err:?}");
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
fn missing_foreign_key() {
    let err = gen_err(&[("en", r#"{ "my_key": "$t(nope)" }"#)], &[]);
    assert!(matches!(err, Error::MissingForeignKey { .. }), "{err:?}");
}

#[test]
fn recursive_foreign_keys() {
    let err = gen_err(&[("en", r#"{ "a": "$t(b)", "b": "$t(a)" }"#)], &[]);
    assert!(matches!(err, Error::RecursiveForeignKey { .. }), "{err:?}");

    let err = gen_err(&[("en", r#"{ "a": "$t(a)" }"#)], &[]);
    assert!(matches!(err, Error::RecursiveForeignKey { .. }), "{err:?}");
}

#[test]
fn invalid_locale_name() {
    let err = gen_err(&[("en", "{}"), ("not a locale", "{}")], &[]);
    assert!(matches!(err, Error::InvalidKey(_)), "{err:?}");
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

#[test]
fn keys_colliding_on_the_same_ident() {
    for file in [
        r#"{ "a-b": "dash", "a_b": "underscore" }"#,
        r#"{ "sub": { "a-b": "dash", "a_b": "underscore" } }"#,
    ] {
        let (_code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        assert!(
            diag.errors()
                .iter()
                .any(|e| matches!(e, Error::CollidingKeys { .. })),
            "{:?}",
            &*diag.errors()
        );
    }
}

#[test]
fn duplicate_key_in_file() {
    let (_code, diag) = gen_code(&[("en", r#"{ "a": "x", "a": "y" }"#)], &[]).unwrap();
    assert!(
        diag.errors()
            .iter()
            .any(|e| matches!(e, Error::CollidingKeys { .. })),
        "{:?}",
        &*diag.errors()
    );
}

#[test]
fn component_with_children_and_self_closed() {
    // (files, key path, locale where the conflict is found)
    for (files, path, locale) in [
        (&[("en", r#"{ "k": "<a>x</a> <a/>" }"#)][..], "k", "en"),
        (&[("en", r#"{ "k": "<a/> <a>x</a>" }"#)], "k", "en"),
        (&[("en", r#"{ "k": "<a-b>x</a-b> <a_b/>" }"#)], "k", "en"),
        (
            &[("en", r#"{ "sub": { "k": "<a>x</a> <a/>" } }"#)],
            "sub.k",
            "en",
        ),
        (
            &[
                ("en", r#"{ "k": "<a>x</a>" }"#),
                ("fr", r#"{ "k": "<a/>" }"#),
            ],
            "k",
            "fr",
        ),
    ] {
        let (_code, diag) = gen_code(files, &[]).unwrap();
        let errors = diag.errors();
        assert!(
            matches!(
                &*errors,
                [Error::ConflictingComponentKind { loc, .. }]
                    if loc.key_path.to_string() == path && loc.locale.name.as_ref() == locale
            ),
            "{files:?}: {:?}",
            &*errors
        );
    }
}

// *********************************************
// * PLURALS
// *********************************************

#[test]
#[cfg(feature = "plurals")]
fn plural_forms_spelled_differently() {
    // (file, colliding, spelling mismatch warning: (`other` key, key spelled differently))
    for (file, colliding, mismatch) in [
        (
            r#"{ "a-b_one": "1", "a_b_other": "o" }"#,
            false,
            Some(("a_b_other", "a-b_one")),
        ),
        // the duplicate `one` collides, the plural is still built whatever the order
        // (the first `one` in the file is kept)
        (
            r#"{ "a_b_one": "x", "a-b_one": "1", "a-b_other": "o" }"#,
            true,
            Some(("a-b_other", "a_b_one")),
        ),
        (
            r#"{ "a-b_one": "1", "a-b_other": "o", "a_b_one": "x" }"#,
            true,
            None,
        ),
    ] {
        let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        assert_eq!(code.matches("fn a_b (").count(), 1, "{file}");
        assert!(!code.contains("fn a_b_one"), "{file}");
        assert!(!code.contains("fn a_b_other"), "{file}");
        let collisions = diag
            .errors()
            .iter()
            .filter(|e| matches!(e, Error::CollidingKeys { .. }))
            .count();
        assert_eq!(
            collisions,
            usize::from(colliding),
            "{file}: {:?}",
            &*diag.errors()
        );
        let warnings = diag.warnings();
        match mismatch {
            Some((other_key, key)) => assert!(
                matches!(
                    &*warnings,
                    [Warning::PluralKeySpellingMismatch { key1, key2, .. }]
                        if key1.key_path.to_string() == other_key && key2.to_string() == key
                ),
                "{file}: {:?}",
                &*warnings
            ),
            None => assert!(warnings.is_empty(), "{file}: {:?}", &*warnings),
        }
    }
}

#[test]
fn plural_with_invalid_base_is_a_normal_key() {
    for (file, keys) in [
        (r#"{ "_one": "1", "_other": "o" }"#, ["_one", "_other"]),
        (
            r#"{ "_ordinal_one": "1", "_ordinal_other": "o" }"#,
            ["_ordinal_one", "_ordinal_other"],
        ),
        (
            r#"{ "type_one": "1", "type_other": "o" }"#,
            ["type_one", "type_other"],
        ),
    ] {
        let (code, _diag) = gen_code(&[("en", file)], &[]).unwrap();
        for key in keys {
            assert!(code.contains(&format!("fn {key} (")), "{file}: {key}");
        }
    }
}

#[test]
#[cfg(feature = "plurals")]
fn cardinal_and_ordinal_plurals_conflict() {
    for (file, path) in [
        (
            r#"{ "count_one": "c1", "count_other": "c", "count_ordinal_one": "o1", "count_ordinal_other": "o" }"#,
            "count",
        ),
        (
            r#"{ "sub": { "count_one": "c1", "count_other": "c", "count_ordinal_one": "o1", "count_ordinal_other": "o" } }"#,
            "sub.count",
        ),
    ] {
        let (_code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        let errors = diag.errors();
        assert!(
            matches!(&*errors, [Error::ConflictingPluralRuleType { loc }] if loc.key_path.to_string() == path),
            "{file}: {:?}",
            &*errors
        );
    }
}

#[test]
#[cfg(feature = "plurals")]
fn cardinal_plural_and_single_ordinal_form() {
    // the lone ordinal form is a normal key, like any single-form group
    let file = r#"{ "count_one": "c1", "count_other": "c", "count_ordinal_other": "o" }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert!(diag.errors().is_empty(), "{:?}", &*diag.errors());
    assert_eq!(code.matches("fn count (").count(), 1);
    assert_eq!(code.matches("fn count_ordinal_other (").count(), 1);
}

#[test]
#[cfg(feature = "plurals")]
fn plurals_at_normal_key() {
    for (file, path) in [
        (
            r#"{ "count": "n", "count_one": "c1", "count_other": "c" }"#,
            "count",
        ),
        (
            r#"{ "sub": { "count": "n", "count_one": "c1", "count_other": "c" } }"#,
            "sub.count",
        ),
    ] {
        let (_code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        let errors = diag.errors();
        assert!(
            matches!(&*errors, [Error::PluralsAtNormalKey { loc }] if loc.key_path.to_string() == path),
            "{file}: {:?}",
            &*errors
        );
    }
}

// *********************************************
// * KNOWN BUGS
// * see https://github.com/Baptistemontan/leptos_i18n/pull/327
// *********************************************

#[test]
fn kebab_case_key_with_interpolation() {
    // panics with `"my-key_builder" is not a valid Ident`
    gen_code(&[("en", r#"{ "my-key": "hello {{ count }}" }"#)], &[]).unwrap();
}

#[test]
fn integer_ranges_with_holes() {
    let (code, _diag) = gen_code(
        &[("en", r#"{ "my_key": [["zero", 0], ["some", "1..=3"]] }"#)],
        &[],
    )
    .unwrap();
    assert!(code.contains("compile_error"));
}

#[test]
fn non_finite_float_values() {
    for value in [".nan", ".inf", "-.inf"] {
        let file = format!("my_key: {value}");
        // panics with `assertion failed: f.is_finite()`
        gen_code_with(&[(None, &[("en", &file)])], &[], FileFormat::Yaml, None).unwrap();
    }
}

#[test]
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

// *********************************************
// * SILENT OVERWRITES AND PANICS
// * see TODO.md
// *********************************************

/// The translation files don't match the config: parsing must return an error.
fn assert_files_error(files: &TranslationFiles, cfg: Config) {
    let cfg = cfg.translations_uri("test");
    assert!(parse_locales_with_files(files, cfg).is_err());
}

#[test]
#[ignore = "known bug: panics with `todo!()` instead of reporting an error"]
fn missing_locale_file() {
    let files = new_files(&[(None, &[("en", "{}")])]);
    let cfg = Config::new("en").unwrap().add_locale("fr").unwrap();
    assert_files_error(&files, cfg);
}

#[test]
#[ignore = "known bug: panics with `todo!()` instead of reporting an error"]
fn missing_namespace_files() {
    let files = new_files(&[(Some("first"), &[("en", "{}")])]);
    let cfg = Config::new("en")
        .unwrap()
        .add_namespaces(["first", "second"])
        .unwrap();
    assert_files_error(&files, cfg);
}

#[test]
#[ignore = "known bug: panics with `todo!()` instead of reporting an error"]
fn files_per_locale_with_namespaces_in_config() {
    let files = new_files(&[(None, &[("en", "{}")])]);
    let cfg = Config::new("en").unwrap().add_namespace("first").unwrap();
    assert_files_error(&files, cfg);
}

#[test]
#[ignore = "known bug: panics with `todo!()` instead of reporting an error"]
fn files_per_namespace_without_namespaces_in_config() {
    let files = new_files(&[(Some("first"), &[("en", "{}")])]);
    let cfg = Config::new("en").unwrap();
    assert_files_error(&files, cfg);
}
