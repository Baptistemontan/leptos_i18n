# Features

You can find here all the available features of the crate.

#### `actix`

This feature must be enabled when building the server with the actix backend

#### `axum`

This feature must be enabled when building the server with the actix backend

### `ssr`

This feature must be enabled when building the server. It is auto enabled by the `actix` or `axum` features, but if you use another backend you can use this feature and provide custom functions to get access to the request headers.

#### `hydrate`

This feature must be enabled when building the client in ssr mode

#### `csr`

This feature must be enabled when building the client in csr mode

#### `cookie` (Default)

Set a cookie to remember the last chosen locale.

#### `sync`

This feature has no impact on the user.
This feature allow the crate to use sync data types such as `Mutex` or `OnceLock`.
Activated when the `actix` or `axum` feature is enabled.

#### `experimental-islands`

This feature is, as it's name says, experimental.
This make this lib somewhat usable when using `islands` with Leptos.

#### `serde`

This feature implement `Serialize` and `Deserialize` for the `Locale` enum

#### `interpolate_display`

This feature generate extra code for each interpolation to allow rendering them as a string instead of a `View`.

#### `show_keys_only`

This feature makes every translations to only display it's corresponding key, this is useful to track untranslated strings in you application.

#### `suppress_key_warnings`

This features disable the warnings when a key is missing or in surplus, we discourage its usage and highly encourage the use of explicit defaults, but if its what's you want, we won't stop you.

#### `json_files` (Default)

To enable when you use JSON files for your locales

#### `yaml_files`

To enable when you use YAML files for your locales

#### `nightly`

Enable the use of some nightly features, like directly calling the context to get/set the current locale.
and allow the `load_locale!` macro to emit better warnings.

#### `track_locale_files`

Allow tracking of locale files as dependencies for rebuilds in stable.
The `load_locales!()` macro using external dependencies the build system is not aware that the macro should be rerun when those files changes,
you may have noticed that if you use `cargo-leptos` with `watch-additional-files = ["locales"]` and running `cargo leptos watch`, even if the file changes and cargo-leptos triggers a rebuild nothing changes.
This feature use a "trick" by using `include_bytes!()` to declare the use of a file, but I'm a bit sceptical of the impact on build time using this.
I've already checked and it does not include the bytes in the final binary, even in debug, but it may slow down compilation time.
If you use the `nightly` feature it use the [path tracking API](https://github.com/rust-lang/rust/issues/99515) so no trick using `include_bytes!` and the possible slowdown in compile times coming with it.
