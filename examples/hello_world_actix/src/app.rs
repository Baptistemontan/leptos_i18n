use crate::i18n::*;
use crate::t;
use leptos::*;
use leptos_i18n::I18nContextProvider;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    leptos_meta::provide_meta_context(cx);

    view! { cx,
        <I18nContextProvider locales=Locales>
            <Inner />
        </I18nContextProvider>
    }
}

#[component]
fn Inner(cx: Scope) -> impl IntoView {
    let get_variant = leptos_i18n::get_variant::<Locales>(cx);
    let set_locale = leptos_i18n::set_locale::<Locales>(cx);
    let on_click = move |_| {
        let locale = get_variant();
        let new_lang = match locale {
            LocaleEnum::en => LocaleEnum::fr,
            LocaleEnum::fr => LocaleEnum::en,
        };

        set_locale(new_lang);
    };

    view! { cx,
        <h1>{t!(cx, hello_world)}</h1>
        <button on:click=on_click >{t!(cx, click_to_change_lang)}</button>
    }
}
