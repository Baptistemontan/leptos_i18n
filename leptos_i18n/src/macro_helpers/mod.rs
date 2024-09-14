use core::fmt;
use std::{borrow::Cow, fmt::Display, marker::PhantomData};

pub mod formatting;
mod interpol_args;
mod scope;

use crate::Locale;
pub use formatting::*;
pub use interpol_args::*;
use leptos::IntoView;
pub use scope::*;

#[doc(hidden)]
pub trait Literal: Sized + Display + IntoView {
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

    pub fn into_view(self) -> impl IntoView {
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
pub fn intern(s: &str) -> &str {
    if cfg!(any(feature = "csr", feature = "hydrate")) {
        wasm_bindgen::intern(s)
    } else {
        s
    }
}
