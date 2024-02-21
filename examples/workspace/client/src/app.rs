use crate::i18n::*;
use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    view! {
        <I18nContextProvider>
            <ChangeLocale />
            <Counter />
        </I18nContextProvider>
    }
}

#[component]
fn ChangeLocale() -> impl IntoView {
    let i18n = use_i18n();

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            Locale::en => Locale::fr,
            Locale::fr => Locale::en,
        };
        i18n.set_locale(new_lang);
    };

    view! {
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
    }
}

#[component]
fn Counter() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, set_counter) = create_signal(0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let count = move || counter.get();

    view! {
        <p>{t!(i18n, click_count, count)}</p>
        // <p>{t!(i18n, click_count, count = move || counter.get())}</p>
        <button on:click=inc>{t!(i18n, click_to_inc)}</button>
    }
}
