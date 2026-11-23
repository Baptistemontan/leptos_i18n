use leptos_i18n_build::{Config, LocaleFiles, TranslationFiles, TranslationsInfos};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

const DATA: &str = include_str!("./data/en.json");

fn tmp_dir(test_name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
    }
    path
}

/// `(locale, file content)` pairs.
type Files<'a> = [(&'a str, &'a str)];

fn parse_fixtures(files: &Files) -> TranslationsInfos {
    let cfg = Config::new(files[0].0)
        .unwrap()
        .add_locales(files.iter().skip(1).map(|l| l.0))
        .unwrap();
    let files = TranslationFiles::Locales {
        locales: LocaleFiles {
            files: files
                .iter()
                .map(|(locale, file)| (*locale, (None, Cow::Borrowed(*file))))
                .collect(),
        },
    };
    TranslationsInfos::parse_with_files_as_string(cfg, &files).unwrap()
}

/// The files written by `write_to_dir` are fetched and decoded as JSON by the
/// `dynamic_load` client, so locale files that are valid JSON have to round
/// trip into files that are valid JSON too.
#[test]
fn written_translations_are_valid_json() {
    let parsed = parse_fixtures(&[("en", DATA)]);
    let strings = parsed
        .get_translations()
        .to_strings()
        .into_locales()
        .unwrap();

    let written = strings.get("en").unwrap();

    let strings: Vec<String> = serde_json::from_str(written)
        .unwrap_or_else(|err| panic!("`{written}` is not valid JSON: {err}"));

    // The strings must survive the round trip untouched.
    assert!(strings.iter().any(|s| s.contains('\u{7}')));
    assert!(strings.iter().any(|s| s.contains('\u{7f}')));
    assert!(strings.iter().any(|s| s.contains('\u{301}')));
}

/// Datagen returns `Result`, so an unusable output path has to surface as an
/// error rather than take the whole build script down with a panic.
#[test]
fn generate_data_reports_io_errors() {
    let tmp_dir = tmp_dir("generate_data_reports_io_errors");
    std::fs::create_dir_all(&tmp_dir).unwrap();

    // A file where a directory is expected: clearing the output path fails.
    let mod_directory = tmp_dir.join("baked_data");
    std::fs::write(&mod_directory, "not a directory").unwrap();

    let err = parse_fixtures(&[("en", DATA)])
        .generate_data(mod_directory)
        .expect_err("generating into a path that cannot be cleared should fail");

    assert!(
        matches!(err.kind, icu_provider::DataErrorKind::Io(_)),
        "expected an IO error, got {err:?}"
    );
}
