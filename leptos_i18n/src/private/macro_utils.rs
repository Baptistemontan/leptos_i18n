use std::{borrow::Cow, ops::Deref};

use leptos::IntoView;
#[cfg(feature = "ssr")]
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::{provide_i18n_context, Locale};

pub trait BuildStr: Sized {
    #[inline]
    fn build(self) -> Self {
        self
    }

    #[inline]
    fn build_display(self) -> Self {
        self
    }

    fn build_string(self) -> Cow<'static, str>;
}

impl BuildStr for &'static str {
    #[inline]
    fn build_string(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

#[repr(transparent)]
pub struct SizedString<const N: usize>([u8; N]);

impl<const N: usize> SizedString<N> {
    pub const fn try_new(s: &str) -> Option<Self> {
        if s.len() != N {
            return None;
        }
        // SAFETY:
        // `s` is exactly N bytes in len, so casting it to a `[u8; N]` is totally valid.
        // There is way to do this without unsafe, with for exemple `TryInto<&[u8; N]>` for &[u8],
        // or create a buffer and manually filling it, but none of these methods are const,
        // and it makes things easier if this method can be const.
        let bytes = s.as_bytes().as_ptr().cast::<[u8; N]>();
        Some(SizedString(unsafe { *bytes }))
    }

    #[track_caller]
    pub const fn new(s: &str) -> Self {
        #[cold]
        #[track_caller]
        #[inline(never)]
        const fn empty() -> ! {
            panic!("Receive &str of wrong len.");
        }
        match Self::try_new(s) {
            Some(v) => v,
            None => empty(),
        }
    }
}

impl<const N: usize> Deref for SizedString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // only way to create this type is through a valid str,
        // so the internal buffer is a valid str.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

#[cfg(not(feature = "embed_translations"))]
pub trait ParseTranslation: Sized {
    fn parse(buff: &mut &str) -> Option<Self>;
    fn pop_str<'a>(buff: &mut &'a str, size: usize) -> Option<&'a str> {
        let (s, rest) = Self::split_str(buff, size)?;
        *buff = rest;
        Some(s)
    }
    fn split_str(s: &str, at: usize) -> Option<(&str, &str)> {
        // this is a replica of `str::split_at` but doesn't panic
        // SAFETY:
        // The len is checked inside `is_char_boundary` and it is safe to split at a char boundary.
        s.is_char_boundary(at)
            .then(|| unsafe { (s.get_unchecked(..at), s.get_unchecked(at..)) })
    }
}

#[cfg(not(feature = "embed_translations"))]
impl<const N: usize> ParseTranslation for SizedString<N> {
    fn parse(buff: &mut &str) -> Option<Self> {
        let s = Self::pop_str(buff, N)?;
        Self::try_new(s)
    }
}

#[cfg(feature = "embed_translations")]
pub trait ParseTranslation {}

#[cfg(feature = "embed_translations")]
impl<T> ParseTranslation for T {}

#[cfg(not(feature = "embed_translations"))]
fn load_translations<T: Translation>() -> Box<T> {
    // let mut s = T::STRING;
    // let Some(translations) = ParseTranslation::parse(&mut s) else {
    //     panic!("failed to parse a translation. end of buff: {:?}", s);
    // };
    // Box::new(translations)
    todo!()
}

#[cfg(not(feature = "embed_translations"))]
pub struct TranslationCell<T: Translation>(std::cell::OnceCell<&'static T>);

#[cfg(not(feature = "embed_translations"))]
impl<T: Translation> TranslationCell<T> {
    pub const fn new() -> Self {
        TranslationCell(std::cell::OnceCell::new())
    }

    fn init() -> &'static T {
        let translations: Box<T> = load_translations();
        Box::leak(translations)
    }

    pub fn get(&self) -> &'static T {
        self.0.get_or_init(Self::init)
    }

    pub fn init_from_str(&self, mut s: &str) {
        // check if the cell is not already init
        if self.0.get().is_some() {
            return;
        }
        let Some(translations) = ParseTranslation::parse(&mut s) else {
            panic!("failed to parse a translation. end of buff: {:?}", s)
        };
        let _ = self.0.set(Box::leak(Box::new(translations)));
    }
}

pub trait Translation: ParseTranslation + 'static {
    const PATH: &'static str;
    #[cfg(feature = "ssr")]
    const STRING: &'static str;
}

#[cfg(feature = "ssr")]
#[derive(Debug, Default)]
struct LoadingContextInner(HashMap<&'static str, &'static str>);

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default)]
pub struct LoadingContext(Rc<RefCell<LoadingContextInner>>);

#[cfg(feature = "ssr")]
impl LoadingContext {
    fn inner_mut(&self) -> RefMut<HashMap<&'static str, &'static str>> {
        RefMut::map(self.0.borrow_mut(), |inner| &mut inner.0)
    }

    fn inner(&self) -> Ref<HashMap<&'static str, &'static str>> {
        Ref::map(self.0.borrow(), |inner| &inner.0)
    }

    fn register_inner<T: Translation>(&self) {
        let mut inner = self.inner_mut();
        inner.insert(T::PATH, T::STRING);
    }

    pub fn register<T: Translation>() {
        if let Some(this) = leptos::use_context::<Self>() {
            this.register_inner::<T>();
        } else if cfg!(debug_assertions) {
            eprintln!(
                "Warning: Tried to register a translation but the LoadingContext was not present, This is probably caused by not wrapping the application with the I18nContextProvider. \
                This won't cause major problems but may indicate a logical error. This error is present only with debug_assertions."
            );
        }
    }

    pub fn to_array(&self) -> String {
        let inner = self.inner();
        let translations: Vec<_> = inner
            .iter()
            .map(|(path, value)| format!("{{\"path\":{:?},\"value\":{:?}}}", path, value))
            .collect();

        translations.join(",")
    }
}

#[cfg(all(feature = "hydrate", not(feature = "embed_translations")))]
fn init_translations<T: Locale>() {
    use wasm_bindgen::UnwrapThrowExt;
    #[derive(serde::Deserialize)]
    struct Trans {
        path: String,
        value: String,
    }

    let translations = js_sys::Reflect::get(
        &web_sys::window().unwrap_throw(),
        &wasm_bindgen::JsValue::from_str("__LEPTOS_I18N_TRANSLATIONS"),
    )
    .expect_throw("No __LEPTOS_I18N_TRANSLATIONS found in the JS global scope");

    let translations: Vec<Trans> = serde_wasm_bindgen::from_value(translations)
        .expect_throw("Failed parsing the translations.");

    for Trans { path, value } in translations {
        T::init_translation(&path, &value);
    }
}

#[cfg(not(feature = "ssr"))]
pub fn provider<T: Locale>(children: leptos::Children) -> impl IntoView {
    #[cfg(all(feature = "hydrate", not(feature = "embed_translations")))]
    init_translations::<T>();
    provide_i18n_context::<T>();
    children()
}

#[cfg(feature = "ssr")]
pub fn provider<T: Locale>(children: leptos::Children) -> impl IntoView {
    provide_i18n_context::<T>();
    let loading_ctx = LoadingContext::default();
    leptos::provide_context(loading_ctx.clone());
    let children = children();
    let translations = loading_ctx.to_array();
    leptos::view! {
        {children}
        <script>
            window.__LEPTOS_I18N_TRANSLATIONS = "[" {translations} "]";
        </script>
    }
}
