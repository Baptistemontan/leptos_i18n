# `I18nRoute`

You can use the `leptos_i18n_router` crate that export the `I18nRoute` component,
this component act exactly like a `leptos_router::Route` and take the same args, except for the path.

What it does is manage a prefix on the URL such that

```rust
use crate::i18n::Locale;
use leptos_i18n_router::I118nRoute;
use leptos::prelude::*;
use leptos_router::*;

view! {
    <Router>
        <Routes fallback=||"Page not found">
            <I18nRoute<Locale, _, _> view=Outlet>
                <Route path=path!("") view=Home />
                <Route path=path!("counter") view=Counter />
            </I18nRoute<Locale, _, _>>
        </Routes>
    </Router>
}
```

Produce default routes `"/"` and `"/counter"`, but also `":locale/"` and `":locale/counter"` for each locale.

if you have `en` and `fr` as your routes, the generated routes will be: `/`, `/counter`, `/en`, `/en/counter`, `/fr` and `/fr/counter`.

This component provide the `I18nContext` if not already provided, and set the locale accordingly.

## Locale resolution

The locale prefix in the URL is considered to have the biggest priority, when accessing `"/en/*"` the locale will be set to `en` no matter what.

But accessing without a locale prefix such as `"/counter"` the locale will be resolved based on other factors like cookie, request `Accept-Language` header or `navigator` API.

see the [Locale Resolution](../infos/01_locale_resol.md) section.

### Redirection

If a locale is found those ways, and it is not the default locale, this will trigger a navigation to the correct locale prefix.

This means if you access `"/counter"` with the cookie set to `fr` (default being `en`) then you will be redirected to `"/fr/counter"`.

## Switching locale

Switching locale updates the prefix accordingly, switching from `en` to `fr` will set the prefix to `fr`, but switching to the default locale will remove the locale prefix entirely.

## State keeping

Switching locale will trigger a navigation, update the `Location` returned by `use_location`, but will not refresh the component tree.

This means that if `Counter` keep a count as a state, and you switch locale from `fr` to `en`, this will trigger a navigation from `"/fr/counter"` to `"/counter"`,
but the component will not be rerendered and the count state will be preserved.

## Navigation

With the way the default route is handled, if you have a `<A href=.. />` link in your application or use `leptos_router::use_navigate`,
you don't have to worry about removing the locale prefix as this will trigger a redirection to the correct locale.

This redirection also set `NavigateOptions.replace` to `true` so the intermediate location will not show in the history.

Basically, if you are at `"/fr/counter"` and trigger a redirection to `"/"`, this will trigger another redirection to `"/fr"`
and the history will look like you directly navigated from `"/fr/counter"` to `"/fr"`.

## Localized path segments

You can use inside the `i18nRoute` the `i18n_path!` to create localized path segments:

```rust
use leptos_i18n_router::i18n_path;

<I18nRoute<Locale, _, _> view=Outlet>
    <Route path=i18n_path!(Locale, |locale| td_string(locale, segment_path_name)) view={/* */} />
</I18nRoute<Locale, _, _>>
```

If you have `segment_path_name = "search"` for english, and `segment_path_name = "rechercher"` for french, the `I18nRoute` will produce 3 paths:

- "/search" (if default = "en")
- "/en/search"
- "/fr/rechercher"

It can be used at any depth, and if not used inside a `i18nRoute` it will default to the default locale.

## Caveat

If you have a layout like this:

```rust
view! {
    <I18nContextProvider>
        <Menu />
        <Router>
            <Routes fallback=||"Page not found">
                <I18nRoute<Locale, _, _> view=Outlet>
                    <Route path=path!("") view=Home />
                </I18nRoute<Locale, _, _>>
            </Routes>
        </Router>
    </I18nContextProvider>
}
```

And the `Menu` component use localization, you could be suprised to see that sometimes there is a mismatch beetween the locale used by the `Menu` and the one inside the router.
This is due to the locale being read from the URL only when the `i18nRoute` is rendered. So the context may be initialized with another locale, and then hit the router that update it.

One solution would be to use the `Menu` component inside the `i18nRoute`:

```rust
view! {
    <I18nContextProvider>
        <Router>
            <Routes fallback=||"Page not found">
                <I18nRoute<Locale, _, _> view=|| view! {
                    <Menu />
                    <Outlet />
                }>
                    <Route path=path!("") view=Home />
                </I18nRoute<Locale, _, _>>
            </Routes>
        </Router>
    </I18nContextProvider>
}
```
