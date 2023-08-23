#[cfg(all(feature = "actix", not(feature = "axum")))]
mod actix;

#[cfg(all(feature = "axum", not(feature = "actix")))]
mod axum;

use leptos::*;

use crate::Locales;

#[cfg(all(feature = "actix", not(feature = "axum")))]
use actix as backend;
#[cfg(all(feature = "axum", not(feature = "actix")))]
use axum as backend;

#[cfg(any(feature = "actix", feature = "axum"))]
pub fn fetch_locale_server_side<T: Locales>(cx: Scope) -> T::Variants {
    backend::fetch_locale_server::<T>(cx)
}

#[cfg(all(feature = "actix", feature = "axum"))]
compile_error!("Can't enable \"actix\" and \"axum\" features together.");

#[cfg(not(any(feature = "actix", feature = "axum")))]
pub fn fetch_locale_server_side<T: Locales>(cx: Scope) -> T::Variants {
    compile_error!("Need either \"actix\" or \"axum\" feature to be enabled in ssr. Don't use the \"ssr\" feature, it is directly enable by the \"actix\" or \"axum\" feature.")
}

#[cfg(any(feature = "actix", feature = "axum"))]
pub(crate) fn parse_header(header: &str) -> Vec<String> {
    let mut parsed_lang: Vec<_> = header
        .split(';')
        .map(|lang| {
            let mut langs = lang.split(',').peekable();
            let q = if let Some(a) = langs
                .peek()
                .and_then(|maybe_q| maybe_q.trim().strip_prefix("q="))
            {
                let q = a.parse::<f32>().unwrap_or(1.0);
                langs.next();
                q
            } else {
                1.0
            };
            (q, langs)
        })
        .collect();

    parsed_lang.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));

    parsed_lang
        .into_iter()
        .flat_map(|(_q, langs)| langs.map(str::trim).map(String::from))
        .collect()
}

#[cfg(all(test, any(feature = "actix", feature = "axum")))]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let parsed_lang = parse_header("fr-CH, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5");

        assert_eq!(parsed_lang, &["fr-CH", "fr", "en", "de", "*"]);
    }

    #[test]
    fn test_parse_unsorted() {
        let parsed_lang = parse_header("q=0.3, fr-CH, fr;q=0.9, en;de;q=0.7, *;q=0.5");

        assert_eq!(parsed_lang, &["de", "en", "*", "fr-CH", "fr"]);
    }
}
