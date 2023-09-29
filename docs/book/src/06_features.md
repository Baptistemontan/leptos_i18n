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

#### `suppress_key_warnings`

This features disable the warnings when a key is missing or in surplus, we discourage its usage and highly encourage the use of explicit defaults, but if its what's you want, we won't stop you.

#### `json_files` (Default)

To enable when you use JSON files for your locales

#### `yaml_files`

To enable when you use YAML files for your locales

#### `cookie` (Default)

Set a cookie to remember the last chosen locale.

#### `nightly`

Enable the use of some nighly features, like directly calling the context to get/set the current locale, also allow the `load_locale!` macro to emit better warnings.
