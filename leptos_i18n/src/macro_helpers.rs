#![doc(hidden)]

use leptos::IntoView;
use std::{borrow::Cow, fmt::Display};

use crate::{
    display::DisplayComponent, scopes::ScopedLocale, ConstScope, I18nContext, Locale, Scope,
};

// Interpolation

pub trait InterpolateVar: IntoView + Clone + 'static {}

impl<T: IntoView + Clone + 'static> InterpolateVar for T {}

pub fn check_var(var: impl InterpolateVar) -> impl InterpolateVar {
    var
}

pub fn check_var_string(var: impl Display) -> impl Display {
    var
}

pub trait InterpolateComp<O: IntoView>: Fn(leptos::ChildrenFn) -> O + Clone + 'static {}

impl<O: IntoView, T: Fn(leptos::ChildrenFn) -> O + Clone + 'static> InterpolateComp<O> for T {}

pub fn check_comp<V: IntoView>(comp: impl InterpolateComp<V>) -> impl InterpolateComp<V> {
    comp
}

pub fn check_comp_string(comp: impl DisplayComponent) -> impl DisplayComponent {
    comp
}

pub trait InterpolateCount<T>: Fn() -> T + Clone + 'static {}

impl<T, F: Fn() -> T + Clone + 'static> InterpolateCount<T> for F {}

pub fn check_count<T>(count: impl InterpolateCount<T>) -> impl InterpolateCount<T> {
    count
}

pub fn check_count_string<T>(count: T) -> T {
    count
}

#[doc(hidden)]
pub struct DisplayBuilder(Cow<'static, str>);

impl DisplayBuilder {
    #[inline]
    pub fn build_display(self) -> Cow<'static, str> {
        self.0
    }
}

/// This is used to call `.build` on `&str` when building interpolations.
///
/// If it's a `&str` it will just return the str,
/// but if it's a builder `.build` will either emit an error for a missing key or if all keys
/// are supplied it will return the correct value
///
/// It has no uses outside of the internals of the `t!` macro.
#[doc(hidden)]
pub trait BuildStr: Sized {
    #[inline]
    fn builder(self) -> Self {
        self
    }

    #[inline]
    fn string_builder(self) -> Self {
        self
    }

    fn display_builder(self) -> DisplayBuilder;

    #[inline]
    fn build(self) -> Self {
        self
    }

    #[inline]
    fn build_string(self) -> Self {
        self
    }
}

impl BuildStr for &'static str {
    #[inline]
    fn display_builder(self) -> DisplayBuilder {
        DisplayBuilder(Cow::Borrowed(self))
    }
}

// Scoping

#[doc(hidden)]
pub const fn scope_ctx_util<L: Locale, OS: Scope<L>, NS: Scope<L>>(
    ctx: I18nContext<L, OS>,
    map_fn: fn(&OS) -> &NS,
) -> I18nContext<L, NS> {
    let old_scope = ConstScope::<L, OS>::new();
    let new_scope = old_scope.map(map_fn);
    ctx.scope(new_scope)
}

#[doc(hidden)]
pub fn scope_locale_util<BL: Locale, L: Locale<BL>, NS: Scope<BL>>(
    locale: L,
    map_fn: fn(&<L as Locale<BL>>::Keys) -> &NS,
) -> ScopedLocale<BL, NS> {
    let _ = map_fn;
    ScopedLocale::new(locale.to_base_locale())
}
