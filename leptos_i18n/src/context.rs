//! This module contains the `I18nContext` and helpers for it.

use codee::string::FromToStringCodec;
use core::marker::PhantomData;
use leptos::{children, either::Either, prelude::*};
use leptos_meta::{provide_meta_context, Html};
use leptos_use::UseCookieOptions;
use std::borrow::Cow;
use tachys::{html::directive::IntoDirective, reactive_graph::OwnedView, view::any_view::AnyView};

use crate::{
    fetch_locale::{self, signal_maybe_once_then},
    locale_traits::*,
    scopes::ConstScope,
    Scope,
};

pub use leptos_use::UseLocalesOptions;

/// This context is the heart of the i18n system:
///
/// It servers as a signal to the current locale and enable reactivity to locale change.
///
/// You access the translations and read/update the current locale through it.
#[derive(Debug)]
pub struct I18nContext<L: Locale, S: Scope<L> = <L as Locale>::Keys> {
    locale_signal: RwSignal<L>,
    scope_marker: PhantomData<S>,
}

impl<L: Locale, S: Scope<L>> Clone for I18nContext<L, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Locale, S: Scope<L>> Copy for I18nContext<L, S> {}

impl<L: Locale, S: Scope<L>> I18nContext<L, S> {
    /// Return the current locale subscribing to any changes.
    #[inline]
    #[track_caller]
    pub fn get_locale(self) -> L {
        self.locale_signal.get()
    }

    /// Return the current locale but does not subscribe to changes
    #[inline]
    #[track_caller]
    pub fn get_locale_untracked(self) -> L {
        self.locale_signal.get_untracked()
    }

    /// Return the keys for the current locale subscribing to any changes
    #[inline]
    #[track_caller]
    pub fn get_keys(self) -> S::Keys {
        LocaleKeys::from_locale(self.get_locale())
    }

    /// Return the keys for the current locale but does not subscribe to changes
    #[inline]
    #[track_caller]
    pub fn get_keys_untracked(self) -> S::Keys {
        LocaleKeys::from_locale(self.get_locale_untracked())
    }

    /// Set the locale and notify all subscribers
    #[inline]
    #[track_caller]
    pub fn set_locale(self, lang: L) {
        self.locale_signal.set(lang)
    }

    /// Set the locale but does not notify the subscribers
    #[inline]
    #[track_caller]
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

impl<L: Locale, S: Scope<L>, R: Renderer> IntoDirective<(R::Element,), (), R>
    for I18nContext<L, S>
{
    type Cloneable = Self;

    fn run(&self, el: <R as Renderer>::Element, _param: ()) {
        let this = *self;
        Effect::new(move || {
            let locale = this.get_locale();
            R::set_attribute(&el, "lang", locale.as_str());
        });
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }
}

/// Cookies options for functions initializing or providing a `I18nContext`
pub type CookieOptions<L> = UseCookieOptions<
    L,
    <FromToStringCodec as codee::Encoder<L>>::Error,
    <FromToStringCodec as codee::Decoder<L>>::Error,
>;

const ENABLE_COOKIE: bool = cfg!(feature = "cookie");

const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

#[track_caller]
fn init_context_inner<L: Locale>(
    set_lang_cookie: WriteSignal<Option<L>>,
    initial_locale: Memo<L>,
) -> I18nContext<L> {
    let locale_signal = RwSignal::new(initial_locale.get_untracked());

    Effect::new(move |_| {
        locale_signal.set(initial_locale.get());
    });

    Effect::new_isomorphic(move |_| {
        let new_lang = locale_signal.get();
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

#[track_caller]
fn init_context_with_options<L: Locale>(
    enable_cookie: bool,
    cookie_name: Cow<str>,
    cookie_options: CookieOptions<L>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> I18nContext<L> {
    let (lang_cookie, set_lang_cookie) = if ENABLE_COOKIE && enable_cookie {
        leptos_use::use_cookie_with_options::<L, FromToStringCodec>(&cookie_name, cookie_options)
    } else {
        let (lang_cookie, set_lang_cookie) = signal(None);
        (lang_cookie.into(), set_lang_cookie)
    };

    let initial_locale = fetch_locale::fetch_locale(
        lang_cookie.get_untracked(),
        ssr_lang_header_getter.unwrap_or_default(),
    );

    init_context_inner::<L>(set_lang_cookie, initial_locale)
}

/// Same as `init_i18n_context` but with some cookies options.
#[track_caller]
pub fn init_i18n_context_with_options<L: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> I18nContext<L> {
    let enable_cookie = enable_cookie.unwrap_or(ENABLE_COOKIE);
    let cookie_name = cookie_name.unwrap_or(Cow::Borrowed(COOKIE_PREFERED_LANG));
    init_context_with_options(
        enable_cookie,
        cookie_name,
        cookie_options.unwrap_or_default(),
        ssr_lang_header_getter,
    )
}

/// Initialize a `I18nContext` without providing it.
#[track_caller]
pub fn init_i18n_context<L: Locale>() -> I18nContext<L> {
    init_i18n_context_with_options(None, None, None, None)
}

/// Initialize and provide a `I18nContext`.
///
/// This function must be called at the highest possible level of the application.
///
/// It returns the newly created context.
///
/// If called when a context is already present it will not overwrite it and just return the current context.
#[deprecated(
    note = "It is now preferred to use the <I18nContextProvider> component in the generated i18n module."
)]
#[track_caller]
pub fn provide_i18n_context<L: Locale>() -> I18nContext<L> {
    use_context().unwrap_or_else(|| {
        let ctx = init_i18n_context::<L>();
        provide_context(ctx);
        ctx
    })
}

#[doc(hidden)]
#[track_caller]
pub fn provide_i18n_context_with_options_inner<L: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> I18nContext<L> {
    provide_meta_context();
    use_context().unwrap_or_else(move || {
        let ctx = init_i18n_context_with_options(
            enable_cookie,
            cookie_name,
            cookie_options,
            ssr_lang_header_getter,
        );
        provide_context(ctx);
        ctx
    })
}

/// Same as `provide_i18n_context`  but with some cookies options.
#[deprecated(
    note = "It is now preferred to use the <I18nContextProvider> component in the generated i18n module."
)]
#[track_caller]
pub fn provide_i18n_context_with_options<L: Locale>(
    enable_cookie: Option<bool>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> I18nContext<L> {
    provide_i18n_context_with_options_inner(
        enable_cookie,
        cookie_name,
        cookie_options,
        ssr_lang_header_getter,
    )
}

/// *********************************************
/// * SUB CONTEXT
/// *********************************************

#[track_caller]
fn init_subcontext_with_options<L: Locale>(
    initial_locale: Signal<Option<L>>,
    cookie_name: Option<Cow<str>>,
    cookie_options: CookieOptions<L>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> I18nContext<L> {
    let (lang_cookie, set_lang_cookie) = match cookie_name {
        Some(cookie_name) if ENABLE_COOKIE => leptos_use::use_cookie_with_options::<
            L,
            FromToStringCodec,
        >(&cookie_name, cookie_options),
        _ => {
            let (lang_cookie, set_lang_cookie) = signal(None);
            (lang_cookie.into(), set_lang_cookie)
        }
    };

    let fetch_locale_memo =
        fetch_locale::fetch_locale(None, ssr_lang_header_getter.unwrap_or_default());

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

    init_context_inner::<L>(set_lang_cookie, initial_locale_listener)
}

#[track_caller]
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
#[track_caller]
pub fn init_i18n_subcontext_with_options<L: Locale>(
    initial_locale: Option<Signal<L>>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> I18nContext<L> {
    let initial_locale = derive_initial_locale_signal(initial_locale);

    init_subcontext_with_options::<L>(
        initial_locale,
        cookie_name,
        cookie_options.unwrap_or_default(),
        ssr_lang_header_getter,
    )
}

/// Initialize a `I18nContext` subcontext without providing it.
///
/// Can be supplied with a initial locale to use for this subcontext
///
/// The locale to init the subcontext with is determined in this order:
/// - `initial_locale` if set
/// - locale of the parent context
/// - if no parent context, use the same resolution used by a main context.
#[track_caller]
pub fn init_i18n_subcontext<L: Locale>(initial_locale: Option<Signal<L>>) -> I18nContext<L> {
    init_i18n_subcontext_with_options::<L>(initial_locale, None, None, None)
}

/// This function should not be used, it is only there to serves as documentation point.
/// It is marked as `deprecated` to discourage users from using it.
///
/// # Warning: Shadowing correctly
///
/// There is a section on [`leptos::provide_context`] about shadowing, it is easy to screw it up.
/// This is why you should be careful about using this function.
///
/// The recommended way is to use the `I18nSubContextProvider`.
///
/// Or you can create a subcontext with `init_i18n_subcontext_*` and manually provide it with `Provider` or `provide_context`.
#[deprecated = "see function documentation"]
#[track_caller]
pub fn provide_i18n_subcontext<L: Locale>(initial_locale: Option<Signal<L>>) -> I18nContext<L> {
    let ctx = init_i18n_subcontext::<L>(initial_locale);
    provide_context(ctx);
    ctx
}

fn run_as_children<L: Locale, Chil: IntoView>(
    ctx: I18nContext<L>,
    children: impl FnOnce() -> Chil,
) -> impl IntoView {
    let owner = Owner::current()
        .expect("no current reactive Owner found")
        .child();
    let children = owner.with(|| {
        provide_context(ctx);
        children()
    });
    OwnedView::new_with_owner(children, owner)
}

#[doc(hidden)]
#[track_caller]
pub fn i18n_sub_context_provider_inner<L: Locale, Chil: IntoView>(
    children: TypedChildren<Chil>,
    initial_locale: Option<Signal<L>>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
) -> impl IntoView {
    let ctx = init_i18n_subcontext_with_options::<L>(
        initial_locale,
        cookie_name,
        cookie_options,
        ssr_lang_header_getter,
    );
    run_as_children(ctx, children.into_inner())
}

#[doc(hidden)]
#[track_caller]
pub fn i18n_sub_context_provider_island<L: Locale>(
    children: children::Children,
    initial_locale: Option<L>,
    cookie_name: Option<Cow<str>>,
) -> impl IntoView {
    let initial_locale = initial_locale.map(|l| Signal::derive(move || l));
    let ctx = init_i18n_subcontext_with_options::<L>(initial_locale, cookie_name, None, None);
    run_as_children(ctx, children)
}

/// Return the `I18nContext` previously set.
///
/// ## Panic
///
/// Panics if the context is missing.
#[inline]
#[track_caller]
pub fn use_i18n_context<L: Locale>() -> I18nContext<L> {
    use_context().expect("I18n context is missing")
}

#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
fn embed_translations<L: Locale>(
    reg_ctx: crate::fetch_translations::RegisterCtx<L>,
) -> impl IntoView {
    let translations = reg_ctx.to_array();
    view! {
        <script inner_html=translations />
    }
}

#[track_caller]
fn provide_i18n_context_component_inner<L: Locale, Chil: IntoView>(
    set_lang_attr_on_html: Option<bool>,
    enable_cookie: Option<bool>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
    children: impl FnOnce() -> Chil,
) -> impl IntoView {
    #[cfg(all(feature = "dynamic_load", feature = "hydrate"))]
    crate::fetch_translations::init_translations::<L>();
    #[cfg(all(feature = "dynamic_load", feature = "ssr"))]
    let reg_ctx = crate::fetch_translations::RegisterCtx::<L>::provide_context();
    let i18n = provide_i18n_context_with_options_inner(
        enable_cookie,
        cookie_name,
        cookie_options,
        ssr_lang_header_getter,
    );
    let children = children();
    #[cfg(all(feature = "dynamic_load", feature = "ssr"))]
    let embed_translations = move || embed_translations(reg_ctx.clone());
    #[cfg(not(all(feature = "dynamic_load", feature = "ssr")))]
    let embed_translations = ();
    if set_lang_attr_on_html.unwrap_or(true) {
        let lang = move || i18n.get_locale().as_str();
        Either::Left(view! {
            <Html attr:lang=lang />
            {children}
            {embed_translations}
        })
    } else {
        Either::Right(view! {
            {children}
            {embed_translations}
        })
    }
}

#[doc(hidden)]
#[track_caller]
pub fn provide_i18n_context_component<L: Locale, Chil: IntoView>(
    set_lang_attr_on_html: Option<bool>,
    enable_cookie: Option<bool>,
    cookie_name: Option<Cow<str>>,
    cookie_options: Option<CookieOptions<L>>,
    ssr_lang_header_getter: Option<UseLocalesOptions>,
    children: TypedChildren<Chil>,
) -> impl IntoView {
    provide_i18n_context_component_inner(
        set_lang_attr_on_html,
        enable_cookie,
        cookie_name,
        cookie_options,
        ssr_lang_header_getter,
        children.into_inner(),
    )
}

#[doc(hidden)]
#[track_caller]
pub fn provide_i18n_context_component_island<L: Locale>(
    set_lang_attr_on_html: Option<bool>,
    enable_cookie: Option<bool>,
    cookie_name: Option<Cow<str>>,
    children: children::Children,
) -> impl IntoView {
    provide_i18n_context_component_inner::<L, AnyView<Dom>>(
        set_lang_attr_on_html,
        enable_cookie,
        cookie_name,
        None,
        None,
        children,
    )
}

// get locale
#[cfg(feature = "nightly")]
impl<L: Locale, S: Scope<L>> FnOnce<()> for I18nContext<L, S> {
    type Output = L;
    #[inline]
    #[track_caller]
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<L: Locale, S: Scope<L>> FnMut<()> for I18nContext<L, S> {
    #[inline]
    #[track_caller]
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

#[cfg(feature = "nightly")]
impl<L: Locale, S: Scope<L>> Fn<()> for I18nContext<L, S> {
    #[inline]
    #[track_caller]
    extern "rust-call" fn call(&self, _args: ()) -> Self::Output {
        self.get_locale()
    }
}

// set locale
#[cfg(feature = "nightly")]
impl<L: Locale, S: Scope<L>> FnOnce<(L,)> for I18nContext<L, S> {
    type Output = ();
    #[inline]
    #[track_caller]
    extern "rust-call" fn call_once(self, (locale,): (L,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<L: Locale, S: Scope<L>> FnMut<(L,)> for I18nContext<L, S> {
    #[inline]
    #[track_caller]
    extern "rust-call" fn call_mut(&mut self, (locale,): (L,)) -> Self::Output {
        self.set_locale(locale)
    }
}

#[cfg(feature = "nightly")]
impl<L: Locale, S: Scope<L>> Fn<(L,)> for I18nContext<L, S> {
    #[inline]
    #[track_caller]
    extern "rust-call" fn call(&self, (locale,): (L,)) -> Self::Output {
        self.set_locale(locale)
    }
}
