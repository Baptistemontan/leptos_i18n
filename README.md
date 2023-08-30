[![crates.io](https://img.shields.io/crates/v/leptos_i18n.svg)](https://crates.io/crates/leptos_i18n)

# Leptos i18n

This crate is made to simplify internalisation in a [Leptos](https://crates.io/crates/leptos) application, that load locales at **_compile time_** and provide compile time check for keys and selected locale.

The main focus is ease of you use with leptos, a typical component using this crate will look like this:

```rust
let i18n = get_i18n_context(cx);

let (counter, set_counter) = create_signal(cx, 0);
let inc = move |_| set_counter.update(|count| *count += 1);


view! { cx,
    {/* click_to_inc = "Click to increment" */}
    <button on:click=inc>{t!(i18n, click_to_inc)}</button>
    {/* click_count = "You have clicked {{ count }} times" */}
    <p>{t!(i18n, click_count, count = move || counter.get())}</p>
}
```

You just need a configuration file named `i18n.json` and one file per locale name `{locale}.json` in the `/locales` folder of your application.

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

The file structure must look like this:

```bash
./locales
├── en.json
└── fr.json
```

And the files must look like this:

`/locales/en.json`:

```json
{
  "hello_world": "Hello World!"
}
```

`/locales/fr.json`:

```json
{
  "hello_world": "Bonjour le monde!"
}
```

All locales files need to have exactly the same keys.

### Loading the locales

You can then use the `load_locales!()` macro in a module of the project, this will load _at compile time_ the locales, and create a struct that describe your locales:

```rust
struct I18nKeys {
    pub hello_world: &'static str
}
```

Two other helper types are created, one enum representing the locales:

```rust
enum LocaleEnum {
    en,
    fr
}
```

and an empty struct named `Locales` that serves as a link beetween the two, it is this one that is the most important, most functions of the crate need this type, not the one containing the locales nor the enum.

### I18nContext

The heart of this library is the `I18nContext`, it must be provided at the highest possible level in the application with the `provide_i18n_context` function:

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

You can then call the `get_context<T>` function to access it:

```rust
let i18n_context = leptos_i18n::get_context::<Locales>(cx);
```

It is advised to make your own function to suppress the need to pass the `Locales` type every time:

```rust
#[inline]
pub fn get_i18n_context(cx: Scope) -> I18nContext<Locales> {
    leptos_i18n::get_context(cx)
}
```

The `provide_i18n_context` function return the context, so instead of

```rust
leptos_i18n::provide_i18n_context::<Locales>(cx);

let i18n = get_i18n_context(cx);
```

You can write

```rust
let i18n = leptos_i18n::provide_i18n_context::<Locales>(cx);
```

The context implement 3 key functions: `.get_locale()`, `.get_keys()` and `.set_locale(locale)`.

### Accessing the current locale

You may need to know what locale is currenly used, for that you can call `.get_locale` on the context, it will return the `LocaleEnum` defined by the `load_locales!()` macro. This function actually call `.get` on a signal, this means you should call it in a function like any signal.

### Accessing the keys

You can access the keys by calling `.get_keys` on the context, it will return the `I18nKeys` struct defined above, build with the current locale. This is also based on the locale signal, so call it in a function too.

### Setting a locale

When the user make a request for your application, the request headers contains a weighted list of accepted locales, this library take them into account and try to match it against the loaded locales, but you probably want to give your users the possibility to manually choose there prefered locale, for that you can set the current locale with the `.set_locale` function:

```rust
let i18n = get_i18n_context(cx);

let on_click = move |_| {
    let current_locale = i18n.get_locale();
    let new_locale = match current_locale {
        LocaleEnum::en => LocaleEnum::fr,
        LocaleEnum::fr => LocaleEnum::en,
    };
    i18n.set_locale(new_locale);
};

view! { cx,
    <button on:click=on_click>
        {move || i18n.get_keys().click_to_switch_locale}
    </button>
}
```

### The `t!()` macro

As seen above, it can be pretty verbose to do `move || i18n.get_keys().$key` every time, so the crate expose a macro to help with that, the `t!()` macro.

```rust
let i18n = get_i18n_context(cx);

view! { cx,
    <p>{t!(i18n, hello_world)}</p>
}
```

It takes the context as the first parameter and the key in second.
It also help with interpolation:

### Interpolation

You may need to interpolate values in your translation, for that you can add variables by wrapping it in `{{  }}` in the locale definition:

```json
{
  "click_to_inc": "Click to increment",
  "click_count": "You have clicked {{ count }} times"
}
```

You can then do

```rust
let i18n = get_i18n_context(cx);

let (counter, set_counter) = create_signal(cx, 0);
let inc = move |_| set_counter.update(|count| *count += 1);


view! { cx,
    <p>{t!(i18n, click_count, count = move || counter.get())}</p>
    <button on:click=inc>{t!(i18n, click_to_inc)}</button>
}
```

You can pass anything that implement `leptos::IntoView + Clone + 'static` as your variable. If a variable is not supplied it will not compile, same for an unknown variable key.

You may also need to interpolate components, to highlight some part of a text for exemple, you can define them with html tags:

```json
{
  "important_text": "this text is <b>very</b> important"
}
```

You can supply them the same way as variables to the `t!` macro, just wrapped beetween `< >`. The supplied value must be a `T: Fn(leptos::Scope, leptos::ChildrenFn) -> impl IntoView + Clone + 'static`.

```rust
let i18n = get_i18n_context(cx);

view! { cx,
    <p>
        {t!(i18n, important_text, <b> = |cx, children| view!{ cx, <b>{children(cx)}</b> })}
    </p>
}
```

The only restriction on variables/components names is that it must be a valid rust identifier (`-` are allowed, but are replaced by `_` for the identifier). You can define variables inside components: `You have clicked <b>{{ count }}</b> times`, and you can nest components, even with the same identifier: `<b><b><i>VERY IMPORTANT</i></b></b>`.

For plain strings, `.get_keys().$key` return a `&'static str`, but for interpolated keys it return a struct that implement a builder pattern where variables are passed to functions called `.var_$name(var)` and components to `.comp_$name(comp)`, so for the counter above but without the `t!` macro it will look like this:

```rust
let i18n = get_i18n_context(cx);

let (counter, set_counter) = create_signal(cx, 0);
let inc = move |_| set_counter.update(|count| *count += 1);


view! { cx,
    <p>{move || i18n.get_keys().click_count.var_count(move || counter.get())}</p>
    <button on:click=inc>{move || i18n.get_keys().click_to_inc}</button>
}
```

If a variable or a component is only needed for one local, it is totally acceptable to do:

`/locales/en.json`:

```json
{
  "hello_world": "Hello World!"
}
```

`/locales/fr.json`:

```json
{
  "hello_world": "Bonjour <i>le monde!</i>"
}
```

When accessing the key it will return a builder that need the total keys of variables/components of every locales.

If your value as the same name as the variable/component, you can drop the assignement, this:

```rust
t!(i18n, key, count = count, <b> = b, other_key = ..)
```

can we shorten to

```rust
t!(i18n, key, count, <b>, other_key = ..)
```

### Plurals

You may need to display different messages depending on a count, for exemple one when there is 0 elements, another when there is only one, and a last one when the count is anything else. For that you can do:

```json
{
  "click_count": {
    "0": "You have not clicked yet",
    "1": "You clicked once",
    "_": "You clicked {{ count }} times"
  }
}
```

When using plurals, variable name `count` is reserved and takes as a value `T: Fn() -> Into<N> + Clone + 'static` where `N` is the specified type.
By default `N` is `i64` but you can change that with the key `type`:

```json
{
  "money_in_da_bank": {
    "type": "f32",
    "0.0": "You are broke",
    "..0.0": "You owe money",
    "_": "You have {{ count }}€"
  }
}
```

The supported types are `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32` and `f64` and he `type` key must be the first.

You can also supply a range:

```json
{
  "click_count": {
    "0": "You have not clicked yet",
    "1": "You clicked once",
    "2..=10": "You clicked {{ count }} times",
    "11..": "You clicked <b>a lot</b>"
  }
}
```

The resulting code looks something like this:

```rust
match N::from(count()) {
    0 => // render "You have not clicked yet",
    1 => // render "You clicked once",
    _ => // render "You clicked {{ count }} times"
}
```

But this exemple will not compile, because the resulting match statement will not cover the full `i64` range (even if your count is not a `i64`, it is till converted to one and need to match the full range), so you will either need to introduce a fallback, or the missing range: `"..0": "You clicked a negative amount ??"`, or set `type` to a unsigned like `u64`.

Because floats (`f32` and `f64`) are not accepted in match statements so it can't be known if the full range is covered, therefore floats must have a fallback (`"_"`) at the end.

Those plurals:

```json
{
  "money_in_da_bank": {
    "type": "f32",
    "0.0": "You are broke",
    "..0.0": "You owe money",
    "_": "You have {{ count }}€"
  }
}
```

Would generate code similar to this:

```rust
let plural_count = f32::from(count());
if plural_count == 0.0 {
  // render "You are broke"
} else if (..0.0).contains(&plural_count) {
  // render "You owe money"
} else {
  // render "You have {{ count }}€"
}
```

If one locale use plurals for a key, another locale does not need to use it, but the `count` variable will still be reserved, but it still can access it as a variable, it will just be constrained to a `T: Fn() -> Into<N> + Clone + 'static`.

You are not required to use the `count` variable in the locale, but it must be provided.

If multiple locales use plurals for the same key, the count `type` must be the same.

(PS: Floats are generaly not a good idea for money.)

You can also have multiple conditions:

```json
{
  "click_count": {
    "type": "u32",
    "0 | 5": "You clicked 0 or 5 times",
    "1": "You clicked once",
    "2..=10 | 20": "You clicked {{ count }} times",
    "11..": "You clicked <b>a lot</b>"
  }
}
```

### Namespaces

Being constrained to put every translation in one unique file can make the locale file overly big, and keys must be unique making things even more complex. To avoid this situation you can introduce namespaces in the config file (i18n.json):

```json
{
  "default": "en",
  "locales": ["en", "fr"],
  "namespaces": ["common", "home"]
}
```

Then your file structures must look like this int the `/locales` directory:

```bash
./locales
├── en
│   ├── common.json
│   └── home.json
└── fr
    ├── common.json
    └── home.json
```

Accessing your values with the `t!` macro will be like this:

```rust
t!(i18n, $namespace.$key)
```

You can have as many namespaces as you want, but the name should be a valid rust identifier (same as component/variable names, `-` are replaced by `_`).

### Examples

If examples works better for you, you can look at the different examples available on the Github.

## Features

You must enable the `hydrate` feature when building the client, and when building the server you must enable either the `actix` or `axum` feature. There is no support for `csr` at the moment.

The `cookie` feature enable to set a cookie when a locale is chosen by the user, this feature is enabled by default.

The `serde` feature implement `serde::Serialize` and `serde::Deserialize` for the locale enum.

The `nightly` feature enable to do `i18n()` to get the locale instead of `i18n.get_locale()` and `i18n(new_locale)` instead of `i18n.set_locale(new_locale)`.

## Contributing

Errors are a bit clunky or obscure for now, there is a lot of edge cases and I did not had time to track every failing scenario, feel free to open an issue on github so I can improve those.

Also feel free to open PR for any improvement or new feature.
