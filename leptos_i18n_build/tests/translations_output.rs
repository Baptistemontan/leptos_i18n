use leptos_i18n_build::{Config, TranslationsInfos};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn out_dir(test_name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
    }
    path
}

fn parse_fixtures() -> TranslationsInfos {
    let cfg = Config::new("en").unwrap();
    TranslationsInfos::parse_at_dir(fixtures_dir(), cfg).unwrap()
}

/// The files written by `write_to_dir` are fetched and decoded as JSON by the
/// `dynamic_load` client, so locale files that are valid JSON have to round
/// trip into files that are valid JSON too.
#[test]
fn written_translations_are_valid_json() {
    let out = out_dir("written_translations_are_valid_json");

    parse_fixtures()
        .get_translations()
        .write_to_dir(&out)
        .unwrap();

    let written = std::fs::read_to_string(out.join("en.json")).unwrap();
    let strings: Vec<String> = serde_json::from_str(&written)
        .unwrap_or_else(|err| panic!("`{written}` is not valid JSON: {err}"));

    // The strings must survive the round trip untouched.
    assert!(strings.iter().any(|s| s.contains('\u{7}')));
    assert!(strings.iter().any(|s| s.contains('\u{7f}')));
    assert!(strings.iter().any(|s| s.contains('\u{301}')));
}
