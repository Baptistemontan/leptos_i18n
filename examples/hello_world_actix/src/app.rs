use crate::i18n::{i18n_context, LocaleEnum, Locales};
use leptos::*;
use leptos_i18n::t;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    leptos_meta::provide_meta_context(cx);

    leptos_i18n::provide_i18n_context::<Locales>(cx);

    let i18n = i18n_context(cx);

    let (counter, set_counter) = create_signal(cx, 0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        i18n.set_locale(new_lang);
    };

    view! { cx,
        <h1>{t!(i18n, hello_world)}</h1>
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
        <p>
            {t!{ i18n,
                click_count,
                count = move || counter.get(),
                <b> = |cx, children| view!{ cx, <b>{children(cx)}</b> },
            }}
        </p>
        <button on:click=inc>{t!(i18n, inc)}</button>
    }
}
