# Foreign keys

Foreign keys let you re-use already declared translations, you declare them like variables but with a '@' before the path:

```json
{
  "hello_world": "Hello World!",
  "reuse": "message: {{ @hello_world }}"
}
```

This will replace `{{ @hello_world }}` by the value of the key `hello_world`, making `reuse` equal to `"message: Hello World!"`.

You can point to any key other than plurals and keys containing subkeys.

To point to subkeys you give the path by separating the the key by `.`: `{{ @key.subkey.subsubkey }}`.

When using namespaces you _must_ specify the namespace of the key you are looking for, using `::`: `{{ @namespace::key }}`.

You can point to explicitly defaulted keys, but not implicitly defaulted ones.
