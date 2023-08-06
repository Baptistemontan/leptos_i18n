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

// pub fn translate(cx: Scope, key: &str, default: Option<&str>) -> String {
//     let context = get_context(cx);

//     let locale = context.locale.read(cx);

//     let value = match default {
//         Some(default) => locale
//             .as_ref()
//             .map(|l| l.get_by_key_with_default(key, default)),
//         None => locale
//             .as_ref()
//             .map(|l| l.get_by_key(key).unwrap_or_else(|| no_key_present(key))),
//     };

//     value.unwrap_or(key).into()
// }

pub fn get_context<T: Locales>(cx: Scope) -> I18nContext<T> {
    use_context(cx)
        .expect("I18nContext is missing, is the application wrapped in a I18nContextProvider ?")
}

pub fn set_locale<T: Locales>(cx: Scope) -> impl Fn(T::Variants) + Copy + 'static {
    let context = get_context::<T>(cx);

    move |lang| context.set_locale(lang)
}

pub fn get_variant<T: Locales>(cx: Scope) -> impl Fn() -> T::Variants + Copy + 'static {
    let context = get_context::<T>(cx);

    move || context.get_variant()
}

pub fn get_locale<T: Locales>(cx: Scope) -> impl Fn() -> T::LocaleKeys + Copy + 'static {
    let context = get_context::<T>(cx);

    move || context.get_locale()
}
