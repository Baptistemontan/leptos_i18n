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

To point to subkeys you give the path by separating the the key by `.`: `$t(key.subkey.subsubkey)`.

When using namespaces you _must_ specify the namespace of the key you are looking for, using `:`: `$t(namespace:key)`.

You can point to explicitly defaulted keys, but not implicitly defaulted ones.

## Supply arguments

You can also supply arguments to fill variables of the pointed key:

```json
{
  "click_count": "You clicked {{ count }} times",
  "clicked_twice": "$t(click_count, {'count': 'two'})"
}
```

This will result to `clicked_twice` to have the value `"You clicked two times"`.

Arguments must be string, delimited by either single quotes or double quotes.

**Note**: Any argument with no matching variable are just discarded, they will not emit any warning/error.
