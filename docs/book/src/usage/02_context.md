# `I18nContext`

The `I18nContext` type is here to make all your application reactive to the change of the locale, you will use it to access the current locale or change it.

The context is a wrapper around a `RwSignal` of the current locale, every getter/setter must be used with the same reasoning as signals.

## Provide the context

The `load_locales!` macro generates the `provide_i18n_context` function in the `i18n` module,
you can use this function in a component to make the context accessible to all child components.

```rust
use crate::i18n::*;
use leptos::prelude::*;

// root of the application
#[component]
pub fn App() -> impl IntoView {
    provide_i18n_context();

    view! {
        /* */
    }
}
```

## Access the context

Once provided, you can access it with the `use_i18n` function, also generated in the `i18n` module.

```rust
use crate::i18n::*;
use leptos::prelude::*;

// somewhere else in the application
#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        /* */
    }
}
```

## Access the current locale

With the context you can access the current locale with the `get_locale` method:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    create_effect(|_| {
        let locale = i18n.get_locale();
        match locale {
            Locale::en => {
                log!("locale en");
            },
            Locale::fr => {
                log!("locale fr");
            }
        }
    })

    view! {
        /* */
    }
}
```

If you enable the `nightly` feature you can directly call the context: `let locale = i18n();`.

A non-reactive counterpart to `get_locale` exist: `get_locale_untracked`.

## Change the locale

With the context you can change the current locale with the `set_locale` method, for example this component will switch between `en` and `fr` with a button:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let on_switch = move |_| {
        let new_locale = match i18n.get_locale() {
            Locale::en => Locale::fr,
            Locale::fr => Locale::en,
        };
        i18n.set_locale(new_locale);
    };

    view! {
        <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
    }
}
```

If you enable the `nightly` feature you can directly call the context`i18n(new_locale);`.

A non-reactive counterpart to `set_locale` exist: `set_locale_untracked`.

## `"lang"` HTML attribute

When creating a context, it listens to any changes to the locale and update the `<html>` element `"lang"` attribute according to the set locale.

## `cookie` feature

When using the `cookie` feature the context will set a cookie whenever the locale changes,
this cookie will be used to decide what locale to use on the page load in CSR,
and on request to the server in SSR by looking at the request headers.

## Note on island

If you use the `experimental-islands` feature from Leptos this will not work and cause an error on the client:

```rust
#[component]
fn App() -> impl IntoView {
    provide_i18n_context();

    view! {
        <HomePage />
    }
}

#[island]
fn HomePage() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <p>{t!(i18n, hello_world)}</p>
    }
}
```

Because `App` is only rendered on the server, and the code is never called on the client, thus the context is never provided on the client, making `use_i18n` panic when trying to access it.

To fix it first enable the `experimental-islands` feature for `leptos_i18n` and use the `I18nContextProvider` component exported by the `i18n` module:

```rust
#[component]
fn App() -> impl IntoView {
    view! {
        <I18nContextProvider>
            <HomePage />
        </I18nContextProvider>
    }
}
```

## Context options

You may want to customize the context behavior, such as how the cookie is set or what element should receive the `"lang"` attribute,
for this you can use some helpers in the `leptos_i18n::context` module:

`provide_i18n_context_with_options` takes options for the cookie, such as the name, if the cookie should be enabled (will always be `false` if the `cookie` feature is not enabled), and some options about how the cookie is set.

`provide_i18n_context_with_root` takes a `leptos::NodeRef` to an element to set the `"lang"` HTML attribute on.

`provide_i18n_context_with_options_and_root` is a combination of both the above.

There are variants of those with `init_*` instead of `provide_*` that returns the context without providing it.

`provide_*` functions are basically:

```rust
fn provide_*(..args) -> I18nContext<Locale> {
    use_context().unwrap_or_else(move || {
        let ctx = init_*(..args);
        leptos::provide_context(ctx);
        ctx
    })
}
```
