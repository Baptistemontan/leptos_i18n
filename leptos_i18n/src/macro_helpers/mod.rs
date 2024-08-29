#![doc(hidden)]
use std::{borrow::Cow, fmt::Display};

pub mod formatting;
mod interpol_args;
mod scope;

pub use formatting::*;
pub use interpol_args::*;
use leptos::IntoView;
pub use scope::*;

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
