use std::rc::Rc;

use leptos::*;
use leptos_meta::*;

use crate::Locale;

#[derive(Debug, Clone)]
pub struct I18nContext {
    selected_lang: RwSignal<Option<String>>,
    locale: Locale,
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

    let locales = create_blocking_resource(
        cx,
        move || selected_lang.get(),
        move |selected_lang| async move { fetch_locales(cx, selected_lang).await.unwrap() },
    );

    let children = store_value(cx, Rc::new(children));

    let render = move || {
        locales.read(cx).map(move |locale| {
            let lang = locale.lang.clone();
            provide_context(
                cx,
                I18nContext {
                    locale,
                    selected_lang,
                },
            );
            view! { cx,
                <Html lang=lang />
                {children.get_value()(cx)}
            }
        })
    };

    view! { cx,
        <Suspense fallback=move || view! { cx, "" }>
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

    let value = match default {
        Some(default) => context.locale.get_by_key_with_default(key, default),
        None => context
            .locale
            .get_by_key(key)
            .unwrap_or_else(|| no_key_present(key)),
    };

    value.to_string()
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

pub fn get_locale(cx: Scope) -> String {
    let context = get_context(cx);

    context.locale.lang.clone()
}
