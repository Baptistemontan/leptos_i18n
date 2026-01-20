# Server functions

There is no context in server functions, so you can't call `use_i18n`. You could provide a context if you want,
and it would work as expected, but if you just want to access the user's locale, you can use the `resolve_locale` function:

```rust
#[server]
async fn get_locale() -> Result<Locale, ServerFnError> {
    let locale: Locale = leptos_i18n::locale::resolve_locale();
    Ok(locale)
}
```
