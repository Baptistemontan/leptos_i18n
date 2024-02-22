This document contain what the `td!` macro should expand to. The `td!` macro output the same code whatever the feature flags, so no flags are relevant.

Code:

```rust
td!(locale, $key)
```

Expanded code:

```rust
{
    move || {
        #[allow(unused)]
        use leptos_i18n::__private::BuildStr;
        let _key = leptos_i18n::Locale::get_keys(locale).$key;
        _key.build()
    }
}
```

Code:

```rust
td!(locale, $key, $variable = $value_expr)
```

Expanded code:

```rust
{
    // this is for the possibility that $value_expr is doing some work, like `value.clone()`,
    // we don't want to move `value` in the closure but the computed value.
    // it's done with a tuple to avoid name collisions, if multiple variables were passed we are sure to not shadow a variable used in a next expression.
    let ($variable,) = ($value_expr,);
    move || {
        let _key = leptos_i18n::Locale::get_keys(locale)
            .$key()
            .var_$variable($variable);
        #[deny(deprecated)]
        _key.build()
    }
}
```

Code:

```rust
td!(locale, $key, $variable)
```

Expanded code:

```rust
{
    let ($variable,) = ($variable,);
    move || {
        let _key = leptos_i18n::Locale::get_keys(locale)
            .$key()
            .var_$variable($variable);
        #[deny(deprecated)]
        _key.build()
    }
}
```

Code:

```rust
td!(locale, $key, <$component> = $component_expr)
```

Expanded code:

```rust
{
    let ($component,) = ($component_expr,);
    move || {
        let _key = leptos_i18n::Locale::get_keys(locale)
            .$key()
            .comp_$component($component);
        #[deny(deprecated)]
        _key.build()
    }
}
```

```rust
td!(locale, $key, <$component>)
```

Expanded code:

```rust
{
    let ($component,) = ($component,);
    move || {
        let _key = leptos_i18n::Locale::get_keys(locale)
            .$key()
            .comp_$component($component);
        #[deny(deprecated)]
        _key.build()
    }
}
```

Code:

```rust
td!(locale, $key, $variable = $variable_expr, <$component> = $component_expr)
```

Expanded code:

```rust
{
    // as you can see here, if multiple expr are passed they can all execute before the new variables goes into scope, avoiding name collisions.
    let ($variable, $component,) = ($variable_expr, $component_expr,);
    move || {
        let _key = leptos_i18n::Locale::get_keys(locale)
            .$key()
            .var_$variable($variable)
            .comp_$component($component);
        #[deny(deprecated)]
        _key.build()
    }
}
```

Code:

```rust
td!(locale, $key, <$component> = <$component_name $($attrs:tt)* />)
```

Expanded code:

```rust
{
    let ($component,) = (move |__children: leptos::ChildrenFn| { leptos::view! { <$component_name $($attrs)* >{move || __children()}</$component_name> } },);
    move || {
        let _key = leptos_i18n::Locale::get_keys(locale)
            .$key()
            .comp_$component($component);
        #[deny(deprecated)]
        _key.build()
    }
}
```
