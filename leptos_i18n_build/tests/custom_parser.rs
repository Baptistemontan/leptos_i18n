//! A custom file format, implemented with the public API only (`leptos_i18n_build::options::parser`).

use leptos_i18n_build::{
    Config, LocaleFiles, TranslationFiles, TranslationsInfos,
    options::{
        ParseOptions,
        parser::{Parser, Seed, SerdeError, Value},
    },
};
use serde::de::DeserializeSeed;
use std::{borrow::Cow, path::Path};

/// JSON, read through the custom parser API. Errors are prefixed to check that this parser is the one used.
struct CustomJson;

impl Parser for CustomJson {
    fn deserialize(
        &self,
        file: &str,
        _path: Option<&Path>,
        seed: Seed,
    ) -> Result<Value, SerdeError> {
        let mut deserializer = serde_json::Deserializer::from_str(file);
        seed.deserialize(&mut deserializer)
            .map_err(|err| SerdeError::custom(format!("custom parser: {err}")))
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["cjson"]
    }
}

fn files(file: &str) -> TranslationFiles<'_> {
    TranslationFiles::Locales {
        locales: LocaleFiles {
            files: [("en", (None, Cow::Borrowed(file)))].into_iter().collect(),
        },
    }
}

fn config() -> Config {
    Config::new("en")
        .unwrap()
        .parse_options(ParseOptions::default().with_custom_parser(CustomJson))
}

#[test]
fn custom_parser_is_used() {
    let files = files(r#"{ "hello": "Hello", "sub": { "count": "{{ count }}" } }"#);
    let infos = TranslationsInfos::parse_with_files_as_string(config(), &files).unwrap();
    assert_eq!(infos.get_locales().collect::<Vec<_>>(), ["en".into()]);
}

#[test]
fn custom_parser_errors_are_reported() {
    let files = files(r#"{ "hello": "#);
    let err = TranslationsInfos::parse_with_files_as_string(config(), &files)
        .err()
        .expect("expected an error");
    assert!(err.to_string().contains("custom parser: "), "{err}");
}
