use std::sync::Arc;

use crate::{Locale, LocaleInner};

pub const COOKIE_PREFERED_LANG: &str = "i18n_pref_local";

pub struct I18nConfig {
    pub default_locale: Locale,
    pub locales: Vec<Locale>,
}

impl I18nConfig {
    pub fn get_local(&self, lang: &str) -> Option<&Locale> {
        self.locales.iter().find(|loc| loc.lang == lang)
    }
}

pub fn load_i18n_config() -> actix_web::web::Data<I18nConfig> {
    use std::fs::File;

    #[derive(serde::Deserialize)]
    struct Config {
        default: String,
        locales: Vec<String>,
    }

    let config_file = File::open("./i18n.json").expect("i18n config file not present");
    let Config { default, locales } =
        serde_json::from_reader(config_file).expect("Error parsing the i18n config file");

    let mut loaded_locales = Vec::with_capacity(locales.len());

    let mut default_loc = None;

    for locale in locales {
        let file = match File::open(format!("./locales/{}.json", locale)) {
            Ok(file) => file,
            Err(err) => panic!("Unable to access file for locale {:?} : {}", locale, err),
        };
        let translations = match serde_json::from_reader(file) {
            Ok(val) => val,
            Err(err) => panic!("Error parsing the file for locale {:?} : {}", locale, err),
        };

        let locale = Locale(Arc::new(LocaleInner {
            lang: locale,
            translations,
        }));

        if locale.lang == default {
            default_loc.replace(locale.clone());
        }

        loaded_locales.push(locale);
    }

    let config = I18nConfig {
        default_locale: default_loc.expect("Default locale not found."),
        locales: loaded_locales,
    };
    actix_web::web::Data::new(config)
}

#[derive(serde::Deserialize)]
struct SetLocaleParams {
    lang: String,
    origin: String,
}

#[actix_web::get("/api/locale/set")]
async fn set_locale_api(
    params: actix_web::web::Query<SetLocaleParams>,
) -> impl actix_web::Responder {
    use actix_web::{cookie::*, http::header};
    let params = params.into_inner();
    let cookie = CookieBuilder::new(COOKIE_PREFERED_LANG, params.lang)
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::MAX)
        .path("/")
        .finish()
        .encoded()
        .to_string();

    let mut res = actix_web::HttpResponse::Found();
    res.append_header((header::SET_COOKIE, cookie));
    res.append_header((header::LOCATION, params.origin));

    res.finish()
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.app_data(load_i18n_config()).service(set_locale_api);
}
