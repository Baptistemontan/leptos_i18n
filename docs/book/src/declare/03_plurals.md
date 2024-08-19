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

t!(i18n, items, count = || 0) // -> "0 items"
t!(i18n, items, count = || 1) // -> "1 item"
t!(i18n, items, count = || 4) // -> "4 items"
```

`{{ count }}` is a special variable when using plurals, you don't supply it as `t!(i18n, key, count = ..)` but with the `$`.

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

## Ordinal plurals

What I describe above are "Cardinal" plurals, but they don't work with like "1rst place", "2nd place", ect..

The English language use 4 ordinal plurals, and French 2:

- one: "1rst place", "21rst place"
- two: "2nd place", "22nd place"
- few: "3rd place", "33rd place"
- other: "4th place", "5th place", "7th place"

And French:

- one: "1ère place"
- other: "2ème place", "21ème place"

You can use them by using the `_ordinal` suffix:

```json
{
  "key_ordinal_one": "{{ count }}rst place",
  "key_ordinal_two": "{{ count }}nd place",
  "key_ordinal_few": "{{ count }}rd place",
  "key_ordinal_other": "{{ count }}th place"
}
```

> the `_ordinal` suffix is removed, in this example you access it with `t!(i18n, key, count = ..)`

## How to know which to use:

There are ressources online to help you find what you should use, my personnal favorite is the [unicode CLDR Charts](https://www.unicode.org/cldr/charts/44/supplemental/language_plural_rules.html).
