# The `t!` Macro

To access your translations, use the `t!` macro. You can access a string with a simple `t!(i18n, $key)`:

```rust,ignore
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

```rust,ignore
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

If your variable has the same name as the placeholder, you can pass it directly:

```rust,ignore
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

You can pass anything that implements `IntoView + Clone + 'static`, you can pass a view if you want:

```rust,ignore
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

## Interpolate Components

If some components are declared for this key, you can pass them like this:

```rust,ignore
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);
    let count = move || counter.get();

    view! {
        {/* "click_count": "you clicked <b>{{ count }}</b> times<br/>Keep going!" */}
        <p>{t!(i18n, click_count, count, <br/> = || view! { <br/> }, <b> = |children| view!{ <b>{children}</b> })}</p>
    }
}
```

Please note usage of self-closing components.

If your variable has the same name as the component, you can pass it directly:

```rust,ignore
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Foo() -> impl IntoView {
    let i18n = use_i18n();

    let (counter, _set_counter) = signal(0);
    let count = move || counter.get();

    let b = |children| view!{ <b>{children}</b> };

    view! {
        {/* "click_count": "you clicked <b>{{ count }}</b> times<br/>Keep going!" */}
        <p>{t!(i18n, click_count, count, <b>, <br/> = <br/>)}</p>
    }
}
```

You can pass anything that implements `Fn(leptos::ChildrenFn) -> V + Clone + 'static` where `V: IntoView` for normal components or `Fn() -> V + Clone + 'static` where `V: IntoView` for self-closed components.

Any missing components will generate an error.

`|children| view! { <b>{children}</b> }` can be verbose for simple components; you can use this syntax when the children are wrapped by a single component:

```rust,ignore
// key = "<b>{{ count }}</b>"
t!(i18n, key, <b> = <span />, count = 32);
```

This will render `<span>32</span>`.

You can set attributes, event handlers, props, etc.:

```rust,ignore
t!(i18n, key, <b> = <span attr:id="my_id" on:click=|_| { /* do stuff */} />, count = 0);
```

Basically `<name .../>` expands to `move |children| view! { <name ...>{children}</name> }`

## Components Attributes

If you declared attributes with your components

```json
{
  "highlight_me": "highlight <b id={{ id }}>me</b>"
}
```

You can either retrieve them with a closure:

```rust
use leptos::children::ChildrenFn;
use leptos::attr::any_attribute::AnyAttribute;
let b = |children: ChildrenFn, attr: Vec<AnyAttribute>| view!{ <b {..attr} >{children}</b> }
t!(i18n, highlight_me, id = "my_id", <b>)
```

Or they will be passed to direct components alongside code defined attributes:

```rust
// this will spread the attributes into `b` alongside the given attributes
t!(i18n, highlight_me, id = "my_id", <b> = <b attr:foo="bar" />)
```

The same works for self-closing components; for the closure syntax you can take the attributes as the only argument:

```json
{
  "foo": "before<br id={{ id }} />after"
}
```

```rust
let br = |attr: Vec<AnyAttribute>| view!{ <br {..attr} /> }
t!(i18n, highlight_me, id = "my_id", <br>)
```

> _note_: variables to attributes expect the value to implement `leptos::attr::AttributeValue`.

Components with children can accept `Fn(ChildrenFn, Vec<AnyAttribute>)` or `Fn(ChildrenFn)`,
and self-closing components can accept  `Fn()` or `Fn(Vec<AnyAttribute>)`.

## Plurals

Plurals expect a variable `count` that implements `Fn() -> N + Clone + 'static` where `N` implements `Into<icu_plurals::PluralsOperands>` ([`PluralsOperands`](https://docs.rs/icu/latest/icu/plurals/struct.PluralOperands.html)). Integers and unsigned primitives implement it, along with `FixedDecimal`.

```rust,ignore
t!(i18n, key_to_plurals, count = count);
```

## Access Subkeys

You can access subkeys by simply separating the path with `.`:

```rust,ignore
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

## Access Namespaces

Namespaces are implemented as subkeys. You first access the namespace, then the keys in that namespace:

```rust,ignore
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

To avoid confusion with subkeys, you can use `::` to separate the namespace name from the rest of the path:

```rust,ignore
t!(i18n, my_namespace::hello_world)
```

## `tu!`

The `tu!` macro is the same as `t!` but untracked.
