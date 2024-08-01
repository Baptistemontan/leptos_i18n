use leptos::*;
use leptos_router::*;

use crate::{provide_i18n_context, use_i18n_context, I18nContext, Locale};

#[derive(Debug)]
struct PathBuilder<'a>(Vec<&'a str>);

impl<'a> Default for PathBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PathBuilder<'a> {
    pub fn new() -> Self {
        PathBuilder(vec![""])
    }

    pub fn push(&mut self, s: &'a str) {
        let s = s.trim_matches('/');
        if !s.is_empty() {
            self.0.push(s);
        }
    }

    pub fn build(&self) -> String {
        let s = self.0.join("/");
        if s.is_empty() {
            "/".to_owned()
        } else {
            s
        }
    }
}

fn get_new_path<L: Locale>(
    location: &Location,
    base_path: &str,
    new_locale: L,
    locale: L,
) -> String {
    let mut new_path = location.pathname.with_untracked(|path_name| {
        let mut path_builder = PathBuilder::default();
        path_builder.push(base_path);
        if new_locale != L::default() {
            path_builder.push(new_locale.as_str());
        }
        if let Some(path_rest) = path_name.strip_prefix(base_path) {
            if let Some(path_rest) = path_rest.strip_prefix(locale.as_str()) {
                path_builder.push(path_rest)
            } else if locale == L::default() {
                path_builder.push(path_rest)
            }
            // else ?
        }
        path_builder.build()
    });
    location.search.with_untracked(|search| {
        new_path.push_str(search);
    });
    location.hash.with_untracked(|hash| {
        new_path.push_str(hash);
    });
    new_path
}

fn navigate_effect<L: Locale>(
    i18n: I18nContext<L>,
    locale: L,
    base_path: &'static str,
) -> impl Fn(Option<()>) + 'static {
    let location = use_location();
    let navigate = use_navigate();
    move |_| {
        let new_locale = i18n.get_locale();
        if new_locale == locale {
            return;
        }
        let new_path = get_new_path(&location, base_path, new_locale, locale);

        navigate(
            &new_path,
            NavigateOptions {
                resolve: false,
                scroll: false,
                ..Default::default()
            },
        )
    }
}

fn ssr_redirection<L: Locale>(i18n: I18nContext<L>, locale: L, base_path: &str) -> Option<String> {
    let current_locale = i18n.get_locale_untracked();
    if cfg!(not(feature = "ssr")) || current_locale == locale {
        return None;
    }
    let location = use_location();
    let new_path = get_new_path(&location, base_path, current_locale, locale);
    Some(new_path)
}

fn outlet_wrapper<L: Locale>(locale: L, base_path: &'static str) -> impl IntoView {
    let i18n = use_i18n_context::<L>();

    let redir = ssr_redirection(i18n, locale, base_path);

    if cfg!(not(feature = "ssr")) {
        i18n.set_locale(locale);
    }

    create_effect(navigate_effect(i18n, locale, base_path));

    match redir {
        None => view! { <Outlet /> },
        Some(path) => view! { <Redirect path=path/>},
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
        move || outlet_wrapper::<L>(L::default(), base_path),
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
                move || outlet_wrapper(l, base_path),
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
