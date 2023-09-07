use crate::i18n::*;
use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    let i18n = provide_i18n_context();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
        <Subkeys />
    }
}

#[component]
fn Subkeys() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <p>{t!(i18n, sub_keys.sub_key)}</p>
        <p>{t!(i18n, sub_keys.sub_sub_keys.sub_sub_key)}</p>
    }
}
