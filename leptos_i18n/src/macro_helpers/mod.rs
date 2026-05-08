use core::fmt;
use std::{fmt::Display, marker::PhantomData};

pub mod formatting;
mod interpol_args;
mod scope;

use crate::{
    Locale,
    keys::display::{DisplayArgs, DisplayKey},
};
pub use formatting::*;
pub use interpol_args::*;
pub use scope::*;

fn write_locale_array<L: Locale>(f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let mut locale_iter = L::iter_variants();
    let first = locale_iter
        .next()
        .expect("Locale should have at least one variant");
    write!(f, "[{}", first)?;
    for locale in locale_iter {
        write!(f, ", {}", locale.as_str())?;
    }
    write!(f, "]")
}

#[derive(Debug, Clone)]
pub struct LocaleFromStrError<L> {
    got: String,
    _marker: PhantomData<L>,
}

impl<L: Locale> Display for LocaleFromStrError<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown locale {}, expected one of ", self.got)?;
        write_locale_array::<L>(f)
    }
}

impl<L> LocaleFromStrError<L> {
    pub fn new(got: String) -> Self {
        LocaleFromStrError {
            got,
            _marker: PhantomData,
        }
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
        write_locale_array::<L>(formatter)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match L::from_str(v) {
            Ok(v) => Ok(v),
            Err(err) => Err(E::custom(err)),
        }
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
#[cfg(any(not(feature = "dynamic_load"), feature = "ssr"))]
pub const fn index_translations<const N: usize, const I: usize>(
    translations: &'static [&'static str; N],
) -> &'static str {
    translations[I]
}

#[doc(hidden)]
#[track_caller]
#[cfg(not(any(not(feature = "dynamic_load"), feature = "ssr")))]
pub fn index_translations<const N: usize, const I: usize>(
    translations: &'static [Box<str>; N],
) -> &'static str {
    &translations[I]
}

#[doc(hidden)]
#[cfg(feature = "dynamic_load")]
pub fn future_renderer<F>(fut: impl Fn() -> F + 'static) -> impl leptos::IntoView
where
    F: Future + 'static,
    F::Output: leptos::IntoView + 'static + Clone,
{
    use leptos::prelude::{AsyncDerived, Get};
    let fut = AsyncDerived::new_unsync(fut);
    move || fut.get()
}

#[doc(hidden)]
#[cfg(feature = "plurals")]
pub fn get_plural_category_for<L, F>(
    locale: L,
    count: &F,
    plural_rule_type: icu_plurals::PluralRuleType,
) -> icu_plurals::PluralCategory
where
    L: Locale,
    F: InterpolatePluralCount,
{
    formatting::get_plural_rules(locale, plural_rule_type).category_for(count())
}

#[doc(hidden)]
#[cfg(not(feature = "dynamic_load"))]
pub fn key_to_string<A: DisplayArgs>(key: DisplayKey<A>) -> String {
    key.to_string()
}

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub async fn key_to_string<A, F>(key: F) -> String
where
    A: DisplayArgs,
    F: Future<Output = DisplayKey<A>>,
{
    let key = key.await;
    key.to_string()
}

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
pub async fn key_to_string<A: DisplayArgs>(key: DisplayKey<A>) -> String {
    key.to_string()
}

#[doc(hidden)]
#[track_caller]
pub fn cast_unsized_strings<T, const N: usize>(data: &'static [T]) -> &'static [T; N] {
    data.try_into().expect("wrong size for display data")
}
