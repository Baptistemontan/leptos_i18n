use crate::i18n::{i18n_context, LocaleEnum, Locales};
use leptos::*;
use leptos_i18n::t;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    leptos_meta::provide_meta_context(cx);

    let i18n = leptos_i18n::provide_i18n_context::<Locales>(cx);

    let on_switch = move |_| {
        let new_lang = match i18n.get_locale() {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };
        i18n.set_locale(new_lang);
    };

    view! { cx,
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
        <Counter />
        <Tests />
    }
}

#[component]
fn Counter(cx: Scope) -> impl IntoView {
    let i18n = i18n_context(cx);

    let (counter, set_counter) = create_signal(cx, 0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let count = move || counter.get();

    view! { cx,
        <p>{t!(i18n, click_count, count)}</p>
        // <p>{t!(i18n, click_count, count = move || counter.get())}</p>
        <button on:click=inc>{t!(i18n, click_to_inc)}</button>
    }
}

#[component]
fn Tests(cx: Scope) -> impl IntoView {
    let i18n = i18n_context(cx);

    view! { cx,
        <p>{t!(i18n, f32_plural, count = || 42f32)}</p>
        <p>{t!(i18n, u32_plural, count = || 42u16)}</p>// u16 to test conversion
        <p>{t!(i18n, f32_OR_plural, count = || 42f32)}</p>
        <p>{t!(i18n, OR_plural, count = || 42u8)}</p>
    }
}
