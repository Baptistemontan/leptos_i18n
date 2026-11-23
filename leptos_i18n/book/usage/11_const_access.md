# Access Translations in a Const Context

You can access the translations in a const context with the `td_const!` macro, if you have those things:

- Constant Locale
- No arguments
- No using the "dynamic_load" feature

If you have

```json
{
  "subkeys": {
    "key": "my value"
  }
}
```

You can do

```rust,ignore
use crate::i18n::*;
use leptos_i18n::td_const;

const MY_VALUE: &str = td_const!(Locale::en, subkeys.key);
```

It returns the value itself: a `&'static str` for a string, and a `bool`, `i64`, `u64` or `f64` for the other literals. If the type of the value is not the same in every locale, it returns a `leptos_i18n::keys::comp_time::Literal`, an enum of those types.

It also accepts a key built with `build_key!` (see the "Keys As Values" chapter):

```rust,ignore
use leptos_i18n::{build_key, key_builder, td_const};

type SubkeysScope = define_scope!(Locale, subkeys);

const MY_VALUE: &str = td_const!(
    Locale::en,
    key = build_key!(key_builder!(scope = SubkeysScope, key))
);
```
