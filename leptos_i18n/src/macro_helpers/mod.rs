use core::fmt;
use std::{borrow::Cow, fmt::Display, future::Future, marker::PhantomData};

pub mod formatting;
mod interpol_args;
mod scope;

use crate::Locale;
pub use formatting::*;
pub use interpol_args::*;
use leptos::{
    prelude::{AsyncDerived, Get},
    IntoView,
};
pub use scope::*;

#[doc(hidden)]
pub trait Literal: Sized + Display + IntoView + Copy {
    fn into_str(self) -> Cow<'static, str>;
}

impl Literal for &'static str {
    fn into_str(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

impl Literal for bool {
    fn into_str(self) -> Cow<'static, str> {
        match self {
            true => Cow::Borrowed("true"),
            false => Cow::Borrowed("false"),
        }
    }
}

macro_rules! impl_build_lit_nums {
    ($t:ty) => {
        impl Literal for $t {
            fn into_str(self) -> Cow<'static, str> {
                Cow::Owned(self.to_string())
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

    pub fn build_string(self) -> Cow<'static, str> {
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

    pub async fn build_string(self) -> Cow<'static, str> {
        self.0.await.build_string()
    }

    pub async fn build_display(self) -> impl Display {
        self.0.await.build_display()
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
pub const fn index_translations<const N: usize, const I: usize>(
    translations: &'static [&'static str; N],
) -> &'static str {
    assert!(N > I);
    translations[I]
}

#[doc(hidden)]
pub fn future_renderer<IV: IntoView + 'static + Clone, F: Future<Output = IV> + 'static>(
    fut: impl Fn() -> F + 'static,
) -> impl Fn() -> Option<IV> {
    let fut = AsyncDerived::new_unsync(fut);
    move || fut.get()
}
