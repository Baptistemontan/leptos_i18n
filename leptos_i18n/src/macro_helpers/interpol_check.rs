use leptos::IntoView;

pub trait InterpolateVar: IntoView + Clone + 'static {}

impl<T: IntoView + Clone + 'static> InterpolateVar for T {}

pub trait InterpolateComp<O: IntoView>: Fn(leptos::ChildrenFn) -> O + Clone + 'static {}

impl<O: IntoView, T: Fn(leptos::ChildrenFn) -> O + Clone + 'static> InterpolateComp<O> for T {}

pub trait InterpolateCount<T>: Fn() -> T + Clone + 'static {}

impl<T, F: Fn() -> T + Clone + 'static> InterpolateCount<T> for F {}
