# Sub Context

You may want to have sections of you application to use the translations but be isolated from the "main" locale, this is what sub-context are for.

## Why not just use `provide_i18n_context` ?

`provide_i18n_context` does not shadow any context if one already exist,
this is because it should only be one "main" context, or they will battle for the cookie, the "lang" attribute, the routing, ect..

`init_i18n_subcontext_*` functions create a context that does not battle with the main context and makes it more obvious that a sub context is created, improving code clarity.

## Initialize a sub-context

`leptos_i18n::context::init_i18n_subcontext` takes an `initial_locale: Option<Signal<L>>` argument, this is so you can control the sub-context locale outside of it, you can for example makes it so the locale of the sub-context is always the opposite of the "main" one:

```rust
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

If it is not supplied, it takes the "main" context locale as a default, and if no "main" context exist (yes you can use sub-context as a "main" context if you want) it uses the same locale resolution as the normal context.

## Providing a sub-context

There is no `provide_i18n_subcontext`. It does exist but is marked as deprecated, it is not actually deprecated, it is only there as a information point, although is does what you thing.

#### Shadowing correctly

Shadowing a context is not as easy as it sounds:

```rust
use crate::i18n::*;
use leptos::*;
use leptos_i18n::context::provide_i18n_subcontext;

#[component]
fn Foo() -> impl IntoView {
    provide_i18n_context();
    view! {
        <Sub />
        <Home />
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

This will actually make the sub-context provided in the `<Sub />` component replace the main context and leak into the `<Home />` component.

`leptos::provide_context` has a section about shadowing in there docs, the best approach is to use a provider:

```rust
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

So this crate has a `I18nSubContextProvider` in the `context` module:

```rust
use crate::i18n::*;
use leptos::*;
use leptos_i18n::context::I18nSubContextProvider;

#[component]
fn Foo() -> impl IntoView {
    provide_i18n_context();
    view! {
        <I18nSubContextProvider>
            <Sub />
        </I18nSubContextProvider>
        <Home />
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

Same has with the normal context, sub-contexts have behavior control options, they all take the `initial_locale: Option<Signal<L>>` as their first argument.

`init_i18n_subcontext_with_options` takes options a cookie,
that function is useless without the `cookie` feature.

- `cookie_name` is an option to a name for a cookie to be set to keep state of the chosen locale
- `cookie_options` is an option to some options for a cookie.

`init_i18n_subcontext_with_root` takes a `leptos::NodeRef` to an element to set the `"lang"` HTML attribute on.

`init_i18n_subcontext_with_options_and_root` is a combination of both the above.
