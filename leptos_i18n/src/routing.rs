use std::rc::Rc;

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
    locale: Option<L>,
) -> String {
    let mut new_path = location.pathname.with_untracked(|path_name| {
        let mut path_builder = PathBuilder::default();
        path_builder.push(base_path);
        if new_locale != L::default() {
            path_builder.push(new_locale.as_str());
        }
        if let Some(path_rest) = path_name.strip_prefix(base_path) {
            match locale {
                None => path_builder.push(path_rest),
                Some(l) if l == L::default() => path_builder.push(path_rest),
                Some(l) => {
                    if let Some(path_rest) = path_rest.strip_prefix(l.as_str()) {
                        path_builder.push(path_rest)
                    } else {
                        path_builder.push(path_rest) //? Should'nt happen tho
                    }
                }
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

/// navigate to a new path when the locale changes
fn update_path_effect<L: Locale>(
    i18n: I18nContext<L>,
    base_path: &'static str,
    history_changed_locale: StoredValue<Option<L>>,
) -> impl Fn(Option<L>) -> L + 'static {
    let location = use_location();
    let navigate = use_navigate();
    move |prev_loc: Option<L>| {
        let new_locale = i18n.get_locale();
        // don't react on history change.
        if let Some(new_locale) = history_changed_locale.get_value() {
            history_changed_locale.set_value(None);
            return new_locale;
        }
        let Some(prev_loc) = prev_loc else {
            return new_locale;
        };
        if new_locale == prev_loc {
            return new_locale;
        }

        let new_path = get_new_path(&location, base_path, new_locale, Some(prev_loc));

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

fn correct_locale_prefix_effet<L: Locale>(
    i18n: I18nContext<L>,
    base_path: &'static str,
) -> impl Fn(Option<()>) + 'static {
    let location = use_location();
    let navigate = use_navigate();
    move |_| {
        let path_locale = get_locale_from_path::<L>(&location, base_path);
        let current_locale = i18n.get_locale_untracked();

        if current_locale == path_locale.unwrap_or_default() {
            return;
        }

        let new_path = get_new_path(&location, base_path, current_locale, path_locale);

        navigate(
            &new_path,
            NavigateOptions {
                resolve: false,
                replace: true,
                scroll: false,
                ..Default::default()
            },
        )
    }
}

fn get_locale_from_path<L: Locale>(location: &Location, base_path: &'static str) -> Option<L> {
    location.pathname.with(|path| {
        let stripped_path = path.strip_prefix(base_path)?;
        L::get_all()
            .iter()
            .copied()
            .find(|l| stripped_path.starts_with(l.as_str()))
    })
}

fn check_history_change<L: Locale>(
    i18n: I18nContext<L>,
    base_path: &'static str,
    sync: StoredValue<Option<L>>,
) -> impl Fn(ev::PopStateEvent) + 'static {
    let location = use_location();

    move |_| {
        let path_locale = get_locale_from_path::<L>(&location, base_path).unwrap_or_default();

        sync.set_value(Some(path_locale));

        if i18n.get_locale_untracked() != path_locale {
            i18n.set_locale(path_locale);
        }
    }
}

fn maybe_redirect<L: Locale>(previously_resolved_locale: L, base_path: &str) -> Option<String> {
    if previously_resolved_locale == L::default() {
        return None;
    }
    let location = use_location();
    let new_path = get_new_path(&location, base_path, previously_resolved_locale, None);
    Some(new_path)
}

fn outlet_wrapper<L: Locale>(route_locale: Option<L>, base_path: &'static str) -> impl IntoView {
    let i18n = use_i18n_context::<L>();

    let previously_resolved_locale = i18n.get_locale_untracked();

    // By precedence if there is a locale prefix in the URL it takes priority.
    // if there is none, use the one computed beforehand.

    let redir = if let Some(locale) = route_locale {
        i18n.set_locale(locale);
        None
    } else {
        maybe_redirect(previously_resolved_locale, base_path)
    };

    // This variable is there to sync history changes, because we step out of the Leptos routes reactivity we don't get forward and backward history changes triggers
    // So we have to do it manually
    // but chnaging the locale on history change will trigger the locale change effect, causing to change the URL again but with a wrong previous locale
    // so this variable sync them together on what is the locale currently in the URL.
    // it starts at None such that on the first render the effect don't change the locale instantly.
    let history_changed_locale = StoredValue::new(None);

    create_effect(update_path_effect(i18n, base_path, history_changed_locale));
    // listen for history changes
    leptos::window_event_listener(
        ev::popstate,
        check_history_change(i18n, base_path, history_changed_locale),
    );
    // correct the url when using <a> that removes the locale prefix
    create_effect(correct_locale_prefix_effet(i18n, base_path));

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
        move || outlet_wrapper::<L>(None, base_path),
        ssr,
        methods,
        data.clone(),
        trailing_slash.clone(),
    );

    let mut locale_routes: Vec<RouteDefinition> = L::get_all()
        .iter()
        .copied()
        .map(|l| RouteDefinition {
            path: l.to_string(),
            view: Rc::new(move || outlet_wrapper::<L>(Some(l), base_path).into_view()),
            ..default_route.clone()
        })
        .collect();

    locale_routes.push(default_route);

    root_route.children = locale_routes;

    root_route
}
