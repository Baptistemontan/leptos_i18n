use core::fmt;
use std::{fmt::Display, marker::PhantomData};

pub mod formatting;
mod interpol_args;
mod scope;

pub use formatting::*;
pub use interpol_args::*;
pub use scope::*;

use crate::Locale;

#[doc(hidden)]
pub trait Lit {
    type AsSTring: Display;

    fn into_string(self) -> Self::AsSTring;
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DisplayBuilder<T>(T);

impl<T: Lit> DisplayBuilder<T> {
    #[inline]
    pub fn build_display(self) -> T::AsSTring {
        self.0.into_string()
    }

    #[inline]
    pub fn build_string(self) -> T::AsSTring {
        self.0.into_string()
    }
}

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

    fn display_builder(self) -> DisplayBuilder<Self> {
        DisplayBuilder(self)
    }

    #[inline]
    fn build(self) -> Self {
        self
    }

    #[inline]
    fn build_string(self) -> Self {
        self
    }
}

impl BuildLit for &'static str {}
impl BuildLit for bool {}

impl Lit for &'static str {
    type AsSTring = Self;
    fn into_string(self) -> Self::AsSTring {
        self
    }
}

impl Lit for bool {
    type AsSTring = &'static str;
    fn into_string(self) -> Self::AsSTring {
        match self {
            true => "true",
            false => "false",
        }
    }
}

macro_rules! impl_build_lit_nums {
    ($t:ty) => {
        impl BuildLit for $t {}

        impl Lit for $t {
            type AsSTring = String;
            fn into_string(self) -> Self::AsSTring {
                ToString::to_string(&self)
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

#[doc(hidden)]
pub fn intern(s: &str) -> &str {
    if cfg!(any(feature = "csr", feature = "hydrate")) {
        wasm_bindgen::intern(s)
    } else {
        s
    }
}
