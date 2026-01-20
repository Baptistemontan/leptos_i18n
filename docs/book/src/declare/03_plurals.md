# Plurals

## What Are Plurals?

Plurals are a standardized way to deal with quantities. For example, English uses with 2 plurals: _"one"_ (1) and _"other"_ (0, 2, 3, ..).

If you have

```json
{
  "items": "{{ count }} items"
}
```

this would produce "1 items", which is incorrect English.

This can be solved by defining 2 plural forms:

```json
{
  "items_one": "{{ count }} item",
  "items_other": "{{ count }} items"
}
```

When providing the count to the `t!` macro, this will result in:

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

This will still require you to supply the `count` variable: `t!(i18n, items, count = ...)`.

## Why Bother?

Why bother, instead of just doing:

```rust,ignore
if item_count == 1 {
    t!(i18n, items_one)
} else {
    t!(i18n, items_other, count = move || item_count)
}
```

Because not all languages use the same plural rules.

For example, in French, 0 is considered singular, so this could produce "0 choses" instead of "0 chose", which is incorrect in French (with some exceptions — French has many of them).

## Ordinal Plurals

What I described above are "cardinal" plurals, but they don’t work for cases like "1st place", "2nd place", etc.

The English language uses 4 ordinal plural forms, while French uses 2:

- one: "1st place", "21st place"
- two: "2nd place", "22nd place"
- few: "3rd place", "33rd place"
- other: "4th place", "5th place", "7th place"

And French:

- one: "1ère place"
- other: "2ème place", "21ème place"

You can use ordinal plurals by using the `_ordinal` suffix:

```json
{
  "key_ordinal_one": "{{ count }}st place",
  "key_ordinal_two": "{{ count }}nd place",
  "key_ordinal_few": "{{ count }}rd place",
  "key_ordinal_other": "{{ count }}th place"
}
```

> The `_ordinal` suffix is removed, in this example you access it with `t!(i18n, key, count = ..)`

## How to Know Which to Use

There are online resources that help determine which plural rules to use, my personal favorite is the [Unicode CLDR Charts](https://www.unicode.org/cldr/charts/44/supplemental/language_plural_rules.html).

## What if I Need Multiple Counts?

If you need multiple counts, for example:

```json
{
  "key": "{{ boys_count }} boys and {{ girls_count }} girls"
}
```

There isn't a way to represent this in a single key, you will need `Foreign keys`, which you can read about in a later chapter.

## Activate the Feature

To use plurals in your translations, enable the "plurals" feature.
