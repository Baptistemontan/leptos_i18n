use std::marker::PhantomData;

use leptos::component;
use leptos_i18n::Locale;
use leptos_router::{
    any_nested_route::IntoAnyNestedRoute, components::RouteChildren, ChooseView, MatchNestedRoutes,
    SsrMode,
};

#[component(transparent)]
pub fn I18nRoute<L, View, Chil>(
    /// The base path of this application.
    /// If you setup your i18n route such that the path is `/foo/:locale/bar`,
    /// the expected base path is `"foo"`, `"/foo"`, `"foo/"` or `"/foo/"`.
    /// Defaults to `"/"`.
    #[prop(default = "/")]
    base_path: &'static str,
    /// The view that should be shown when this route is matched. This can be any function
    /// that returns a type that implements [`IntoView`] (like `|| view! { <p>"Show this"</p> })`
    /// or `|| view! { <MyComponent/>` } or even, for a component with no props, `MyComponent`).
    /// If you use nested routes you can just set it to `view=Outlet`
    view: View,
    /// The mode that this route prefers during server-side rendering. Defaults to out-of-order streaming
    #[prop(optional)]
    ssr: SsrMode,
    /// `children` may be empty or include nested routes.
    children: RouteChildren<Chil>,
    #[prop(optional)] _marker: PhantomData<L>,
) -> impl MatchNestedRoutes + Clone
where
    View: ChooseView + Clone,
    Chil: MatchNestedRoutes + Send + Clone + 'static,
    L: Locale,
{
    crate::routing::i18n_routing::<L, View, Chil>(base_path, children, ssr, view)
        .into_any_nested_route()
}
