# Ranges

We just talked about plurals, which are standardized, but we have a little unorthodox features that I called ranges.

They are based around a count and display different translations based on this count.

To declare them the key takes a sequence where each element is a sequence with the first element being the value, and the other element the count to match against:

```json
{
  "click_count": [
    ["You have not clicked yet", 0],
    ["You clicked once", 1],
    ["You clicked {{ count }} times", "_"]
  ]
}
```

## Multiple exact values

You can declare multiple counts to match against:

```json
{
  "click_count": [
    ["0 or 5", 0, 5],
    ["1, 2, 3 or 4", 1, 2, 3, 4],
    ["You clicked {{ count }} times", "_"]
  ]
}
```

## Ranges

You can also declare a range where the translations is used:

```json
{
  "click_count": [
    ["0 or 5", 0, 5],
    ["1, 2, 3 or 4", "1..=4"],
    ["You clicked {{ count }} times", "_"]
  ]
}
```

You can use all Rust ranges syntax: `s..e`, `..e`, `s..`, `s..=e`, `..=e` or even `..` ( `..` will be considered fallback `_`)

## Number type

By default the count is expected to be an `i32`, but you can change that by specifying the type as the first element of the sequence:

```json
{
  "click_count": [
    "u32",
    ["You have not clicked yet", 0],
    ["You clicked once", 1],
    ["You clicked {{ count }} times", "_"]
  ]
}
```

Now you only have to cover the `u32` range.

The supported types are `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32` and `f64`.

## Fallback

If all the given counts don't fill the range of the number type, you can use a fallback (`"_"` or `".."`) as seen above, but it can be completely omitted on the last element of the sequence:

```json
{
  "click_count": [
    ["You have not clicked yet", 0],
    ["You clicked once", 1],
    ["You clicked {{ count }} times"]
  ]
}
```

Fallbacks are not required if you already cover the full number range:

```json
{
  "click_count": [
    "u8",
    ["You have not clicked yet", 0],
    ["1 to 254", "1..=254"],
    ["255", 255]
  ]
}
```

Fallbacks are always required for `f32` and `f64`.

## Order

The order of the ranges matter, for example:

```json
{
  "click_count": [
    ["first", "0..5"],
    ["second", "0..=5"],
    ["You clicked {{ count }} times"]
  ]
}
```

Here "second" will only be printed if count is 5, if `0 <= count < 5` then "first" will be printed.

## Mix ranges with exact values

You can totally mix them, this is valid:

```json
{
  "click_count": [
    ["first", 0, "3..5", "10..=56"],
    ["second", "0..3", "..78"],
    ["You clicked {{ count }} times"]
  ]
}
```

## Use interpolation

The "You clicked {{ count }} times" kind of gave it away, but you can use interpolation in your ranges, this is valid:

```json
{
  "click_count": [
    ["<b>first</b>", 0, "3..5", "10..=56"],
    ["<i>second</i>", "0..3", "..78"],
    ["You clicked {{ count }} times and have {{ banana_count }} bananas"]
  ]
}
```

With ranges, `{{ count }}` is a special variable that refers to the count provided to the range, so you don't need to also provide it:

```json
{
  "click_count": [
    ["You have not clicked yet", 0],
    ["You clicked once", 1],
    ["You clicked {{ count }} times"]
  ]
}
```

```rust
t!(i18n, click_count, count = || 0);
```

Will result in `"You have not clicked yet"` and

```rust
t!(i18n, click_count, count = || 5);
```

Will result in `"You clicked 5 times"`.

Providing `count` will create an error:

```rust
t!(i18n, click_count, count = 12, count = || 5); // compilation error
```
