use crate::i18n::*;
use crate::t;
use leptos::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    leptos_meta::provide_meta_context(cx);

    leptos_i18n::provide_i18n_context::<Locales>(cx);

    let get_variant = leptos_i18n::get_variant::<Locales>(cx);
    let set_locale = leptos_i18n::set_locale::<Locales>(cx);

    let (counter, set_counter) = create_signal(cx, 0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let on_click = move |_| {
        let new_lang = match get_variant() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        set_locale(new_lang);
    };

    view! { cx,
        <h1>{t!(cx, hello_world)}</h1>
        <button on:click=on_click >{t!(cx, click_to_change_lang)}</button>
        <p>
            {t!{ cx,
                click_count,
                count = move || counter.get(),
                b = |cx, children| view!{ cx, <b>{children(cx)}</b> },
            }}
        </p>
        <button on:click=inc>{t!(cx, inc)}</button>
    }
}
