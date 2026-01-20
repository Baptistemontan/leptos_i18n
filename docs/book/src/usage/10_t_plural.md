# `t_plural!`

You can use the `t_plural!` macro to match on the plural form of a given count:

```rust,ignore
let i18n = use_i18n();

let form = t_plural! {
    i18n,
    count = || 0,
    one => "one",
    _ => "other"
};

Effect::new(|| {
    let s = form();
    log!("{}", s);
})
```

This will print "one" with the "fr" locale but "other" with the "en" locale.
Accepted forms are: `zero`, `one`, `two`, `few`, `many`, `other`, and `_`.

This macro is for cardinal plurals; if you want to match against ordinal plurals, use the `t_plural_ordinal!` macro.
