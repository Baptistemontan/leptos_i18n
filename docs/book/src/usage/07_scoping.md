# Scoping

If you are using subkeys or namespaces, access keys can get pretty big and repetitive,
would'nt it be nice to scope a context to a namespace or subkeys ?

Well this page explain how to do it!

## The `scope_i18n!` macro

Using namespaces and subkeys can make things quite cumbersome very fast, imagine you have this:

```rust
let i18n = use_i18n();

t!(i18n, namespace.subkeys.value);
t!(i18n, namespace.subkeys.more_subkeys.subvalue);
t!(i18n, namespace.subkeys.more_subkeys.another_subvalue);
```

This only use `namespace.subkeys.*` but we have to repeat it everywhere,
well here comes the `scope_i18n!` macro, you can rewrite to:

```rust
let i18n = use_i18n();
let i18n = scope_i18n!(i18n, namespace.subkeys);

t!(i18n, value);
t!(i18n, more_subkeys.subvalue);
t!(i18n, more_subkeys.another_subvalue);
```

This macro can be chained:

```rust
let i18n = use_i18n();
let i18n = scope_i18n!(i18n, namespace);
let i18n = scope_i18n!(i18n, subkeys);

t!(i18n, value);

let i18n = scope_i18n!(i18n, more_subkeys);
t!(i18n, subvalue);
t!(i18n, another_subvalue);
```

## The `use_i18n_scoped!` macro

On the above exemple we do `let i18n = use_i18n();` but only access the context to scope it afterward, we could do

```rust
let i18n = scope_i18n!(use_i18n(), namespace.subkeys);
```

Well this is what the `use_i18n_scoped!` macro is for:

```rust
let i18n = use_i18n_scoped!(namespace.subkeys);

t!(i18n, value);
t!(i18n, more_subkeys.subvalue);
t!(i18n, more_subkeys.another_subvalue);
```

## The `scope_locale!` macro

The above exemples are to scope a context, but maybe you use `td!` a lot and run into the same problems:

```rust
fn foo(locale: Locale) {
    td!(locale, namespace.subkeys.value);
    td!(locale, namespace.subkeys.more_subkeys.subvalue);
    td!(locale, namespace.subkeys.more_subkeys.another_subvalue);
}
```

You can use the `scope_locale!` macro here:

```rust
fn foo(locale: Locale) {
    let locale = scope_locale!(locale, namespace.subkeys);
    td!(locale, value);
    td!(locale, more_subkeys.subvalue);
    td!(locale, more_subkeys.another_subvalue);
}
```

And again, it is chainable:

```rust
fn foo(locale: Locale) {
    let locale = scope_locale!(locale, namespace.subkeys);
    td!(locale, value);
    let locale = scope_locale!(locale, more_subkeys);
    td!(locale, subvalue);
    td!(locale, another_subvalue);
}
```

## Caveat

Unfortunatly, it looks too good te be true... What's the catch ? Where is the tradeoff ?

To make this possible, it use a typestate pattern, but some of the types are hard to access as a user as they defined deep in the generated `i18n` module.
This makes it difficult to write the type of a scoped context or a scoped locale.

By default `I18nContext<L, S>` is only generic over `L` because the the `S` scope is the "default" one provided by `L`, so you can easily write `I18nContext<Locale>`.
But once you scope it the `S` parameters will look like `i18n::namespaces::ns_namespace::subkeys::sk_subkeys::subkeys_subkeys`.

Yes. This is the path to the struct holding the keys of `namespace.subkeys`.

This makes it difficult to pass a scoped type around, as it would require to write `I18nContext<Locale, i18n::namespaces::ns_namespace::subkeys::sk_subkeys::subkeys_subkeys>`.

Maybe in the future there will be a macro to write this horrible path for you, but I don't think it is really needed for now.

If you look at the generated code you will see this:

```rust
let i18n = { leptos_i18n::__private::scope_ctx_util(use_i18n(), |_k| &_k.$keys) };
```

Hummm, what is this closure for? it's just here for type inference and key checking! The function parameter is even `_:fn(&OS) -> &NS`, it's never used.
The function is even const (not for `scope_locale` tho, the only one that could really benefit from it lol, because trait functions can't be const...).

But being a typestate using it or not actually result in the same code path.
And with how agressive Rust is with inlining small functions, it probably compile to the exact same thing.
So no runtime performance loss! Yeaah!
