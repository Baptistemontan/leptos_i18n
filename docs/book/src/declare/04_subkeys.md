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
