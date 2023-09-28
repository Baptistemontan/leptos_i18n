This document contain what the `t!` macro should expand to. The `t!` macro output the same code whatever the feature flags, so no flags are relevant.

Code:

```rust
t!(i18n, $key)
```

Expanded code:

```rust
move || {
    #[allow(unused)]
    use leptos_i18n::__private::BuildStr;
    let _key = leptos_i18n::I18nContext::get_keys(i18n).$key;
    _key.build()
}
```

Code:

```rust
t!(i18n, $key, $variable = $variable_value)
```

Expanded code:

```rust
move || {
    let _key = leptos_i18n::I18nContext::get_keys(i18n).$key;
    let _key = _key.var_$variable($variable_value);
    #[deny(deprecated)]
    _key.build()
}
```

Code:

```rust
t!(i18n, $key, $variable)
```

Expanded code:

```rust
move || {
    let _key = leptos_i18n::I18nContext::get_keys(i18n).$key;
    let _key = _key.var_$variable($variable);
    #[deny(deprecated)]
    _key.build()
}
```

Code:

```rust
t!(i18n, $key, <$component> = $component_value)
```

Expanded code:

```rust
move || {
    let _key = leptos_i18n::I18nContext::get_keys(i18n).$key;
    let _key = _key.comp_$component($component_value);
    #[deny(deprecated)]
    _key.build()
}
```

```rust
t!(i18n, $key, <$component>)
```

Expanded code:

```rust
move || {
    let _key = leptos_i18n::I18nContext::get_keys(i18n).$key;
    let _key = _key.comp_$component($component);
    #[deny(deprecated)]
    _key.build()
}
```

Code:

```rust
t!(i18n, $key, $variable = $variable_value, <$component> = $component_value)
```

Expanded code:

```rust
move || {
    let _key = leptos_i18n::I18nContext::get_keys(i18n).$key;
    let _key = _key.var_$variable($variable_value);
    let _key = _key.comp_$component($component_value);
    #[deny(deprecated)]
    _key.build()
}
```
