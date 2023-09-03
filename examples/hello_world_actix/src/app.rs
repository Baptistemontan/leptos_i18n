use crate::i18n::{i18n_context, LocaleEnum, Locales};
use leptos::*;
use leptos_i18n::t;

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    leptos_i18n::provide_i18n_context::<Locales>();

    let i18n = i18n_context();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <h1>{t!(i18n, hello_world)}</h1>
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
    }
}
