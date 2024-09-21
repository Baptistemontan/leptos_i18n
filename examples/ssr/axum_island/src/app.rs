use crate::i18n::*;
use leptos::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    view! {
        <I18nContextProvider
            set_lang_attr_on_html=None
            enable_cookie=None
            cookie_name=None
            ssr_lang_header_getter=None
        >
            <h1>
                {ti!(HelloWorld, hello_world)}
            </h1>
            <Counter />
            <ChangeLang />
        </I18nContextProvider>
    }
}

#[island]
#[allow(non_snake_case)]
fn Counter() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, set_counter) = signal(0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let count = move || counter.get();

    view! {
        <p>
            {t!{
                i18n,
                click_count,
                count,
                <b> = <b />,
            }}
        </p>
        <button on:click=inc>{t!(i18n, click_to_inc)}</button>
    }
}

#[island]
#[allow(non_snake_case)]
fn ChangeLang() -> impl IntoView {
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
