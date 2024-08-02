use leptos::*;
use leptos_router::*;

use crate::{use_i18n_context, I18nContext, Locale};

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
    base_path: &'static str,
) -> impl Fn(Option<L>) -> L + 'static {
    let location = use_location();
    let navigate = use_navigate();
    move |prev_loc: Option<L>| {
        let new_locale = i18n.get_locale();
        let Some(prev_loc) = prev_loc else {
            return new_locale;
        };
        if new_locale == prev_loc {
            return new_locale;
        }

        let new_path = get_new_path(&location, base_path, new_locale, prev_loc);

        navigate(
            &new_path,
            NavigateOptions {
                resolve: false,
                scroll: false,
                ..Default::default()
            },
        );

        new_locale
    }
}

fn get_locale_from_path<L: Locale>(base_path: &'static str) -> L {
    use_location()
        .pathname
        .with_untracked(|path| {
            let stripped_path = path.strip_prefix(base_path)?;
            L::get_all()
                .iter()
                .copied()
                .find(|l| stripped_path.starts_with(l.as_str()))
        })
        .unwrap_or_default()
}

fn ssr_redirection<L: Locale>(base_locale: L, path_locale: L, base_path: &str) -> Option<String> {
    if cfg!(not(feature = "ssr")) || base_locale == path_locale {
        return None;
    }
    let location = use_location();
    let new_path = get_new_path(&location, base_path, base_locale, path_locale);
    Some(new_path)
}

fn outlet_wrapper<L: Locale>(base_path: &'static str) -> impl IntoView {
    let i18n = use_i18n_context::<L>();

    let base_locale = i18n.get_locale_untracked();

    let path_locale = get_locale_from_path::<L>(base_path);

    let redir = ssr_redirection(base_locale, path_locale, base_path);

    if cfg!(not(feature = "ssr")) {
        i18n.set_locale(path_locale);
    }

    create_effect(navigate_effect(i18n, base_path));

    match redir {
        None => view! { <Outlet /> },
        Some(path) => view! { <Redirect path=path/>},
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
pub fn i18n_routing<L: Locale, E, F>(
    base_path: &'static str,
    children: Option<Children>,
    ssr: SsrMode,
    methods: &'static [Method],
    data: Option<Loader>,
    trailing_slash: Option<TrailingSlash>,
    view: F,
) -> RouteDefinition
where
    E: IntoView,
    F: Fn() -> E + 'static,
{
    let mut root_route: RouteDefinition = make_route(
        "",
        None,
        view,
        ssr,
        methods,
        data.clone(),
        Some(TrailingSlash::Drop),
    );

    let default_route = make_route(
        "",
        children,
        move || outlet_wrapper::<L>(base_path),
        ssr,
        methods,
        data.clone(),
        trailing_slash.clone(),
    );

    let mut locale_routes: Vec<RouteDefinition> = L::get_all()
        .iter()
        .copied()
        .map(|l| {
            let mut route = default_route.clone();
            route.path = l.to_string();
            route
        })
        .collect();

    locale_routes.push(default_route);

    root_route.children = locale_routes;

    root_route
}
