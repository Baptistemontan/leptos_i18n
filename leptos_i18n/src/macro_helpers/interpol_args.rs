use std::marker::PhantomData;

use leptos::IntoView;

/// Marker trait for a type that can be used as an interpolation variable.
pub trait InterpolateVar: IntoView + Clone + 'static + Send + Sync {}

impl<T: IntoView + Clone + 'static + Send + Sync> InterpolateVar for T {}

/// Attributes of a parsed component
pub type Attributes = Vec<leptos::attr::any_attribute::AnyAttribute>;

/// Marker for closure that don't take attributes as argument
pub struct WithoutAttributes<O>(PhantomData<O>);
/// Marker for closure that take attributes as argument
pub struct WithAttributes<O>(PhantomData<O>);

/// Marker for closure that don't take children as argument
pub struct WithoutChildren<O>(PhantomData<O>);
/// Marker for closure that take children as argument
pub struct WithChildren<O>(PhantomData<O>);

/// Marker trait for differenciating closures based on their arguments
pub trait CompMarker: 'static {
    /// The actual output
    type Output: IntoView + 'static;
}

impl<O: CompMarker + 'static> CompMarker for WithAttributes<O> {
    type Output = O::Output;
}

impl<O: CompMarker + 'static> CompMarker for WithoutAttributes<O> {
    type Output = O::Output;
}

impl<O: CompMarker + 'static> CompMarker for WithoutChildren<O> {
    type Output = O::Output;
}

impl<O: CompMarker + 'static> CompMarker for WithChildren<O> {
    type Output = O::Output;
}

impl<O: IntoView + 'static> CompMarker for O {
    type Output = O;
}

/// Trait for a type that can be used as an interpolation component.
pub trait InterpolateComp<O: CompMarker>: Clone + 'static + Send + Sync {
    /// Create a view from self
    fn to_view(&self, children: leptos::children::ChildrenFn, attrs: &Attributes) -> O::Output;
}

impl<
    O: IntoView + 'static,
    T: Fn(leptos::children::ChildrenFn) -> O + Clone + 'static + Send + Sync,
> InterpolateComp<WithoutAttributes<O>> for T
{
    fn to_view(&self, children: leptos::children::ChildrenFn, _attrs: &Attributes) -> O {
        self(children)
    }
}

impl<
    O: IntoView + 'static,
    T: Fn(leptos::children::ChildrenFn, Attributes) -> O + Clone + 'static + Send + Sync,
> InterpolateComp<WithAttributes<O>> for T
{
    fn to_view(&self, children: leptos::children::ChildrenFn, attrs: &Attributes) -> O {
        self(children, attrs.clone())
    }
}

/// Trait for a type that can be used as an interpolation self-closed component.
pub trait InterpolateCompSelfClosed<O: CompMarker>: Clone + 'static + Send + Sync {
    /// Create a view from self
    fn to_view(&self, attrs: &Attributes) -> O::Output;
}

impl<O: IntoView + 'static, T: Fn() -> O + Clone + 'static + Send + Sync>
    InterpolateCompSelfClosed<WithoutAttributes<O>> for T
{
    fn to_view(&self, _attrs: &Attributes) -> O {
        self()
    }
}

impl<O: IntoView + 'static, T: Fn(Attributes) -> O + Clone + 'static + Send + Sync>
    InterpolateCompSelfClosed<WithAttributes<O>> for T
{
    fn to_view(&self, attrs: &Attributes) -> O {
        self(attrs.clone())
    }
}

/// Marker trait for dummy components where no information about self-closeness was found.
/// Very rare case, but still possible
#[doc(hidden)]
pub trait InterpolateDummy<O: CompMarker>: Clone + 'static + Send + Sync {}

impl<O: IntoView + 'static, T: Fn() -> O + Clone + 'static + Send + Sync>
    InterpolateDummy<WithoutChildren<WithoutAttributes<O>>> for T
{
}

impl<O: IntoView + 'static, T: Fn(Attributes) -> O + Clone + 'static + Send + Sync>
    InterpolateDummy<WithoutChildren<WithAttributes<O>>> for T
{
}

impl<
    O: IntoView + 'static,
    T: Fn(leptos::children::ChildrenFn) -> O + Clone + 'static + Send + Sync,
> InterpolateDummy<WithChildren<WithoutAttributes<O>>> for T
{
}

impl<
    O: IntoView + 'static,
    T: Fn(leptos::children::ChildrenFn, Attributes) -> O + Clone + 'static + Send + Sync,
> InterpolateDummy<WithChildren<WithAttributes<O>>> for T
{
}

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
