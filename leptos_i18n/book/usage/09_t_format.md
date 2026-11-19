# `t_format!`

You may want to use the formatting capability without needing to create an entry in your translations. 
You can use the `t_format!` macro for that:

```rust,ignore
use crate::i18n::*;
use leptos_i18n::formatting::t_format;

let i18n = use_i18n();

let num = move || 100_000;

t_format!(i18n, num, formatter: number);
```

There are 9 variants, just like the `t!` macro, `td_format!`, `tu_format!`, `*_format_string`, and `*_format_display`.

### Example

```rust,ignore
let date = move || Date::try_new_iso(1970, 1, 2).unwrap().to_any();

let en = td_format_string!(Locale::en, date, formatter: date);
assert_eq!(en, "Jan 2, 1970");
let fr = td_format_string!(Locale::fr, date, formatter: date(date_length: long));
assert_eq!(fr, "2 janvier 1970");
```
