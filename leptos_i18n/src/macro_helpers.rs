#![doc(hidden)]

use core::fmt;
pub use fixed_decimal::FixedDecimal;
use fixed_decimal::FloatPrecision;
use icu::decimal::FixedDecimalFormatter;
use leptos::{IntoView, Signal, SignalGet, SignalWith};
use std::{borrow::Cow, fmt::Display};

use crate::{
    display::DisplayComponent, scopes::ScopedLocale, ConstScope, I18nContext, Locale, Scope,
};

// Interpolation check

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

// Formatting

// Format nums
macro_rules! impl_into_fixed_decimal {
    ($ty:ty) => {
        impl IntoFixedDecimal for $ty {
            fn to_fixed_decimal(self) -> FixedDecimal {
                Into::into(self)
            }
        }
    };
    ($ty:ty, $($tt:tt)*) => {
        impl_into_fixed_decimal!($ty);
        impl_into_fixed_decimal!($($tt)*);
    }
}

pub trait IntoFixedDecimal: Clone {
    fn to_fixed_decimal(self) -> FixedDecimal;
}

// T: Into<FixedDecimal>
impl_into_fixed_decimal!(
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    isize,
    i8,
    i16,
    i32,
    i64,
    i128,
    FixedDecimal
);

impl IntoFixedDecimal for f32 {
    fn to_fixed_decimal(self) -> FixedDecimal {
        FixedDecimal::try_from_f64(self.into(), FloatPrecision::Floating).unwrap()
    }
}

impl IntoFixedDecimal for f64 {
    fn to_fixed_decimal(self) -> FixedDecimal {
        FixedDecimal::try_from_f64(self, FloatPrecision::Floating).unwrap()
    }
}

pub type FixedDecimalSignal = Signal<FixedDecimal>;

pub trait IntoFixedDecimalSignal {
    fn to_signal(self) -> FixedDecimalSignal;
}

impl<T: IntoFixedDecimal> IntoFixedDecimalSignal for T {
    fn to_signal(self) -> FixedDecimalSignal {
        let v = self.to_fixed_decimal();
        Signal::derive(move || v.clone())
    }
}

impl<T: IntoFixedDecimal> IntoFixedDecimalSignal for Signal<T> {
    fn to_signal(self) -> FixedDecimalSignal {
        Signal::derive(move || self.get().to_fixed_decimal())
    }
}

pub fn check_into_fixed_decimal(var: impl IntoFixedDecimalSignal) -> FixedDecimalSignal {
    var.to_signal()
}

pub fn check_into_fixed_decimal_string(var: impl IntoFixedDecimal) -> FixedDecimal {
    var.to_fixed_decimal()
}

pub fn format_number_to_string<L: Locale>(
    locale: L,
    number: Signal<FixedDecimal>,
) -> Signal<String> {
    let formatter =
        FixedDecimalFormatter::try_new(&locale.as_langid().into(), Default::default()).unwrap();

    Signal::derive(move || number.with(|value| formatter.format_to_string(value)))
}

pub fn format_number_to_formatter<L: Locale>(
    locale: L,
    number: &FixedDecimal,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    FixedDecimalFormatter::try_new(&locale.as_langid().into(), Default::default())
        .unwrap()
        .format(number)
        .fmt(f)
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
