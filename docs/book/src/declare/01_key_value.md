# Key-Value Pairs

As expected, translations are declared as key-value pairs:

```json
{
  "hello_world": "Hello World!"
}
```

However, there are additional rules you must follow beyond those of the format you use.

## Keys

Key names must be [valid Rust identifiers](https://doc.rust-lang.org/reference/identifiers.html), with the exception that `-` will be converted to `_`, and do not support [strict](https://doc.rust-lang.org/reference/keywords.html#strict-keywords) or [reserved](https://doc.rust-lang.org/reference/keywords.html#reserved-keywords) keywords.

## Same Keys Across Files

The keys must be the same across all files; otherwise, the codegen will emit warnings. Any difference in keys is based on the default locale.

### Missing key

If a key is present in the default locale but not in another locale, the other locale will use the value from the default locale and emit a warning that a key is missing in that locale.

If you want to explicitly indicate that this value should use the value from the default locale, you can declare it as `null`:

```json
{
  "take_value_of_default": null
}
```

This will prevent a warning from being triggered for that key.

### Surplus key

If a key is present in another locale but not in the default locale, the key will be ignored and a warning will be emitted.

## Value Kinds

You can specify several kinds of values:

- Literals (String, Numbers, Boolean)
- Interpolated String
- Plurals

The next chapters of this section will cover them (apart from literals, which are self-explanatory).
