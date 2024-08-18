# Plurals

## What are plurals ?

Plurals are a standardized way to deal with numbers, for example the English language deal with 2 plurals: _"one"_ (1) and _"other"_ (0, 2, 3, ..).

If you were to have

```json
{
  "items": "{{ count }} items"
}
```

this would produce "1 items", which is not good english.

This can be solved by defining 2 plural forms:

```json
{
  "items_one": "{{ count }} item",
  "items_other": "{{ count }} items"
}
```

Providing the count to the `t!` macro with the `$`, this will result in:

```rust
let i18n = use_i18n();

t!(i18n, items, $ = || 0) // -> "0 items"
t!(i18n, items, $ = || 1) // -> "1 item"
t!(i18n, items, $ = || 4) // -> "4 items"
```

## Why bother ?

Why bother and not just do

```rust
if item_count == 1 {
    t!(i18n, items_one)
} else {
    t!(i18n, items_other, count = move || item_count)
}
```

Because all languages don't use the same plurals!

For example in French, 0 is considered singular, so this could produce "0 choses" instead of "0 chose", which is bad french (except in certain conditions, because french, exceptions are everywhere).
