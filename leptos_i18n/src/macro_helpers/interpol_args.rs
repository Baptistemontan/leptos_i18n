use leptos::IntoView;

/// Marker trait for a type that can be used as an interpolation variable.
pub trait InterpolateVar: IntoView + Clone + 'static {}

impl<T: IntoView + Clone + 'static> InterpolateVar for T {}

/// Marker trait for a type that can be used as an interpolation component.
pub trait InterpolateComp<O: IntoView>:
    Fn(leptos::children::ChildrenFn) -> O + Clone + 'static
{
}

impl<O: IntoView, T: Fn(leptos::children::ChildrenFn) -> O + Clone + 'static> InterpolateComp<O>
    for T
{
}

/// Marker trait for a type that can be used to produce a count for a range key.
pub trait InterpolateRangeCount<T>: Fn() -> T + Clone + 'static {}

impl<T, F: Fn() -> T + Clone + 'static> InterpolateRangeCount<T> for F {}

/// Marker trait for a type that can produce a `icu::plurals::PluralOperands`
pub trait InterpolatePluralCount: Fn() -> Self::Count + Clone + 'static {
    /// The returned value that can be turned into a `icu::plurals::PluralOperands`
    type Count: Into<icu::plurals::PluralOperands>;
}

impl<T: Into<icu::plurals::PluralOperands>, F: Fn() -> T + Clone + 'static> InterpolatePluralCount
    for F
{
    type Count = T;
}
