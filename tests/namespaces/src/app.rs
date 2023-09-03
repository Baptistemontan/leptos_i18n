use crate::i18n::{i18n_context, LocaleEnum, Locales};
use leptos::*;
use leptos_i18n::t;

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    let i18n = leptos_i18n::provide_i18n_context::<Locales>();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <button on:click=on_switch>{t!(i18n, first_namespace.click_to_change_lang)}</button>
        <Tests />
    }
}

#[component]
fn Counter() -> impl IntoView {
    let i18n = i18n_context();

    let (counter, set_counter) = create_signal(0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let count = move || counter.get();

    view! {
        <p>{t!(i18n, second_namespace.click_count, count)}</p>
        // <p>{t!(i18n, click_count, count = move || counter.get())}</p>
        <button on:click=inc>{t!(i18n, second_namespace.click_to_inc)}</button>
    }
}

#[component]
fn Tests() -> impl IntoView {
    let i18n = i18n_context();

    view! {
        <p>{t!(i18n, first_namespace.common_key)}</p>
        <p>{t!(i18n, second_namespace.common_key)}</p>
    }
}
