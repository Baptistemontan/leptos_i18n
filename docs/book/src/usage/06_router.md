# `I18nRoute`

The `i18n` module generated from the `load_locales!()` macro export the `I18nRoute` component,
this component act exactly like a `leptos_router::Route` and take the same args, except for the path.

What it does is manage a prefix on the URL such that

```rust
use crate::i18n::I18nRoute;
use leptos::*;
use leptos_router::*;

view! {
    <Router>
        <Routes>
            <I18nRoute>
                <Route path="/" view=Home />
                <Route path="/counter" view=Counter />
            </I18nRoute>
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

This means that if `Counter` keep a count as a state, and you switch locale from `fr` to `en`, this will trigger a navigation from `"/fr/counter"` to `"/counter"`, but the component will not be rerendered.

## Navigation

With the way the default route is handled, if you have a `<A href=.. />` link in your application or use `leptos_router::use_navigate`,
you don't have to worry about removing the locale prefix as this will trigger a redirection to the correct locale.

This redirection also set `NavigateOptions.replace` to `true` so the intermidiate location will not show in the history.

Basically, if you are at `"/fr/counter"` and trigger a redirection to `"/"`, this will trigger another redirection to `"/fr"`
and the history will look like you directly navigated from `"/fr/counter"` to `"/fr"`.
