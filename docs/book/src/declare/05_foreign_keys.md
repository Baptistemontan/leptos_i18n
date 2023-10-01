# Foreign keys

Foreign keys let you re-use already declared translations, you declare them like variables but with a '@' before the path:

```json
{
  "hello_world": "Hello World!",
  "reuse": "message: {{ @hello_world }}"
}
```

This will replace `{{ @hello_world }}` by the value of the key `hello_world`, making `reuse` equal to `"message: Hello World!"`.

You can point to any key other than plurals and key containing subkeys.

To point to subkeys you give the path by separating the the key by `.`: `{{ @key.subkey.subsubkey }}`.

When using namespaces you _must_ specify the namespace of the key you are looking for, using `::`: `{{ @namespace::key }}`.

You can point to explicitly defaulted keys, but not implictly defaulted ones.

## Supply arguments

You can also supply arguments to fill variables of the pointed key:

```json
{
  "click_count": "You clicked {{ count }} times",
  "clicked_twice": "{{ @click_count, count = 'two' }}"
}
```

This will result to `clicked_twice` to have the value `"You clicked two times"`.

Arguments must be string, delimited by either single quotes or double quotes.

**Note**: Any argument with no matching variable are just discarded, they will not emit any warning/error.
