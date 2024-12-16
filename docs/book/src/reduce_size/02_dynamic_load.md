# Dynamic loading of translations

## Why use it ?

By default the translations are loaded at compile time and are baked into the binary,
this has some performance advantages but comes at a cost: binary size.
This is fine when the number of keys and locales are small and the values are not long,
but when supporting a high number of locales and with a lot of keys, binary sizes start to highly increase.

The `"dynamic_load"` feature reduce this binary size increase by removing the baked translations in the binary, and lazy load them on the client.
The way it does that is by using a server function to request the translations in a given "translation unit".
What I call "translation unit" is a group of translations, they are either one unit per locale, one unit per locale per namespaces if you use them.

## How it works

When using SSR, the server will register every units used for a given request and bake only the used one in the sent HTML,
they are then parsed when the client hydrate, so no request for translations is done on page load.
When the client need access to an unloaded unit, it will request it to the server and will update the view when received.

## What changes ?

### Async accessors

For obvious reason, with the `"dynamic_load"` accessing a value is now async, `t!`, `td!` and `tu!` still return `impl Fn() -> impl IntoView`, as the async part is handled inside of it with some optimizations, but the `*_display!` and `*_string!` variants now return a future and need to be awaited.

You can turn them into some kind of `Signal<Option<String>>` using leptos `AsyncDerived`:

```rust
let i18n = use_i18n();
let translation = AsyncDerived::new(move || t_string!(i18n, key)); // .get() will return an `Option<&'static str>`
```

Feel free to make yourself a macro to wrap them:

```rust
macro_rules! t_string_async {
    ($($tt:tt),*) => {
        AsyncDerived::new(move || leptos_i18n::t_string!($($tt),*))
    }
}
```

This could have been the design by default, but there is multiple ways to handle it so I decided to leave the choice to the user.

_note_: They are technically not needed to be async on the server, as translations are still baked in for them,
but for the API to be the same on the client and the server they return the value wrapped in an async block.

### Server Fn

If you use a backend that need to manually register server functions,
you can use the `ServerFn` associated type on the `Locale` trait implemented by the generated `Locale` enum:

```rust
use i18n::Locale;
use leptos_i18n::Locale as LocaleTrait;

register_server_fn::<<Locale as LocaleTrait>::ServerFn>();
```

## Final note

Other than that, this is mostly a drop in feature and do not require much from the user.

## Disclaimers

1.  There is a chance that enabling this feature actually increase binary sizes if there isn't much translations,
    as there is additional code being generated to request, parse and load the translations. But this is mostly a fixed cost,
    so with enough translations the trade will be beneficial. So do some testing.

2.  Only the raw strings are removed from the binary, the code to display each keys is still baked in it, whatever the locale or the namespace.
