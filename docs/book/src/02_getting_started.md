# Getting started

First thing we need is a `Leptos` project, you can find documentation on how to set one up in the `Leptos` [book](https://leptos-rs.github.io/leptos/02_getting_started.html).

Once you have set one up, you can add this crate to your project with

```bash
cargo add leptos_i18n leptos_i18n_build
```

Or by adding this line to your `Cargo.toml`:

```toml
[dependencies]
leptos_i18n = "0.6"

[build-dependencies]
leptos_i18n_build = "0.6"
```

We actually need 2 crates, we will talk about the second one later

## `actix-web` Backend

When compiling for the backend using `actix-web`, enable the `actix` feature for the `leptos_i18n` crate:

```toml
# Cargo.toml

[features]
ssr = [
    "leptos_i18n/actix",
    "leptos_i18n_build/ssr",
]
```

## `axum` Backend

When compiling for the backend using `axum`, enable the `axum` feature for the `leptos_i18n` crate:

```toml
# Cargo.toml

[features]
ssr = [
    "leptos_i18n/axum",
    "leptos_i18n_build/ssr",
]
```

## Hydrate

When compiling for the client, enable the `hydrate` feature:

```toml
# Cargo.toml

[features]
hydrate = [
    "leptos_i18n/hydrate",
    "leptos_i18n_build/hydrate",
]
```

## Client Side Rendering

When compiling for the client, enable the `csr` feature:

```toml
# Cargo.toml

[dependencies.leptos_i18n]
features = ["csr"]


[build-dependencies.leptos_i18n_build]
features = ["csr"]
```

You can find examples using CSR on the [github repo](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/csr)
