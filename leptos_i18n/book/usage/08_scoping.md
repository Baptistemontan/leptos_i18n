# Scoping

If you are using subkeys or namespaces, access keys can get pretty big and repetitive,
wouldn't it be nice to scope a context to a namespace or subkeys?

Well, this page explains how to do it!

## The `scope_i18n!` Macro

Using namespaces and subkeys can make things quite cumbersome very fast. Imagine you have this:

```rust,ignore
let i18n = use_i18n();

t!(i18n, namespace.subkeys.value);
t!(i18n, namespace.subkeys.more_subkeys.subvalue);
t!(i18n, namespace.subkeys.more_subkeys.another_subvalue);
```

This only uses `namespace.subkeys.*`, but we have to repeat it everywhere.
Here comes the `scope_i18n!` macro. You can rewrite it to:

```rust,ignore
let i18n = use_i18n();
let i18n = scope_i18n!(i18n, namespace.subkeys);

t!(i18n, value);
t!(i18n, more_subkeys.subvalue);
t!(i18n, more_subkeys.another_subvalue);
```

This macro can be chained:

```rust,ignore
let i18n = use_i18n();
let i18n = scope_i18n!(i18n, namespace);
let i18n = scope_i18n!(i18n, subkeys);

t!(i18n, value);

let i18n = scope_i18n!(i18n, more_subkeys);
t!(i18n, subvalue);
t!(i18n, another_subvalue);
```

## The `use_i18n_scoped!` Macro

In the above example, we do `let i18n = use_i18n();` but only access the context to scope it afterward. We could do

```rust,ignore
let i18n = scope_i18n!(use_i18n(), namespace.subkeys);
```

Well, this is what the `use_i18n_scoped!` macro is for:

```rust,ignore
let i18n = use_i18n_scoped!(namespace.subkeys);

t!(i18n, value);
t!(i18n, more_subkeys.subvalue);
t!(i18n, more_subkeys.another_subvalue);
```

## The `scope_locale!` Macro

The above examples are to scope a context, but maybe you use `td!` a lot and run into the same problems:

```rust,ignore
fn foo(locale: Locale) {
    td!(locale, namespace.subkeys.value);
    td!(locale, namespace.subkeys.more_subkeys.subvalue);
    td!(locale, namespace.subkeys.more_subkeys.another_subvalue);
}
```

You can use the `scope_locale!` macro here:

```rust,ignore
fn foo(locale: Locale) {
    let locale = scope_locale!(locale, namespace.subkeys);
    td!(locale, value);
    td!(locale, more_subkeys.subvalue);
    td!(locale, more_subkeys.another_subvalue);
}
```

And again, it is chainable:

```rust,ignore
fn foo(locale: Locale) {
    let locale = scope_locale!(locale, namespace.subkeys);
    td!(locale, value);
    let locale = scope_locale!(locale, more_subkeys);
    td!(locale, subvalue);
    td!(locale, another_subvalue);
}
```

## The `define_scope!` Macro

Allow to define a marker type that represent a scope, it takes a path to the `i18n` module and the key path:

```rust, ignore
type SubkeysScope = define_scope!(crate::i18n, namespace.subkeys);

fn foo(locale: Locale) {
    let locale = locale.scope::<SubkeysScope>();
    td!(locale, value);

    let locale = scope_locale!(locale, more_subkeys);
    td!(locale, subvalue);
    td!(locale, another_subvalue);
}

fn bar() -> impl IntoView {
    let i18n = use_i18n_scoped::<SubkeysScope>();
    // equivalent to `use_i18n().scope::<SubkeysScope>()`

    t!(i18n, value)
}
```

Unfortunately, can't be extended (yet ?).

This is usefull when you have namespaces/subkeys for large parts of your frontend, you have types to reuse:

```rust, ignore
type SubkeysScope = define_scope!(crate::i18n, namespace.subkeys);

type SubkeysCtx = I18nContext<Locale, SubkeysScope>;

let i18n: SubkeysCtx = use_i18n_scoped();
```

Also allow const scoping of `Locale`, still quite cumbersome though:

```rust, ignore
use leptos_i18n::ConstLocale;

const FR_SUBKEYS: ConstLocale<Locale, SubkeysScope> = ConstLocale::new(Locale::fr);
```

## How it works (for nerds only)

Unfortunately, it looks too good to be true... What's the catch? Where is the tradeoff?

To make this possible, it uses a typestate pattern, but some of the types are hard to access as a user as they are defined deep in the generated `i18n` module.
This makes it difficult to write the type of a scoped context or a scoped locale.

By default, `I18nContext<L, S>` is only generic over `L` because the `S` scope is the "default" one provided by `L`, so you can easily write `I18nContext<Locale>`.
But once you scope it, the `S` parameters will look like `i18n::namespaces::ns_namespace::subkeys::sk_subkeys::subkeys_subkeys`.

Yes. This is the path to the struct holding the keys of `namespace.subkeys`.

This makes it difficult to pass a scoped type around, as it would require writing `I18nContext<Locale, i18n::namespaces::ns_namespace::subkeys::sk_subkeys::subkeys_subkeys>`.

That's the type the `define_scope!` macro returns, it recreate the path and retrieve the underlying type.

If you look at the generated code for `use_i18n_scoped!` you will see something like this:

```rust,ignore
let i18n = { leptos_i18n::__private::scope_ctx_util(use_i18n(), |_k| &_k.$keys) };
```

Hmm, what is this closure for? It’s just here for type inference and key checking! The function parameter is even `_:fn(&OS) -> &NS`, it's never used.
The function is even const (not for `scope_locale!` though).

But being a typestate, using it or not actually results in the same code path.
And with how aggressive Rust is with inlining small functions, it probably compiles to the exact same thing.
