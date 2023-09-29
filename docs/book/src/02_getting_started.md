# Getting started

First thing we need is a `Leptos` project, you can find documentation on how to set one up in the `Leptos` [book](https://leptos-rs.github.io/leptos/02_getting_started.html).

Once you have set one up, you can add this crate to your project with

```bash
cargo add leptos_i18n@0.2.0-rc
```

## `actix-web` Backend

When compiling for the backend using `actix-web`, enable the `actix` feature:

```toml
# Cargo.toml

[features]
ssr = [
    "leptos_i18n/actix",
]
```

You can see an exemple using `actix-web` [here](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/hello_world_actix)

## `axum` Backend

When compiling for the backend using `axum`, enable the `axum` feature:

```toml
# Cargo.toml

[features]
ssr = [
    "leptos_i18n/axum",
]
```

You can see an exemple using `axum` [here](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/hello_world_axum)

## Hydrate

When compiling for the client, enable the `hydrate` feature:

```toml
# Cargo.toml

[features]
hydrate = [
    "leptos_i18n/hydrate",
]
```

There exist 3 exemples using hydratation:

- [Hello World Actix](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/hello_world_actix)
- [Hello World Axum](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/hello_world_axum)
- [Using Workspace](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/workspace)

## Client Side Rendering

When compiling for the client, enable the `csr` feature:

```toml
# Cargo.toml

[dependencies.leptos_i18n]
features = ["csr"]
```

You can find an exemple using CSR [here](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/csr)

> **Note**: This version of the book and the linked exemples reflects the upcoming `Leptos_i18n` `v0.2.0` release, using upcoming `Leptos` `v0.5.0`.
