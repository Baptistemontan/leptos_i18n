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

/// Marker trait for differenciating closure based on their arguments
pub trait AttributesArgMarker: 'static {
    /// The actual type being turned into a view
    type IntoView: IntoView + 'static;
}

impl<O: AttributesArgMarker + 'static> AttributesArgMarker for WithAttributes<O> {
    type IntoView = O::IntoView;
}

impl<O: AttributesArgMarker + 'static> AttributesArgMarker for WithoutAttributes<O> {
    type IntoView = O::IntoView;
}

impl<O: AttributesArgMarker + 'static> AttributesArgMarker for WithoutChildren<O> {
    type IntoView = O::IntoView;
}

impl<O: AttributesArgMarker + 'static> AttributesArgMarker for WithChildren<O> {
    type IntoView = O::IntoView;
}

impl<O: IntoView + 'static> AttributesArgMarker for O {
    type IntoView = O;
}

/// Trait for a type that can be used as an interpolation component.
pub trait InterpolateComp<O: AttributesArgMarker>: Clone + 'static + Send + Sync {
    /// Create a view from self
    fn to_view(&self, children: leptos::children::ChildrenFn, attrs: &Attributes) -> O::IntoView;
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
pub trait InterpolateCompSelfClosed<O: AttributesArgMarker>: Clone + 'static + Send + Sync {
    /// Create a view from self
    fn to_view(&self, attrs: &Attributes) -> O::IntoView;
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
pub trait InterpolateDummy<O: AttributesArgMarker>: Clone + 'static + Send + Sync {}

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
