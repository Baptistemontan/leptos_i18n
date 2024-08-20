# Subkeys

You can declare subkeys by just giving a map to the key:

```json
{
  "subkeys": {
    "subkey_1": "This is subkey_1",
    "subkey_n": "This is subkey <b>{{ n }}</b>",
    "nested_subkeys": {
      "nested_subkey_1": "you can nest subkeys"
    }
  }
}
```

```rust
t!(i18n, subkeys.subkey_1); // -> "This is subkey_1"
t!(i18n, subkeys.nested_subkeys.nested_subkey_1) // -> "you can nest subkeys"
```
