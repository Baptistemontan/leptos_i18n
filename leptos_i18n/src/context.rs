//! This module contains the `I18nContext` and helpers for it.

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

/// Cookies options for functions initializing or providing a `I18nContext`
pub type CookieOptions<L> = UseCookieOptions<
    L,
    <FromToStringCodec as codee::Encoder<L>>::Error,
    <FromToStringCodec as codee::Decoder<L>>::Error,
>;

enum HtmlOrNodeRef<El: ElementDescriptor + 'static> {
    Html,
    Custom(NodeRef<El>),
}

const ENABLE_COOKIE: bool = cfg!(feature = "cookie");

const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

fn init_context_inner<L: Locale, El: ElementDescriptor + 'static + Clone>(
    root_element: Option<HtmlOrNodeRef<El>>,
    set_lang_cookie: WriteSignal<Option<L>>,
    initial_locale: impl Fn(Option<L>) -> L + 'static,
) -> I18nContext<L> {
    let locale_signal = create_rw_signal(L::default());

    create_isomorphic_effect(move |old_locale| {
        let new_locale = initial_locale(old_locale);
        locale_signal.set(new_locale);
        new_locale
    });

    let node_ref = match root_element {
        Some(HtmlOrNodeRef::Html) => {
            set_html_lang_attr(move || locale_signal.get().as_str());
            NodeRef::new()
        }
        Some(HtmlOrNodeRef::Custom(node_ref)) => node_ref,
        None => NodeRef::new(),
    };

    create_isomorphic_effect(move |_| {
        let new_lang = locale_signal.get();
        if let Some(el) = node_ref.get() {
            let _ = el.attr("lang", new_lang.as_str());
        }
        set_lang_cookie.set(Some(new_lang));
    });

    I18nContext::<L> {
        locale_signal,
        scope_marker: PhantomData,
    }
}

/// *********************************************
/// * CONTEXT
/// *********************************************

fn init_context_with_options<L: Locale, El: ElementDescriptor + 'static + Clone>(
    root_element: Option<HtmlOrNodeRef<El>>,
    enable_cookie: bool,
    cookie_name: &str,
    cookie_options: CookieOptions<L>,
) -> I18nContext<L> {
    let (lang_cookie, set_lang_cookie) = if ENABLE_COOKIE && enable_cookie {
        leptos_use::use_cookie_with_options::<L, FromToStringCodec>(cookie_name, cookie_options)
    } else {
        let (lang_cookie, set_lang_cookie) = create_signal::<Option<L>>(None);
        (lang_cookie.into(), set_lang_cookie)
    };

    let initial_locale = fetch_locale::fetch_locale(lang_cookie.get_untracked());

    init_context_inner::<L, El>(root_element, set_lang_cookie, move |_| initial_locale)
}

/// Same as `init_i18n_context` but with some cookies options.
pub fn init_i18n_context_with_options<L: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
) -> I18nContext<L> {
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(COOKIE_PREFERED_LANG);
    init_context_with_options(
        Some(HtmlOrNodeRef::<AnyElement>::Html),
        enable_cookie,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Same as `init_i18n_context` but with some cookies options and a root element to bind the `"lang"` HTML attribute.
pub fn init_i18n_context_with_options_and_root<
    L: Locale,
    El: ElementDescriptor + 'static + Clone,
>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L> {
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(COOKIE_PREFERED_LANG);
    init_context_with_options(
        Some(HtmlOrNodeRef::Custom(root_element)),
        enable_cookie,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Initialize a `I18nContext` without providing it.
pub fn init_i18n_context<L: Locale>() -> I18nContext<L> {
    init_i18n_context_with_options(None, None, None)
}

/// Initialize and provide a `I18nContext`.
///
/// This function must be called at the highest possible level of the application.
///
/// It returns the newly created context.
///
/// If called when a context is already present it will not overwrite it and just return the current context.
pub fn provide_i18n_context<L: Locale>() -> I18nContext<L> {
    use_context().unwrap_or_else(|| {
        let ctx = init_i18n_context();
        provide_context(ctx);
        ctx
    })
}

/// Same as `provide_i18n_context`  but with some cookies options.
pub fn provide_i18n_context_with_options<L: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
) -> I18nContext<L> {
    use_context().unwrap_or_else(move || {
        let ctx = init_i18n_context_with_options(enable_cookie, cookie_name, cookie_options);
        provide_context(ctx);
        ctx
    })
}

/// Same as `provide_i18n_context`  but with some cookies options and a root element to bind the `"lang"` HTML attribute.
pub fn provide_i18n_context_with_options_and_root<
    L: Locale,
    El: ElementDescriptor + 'static + Clone,
>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L> {
    use_context().unwrap_or_else(move || {
        let ctx = init_i18n_context_with_options_and_root(
            enable_cookie,
            cookie_name,
            cookie_options,
            root_element,
        );
        provide_context(ctx);
        ctx
    })
}

/// *********************************************
/// * SUB CONTEXT
/// *********************************************

fn init_subcontext_with_options<L: Locale, El: ElementDescriptor + 'static + Clone>(
    root_element: Option<NodeRef<El>>,
    initial_locale: Signal<Option<L>>,
    cookie_name: Option<&str>,
    cookie_options: CookieOptions<L>,
) -> I18nContext<L> {
    let (lang_cookie, set_lang_cookie) = match cookie_name {
        Some(cookie_name) if ENABLE_COOKIE => {
            leptos_use::use_cookie_with_options::<L, FromToStringCodec>(cookie_name, cookie_options)
        }
        _ => {
            let (lang_cookie, set_lang_cookie) = create_signal::<Option<L>>(None);
            (lang_cookie.into(), set_lang_cookie)
        }
    };

    let parent_locale = use_context::<I18nContext<L>>()
        .map(|ctx| ctx.get_locale_untracked())
        .unwrap_or_else(|| fetch_locale::fetch_locale(None));

    let initial_locale_listener = move |prev_locale: Option<L>| {
        let initial_locale = initial_locale.get();
        let cookie = lang_cookie.get_untracked();
        // first execution, cookie takes precedence
        if prev_locale.is_none() {
            cookie.or(initial_locale).unwrap_or(parent_locale)
        } else {
            // triggers if initial_locale updates, so it takes precedence here
            initial_locale.or(cookie).unwrap_or(parent_locale)
        }
    };

    let root_element = root_element.map(HtmlOrNodeRef::Custom);

    init_context_inner::<L, El>(root_element, set_lang_cookie, initial_locale_listener)
}

fn derive_initial_locale_signal<L: Locale>(initial_locale: Option<Signal<L>>) -> Signal<Option<L>> {
    initial_locale
        .map(|s| Signal::derive(move || Some(s.get())))
        .unwrap_or_default()
}

/// Same as `init_i18n_subcontext` but with some options.
///
/// The `cookie_name` option make it possible to save the locale in a cookie of the given name (does nothing without the `cookie` feature).
/// If none no cookie will be set.
///
/// The locale to init the subcontext with is determined in this order:
/// - locale in the cookie
/// - `initial_cookie` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext_with_options<L: Locale>(
    initial_locale: Option<Signal<L>>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
) -> I18nContext<L> {
    let initial_locale = derive_initial_locale_signal(initial_locale);

    init_subcontext_with_options::<L, AnyElement>(
        None,
        initial_locale,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Same as `init_i18n_subcontext` but with some options
///
/// The `cookie_name` option make it possible to save the locale in a cookie of the given name (does nothing without the `cookie` feature).
/// If none no cookie will be set.
///
/// The `root_element` is a `NodeRef` to an element that will receive the HTML `"lang"` attribute.
///
/// The locale to init the subcontext with is determined in this order:
/// - locale in the cookie
/// - `initial_cookie` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext_with_options_and_root<
    L: Locale,
    El: ElementDescriptor + 'static + Clone,
>(
    initial_locale: Option<Signal<L>>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L> {
    let initial_locale = derive_initial_locale_signal(initial_locale);

    init_subcontext_with_options(
        Some(root_element),
        initial_locale,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Initialize a `I18nContext` subcontext without providing it.
///
/// Can be supplied with a initial locale to use for this subcontext
///
/// The locale to init the subcontext with is determined in this order:
/// - `initial_cookie` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext<L: Locale>(initial_locale: Option<Signal<L>>) -> I18nContext<L> {
    init_i18n_subcontext_with_options(initial_locale, None, None)
}

/// This function should not be used, it is only there to serves as documentation point.
/// It is marked as `deprecated` to discourage users from using it.
///
/// # Warning: Shadowing correctly
///
/// There is a section on `leptos::provide_context` about shadowing, it is easy to screw it up.
/// This is why you should be carefull about using this function.
///
/// The recommanded way is to use the `I18nSubContextProvider` generated with the `i18n` module.
///
/// Or you can create a subcontext with `init_i18n_subcontext_*` and manually provide it with `Provider` or `provide_context`.
#[deprecated = "see function documentation"]
pub fn provide_i18n_subcontext<L: Locale>(initial_locale: Option<Signal<L>>) -> I18nContext<L> {
    let ctx = init_i18n_subcontext(initial_locale);
    provide_context(ctx);
    ctx
}

#[component]
#[allow(non_snake_case)]
pub fn I18nSubContextProvider<L: Locale>(
    children: Children,
    /// The initial locale for this subcontext.
    /// Default to the locale set in the cookie if set and some,
    /// if not use the parent context locale.
    /// if no parent context, use the default locale.
    #[prop(optional, into)]
    initial_locale: Option<Signal<L>>,
    /// If set save the locale in a cookie of the given name (does nothing without the `cookie` feature).
    #[prop(optional)]
    cookie_name: Option<&'static str>,
    /// Options for the cookie.
    #[prop(optional)]
    cookie_options: Option<CookieOptions<L>>,
) -> impl IntoView {
    let ctx = init_i18n_subcontext_with_options::<L>(initial_locale, cookie_name, cookie_options);
    leptos::run_as_child(move || {
        provide_context(ctx);
        children()
    })
}

#[leptos::component]
#[allow(non_snake_case)]
pub fn I18nSubContextProviderWithRoot<L, El>(
    children: Children,
    /// The initial locale for this subcontext.
    /// Default to the locale set in the cookie if set and some,
    /// if not use the parent context locale.
    /// if no parent context, use the default locale.
    #[prop(optional, into)]
    initial_locale: Option<Signal<L>>,
    /// If set save the locale in a cookie of the given name (does nothing without the `cookie` feature).
    #[prop(optional)]
    cookie_name: Option<&'static str>,
    /// Options for the cookie.
    #[prop(optional)]
    cookie_options: Option<CookieOptions<L>>,
    /// A `NodeRef` to an element that will receive the HTML `"lang"` attribute.
    root_element: NodeRef<El>,
) -> impl IntoView
where
    L: Locale,
    El: ElementDescriptor + 'static + Clone,
{
    let ctx = init_i18n_subcontext_with_options_and_root::<L, El>(
        initial_locale,
        cookie_name,
        cookie_options,
        root_element,
    );
    leptos::run_as_child(move || {
        provide_context(ctx);
        children()
    })
}

/// Return the `I18nContext` previously set.
///
/// ## Panic
///
/// Panics if the context is missing.
#[inline]
pub fn use_i18n_context<L: Locale>() -> I18nContext<L> {
    use_context().expect("I18n context is missing")
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
