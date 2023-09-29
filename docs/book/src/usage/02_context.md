# `I18nContext`

The `I18nContext` type is here to make all your application reactive to the change of the locale, you will use it to access the current locale or change it.

The context is a wrapper around a `RwSignal` of the current locale, every getter/setter must be used with the same reasoning as signals.

## Provide the context

The `load_locales!` macro generate the `provide_i18n_context` function in the `i18n` module,
you can use this fonction in a component to make the context accessible to all child components.

```rust
use crate::i18n::*;
use leptos::*;

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
use leptos::*;

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
use leptos::*;

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

With the context you can change the current locale with the `set_locale` method, for exemple this component will switch beetween `en` and `fr` with a button:

```rust
use crate::i18n::*;
use leptos::*;

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
