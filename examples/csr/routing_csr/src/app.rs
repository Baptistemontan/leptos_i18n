use crate::i18n::*;
use leptos::prelude::*;
use leptos_router::{components::*, path};

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    view! {
        <I18nContextProvider>
            <Router>
                <Routes fallback=|| "This page could not be found.">
                    <I18nRoute view=|| view! { <Outlet /> }>
                        <Route path=path!("/") view=Home />
                        <Route path=path!("/counter") view=Counter />
                    </I18nRoute>
                </Routes>
                <br/>
                <SwitchLang />
            </Router>
        </I18nContextProvider>
    }
}

#[component]
#[allow(non_snake_case)]
pub fn SwitchLang() -> impl IntoView {
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
#[allow(non_snake_case)]
fn Home() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <h1>{t!(i18n, hello_world)}</h1>
        <A href="/counter">{t!(i18n, go_counter)}</A>
    }
}

#[component]
#[allow(non_snake_case)]
fn Counter() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, set_counter) = signal(0);

    let inc = move |_| set_counter.update(|count| *count += 1);

    let count = move || counter.get();

    view! {
        <div>
            <p>
                {t!{ i18n,
                    click_count,
                    count,
                    <b> = <b />,
                }}
            </p>
            <button on:click=inc>{t!(i18n, click_to_inc)}</button>
            <br />
            <A href="/">{t!(i18n, go_home)}</A>
        </div>
    }
}
