# Leptos i18n

This crate is made to simplify internalisation in a Leptos application, that load locales at **_compile time_** and provide compile time check for keys and selected locale.

## How to use

### Configuration files

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

### Loading the locales

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

### The `t!()` macro

A typical `i18n.rs` module would look like this:

```rust
leptos_i18n::load_locales!();

#[macro_export]
macro_rules! t {
    ($cx: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales)
    };
    ($cx: ident, $key: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales, $key)
    };
    ($cx: ident, $key: ident, $($variable:ident = $value:expr,)*) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales, $key, $($variable = $value,)*)
    };
}
```

First line is the macro that load and parse the locales and then create the types.

the crate export a macro named `t!()` that help with extracting the local from the context, but it needs the `Locales` type,
so to avoid retyping it every time we can redefine the macro to already contain the path to the `Locales` type.

```rust
view! { cx,
    <p>{t!(cx, hello_world)}</p>
}
```

The `t!(cx)` macro return the current locale, so you can do `t!(cx).key`, the second one, `t!(cx, key)`, wraps it in a closure, it basically expand to `move || t!(cx).key`, but with some optimizations.
The third macro is when interpolation is need.

### Interpolation

You can add variables by wrapping it in `{{  }}` in the locale key definition:

```json
{
  "click_to_inc": "Click to increment",
  "click_count": "You have clicked {{ count }} times"
}
```

You can then do

```rust
let (counter, set_counter) = create_signal(cx, 0);
let inc = move |_| set_counter.update(|count| *count += 1);

view! { cx,
    <p>{t!{ cx, click_count,
        count = move || counter.get()
    }}</p>
    <button on:click=inc>{t!(cx, click_to_inc)}</button>
}

```

You can pass anything that implement `leptos::IntoView` as your variable, it must also be `Clone + 'static`. If a key is not supplied it will not compile, same for an unknown key.

You may also need to interpolate components, you can define them with html tags:

```json
{
  "important_text": "this text is <b>very</b> important"
}
```

You can supply them the same way as variables, but the supplied value must be a `Fn(leptos::Scope, leptos::ChildrenFn) -> impl IntoView`, and also must be `Clone + 'static`.

```rust
view! { cx,
    <p>
        {t!{ cx,
            important_text,
            b = |cx, children| view!{ cx, <b>{children(cx)}</b> },
        }}
    </p>
}

```

You can not define a variable with the same name of a component, you can name them how you want but it has to be a legal rust identifier. You can define variables inside components like `You have clicked <b>{{ count }}</b> times`, and you can nest components, even with the same identifier: `<b><b><b>VERY IMPORTANT</b></b></b>`.

### Context Provider

To make all of that work, it needs to have the `I18nContext` available, for that call the `provide_i18n_context()` function at the highest possible level:

```rust
// root of the application
#[component]
pub fn App(cx: Scope) -> impl IntoView {

    leptos_i18n::provide_i18n_context::<Locales>(cx);

    view! { cx,
        {/* ... */}
    }
}
```

### Setting the locale

You can use the `set_locale` function to change the locale:

```rust
let set_locale = leptos_i18n::set_locale::<Locales>(cx);
let on_click = move |_| set_locale(LocaleEnum::fr);

view! { cx,
    <button on:click=on_click>
        {t!(cx, set_locale_to_french)}
    </button>
}

```

The `t!()` macro suscribe to locale change so every translation will switch to the new locale.

When a new locale is set, a cookie is set on the client side to remember the prefered locale. If you are using Chromium on localhost it may not work, as it blocks cookie set on the client side, try with another browser like Firefox.

### Get the current locale

You can use the `get_variant` function to get the current locale enum, this function return a function, this is to allow to subscribe to the locale update:

```rust
let get_variant = leptos_i18n::get_variant::<Locales>(cx);
let set_locale = leptos_i18n::set_locale::<Locales>(cx);
let on_click = move |_| {
    let locale = get_variant();
    let new_lang = match locale {
        LocaleEnum::en => LocaleEnum::fr,
        LocaleEnum::fr => LocaleEnum::en,
    };

    set_locale(new_lang);
};

view! { cx,
    <h1>{t!(cx, hello_world)}</h1>
    <button on:click=on_click >{t!(cx, switch_locale)}</button>
}
```

if you need access to the actual locale type, you can use the `get_locale` function, which also return a function for reactiveness:

```rust
let get_locale = leptos_i18n::get_locale::<Locales>(cx);

view! { cx,
    <h1>{move || get_locale().hello_world}</h1>
}

```

For keys with interpolation, it implement a builder pattern, so if you have a variable named `count` you can use it like this:

```rust
let get_locale = leptos_i18n::get_locale::<Locales>(cx);

let count = ...;

view! { cx,
    <h1>{move || get_locale().click_count.count(count)}</h1>
}
```

If examples works better for you, you can look at the different examples available on the Github.

## Features

You must enable the `hydrate` feature when building the client, and when building the server you must enable either the `actix` or `axum` feature.

The `cookie` feature enable to set a cookie when a locale is chosen by the user, this feature is enabled by default.

```

```
