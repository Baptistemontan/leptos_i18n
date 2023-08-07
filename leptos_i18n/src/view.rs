use std::rc::Rc;

use leptos::*;
use leptos_meta::*;

use crate::{fetch_locale, locale_traits::*};

#[derive(Debug, Clone, Copy)]
pub struct I18nContext<T: Locales>(RwSignal<T::Variants>);

impl<T: Locales> I18nContext<T> {
    pub fn get_variant(self) -> T::Variants {
        self.0.get()
    }

    pub fn get_locale(self) -> T::LocaleKeys {
        let variant = self.get_variant();
        LocaleKeys::from_variant(variant)
    }

    pub fn set_locale(self, lang: T::Variants) {
        self.0.set(lang)
    }
}

#[component]
pub fn I18nContextProvider<T: Locales>(
    cx: Scope,
    locales: T,
    children: ChildrenFn,
) -> impl IntoView {
    let _ = locales;

    let locale = fetch_locale::fetch_locale::<T>(cx);

    let locale = create_rw_signal(cx, locale);

    provide_context(cx, I18nContext::<T>(locale));

    let lang = move || {
        let lang = locale.get();
        let lang = lang.as_str();
        view! { cx,
            <Html lang=lang />
        }
    };
    let children = store_value(cx, Rc::new(children));

    let render_children = move || children.get_value()(cx);

    view! { cx,
        {lang}
        {render_children}
    }
}

pub fn get_context<T: Locales>(cx: Scope) -> I18nContext<T> {
    use_context(cx)
        .expect("I18nContext is missing, is the application wrapped in a I18nContextProvider ?")
}

pub fn set_locale<T: Locales>(cx: Scope) -> impl Fn(T::Variants) + Copy + 'static {
    let context = get_context::<T>(cx);

    move |lang| {
        context.set_locale(lang);
        set_lang_cookie::<T>(lang);
    }
}

pub fn get_variant<T: Locales>(cx: Scope) -> impl Fn() -> T::Variants + Copy + 'static {
    let context = get_context::<T>(cx);

    move || context.get_variant()
}

pub fn get_locale<T: Locales>(cx: Scope) -> impl Fn() -> T::LocaleKeys + Copy + 'static {
    let context = get_context::<T>(cx);

    move || context.get_locale()
}

#[cfg(feature = "hydrate")]
fn set_lang_cookie<T: Locales>(lang: T::Variants) -> Option<()> {
    use crate::COOKIE_PREFERED_LANG;
    use wasm_bindgen::JsCast;
    let document = document().dyn_into::<web_sys::HtmlDocument>().ok()?;
    let cookie = format!(
        "{}={}; SameSite=Lax; Secure; Path=/",
        COOKIE_PREFERED_LANG,
        lang.as_str()
    );
    document.set_cookie(&cookie).ok()
}

#[cfg(not(feature = "hydrate"))]
fn set_lang_cookie<T: Locales>(lang: T::Variants) -> Option<()> {
    let _ = lang;
    Some(())
}
