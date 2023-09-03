use crate::i18n::{i18n_context, LocaleEnum, Locales};
use leptos::*;
use leptos_i18n::t;

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    leptos_i18n::provide_i18n_context::<Locales>();

    let i18n = i18n_context();

    let (counter, set_counter) = create_signal(0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        i18n.set_locale(new_lang);
    };

    let count = move || counter.get();

    view! {
        <h1>{t!(i18n, hello_world)}</h1>
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
        <p>
            {t!{ i18n,
                click_count,
                count,
                <b> = |children| view!{ <b>{children}</b> },
            }}
        </p>
        <button on:click=inc>{t!(i18n, inc)}</button>
    }
}
