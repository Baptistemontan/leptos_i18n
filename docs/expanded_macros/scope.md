This document contain what the different scoping macros should expand to. Those macros output the same code whatever the feature flags, so no flags are relevant.

# `use_i18n_scoped!`

Code:

```rust
let i18n = use_i18n_scoped!($keys);
```

Expanded code:

```rust
let i18n = {
    leptos_i18n::__private::scope_ctx_util(use_i18n(), |_k| &_k.$keys)
};
```

# `scope_i18n!`

Code:

```rust
let i18n = scope_i18n!($ctx, $keys);
```

Expanded code:

```rust
let i18n = {
    leptos_i18n::__private::scope_ctx_util($ctx, |_k| &_k.$keys)
};
```

Yes. `use_i18n_scoped!($keys)` is just `scope_i18n!(use_i18n(), $keys)`.

# `scope_locale!`

Code:

```rust
let locale = scope_i18n!($locale, $keys);
```

Expanded code:

```rust
let locale = {
    leptos_i18n::__private::scope_locale_util($locale, |_k| &_k.$keys)
};
```
