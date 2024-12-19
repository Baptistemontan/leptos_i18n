#![doc(hidden)]

use crate::Locale;

#[cfg(feature = "dynamic_load")]
pub use async_once_cell::OnceCell;

pub trait TranslationUnit: Sized {
    type Locale: Locale;
    const ID: <Self::Locale as Locale>::TranslationUnitId;
    const LOCALE: Self::Locale;
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    const STRING: &'static str;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_strings_lock() -> &'static OnceCell<String>;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn request_strings() -> impl std::future::Future<Output = &'static str> + Send + Sync + 'static
    {
        let string_lock = Self::get_strings_lock();
        let fut = string_lock.get_or_init(async {
            Locale::request_translations(Self::LOCALE, Self::ID)
                .await
                .unwrap()
                .into_owned()
        });
        async move { core::ops::Deref::deref(fut.await) }
    }

    #[cfg(all(feature = "dynamic_load", feature = "hydrate", not(feature = "ssr")))]
    fn init_translations(values: String) {
        let string_lock = Self::get_strings_lock();
        let fut = string_lock.get_or_init(async { values });
        futures::executor::block_on(fut);
    }

    #[cfg(all(feature = "dynamic_load", feature = "ssr"))]
    fn register() {
        RegisterCtx::register::<Self>();
    }
}

#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
mod register {
    use super::*;
    use crate::locale_traits::TranslationUnitId;
    use leptos::prelude::{provide_context, use_context};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    type RegisterCtxMap<L, Id> = HashMap<(L, Id), &'static str>;

    #[derive(Clone)]
    pub struct RegisterCtx<L: Locale>(Arc<Mutex<RegisterCtxMap<L, L::TranslationUnitId>>>);

    impl<L: Locale> RegisterCtx<L> {
        pub fn provide_context() -> Self {
            let inner = Arc::new(Mutex::new(HashMap::new()));
            provide_context(RegisterCtx(inner.clone()));
            RegisterCtx(inner)
        }

        pub fn register<T: TranslationUnit<Locale = L>>() {
            if let Some(this) = use_context::<Self>() {
                let mut inner_guard = this.0.lock().unwrap();
                inner_guard.insert((T::LOCALE, T::ID), T::STRING);
            }
        }

        pub fn to_array(&self) -> String {
            let mut buff = String::from("window.__LEPTOS_I18N_TRANSLATIONS = [");
            let inner_guard = self.0.lock().unwrap();
            let mut first = true;
            for ((locale, id), values) in &*inner_guard {
                if !std::mem::replace(&mut first, false) {
                    buff.push(',');
                }
                buff.push_str("{\"locale\":\"");
                buff.push_str(locale.as_str());
                if let Some(id_str) = TranslationUnitId::to_str(*id) {
                    buff.push_str("\",\"id\":\"");
                    buff.push_str(id_str);
                    buff.push_str("\",\"values\":[");
                } else {
                    buff.push_str("\",\"id\":null,\"values\":\"");
                }
                buff.push_str(values);
                buff.push_str("\"}");
            }
            buff.push_str("];");
            buff
        }
    }
}

#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
pub use register::RegisterCtx;

#[cfg(all(feature = "dynamic_load", feature = "hydrate"))]
pub fn init_translations<L: Locale>() -> impl leptos::IntoView {
    use leptos::{html::InnerHtmlAttribute, view, web_sys};
    use wasm_bindgen::UnwrapThrowExt;
    #[derive(serde::Deserialize)]
    struct Trans<L, Id> {
        locale: L,
        id: Id,
        values: String,
    }

    let translations = js_sys::Reflect::get(
        &web_sys::window().unwrap_throw(),
        &wasm_bindgen::JsValue::from_str("__LEPTOS_I18N_TRANSLATIONS"),
    )
    .expect_throw("No __LEPTOS_I18N_TRANSLATIONS found in the JS global scope");

    let translations: Vec<Trans<L, L::TranslationUnitId>> =
        serde_wasm_bindgen::from_value(translations)
            .expect_throw("Failed parsing the translations.");

    let mut buff = String::from("window.__LEPTOS_I18N_TRANSLATIONS = [");

    for Trans { locale, id, values } in translations {
        let mut first = true;
        if !std::mem::replace(&mut first, false) {
            buff.push(',');
        }
        buff.push_str("{\"locale\":\"");
        buff.push_str(locale.as_str());
        if let Some(id_str) = crate::locale_traits::TranslationUnitId::to_str(id) {
            buff.push_str("\",\"id\":\"");
            buff.push_str(id_str);
            buff.push_str("\",\"values\":[");
        } else {
            buff.push_str("\",\"id\":null,\"values\":\"");
        }
        buff.push_str(&values);
        buff.push_str("\"}");
        L::init_translations(locale, id, values);
    }

    buff.push_str("];");

    view! {
        <script inner_html = buff />
    }
}
