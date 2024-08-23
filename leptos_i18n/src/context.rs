//! This module contains the `I18nContext` and helpers for it.

use codee::string::FromToStringCodec;
use core::marker::PhantomData;
use tachys::{html, reactive_graph::OwnedView};
// use html::{AnyElement, ElementType};
use leptos::{
    html::{ElementType, Html},
    prelude::*,
    text_prop::TextProp,
};
// use leptos_meta::*;
use leptos_use::UseCookieOptions;

use crate::{
    fetch_locale::{self, signal_maybe_once_then},
    locale_traits::*,
    scopes::ConstScope,
    Scope,
};

/// This context is the heart of the i18n system:
///
/// It servers as a signal to the current locale and enable reactivity to locale change.
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
        let mut guard = self.locale_signal.write_untracked();
        *guard = lang;
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

fn set_html_lang_attr(_lang: impl Into<TextProp>) {
    // Html(HtmlProps {
    //     lang: Some(lang.into()),
    //     dir: None,
    //     class: None,
    //     attributes: vec![],
    // });
}

/// Cookies options for functions initializing or providing a `I18nContext`
pub type CookieOptions<L> = UseCookieOptions<
    L,
    <FromToStringCodec as codee::Encoder<L>>::Error,
    <FromToStringCodec as codee::Decoder<L>>::Error,
>;

enum HtmlOrNodeRef<El: ElementType + 'static> {
    Html,
    Custom(NodeRef<El>),
}

const ENABLE_COOKIE: bool = cfg!(feature = "cookie");

const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

fn init_context_inner<L, El>(
    root_element: Option<HtmlOrNodeRef<El>>,
    set_lang_cookie: WriteSignal<Option<L>>,
    initial_locale: Memo<L>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    let locale_signal = RwSignal::new(L::default());

    Effect::new_isomorphic(move |_| {
        locale_signal.set(initial_locale.get());
    });

    let _node_ref = match root_element {
        Some(HtmlOrNodeRef::Html) => {
            set_html_lang_attr(move || locale_signal.get().as_str());
            NodeRef::new()
        }
        Some(HtmlOrNodeRef::Custom(node_ref)) => node_ref,
        None => NodeRef::new(),
    };

    Effect::new_isomorphic(move |_| {
        let new_lang = locale_signal.get();
        // TODO
        // if let Some(el) = node_ref.get() {
        //     // let _ = el.attr("lang", new_lang.as_str());
        // }
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

fn init_context_with_options<L, El>(
    root_element: Option<HtmlOrNodeRef<El>>,
    enable_cookie: bool,
    cookie_name: &str,
    cookie_options: CookieOptions<L>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    let (lang_cookie, set_lang_cookie) = if ENABLE_COOKIE && enable_cookie {
        leptos_use::use_cookie_with_options::<L, FromToStringCodec>(cookie_name, cookie_options)
    } else {
        let (lang_cookie, set_lang_cookie) = RwSignal::<Option<L>>::new(None).split();
        (lang_cookie.into(), set_lang_cookie)
    };

    let initial_locale = fetch_locale::fetch_locale(lang_cookie.get_untracked());

    init_context_inner::<L, El>(root_element, set_lang_cookie, initial_locale)
}

/// Same as `init_i18n_context` but with some cookies options.
pub fn init_i18n_context_with_options<L: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
) -> I18nContext<L> {
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(COOKIE_PREFERED_LANG);
    init_context_with_options::<L, Html>(
        Some(HtmlOrNodeRef::Html),
        enable_cookie,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Same as `init_i18n_context` but takes a root element to bind the `"lang"` HTML attribute.
/// (That attribute will not be set on ssr, only on the client.)
pub fn init_i18n_context_with_root<L, El>(root_element: NodeRef<El>) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    init_i18n_context_with_options_and_root::<L, El>(None, None, None, root_element)
}

/// Same as `init_i18n_context` but with some cookies options and a root element to bind the `"lang"` HTML attribute.
/// (That attribute will not be set on ssr, only on the client.)
pub fn init_i18n_context_with_options_and_root<L, El>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(COOKIE_PREFERED_LANG);
    init_context_with_options::<L, El>(
        Some(HtmlOrNodeRef::Custom(root_element)),
        enable_cookie,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Initialize a `I18nContext` without providing it.
pub fn init_i18n_context<L: Locale>() -> I18nContext<L> {
    init_i18n_context_with_options::<L>(None, None, None)
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
        let ctx = init_i18n_context::<L>();
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
        let ctx = init_i18n_context_with_options::<L>(enable_cookie, cookie_name, cookie_options);
        provide_context(ctx);
        ctx
    })
}

/// Same as `provide_i18n_context`  but takes a root element to bind the `"lang"` HTML attribute.
pub fn provide_i18n_context_with_root<L, El>(root_element: NodeRef<El>) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    use_context().unwrap_or_else(move || {
        let ctx = init_i18n_context_with_root::<L, El>(root_element);
        provide_context(ctx);
        ctx
    })
}

/// Same as `provide_i18n_context`  but with some cookies options and a root element to bind the `"lang"` HTML attribute.
/// (That attribute will not be set on ssr, only on the client.)
pub fn provide_i18n_context_with_options_and_root<L, El>(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    use_context().unwrap_or_else(move || {
        let ctx = init_i18n_context_with_options_and_root::<L, El>(
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

fn init_subcontext_with_options<L, El>(
    root_element: Option<NodeRef<El>>,
    initial_locale: Signal<Option<L>>,
    cookie_name: Option<&str>,
    cookie_options: CookieOptions<L>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    let (lang_cookie, set_lang_cookie) = match cookie_name {
        Some(cookie_name) if ENABLE_COOKIE => {
            leptos_use::use_cookie_with_options::<L, FromToStringCodec>(cookie_name, cookie_options)
        }
        _ => {
            let (lang_cookie, set_lang_cookie) = RwSignal::<Option<L>>::new(None).split();
            (lang_cookie.into(), set_lang_cookie)
        }
    };

    let fetch_locale_memo = fetch_locale::fetch_locale(None);

    let parent_locale = use_context::<I18nContext<L>>().map(|ctx| ctx.get_locale_untracked());

    let parent_locale = signal_maybe_once_then(parent_locale, fetch_locale_memo);

    let initial_locale_listener = Memo::new(move |prev_locale| {
        let initial_locale = initial_locale.get();
        let cookie = lang_cookie.get_untracked();
        let parent_locale = parent_locale.get();
        // first execution, cookie takes precedence
        if prev_locale.is_none() {
            cookie.or(initial_locale).unwrap_or(parent_locale)
        } else {
            // triggers if initial_locale updates, so it takes precedence here
            initial_locale.or(cookie).unwrap_or(parent_locale)
        }
    });

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
/// - `initial_locale` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext_with_options<L: Locale>(
    initial_locale: Option<Signal<L>>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
) -> I18nContext<L> {
    let initial_locale = derive_initial_locale_signal(initial_locale);

    init_subcontext_with_options::<L, html::element::Html>(
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
/// (That attribute will not be set on ssr, only on the client.)
///
/// The locale to init the subcontext with is determined in this order:
/// - locale in the cookie
/// - `initial_locale` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext_with_options_and_root<L, El>(
    initial_locale: Option<Signal<L>>,
    cookie_name: Option<&str>,
    cookie_options: Option<CookieOptions<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    let initial_locale = derive_initial_locale_signal(initial_locale);

    init_subcontext_with_options::<L, El>(
        Some(root_element),
        initial_locale,
        cookie_name,
        cookie_options.unwrap_or_default(),
    )
}

/// Same as `init_i18n_subcontext` but with some options
///
/// The `root_element` is a `NodeRef` to an element that will receive the HTML `"lang"` attribute.
/// (That attribute will not be set on ssr, only on the client.)
///
/// The locale to init the subcontext with is determined in this order:
/// - `initial_locale` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext_with_root<L, El>(
    initial_locale: Option<Signal<L>>,
    root_element: NodeRef<El>,
) -> I18nContext<L>
where
    L: Locale,
    El: ElementType + 'static + Clone,
{
    init_i18n_subcontext_with_options_and_root::<L, El>(initial_locale, None, None, root_element)
}

/// Initialize a `I18nContext` subcontext without providing it.
///
/// Can be supplied with a initial locale to use for this subcontext
///
/// The locale to init the subcontext with is determined in this order:
/// - `initial_locale` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
pub fn init_i18n_subcontext<L: Locale>(initial_locale: Option<Signal<L>>) -> I18nContext<L> {
    init_i18n_subcontext_with_options::<L>(initial_locale, None, None)
}

/// This function should not be used, it is only there to serves as documentation point.
/// It is marked as `deprecated` to discourage users from using it.
///
/// # Warning: Shadowing correctly
///
/// There is a section on `leptos::provide_context` about shadowing, it is easy to screw it up.
/// This is why you should be careful about using this function.
///
/// The recommended way is to use the `I18nSubContextProvider`.
///
/// Or you can create a subcontext with `init_i18n_subcontext_*` and manually provide it with `Provider` or `provide_context`.
#[deprecated = "see function documentation"]
pub fn provide_i18n_subcontext<L: Locale, Rndr: Renderer>(
    initial_locale: Option<Signal<L>>,
) -> I18nContext<L> {
    let ctx = init_i18n_subcontext::<L>(initial_locale);
    provide_context(ctx);
    ctx
}

fn run_as_children<L: Locale, Chil: IntoView>(
    ctx: I18nContext<L>,
    children: TypedChildren<Chil>,
) -> impl IntoView {
    let owner = owner::Owner::current()
        .expect("no current reactive Owner found")
        .child();
    let children = children.into_inner();
    let children = owner.with(|| {
        provide_context(ctx);
        children()
    });
    OwnedView::new_with_owner(children, owner)
}

/// Create and provide a subcontext for all children components, directly accessible with `use_i18n`.
#[component]
#[allow(non_snake_case)]
pub fn I18nSubContextProvider<L: Locale, Chil: IntoView>(
    children: TypedChildren<Chil>,
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
    run_as_children(ctx, children)
}

/// Create and provide a subcontext for all children components, directly accessible with `use_i18n`.
///
/// Like `I18nSubContextProvider` but can be given a `root_element: NodeRef` to set the `"lang"` attribute on.
/// (That attribute will not be set on ssr, only on the client.)
#[leptos::component]
#[allow(non_snake_case)]
pub fn I18nSubContextProviderWithRoot<L, El, Chil>(
    children: TypedChildren<Chil>,
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
    El: ElementType + 'static + Clone,
    Chil: IntoView,
{
    let ctx = init_i18n_subcontext_with_options_and_root::<L, El>(
        initial_locale,
        cookie_name,
        cookie_options,
        root_element,
    );
    run_as_children(ctx, children)
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
impl<T: Locale, S: Scope<L>> FnOnce<()> for I18nContext<T, S> {
    type Output = T;
    #[inline]
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale, S: Scope<L>> FnMut<()> for I18nContext<T, S> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale, S: Scope<L>> Fn<()> for I18nContext<T, S> {
    #[inline]
    extern "rust-call" fn call(&self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

// set locale
#[cfg(feature = "nightly")]
impl<T: Locale, S: Scope<L>> FnOnce<(T,)> for I18nContext<T, S> {
    type Output = ();
    #[inline]
    extern "rust-call" fn call_once(self, (locale,): (T,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale, S: Scope<L>> FnMut<(T,)> for I18nContext<T, S> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, (locale,): (T,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<T: Locale, S: Scope<L>> Fn<(T,)> for I18nContext<T, S> {
    #[inline]
    extern "rust-call" fn call(&self, (locale,): (T,)) -> Self::Output {
        self.set_locale(locale)
    }
}
