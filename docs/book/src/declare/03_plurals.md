# Plurals

## What are plurals ?

Plurals are a standardized way to deal with numbers. For example, the English language deals with 2 plurals: _"one"_ (1) and _"other"_ (0, 2, 3, ..).

If you were to have

```json
{
  "items": "{{ count }} items"
}
```

this would produce "1 items", which is not good English.

This can be solved by defining 2 plural forms:

```json
{
  "items_one": "{{ count }} item",
  "items_other": "{{ count }} items"
}
```

Providing the count to the `t!` macro, this will result in:

```rust,ignore
let i18n = use_i18n();

t!(i18n, items, count = || 0) // -> "0 items"
t!(i18n, items, count = || 1) // -> "1 item"
t!(i18n, items, count = || 4) // -> "4 items"
```

> All `items_*` are merged into the single key `items`.

`{{ count }}` is a special variable when using plurals. Even if you don't interpolate it, you must supply it:

```json
{
  "items_one": "one item",
  "items_other": "some items"
}
```

This will still need you to supply the `count` variable: `t!(i18n, items, count = ...)`.

## Why bother ?

Why bother and not just do

```rust,ignore
if item_count == 1 {
    t!(i18n, items_one)
} else {
    t!(i18n, items_other, count = move || item_count)
}
```

Because all languages don't use the same plurals!

For example, in French, 0 is considered singular, so this could produce "0 choses" instead of "0 chose", which is bad French (except in certain conditions, because French, exceptions are everywhere).

## Ordinal plurals

What I describe above are "Cardinal" plurals, but they don't work with like "1st place", "2nd place", etc.

The English language uses 4 ordinal plurals, and French 2:

- one: "1st place", "21st place"
- two: "2nd place", "22nd place"
- few: "3rd place", "33rd place"
- other: "4th place", "5th place", "7th place"

And French:

- one: "1ère place"
- other: "2ème place", "21ème place"

You can use them by using the `_ordinal` suffix:

```json
{
  "key_ordinal_one": "{{ count }}st place",
  "key_ordinal_two": "{{ count }}nd place",
  "key_ordinal_few": "{{ count }}rd place",
  "key_ordinal_other": "{{ count }}th place"
}
```

> The `_ordinal` suffix is removed, in this example you access it with `t!(i18n, key, count = ..)`

## How to know which to use:

There are resources online to help you find what you should use, my personal favorite is the [Unicode CLDR Charts](https://www.unicode.org/cldr/charts/44/supplemental/language_plural_rules.html).

## What if I need multiple counts ?

If you need multiple counts, for example:

```json
{
  "key": "{{ boys_count }} boys and {{ girls_count }} girls"
}
```

There isn't a way to represent this in a single key, you will need `Foreign keys` that you can read about in a next chapter.

## Activate the feature

To use plurals in your translations, enable the "plurals" feature.
