#![doc(hidden)]

use std::fmt::Debug;

use crate::Locale;

#[cfg(feature = "dynamic_load")]
pub use async_once_cell::OnceCell;

pub trait TranslationUnit: Sized {
    type Locale: Locale;
    const ID: <Self::Locale as Locale>::TranslationUnitId;
    const LOCALE: Self::Locale;
    type Strings: StringArray;
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    const STRINGS: &'static Self::Strings;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_strings_lock() -> &'static OnceCell<Box<Self::Strings>>;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn request_strings()
    -> impl std::future::Future<Output = &'static Self::Strings> + Send + Sync + 'static {
        let string_lock = Self::get_strings_lock();
        let fut = string_lock.get_or_init(async {
            let translations = Locale::request_translations(Self::LOCALE, Self::ID)
                .await
                .unwrap();
            StringArray::cast(translations.0)
        });
        async move { core::ops::Deref::deref(fut.await) }
    }

    #[cfg(all(feature = "dynamic_load", feature = "hydrate", not(feature = "ssr")))]
    fn init_translations(values: Vec<Box<str>>) {
        let string_lock = Self::get_strings_lock();
        let fut = string_lock.get_or_init(async { StringArray::cast(values) });
        futures::executor::block_on(fut);
    }

    #[cfg(all(feature = "dynamic_load", feature = "ssr"))]
    fn register() {
        RegisterCtx::register::<Self>();
    }
}

pub trait StringArray: 'static + Send + Sync + Debug {
    fn cast(strings: Vec<Box<str>>) -> Box<Self>;
    fn as_slice(&self) -> &[&'static str];
}

impl<const SIZE: usize> StringArray for [Box<str>; SIZE] {
    fn cast(strings: Vec<Box<str>>) -> Box<Self> {
        strings.into_boxed_slice().try_into().unwrap()
    }

    fn as_slice(&self) -> &[&'static str] {
        unreachable!("This function should not have been called on the client !")
    }
}

impl<const SIZE: usize> StringArray for [&'static str; SIZE] {
    fn cast(_: Vec<Box<str>>) -> Box<Self> {
        unreachable!("This function should not have been called on the server !")
    }

    fn as_slice(&self) -> &[&'static str] {
        self
    }
}

#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
pub type LocaleServerFnOutput = LocaleServerFnOutputServer;

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub type LocaleServerFnOutput = LocaleServerFnOutputClient;

pub struct LocaleServerFnOutputServer(&'static [&'static str]);
pub struct LocaleServerFnOutputClient(pub Vec<Box<str>>);

impl LocaleServerFnOutputServer {
    pub const fn new(strings: &'static [&'static str]) -> Self {
        LocaleServerFnOutputServer(strings)
    }
}

impl LocaleServerFnOutputClient {
    pub fn new(_: &'static [&'static str]) -> Self {
        unreachable!("This function should not have been called on the server !")
    }
}

impl serde::Serialize for LocaleServerFnOutputServer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(self.0, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for LocaleServerFnOutputServer {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unreachable!("This function should not have been called on the server !")
    }
}

impl serde::Serialize for LocaleServerFnOutputClient {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unreachable!("This function should not have been called on the client !")
    }
}

impl<'de> serde::Deserialize<'de> for LocaleServerFnOutputClient {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr = serde::Deserialize::deserialize(deserializer)?;
        Ok(LocaleServerFnOutputClient(arr))
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

    type RegisterCtxMap<L, Id> = HashMap<(L, Id), &'static [&'static str]>;

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
                inner_guard.insert((T::LOCALE, T::ID), T::STRINGS.as_slice());
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
                    buff.push_str("\",\"id\":null,\"values\":[");
                }
                let mut first = true;
                for value in *values {
                    if !std::mem::replace(&mut first, false) {
                        buff.push(',');
                    }
                    buff.push('\"');
                    buff.push_str(value);
                    buff.push('\"');
                }
                buff.push_str("]}");
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
        values: Vec<Box<str>>,
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
            buff.push_str("\",\"id\":null,\"values\":[");
        }
        let mut first = true;
        for value in &values {
            if !std::mem::replace(&mut first, false) {
                buff.push(',');
            }
            buff.push('\"');
            buff.push_str(value);
            buff.push('\"');
        }
        buff.push_str("]}");
        L::init_translations(locale, id, values);
    }

    buff.push_str("];");

    view! {
        <script inner_html = buff />
    }
}
