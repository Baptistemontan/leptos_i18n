[![crates.io](https://img.shields.io/crates/v/leptos_i18n.svg)](https://crates.io/crates/leptos_i18n)

[Docs.rs](https://docs.rs/leptos_i18n/latest/leptos_i18n/) | [Book](https://baptistemontan.github.io/leptos_i18n)

# Leptos i18n

This crate is made to simplify internationalization in a [Leptos](https://crates.io/crates/leptos) application, that loads locales at **_compile time_** and provides compile time checks for translation keys, interpolation keys and the selected locale.

The main focus is ease of use with leptos, a typical component using this crate will look like this:

```rust
use crate::i18n::*;
use leptos::*;

#[component]
fn Counter() -> impl IntoView {
  let i18n = use_i18n();

  let (counter, set_counter) = create_signal(0);
  let inc = move |_| set_counter.update(|count| *count += 1);


  view! {
    <button on:click=inc>
      {/* click_to_inc = "Click to increment" */}
      {t!(i18n, click_to_inc)}
    </button>
    <p>
      {/* click_count = "You have clicked {{ count }} times" */}
      {t!(i18n, click_count, count = move || counter.get())}
     </p>
  }
}
```

You just need to declare the locales in you `Cargo.toml` and one file per locale named `{locale}.json` in the `/locales` folder of your application.

## Getting started

You can add the crate to your project with

```bash
cargo add leptos_i18n
```

Or by adding this line to your `Cargo.toml` under `[dependencies]`:

```toml
leptos_i18n = "0.3"
```

## Version compatibility with leptos

| Leptos     | Leptos i18n   |
| ---------- | ------------- |
| `< v0.4.x` | not supported |
| `v0.4.x`   | `v0.1.x`      |
| `v0.5.x`   | `v0.2.x`      |
| `v0.6.x`   | `v0.3.x`      |

## How to use

### Configuration files

First You need to declare your locales in your cargo manifest `Cargo.toml`:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
```

You can then put your translations files in the `/locales` directory at root of the project, they should be named `{locale}.json`, one per locale declared in the configuration.

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

If you need your locales to be in a different folders than `./locales` you can specify the path in the configuration:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
locales-dir = "./path/to/locales"
```

### Other file format

By default the macro expect JSON files, you can remove the default features and add the `${format}_files` feature to change what file format to use.
Supported formats are:

- JSON
- YAML

The crate won't compile if you enable multiple of them at the same time, or none

### Loading the locales

You can then use the `leptos_i18n::load_locales!()` macro, this will load _at compile time_ the locales, and create a module named `i18n` that expose multiple things:

#### The keys

The macro create a struct `I18nKeys` that represent your declared translations:

```rust
struct I18nKeys {
    pub hello_world: &'static str
}
```

#### The declared locales

It also create an enum that describe the supported locales:

```rust
enum Locale {
    en,
    fr
}
```

#### Helper functions

The `i18n` module also exposes 2 functions: `provide_i18n_context` and `use_i18n`.

### I18nContext

The heart of this library is the `I18nContext`, it must be provided at the highest possible level in the application with the `provide_i18n_context` function created with the `i18n` module:

```rust
use crate::i18n::provide_i18n_context;

// root of the application
#[component]
pub fn App() -> impl IntoView {

    provide_i18n_context();

    view! {
        /* ... */
    }
}
```

You can then call the `use_i18n` function in the `i18n` module to access it:

```rust
use crate::i18n::use_i18n;
let i18n_context = use_i18n();
```

The `provide_i18n_context` function return the context, so instead of

```rust
use crate::i18n::{use_i18n, provide_i18n_context};
provide_i18n_context();

let i18n = use_i18n();
```

You can write

```rust
use crate::i18n::provide_i18n_context;
let i18n = provide_i18n_context();
```

The context implement 3 key functions: `.get_locale()`, `.get_keys()` and `.set_locale(locale)`.

### Accessing the current locale

You may need to know what locale is currenly used, for that you can call `.get_locale` on the context, it will return the `Locale` defined by the `load_locales!()` macro. This function actually call `.get` on a signal, this means you should call it in a function like any signal.

### Accessing the keys

You can access the keys by calling `.get_keys` on the context, it will return the `I18nKeys` struct defined above, build with the current locale. This is also based on the locale signal, so call it in a function too.

### Setting a locale

When the user make a request for your application, the request headers contains a weighted list of accepted locales, this library take them into account and try to match it against the loaded locales, but you probably want to give your users the possibility to manually choose there prefered locale, for that you can set the current locale with the `.set_locale` function:

```rust
let i18n = use_i18n();

let on_click = move |_| {
    let current_locale = i18n.get_locale();
    let new_locale = match current_locale {
        Locale::en => Locale::fr,
        Locale::fr => Locale::en,
    };
    i18n.set_locale(new_locale);
};

view! {
    <button on:click=on_click>
        {move || i18n.get_keys().click_to_switch_locale}
    </button>
}
```

### The `t!()` macro

As seen above, it can be pretty verbose to do `move || i18n.get_keys().$key` every time, so the crate expose a macro to help with that, the `t!()` macro.

```rust
use crate::i18n::use_i18n;
use leptos_i18n::t;

let i18n = use_i18n();

view! {
    <p>{t!(i18n, hello_world)}</p>
}
```

It takes the context as the first parameter and the key in second.

Because you often use the`t!` macro with the `i18n` module, the `i18n` module re-export it, so you can do `use crate::i18n::*` to import the `use_i18n` function and the `t!` macro together.

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
let i18n = use_i18n();

let (counter, set_counter) = create_signal(0);
let inc = move |_| set_counter.update(|count| *count += 1);


view! {
    <p>{t!(i18n, click_count, count = move || counter.get())}</p>
    <button on:click=inc>{t!(i18n, click_to_inc)}</button>
}
```

You can pass anything that implement `leptos::IntoView + Clone + 'static` as your variable. If a variable is not supplied it will not compile, same for an unknown variable key.

You may also need to interpolate components, to highlight some part of a text for example, you can define them with html tags:

```json
{
  "important_text": "this text is <b>very</b> important"
}
```

You can supply them the same way as variables to the `t!` macro, just wrapped beetween `< >`. The supplied value must be a `T: Fn(leptos::ChildrenFn) -> impl IntoView + Clone + 'static`.

```rust
let i18n = use_i18n();

view! {
    <p>
        {t!(i18n, important_text, <b> = |children| view!{ <b>{children}</b> })}
    </p>
}
```

The only restriction on variables/components names is that it must be a valid rust identifier (`-` are allowed, but are replaced by `_` for the identifier). You can define variables inside components: `You have clicked <b>{{ count }}</b> times`, and you can nest components, even with the same identifier: `<b><b><i>VERY IMPORTANT</i></b></b>`.

For plain strings, `.get_keys().$key` return a `&'static str`, but for interpolated keys it return a struct that implement a builder pattern where variables are passed to functions called `.var_$name(var)` and components to `.comp_$name(comp)`, so for the counter above but without the `t!` macro it will look like this:

```rust
let i18n = use_i18n();

let (counter, set_counter) = create_signal(0);
let inc = move |_| set_counter.update(|count| *count += 1);


view! {
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

You may need to display different messages depending on a count, for example one when there is 0 elements, another when there is only one, and a last one when the count is anything else.

You declare them in a sequence of plurals with a sequence where the first element is the value and the other elements are the counts:

```json
{
  "click_count": [
    ["You have not clicked yet", "0"],
    ["You clicked once", 1],
    ["You clicked {{ count }} times", "_"]
  ]
}
```

The count can be a string `"0"` or a litteral `0`.

When using plurals, variable name `count` is reserved and takes as a value `T: Fn() -> N + Clone + 'static` where `N` is the specified type.
By default `N` is `i32` but you can change that by specifying the type as the **first** value in the sequence:

```json
{
  "money_count": [
    "f32",
    ["you are broke", 0.0]
    ["You owe money", "..0.0"],
    ["You have {{ count }}€"]
  ]
}
```

The supported types are `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32` and `f64`.

As seen above with the second plural you can supply a range: `s..e`, `..e`, `s..`, `s..=e`, `..=e` or even `..` ( `..` will considered fallback `_`)

The resulting code looks something like this:

```rust
match N::from(count()) {
    0 => // render "You have not clicked yet",
    1 => // render "You clicked once",
    2..=20 => // render "You clicked beetween 2 and 20 times"
    _ => // render "You clicked {{ count }} times"
}
```

Because it expand to a match statement, a compilation error will be produced if the full range of `N` is not covered.

But floats (`f32` and `f64`) are not accepted in match statements, so they expand to a `if-else` chain, therefore must end by a `else` block, so a fallback `_` or `..` is required.

The plural above would generate code similar to this:

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

If one locale use plurals for a key, another locale does not need to use it, but the `count` variable will still be reserved, but it still can access it as a variable, it will just be constrained to a `T: Fn() -> N + Clone + 'static`.

You are not required to use the `count` variable in the locale, but it must be provided.

If multiple locales use plurals for the same key, the count type must be the same.

(PS: Floats are generaly not a good idea for money.)

```json
{
  "click_count": [
    "u32",
    ["You clicked 0 or 5 times", 0, 5]
    ["You clicked once", 1],
    ["You clicked {{ count }} times", "2..=10", 20]
    ["You clicked 30 or 40 times", 30, 40],
    ["value": "You clicked <b>a lot</b>"]
  ]
}
```

If a plural is a fallback it can only supply the value: `["fallback value"]`

To supply the count for the plural in the `t!` macro, use `$`:

```rust
let count = move || counter.get();
t!(i18n, click_count, $ = count)
```

### Subkeys

You may want to compartmentalize your locales for specific area of your application, you can do this with subkeys:

```json
{
  "parent_key": {
    "child_key_1": "this is a child key",
    "child_key_2": "this is a <b>second</b> child key"
  }
}
```

You can then access your nested keys like this:

```rust
t!(i18n, parent_key.child_key_1)
t!(i18n, parent_key.child_key_2, <b>)
```

You can nest how many you want, but must have the same subkeys across all locales and follow the same interpolation/plurals rules as normal keys.

### Namespaces

Being constrained to put every translation in one unique file can make the locale file overly big, and keys must be unique making things even more complex. To avoid this situation you can introduce namespaces in the configuration:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
namespaces = ["common", "home"]
```

Then your file structures must look like this in the `/locales` directory:

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

To differentiate beetween namespaces and subkeys you can put `::` after the namespace (this is optionnal):

```rust
t!(i18n, $namespace::$key.$subkey)
```

You can have as many namespaces as you want, but the name should be a valid rust identifier (same as component/variable names, `-` are replaced by `_`).

### The `td!` macro (`d` for direct)

The `td!` macro works just like the `t!` macro but instead of taking the context as it first argument it directly take the locale:

```rust
td!(Locale::fr, $key, ...)
```

This let you use a translation regardless of the the current locale, enabling the use of multiple locales at the same time:

```rust
use crate::i18n::*;

view! {
  <p>"In English:"</p>
  <p>{td!(Locale::en, hello_world)}</p>
  <p>"En Français:"</p>
  <p>{td!(Locale::fr, hello_world)}</p>
}
```

(It's a shame `const` function are not allowed in traits, if that was the case the code outputed by `td!` would be entirly const, making it the same as directly pasting the locale)

### Foreign keys

Foreign keys let you re-use already declared translations, you declare them like variables but with a '@' before the path:

```json
{
  "hello_world": "Hello World!",
  "reuse": "message: {{ @hello_world }}"
}
```

This will replace `{{ @hello_world }}` by the value of the key `hello_world`, making `reuse` equal to `"message: Hello World!"`.

#### Supply arguments

You can also supply arguments to fill variables of the pointed key:

```json
{
  "click_count": "You clicked {{ count }} times",
  "clicked_twice": "{{ @click_count, count = 'two' }}"
}
```

This will result to `clicked_twice` to have the value `"You clicked two times"`.

Arguments must be string, delimited by either single quotes or double quotes.

**Note**: Any argument with no matching variable are just discarded, they will not emit any warning/error.

### Book

A more in-depth doc is available on github, the [book](https://baptistemontan.github.io/leptos_i18n)

### Examples

If examples works better for you, you can look at the different examples available on the Github. If something is missing or not clear feel free to open a discussion on github!

## Features

You must enable the `hydrate` feature when building the client, either the `actix` or `axum` feature when building the server, and the `csr` feature when building with CSR. Only one of these features
should be enabled at a time.

The `cookie` feature enable to set a cookie when a locale is chosen by the user, this feature is enabled by default.

The `serde` feature implement `serde::Serialize` and `serde::Deserialize` for the locale enum.

The `nightly` feature enable to do `i18n()` to get the locale instead of `i18n.get_locale()` and `i18n(new_locale)` instead of `i18n.set_locale(new_locale)`.
It also allow macros to use unstable APIs for better warnings.

The `debug_interpolations` feature enable the macros to generate code to emit a warning if a key is supplied twice in interpolations and a better compilation error when a key is missing.
Better compilation errors are generated for interpolations with 4 keys or less.
This is a feature as this code is not "necessary" and could slow compile times,
advice is to enable it for debug builds but disable it for release builds.

The `show_keys_only` feature makes every translations to only display it's corresponding key, this is usefull to track untranslated strings in you application.

The `suppress_key_warnings` feature remove the warning emission of the `load_locales!()` macro when some keys are missing or ignored.

The `json_files` feature tell the macro to expect JSON files for the locales, enabled by default

The `yaml_files` feature tell the macro to expect YAML files for the locales

The `track_locale_files` feature is to track files for rebuilds. The `load_locales!()` macro using external dependencies the build system is not aware that the macro should be rerun when those files changes,
you may have noticed that if you use `cargo-leptos` with `watch-additional-files = ["locales"]` and running `cargo leptos watch`, even if the file changes and cargo-leptos triggers a rebuild nothing changes.
This feature use a "trick" by using `include_bytes!()` to declare the use of a file, but I'm a bit sceptical of the impact on build time using this.
I've already checked and it does not include the bytes in the final binary, even in debug, but it may slow down compilation time.
If you use the `nighly` feature it use the [path tracking API](https://github.com/rust-lang/rust/issues/99515) so no trick using `include_bytes!` and the possible slowdown in compile times coming with it.

## Contributing

Errors are a bit clunky or obscure for now, there is a lot of edge cases and I did not had time to track every failing scenario, feel free to open an issue on github so I can improve those.

Also feel free to open PR for any improvement or new feature.
