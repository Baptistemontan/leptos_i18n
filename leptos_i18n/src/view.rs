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

fn set_html_lang_attr(cx: Scope, lang: &'static str) {
    let lang = || lang.to_string();
    Html(
        cx,
        HtmlProps {
            lang: Some(lang.into()),
            dir: None,
            class: None,
            attributes: None,
        },
    );
}

pub fn provide_i18n_context<T: Locales>(cx: Scope) -> I18nContext<T> {
    provide_meta_context(cx);

    let locale = fetch_locale::fetch_locale::<T>(cx);

    set_html_lang_attr(cx, locale.as_str());

    let locale_sig = create_rw_signal(cx, locale);

    let context = I18nContext::<T>(locale_sig);

    provide_context(cx, context);

    context
}

pub fn get_context<T: Locales>(cx: Scope) -> I18nContext<T> {
    use_context(cx).expect("I18nContext is missing, use provide_i18n_context() to provide it.")
}

pub fn set_locale<T: Locales>(cx: Scope) -> impl Fn(T::Variants) + Copy + 'static {
    let context = get_context::<T>(cx);

    move |lang| {
        context.set_locale(lang);
        set_lang_cookie::<T>(lang);
        set_html_lang_attr(cx, lang.as_str())
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
        "{}={}; SameSite=Lax; Secure; Path=/; Max-Age=31536000",
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
