# Key-Value Pairs

As expected, you declare your translations as key-value pairs:

```json
{
  "hello_world": "Hello World!"
}
```

But there are additional rules you must follow in addition to those of the format you use.

## Keys

Key names must be [valid Rust identifiers](https://doc.rust-lang.org/reference/identifiers.html), with the exception of `-` that would be converted to `_`, and does not support [strict](https://doc.rust-lang.org/reference/keywords.html#strict-keywords) or [reserved](https://doc.rust-lang.org/reference/keywords.html#reserved-keywords) keywords.

## Same keys across files

The keys must be the same across all files, else the codegen will emit warnings. The difference in keys is based on the default locale.

### Missing key

If a key is present in the default locale but not in another locale, the other locale will default its value to the default locale one and emit a warning that a key is missing in that locale.

If you want to explicitly state that this value takes the value of the default locale, you can declare it as `null`:

```json
{
  "take_value_of_default": null
}
```

This will no longer trigger a warning for that key.

### Surplus key

If a key is present in another locale but not in the default locale, this key will be ignored and a warning will be emitted.

## Value Kinds

You can specify multiple kinds of values:

- Literals (String, Numbers, Boolean)
- Interpolated String
- Plurals

The next chapters of this section will cover them, apart from literals, those are self-explanatory.
