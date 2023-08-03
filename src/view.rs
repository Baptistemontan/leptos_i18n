use std::rc::Rc;

use leptos::*;
use leptos_meta::*;

use crate::Locale;

#[derive(Debug, Clone, Copy)]
pub struct I18nContext {
    selected_lang: RwSignal<Option<String>>,
    locale: Resource<Option<String>, Locale>,
}

#[server(GetLocales, "/api")]
async fn fetch_locales(cx: Scope, selected_lang: Option<String>) -> Result<Locale, ServerFnError> {
    leptos_actix::extract(
        cx,
        |config: actix_web::web::Data<crate::server::I18nConfig>,
         accepted_lang: crate::accepted_lang::AcceptedLang| async move {
            if let Some(locale) = selected_lang.and_then(|sl| config.get_local(&sl)) {
                return locale.clone();
            }
            accepted_lang
                .find_first_lang(&config.locales)
                .unwrap_or(&config.default_locale)
                .clone()
        },
    )
    .await
}

#[component]
pub fn I18nContextProvider(cx: Scope, children: ChildrenFn) -> impl IntoView {
    let selected_lang = create_rw_signal(cx, None);

    let locale = create_blocking_resource(
        cx,
        move || selected_lang.get(),
        move |sl| async move { fetch_locales(cx, sl).await.unwrap() },
    );

    provide_context(
        cx,
        I18nContext {
            locale,
            selected_lang,
        },
    );

    let lang = move || {
        locale.with(cx, |l| {
            let lang = l.lang.clone();
            view! { cx,
                <Html lang=lang />
            }
        })
    };

    let children = store_value(cx, Rc::new(children));

    let render = move || children.get_value()(cx);

    view! { cx,
        <Suspense fallback=render >
            {lang}
            {render}
        </Suspense>
    }
}

#[cold]
fn no_key_present(key: &str) -> ! {
    panic!("i18n key {:?} is not valid.", key)
}

pub fn translate(cx: Scope, key: &str, default: Option<&str>) -> String {
    let context = get_context(cx);

    let locale = context.locale.read(cx);

    let value = match default {
        Some(default) => locale
            .as_ref()
            .map(|l| l.get_by_key_with_default(key, default)),
        None => locale
            .as_ref()
            .map(|l| l.get_by_key(key).unwrap_or_else(|| no_key_present(key))),
    };

    value.unwrap_or(key).into()
}

pub fn get_context(cx: Scope) -> I18nContext {
    use_context::<I18nContext>(cx)
        .expect("I18nContext is missing, is the application wrapped in a I18nContextProvider ?")
}

pub fn set_locale<T: Into<String>>(cx: Scope, lang: T) {
    let lang = lang.into();
    let context = get_context(cx);

    #[cfg(feature = "hydrate")]
    let location = leptos::window().location();
    #[cfg(feature = "hydrate")]
    let redirect_url = location
        .href()
        .map(|origin| format!("/api/locale/set?origin={}&lang={}", origin, lang));

    context.selected_lang.set(Some(lang));

    #[cfg(feature = "hydrate")]
    if let Ok(redirect_url) = redirect_url {
        let _ = location.set_href(&redirect_url);
    }
}

pub fn get_locale(cx: Scope) -> Option<String> {
    let context = get_context(cx);

    context.locale.with(cx, |l| l.lang.clone())
}
