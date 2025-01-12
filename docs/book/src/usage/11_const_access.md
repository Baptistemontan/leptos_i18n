# Access translations in a const context

You can access the translations in a const context if you have those things:

- Constant Locale
- No arguments
- No using the "dynamic_load" feature

If you have

```json
{
  "subkeys:": {
    "key": "my value"
  }
}
```

You can do

```rust,ignore
use crate::i18n::*;
const MY_VALUE: &str = Locale::en.get_keys_const().subkeys().key().inner();
```

If you want a macro:

```rust,ignore
macro_rules! td_const {
    ($locale:expr, $first_key:ident $(.$key:ident)*) => {
        ($locale).get_keys_const()
            .$first_key()
            $(.$key())*
            .inner()
    };
}

const MY_VALUE: &str = td_const(Locale::en, subkeys.key);


```
