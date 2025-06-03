# Dynamic loading of translations

## Why use it ?

By default, the translations are loaded at compile time and are baked into the binary,
this has some performance advantages but comes at a cost: binary size.
This is fine when the number of keys and locales is small and the values are not long,
but when supporting a high number of locales and with a lot of keys, binary sizes start to increase highly.

The `"dynamic_load"` feature reduces this binary size increase by removing the baked translations in the binary and lazy loading them on the client.
The way it does that is by using a server function to request the translations in a given "translation unit".
What I call "translation unit" is a group of translations; they are either one unit per locale or one unit per locale per namespaces if you use them.

## How it works

When using SSR, the server will register every unit used for a given request and bake only the used one in the sent HTML.
They are then parsed when the client hydrates, so no request for translations is done on page load.
When the client needs access to an unloaded unit, it will request it from the server and will update the view when received.

## What changes ?

### Async accessors

For obvious reasons, with the `"dynamic_load"` accessing a value is now async, `t!`, `td!` and `tu!` still return `impl Fn() -> impl IntoView`, as the async part is handled inside of it with some optimizations, but the `*_display!` and `*_string!` variants now return a future and need to be awaited.

You can turn them into some kind of `Signal<Option<String>>` using leptos `AsyncDerived`:

```rust,ignore
let i18n = use_i18n();
let translation = AsyncDerived::new(move || t_string!(i18n, key)); // .get() will return an `Option<&'static str>`
```

Feel free to make yourself a macro to wrap them:

```rust,ignore
macro_rules! t_string_async {
    ($($tt:tt),*) => {
        leptos::prelude::AsyncDerived::new(move || leptos_i18n::t_string!($($tt),*))
    }
}
```

This could have been the design by default, but there are multiple ways to handle it so I decided to leave the choice to the user.

_note_: They are technically not needed to be async on the server, as translations are still baked in for them,
but for the API to be the same on the client and the server they return the value wrapped in an async block.

### Server Fn

If you use a backend that needs to manually register server functions,
you can use the `ServerFn` associated type on the `Locale` trait implemented by the generated `Locale` enum:

```rust,ignore
use i18n::Locale;
use leptos_i18n::Locale as LocaleTrait;

register_server_fn::<<Locale as LocaleTrait>::ServerFn>();
```

### CSR

With SSR the translations are served by a server functions, but they don't exist with CSR, so you will need to create a static JSON containing them, so a bit more work is needed.
To do that you can use a build script and use the `leptos_i18n_build` crate:

```toml
# Cargo.toml
[build-dependencies]
leptos_i18n_build = "0.5.0"
```

```rust
use leptos_i18n_build::TranslationsInfos;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let translations_infos = TranslationsInfos::parse(Default::default()).unwrap();

    translations_infos.rerun_if_locales_changed();

    translations_infos
        .get_translations()
        .write_to_dir("path/to/dir")
        .unwrap();
}
```

This will generate the need JSON files in the given directory, for exemple you could generate them in `target/i18n`, giving this file structure:

```bash
./target
└── i18n
    ├── locale1.json
    └── locale2.json
```

If you are using namespaces it would have this one:

```bash
./target
└── i18n
    └── namespace1
        ├── locale1.json
        └── locale2.json
    └── namespace2
        ├── locale1.json
        └── locale2.json
```

Then if you are using Trunk you just have to add the directory to the build pipeline:

```html
<!-- index.html -->
<!DOCTYPE html>
<html>
  <head>
    <link data-trunk rel="copy-dir" href="./target/i18n" />
  </head>
  <body></body>
</html>
```

Now the translations will be available at `i18n/{locale}.json`
To inform `leptos_i18n` where to find those translations you need to supply the `translations-path` field under `[package.metadata.leptos-i18n]`:

```toml
# Cargo.toml
[package.metadata.leptos-i18n]
translations-path = "i18n/{locale}.json" # or "i18n/{namespace}/{locale}.json" when using namespaces
```

And this is it!

## Disclaimers

1.  There is a chance that enabling this feature actually increases binary sizes if there aren’t many translations,
    as there is additional code being generated to request, parse, and load the translations. But this is mostly a fixed cost,
    so with enough translations, the trade will be beneficial. So do some testing.

2.  Only the raw strings are removed from the binary; the code to render each key is still baked in it, whatever the locale or the namespace.
