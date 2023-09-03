use leptos::*;
use leptos_meta::*;

use crate::{fetch_locale, locale_traits::*};

/// This context is the heart of the i18n system:
///
/// It servers as a signal to the the current locale and enable reactivity to locale change.
///
/// You access the translations and read/update the current locale through it.
#[derive(Debug, Clone, Copy)]
pub struct I18nContext<T: Locales>(RwSignal<T::Variants>);

impl<T: Locales> I18nContext<T> {
    /// Return the current locale subscribing to any changes.
    #[inline]
    pub fn get_locale(self) -> T::Variants {
        self.0.get()
    }

    /// Return the current locale but does not subscribe to changes
    #[inline]
    pub fn get_locale_untracked(self) -> T::Variants {
        self.0.get_untracked()
    }

    /// Return the keys for the current locale subscribing to any changes
    #[inline]
    pub fn get_keys(self) -> &'static T::LocaleKeys {
        let variant = self.get_locale();
        LocaleKeys::from_variant(variant)
    }

    /// Return the keys for the current locale but does not subscribe to changes
    #[inline]
    pub fn get_keys_untracked(self) -> &'static T::LocaleKeys {
        let variant = self.get_locale();
        LocaleKeys::from_variant(variant)
    }

    /// Set the locale and notify all subscribers
    #[inline]
    pub fn set_locale(self, lang: T::Variants) {
        self.0.set(lang)
    }

    /// Set the locale but does not notify the subscribers
    #[inline]
    pub fn set_locale_untracked(self, lang: T::Variants) {
        self.0.set_untracked(lang)
    }
}

fn set_html_lang_attr(lang: &'static str) {
    let lang = || lang.to_string();
    Html(HtmlProps {
        lang: Some(lang.into()),
        dir: None,
        class: None,
        attributes: None,
    });
}

fn init_context<T: Locales>() -> I18nContext<T> {
    provide_meta_context();

    let locale = fetch_locale::fetch_locale::<T>();

    let locale = create_rw_signal(locale);

    create_isomorphic_effect(move |_| {
        let new_lang = locale.get();
        set_html_lang_attr(new_lang.as_str());
        #[cfg(all(feature = "cookie", feature = "hydrate"))]
        set_lang_cookie::<T>(new_lang);
    });

    let context = I18nContext::<T>(locale);

    provide_context(context);

    context
}

/// Provide the `I18nContext` for the application.
///
/// This function must be called at the highest possible level of the application.
///
/// It returns the newly created context.
///
/// If called when a context is already present it will not overwrite it and just return the current context.
pub fn provide_i18n_context<T: Locales>() -> I18nContext<T> {
    use_context().unwrap_or_else(init_context)
}

/// Return the `I18nContext` previously set.
///
/// ## Panic
///
/// Panics if the context is missing.
#[inline]
pub fn get_context<T: Locales>() -> I18nContext<T> {
    #[cold]
    pub fn not_present() -> ! {
        panic!("I18nContext is missing, use provide_i18n_context() to provide it.")
    }
    use_context().unwrap_or_else(not_present)
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

// get locale
#[cfg(feature = "nightly")]
impl<T: Locales> FnOnce<()> for I18nContext<T> {
    type Output = T::Variants;
    #[inline]
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<T: Locales> FnMut<()> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<T: Locales> Fn<()> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call(&self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

// set locale
#[cfg(feature = "nightly")]
impl<T: Locales> FnOnce<(T::Variants,)> for I18nContext<T> {
    type Output = ();
    #[inline]
    extern "rust-call" fn call_once(self, (locale,): (T::Variants,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<T: Locales> FnMut<(T::Variants,)> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, (locale,): (T::Variants,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<T: Locales> Fn<(T::Variants,)> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call(&self, (locale,): (T::Variants,)) -> Self::Output {
        self.set_locale(locale)
    }
}
