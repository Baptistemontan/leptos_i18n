use leptos::IntoView;

/// Marker trait for a type that can be used as an interpolation variable.
pub trait InterpolateVar: IntoView + Clone + 'static + Send + Sync {}

impl<T: IntoView + Clone + 'static + Send + Sync> InterpolateVar for T {}

/// Marker trait for a type that can be used as an interpolation component.
pub trait InterpolateComp<O: IntoView>:
    Fn(leptos::children::ChildrenFn) -> O + Clone + 'static + Send + Sync
{
}

impl<O: IntoView, T: Fn(leptos::children::ChildrenFn) -> O + Clone + 'static + Send + Sync>
    InterpolateComp<O> for T
{
}

/// Marker trait for a type that can be used to produce a count for a range key.
pub trait InterpolateRangeCount<T>: Fn() -> T + Clone + 'static + Send + Sync {}

impl<T, F: Fn() -> T + Clone + 'static + Send + Sync> InterpolateRangeCount<T> for F {}

/// Marker trait for a type that can produce a `icu::plurals::PluralOperands`
#[cfg(feature = "plurals")]
pub trait InterpolatePluralCount: Fn() -> Self::Count + Clone + 'static + Send + Sync {
    /// The returned value that can be turned into a `icu::plurals::PluralOperands`
    type Count: Into<icu_plurals::PluralOperands>;
}

#[cfg(feature = "plurals")]
impl<T: Into<icu_plurals::PluralOperands>, F: Fn() -> T + Clone + 'static + Send + Sync>
    InterpolatePluralCount for F
{
    type Count = T;
}
