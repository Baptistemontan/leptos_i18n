This document contain what the `td!` macro should expand to. The `td!` macro output the same code whatever the feature flags, so no flags are relevant.

Code:

```rust
td!(locale, $key)
```

Expanded code:

```rust
{
    #[allow(unused)]
    use leptos_i18n::__private::BuildStr;
    let _key = leptos_i18n::Locale::get_keys(locale).$key;
    _key.build()
}
```

Code:

```rust
td!(locale, $key, $variable = $variable_value)
```

Expanded code:

```rust
{
    let _key = leptos_i18n::Locale::get_keys(locale).$key;
    let _key = _key.var_$variable($variable_value);
    #[deny(deprecated)]
    _key.build()
}
```

Code:

```rust
td!(locale, $key, $variable)
```

Expanded code:

```rust
{
    let _key = leptos_i18n::Locale::get_keys(locale).$key;
    let _key = _key.var_$variable($variable);
    #[deny(deprecated)]
    _key.build()
}
```

Code:

```rust
td!(locale, $key, <$component> = $component_value)
```

Expanded code:

```rust
{
    let _key = leptos_i18n::Locale::get_keys(locale).$key;
    let _key = _key.comp_$component($component_value);
    #[deny(deprecated)]
    _key.build()
}
```

```rust
td!(locale, $key, <$component>)
```

Expanded code:

```rust
{
    let _key = leptos_i18n::Locale::get_keys(locale).$key;
    let _key = _key.comp_$component($component);
    #[deny(deprecated)]
    _key.build()
}
```

Code:

```rust
td!(locale, $key, $variable = $variable_value, <$component> = $component_value)
```

Expanded code:

```rust
{
    let _key = leptos_i18n::Locale::get_keys(locale).$key;
    let _key = _key.var_$variable($variable_value);
    let _key = _key.comp_$component($component_value);
    #[deny(deprecated)]
    _key.build()
}
```
