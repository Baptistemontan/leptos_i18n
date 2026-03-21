use crate::i18n::*;
use leptos::prelude::*;
use leptos_i18n::translation_map_builder;

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    view! {
        <I18nContextProvider>
            <Inner />
        </I18nContextProvider>
    }
}

#[component]
#[allow(non_snake_case)]
fn Inner() -> impl IntoView {
    let i18n = use_i18n();

    let on_change = move |ev: leptos::ev::Event| {
        let locale = match event_target_value(&ev).as_str() {
            "fr" => Locale::fr,
            "zh" => Locale::zh,
            "es" => Locale::es,
            _ => Locale::en,
        };
        i18n.set_locale(locale);
    };

    // Fetch "hello world" from a server we don't control.
    let response = LocalResource::new(|| async {
        gloo_net::http::Request::get("https://httpbin.org/base64/aGVsbG8gd29ybGQ=")
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()
    });

    // Map English string values to the current locale. Recomputes on locale change.
    let lookup = Memo::new(move |_| {
        let en = include_str!("../locales/en.json");
        let target = match i18n.get_locale() {
            Locale::en => en,
            Locale::fr => include_str!("../locales/fr.json"),
            Locale::zh => include_str!("../locales/zh.json"),
            Locale::es => include_str!("../locales/es.json"),
        };
        translation_map_builder(en, target).expect("locale JSON is valid")
    });

    view! {
        <label>{t!(i18n, select_lang)}": "</label>
        <select on:change=on_change>
            <option value="en">"English"</option>
            <option value="fr">"Français"</option>
            <option value="zh">"中文"</option>
            <option value="es">"Español"</option>
        </select>
        <p>{t!(i18n, server_says)}</p>
        <p>{move || {
            response.read().as_ref().map(|r| match r {
                Some(text) => lookup.read().get(text).cloned().unwrap_or_else(|| text.clone()),
                None => "Failed to fetch".to_string(),
            })
        }}</p>
    }
}
