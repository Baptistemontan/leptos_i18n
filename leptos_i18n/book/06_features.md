# Features

You can find here all the available features of the crate.

#### `actix`

This feature must be enabled when building the server with the actix backend.

#### `axum`

This feature must be enabled when building the server with the axum backend.

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

#### `nightly`

Enable the use of some nightly features, like directly calling the context to get/set the current locale
and allow the `load_locale!` macro to emit better warnings.

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

#### `format_currency`

Allow the use of the `currency` formatter.

#### `unified_contexts`

By default the context are exclusive to their `Locale` enum, this means that one `i18n` module will have a different context than another `i18n` module.
This can either be great if you want them to be kept seperate, or it can be a nightmare to sync all of them.
By enabling the `unified_contexts` feature they all get unified, this means you can provide the context with one `Locale`, and other can still tap into the same context.
If there is a mismatch in supported locale, for example one module support "en" and "fr", and another support "en" and "de", then if the first one set the context to "fr",
the second one will use the default locale.
