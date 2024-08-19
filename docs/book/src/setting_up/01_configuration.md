# Configuration

This crate in basically entirely based around one macro: the `load_locales!` macro. We will cover it in a later chapter, but for now just know that it looks at your translation files and generates code for them.

To load those translations it first needs to know what to look for, so you need to declare what locales you are supporting and which one is the default.
To do that you use the `[package.metadata.leptos-i18n]` section in your `Cargo.toml`.

To declare `en` and `fr` as locales, with `en` being the default you would write:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
```

There is 2 more optional values you can supply:

- `namespaces`: This is to split your translations in multiple files, we will cover it in a later chapter
- `locales-dir`: This is to have a custom path to the directory containing the locales files, it defaults to `"./locales"`.

Once this configuration is done, you can start writing your translations.
