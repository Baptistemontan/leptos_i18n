# `I18nContext`

The `I18nContext` type is here to make all your application reactive to the change of the locale, you will use it to access the current locale or change it.

The context is a wrapper around a `RwSignal` of the current locale, every getter/setter must be used with the same reasoning as signals.

## Provide the context

The `load_locales!` macro generates the `I18nContextProvider` component in the `i18n` module,
you can use this component to make the context accessible to all child components.

```rust
use crate::i18n::*;
use leptos::prelude::*;

// root of the application
#[component]
pub fn App() -> impl IntoView {
    view! {
        <I18nContextProvider>
            /* */
        </I18nContextProvider>
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

## `cookie` feature

When using the `cookie` feature the context will set a cookie whenever the locale changes,
this cookie will be used to decide what locale to use on the page load in CSR,
and on request to the server in SSR by looking at the request headers.

## Context options

The `I18nContextProvider` component accept multiple props, all optionnal (except children)

- `children`: obviously
- `set_lang_attr_on_html`: should or not set the "lang" attribute on the root `<html>` element (default to true)
- `set_dir_attr_on_html`: should or not set the "dir" attribute on the root `<html>` element (default to true)
- `enable_cookie`: should set a cookie to keep track of the locale when page reload (default to true) (do nothing without the "cookie" feature)
- `cookie_name`: give a custom name to the cookie (default to the crate default value) (do nothing without the "cookie" feature or if `enable_cookie` is false)
- `cookie_options`: options for the cookie, the value is of type `leptos_use::UseCookieOptions<Locale>` (default to `Default::default`)

## Note on island

If you use the `experimental-islands` feature from Leptos the `I18nContextProvider` loose two props: `cookie_options` and `ssr_lang_header_getter`, because they are not serializable. If you need them you can use the `init_context_with_options` function and provide the context yourself:

```rust
use leptos_i18n::init_i18n_context_with_options;
use leptos_i18n::context::{CookieOptions, UseLocalesOptions};
use leptos_meta::Html;
use leptos::prelude::*;
use crate::i18n::*;

#[island]
fn MyI18nProvider(
    enable_cookie: Option<bool>,
    cookie_name: Option<&str>,
    children: Children
) -> impl IntoView {
    let my_cookie_options: CookieOptions<Locale> = /* create your options here */;
    let ssr_lang_header_getter: UseLocalesOptions = /* create your options here */;
    let i18n = init_i18n_context_with_options::<Locale>(
        enable_cookie,
        cookie_name,
        Some(my_cookie_options),
        Some(ssr_lang_header_getter)
    );
    provide_context(i18n);
    let lang = move || i18n.get_locale().as_str();
    let dir = move || i18n.get_locale().direction().as_str();
    view! {
        <Html
            attr:lang=lang
            attr:dir=dir
        />
        {children}
    }
}
```

## "lang" and "dir" html attributes

You may want to add a "lang" or/and "dir" attribute on a html element such that

```html
<div lang="fr"></div>
```

You could do it yourself by tracking the locale and setting the attribute yourself, but there is a simpler way:

The `I18nContext` implement `Directive` from leptos to set the "lang" attribute, so you can just do

```rust
let i18n = use_i18n();

view! {
    <div use:i18n />
}
```

And it will set the "lang" and "dir" attributes for you on the `<div>` element !
_note_ : use directives don't work on the server, so don't rely on this for server side rendering.
