use leptos::component;
use leptos_i18n::Locale;
#[cfg(erase_components)]
use leptos_router::any_nested_route::IntoAnyNestedRoute;
use leptos_router::{ChooseView, MatchNestedRoutes, SsrMode, components::RouteChildren};
use std::marker::PhantomData;

/// Whether the default locale gets a URL prefix like the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub enum PrefixDefault {
    /// Default locale is served unprefixed (`/path`).
    /// Prefixed path for the default locale (`/{def}/path`) redirects to the unprefixed one.
    /// Redirection sets the 301 status on servers
    #[default]
    Never,
    /// Every locale is prefixed, including the default (`/{def}/path`).
    /// Unprefixed path redirects to "/{loc}/path", where `loc` is the resolved locale for that client
    /// Redirection sets the 302 status on servers
    Always,
}

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
    #[prop(optional)] _marker: PhantomData<L>,
    /// Whether the default locale is prefixed. Defaults to `PrefixDefault::Never`
    #[prop(optional)]
    prefix_default: PrefixDefault,
    /// `children` may be empty or include nested routes.
    children: RouteChildren<Chil>,
) -> impl MatchNestedRoutes + Clone
where
    View: ChooseView + Clone,
    Chil: MatchNestedRoutes + Send + Clone + 'static,
    L: Locale,
{
    let routes = crate::routing::i18n_routing::<L, View, Chil>(
        base_path,
        children,
        ssr,
        view,
        prefix_default,
    );
    #[cfg(erase_components)]
    return routes.into_any_nested_route();
    #[cfg(not(erase_components))]
    return routes;
}
