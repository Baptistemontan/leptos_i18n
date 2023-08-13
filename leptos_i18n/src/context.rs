use leptos::*;
use leptos_meta::*;

use crate::{fetch_locale, locale_traits::*};

#[derive(Debug, Clone, Copy)]
pub struct I18nContext<T: Locales>(RwSignal<T::Variants>);

impl<T: Locales> I18nContext<T> {
    #[inline]
    pub fn get_locale(self) -> T::Variants {
        self.0.get()
    }

    #[inline]
    pub fn get_keys(self) -> T::LocaleKeys {
        let variant = self.get_locale();
        LocaleKeys::from_variant(variant)
    }

    #[inline]
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

    let locale = create_rw_signal(cx, locale);

    create_isomorphic_effect(cx, move |_| {
        let new_lang = locale.get();
        set_html_lang_attr(cx, new_lang.as_str());
        #[cfg(feature = "cookie")]
        set_lang_cookie::<T>(new_lang);
    });

    let context = I18nContext::<T>(locale);

    provide_context(cx, context);

    context
}

pub fn get_context<T: Locales>(cx: Scope) -> I18nContext<T> {
    use_context(cx).expect("I18nContext is missing, use provide_i18n_context() to provide it.")
}

#[cfg(all(feature = "hydrate", feature = "cookie"))]
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

#[cfg(all(not(feature = "hydrate"), feature = "cookie"))]
fn set_lang_cookie<T: Locales>(lang: T::Variants) -> Option<()> {
    let _ = lang;
    Some(())
}
