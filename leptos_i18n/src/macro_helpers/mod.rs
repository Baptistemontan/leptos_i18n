use core::fmt;
use std::{fmt::Display, future::Future, marker::PhantomData};

pub mod formatting;
mod interpol_args;
mod scope;

use crate::Locale;
pub use formatting::*;
pub use interpol_args::*;
use leptos::IntoView;
pub use scope::*;

#[doc(hidden)]
pub trait Literal: Sized + Display + IntoView + Copy {
    type AsStr;
    fn into_str(self) -> Self::AsStr;
}

impl Literal for &'static str {
    type AsStr = Self;
    fn into_str(self) -> Self::AsStr {
        self
    }
}

impl Literal for bool {
    type AsStr = &'static str;
    fn into_str(self) -> Self::AsStr {
        match self {
            true => "true",
            false => "false",
        }
    }
}

macro_rules! impl_build_lit_nums {
    ($t:ty) => {
        impl Literal for $t {
            type AsStr = String;
            fn into_str(self) -> Self::AsStr {
                self.to_string()
            }
        }
    };
    ($t:ty, $($tt:tt)*) => {
        impl_build_lit_nums!($t);
        impl_build_lit_nums!($($tt)*);
    }
}

impl_build_lit_nums!(u64, i64, f64);

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Interpolated values can't be used inside t_string/t_display without the \"interpolate_display\" feature enabled."
)]
pub trait InterpolationStringBuilder
where
    Self: Sized,
{
    fn check(self) -> Self {
        self
    }
}

#[doc(hidden)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LitWrapper<T>(T);

impl<T: Literal> LitWrapper<T> {
    pub const fn new(v: T) -> Self {
        LitWrapper(v)
    }

    pub const fn builder(self) -> Self {
        self
    }

    pub const fn display_builder(self) -> Self {
        self
    }

    pub const fn build(self) -> Self {
        self
    }

    pub fn into_view(self) -> impl IntoView + Copy {
        self.0
    }

    pub fn build_string(self) -> T::AsStr {
        Literal::into_str(self.0)
    }

    pub fn build_display(self) -> impl Display {
        self.0
    }
}

#[doc(hidden)]
#[repr(transparent)]
pub struct LitWrapperFut<T>(T);

impl<T: Literal, F: Future<Output = LitWrapper<T>>> LitWrapperFut<F> {
    pub const fn new(v: F) -> Self {
        LitWrapperFut(v)
    }

    pub const fn builder(self) -> Self {
        self
    }

    pub const fn display_builder(self) -> Self {
        self
    }

    pub const fn build(self) -> Self {
        self
    }

    pub async fn into_view(self) -> impl IntoView + Copy {
        self.0.await.into_view()
    }

    pub async fn build_string(self) -> T::AsStr {
        self.0.await.build_string()
    }

    pub async fn build_display(self) -> impl Display {
        self.0.await.build_display()
    }
}

impl<T: Literal> LitWrapperFut<LitWrapper<T>> {
    pub const fn new_not_fut(v: T) -> Self {
        LitWrapperFut(LitWrapper::new(v))
    }

    pub const fn builder(self) -> Self {
        self
    }

    pub const fn display_builder(self) -> Self {
        self
    }

    pub const fn build(self) -> Self {
        self
    }

    pub fn into_view(self) -> impl IntoView + Copy {
        self.0.into_view()
    }

    pub async fn build_string(self) -> T::AsStr {
        self.0.build_string()
    }

    pub async fn build_display(self) -> impl Display {
        self.0.build_display()
    }
}

#[doc(hidden)]
pub struct LocaleVisitor<L>(PhantomData<L>);

impl<L> Default for LocaleVisitor<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L> LocaleVisitor<L> {
    pub fn new() -> Self {
        LocaleVisitor(PhantomData)
    }
}

impl<'de, L: Locale> serde::de::Visitor<'de> for LocaleVisitor<L> {
    type Value = L;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "one of: [")?;
        let mut locale_iter = L::get_all().iter();
        let first = locale_iter
            .next()
            .expect("Locale should have at least one variant");
        write!(formatter, "{}", first.as_str())?;
        for locale in locale_iter {
            write!(formatter, ", {}", locale.as_str())?;
        }
        write!(formatter, "]")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(L::from_str(v).unwrap_or_default())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::visit_borrowed_str(self, v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::visit_str(self, &v)
    }
}

#[doc(hidden)]
pub struct StrVisitor;

impl<'de> serde::de::Visitor<'de> for StrVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(String::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::visit_borrowed_str(self, v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }
}

#[doc(hidden)]
pub fn intern(s: &str) -> &str {
    if cfg!(any(feature = "csr", feature = "hydrate")) {
        wasm_bindgen::intern(s)
    } else {
        s
    }
}

#[doc(hidden)]
#[track_caller]
#[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
pub const fn index_translations<const N: usize, const I: usize>(
    translations: &'static [&'static str; N],
) -> &'static str {
    translations[I]
}

#[doc(hidden)]
#[track_caller]
#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub fn index_translations<const N: usize, const I: usize>(
    translations: &'static [Box<str>; N],
) -> &'static str {
    &translations[I]
}

#[doc(hidden)]
#[cfg(feature = "dynamic_load")]
pub fn future_renderer<IV: IntoView + 'static + Clone, F: Future<Output = IV> + 'static>(
    fut: impl Fn() -> F + 'static,
) -> impl Fn() -> Option<IV> {
    use leptos::prelude::{AsyncDerived, Get};
    use std::task::Context;
    fn poll_once<F: Future>(fut: F) -> Option<F::Output> {
        let pinned = std::pin::pin!(fut);
        let waker = noop_waker::noop_waker();
        let mut cx = Context::from_waker(&waker);
        match Future::poll(pinned, &mut cx) {
            std::task::Poll::Ready(v) => Some(v),
            std::task::Poll::Pending => None,
        }
    }
    let maybe_ready = poll_once(fut());
    let fut = AsyncDerived::new_unsync(fut);
    move || fut.get().or_else(|| maybe_ready.clone())
}
