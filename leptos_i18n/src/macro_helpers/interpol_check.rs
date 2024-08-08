use crate::display::DisplayComponent;
use leptos::IntoView;
use std::fmt::Display;

pub trait InterpolateVar: IntoView + Clone + 'static {}

impl<T: IntoView + Clone + 'static> InterpolateVar for T {}

pub fn check_var(var: impl InterpolateVar) -> impl InterpolateVar {
    var
}

pub fn check_var_string(var: impl Display) -> impl Display {
    var
}

pub trait InterpolateComp<O: IntoView>: Fn(leptos::ChildrenFn) -> O + Clone + 'static {}

impl<O: IntoView, T: Fn(leptos::ChildrenFn) -> O + Clone + 'static> InterpolateComp<O> for T {}

pub fn check_comp<V: IntoView>(comp: impl InterpolateComp<V>) -> impl InterpolateComp<V> {
    comp
}

pub fn check_comp_string(comp: impl DisplayComponent) -> impl DisplayComponent {
    comp
}

pub trait InterpolateCount<T>: Fn() -> T + Clone + 'static {}

impl<T, F: Fn() -> T + Clone + 'static> InterpolateCount<T> for F {}

pub fn check_count<T>(count: impl InterpolateCount<T>) -> impl InterpolateCount<T> {
    count
}

pub fn check_count_string<T>(count: T) -> T {
    count
}
