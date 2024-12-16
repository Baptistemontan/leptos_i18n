# Access translations in a const context

You can access the translations in a const context if you have those two things:

- Constant Locale
- No arguments

If you have

```json
{
  "subkeys:": {
    "key": "my value"
  }
}
```

You can do

```rust
use crate::i18n::*;
const MY_VALUE: &str = Locale::en.get_keys_const().subkeys().key().inner();
```

If you want a macro:

```rust
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
