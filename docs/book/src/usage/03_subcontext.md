# Sub Context

You may want to have sections of your application use translations but be isolated from the "main" locale; this is what sub-contexts are for.

## Why not just use `I18nContextProvider` ?

`I18nContextProvider` does not shadow any context if one already exists,
this is because there should only be one "main" context, or they will conflict over the cookie, the "lang" attribute, the routing, etc.

`init_i18n_subcontext_*` functions create a context that does not battle with the main context and makes it more obvious that a sub-context is created, improving code clarity.

## Initialize a sub-context

`leptos_i18n::context::init_i18n_subcontext` takes an `initial_locale: Option<Signal<L>>` argument, this is so you can control the sub-context locale outside of it, you can for example make it so the locale of the sub-context is always the opposite of the "main" one:

```rust,ignore
fn neg_locale(locale: Locale) -> Locale {
    match locale {
        Locale::en => Locale::fr,
        Locale::fr => Locale::en
    }
}

fn neg_i18n_signal(i18n: I18nContext<Locale>) -> Signal<Locale> {
    Signal::derive(move || neg_locale(i18n.get()))
}

fn opposite_context() {
    let i18n = use_i18n();
    let ctx = init_i18n_subcontext(Some(neg_i18n_signal(i18n)));
    // ..
}
```

If it is not supplied, it takes the parent context locale as a default, and if no parent context exists (yes, you can use sub-context as a "main" context if you want), it uses the same locale resolution as the normal context.

## Providing a sub-context

There is no `provide_i18n_subcontext`. It does exist but is marked as deprecated; it is not actually deprecated, it is only there as an information point, although it does what you think.

#### Shadowing correctly

Shadowing a context is not as easy as it sounds:

```rust,ignore
use crate::i18n::*;
use leptos::prelude::*;
use leptos_i18n::context::provide_i18n_subcontext;

#[component]
fn Foo() -> impl IntoView {
    view! {
        <I18nContextProvider>
            <Sub />
            <Home />
        </I18nContextProvider>
    }
}

#[component]
fn Sub() -> impl IntoView {
    let i18n = provide_i18n_subcontext();
    view! {
        <p>{t!(i18n, sub)}</p>
    }
}

#[component]
fn Home() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <p>{t!(i18n, home)}</p>
    }
}
```

This will actually make the sub-context provided in the `<Sub />` component replace the parent context and leak into the `<Home />` component.

`leptos::provide_context` has a section about shadowing in their docs. The best approach is to use a provider:

```rust,ignore
#[component]
fn Sub() -> impl IntoView {
    let i18n = init_i18n_subcontext();
    view! {
        <Provider value=i18n>
            <p>{t!(i18n, sub)}</p>
        </Provider>
    }
}
```

So this crate has a `I18nSubContextProvider` generated in the `i18n` module:

```rust,ignore
use crate::i18n::*;
use leptos::prelude::*;

#[component]
fn Foo() -> impl IntoView {
    view! {
        <I18nContextProvider>
            <I18nSubContextProvider>
                <Sub />
            </I18nSubContextProvider>
            <Home />
        </I18nContextProvider>
    }
}

#[component]
fn Sub() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <p>{t!(i18n, sub)}</p>
    }
}
```

## Options

Same as with the normal context, sub-contexts have behavior control options; they all take the `initial_locale: Option<Signal<L>>` as their first argument.

`init_i18n_subcontext_with_options` takes cookie options;
that function is useless without the `cookie` feature.

- `cookie_name` is an option for a cookie name to be set to keep the state of the chosen locale.
- `cookie_options` is an option for cookie options.
