#![doc(hidden)]

use std::borrow::Cow;

mod formatting;
mod interpol_check;
mod scope;

pub use formatting::*;
pub use interpol_check::*;
pub use scope::*;

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
