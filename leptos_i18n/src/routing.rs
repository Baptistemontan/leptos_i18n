use leptos::*;
use leptos_router::*;

use crate::{provide_i18n_context, use_i18n_context, Locale};

fn outlet_wrapper<L: Locale>(locale: Option<L>, base_path: &'static str) -> impl IntoView {
    let locale = locale.unwrap_or_default();
    let i18n = use_i18n_context::<L>();
    i18n.set_locale(locale);

    let location = use_location();
    let navigate = use_navigate();

    create_effect(move |_| {
        let new_locale = i18n.get_locale();
        if new_locale == locale {
            return;
        }
        let mut new_path = base_path.to_string();
        if new_locale != L::default() {
            new_path.push_str(new_locale.as_str());
        }
        location.pathname.with_untracked(|path_name| {
            if let Some(path_rest) = path_name
                .strip_prefix(base_path)
                .and_then(|s| s.strip_prefix(locale.as_str()))
            {
                new_path.push_str(path_rest)
            }
        });
        location.search.with_untracked(|search| {
            new_path.push_str(search);
        });
        location.hash.with_untracked(|hash| {
            new_path.push_str(hash);
        });
        navigate(
            &new_path,
            NavigateOptions {
                resolve: false,
                scroll: false,
                ..Default::default()
            },
        )
    });

    view! {
        <Outlet />
    }
}

fn root_outlet_wrapper<L: Locale>() -> impl IntoView {
    provide_i18n_context::<L>();

    view! {
        <Outlet />
    }
}

fn make_route<V: IntoView>(
    path: &str,
    children: Option<Children>,
    view: impl Fn() -> V + 'static,
    ssr: SsrMode,
    methods: &'static [Method],
    data: Option<Loader>,
    trailing_slash: Option<TrailingSlash>,
) -> RouteDefinition {
    Route(RouteProps {
        path,
        view,
        ssr,
        methods,
        data,
        trailing_slash,
        children,
    })
    .into_view()
    .as_transparent()
    .and_then(|t| t.downcast_ref::<RouteDefinition>())
    .cloned()
    .unwrap()
}

#[doc(hidden)]
pub fn i18n_routing<L: Locale>(
    base_path: &'static str,
    children: Option<ChildrenFn>,
    ssr: SsrMode,
    methods: &'static [Method],
    data: Option<Loader>,
    trailing_slash: Option<TrailingSlash>,
) -> RouteDefinition {
    let get_children = move || children.clone().map(|c| Box::from(move || c()) as Children);

    let mut root_route: RouteDefinition = make_route(
        "",
        None,
        root_outlet_wrapper::<L>,
        ssr,
        methods,
        data.clone(),
        trailing_slash.clone(),
    );

    let default_route = make_route(
        "",
        get_children(),
        move || outlet_wrapper::<L>(None, base_path),
        ssr,
        methods,
        data.clone(),
        trailing_slash.clone(),
    );

    let mut locale_routes: Vec<RouteDefinition> = L::get_all()
        .iter()
        .copied()
        .map(move |l| {
            make_route(
                l.as_str(),
                get_children(),
                move || outlet_wrapper(Some(l), base_path),
                ssr,
                methods,
                data.clone(),
                trailing_slash.clone(),
            )
        })
        .collect();

    locale_routes.push(default_route);

    root_route.children = locale_routes;

    root_route
}
