use codee::string::FromToStringCodec;
use core::marker::PhantomData;
use html::{AnyElement, ElementDescriptor};
use leptos::*;
use leptos_meta::*;
use leptos_use::UseCookieOptions;

use crate::{fetch_locale, locale_traits::*, scopes::ConstScope, Scope};

/// This context is the heart of the i18n system:
///
/// It servers as a signal to the the current locale and enable reactivity to locale change.
///
/// You access the translations and read/update the current locale through it.
#[derive(Debug, Clone, Copy)]
pub struct I18nContext<L: Locale, S: Scope<L> = <L as Locale>::Keys> {
    locale_signal: RwSignal<L>,
    scope_marker: PhantomData<S>,
}

impl<L: Locale, S: Scope<L>> I18nContext<L, S> {
    /// Return the current locale subscribing to any changes.
    #[inline]
    pub fn get_locale(self) -> L {
        self.locale_signal.get()
    }

    /// Return the current locale but does not subscribe to changes
    #[inline]
    pub fn get_locale_untracked(self) -> L {
        self.locale_signal.get_untracked()
    }

    /// Return the keys for the current locale subscribing to any changes
    #[inline]
    pub fn get_keys(self) -> &'static S::Keys {
        LocaleKeys::from_locale(self.get_locale())
    }

    /// Return the keys for the current locale but does not subscribe to changes
    #[inline]
    pub fn get_keys_untracked(self) -> &'static S::Keys {
        LocaleKeys::from_locale(self.get_locale_untracked())
    }

    /// Set the locale and notify all subscribers
    #[inline]
    pub fn set_locale(self, lang: L) {
        self.locale_signal.set(lang)
    }

    /// Set the locale but does not notify the subscribers
    #[inline]
    pub fn set_locale_untracked(self, lang: L) {
        self.locale_signal.set_untracked(lang)
    }

    /// Map the context to a new scope
    #[inline]
    pub const fn scope<NS: Scope<L>>(self, scope: ConstScope<L, NS>) -> I18nContext<L, NS> {
        let _ = scope;
        I18nContext {
            locale_signal: self.locale_signal,
            scope_marker: PhantomData,
        }
    }
}

fn set_html_lang_attr(lang: impl Into<TextProp>) {
    Html(HtmlProps {
        lang: Some(lang.into()),
        dir: None,
        class: None,
        attributes: vec![],
    });
}

pub type CookieOptions<T> = UseCookieOptions<
    T,
    <FromToStringCodec as codee::Encoder<T>>::Error,
    <FromToStringCodec as codee::Decoder<T>>::Error,
>;

enum HtmlOrNodeRef<El: ElementDescriptor + 'static> {
    Html,
    Custom(NodeRef<El>),
}

fn init_context_with_options<T: Locale, El: ElementDescriptor + 'static + Clone>(
    root_element: Option<HtmlOrNodeRef<El>>,
    enable_cookie: bool,
    cookie_name: &str,
    cookie_options: CookieOptions<T>,
) -> I18nContext<T> {
    provide_meta_context();

    let (lang_cookie, set_lang_cookie) = if ENABLE_COOKIE && enable_cookie {
        leptos_use::use_cookie_with_options::<T, FromToStringCodec>(cookie_name, cookie_options)
    } else {
        let (lang_cookie, set_lang_cookie) = create_signal::<Option<T>>(None);
        (lang_cookie.into(), set_lang_cookie)
    };

    let locale = fetch_locale::fetch_locale(lang_cookie.get_untracked());

    let locale = create_rw_signal(locale);

    let node_ref = match root_element {
        Some(HtmlOrNodeRef::Html) => {
            set_html_lang_attr(move || locale.get().as_str());
            NodeRef::new()
        }
        Some(HtmlOrNodeRef::Custom(node_ref)) => node_ref,
        None => NodeRef::new(),
    };

    create_isomorphic_effect(move |_| {
        let new_lang = locale.get();
        if let Some(el) = node_ref.get() {
            let _ = el.attr("lang", new_lang.as_str());
        }
        set_lang_cookie.set(Some(new_lang));
    });

    let context = I18nContext::<T> {
        locale_signal: locale,
        scope_marker: PhantomData,
    };

    provide_context(context);

    context
}

const ENABLE_COOKIE: bool = cfg!(feature = "cookie");

const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

const MAX_COOKIE_AGE_MS: i64 = 399 * 24 * 60 * 60 * 1000; // 399 days in millis

/// Provide the `I18nContext` for the application.
///
/// This function must be called at the highest possible level of the application.
///
/// It returns the newly created context.
///
/// If called when a context is already present it will not overwrite it and just return the current context.
pub fn provide_i18n_context<T: Locale>() -> I18nContext<T> {
    provide_i18n_context_with_options(None, None, None)
}

/// Same as `provide_i18n_context` but with more options about the cookies usage.
///
/// If called when a context is already present it will not overwrite it nor the options and just return the current one.
pub fn provide_i18n_context_with_options<T: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<T>>,
) -> I18nContext<T> {
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(COOKIE_PREFERED_LANG);
    let cookie_options =
        cookie_options.unwrap_or_else(|| CookieOptions::default().max_age(MAX_COOKIE_AGE_MS));

    use_context().unwrap_or_else(move || {
        init_context_with_options(
            Some(HtmlOrNodeRef::<AnyElement>::Html),
            enable_cookie,
            cookie_name,
            cookie_options,
        )
    })
}

/// Same as `provide_i18n_context_with_options` but with an additional argument for the root node
///
/// The root node is the one getting the `"lang"` attribute.
///
/// If called when a context is already present it will not overwrite it nor the options and just return the current one.
pub fn provide_i18n_context_with_options_and_root_element<
    T: Locale,
    El: ElementDescriptor + 'static + Clone,
>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<T>>,
    root_element: NodeRef<El>,
) -> I18nContext<T> {
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(COOKIE_PREFERED_LANG);
    let cookie_options =
        cookie_options.unwrap_or_else(|| CookieOptions::default().max_age(MAX_COOKIE_AGE_MS));

    use_context().unwrap_or_else(move || {
        init_context_with_options(
            Some(HtmlOrNodeRef::Custom(root_element)),
            enable_cookie,
            cookie_name,
            cookie_options,
        )
    })
}

/// Return the `I18nContext` previously set.
///
/// ## Panic
///
/// Panics if the context is missing.
#[inline]
pub fn use_i18n_context<T: Locale>() -> I18nContext<T> {
    use_context().expect("I18nContext is missing, use provide_i18n_context() to provide it.")
}

// get locale
#[cfg(feature = "nightly")]
impl<T: Locale> FnOnce<()> for I18nContext<T> {
    type Output = T;
    #[inline]
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale> FnMut<()> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale> Fn<()> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call(&self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

// set locale
#[cfg(feature = "nightly")]
impl<T: Locale> FnOnce<(T,)> for I18nContext<T> {
    type Output = ();
    #[inline]
    extern "rust-call" fn call_once(self, (locale,): (T,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale> FnMut<(T,)> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, (locale,): (T,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale> Fn<(T,)> for I18nContext<T> {
    #[inline]
    extern "rust-call" fn call(&self, (locale,): (T,)) -> Self::Output {
        self.set_locale(locale)
    }
}
