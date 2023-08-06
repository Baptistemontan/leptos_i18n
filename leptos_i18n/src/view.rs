use std::future::Future;
use std::rc::Rc;

use leptos::*;
use leptos_meta::*;

use crate::locale_traits::*;

#[derive(Debug, Clone, Copy)]
pub struct I18nContext<T: Locales> {
    locale: Resource<(), T::Variants>,
}

impl<T: Locales> I18nContext<T> {
    pub fn get_variant(self, cx: Scope) -> T::Variants {
        self.locale.with(cx, |l| *l).unwrap_or_default()
    }

    pub fn get_locale(self, cx: Scope) -> T::LocaleKeys {
        let variant = self.get_variant(cx);
        LocaleKeys::from_variant(variant)
    }

    pub fn set_locale(self, lang: T::Variants) {
        self.locale.set(lang);
    }
}

#[component]
pub fn I18nContextProvider<T, F>(
    cx: Scope,
    locales: T,
    fetch_locale: fn(Scope) -> F,
    children: ChildrenFn,
) -> impl IntoView
where
    T: Locales,
    F: Future<Output = Result<T::Variants, ServerFnError>> + 'static,
{
    let _ = locales;
    let locale = create_blocking_resource(
        cx,
        || (),
        move |()| async move { fetch_locale(cx).await.unwrap() },
    );

    provide_context(cx, I18nContext::<T> { locale });

    let lang = move || {
        locale.with(cx, |lang| {
            let lang = LocaleVariant::as_str(lang);
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

pub fn get_variant<T: Locales>(cx: Scope) -> impl Fn(Scope) -> T::Variants + Copy + 'static {
    let context = get_context::<T>(cx);

    move |cx| context.get_variant(cx)
}

pub fn get_locale<T: Locales>(cx: Scope) -> impl Fn(Scope) -> T::LocaleKeys + Copy + 'static {
    let context = get_context::<T>(cx);

    move |cx: Scope| context.get_locale(cx)
}
