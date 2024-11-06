use std::{future::Future, sync::Arc};

use leptos::{either::Either, ev, prelude::*};
use leptos_router::{
    components::*,
    hooks::{use_location, use_navigate},
    location::Location,
    ChooseView, MatchInterface, MatchNestedRoutes, MatchParams, NavigateOptions, NestedRoute,
    PathSegment, PossibleRouteMatch, SsrMode, StaticSegment,
};

use crate::{use_i18n_context, I18nContext, Locale};

// this whole file is a hack into `leptos_router`, it absolutely should'nt be used like that, but eh I'm a professional (or not.)

#[derive(Debug)]
struct PathBuilder<'a>(Vec<&'a str>);

impl Default for PathBuilder<'_> {
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
                Some(l) => {
                    if let Some(path_rest) = path_rest.strip_prefix(l.as_str()) {
                        path_builder.push(path_rest)
                    } else {
                        path_builder.push(path_rest) // Should happen only if l == L::default()
                    }
                }
            }
            // else ?
        }
        path_builder.build()
    });
    location.search.with_untracked(|search| {
        if !search.is_empty() {
            new_path.push('?');
            new_path.push_str(search);
        }
    });
    location.hash.with_untracked(|hash| {
        if !hash.is_empty() {
            new_path.push('#');
            new_path.push_str(hash);
        }
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

        let navigate = navigate.clone();

        // TODO FIXME: see https://github.com/leptos-rs/leptos/issues/2979
        // It works for now, but it is not ideal.
        request_animation_frame(move || {
            navigate(
                &new_path,
                NavigateOptions {
                    resolve: false,
                    scroll: false,
                    ..Default::default()
                },
            );
        });

        new_locale
    }
}

fn correct_locale_prefix_effect<L: Locale>(
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

        let navigate = navigate.clone();

        // TODO FIXME: see https://github.com/leptos-rs/leptos/issues/2979
        // It works for now, but it is not ideal.
        request_animation_frame(move || {
            navigate(
                &new_path,
                NavigateOptions {
                    resolve: false,
                    replace: true,
                    scroll: false,
                    ..Default::default()
                },
            );
        });
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
    let location = use_location();
    if cfg!(not(feature = "ssr")) || previously_resolved_locale == L::default() {
        return None;
    }
    let new_path = get_new_path(&location, base_path, previously_resolved_locale, None);
    Some(new_path)
}

#[doc(hidden)]
#[derive(Clone)]
pub struct RedirectView(Arc<dyn Fn() -> leptos::prelude::View<()> + Send + Sync>);

struct ViewWrapper<View>(Arc<dyn Fn() -> Either<View, RedirectView> + Send + Sync>);

impl<View> Clone for ViewWrapper<View> {
    fn clone(&self) -> Self {
        ViewWrapper(self.0.clone())
    }
}

impl ChooseView for RedirectView {
    async fn choose(self) -> AnyView {
        self.0().into_any()
    }

    async fn preload(&self) {}
}

impl<View: ChooseView> ChooseView for ViewWrapper<View> {
    fn choose(self) -> impl Future<Output = AnyView> {
        let inner = self.0();
        ChooseView::choose(inner)
    }

    async fn preload(&self) {}
}

fn view_wrapper<L: Locale, View: ChooseView>(
    view: View,
    route_locale: Option<L>,
    base_path: &'static str,
) -> Either<View, RedirectView> {
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
    // but changing the locale on history change will trigger the locale change effect, causing to change the URL again but with a wrong previous locale
    // so this variable sync them together on what is the locale currently in the URL.
    // it starts at None such that on the first render the effect don't change the locale instantly.
    let history_changed_locale = StoredValue::new(None);

    Effect::new(update_path_effect(i18n, base_path, history_changed_locale));

    // listen for history changes
    let handle = window_event_listener(
        ev::popstate,
        check_history_change(i18n, base_path, history_changed_locale),
    );

    on_cleanup(move || handle.remove());

    // correct the url when using <a> that removes the locale prefix
    Effect::new(correct_locale_prefix_effect(i18n, base_path));

    match redir {
        None => Either::Left(view),
        Some(path) => {
            let view = Arc::new(move || view! { <Redirect path={ path.clone() }/> });
            Either::Right(RedirectView(view))
        }
    }
}

#[doc(hidden)]
pub fn i18n_routing<L: Locale, View, Chil>(
    base_path: &'static str,
    children: RouteChildren<Chil>,
    ssr_mode: SsrMode,
    view: View,
) -> L::Routes<View, Chil>
where
    View: ChooseView,
{
    let children = children.into_inner();
    let base_route = NestedRoute::new(StaticSegment(""), view)
        .ssr_mode(ssr_mode)
        .child(children);
    let base_route = Arc::new(base_route);

    L::make_routes(base_route, base_path)
}

#[doc(hidden)]
pub struct I18nNestedRoute<L, View, Chil> {
    route: Arc<NestedRoute<StaticSegment<&'static str>, Chil, (), View>>,
    locale: Option<L>,
    base_path: &'static str,
}

impl<L: Clone, View, Chil> Clone for I18nNestedRoute<L, View, Chil> {
    fn clone(&self) -> Self {
        let route = self.route.clone();
        let locale = self.locale.clone();
        let base_path = self.base_path;
        I18nNestedRoute {
            route,
            locale,
            base_path,
        }
    }
}

impl<L: Locale, View: ChooseView, Chil> I18nNestedRoute<L, View, Chil> {
    pub fn new(
        locale: Option<L>,
        base_path: &'static str,
        route: Arc<NestedRoute<StaticSegment<&'static str>, Chil, (), View>>,
    ) -> Self {
        Self {
            route,
            locale,
            base_path,
        }
    }
}

// what you will see after this comment is an absolute fuckery.
// The goal here is to create N + 1 routes where N is the number of locales (last being for empty).
// not very difficult.
// but if you do it the "normal" way, changing locales will rebuild the entire tree, making the application loose state when it does'nt need to.
// So, we want to create N + 1 routes, that are "the same"
// Leptos differentiate them with their "RouteId"
// So we basically create N+1 route with the same route id
// All the stupidity you will see under this comment is done just to archieve this.

#[doc(hidden)]
pub type BaseRoute<View, Chil> = Arc<NestedRoute<StaticSegment<&'static str>, Chil, (), View>>;

#[doc(hidden)]
pub struct I18nRouteMatch<L, View, Chil>
where
    Chil: MatchNestedRoutes + 'static,
    Chil::Match: MatchParams,
    View: ChooseView + Clone,
{
    locale: Option<L>,
    base_path: &'static str,
    matched: String,
    inner_match:
        <NestedRoute<StaticSegment<&'static str>, Chil, (), View> as MatchNestedRoutes>::Match,
}

impl<L, View, Chil> MatchParams for I18nRouteMatch<L, View, Chil>
where
    Chil: MatchNestedRoutes + 'static,
    Chil::Match: MatchParams,
    View: ChooseView + Clone,
{
    fn to_params(&self) -> Vec<(std::borrow::Cow<'static, str>, String)> {
        MatchParams::to_params(&self.inner_match)
    }
}

impl<L, View, Chil> MatchInterface for I18nRouteMatch<L, View, Chil>
where
    L: Locale,
    Chil: MatchNestedRoutes + 'static,
    Chil::Match: MatchParams,
    View: ChooseView + Clone + Sync,
{
    type Child = <<NestedRoute<StaticSegment<&'static str>, Chil, (), View> as MatchNestedRoutes>::Match as MatchInterface>::Child;

    fn as_id(&self) -> leptos_router::RouteMatchId {
        MatchInterface::as_id(&self.inner_match)
    }

    fn as_matched(&self) -> &str {
        &self.matched
    }

    fn into_view_and_child(self) -> (impl ChooseView, Option<Self::Child>) {
        let (view, child) = MatchInterface::into_view_and_child(self.inner_match);
        let new_view = Arc::new(move || view_wrapper(view.clone(), self.locale, self.base_path));
        (ViewWrapper(new_view), child)
    }
}

impl<L: Locale, View, Chil> MatchNestedRoutes for I18nNestedRoute<L, View, Chil>
where
    Chil: MatchNestedRoutes + 'static,
    Chil::Match: MatchParams,
    View: ChooseView + Clone + Sync,
{
    type Data = ();

    type Match = I18nRouteMatch<L, View, Chil>;

    fn match_nested<'a>(
        &'a self,
        path: &'a str,
    ) -> (Option<(leptos_router::RouteMatchId, Self::Match)>, &'a str) {
        if let Some(locale) = self.locale {
            StaticSegment(locale.as_str())
                .test(path)
                .and_then(|partial_path_match| {
                    let remaining = partial_path_match.remaining();
                    let matched = partial_path_match.matched();
                    let (inner_match, remaining) =
                        MatchNestedRoutes::match_nested(&*self.route, remaining);
                    let (route_match_id, inner_match) = inner_match?;
                    let matched = matched.to_string();
                    let route_match = I18nRouteMatch {
                        locale: Some(locale),
                        matched,
                        inner_match,
                        base_path: self.base_path,
                    };
                    Some((Some((route_match_id, route_match)), remaining))
                })
                .unwrap_or((None, path))
        } else {
            let (inner_match, remaining) = MatchNestedRoutes::match_nested(&*self.route, path);
            inner_match
                .map(|(route_match_id, inner_match)| {
                    let route_match = I18nRouteMatch {
                        locale: None,
                        matched: String::new(),
                        inner_match,
                        base_path: self.base_path,
                    };
                    (Some((route_match_id, route_match)), remaining)
                })
                .unwrap_or((None, path))
        }
    }

    fn generate_routes(&self) -> impl IntoIterator<Item = leptos_router::GeneratedRouteData> + '_ {
        MatchNestedRoutes::generate_routes(&*self.route)
            .into_iter()
            .map(|mut generated_route| {
                if let (Some(locale), Some(first)) =
                    (self.locale, generated_route.segments.first_mut())
                {
                    // replace the empty segment set by the inner route with the locale one
                    *first = PathSegment::Static(locale.as_str().into())
                }
                generated_route
            })
    }
}
