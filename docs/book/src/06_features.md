# Features

You can find here all the available features of the crate.

#### `actix`

This feature must be enabled when building the server with the actix backend

#### `axum`

This feature must be enabled when building the server with the actix backend

#### `hydrate`

This feature must be enabled when building the client in ssr mode

#### `csr`

This feature must be enabled when building the client in csr mode

#### `serde`

This feature implement `Serialize` and `Deserialize` for the `Locale` enum

#### `debug_interpolations`

This features allow the `load_locales!` macro to generate more code for interpolations, allowing better error reporting when keys are missing.

#### `show_keys_only`

This feature makes every translations to only display it's corresponding key, this is usefull to track untranslated strings in you application.

#### `suppress_key_warnings`

This features disable the warnings when a key is missing or in surplus, we discourage its usage and highly encourage the use of explicit defaults, but if its what's you want, we won't stop you.

#### `json_files` (Default)

To enable when you use JSON files for your locales

#### `yaml_files`

To enable when you use YAML files for your locales

#### `cookie` (Default)

Set a cookie to remember the last chosen locale.

#### `nightly`

- Enable the use of some nightly features, like directly calling the context to get/set the current locale.
- Allow the `load_locale!` macro to emit better warnings.

#### `track_locale_files`

Allow tracking of locale files as dependencies for rebuilds in stable.
The `load_locales!()` macro using external dependencies the build system is not aware that the macro should be rerun when those files changes,
you may have noticed that if you use `cargo-leptos` with `watch-additional-files = ["locales"]` and running `cargo leptos watch`, even if the file changes and cargo-leptos triggers a rebuild nothing changes.
This feature use a "trick" by using `include_bytes!()` to declare the use of a file, but I'm a bit sceptical of the impact on build time using this.
I've already checked and it does not include the bytes in the final binary, even in debug, but it may slow down compilation time.
If you use the `nightly` feature it use the [path tracking API](https://github.com/rust-lang/rust/issues/99515) so no trick using `include_bytes!` and the possible slowdown in compile times coming with it.
