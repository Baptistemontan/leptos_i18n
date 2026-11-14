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

Allow to define a marker type that represent a scope, it takes a scope and the key path fro that scope (the `Locale` enum is itself a scope):

```rust, ignore
type SubkeysScope = define_scope!(Locale, namespace.subkeys);

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

It can also be extended:

```rust, ignore
type NsScope = define_scope!(Locale, namespace);
type SubkeysScope = define_scope!(NsScope, subkeys);
```

> **note**: In order to support extension, it uses a associated type in a generic trait,
> the path is cut into each individual char and create the marker,
> for exemple `subkeys` becomes `(Marker<'s'>, Marker<'u'>, ..)`,
> this means if the path does'nt exist the compile error will look absolutely terrible (sorry).

This is usefull when you have namespaces/subkeys for large parts of your frontend, you have types to reuse:

```rust, ignore
type SubkeysScope = define_scope!(Locale, namespace.subkeys);

type SubkeysCtx = I18nContext<Locale, SubkeysScope>;

let i18n: SubkeysCtx = use_i18n_scoped();
```

Also allow const scoping of `Locale`, still quite cumbersome though:

```rust, ignore
use leptos_i18n::ScopedLocale;

const FR_SUBKEYS: ScopedLocale<Locale, SubkeysScope> = ScopedLocale::new(Locale::fr);
```
