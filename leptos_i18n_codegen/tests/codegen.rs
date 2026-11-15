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

    let warnings = diag.warnings();
    assert!(
        matches!(
            &*warnings,
            [Warning::MissingKey { loc }]
                if loc.key_path.to_string() == "my_key" && loc.locale.key.name.as_ref() == "fr"
        ),
        "{:?}",
        &*warnings
    );
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

    let warnings = diag.warnings();
    assert!(
        matches!(
            &*warnings,
            [Warning::SurplusKey { loc }]
                if loc.key_path.to_string() == "surplus" && loc.locale.key.name.as_ref() == "fr"
        ),
        "{:?}",
        &*warnings
    );
}

#[test]
fn suppress_missing_key_on_inherit() {
    let (_code, diag) = gen_code(
        &[("en", r#"{ "my_key": "" }"#), ("fr", r"{}")],
        &[("fr", "en")],
    )
    .unwrap();

    assert!(diag.warnings().is_empty(), "{:?}", &*diag.warnings());
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
                        && provided_key.locale.key.name.as_ref() == "fr"
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
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
                    if loc.key_path.to_string() == "k" && loc.locale.key.name.as_ref() == locale
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
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
                        && loc.locale.key.name.as_ref() == locale
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
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
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": null }"#)], &[]).unwrap();
    let errors = diag.errors();
    assert!(
        matches!(&*errors, [Error::ExplicitDefaultInDefault(_)]),
        "{:?}",
        &*errors
    );
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
    let errors = diag.errors();
    assert!(
        matches!(&*errors, [Error::SubKeyMissmatch { .. }]),
        "{:?}",
        &*errors
    );
}

#[test]
fn unknown_formatter() {
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": "{{ var, nope }}" }"#)], &[]).unwrap();
    let errors = diag.errors();
    assert!(
        matches!(
            &*errors,
            [Error::UnknownFormatter { name, loc }]
                if name == "nope" && loc.key_path.to_string() == "my_key"
        ),
        "{:?}",
        &*errors
    );
}

#[test]
fn invalid_foreign_key() {
    let (_code, diag) = gen_code(&[("en", r#"{ "my_key": "$t(nope)" }"#)], &[]).unwrap();
    let errors = diag.errors();
    assert!(
        matches!(&*errors, [Error::InvalidForeignKey { .. }]),
        "{:?}",
        &*errors
    );
}

#[test]
fn recursive_foreign_keys() {
    for file in [r#"{ "a": "$t(b)", "b": "$t(a)" }"#, r#"{ "a": "$t(a)" }"#] {
        let (_code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        let errors = diag.errors();
        assert!(
            !errors.is_empty()
                && errors
                    .iter()
                    .all(|e| matches!(e, Error::RecursiveForeignKey { .. })),
            "{file}: {:?}",
            &*errors
        );
    }
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

#[test]
fn keys_colliding_on_the_same_ident() {
    // (file, path of the colliding key, path of the key already there)
    for (file, path1, path2) in [
        (r#"{ "a-b": "dash", "a_b": "underscore" }"#, "a_b", "a-b"),
        (
            r#"{ "sub": { "a-b": "dash", "a_b": "underscore" } }"#,
            "sub.a_b",
            "sub.a-b",
        ),
    ] {
        let (_code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        let errors = diag.errors();
        assert!(
            matches!(
                &*errors,
                [Error::CollidingKeys { loc1, loc2 }]
                    if loc1.key_path.to_string() == path1 && loc2.key_path.to_string() == path2
            ),
            "{file}: {:?}",
            &*errors
        );
    }
}

#[test]
fn duplicate_key_in_file() {
    let (_code, diag) = gen_code(&[("en", r#"{ "a": "x", "a": "y" }"#)], &[]).unwrap();
    let errors = diag.errors();
    assert!(
        matches!(
            &*errors,
            [Error::CollidingKeys { loc1, loc2 }]
                if loc1.key_path.to_string() == "a" && loc2.key_path.to_string() == "a"
        ),
        "{:?}",
        &*errors
    );
}

#[test]
fn component_with_children_and_self_closed() {
    // (files, key path)
    for (files, path) in [
        (&[("en", r#"{ "k": "<a>x</a> <a/>" }"#)][..], "k"),
        (&[("en", r#"{ "k": "<a/> <a>x</a>" }"#)], "k"),
        (&[("en", r#"{ "k": "<a-b>x</a-b> <a_b/>" }"#)], "k"),
        (&[("en", r#"{ "sub": { "k": "<a>x</a> <a/>" } }"#)], "sub.k"),
        (
            &[
                ("en", r#"{ "k": "<a>x</a>" }"#),
                ("fr", r#"{ "k": "<a/>" }"#),
            ],
            "k",
        ),
    ] {
        let (_code, diag) = gen_code(files, &[]).unwrap();
        let errors = diag.errors();
        assert!(
            matches!(
                &*errors,
                [Error::ComponentSelfClosedMissmatch { path: p, .. }] if p.to_string() == path
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
fn plural_forms_spelled_differently() {
    /// A single plural `a_b`, no normal keys for its forms.
    fn assert_plural(file: &str, code: &str) {
        assert_eq!(code.matches("fn a_b (").count(), 1, "{file}");
        assert!(!code.contains("fn a_b_one"), "{file}");
        assert!(!code.contains("fn a_b_other"), "{file}");
    }

    // spelled differently: the plural is built, with a warning
    let file = r#"{ "a-b_one": "1", "a_b_other": "o" }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert_plural(file, &code);
    assert!(diag.errors().is_empty(), "{:?}", &*diag.errors());
    let warnings = diag.warnings();
    assert!(
        matches!(
            &*warnings,
            [Warning::PluralKeySpellingMismatch { key1, key2, .. }]
                if key1.key_path.to_string() == "a_b_other" && key2.to_string() == "a-b_one"
        ),
        "{:?}",
        &*warnings
    );

    // a duplicate `one`: it collides, the first one in the file is kept,
    // and it's spelled differently than `other`
    let file = r#"{ "a_b_one": "x", "a-b_one": "1", "a-b_other": "o" }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert_plural(file, &code);
    let errors = diag.errors();
    assert!(
        matches!(
            &*errors,
            [Error::CollidingKeys { loc1, loc2 }]
                if loc1.key_path.to_string() == "a-b_one" && loc2.key_path.to_string() == "a_b_one"
        ),
        "{:?}",
        &*errors
    );
    let warnings = diag.warnings();
    assert!(
        matches!(
            &*warnings,
            [Warning::PluralKeySpellingMismatch { key1, key2, .. }]
                if key1.key_path.to_string() == "a-b_other" && key2.to_string() == "a_b_one"
        ),
        "{:?}",
        &*warnings
    );

    // a duplicate `one` after the others: it collides, the kept `one` has the same spelling as `other`
    let file = r#"{ "a-b_one": "1", "a-b_other": "o", "a_b_one": "x" }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert_plural(file, &code);
    let errors = diag.errors();
    assert!(
        matches!(
            &*errors,
            [Error::CollidingKeys { loc1, loc2 }]
                if loc1.key_path.to_string() == "a_b_one" && loc2.key_path.to_string() == "a-b_one"
        ),
        "{:?}",
        &*errors
    );
    assert!(diag.warnings().is_empty(), "{:?}", &*diag.warnings());
}

#[test]
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
fn plural_with_invalid_base_is_a_normal_key() {
    for (file, keys, expected_base) in [
        (r#"{ "_one": "1", "_other": "o" }"#, ["_one", "_other"], ""),
        (
            r#"{ "_ordinal_one": "1", "_ordinal_other": "o" }"#,
            ["_ordinal_one", "_ordinal_other"],
            "",
        ),
        (
            r#"{ "type_one": "1", "type_other": "o" }"#,
            ["type_one", "type_other"],
            "type",
        ),
    ] {
        let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        for key in keys {
            assert!(code.contains(&format!("fn {key} (")), "{file}: {key}");
        }
        assert!(diag.errors().is_empty(), "{file}: {:?}", &*diag.errors());

        // one warning per key
        let warnings = diag.warnings();
        let mut warned_keys = warnings
            .iter()
            .map(|w| match w {
                Warning::PluralLookingKeyInvalidBase { loc, base } if &**base == expected_base => {
                    loc.key_path.to_string()
                }
                other => panic!("{file}: unexpected warning {other:?}"),
            })
            .collect::<Vec<_>>();
        warned_keys.sort();
        assert_eq!(warned_keys, keys, "{file}");
    }
}

#[test]
#[cfg_attr(not(feature = "plurals"), ignore = "requires the `plurals` feature")]
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
#[ignore = "not ported to the redesign yet, see TODO.md (15)"]
fn cardinal_plural_and_single_ordinal_form() {
    // the lone ordinal form is a normal key, like any single-form group
    let file = r#"{ "count_one": "c1", "count_other": "c", "count_ordinal_other": "o" }"#;
    let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
    assert!(diag.errors().is_empty(), "{:?}", &*diag.errors());
    assert_eq!(code.matches("fn count (").count(), 1);
    assert_eq!(code.matches("fn count_ordinal_other (").count(), 1);
}

#[test]
#[cfg_attr(not(feature = "plurals"), ignore = "requires the `plurals` feature")]
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

#[test]
#[ignore = "not ported to the redesign yet, see TODO.md (20)"]
fn plural_looking_keys_with_no_other() {
    use leptos_i18n_parser::extraction::PluralRuleType;

    // (file, base key path, keys, rule type, normal keys generated)
    for (file, path, keys, rule_type, fns) in [
        (
            r#"{ "c_one": "1", "c_few": "f" }"#,
            "c",
            &["c_one", "c_few"][..],
            PluralRuleType::Cardinal,
            &["c_one", "c_few"][..],
        ),
        (
            r#"{ "sub": { "c_ordinal_one": "1", "c_ordinal_two": "t" } }"#,
            "sub.c",
            &["c_ordinal_one", "c_ordinal_two"],
            PluralRuleType::Ordinal,
            &["c_ordinal_one", "c_ordinal_two"],
        ),
        // the cardinal plural is fine, only the ordinal forms are reported
        (
            r#"{ "c_one": "1", "c_other": "o", "c_ordinal_one": "1", "c_ordinal_few": "f" }"#,
            "c",
            &["c_ordinal_one", "c_ordinal_few"],
            PluralRuleType::Ordinal,
            &["c", "c_ordinal_one", "c_ordinal_few"],
        ),
    ] {
        let (code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        for f in fns {
            assert_eq!(code.matches(&format!("fn {f} (")).count(), 1, "{file}: {f}");
        }
        assert!(diag.errors().is_empty(), "{file}: {:?}", &*diag.errors());
        let warnings = diag.warnings();
        assert!(
            matches!(
                &*warnings,
                [Warning::PluralLookingKeysWithNoOther { base_key_location, keys: warned_keys, rule_type: rt }]
                    if base_key_location.key_path.to_string() == path
                        && warned_keys.iter().map(|k| &*k.name).eq(keys.iter().copied())
                        && *rt == rule_type
            ),
            "{file}: {:?}",
            &*warnings
        );
    }

    // no warning: a valid plural, a single form, or a single form of each rule type
    for file in [
        r#"{ "c_one": "1", "c_other": "o" }"#,
        r#"{ "c_ordinal_one": "1", "c_ordinal_other": "o" }"#,
        r#"{ "c_one": "1" }"#,
        r#"{ "c_one": "1", "c_ordinal_one": "1" }"#,
    ] {
        let (_code, diag) = gen_code(&[("en", file)], &[]).unwrap();
        assert!(
            diag.warnings().is_empty(),
            "{file}: {:?}",
            &*diag.warnings()
        );
    }
}

// *********************************************
// * REGRESSIONS
// * fixed bugs, see https://github.com/Baptistemontan/leptos_i18n/pull/327
// *********************************************

#[test]
fn kebab_case_key_with_interpolation() {
    // used to panic with `"my-key_builder" is not a valid Ident`
    gen_code(&[("en", r#"{ "my-key": "hello {{ count }}" }"#)], &[]).unwrap();
}

#[test]
#[ignore = "known bug: non-finite floats panic in proc-macro2, see PR #327"]
fn non_finite_float_values() {
    for value in [".nan", ".inf", "-.inf"] {
        let file = format!("my_key: {value}");
        // used to panic with `assertion failed: f.is_finite()`
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
// * TRANSLATION FILES NOT MATCHING THE CONFIG
// *********************************************

/// The translation files don't match the config: parsing must return an error.
fn files_error(files: &TranslationFiles, cfg: Config) -> Error {
    let result = parse_locales_raw_with_files(files, &cfg)
        .and_then(|raw| extract_locales(raw.values, &cfg, Some(raw.diag)));
    match result {
        Ok(_) => panic!("expected an error"),
        Err(err) => err.into_inner(),
    }
}

#[test]
fn missing_locale_file() {
    let files = new_files(&[(None, &[("en", "{}")])]);
    let cfg = Config::new("en").unwrap().add_locale("fr").unwrap();
    let err = files_error(&files, cfg);
    assert!(
        matches!(&err, Error::MissingLocaleFile { locale, namespace: None } if locale.name.as_ref() == "fr"),
        "{err:?}"
    );

    // in a namespace
    let files = new_files(&[(Some("first"), &[("en", "{}")])]);
    let cfg = Config::new("en")
        .unwrap()
        .add_locale("fr")
        .unwrap()
        .add_namespace("first")
        .unwrap();
    let err = files_error(&files, cfg);
    assert!(
        matches!(
            &err,
            Error::MissingLocaleFile { locale, namespace: Some(namespace) }
                if locale.name.as_ref() == "fr" && namespace.name.as_ref() == "first"
        ),
        "{err:?}"
    );
}

#[test]
fn missing_namespace_files() {
    let files = new_files(&[(Some("first"), &[("en", "{}")])]);
    let cfg = Config::new("en")
        .unwrap()
        .add_namespaces(["first", "second"])
        .unwrap();
    let err = files_error(&files, cfg);
    assert!(
        matches!(&err, Error::MissingNamespaceFiles(namespace) if namespace.name.as_ref() == "second"),
        "{err:?}"
    );
}

#[test]
fn files_per_locale_with_namespaces_in_config() {
    let files = new_files(&[(None, &[("en", "{}")])]);
    let cfg = Config::new("en").unwrap().add_namespace("first").unwrap();
    let err = files_error(&files, cfg);
    assert!(
        matches!(
            err,
            Error::TranslationFilesMismatch {
                expected_namespaces: true
            }
        ),
        "{err:?}"
    );
}

#[test]
fn files_per_namespace_without_namespaces_in_config() {
    let files = new_files(&[(Some("first"), &[("en", "{}")])]);
    let cfg = Config::new("en").unwrap();
    let err = files_error(&files, cfg);
    assert!(
        matches!(
            err,
            Error::TranslationFilesMismatch {
                expected_namespaces: false
            }
        ),
        "{err:?}"
    );
}
