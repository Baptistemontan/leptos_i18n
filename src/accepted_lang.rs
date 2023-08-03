use std::{
    fmt::Display,
    future::{ready, Ready},
};

use actix_web::{http::header, FromRequest, ResponseError};

use crate::server::COOKIE_PREFERED_LANG;

pub struct AcceptedLang {
    langs: Vec<String>,
    preffered_lang: Option<String>,
}

impl AcceptedLang {
    pub fn find_first_lang<'a, T: AsRef<str>>(&self, locales: &'a [T]) -> Option<&'a T> {
        for lang in self.preffered_lang.as_ref().into_iter().chain(&self.langs) {
            if let Some(lang) = locales.iter().find(|l| l.as_ref() == lang) {
                return Some(lang);
            }
        }
        None
    }

    fn parse_header(header: &str) -> Vec<String> {
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
}

impl FromRequest for AcceptedLang {
    type Error = Impossible;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let preffered_lang = req
            .cookie(COOKIE_PREFERED_LANG)
            .map(|ck| ck.value().to_string());

        let Some(header) = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|header| header.to_str().ok())
        else {
            return ready(Ok(AcceptedLang {
                langs: Vec::new(),
                preffered_lang,
            }));
        };

        let langs = Self::parse_header(header);

        ready(Ok(AcceptedLang {
            langs,
            preffered_lang,
        }))
    }
}

#[derive(Debug)]
pub enum Impossible {}

impl Display for Impossible {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl ResponseError for Impossible {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let parsed_lang =
            AcceptedLang::parse_header("fr-CH, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5");

        assert_eq!(parsed_lang, &["fr-CH", "fr", "en", "de", "*"]);
    }

    #[test]
    fn test_parse_unsorted() {
        let parsed_lang =
            AcceptedLang::parse_header("q=0.3, fr-CH, fr;q=0.9, en;de;q=0.7, *;q=0.5");

        assert_eq!(parsed_lang, &["de", "en", "*", "fr-CH", "fr"]);
    }
}
