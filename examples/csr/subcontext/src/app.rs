use crate::i18n::*;
use leptos::*;
use leptos_i18n::context::{init_i18n_subcontext, I18nSubContextProvider};
use leptos_i18n::I18nContext;
use leptos_i18n::Locale as _;

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    view! {
        <I18nContextProvider>
            <Main />
            <Opposite />
            <Cookie />
            <LangAttr />
        </I18nContextProvider>
    }
}

#[component]
#[allow(non_snake_case)]
fn Opposite() -> impl IntoView {
    let i18n = use_i18n();

    let sub_context_locale = move || neg_locale(i18n.get_locale());

    view! {
        <h2>{t!(i18n, examples.opposite)}</h2>
        <I18nSubContextProvider
            initial_locale=sub_context_locale
        >
            <Counter />
        </I18nSubContextProvider>
    }
}

#[component]
#[allow(non_snake_case)]
fn Cookie() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <h2>{t!(i18n, examples.cookie)}</h2>
        <I18nSubContextProvider
            initial_locale=move || Locale::fr
            cookie_name="cookie_example_locale"
        >
            <Counter />
        </I18nSubContextProvider>
    }
}

#[component]
#[allow(non_snake_case)]
fn Main() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <h2>{t!(i18n, examples.main)}</h2>
        <Counter />
    }
}

#[component]
#[allow(non_snake_case)]
fn LangAttr() -> impl IntoView {
    let i18n = use_i18n();

    let i18n_sub = init_i18n_subcontext::<Locale>(None);
    view! {
        <h2>{t!(i18n, examples.lang_attr)}</h2>
        <div use:i18n_sub >
            <Provider value=i18n_sub>
                <Counter />
            </Provider>
        </div>
    }
}

#[component]
#[allow(non_snake_case)]
fn Counter() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, set_counter) = create_signal(0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let count = move || counter.get();

    let on_switch = make_on_switch(i18n);

    view! {
        <h1>{move || i18n.get_locale().as_str()}</h1>
        <p>{t!(i18n, click_count, count)}</p>
        <button on:click=inc>{t!(i18n, click_to_inc)}</button>
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
    }
}

fn neg_locale(locale: Locale) -> Locale {
    match locale {
        Locale::en => Locale::fr,
        Locale::fr => Locale::en,
    }
}

fn make_on_switch<E>(i18n: I18nContext<Locale>) -> impl Fn(E) + 'static {
    move |_| {
        let new_lang = neg_locale(i18n.get_locale());
        i18n.set_locale(new_lang);
    }
}
