# Foreign keys

Foreign keys let you re-use already declared translations:

```json
{
  "hello_world": "Hello World!",
  "reuse": "message: $t(hello_world)"
}
```

This will replace `$t(hello_world)` by the value of the key `hello_world`, making `reuse` equal to `"message: Hello World!"`.

You can point to any key other than ranges and keys containing subkeys.

To point to subkeys, you give the path by separating the key by `.`: `$t(key.subkey.subsubkey)`.

When using namespaces, you _must_ specify the namespace of the key you are looking for, using `:`: `$t(namespace:key)`.

You can point to explicitly defaulted keys, but not implicitly defaulted ones.

## Supply arguments

You can also supply arguments to fill variables of the pointed key:

```json
{
  "click_count": "You clicked {{ count }} times",
  "clicked_twice": "$t(click_count, {\"count\": \"two\"})"
}
```

This will result in `clicked_twice` to have the value `"You clicked two times"`.

Arguments must be strings, delimited by either single quotes or double quotes.

> **Note**: Any argument with no matching variable is just discarded; they will not emit any warning/error.

Arguments can be anything that could be parsed as a normal key-value:

```json
{
  "key": "{{ arg }}",
  "string_arg": "$t(key, {\"arg\": \"str\"})",
  "boolean_arg": "$t(key, {\"arg\": true})",
  "number_arg": "$t(key, {\"arg\": 56})",
  "interpolated_arg": "$t(key, {\"arg\": \"value: {{ new_arg }}\"})",
  "foreign_key_arg": "$t(key, {\"arg\": \"value: $t(interpolated_arg)\"})"
}
```

```rust
t!(i18n, string_arg); // -> "str"
t!(i18n, boolean_arg); // -> "true"
t!(i18n, number_arg); // -> "56"
t!(i18n, interpolated_arg, new_arg = "a value"); // -> "value: a value"
t!(i18n, foreign_key_arg, new_arg = "a value"); // -> "value: value: a value"
```

## `"count"` arg for plurals/ranges

If you have a plural like

```json
{
  "key_one": "one item",
  "key_other": "{{ count }} items"
}
```

You can supply the count as a foreign key in 2 ways, as a variable:

```json
{
  "new_key": "$t(key, {\"count\": \"{{ new_count }}\"})"
}
```

This will just rename the key.

```rust
t!(i18n, new_key, new_count = move || 1); // -> "one item"
t!(i18n, new_key, new_count = move || 2); // -> "2 items"
```

> **note**: for the `count` arg to plurals/ranges, the value provided must be a single variable (whitespaces around are supported though).

Or by an actual value:

```json
{
  "singular_key": "$t(key, {\"count\": 1})",
  "multiple_key": "$t(key, {\"count\": 6})"
}
```

```rust
t!(i18n, singular_key); // -> "one item"
t!(i18n, multiple_key); // -> "6 items"
```

> **note**: while floats are supported, they don't carry all the information once deserialized such as leading 0, so some truncation may occur.

## Multiple counts ranges or plurals

If you need multiple counts for a plural or a range, for example:

```json
{
  "key": "{{ boys_count }} boys and {{ girls_count }} girls"
}
```

You can use `Foreign keys` to construct a single key from multiple plurals/ranges by overriding their `"count"` variable:

```json
{
  "key": "$t(key_boys, {\"count\": \"{{ boys_count }}\"}) and $t(key_girls, {\"count\": \"{{ girls_count }}\"})",
  "key_boys_one": "{{ count }} boy",
  "key_boys_other": "{{ count }} boys",
  "key_girls_one": "{{ count }} girl",
  "key_girls_other": "{{ count }} girls"
}
```

```rust
t!(i18n, key, boys_count = move || 0, girls_count = move || 0); // -> "0 boys and 0 girls"
t!(i18n, key, boys_count = move || 0, girls_count = move || 1); // -> "0 boys and 1 girl"
t!(i18n, key, boys_count = move || 1, girls_count = move || 0); // -> "1 boy and 0 girls"
t!(i18n, key, boys_count = move || 56, girls_count = move || 39); // -> "56 boys and 39 girls"
```
