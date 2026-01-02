use crate::i18n::*;
use leptos::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    view! {
        <I18nContextProvider>
            <Component />
        </I18nContextProvider>
    }
}

#[component]
#[allow(non_snake_case)]
pub fn Component() -> impl IntoView {
    let i18n = use_i18n();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            Locale::en => Locale::fr,
            Locale::fr => Locale::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <div>
            <span style="white-space: pre-wrap">
                {t!(i18n, custom_formatter, var = || "test")}
            </span>
            <br/>
            <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
        </div>
    }
}
