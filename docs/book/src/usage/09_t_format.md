# `t_format!`

You may want to use the formatting capability without the need to create an entry in you translations, you can use the `t_format!` macro for that:

```rust
use crate::i18n::*;
use leptos_i18n::formatting::t_format;

let i18n = use_i18n();

let num = move || 100_000;

t_format!(i18n, num, formatter: number);
```

There are 9 variants, just like the `t!` macro, `td_format!`, `tu_format!`, `*_format_string` and `*_format_display`.

### Example

```rust
let date = move || Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();

let en = td_format_string!(Locale::en, date, formatter: date);
assert_eq!(en, "Jan 2, 1970");
let fr = td_format_string!(Locale::fr, date, formatter: date(date_length: full));
assert_eq!(fr, "vendredi 2 janvier 1970");
```
