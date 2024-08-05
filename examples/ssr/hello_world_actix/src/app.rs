use crate::i18n::*;
use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    provide_i18n_context();

    let i18n = use_i18n();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            Locale::en => Locale::fr,
            Locale::fr => Locale::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <h1>{t!(i18n, hello_world)}</h1>
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
    }
}
