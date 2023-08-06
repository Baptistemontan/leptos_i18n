# Leptos i18n

This crate is made to simplify internalisation in a Leptos application.

## How to use

There are files that need to exist, the first one is the `i18n.json` file that describe the default locale and supported locales, it need to be at the root of the project and look like this:

```json
{
  "default": "en",
  "locales": ["en", "fr"]
}
```

The other ones are the files containing the translation, they are key-value pairs and need to be situated in the `/locales` directory at root of the project, they should be named `{locale}.json`, one per locale defined in the `i18n.json` file.
They look like this:

```
/locales/en.json

{
    "hello_world": "Hello World!"
}

/locales/fr.json

{
    "hello_world": "Bonjour le monde!"
}

```

All locales files need to have exactly the same keys.

you can then use the `load_locales!()` macro in a module of the project, this will load _at compile time_ the locales, and create a struct that describe your locales:

```rust
struct Locale {
    pub hello_world: &'static str
}
```

Two other helper types are created, one enum representing the locales:

```rust
enum LocalesVariants {
    en,
    fr
}
```

and an empty struct named `Locales` that serves as a link beetween the two, it is this one that is the most important, most functions of the crate need this type, not the one containing the locales nor the enum.

A typical `i18n.rs` module would look like this:

```rust
leptos_i18n::load_locales!();

#[macro_export]
macro_rules! t {
    ($cx: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales)
    };
    ($cx: ident, $key: ident) => {
        move || t!($cx).$key
    };
}
```

First line is the macro that load and parse the locales and then create the types.

the crate export a macro named `t!()` that help with extracting the local from the context, but it needs the `Locales` type,
so to avoid retyping it every time we can redefine the macro to already contain the path to the `Locales` type.

The first macro version return the entire locale struct, and you can access every key, the second one is when you just want to put the string in the html:

```rust
view! { cx,
    <p>{t!(cx, hello_world)}</p>
}
```

by wrapping it in a function it allows it to be reactive and if the selected locale change it will display the correct one.

To make all of that work, it needs to have a I18nContext available, you wrap your application in the `I18nContextProvider`:

```rust
use leptos_i18n::I18nContextProvider;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    leptos_meta::provide_meta_context(cx);

    view! { cx,
        <I18nContextProvider locales=Locales>
            {/* ... */}
        </I18nContextProvider>
    }
}
```

You must provide you `Locales` type to the context provider so it can infer the needed related types, this type being an empty struct it can be created for 0 cost.

If examples works better for you, you can look at the different examples available on the Github.

## Features

You must enable the `hydrate` feature when building the client, and when building the server you must enable either the `actix` or `axum` feature (axum is not available for now, but will be).

## What's to come ?

The two main focus are to be able to interpolate values in the translation, so you could have

```json
{
  "bananas": "Henry as {{ count }} bananas"
}
```

and being able to do something like this:

```rust
let bananas = ...;

view! { cx,
    <p>{t!(cx, hello_world, count = bananas)}</p>
}
```

The second one is a feature to set a cookie when a locale is selected so when reloading the page the server can know what is the prefered locale, the server side is already handle, but for now the client don't set the cookie. I am no expert, and cookies seems to be a pain to set client side and need some authorization in a manifest, I did not go deep in the research, but I want to make it work in the future.
