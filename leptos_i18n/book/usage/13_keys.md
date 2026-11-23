# Keys As Values

`t!(i18n, my_key, count = 3)` does three things at once: it finds the key `my_key`, gives it its arguments, and renders it. Sometimes you need to do these steps separately, for example to:

- pass a translation to a component, and let the component render it,
- store translations in a list (menu entries, table headers, ...),
- pick the key to display at runtime.

For that, you can build a key as a value, and render it later. The macros below are not re-exported by the generated `i18n` module, import them from `leptos_i18n`:

```rust,ignore
use leptos_i18n::{build_key, key, key_builder};
```

## Building A Key

### `key_builder!`

`key_builder!` finds a key, the same way `t!` does, and returns a key builder: a key that doesn't have its arguments yet.

```rust,ignore
let i18n = use_i18n();

let builder = key_builder!(i18n, hello_world);
let builder = key_builder!(i18n, namespace.subkeys.value);
```

The first argument can be anything `t!` or `td!` accept: a context, a `Locale`, or a scoped one. You can also give a scope type (see `define_scope!` in the scoping chapter) instead of a value, the key is then found in a constant context, at compile time:

```rust,ignore
type SubkeysScope = define_scope!(Locale, namespace.subkeys);

let builder = key_builder!(scope = SubkeysScope, value);
```

### `build_key!`

`build_key!` gives the builder its arguments, with the same syntax as `t!`, and returns the key:

```rust,ignore
// "click_count": "You clicked {{ count }} times"
let builder = key_builder!(i18n, click_count);
let click_count = build_key!(builder, count = move || counter.get());

// "hello_world": "Hello <b>World</b> !"
let builder = key_builder!(i18n, hello_world);
let hello_world = build_key!(builder, <b> = <span />);
```

For a key without arguments, `build_key!(builder)` is enough, and can be used in a constant context.

### `key!`

`key!` does both at once:

```rust,ignore
let click_count = key!(i18n, click_count, count = move || counter.get());
let hello = key!(i18n, hello_world, <b> = <span />);
```

It also accepts the `scope = ..` form: `key!(scope = SubkeysScope, value)`.

## Rendering A Key

Every rendering macro accepts a key value with `key = ..` instead of a path:

```rust,ignore
let i18n = use_i18n();
let hello = key!(i18n, hello_world, <b> = <span />);

view! {
    <p>{t!(i18n, key = hello)}</p>
}
```

The same goes for `td!(locale, key = ..)` and `tu!(i18n, key = ..)`.

## Keys For Strings

The keys above are made to be rendered as views. To get a string out of a key (`t_string!`, `t_display!`), the components arguments must be values implementing `DisplayComponent` instead of views (see the `td_string!` chapter), so the key is built with the display variants:

- `build_key_display!(builder, args..)` instead of `build_key!`
- `display_key!(ctx, path, args..)` instead of `key!`

```rust,ignore
use leptos_i18n::{display_key, t_string};

let hello = display_key!(i18n, hello_world, <b> = "span");

let s: String = t_string!(i18n, key = hello);
```

`t_display!`, `td_string!`, `td_display!`, `tu_string!` and `tu_display!` also accept `key = ..`.

## The Type Of A Key

A key has the type `leptos_i18n::keys::Key<A>`, where `A` depends on the key and its arguments: two different keys don't have the same type. To store keys together, or to pass them around, you can downgrade them to a common type. There are two levels.

### Keys With The Same Arguments

Keys that take the same arguments (same names, same kinds, same formatters, ...) share the same builder. You can downgrade a key builder or a key to that shared type with `.downgrade()`, and they then have the same type as the other keys with the same arguments:

```rust,ignore
// "home": "Home", "about": "About"
let menu = [
    key!(i18n, home).downgrade(),
    key!(i18n, about).downgrade(),
];

view! {
    <ul>
        {menu.into_iter().map(|entry| view! { <li>{t!(i18n, key = entry)}</li> }).collect_view()}
    </ul>
}
```

Downgrading the builder gives the same result, and can be done in a constant context:

```rust,ignore
// "first_click": "You clicked {{ count }} time", "click_count": "You clicked {{ count }} times"
let builder = if counter.get() == 1 {
    key_builder!(i18n, first_click).downgrade()
} else {
    key_builder!(i18n, click_count).downgrade()
};
let key = build_key!(builder, count = move || counter.get());
```

This still uses static dispatch, the key is found with a `match`. The shared types are generated and not meant to be named, so this works best when the type can be inferred. Keys for strings use `.downgrade_display()` instead.

### Any Key

To mix keys with different arguments, or to name the type (in a component's props for example), downgrade them to `AnyKey<Locale>` with `.downgrade_any()` (or call `.downgrade()` on an already downgraded key). This uses dynamic dispatch:

```rust,ignore
use leptos_i18n::keys::AnyKey;

#[component]
fn Title(title: AnyKey<Locale>) -> impl IntoView {
    let i18n = use_i18n();
    view! { <h1>{t!(i18n, key = title)}</h1> }
}

view! {
    <Title title=key!(i18n, home_title).downgrade_any() />
    <Title title=key!(i18n, welcome, name = "John").downgrade_any() />
}
```

`.downgrade_any()` requires the arguments to be `Clone + Send + Sync + 'static`. Keys for strings use `.downgrade_any_display()` instead.
