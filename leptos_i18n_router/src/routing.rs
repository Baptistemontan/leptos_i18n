use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use leptos::{either::Either, ev, prelude::*};
use leptos_router::{
    ChooseView, MatchInterface, MatchNestedRoutes, MatchParams, NavigateOptions, NestedRoute,
    PathSegment, PossibleRouteMatch, SsrMode, StaticSegment,
    components::*,
    hooks::{use_location, use_navigate},
    location::Location,
};

use leptos_i18n::{I18nContext, Locale, use_i18n_context};

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
        if s.is_empty() { "/".to_owned() } else { s }
    }
}

fn match_path_segments(segments: &[&str], old_segments: &[PathSegment]) -> Option<HashSet<usize>> {
    // This hurt my eyes

    let mut optionals = HashSet::new();

    let mut segments_iter = old_segments.iter().enumerate();
    'outer: for seg in segments {
        'inner: loop {
            let (index, next_seg) = segments_iter.next()?;

            match next_seg {
                PathSegment::Unit => continue 'inner,
                PathSegment::Param(_) => continue 'outer,
                PathSegment::OptionalParam(to_match) if to_match == seg => {
                    optionals.insert(index);
                    continue 'outer;
                }
                PathSegment::OptionalParam(_) => continue 'inner,
                PathSegment::Static(to_match) if to_match.is_empty() => continue 'inner,
                PathSegment::Static(to_match) if to_match == seg => continue 'outer,
                PathSegment::Static(_) => return None,
                PathSegment::Splat(_) => return Some(optionals),
            }
        }
    }

    // if iter is empty, perfect match !
    segments_iter.next().is_none().then_some(optionals)
}

fn get_locale_from_path<L: Locale>(path: &str, base_path: &str) -> Option<L> {
    let base_path = base_path.trim_start_matches('/');
    let stripped_path = path
        .trim_start_matches('/')
        .strip_prefix(base_path)?
        .trim_start_matches('/');
    let (to_match, _) = stripped_path.split_once('/').unwrap_or((stripped_path, ""));
    L::get_all()
        .iter()
        .copied()
        .find(|l| l.as_str() == to_match)
}

fn construct_path_segments<'b, 'p: 'b>(
    segments: &[&'p str],
    new_segments: &'p [PathSegment],
    path_builder: &mut PathBuilder<'b>,
    optionals: &HashSet<usize>,
) {
    let mut segments_iter = new_segments.iter().enumerate();
    let mut outer_seg_iter = segments.iter();
    'outer: for seg in &mut outer_seg_iter {
        'inner: loop {
            let (index, next_seg) = segments_iter.next().unwrap();

            match next_seg {
                PathSegment::Unit => continue 'inner,
                PathSegment::Param(_) => {
                    path_builder.push(seg);
                    continue 'outer;
                }
                PathSegment::OptionalParam(_) if optionals.contains(&index) => {
                    path_builder.push(seg);
                    continue 'outer;
                }
                PathSegment::OptionalParam(_) => continue 'inner,
                PathSegment::Static(to_push) if to_push.is_empty() => continue 'inner,
                PathSegment::Static(to_push) => {
                    path_builder.push(to_push);
                    continue 'outer;
                }
                PathSegment::Splat(_) => {
                    path_builder.push(seg);
                    break 'outer;
                }
            }
        }
    }
    for seg in outer_seg_iter {
        path_builder.push(seg);
    }
}

fn localize_path<'b, 'p: 'b>(
    path: &'p str,
    old_locale_segments: &[Vec<PathSegment>],
    new_locale_segments: &'p [Vec<PathSegment>],
    path_builder: &mut PathBuilder<'b>,
) -> Option<()> {
    let path_segments = path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let (pos, optionals) =
        old_locale_segments
            .iter()
            .enumerate()
            .find_map(|(pos, old_segments)| {
                match_path_segments(&path_segments, old_segments).map(|op| (pos, op))
            })?;

    let new_segments = &new_locale_segments[pos];

    construct_path_segments(&path_segments, new_segments, path_builder, &optionals);

    Some(())
}

fn get_new_path<L: Locale>(
    location: &Location,
    base_path: &str,
    new_locale: L,
    locale: Option<L>,
    segments: RouteSegments<L>,
) -> String {
    let _ = segments;
    let mut new_path = location.pathname.with_untracked(|path_name| {
        let segments = segments.0.lock().unwrap();
        let mut path_builder = PathBuilder::default();
        path_builder.push(base_path);
        if new_locale != L::default() {
            path_builder.push(new_locale.as_str());
        }
        if let Some(path_rest) = path_name.strip_prefix(base_path) {
            let path_rest = match locale {
                None => path_rest,
                Some(l) => {
                    if let Some(path_rest) = path_rest.strip_prefix(l.as_str()) {
                        path_rest
                    } else {
                        path_rest // Should happen only if l == L::default()
                    }
                }
            };

            let old_locale_segments = segments.get(&locale.unwrap_or_default());
            let new_locale_segments = segments.get(&new_locale);

            let localized = match (old_locale_segments, new_locale_segments) {
                (Some(old_locale_segments), Some(new_locale_segments)) => localize_path(
                    path_rest,
                    old_locale_segments,
                    new_locale_segments,
                    &mut path_builder,
                )
                .is_some(),
                _ => false,
            };

            if !localized {
                path_builder.push(path_rest);
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
            // Remove leading '#' if present
            let hash = hash.trim_start_matches('#');
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
    segments: RouteSegments<L>,
) -> impl Fn(Option<L>) -> L + 'static {
    let location = use_location();
    let navigate = use_navigate();
    move |prev_loc: Option<L>| {
        let path_locale = location
            .pathname
            .with_untracked(|path| get_locale_from_path::<L>(path, base_path));
        let new_locale = i18n.get_locale();
        // don't react on history change.
        if let Some(new_locale) = history_changed_locale.get_value() {
            history_changed_locale.set_value(None);
            return new_locale;
        }
        let Some(prev_loc) = prev_loc else {
            return new_locale;
        };
        if new_locale == prev_loc || new_locale == path_locale.unwrap_or_default() {
            return new_locale;
        }

        let new_path = get_new_path(
            &location,
            base_path,
            new_locale,
            Some(prev_loc),
            segments.clone(),
        );

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
    segments: RouteSegments<L>,
    history_changed: StoredValue<bool>,
) -> impl Fn(Option<()>) + 'static {
    let location = use_location();
    let navigate = use_navigate();
    move |_| {
        let path_locale = location
            .pathname
            .with(|path| get_locale_from_path::<L>(path, base_path));
        let current_locale = i18n.get_locale_untracked();

        if current_locale == path_locale.unwrap_or_default() {
            return;
        }

        let new_locale = if history_changed.get_value() {
            history_changed.set_value(false);
            current_locale
        } else {
            path_locale.unwrap_or(current_locale)
        };

        let new_path = get_new_path(
            &location,
            base_path,
            new_locale,
            path_locale,
            segments.clone(),
        );

        let navigate = navigate.clone();

        i18n.set_locale(new_locale);

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

fn check_history_change<L: Locale>(
    i18n: I18nContext<L>,
    base_path: &'static str,
    sync: StoredValue<Option<L>>,
    history_changed: StoredValue<bool>,
) -> impl Fn(ev::PopStateEvent) + 'static {
    let location = use_location();

    move |_| {
        let path_locale = location
            .pathname
            .with_untracked(|path| get_locale_from_path::<L>(path, base_path).unwrap_or_default());

        sync.set_value(Some(path_locale));
        history_changed.set_value(true);

        if i18n.get_locale_untracked() != path_locale {
            i18n.set_locale(path_locale);
        }
    }
}

fn maybe_redirect<L: Locale>(
    previously_resolved_locale: L,
    base_path: &str,
    segments: RouteSegments<L>,
) -> Option<String> {
    let location = use_location();
    if cfg!(not(feature = "ssr")) || previously_resolved_locale == L::default() {
        return None;
    }
    let new_path = get_new_path(
        &location,
        base_path,
        previously_resolved_locale,
        None,
        segments,
    );
    Some(new_path)
}

#[derive(Clone)]
struct ViewWrapper<T, A, B>(T)
where
    T: Fn() -> Either<A, B> + Send + Clone + 'static,
    A: ChooseView,
    B: ChooseView;

impl<T, A, B> ChooseView for ViewWrapper<T, A, B>
where
    T: Fn() -> Either<A, B> + Send + Clone + 'static,
    A: ChooseView,
    B: ChooseView,
{
    fn choose(self) -> impl Future<Output = AnyView> {
        let inner = self.0();
        ChooseView::choose(inner)
    }

    async fn preload(&self) {}
}

fn view_wrapper<L, View>(
    view: View,
    route_locale: Option<L>,
    base_path: &'static str,
    segments: RouteSegments<L>,
) -> Either<View, impl ChooseView>
where
    L: Locale,
    View: ChooseView,
{
    let i18n = use_i18n_context::<L>();

    let previously_resolved_locale = i18n.get_locale_untracked();

    // By precedence if there is a locale prefix in the URL it takes priority.
    // if there is none, use the one computed beforehand.

    let redir = if let Some(locale) = route_locale {
        i18n.set_locale(locale);
        None
    } else {
        maybe_redirect(previously_resolved_locale, base_path, segments.clone())
    };

    // This variable is there to sync history changes, because we step out of the Leptos routes reactivity we don't get forward and backward history changes triggers
    // So we have to do it manually
    // but changing the locale on history change will trigger the locale change effect, causing to change the URL again but with a wrong previous locale
    // so this variable sync them together on what is the locale currently in the URL.
    // it starts at None such that on the first render the effect don't change the locale instantly.
    let sync = StoredValue::new(None);
    let history_changed = StoredValue::new(false);

    Effect::new(update_path_effect(i18n, base_path, sync, segments.clone()));

    // listen for history changes
    let handle = window_event_listener(
        ev::popstate,
        check_history_change(i18n, base_path, sync, history_changed),
    );

    on_cleanup(move || handle.remove());

    // correct the url when using <a> that removes the locale prefix
    Effect::new(correct_locale_prefix_effect(
        i18n,
        base_path,
        segments,
        history_changed,
    ));

    match redir {
        None => Either::Left(view),
        Some(path) => {
            let view = move || view! { <Redirect path=path.clone() /> };
            Either::Right(view)
        }
    }
}

#[doc(hidden)]
pub fn i18n_routing<L: Locale, View, Chil>(
    base_path: &'static str,
    children: RouteChildren<Chil>,
    ssr_mode: SsrMode,
    view: View,
) -> impl MatchNestedRoutes + Clone
where
    View: ChooseView + Clone,
    Chil: MatchNestedRoutes + Clone + 'static,
{
    let children = children.into_inner();
    let base_route = NestedRoute::new(StaticSegment(""), view)
        .ssr_mode(ssr_mode)
        .child(children);

    let segments = RouteSegments::<L>::default();

    let routes = I18nNestedRoute::new(base_path, base_route, segments.clone());

    let inner_segments = routes.generate_routes_for_each_locale();

    let mut guard = segments.0.lock().unwrap();

    *guard = inner_segments;

    routes
}

#[derive(Clone, Default)]
struct RouteSegments<L>(Arc<Mutex<RouteSegmentsInner<L>>>);

type RouteSegmentsInner<L> = HashMap<L, Vec<Vec<PathSegment>>>;

#[derive(Clone)]
struct I18nNestedRoute<L, View, Chil> {
    route: BaseRoute<View, Chil>,
    base_path: &'static str,
    segments: RouteSegments<L>,
}

impl<L, View, Chil> I18nNestedRoute<L, View, Chil> {
    pub fn new(
        base_path: &'static str,
        route: BaseRoute<View, Chil>,
        segments: RouteSegments<L>,
    ) -> Self {
        Self {
            route,
            base_path,
            segments,
        }
    }
}

// what you will see after this comment is a bit of a hack.
// The goal here is to create N + 1 routes where N is the number of locales (last being for empty).
// not very difficult.
// but if you do it the "normal" way, changing locales will rebuild the entire tree, making the application
// lose state when it doesn't need to.
// So, we want to create N + 1 routes that are "the same".
// Leptos differentiates them with their "RouteId".
// So we basically create N+1 routes with the same route id.
// All the complexity you will see under this comment is done just to achieve this.

#[doc(hidden)]
pub type BaseRoute<View, Chil> = NestedRoute<StaticSegment<&'static str>, Chil, (), View>;

thread_local! {
    static CURRENT_ROUTE_LOCALE: RefCell<Option<&'static str>> = const { RefCell::new(None) };
}

fn set_current_route_locale<L: Locale>(new_locale: L) {
    CURRENT_ROUTE_LOCALE.with_borrow_mut(|locale| {
        *locale = Some(new_locale.as_str());
    })
}

fn reset_current_route_locale() {
    CURRENT_ROUTE_LOCALE.with_borrow_mut(|locale| *locale = None);
}

fn get_current_route_locale<L: Locale>() -> L {
    CURRENT_ROUTE_LOCALE
        .with_borrow(|locale| locale.as_ref().and_then(|locale| L::from_str(locale).ok()))
        .unwrap_or_default()
}

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
    inner_match: <BaseRoute<View, Chil> as MatchNestedRoutes>::Match,
    segments: RouteSegments<L>,
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
    View: ChooseView + Clone,
{
    type Child = <<BaseRoute<View, Chil> as MatchNestedRoutes>::Match as MatchInterface>::Child;

    fn as_id(&self) -> leptos_router::RouteMatchId {
        MatchInterface::as_id(&self.inner_match)
    }

    fn as_matched(&self) -> &str {
        &self.matched
    }

    fn into_view_and_child(self) -> (impl ChooseView, Option<Self::Child>) {
        let (view, child) = MatchInterface::into_view_and_child(self.inner_match);
        let new_view = move || {
            view_wrapper(
                view.clone(),
                self.locale,
                self.base_path,
                self.segments.clone(),
            )
        };
        (ViewWrapper(new_view), child)
    }
}

impl<L: Locale, View, Chil> MatchNestedRoutes for I18nNestedRoute<L, View, Chil>
where
    Chil: MatchNestedRoutes + 'static,
    Chil::Match: MatchParams,
    View: ChooseView + Clone,
{
    type Data = ();

    type Match = I18nRouteMatch<L, View, Chil>;

    fn match_nested<'a>(
        &'a self,
        path: &'a str,
    ) -> (Option<(leptos_router::RouteMatchId, Self::Match)>, &'a str) {
        let res = L::get_all()
            .iter()
            .copied()
            .find_map(|locale| {
                set_current_route_locale(locale);
                StaticSegment(locale.as_str())
                    .test(path)
                    .and_then(|partial_path_match| {
                        let remaining = partial_path_match.remaining();
                        let matched = partial_path_match.matched();
                        let (inner_match, remaining) =
                            MatchNestedRoutes::match_nested(&self.route, remaining);
                        let (route_match_id, inner_match) = inner_match?;
                        let matched = matched.to_string();
                        let route_match = I18nRouteMatch {
                            locale: Some(locale),
                            matched,
                            inner_match,
                            base_path: self.base_path,
                            segments: self.segments.clone(),
                        };
                        Some((Some((route_match_id, route_match)), remaining))
                    })
            })
            .or_else(|| {
                set_current_route_locale(L::default());
                let (inner_match, remaining) = MatchNestedRoutes::match_nested(&self.route, path);
                inner_match.map(|(route_match_id, inner_match)| {
                    let route_match = I18nRouteMatch {
                        locale: None,
                        matched: String::new(),
                        inner_match,
                        base_path: self.base_path,
                        segments: self.segments.clone(),
                    };
                    (Some((route_match_id, route_match)), remaining)
                })
            })
            .unwrap_or((None, path));
        reset_current_route_locale();
        res
    }

    fn generate_routes(&self) -> impl IntoIterator<Item = leptos_router::GeneratedRouteData> + '_ {
        let reset = std::iter::from_fn(|| {
            reset_current_route_locale();
            None
        });
        let default_locale_routes = std::iter::once_with(|| {
            set_current_route_locale(L::default());
            MatchNestedRoutes::generate_routes(&self.route)
                .into_iter()
                .map(|mut generated_route| {
                    // remove empty segment set by the inner route
                    generated_route.segments.remove(0);
                    generated_route
                })
        })
        .flatten();
        L::get_all()
            .iter()
            .copied()
            .flat_map(|locale| {
                set_current_route_locale(locale);
                MatchNestedRoutes::generate_routes(&self.route)
                    .into_iter()
                    .map(move |mut generated_route| {
                        if let Some(first) = generated_route.segments.first_mut() {
                            // replace the empty segment set by the inner route with the locale one
                            *first = PathSegment::Static(locale.as_str().into())
                        }
                        generated_route
                    })
            })
            .chain(default_locale_routes)
            .chain(reset)
    }

    fn optional(&self) -> bool {
        false
    }
}

impl<L: Locale, View, Chil> I18nNestedRoute<L, View, Chil>
where
    L: Locale,
    View: ChooseView + Clone,
    Chil: MatchNestedRoutes + Clone + 'static,
{
    fn generate_routes_for_each_locale(&self) -> RouteSegmentsInner<L> {
        let mut segments = RouteSegmentsInner::default();

        for locale in L::get_all() {
            set_current_route_locale(*locale);
            let inner_segments: Vec<_> = MatchNestedRoutes::generate_routes(&self.route)
                .into_iter()
                .map(|generated_route| generated_route.segments)
                .collect();

            segments.insert(*locale, inner_segments);
        }

        reset_current_route_locale();

        segments
    }
}

#[derive(Clone, Copy)]
pub struct I18nPath<L, F> {
    func: F,
    marker: PhantomData<L>,
}

impl<L, F> Debug for I18nPath<L, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18nPath").finish()
    }
}

impl<L: Locale, F> I18nPath<L, F>
where
    F: Fn(L) -> &'static str,
{
    fn segments_for_current_locale(
        &self,
    ) -> impl Iterator<Item = leptos_router::StaticSegment<&'static str>> {
        let locale = get_current_route_locale::<L>();
        let s = (self.func)(locale);

        s.split('/')
            .filter(|p| !p.is_empty())
            .map(leptos_router::StaticSegment)
    }
}

impl<L: Locale, F> PossibleRouteMatch for I18nPath<L, F>
where
    F: Fn(L) -> &'static str,
{
    fn optional(&self) -> bool {
        false
    }

    fn test<'a>(&self, path: &'a str) -> Option<leptos_router::PartialPathMatch<'a>> {
        use leptos_router::PartialPathMatch;

        let mut segments = self.segments_for_current_locale().peekable();

        if segments.peek().is_none() {
            return ().test(path);
        }

        let mut remaining = path;
        let mut all_params = Vec::new();
        let mut matched_len = 0usize;

        for static_seg in segments {
            let pm = static_seg.test(remaining)?;

            let matched = pm.matched();
            matched_len += matched.len();
            remaining = pm.remaining();
            all_params.extend(pm.params());
        }

        Some(PartialPathMatch::new(
            remaining,
            all_params,
            &path[..matched_len],
        ))
    }

    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        for static_seg in self.segments_for_current_locale() {
            static_seg.generate_path(path);
        }
    }
}

#[doc(hidden)]
pub fn make_i18n_path<L, F>(f: F) -> I18nPath<L, F>
where
    L: Locale,
    F: Fn(L) -> &'static str + Clone + 'static,
{
    I18nPath {
        func: f,
        marker: PhantomData,
    }
}
