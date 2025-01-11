# Load The Translations

Loading all those translations is the role of the `load_locales!` macro. Just call this macro anywhere in your codebase, and it will generate the code needed to use your translations.

```rust,ignore
// lib.rs/main.rs

leptos_i18n::load_locales!();
```

## The `i18n` module

The macro will generate a module called `i18n`. This module contains everything you need to use your translations.

### The `Locale` enum

You can find the enum `Locale` in this module. It represents all the locales you declared. For example, this configuration:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
```

Generate this enum:

```rust,ignore
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
#[allow(non_camel_case_types)]
pub enum Locale {
    #[default]
    en,
    fr
}
```

### The `I18nKeys` struct

This generated struct represents the structure of your translations, with each translation key being a key in this struct.

It contains an associated constant for each locale, where every field is populated with the values for the locale.

`en.json`

```json
{
  "hello_world": "Hello World!"
}
```

`fr.json`

```json
{
  "hello_world": "Bonjour le Monde!"
}
```

This will generate this struct:

```rust,ignore
pub struct I18nKeys {
  pub hello_world: &'static str,
}

impl I18nKeys {
  const en: Self = I18nKeys { hello_world: "Hello World!" };
  const fr: Self = I18nKeys { hello_world: "Bonjour le Monde!" };
}

leptos_i18n::load_locales!();

assert_eq!(i18n::I18nKeys::en.hello_world, "Hello World!");
assert_eq!(i18n::I18nKeys::fr.hello_world, "Bonjour le Monde!");
```

This way of accessing the values is possible, but it's not practical and most importantly not reactive. We will cover the `t!` macro later, which lets you access the values based on the context:

```rust,ignore
t!(i18n, hello_world)
```
