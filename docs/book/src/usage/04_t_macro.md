# The `t!` Macro

To access your translations the `t!` macro is used, you can access a string with a simple `t!(i18n, $key)`:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        {/* "hello_world": "Hello World!" */}
        <p>{t!(i18n, hello_world)}</p>
    }
}
```

## Interpolate Values

If some variables are declared for this key, you can pass them like this:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);

    view! {
        {/* "click_count": "you clicked {{ count }} times" */}
        <p>{t!(i18n, click_count, count = move || counter.get())}</p>
    }
}
```

If your variable has the same name as the value, you can pass it directly:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);

    let count = move || counter.get();

    view! {
        {/* "click_count": "you clicked {{ count }} times" */}
        <p>{t!(i18n, click_count, count)}</p>
    }
}
```

You can pass anything that implement `IntoView + Clone + 'static`, you can pass a view if you want:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);

    let count = view!{
        <b>
            { move || counter.get() }
        </b>
    };

    view! {
        {/* "click_count": "you clicked {{ count }} times" */}
        <p>{t!(i18n, click_count, count)}</p>
    }
}
```

Any missing values will generate an error.

## Interpolate components

If some components are declared for this key, you can pass them like this:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);
    let count = move || counter.get();

    view! {
        {/* "click_count": "you clicked <b>{{ count }}</b> times" */}
        <p>{t!(i18n, click_count, count, <b> = |children| view!{ <b>{children}</b> })}</p>
    }
}
```

If your variable as the same name as the component, you can pass it directly:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);
    let count = move || counter.get();

    let b = |children| view!{ <b>{children}</b> };

    view! {
        {/* "click_count": "you clicked <b>{{ count }}</b> times" */}
        <p>{t!(i18n, click_count, count, <b>)}</p>
    }
}
```

You can pass anything that implement `Fn(leptos::ChildrenFn) -> V + Clone + 'static` where `V: IntoView`.

Any missing components will generate an error.

`|children| view! { <b>{children}</b> }` can be verbose for simple components, you can use this syntax when the children is wrapped by a single component:

```rust
// key = "<b>{{ count }}</b>"
t!(i18n, key, <b> = <span />, count = 32);
```

This will render `<span>32</span>`.

You can set attributes, event handlers, props ect:

```rust
t!(i18n, key, <b> = <span attr:id="my_id" on:click=|_| { /* do stuff */} />, count = 0);
```

Basically `<name .../>` expand to `move |children| view! { <name ...>{children}</name> }`

## Ranges

Ranges expect a variable `count` that implement `Fn() -> N + Clone + 'static` where `N` is the specified type of the range (default is `i32`).

```rust
t!(i18n, key_to_range, count = count);
```

## Plurals

Plurals expect a variable `count` that implement `Fn() -> N + Clone + 'static` where `N` implement `Into<icu_plurals::PluralsOperands>` ([`PluralsOperands`](https://docs.rs/icu/latest/icu/plurals/struct.PluralOperands.html)). Integers and unsigned primitives implements it, along with `FixedDecimal`.

```rust
t!(i18n, key_to_plurals, count = count);
```

## Access subkeys

You can access subkeys by simply separating the path with `.`:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        {/*
            "subkeys": {
                "subkey_1": "This is subkeys.subkey_1"
            }
        */}
        <p>{t!(i18n, subkeys.subkey_1)}</p>
    }
}
```

## Access namespaces

Namespaces are implemented as subkeys, you first access the namespace then the keys in that namespace:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <p>{t!(i18n, my_namespace.hello_world)}</p>
    }
}
```

To avoid confusion with subkeys you can use `::` to separate the namespace name from the rest of the path:

```rust
t!(i18n, my_namespace::hello_world)
```

## `tu!`

the `tu!` macro is the same as `t!` but untracked.
