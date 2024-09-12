use core::fmt;
use std::{borrow::Cow, marker::PhantomData};

pub mod formatting;
mod interpol_args;
mod scope;

pub use formatting::*;
pub use interpol_args::*;
pub use scope::*;

use crate::Locale;

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
pub trait BuildLit: Sized {
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

impl BuildLit for &'static str {
    #[inline]
    fn display_builder(self) -> DisplayBuilder {
        DisplayBuilder(Cow::Borrowed(self))
    }
}

impl BuildLit for bool {
    #[inline]
    fn display_builder(self) -> DisplayBuilder {
        match self {
            true => DisplayBuilder(Cow::Borrowed("true")),
            false => DisplayBuilder(Cow::Borrowed("false")),
        }
    }
}

macro_rules! impl_build_lit_nums {
    ($t:ty) => {
        impl BuildLit for $t {
            fn display_builder(self) -> DisplayBuilder {
                DisplayBuilder(Cow::Owned(ToString::to_string(&self)))
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
