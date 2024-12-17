# Features

You can find here all the available features of the crate.

#### `actix`

This feature must be enabled when building the server with the actix backend.

#### `axum`

This feature must be enabled when building the server with the actix backend.

### `ssr`

This feature must be enabled when building the server. It is automatically enabled by the `actix` or `axum` features, but if you use another backend, you can use this feature and provide custom functions to get access to the request headers.

#### `hydrate`

This feature must be enabled when building the client in ssr mode.

#### `csr`

This feature must be enabled when building the client in csr mode.

#### `cookie` (Default)

Set a cookie to remember the last chosen locale.

#### `islands`

This feature is, as its name says, experimental.
This makes this lib somewhat usable when using `islands` with Leptos.

#### `serde`

This feature implements `Serialize` and `Deserialize` for the `Locale` enum.

#### `interpolate_display`

This feature generates extra code for each interpolation to allow rendering them as a string instead of a `View`.

#### `show_keys_only`

This feature makes every translation to only display its corresponding key; this is useful to track untranslated strings in your application.

#### `suppress_key_warnings`

This feature disables the warnings when a key is missing or in surplus; we discourage its usage and highly encourage the use of explicit defaults, but if it’s what you want, we won't stop you.

#### `json_files` (Default)

To enable when you use JSON files for your locales.

#### `yaml_files`

To enable when you use YAML files for your locales.

#### `nightly`

Enable the use of some nightly features, like directly calling the context to get/set the current locale
and allow the `load_locale!` macro to emit better warnings.

#### `track_locale_files`

Allow tracking of locale files as dependencies for rebuilds in stable.
The `load_locales!()` macro uses external dependencies that the build system is not aware of. The macro should be rerun when those files changes,
you may have noticed that if you use `cargo-leptos` with `watch-additional-files = ["locales"]` and running `cargo leptos watch`, even if the file changes and cargo-leptos triggers a rebuild, nothing changes.
This feature uses a "trick" by using `include_bytes!()` to declare the use of a file, but I'm a bit sceptical of the impact on build time using this.
I've already checked and it does not include the bytes in the final binary, even in debug, but it may slow down compilation time.
If you use the `nightly` feature, it uses the [path tracking API](https://github.com/rust-lang/rust/issues/99515) so no trick using `include_bytes!` and the possible slowdown in compile times coming with it.

#### `icu_compiled_data` (Default)

ICU4X is used as a backend for formatting and plurals. They bring their own data to know what to do for each locale. This is great when starting up a project without knowing exactly what you need. This is why it is enabled by default, so things work right out of the box.
But those baked data can take quite a lot of space in the final binary as they bring information for all possible locales, so if you want to reduce this footprint, you can disable this feature and provide your own data with selected information. See the datagen section in the reduce binary size chapter for more information.

#### `plurals`

Allow the use of plurals in translations.

#### `format_datetime`

Allow the use of the `date`, `time`, and `datetime` formatters.

#### `format_list`

Allow the use of the `list` formatter.

#### `format_nums`

Allow the use of the `number` formatter.
